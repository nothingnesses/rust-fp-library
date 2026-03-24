# Analysis: `fp-library/src/types/lazy.rs`

## Overview

This file implements `Lazy<'a, A, Config>`, a memoized lazy evaluation type parameterized by a `LazyConfig` trait that abstracts over `Rc<LazyCell>` (single-threaded) vs `Arc<LazyLock>` (thread-safe). It provides two concrete aliases, `RcLazy` and `ArcLazy`, along with fixed-point combinators (`rc_lazy_fix`, `arc_lazy_fix`) and type class implementations for `Deferrable`, `SendDeferrable`, `RefFunctor`, `SendRefFunctor`, `Semigroup`, `Monoid`, `Foldable`, `PartialEq`, `PartialOrd`, `Eq`, `Ord`, and `Debug`.

## 1. Overall Design Assessment

The design is sound and well-motivated. The core insight is correct: because `Lazy::evaluate` returns `&A` rather than owned `A`, the standard `Functor` trait cannot be implemented, necessitating the `RefFunctor` family. The `LazyConfig` trait is a reasonable abstraction for parameterizing over pointer/cell strategy.

The file is well-structured: config traits first, then the `Lazy` struct, then convenience constructors per config, then conversions, then the type class implementations, then standard trait impls, then tests.

## 2. The `LazyConfig` Abstraction

### Strengths

- Clean separation of concerns: the pointer type, cell type, and thunk type are all bundled together.
- The `PointerBrand` associated type enables generic code to recover the pointer brand from a config, which is useful for the broader pointer hierarchy.
- The trait is documented as extensible to third parties (e.g., `parking_lot`-based locks), which is a reasonable design goal.
- `TryLazyConfig` is cleanly separated as a supertrait, so configs can opt out of fallible memoization.

### Concerns

- **Verbosity vs value.** The `LazyConfig` trait exists primarily to unify two concrete configs (`RcLazyConfig` and `ArcLazyConfig`). There is no generic code that operates over `Config: LazyConfig` in a meaningful polymorphic way; most of the file has separate `impl` blocks for each config anyway (separate `new`, `pure`, `ref_map`, `Deferrable`, `Semigroup`, `Monoid` impls). The trait does earn its keep for `PartialEq`/`PartialOrd`/`Eq`/`Ord`/`Clone`/`Debug`/`evaluate` which are generic over `Config`, but those are a minority.
- **`lazy_new` takes `Box<Self::Thunk<'a, A>>`.** This means every `Lazy` construction allocates a `Box` for the thunk. For `RcLazyConfig`, `Self::Thunk` is `dyn FnOnce() -> A + 'a`, so `Box<dyn FnOnce()>` is unavoidable. For `ArcLazyConfig`, `Self::Thunk` is `dyn FnOnce() -> A + Send + 'a`, same situation. This is inherent to how `LazyCell`/`LazyLock` work with type-erased closures, but it means every `Lazy::new` call involves a heap allocation for the thunk on top of the `Rc`/`Arc` allocation. This is not a bug, just an inherent cost worth noting.
- **The `'static` bound on `LazyConfig`.** The trait requires `LazyConfig: 'static`, which means configs cannot hold borrowed data. This is fine for the current zero-sized marker structs but would prevent a hypothetical config that borrows runtime state. This is likely an acceptable restriction.

### Verdict

The abstraction is not overengineered, but it is also not pulling as much weight as it could. Most of the duplication between `RcLazy` and `ArcLazy` impls is due to differing `Send`/`Sync` bounds that the trait cannot abstract over. The trait's primary value is in the shared `evaluate` method, the `Kind` impl, the standard trait impls, and extensibility. Reasonable for a library of this nature.

## 3. `Deferrable` Implementations

### `RcLazy` Deferrable

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self {
    RcLazy::new(move || f().evaluate().clone())
}
```

This flattens `Lazy<Lazy<A>>` into `Lazy<A>` by evaluating the inner lazy and cloning. The `Clone` bound on `A` is required because `evaluate()` returns `&A`. This is correct and mirrors PureScript's `Lazy (Lazy a)` bind semantics.

### `ArcLazy` Deferrable

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self {
    f()
}
```

**This is a significant design concern.** The `ArcLazy` `Deferrable` implementation calls `f()` eagerly. The documentation explains this is because `Deferrable::defer` does not require `Send` on the thunk, while `ArcLazy::new` does. So the only safe option is to call `f` immediately.

This means `Deferrable::defer` for `ArcLazy` is not actually lazy; it forces immediate evaluation of the thunk. This violates the spirit of `Deferrable`, even though the transparency law is technically satisfied (since `defer(|| x)` produces a value observationally equivalent to `x` when evaluated). The problem is that `defer(|| expensive_computation())` will run `expensive_computation()` immediately for `ArcLazy`, while for `RcLazy` it will be deferred.

This is not a bug per se, since the law only cares about the evaluated result, not when computation happens. But it is a semantic surprise. A user writing generic code over `Deferrable` may reasonably expect laziness. The documentation is honest about this, which mitigates the concern, but it remains a design wart.

The `SendDeferrable` impl for `ArcLazy` does the right thing (truly defers), which is the intended path for thread-safe deferred construction.

**Possible alternative:** The `Deferrable` impl for `ArcLazy` could be removed entirely, forcing users to use `SendDeferrable` when working with `ArcLazy`. This would make the semantics more honest at the cost of `ArcLazy` not being usable in generic `Deferrable` contexts. Whether this trade-off is worth it depends on how often `ArcLazy` is used in `Deferrable`-generic code.

## 4. `RefFunctor` and `SendRefFunctor` Implementations

### `RefFunctor for LazyBrand<RcLazyConfig>`

Delegates to the inherent `ref_map` method. Sound.

### `SendRefFunctor for LazyBrand<ArcLazyConfig>`

Also delegates to the inherent `ref_map` method. Sound. The `Send + Sync` bounds on `A` and `Send` on `B` and `f` are correctly propagated.

### Missing: `RefFunctor for LazyBrand<ArcLazyConfig>`

The comment on line 776 explains why `ArcLazy` does not implement `RefFunctor`: the trait does not require `Send` on the mapping function, but `ArcLazy::new` requires `Send`. This means `ArcLazy` can only be used with `SendRefFunctor`, not `RefFunctor`.

This is correct reasoning, but it means generic code written against `RefFunctor` cannot use `ArcLazy`. This is an inherent limitation of the split-trait design. The alternative (making `SendRefFunctor` extend `RefFunctor`) would require `RefFunctor` impls to work without `Send`, which `ArcLazy` cannot do. So the current design is the right call.

## 5. `RcLazy` vs `ArcLazy` Comparison

The two variants are largely symmetric, with the following differences:

| Aspect | `RcLazy` | `ArcLazy` |
|--------|----------|-----------|
| Pointer | `Rc<LazyCell<A, Box<dyn FnOnce() -> A + 'a>>>` | `Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send + 'a>>>` |
| Thread safety | `!Send`, `!Sync` | `Send + Sync` |
| `Deferrable` | Truly lazy (deferred) | Eager (calls `f()` immediately) |
| `SendDeferrable` | Not implemented | Truly lazy |
| `RefFunctor` | Yes | No (only `SendRefFunctor`) |
| `From<Thunk>` | Lazy (wraps thunk evaluation) | Eager (evaluates thunk immediately) |
| `From<Trampoline>` | Lazy | Eager |
| `pure` | No `Send` bound on `A` | Requires `A: Send` |

The `From` conversions for `ArcLazy` are necessarily eager because `Thunk` and `Trampoline` are `!Send`. This is correct and well-documented.

## 6. Fixed-Point Combinators

### `rc_lazy_fix` / `arc_lazy_fix`

These use `OnceCell`/`OnceLock` to break the self-referential cycle. The pattern is:

1. Create an empty cell.
2. Create a `Lazy` whose closure reads from the cell and applies `f`.
3. Populate the cell with the `Lazy`.
4. Return the `Lazy`.

**Correctness concern with `rc_lazy_fix`:** The function `f` is `Fn` (not `FnOnce`), which means it can be called multiple times. But in the closure inside `RcLazy::new`, `f(go)` is called, and `f` produces a new `RcLazy<'a, A>`, which is then evaluated and cloned. This means:

- Each time the outer lazy is forced, `f` is called again with a clone of the outer lazy.
- But because the outer lazy is itself memoized, this only happens once.
- The inner lazy returned by `f(go)` is freshly created each time `f` is called, but again, this only happens once due to outer memoization.

This is correct, but there is a subtlety: `f` takes `Fn` instead of `FnOnce`. This is because the closure captured in the `RcLazy` might conceptually need to call `f` each time it is forced (though in practice it is only forced once). Since `RcLazy`'s closure is `FnOnce`, this is fine. However, `f: impl Fn(...)` means `f` must be callable multiple times even though it will only be called once. Using `FnOnce` would be more precise, but it would complicate the ownership story since the closure needs to own `f` and it is captured in a `move` closure that is itself `FnOnce`.

Actually, looking more carefully: the `Fn` bound is needed because the `rc_lazy_fix` closure captures `f` by move, and `f` is used inside that closure. Since the closure passed to `RcLazy::new` is `FnOnce`, `f` only needs to be `FnOnce` as well. The `Fn` bound here is unnecessarily restrictive. It should be `FnOnce`.

Wait, re-reading: `f` is captured in the closure passed to `RcLazy::new`. That closure is `FnOnce`. So `f` would only be called once. `FnOnce` should suffice. The current `Fn` bound unnecessarily restricts callers. Same issue applies to `arc_lazy_fix`.

**Performance note:** The `.evaluate().clone()` inside the fix combinator means the value `A` must be `Clone`, and a clone is always performed even though the inner lazy's `evaluate()` returns `&A`. This is inherent given the reference-based design.

**Edge case:** If `f` eagerly forces the self-reference during construction (i.e., inside the call to `f` in `rc_lazy_fix`), the `OnceCell` has not been populated yet, causing a panic on `unwrap()`. This is documented.

However, there is a subtlety: `f` is called inside the `RcLazy::new` closure, not during construction. The `OnceCell` is populated with the result of `RcLazy::new` before the closure executes. So the panic would only occur if `f` itself forces the lazy value it receives as argument during the `f` call. Since `f` receives a clone of the result (via `cell_ref.get().unwrap().clone()`), and the result is populated before being returned, this is only a problem if the user's `f` forces the self-reference synchronously. The documentation correctly warns about this.

## 7. Documentation Quality

### Strengths

- Module-level documentation clearly explains why `Lazy` does not implement `Functor`.
- Each method has thorough doc comments with `document_signature`, `document_parameters`, `document_returns`, and `document_examples` attributes.
- The `LazyConfig` and `TryLazyConfig` traits have good extensibility documentation.
- Panic behavior is documented.
- The fixed-point combinators have clear explanations of the `OnceCell`/`OnceLock` trick.

### Issues

- **Line 117, `evaluate` doc for `LazyConfig`:** The `document_type_parameters` lists "The lifetime of the computation." twice. The second should describe `'b` (the borrow lifetime).
- **Line 245, `RcLazyConfig::evaluate` doc:** Same duplicate "The lifetime of the computation." issue for `'a` and `'b`.
- **Line 363, `ArcLazyConfig::evaluate` doc:** Same issue.
- **`ArcLazy::pure` requires `A: Send` but not `A: Sync`.** The `where A: Send` bound on `pure` (line 758) is missing `Sync`. The inner `LazyLock` stores `A`, and `Arc<LazyLock<A>>` requires `A: Send + Sync` to be `Send + Sync` itself. However, `LazyLock::new` itself does not require `Send` on `A`; the bounds are on the closure. So `pure(a)` wraps `a` in a closure `move || a`, and the closure is `Send` if `A: Send`. The `LazyLock<A>` will contain `A` after evaluation, and `Arc<LazyLock<A>>` is `Send + Sync` only if `A: Send + Sync`. So there may be a soundness issue if `A: Send` but `!Sync`, since the `ArcLazy` could be shared across threads and `evaluate()` would return `&A` from multiple threads, which requires `A: Sync`. This warrants investigation; the compiler may enforce the `Sync` bound elsewhere through `Arc`'s auto-trait impls, making the explicit bound unnecessary, but the `pure` signature should probably require `A: Send + Sync` for clarity and correctness.
- **`Debug` impl says `Lazy(..)` regardless of evaluation state.** This is a reasonable choice (avoids accidentally forcing evaluation), but it means there is no way to see the evaluated value via `Debug`. PureScript's `Show` instance forces evaluation and shows the value. A `Display` impl that forces evaluation (matching PureScript's `Show`) would complement the non-forcing `Debug` impl.

## 8. Comparison to PureScript's `Data.Lazy`

PureScript's `Data.Lazy` is a foreign-imported opaque type with `defer` and `force` as primitives. The Rust version necessarily has more complexity due to the `Rc`/`Arc` split and explicit lifetime management.

### Instances present in PureScript but absent in Rust

| PureScript | Rust equivalent | Status |
|------------|----------------|--------|
| `Functor` | `Functor` | Cannot implement (returns `&A`); `RefFunctor` used instead. |
| `Apply` | `Apply`/`Applicative` | Missing. Could be implemented via `RefFunctor`-like pattern or with `Clone` bound. |
| `Applicative` | `Applicative` | Missing. |
| `Bind` | `Bind`/`Monad` | Missing. Same issue as `Functor`. |
| `Monad` | `Monad` | Missing. |
| `Extend` | `Extend` | Missing. Would be natural: `extend f x = Lazy::new(move \|\| f(x))`. |
| `Comonad` | `Comonad` | Missing. `extract` = `evaluate().clone()` (requires `Clone`). |
| `Traversable` | `Traversable` | Missing. Would require the HKT machinery to handle `Applicative` lifting. |
| `Foldable1` | (no equivalent trait) | N/A. |
| `Traversable1` | (no equivalent trait) | N/A. |
| `FunctorWithIndex Unit` | `FunctorWithIndex` | Missing (and would need `RefFunctor` variant). |
| `FoldableWithIndex Unit` | `FoldableWithIndex` | Missing. Straightforward with `Unit` index. |
| `TraversableWithIndex Unit` | `TraversableWithIndex` | Missing. |
| `Invariant` | (no equivalent trait) | N/A. |
| `Semiring` | (no equivalent trait) | N/A. |
| `Ring` | (no equivalent trait) | N/A. |
| `CommutativeRing` | (no equivalent trait) | N/A. |
| `EuclideanRing` | (no equivalent trait) | N/A. |
| `HeytingAlgebra` | (no equivalent trait) | N/A. |
| `BooleanAlgebra` | (no equivalent trait) | N/A. |
| `Bounded` | (no equivalent trait) | N/A. |
| `Lazy` (Control.Lazy) | `Deferrable` | Implemented. |
| `Show` | `Display` | Missing. Only `Debug` is implemented (non-forcing). |

The most notable gaps are `Extend`/`Comonad`, which would be natural for a memoized container of exactly one value, and `Display`/`Show`, which PureScript implements by forcing evaluation.

### Semantic differences

- PureScript's `map f l = defer \_ -> f (force l)` creates a new lazy value. The Rust `ref_map` does the same but the mapping function takes `&A` instead of `A`.
- PureScript's `bind l f = defer \_ -> force $ f (force l)` (monadic bind) flattens `Lazy (Lazy A)` to `Lazy A`. The Rust `Deferrable::defer` for `RcLazy` is analogous.
- PureScript's `extend f x = defer \_ -> f x` creates a lazy value from a comonadic extension. This could be implemented in Rust.
- PureScript's `extract = force`. In Rust, `evaluate()` returns `&A`, so `extract` would need `Clone` to produce owned `A`.

## 9. Specific Issues and Recommendations

### Issue 1: `Fn` vs `FnOnce` in fix combinators

`rc_lazy_fix` and `arc_lazy_fix` take `f: impl Fn(...)` but only call `f` once (inside a `FnOnce` closure). The bound should be `FnOnce` to be maximally permissive.

However, there is a subtle issue: in `rc_lazy_fix`, `f` is called inside the closure passed to `RcLazy::new`. If someone calls `rc_lazy_fix` and the resulting `RcLazy` is somehow forced multiple times in a way that bypasses memoization (which should not happen given `LazyCell` semantics), `f` would need to be callable multiple times. But since `LazyCell` guarantees at-most-once execution, `FnOnce` is safe.

**Recommendation:** Change `Fn` to `FnOnce` in both fix combinators.

### Issue 2: `ArcLazy::Deferrable::defer` is eager

As discussed above, this is a semantic surprise. Consider one of:
- Remove the `Deferrable` impl for `ArcLazy` entirely.
- Add a lint/documentation warning at the `Deferrable` trait level that some implementations may be eager.
- Keep as-is (current choice) with clear documentation.

### Issue 3: Missing `Display` implementation

Adding `impl Display for Lazy` that forces evaluation (like PureScript's `Show`) would be useful. The format could mirror PureScript: `(defer \_ -> <value>)`.

### Issue 4: Duplicated `Foldable` implementations

The `Foldable` implementations for `LazyBrand<RcLazyConfig>` and `LazyBrand<ArcLazyConfig>` are nearly identical. They differ only in the brand parameter. This duplication is forced by the trait system (no generic `impl Foldable for LazyBrand<C: LazyConfig>` because the `Clone` bounds differ between `Rc` and `Arc` variants). Consider whether a macro could reduce this duplication.

### Issue 5: `Semigroup::append` clones both sides

```rust
fn append(a: Self, b: Self) -> Self {
    RcLazy::new(move || Semigroup::append(a.evaluate().clone(), b.evaluate().clone()))
}
```

Both `a.evaluate()` and `b.evaluate()` are cloned. This is necessary because `evaluate()` returns `&A`, but it means the `Semigroup` impl is less efficient than it could be if `Lazy` stored owned values. This is an inherent consequence of the reference-based design and is not fixable without changing the fundamental semantics.

### Issue 6: `pure` boxes a closure unconditionally

`RcLazy::pure(a)` creates `RcLazyConfig::lazy_new(Box::new(move || a))`, which allocates a `Box` for a trivial closure and then an `Rc` for the `LazyCell`. For pre-computed values, this could be optimized by using a pre-evaluated `LazyCell` (one that is already in the "initialized" state). However, `LazyCell::new` in the standard library does not support this; it always starts uninitialized. A workaround would be to use `Rc::new(OnceCell)` initialized with the value, but that would change the internal representation. This is a minor performance concern for hot paths.

### Issue 7: No `Hash` implementation

`Lazy` implements `Eq` and `Ord` but not `Hash`. If `A: Hash`, `Lazy<A>` should also be `Hash` (by forcing and hashing the value). This would enable using `Lazy` values as keys in hash-based collections.

### Issue 8: Documentation typo in `evaluate` type parameter descriptions

The `evaluate` method's `document_type_parameters` lists "The lifetime of the computation." for both `'a` and `'b`. The second should be "The lifetime of the borrow." or similar. This appears in `LazyConfig::evaluate`, `RcLazyConfig::evaluate`, `ArcLazyConfig::evaluate`, and the `TryLazyConfig` equivalents.

## 10. Test Coverage Assessment

The test suite is thorough:

- Caching behavior (computed once).
- Shared cache across clones.
- Thread safety for `ArcLazy`.
- Conversions from `Thunk` and `Trampoline`.
- `Deferrable` and `SendDeferrable`.
- `pure` constructors.
- `ref_map` for both variants.
- `Semigroup` and `Monoid` with law verification.
- `Foldable` (`fold_right`, `fold_left`, `fold_map`).
- `PartialEq` and `PartialOrd`.
- Fixed-point combinators (correctness, memoization, clone sharing, thread safety).
- QuickCheck properties for memoization, clone sharing, determinism, RefFunctor laws, Deferrable laws, Semigroup associativity, and Monoid identity.
- Panic poisoning.
- `PointerBrand` type-level assertions.

**Missing test coverage:**
- `Ord` (only `PartialOrd` is tested, though `Ord` is trivially correct if `PartialOrd` is correct).
- `Debug` formatting.
- Generic `Foldable` for `ArcLazy` (only `RcLazy` Foldable is tested).
- `SendRefFunctor` law tests (only `RefFunctor` laws are tested via QuickCheck).
- Edge case: `rc_lazy_fix`/`arc_lazy_fix` where `f` actually uses the self-reference (all current tests ignore it).

## 11. Summary

The `lazy.rs` file is a well-designed, thoroughly documented, and extensively tested implementation of memoized lazy evaluation. The `LazyConfig` abstraction, while not eliminating all duplication, provides a reasonable unified interface for `Rc` vs `Arc` variants. The main areas for improvement are:

1. **`Fn` should be `FnOnce` in fix combinators** (unnecessarily restrictive).
2. **`ArcLazy::Deferrable::defer` being eager** is a known compromise worth revisiting.
3. **Missing `Display`, `Hash`, and potentially `Extend`/`Comonad` implementations** compared to PureScript.
4. **Documentation typo** in `evaluate` type parameter descriptions (duplicate lifetime description).
5. **Foldable duplication** between `Rc` and `Arc` variants could be reduced with a macro.
6. **Test coverage** could be expanded for `ArcLazy` Foldable, `SendRefFunctor` laws, and fix combinators that use the self-reference.
