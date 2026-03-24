# Comprehensive Analysis: `Lazy<'a, A, Config>`

**File:** `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/lazy.rs`
**PureScript reference:** `/home/jessea/Documents/projects/purescript-lazy/src/Data/Lazy.purs`

---

## 1. Design

### Type Structure

`Lazy<'a, A, Config>` is a memoized lazy value parameterized over:
- `'a` -- lifetime of the computation closure.
- `A` -- the computed value type.
- `Config: LazyConfig` -- a strategy object selecting the pointer/cell types (defaults to `RcLazyConfig`).

The actual storage is `Config::Lazy<'a, A>`, which resolves to:
- **RcLazyConfig:** `Rc<LazyCell<A, Box<dyn FnOnce() -> A + 'a>>>`
- **ArcLazyConfig:** `Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send + 'a>>>`

### How Memoization Works

The design delegates entirely to Rust's standard library lazy cells:
- `LazyCell` (single-threaded, interior mutability via `UnsafeCell`).
- `LazyLock` (thread-safe, uses `Once` / mutex internally).

Wrapping these in `Rc`/`Arc` gives shared ownership, meaning `Clone` on `Lazy` creates a new handle to the same cache. This is the critical design choice: cloning is O(1) and all clones observe the same cached value.

### Comparison to PureScript

PureScript's `Lazy` is a foreign (runtime-provided) type with `defer` and `force` as the core API. It implements the full type class hierarchy: `Functor`, `Apply`, `Applicative`, `Bind`, `Monad`, `Extend`, `Comonad`, `Foldable`, `Traversable`, `Semigroup`, `Monoid`, `Eq`, `Ord`, `Show`, and the `Lazy` type class from `Control.Lazy`.

The Rust implementation covers a subset:
- `RefFunctor` / `SendRefFunctor` (instead of `Functor`)
- `Deferrable` / `SendDeferrable` (analogous to `Control.Lazy`)
- `Foldable`
- `Semigroup`, `Monoid`
- `PartialEq`, `PartialOrd`, `Debug`
- `rc_lazy_fix` / `arc_lazy_fix` (analogous to `fix` from `Control.Lazy`)

**Missing from PureScript:** `Functor`, `Apply`, `Applicative`, `Bind`, `Monad`, `Extend`, `Comonad`, `Traversable`, `Eq` (only `PartialEq`), `Ord` (only `PartialOrd`), `Show` (only `Debug`).

The PureScript version is simpler because it operates in a garbage-collected runtime where `force` returns `a` (owned), not `&a` (reference). This difference is the root cause of most limitations in the Rust port.

---

## 2. Implementation Correctness

### No Bugs Found

The implementation is mechanically correct. The memoization logic is sound because it delegates to well-tested std library types (`LazyCell`, `LazyLock`). The `Rc`/`Arc` wrappers correctly provide shared ownership.

### Subtle Points Worth Noting

**`rc_lazy_fix` / `arc_lazy_fix` safety:** These functions use `OnceCell`/`OnceLock` with an `unwrap()` inside a closure. The comment claims the cell is always populated before the closure can execute. This is true: the cell is set immediately after the `RcLazy`/`ArcLazy` is constructed but before it is returned to the caller. However, this relies on the caller never evaluating the lazy value during `f`'s execution in a way that triggers infinite recursion through the fix point. The `unwrap()` is safe as long as the cell is populated, and it always is. The `#[allow(clippy::unwrap_used)]` is appropriate.

**Panic poisoning:** When the initializer panics, `LazyCell` is poisoned and subsequent calls to `force` re-panic. `LazyLock` similarly panics on poisoning. This is documented in the `Lazy` struct doc comment and tested. The documentation correctly suggests `TryLazy` with `catch_unwind` as a workaround.

**`Deferrable::defer` for `RcLazy`:** The implementation `RcLazy::new(move || f().evaluate().clone())` correctly flattens nested lazy values. However, it requires `A: Clone`. This is a necessary cost because `evaluate()` returns `&A`, and we need an owned value for the new lazy cell.

**Thread safety:** `ArcLazy` requires `Send` on the closure and `Send + Sync` on the value type in the right places. The `ArcLazyConfig::Thunk` type is `dyn FnOnce() -> A + Send + 'a`, which correctly adds `Send`. The `ArcLazy::ref_map` method requires `A: Send + Sync` and `f: Send`, which is correct because the closure captures `self` (an `ArcLazy`) and will be evaluated potentially on another thread.

### Potential Issue: `pure` is Not Actually Lazy

`RcLazy::pure(a)` wraps the value in `Box::new(move || a)`, which is then passed to `LazyCell::new`. The value `a` is moved into the closure, and the closure is invoked on first `evaluate()`. This means the value `a` is created eagerly (it must already exist), and only the "extraction from the closure" is deferred. This is consistent with PureScript's `pure a = defer \_ -> a`, which similarly captures `a` eagerly. This is semantically correct for `pure`/`Applicative` but worth noting: no computation is actually deferred.

---

## 3. Consistency with Library Patterns

### Follows Library Conventions

- Uses `impl_kind!` macro for HKT registration.
- Documentation follows the `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]` pattern consistently.
- Uses `#[fp_macros::document_module]` with the inner module pattern.
- Brand type `LazyBrand<Config>` is defined in `brands.rs` as expected.
- Free functions (`rc_lazy_fix`, `arc_lazy_fix`) are defined in the type module rather than in `functions.rs`, which is different from other free functions. However, these are not dispatching through a type class, so this placement is reasonable.

### Duplication Concern

The `Foldable`, `Semigroup`, `Monoid` implementations are duplicated nearly verbatim for `RcLazyConfig` and `ArcLazyConfig`. The only difference is the `Send + Sync` bounds on the Arc variants. This is approximately 250 lines of duplicated code. Rust's trait system makes it difficult to unify these without a helper trait or macro, but a `macro_rules!` helper could reduce the duplication.

### `pub(crate)` on the inner field

`Lazy`'s inner field is `pub(crate)`, which means other modules within the crate can access the raw `Config::Lazy<'a, A>` directly. This is a reasonable escape hatch for internal code (e.g., `TryLazy` might need it), though it breaks encapsulation. If no other module accesses it, it should be private.

---

## 4. Limitations

### Why Lazy Cannot Implement Full Functor/Monad

The core tension is between memoization and value-level transformation:

**Functor requires `map(f: impl Fn(A) -> B, fa: F<A>) -> F<B>`**, where the function receives an owned `A`. But `Lazy::evaluate()` returns `&A`, not `A`. To implement standard `Functor`, you would need either:
1. `A: Clone`, so you can clone the reference before applying `f`. But `Functor::map` has no `Clone` bound on `A`.
2. Move the value out of the cache, which destroys memoization and is unsound when other clones exist.

**This is why `RefFunctor` exists.** It changes the contract: `ref_map(f: impl FnOnce(&A) -> B, fa: F<A>) -> F<B>`. The function receives `&A` instead of `A`, which is compatible with `evaluate()` returning a reference.

**Monad/Applicative** have the same problem compounded. `bind(fa: F<A>, f: impl Fn(A) -> F<B>) -> F<B>` requires owned `A`. Additionally, `bind`/`join` for Lazy would need to flatten `Lazy<Lazy<A>>` into `Lazy<A>`, which requires cloning the inner value.

**Comonad** (`extract: F<A> -> A`) would require moving the value out of the cache, which is impossible without `Clone`.

### Workarounds

1. **`RefFunctor`** -- the mapping function receives `&A` and produces an owned `B`, which is wrapped in a new `Lazy<B>`. This is the primary workaround, already implemented.
2. **Explicit `.evaluate().clone()` chains** -- users can manually force and clone, then wrap in a new `Lazy`.
3. **`Deferrable::defer`** -- enables flattening `Lazy<Lazy<A>>` when `A: Clone`.
4. **`Foldable`** -- already requires `A: Clone` in its bound, so it works.

### Missing `Eq` and `Ord`

Only `PartialEq` and `PartialOrd` are implemented. `Eq` and `Ord` could be added with the appropriate bounds (`A: Eq`, `A: Ord`), analogous to PureScript. These are straightforward to add.

### Missing `Display`/`Show`

The `Debug` impl outputs `"Lazy(..)"` without evaluating, which is intentional (evaluating during debug formatting would be a side effect). PureScript's `Show` does force evaluation: `show x = "(defer \\_ -> " <> show (force x) <> ")"`. A `Display` impl that forces evaluation could be provided as an opt-in.

### No `From<Thunk>` or `From<Trampoline>` for ArcLazy

`From<Thunk>` and `From<Trampoline>` are only implemented for `RcLazy`. The `ArcLazy` variants would need `Send` bounds on the thunk/trampoline values. This is a minor gap.

---

## 5. Alternative Designs

### Alternative A: Store `A` Directly Instead of Through a Closure

Instead of `Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>`, one could use `Rc<OnceCell<A>>` with a separate closure stored in an `Option`. However, this is essentially what `LazyCell` already does internally, so wrapping `LazyCell` is the right call. The standard library implementation is well-optimized.

### Alternative B: Enum-Based Lazy (No LazyCell)

```rust
enum LazyInner<A> {
    Deferred(Box<dyn FnOnce() -> A>),
    Evaluated(A),
}
```

Wrapped in `Rc<RefCell<LazyInner<A>>>`. This was the traditional approach before `LazyCell` was stabilized. The current design using `LazyCell`/`LazyLock` is superior because:
- No `RefCell` overhead (no runtime borrow checks for `RcLazy`).
- `LazyLock` provides better concurrency semantics than `Arc<Mutex<LazyInner>>`.
- Less code to maintain.

### Alternative C: `Lazy<A>` That Owns the Value (No Reference Semantics)

If `evaluate()` returned `A` instead of `&A` (consuming the lazy value), standard `Functor` could be implemented. But this would eliminate memoization and shared access, making it functionally identical to `Thunk`. The reference-returning design is the correct choice for a memoized type.

### Alternative D: `Lazy<A: Clone>` With Clone Bound on the Type

If the `Lazy` struct itself required `A: Clone`, then `Functor` could be implemented by cloning the value out. However, this would be overly restrictive, preventing use with non-cloneable types. The current design correctly separates the concern: `Lazy` itself does not require `Clone`, but operations that need to produce owned values (like `Semigroup`, `Foldable`, `Deferrable`) add the `Clone` bound on their impls.

### Alternative E: Unify Rc/Arc Via a Generic `LazyConfig` Blanket Impl

The duplicated impls for `RcLazyConfig` and `ArcLazyConfig` could potentially be unified if `LazyConfig` exposed enough operations. For example, if `LazyConfig` had a `new` and `evaluate` method on the `Lazy` type itself, blanket impls could work. The current design already has `lazy_new` and `evaluate` on `LazyConfig`, but the `Send` bounds on Arc variants prevent a single blanket impl from working for both. A `macro_rules!` approach to generate the duplicated impls would be more practical.

---

## 6. Documentation

### Strengths

- The module-level doc comment is concise and accurate.
- The `LazyConfig` trait has thorough documentation explaining extensibility.
- The panic poisoning behavior is documented prominently on the `Lazy` struct.
- The `Deferrable` trait documentation explains why there is no generic `fix` in Rust.
- All methods have `#[document_examples]` with working code examples.
- The test suite is comprehensive, covering memoization, sharing, thread safety, conversions, and law-based property tests.

### Gaps

1. **No module-level explanation of why `Functor` is not implemented.** Users discovering `Lazy` will likely wonder why they cannot `map` over it. The module docs or the `Lazy` struct docs should include a brief explanation pointing to `RefFunctor` and explaining the `&A` vs `A` issue.

2. **`LazyConfig` extensibility claim is untested.** The docs say "This trait is open for third-party implementations" and suggest `parking_lot`-based locks or async-aware cells. However, no third-party config exists in the codebase or tests. The claim is plausible but unverified.

3. **`rc_lazy_fix` / `arc_lazy_fix` lack documentation about recursion limits.** If the function `f` forces the self-reference, it leads to infinite recursion (stack overflow). The docs should warn about this.

4. **The `'a` lifetime parameter is documented as "The lifetime of the reference" on `Lazy`, but "The lifetime of the computation" elsewhere.** The `Lazy` struct docs say "The lifetime of the reference," while `Deferrable` and the fix functions say "The lifetime of the computation." "The lifetime of the computation" is more accurate since `'a` bounds the closure, not a reference in the Rust `&` sense. This inconsistency should be corrected.

5. **No high-level comparison table** showing how `Lazy` relates to `Thunk` and `Trampoline`, as the CLAUDE.md does. The code's own documentation would benefit from a similar table or cross-reference.

---

## 7. The Config/Brand System

### How `LazyBrand<Config>` Works

`LazyBrand<Config>` is defined in `brands.rs` as:
```rust
pub struct LazyBrand<Config>(PhantomData<Config>);
```

The `impl_kind!` macro registers it:
```rust
impl_kind! {
    impl<Config: LazyConfig> for LazyBrand<Config> {
        type Of<'a, A: 'a>: 'a = Lazy<'a, A, Config>;
    }
}
```

This means `LazyBrand<RcLazyConfig>` represents the HKT `RcLazy`, and `LazyBrand<ArcLazyConfig>` represents `ArcLazy`. Type class implementations (e.g., `RefFunctor for LazyBrand<RcLazyConfig>`) then dispatch to the appropriate concrete methods.

### Pointer Abstraction Quality

The `LazyConfig` trait is well-designed:
- It bundles all pointer-dependent choices (cell type, thunk type, pointer type) into a single configuration type.
- The `PointerBrand` associated type links back to the pointer hierarchy, enabling generic code to obtain the pointer brand from a `LazyConfig`.
- The `'static` bound on `LazyConfig` prevents configs from borrowing data, which is correct since configs are type-level markers.

### Strengths of the Design

1. **Separation of concerns.** The pointer choice is isolated in `LazyConfig`, not scattered throughout the code.
2. **Open for extension.** A custom `LazyConfig` with `parking_lot::Mutex` or an async cell is theoretically possible.
3. **Type aliases (`RcLazy`, `ArcLazy`) hide the complexity.** Users rarely need to write `Lazy<'a, A, RcLazyConfig>`.
4. **`PointerBrand` association** allows generic code over lazy types to also be generic over the pointer hierarchy, useful for optics and other abstractions.

### Weaknesses

1. **Duplication tax.** Every type class impl must be written twice (once for Rc, once for Arc), with only minor differences in bounds. The `LazyConfig` trait does not provide enough abstraction to write a single generic impl, because the `Send`/`Sync` requirements for Arc cannot be expressed conditionally.

2. **`TryLazy` and `TryThunk` in `LazyConfig`.** The `LazyConfig` trait bundles fallible types (`TryLazy`, `TryThunk`) alongside infallible ones. This means any custom `LazyConfig` must define both, even if only one is needed. Splitting into `LazyConfig` and `TryLazyConfig` would improve separation, though at the cost of more traits.

3. **`Box<Self::Thunk<'a, A>>` in `lazy_new`.** The `lazy_new` method takes `Box<Self::Thunk<'a, A>>`, which for the Rc config is `Box<dyn FnOnce() -> A + 'a>`. The boxing is necessary because `LazyCell::new` requires a sized type. However, this means every `Lazy` creation incurs a heap allocation for the thunk, even when the closure is small. This is unavoidable given the type-erased design, but users should be aware of it.

4. **`LazyConfig` is not `Sealed`.** The docs advertise it as extensible, which is a deliberate choice. However, if future changes to the trait are anticipated, sealing it would prevent breaking downstream implementations. The extensibility vs stability tradeoff should be documented more explicitly.

---

## Summary of Findings

| Area | Assessment |
|------|-----------|
| Correctness | Sound. No bugs found. Delegates to well-tested std types. |
| Thread safety | Correctly handled via separate `Rc`/`Arc` configs with appropriate bounds. |
| Design | Good. Config-parameterized approach is clean. Main trade-off (reference semantics) is fundamental and well-handled. |
| Limitations | `RefFunctor` instead of `Functor` is the main limitation, unavoidable without `Clone` bounds. Missing `Eq`, `Ord`, `From` impls for `ArcLazy`. |
| Documentation | Good overall, but needs explanation of why `Functor` is absent and should fix `'a` lifetime description inconsistency. |
| Code duplication | Significant between Rc and Arc variants. A macro could help. |
| Test coverage | Excellent. Property-based tests for all laws, unit tests for all features, panic poisoning test. |
| Comparison to PureScript | Covers the core functionality. Missing Monad/Applicative/Comonad is fundamental to the Rust design. |
