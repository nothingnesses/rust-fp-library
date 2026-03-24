# Lazy Evaluation Hierarchy: Consolidated Research Summary

This document consolidates findings from 17 research files analyzing the lazy evaluation subsystem of `fp-library`. Issues are grouped by theme and prioritized by severity.

## Areas Where the Design is Sound (Do Not Change)

These aspects were consistently praised across multiple research files and should be preserved:

- **The `Deferrable` / `SendDeferrable` trait split.** The separation is well-motivated by Rust's `Send` constraints and is the only sound approach given the type system. The lifetime parameterization (`Deferrable<'a>`), `FnOnce` choice, and `Sized` bound are all correct and minimal.
- **The `RefFunctor` / `SendRefFunctor` separation from `Functor`.** Memoized types return `&A` from `evaluate()`, making standard `Functor` impossible without `Clone`. The ref-based mapping traits honestly represent what these types can do.
- **`Thunk` as the lightweight HKT-compatible computation type.** It fills a clear niche as the only type that is both HKT-compatible and supports non-`'static` lifetimes. HKT trait implementations are correct.
- **The newtype wrapper pattern for fallible types.** `TryThunk` wrapping `Thunk<Result<A, E>>`, `TrySendThunk` wrapping `SendThunk<Result<A, E>>`, `TryTrampoline` wrapping `Trampoline<Result<A, E>>`, and `TryLazy` wrapping `Lazy<Result<A, E>>` are all zero-cost, clean, and correct.
- **`Trampoline` as a newtype over `Free<ThunkBrand, A>`.** Provides a focused API, enables inherent methods and `From` conversions, and prevents leaking `Free` internals.
- **`Free` monad's "Reflection without Remorse" core algorithm.** The CatList-based continuation queue provides O(1) amortized bind and is correctly implemented. The `'static` requirement from `Box<dyn Any>` is well-justified and unavoidable.
- **`Evaluable` trait.** Despite having only one implementor (`ThunkBrand`), it cleanly decouples effect execution from the `Free` structure and keeps `Free` generic. Worth keeping.
- **`CatList` core data structure.** The `VecDeque`-based spine is a pragmatic Rust adaptation of the standard catenable list. O(1) `snoc`/`append` and O(1) amortized `uncons` (over a full drain) are correct.
- **`Step<A, B>` naming and design.** `Loop`/`Done` names are self-documenting for `tail_rec_m`. Comprehensive type class coverage across three brands is thorough and principled.
- **The brand hierarchy.** The decomposition along fallibility, thread safety, and memoization axes is principled and consistently named. The dependency ordering (brands -> classes -> types) is respected.
- **`SendThunk` not implementing HKT traits.** The `Functor`/`Semimonad` trait signatures lack `Send` bounds on closures, making HKT implementations impossible without breaking the `Send` invariant. Providing inherent methods instead is the correct approach.
- **`LazyConfig` trait abstraction.** Parameterizing `Lazy` over `RcLazyConfig` / `ArcLazyConfig` is clean, extensible, and avoids type duplication for the shared `evaluate`, `Clone`, `Debug`, `PartialEq`, `Ord` implementations.

---

## 1. Correctness Concerns

### 1.1 `Semigroup::append` does not short-circuit for fallible types

**Files:** `try_thunk.rs`, `try_send_thunk.rs`, `try_lazy.rs`

All three types evaluate both operands before pattern-matching, e.g.:
```rust
match (a.evaluate(), b.evaluate()) { ... }
```
This evaluates `b` even when `a` is `Err`, wasting work and potentially triggering side effects unnecessarily. Every other combinator on these types (`bind`, `then`, `lift2`) short-circuits on the first error. The `Semigroup::append` behavior is inconsistent with them.

**Fix:** Use sequential `?`-style evaluation so that `b` is only evaluated if `a` succeeds. Must be changed consistently across all three types.

### 1.2 `Free::Drop` implementation is incomplete

**File:** `free.rs`

The iterative `Drop` handles `Bind` and `Map` chains but does not:
- Iteratively drop `CatList` continuations (each is a `Box<dyn FnOnce>` that may capture `Free` values).
- Handle `Wrap` variants (which contain `Free` values inside the functor layer).

Deep bind chains with many captured `Free` values in continuations could still overflow the stack during drop.

### 1.3 `Free::resume` relies on a runtime invariant (functor calls map exactly once)

**File:** `free.rs`

The `Wrap` branch in `resume` uses `Cell::take` to move continuations out of a shared closure, relying on the invariant that `Functor::map` calls the mapping function exactly once. Calling it zero times silently drops continuations; calling it twice panics. This is correct for `ThunkBrand` but is not enforced at the type level.

### 1.4 `ArcLazy::pure` may be missing a `Sync` bound

**File:** `lazy.rs`

`ArcLazy::pure` requires `A: Send` but not `A: Sync`. Since `Arc<LazyLock<A>>` returns `&A` from `evaluate()` and `Arc` enables cross-thread sharing, `A` must be `Sync` for safe concurrent reads. The compiler may enforce this elsewhere via `Arc`'s auto-trait impls, but the explicit bound on `pure` should include `Sync` for clarity and correctness.

---

## 2. Design Flaws and Inconsistencies

### 2.1 `SendRefFunctor` does not extend `RefFunctor` (breaks library pattern)

**Files:** `ref_functor.rs`, `send_ref_functor.rs`, `lazy.rs`

`SendDeferrable: Deferrable` and `SendCloneableFn: CloneableFn` follow a supertrait pattern, but `SendRefFunctor` is an independent trait with no `RefFunctor` supertrait. Consequences:
- `LazyBrand<ArcLazyConfig>` implements `SendRefFunctor` but NOT `RefFunctor`.
- Generic code bounded by `RefFunctor` cannot accept `ArcLazy`.
- Confusingly, `TryLazyBrand<E, ArcLazyConfig>` does implement both traits.

**Fix:** Either add the supertrait relationship plus a `RefFunctor` impl for `LazyBrand<ArcLazyConfig>`, or document the limitation prominently. The supertrait approach may not be possible because `RefFunctor::ref_map` lacks `Send` bounds that `ArcLazy` requires.

### 2.2 Documentation falsely claims `ArcLazy` implements both `RefFunctor` and `SendRefFunctor`

**Files:** `send_ref_functor.rs` (trait docs), `ref_functor.rs` (analysis)

The `SendRefFunctor` trait documentation states "ArcLazy implements both RefFunctor and SendRefFunctor." This is false; `LazyBrand<ArcLazyConfig>` only implements `SendRefFunctor`.

### 2.3 Unnecessary `Sync` bound on `SendDeferrable::send_defer`

**File:** `send_deferrable.rs`

The signature requires `impl FnOnce() -> Self + Send + Sync + 'a`, but `Sync` is never needed by any of the four implementations (`SendThunk`, `ArcLazy`, `TrySendThunk`, `ArcTryLazy`). All use the closure via `FnOnce` in a `move ||` capture, where only `Send` matters. The `Sync` bound unnecessarily restricts callers (e.g., closures capturing `Cell<T>`).

**Fix:** Remove `Sync` from the trait method and free function signatures.

### 2.4 `Deferrable::defer` evaluates eagerly for `Send` types

**Files:** `deferrable.rs`, `send_deferrable.rs`, `lazy.rs`, `send_thunk.rs`, `try_send_thunk.rs`, `try_lazy.rs`

For `SendThunk`, `TrySendThunk`, `ArcLazy`, and `ArcTryLazy`, `Deferrable::defer` calls `f()` immediately because the trait does not require `Send` on the closure. The `SendDeferrable` trait exists as the correct fix. However, generic code using `Deferrable` may silently get eager evaluation for these types.

This is an inherent tension in Rust's type system, not a bug. The `SendDeferrable` subtrait is the right mitigation. The trait-level documentation should warn about this.

### 2.5 `Fn` instead of `FnOnce` in `rc_lazy_fix` and `arc_lazy_fix`

**File:** `lazy.rs`

Both fix combinators take `f: impl Fn(...)` but only call `f` once (inside a `FnOnce` closure passed to `Lazy::new`). The `Fn` bound is unnecessarily restrictive; `FnOnce` would be correct and more permissive.

### 2.6 `TryLazy::map_err` clones the success side unnecessarily

**File:** `try_lazy.rs`

The implementation uses `.cloned()` which clones the `Ok` side, then `map_err(f)` transforms only the error. The `Ok(A)` value gets cloned even though `f` only operates on the error. An explicit `match` would clone only the side that needs it, matching the efficiency of `TryLazy::map`.

### 2.7 Missing `Applicative` and `Monad` marker traits on `Step` brands

**File:** `step.rs`

Both `StepLoopAppliedBrand` and `StepDoneAppliedBrand` implement all component traits (`Pointed`, `Semiapplicative`, `Semimonad`, etc.) but never implement the `Applicative` or `Monad` marker traits. Test sections are labeled "Monad Laws" despite `Monad` not being implemented.

### 2.8 `Free::Map` variant complexity vs. benefit

**File:** `free.rs`

The `Map` variant documentation claims it "avoids the type-erasure roundtrip," but the implementation still calls `self.erase_type()` and downcasts via `TypeErasedValue`. The actual benefit is marginal (one fewer continuation in the `CatList`), while the variant adds significant complexity to `evaluate`, `resume`, `drop`, and `erase_type`.

**Options:** Remove `Map` and implement via `bind` (simplifies code), or fix the documentation to accurately describe the actual benefit.

### 2.9 `Foldable` for `TryLazyBrand` has an unnecessarily tight `E: Clone` bound

**File:** `try_lazy.rs`

The `Foldable` impl requires `E: 'static + Clone`, but the fold methods never clone `E`. The `Clone` bound appears inherited from sharing the brand with `RefFunctor`, not from actual need.

### 2.10 `pub(crate)` visibility on `Free`'s inner `Option<FreeInner>` field

**File:** `free.rs`

The field is `pub(crate)`, allowing any crate-internal code to construct `Free(None)` or access the inner option directly, weakening the invariant that `Free` values are consumed exactly once. Should be private with accessor methods.

---

## 3. Missing Implementations

### 3.1 Missing conversions

| Conversion | Files | Notes |
|---|---|---|
| `From<SendThunk> for Thunk` | `send_thunk.rs` | Zero-cost widening (drop `Send` bound). |
| `From<Thunk<'static, A>> for Trampoline<A>` | `thunk.rs` | Natural upgrade to stack safety. |
| `From<TrySendThunk> for TryThunk` | `try_thunk.rs` | Natural widening. |
| `From<ArcTryLazy> for TrySendThunk` | `try_send_thunk.rs` | Interop with memoized layer. |
| `TryThunk::into_inner() -> Thunk<Result<A, E>>` | `try_thunk.rs` | Escape hatch to inner type. |
| `TryTrampoline::into_trampoline()` | `try_trampoline.rs` | Access inner `Trampoline<Result<A, E>>`. |
| `Step <-> Result` and `Step <-> ControlFlow` | `step.rs` | Standard interop conversions. |

### 3.2 Missing methods and combinators

| Method | Files | Notes |
|---|---|---|
| `SendThunk::zip_with` / `apply` | `send_thunk.rs` | No HKT `Semiapplicative`, so inherent combinators needed for multi-value composition. |
| `TryThunk` inherent `bimap` | `try_thunk.rs` | `TrySendThunk` has it; `TryThunk` only has it via `Bifunctor` HKT. |
| `TryLazy::and_then` / `or_else` | `try_lazy.rs` | Standard combinators for fallible types. |
| `TryTrampoline::pure` | `try_trampoline.rs` | `TryThunk` has both `pure` and `ok`; `TryTrampoline` only has `ok`. |
| `Trampoline::ap` / `flatten` | `trampoline.rs` | Minor ergonomic gaps; `bind` subsumes `ap`. |
| `WithIndex` / `FunctorWithIndex` / `FoldableWithIndex` for `TryThunk` brands | `try_thunk.rs` | `Thunk` has these; `TryThunk` does not. Trivial with `Index = ()`. |

### 3.3 Missing trait implementations

| Trait | Files | Notes |
|---|---|---|
| `Lazy: Display` | `lazy.rs` | PureScript's `Show` forces evaluation and displays the value. |
| `Lazy: Hash` | `lazy.rs` | `Eq` and `Ord` are implemented but `Hash` is not. |
| `Lazy: Extend / Comonad` | `lazy.rs` | Natural for a memoized single-element container. Library-level gap (no `Comonad` trait yet). |
| `Lazy: FoldableWithIndex` | `lazy.rs` | PureScript has `FoldableWithIndex Unit` for `Lazy`. |
| `Evaluable` naturality law | `evaluable.rs` | No laws are documented; the naturality law `evaluate(map(f, fa)) == f(evaluate(fa))` should be stated. |
| `Applicative` / `Monad` markers for `Step` brands | `step.rs` | Component traits are all present; marker traits are missing. |
| No `SendTrampoline` type | `trampoline.rs` | Users needing stack-safe computation across thread boundaries have no option. Notable hierarchy gap. |
| `CatList` borrowing iterator | `cat_list.rs` | Forces `Clone` bound on `PartialEq`, `Hash`, `Ord`. |
| `CatListIterator::size_hint` | `cat_list.rs` | Length is tracked; `size_hint` would improve every `collect()` call (used by `map`, `bind`, `from_iter`, parallel methods). |

---

## 4. Documentation Issues

### 4.1 Cross-cutting documentation problems

- **Eager `Deferrable::defer` for `Send` types is insufficiently warned about.** The trait-level docs should note that some implementations evaluate eagerly, and that `SendDeferrable` should be preferred for true deferral with thread-safe types. (Affects `deferrable.rs`, `send_deferrable.rs`.)
- **`Traversable` limitation on `Thunk` is imprecisely explained.** The docs attribute the issue to `FnOnce` not being cloneable, but for a single-element container, `traverse` should work without cloning. The real blocker is likely the `Traversable` trait's bounds, not `Thunk`'s fundamental design. (Affects `thunk.rs`.)
- **`Foldable` error-discarding behavior for `TryLazy` is not mentioned** in the module-level docs. Silently returning the accumulator on `Err` could surprise users. (Affects `try_lazy.rs`.)
- **No guidance on when to use `TryLazy` vs `Lazy<Result<A, E>>` vs `Result<Lazy, E>`.** Three distinct designs with different trade-offs. (Affects `try_lazy.rs`.)

### 4.2 File-specific documentation issues

| Issue | File |
|---|---|
| `evaluate` type parameter docs say "The lifetime of the computation." for both `'a` and `'b`; `'b` should describe "the borrow lifetime." | `lazy.rs` (lines 117, 245, 363, and `TryLazyConfig` equivalents) |
| Duplicated "Stack Safety" section in struct doc comment (lines 56-61 and 92-96). | `try_thunk.rs` |
| `OkAppliedBrand` doc examples are potentially confusing without explaining the dual-channel encoding. | `try_thunk.rs` |
| `pure` and `ok` redundancy is undocumented (both produce `Ok(a)` identically). | `try_thunk.rs` |
| Module-level example uses `Thunk::new(|| 42)` inside `defer`, creating a thunk-of-a-thunk; should use `Thunk::pure(42)`. | `deferrable.rs` |
| `SendCloneableFn` analogy is imprecise; `FnOnce` vs `Fn` semantics differ regarding `Sync` necessity. | `send_deferrable.rs` |
| `RefFunctor` identity law requires `A: Clone` but does not state it. | `ref_functor.rs` |
| No cross-reference from `RefFunctor` docs to `SendRefFunctor`. | `ref_functor.rs` |
| No explanation of why `FnOnce` is used instead of `Fn` for `RefFunctor::ref_map`. | `ref_functor.rs` |
| `Free::Map` documentation inaccurately claims it "avoids the type-erasure roundtrip." | `free.rs` |
| `resume` documentation does not adequately explain the `Cell` trick and its invariant. | `free.rs` |
| `memoize_arc` naming does not convey eager evaluation; `evaluate_into_arc_lazy` would be clearer. | `trampoline.rs` |
| Module doc memoization example is verbose; should reference the `memoize()` method. | `trampoline.rs` |
| `map` inherent method on `Thunk` accepts `FnOnce` but docs do not note the difference from HKT `Functor::map` (which requires `Fn`). | `thunk.rs` |
| `TryLazy::map` naming differs from `Lazy::ref_map`; both take `&A` but use different names. | `try_lazy.rs` |
| `LazyBrand<Config>` and `TryLazyBrand<E, Config>` doc comments do not describe their type parameters. | `brands.rs` |
| No documentation of why `TrySendThunk` lacks partially-applied brands (unlike `TryThunk`). | `brands.rs` |
| Variant doc comments on `Step` lack terminal periods. | `step.rs` |
| `Debug` for `Trampoline` always prints `<unevaluated>` even for `Pure` values where `A: Debug`. | `trampoline.rs` |
| No mention that there is no `Send`-capable stack-safe lazy type in the hierarchy. | `send_thunk.rs` |
| CatList docs say "no reversal overhead" vs two-stack queues, which is slightly misleading (VecDeque still amortizes resizes). | `cat_list.rs` |
| `uncons` amortized complexity nuances (O(k) per individual call where k = sublists) should be documented. | `cat_list.rs` |

---

## 5. Performance Issues

### 5.1 `CatListIterator` lacks `size_hint` / `ExactSizeIterator`

**File:** `cat_list.rs`

`CatList` tracks its length, but the iterator does not report `size_hint`. Every `collect::<Vec<_>>()` (used by `map`, `bind`, `fold_right`, `from_iter`, parallel methods) allocates with the default heuristic instead of pre-allocating. Low-effort, high-impact fix.

### 5.2 Double clone of `f` in `Trampoline::tail_rec_m`

**File:** `trampoline.rs`

Each recursive step clones `f` twice (once for `f_clone` and once inside the `bind` closure). For closures with expensive-to-clone captures, this is suboptimal. The `arc_tail_rec_m` variant avoids this since `Arc::clone` is cheap.

### 5.3 Minor closure indirection in `SendThunk::memoize_arc`

**File:** `send_thunk.rs`

`memoize_arc` wraps `self` in a new closure (`move || self.evaluate()`) rather than directly passing the inner `Box<dyn FnOnce>`. One unnecessary level of indirection; likely optimized away by the compiler.

### 5.4 `Free::erase_type` allocates on every `evaluate`/`resume` call

**File:** `free.rs`

Both `evaluate` and `resume` begin by calling `erase_type()`, which boxes values even for the simplest `Pure(a)` case. This is inherent to the type-erasure design.

---

## 6. Ergonomic Improvements

### 6.1 `Deferrable` / `SendDeferrable` traits are not used as generic bounds anywhere

**Files:** `deferrable.rs`, `send_deferrable.rs`

Neither trait appears as a bound in any generic function or struct outside their own free function wrappers. They serve as a naming convention and documentation anchor. This is acceptable but means the supertrait relationship is untested in generic contexts.

### 6.2 `send_defer` and `defer` may not be reachable via `functions::*`

**File:** `send_deferrable.rs`

Module-level examples import via `functions::*`, but it is unclear whether the re-export macro picks up these free functions. If not, the examples are misleading.

### 6.3 `catch` on fallible types cannot change the error type

**Files:** `try_thunk.rs`, `try_trampoline.rs`

`catch` requires the recovery function to return the same error type `E`. A `catch_with` variant allowing `E -> TryThunk<A, E2>` would be more flexible.

### 6.4 `LazyBrand<Config>` lacks trait bounds at the definition site

**File:** `brands.rs`

`LazyBrand<Config>` has no `Config: LazyConfig` bound on the struct, unlike `FnBrand<PtrBrand: RefCountedPointer>`. This means `LazyBrand<i32>` compiles as a type (though it has no `Kind` impl). Same issue for `TryLazyBrand<E, Config>`.

---

## 7. Missing Tests

| Test gap | File |
|---|---|
| `MonadRec::tail_rec_m` stack safety test with large iteration count. | `thunk.rs` |
| Cross-thread integration test (actually send a `SendThunk` to another thread). | `send_thunk.rs` |
| `Semigroup::append` where second operand fails but first succeeds. | `try_send_thunk.rs`, `try_thunk.rs` |
| `catch` where recovery itself fails. | `try_send_thunk.rs` |
| `ArcLazy` `Foldable` tests (only `RcLazy` is tested). | `lazy.rs` |
| `SendRefFunctor` law tests via QuickCheck. | `lazy.rs` |
| `rc_lazy_fix`/`arc_lazy_fix` where `f` actually uses the self-reference. | `lazy.rs` |
| `memoize` / `memoize_arc` unit tests (only in doc tests). | `try_trampoline.rs` |
| Monad law tests for `Free` (left identity, right identity, associativity). | `free.rs` |
| Mixed deep chains (interleaved `map`, `bind`, `wrap`, `lift_f`). | `free.rs` |
| `FunctorWithIndex` / `FoldableWithIndex` via HKT free functions. | `thunk.rs` |
| `bimap` on both success and error paths simultaneously. | `try_send_thunk.rs` |
