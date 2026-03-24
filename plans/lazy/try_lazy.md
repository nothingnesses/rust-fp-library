# TryLazy Analysis

**File:** `fp-library/src/types/try_lazy.rs`
**Lines:** 1285

## Overview

`TryLazy<'a, A, E, Config>` is a lazily-evaluated, memoized container for fallible computations. It wraps a `Config::TryLazy<'a, A, E>` (which resolves to `Rc<LazyCell<Result<A, E>, ...>>` or `Arc<LazyLock<Result<A, E>, ...>>`). It provides `RcTryLazy` and `ArcTryLazy` type aliases for single-threaded and thread-safe variants respectively.

## 1. Design

### Relationship to Lazy

TryLazy is structurally `Lazy<Result<A, E>>` with a dedicated type and ergonomic API. The internal storage is literally `Rc<LazyCell<Result<A, E>>>` / `Arc<LazyLock<Result<A, E>>>`, defined via the `LazyConfig::TryLazy` associated type. This parallels how `TryThunk` wraps `Thunk<Result<A, E>>` and `TryTrampoline` wraps `Trampoline<Result<A, E>>`.

### Wrapper vs. Dedicated Type: Is This the Right Call?

**Advantages of the wrapper approach (current design):**

- `evaluate()` returns `Result<&A, &E>` rather than `&Result<A, E>`, which is more ergonomic for pattern matching and chaining.
- Dedicated `ok()`, `err()`, `map()`, `map_err()`, and `catch_unwind()` methods provide a tailored API for fallible computation.
- `TryLazyBrand<E, Config>` fixes the error type, enabling a proper `Kind` instance that is polymorphic over the success type `A`. A raw `Lazy<Result<A, E>>` with `LazyBrand` would be polymorphic over `Result<A, E>` as a single unit, making it impossible to express "functor over the success type."
- Consistent with the library's established pattern: `TryThunk` wraps `Thunk<Result<A, E>>`, `TryTrampoline` wraps `Trampoline<Result<A, E>>`.

**What you would lose with just `Lazy<Result<A, E>>`:**

- No way to get `Result<&A, &E>` from evaluate; you'd get `&Result<A, E>` and need manual `.as_ref()` each time.
- No HKT brand that is polymorphic over the success type with a fixed error. The `LazyBrand` is polymorphic over the entire `A` in `Lazy<'a, A>`, so you'd need `LazyBrand` applied to `Result<A, E>`, which is not a functor over `A`.
- No place to hang `catch_unwind` or the `From<TryThunk>` / `From<TryTrampoline>` conversions.

**Verdict:** The wrapper is well-justified. It is not redundant with `Lazy<Result<A, E>>`.

### Integration with LazyConfig

The `TryLazy` associated type and `try_lazy_new` / `try_evaluate` methods are baked directly into the `LazyConfig` trait, which means `TryLazy` was a first-class design concern rather than an afterthought. This is good; it ensures both `RcLazyConfig` and `ArcLazyConfig` provide consistent fallible-memoization support.

## 2. Implementation Correctness

### Core Mechanics

The fundamental evaluate-once, cache-the-result semantics are delegated entirely to `LazyCell` / `LazyLock` from `std`. The `TryLazy` struct is a transparent newtype over `Config::TryLazy<'a, A, E>`, and `evaluate()` calls `Config::try_evaluate()`, which forces the underlying cell and calls `.as_ref()` on the cached `Result`. This is correct and minimal.

### map / map_err Asymmetry in Clone Requirements

- `map<B>(f: FnOnce(&A) -> B)` requires `E: Clone` because the error must be cloned from `&E` to `E` for the new cell's `Result<B, E>`.
- `map_err<E2>(f: FnOnce(&E) -> E2)` requires `A: Clone` because the success value must be cloned from `&A` to `A` for the new cell's `Result<A, E2>`.

This is a fundamental consequence of reference-based evaluation: `evaluate()` returns `Result<&A, &E>`, so to produce an owned `Result` with one side transformed, the other side must be cloned. The implementation is correct.

**Subtle design implication:** Both `map` and `map_err` consume `self` by value, but since `TryLazy` is `Clone` (it's `Rc`/`Arc` wrapped), consuming `self` does not invalidate other clones. The consumed clone is captured into the new cell's initializer. This is fine.

### map Creates a New Cell (No Cache Sharing)

Each `map` or `map_err` call creates an entirely new `TryLazy` cell. The original cell is evaluated as a side effect of evaluating the new cell. The mapped result is cached in the new cell, not in the original. This means:

- Evaluating the mapped cell triggers evaluation of the original (if not already evaluated).
- The original cell's cache is populated as a side effect.
- The mapped cell has its own independent cache.

This is correct behavior but has a cost: a chain of `map` calls creates a chain of cells, each holding an `Rc`/`Arc` reference to the previous. This is analogous to how `Lazy::ref_map` works.

### Deferrable Implementation

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self {
    Self::new(move || f().evaluate().cloned().map_err(Clone::clone))
}
```

This calls `.cloned()` (which clones the `Ok` side from `&A` to `A`) and then `.map_err(Clone::clone)` (which clones the `Err` side from `&E` to `E`). This requires both `A: Clone` and `E: Clone`, which is reflected in the trait bounds. Correct but notable: the cloning is necessary because the inner `TryLazy`'s cache owns the value, and the outer `TryLazy` needs its own owned `Result`.

### Panic Behavior

If the initializer closure panics (without `catch_unwind`), the underlying `LazyCell` / `LazyLock` is poisoned. The test `test_panic_poisoning` verifies this. The `catch_unwind` / `catch_unwind_with` methods provide an escape hatch. This is well-documented in the `Lazy` type's doc comment and tested.

### From Conversions

- `From<TryThunk>`: Creates a new cell that calls `eval.evaluate()`. Since `TryThunk` re-evaluates on each call, this captures the thunk and runs it once when the cell is first forced. Correct.
- `From<TryTrampoline>`: Same pattern. Note `TryTrampoline` has `'static` bounds, but the `From` impl accepts it into a `TryLazy<'a, ...>` because `'static: 'a`. Correct.
- `From<Lazy<A>>`: Requires `A: Clone` because `Lazy::evaluate()` returns `&A`, which must be cloned to produce `Ok(A)`. Correct.
- `From<Result<A, E>>`: Wraps in a closure. Correct.

### No Bugs Found

The implementation appears correct. The type signatures, trait bounds, and runtime behavior are consistent.

## 3. Consistency with the Library

### Parallels with TryThunk

| Aspect | TryThunk | TryLazy |
|--------|----------|---------|
| Wraps | `Thunk<Result<A, E>>` | `Config::TryLazy<Result<A, E>>` |
| Memoized | No | Yes |
| HKT brands | `TryThunkBrand`, `TryThunkErrAppliedBrand<E>`, `TryThunkOkAppliedBrand<A>` | `TryLazyBrand<E, Config>` |
| Functor impl | Yes (via HKT brand) | **No** |
| Bifunctor | Yes | **No** |
| Semimonad/Monad | Yes | **No** |
| Deferrable | Yes | Yes |
| map/map_err | Inherent methods + HKT trait impls | Inherent methods only |
| evaluate returns | `Result<A, E>` (owned) | `Result<&A, &E>` (borrowed) |

The key structural difference is that `TryThunk` has full HKT type class implementations (`Functor`, `Semimonad`, `Bifunctor`, `Foldable`, etc.) while `TryLazy` has **none**. This is a significant gap.

### Parallels with Lazy

`Lazy` implements `RefFunctor` (and `SendRefFunctor` for the Arc variant) because its `evaluate()` returns `&A`, making standard `Functor` (which expects owned values) impossible. `TryLazy` does not implement `RefFunctor` or any equivalent; it only has inherent `map` and `map_err` methods. This is an inconsistency: if `Lazy` has `RefFunctor`, then `TryLazy` could reasonably have some analogous trait implementation, perhaps a `RefBifunctor` or at minimum `RefFunctor` for `TryLazyBrand<E, Config>`.

### LazyConfig Integration

The `LazyConfig` trait carries `TryLazy`, `TryThunk`, `try_lazy_new`, and `try_evaluate` as associated types/methods alongside the infallible variants. This tight coupling is good for consistency but means any new `LazyConfig` implementor must provide both infallible and fallible variants.

## 4. Limitations

### No HKT Type Class Implementations

`TryLazy` has a `Kind` implementation via `impl_kind!` for `TryLazyBrand<E, Config>`, but no type class trait implementations (`Functor`, `Monad`, `Foldable`, `Bifunctor`, etc.). The `map` and `map_err` methods exist as inherent methods only, so generic HKT code cannot operate on `TryLazy` values polymorphically.

This is the most significant limitation. For comparison, `TryThunk` has `Functor`, `Semimonad`, `Bifunctor`, `Bifoldable`, `Foldable`, `Pointed`, `Lift`, and more.

The difficulty is that `TryLazy::evaluate()` returns references, so standard `Functor::map(f: impl Fn(A) -> B, fa) -> fb` does not apply. The same problem exists for `Lazy`, which solved it with `RefFunctor`. TryLazy would need analogous ref-based trait implementations.

### Clone Requirements for map/map_err

Both `map` and `map_err` require `Clone` on the "other" type parameter (`E: Clone` for `map`, `A: Clone` for `map_err`). This is unavoidable given the reference-based evaluate, but it means non-Clone types cannot be mapped. In contrast, `TryThunk::map` does not require `Clone` because `evaluate()` returns owned values.

### No flat_map / bind

There is no monadic bind operation. `Deferrable::defer` provides a form of flattening but not a general `flat_map`. Given the reference semantics and Clone requirements, implementing a true `Monad` would be awkward.

### No Semigroup/Monoid

`Lazy` implements `Semigroup` and `Monoid` (combining the inner values). `TryLazy` does not. It could, with the semantics of "combine if both Ok, propagate first Err" (matching `Result`'s semigroup behavior).

### ArcLazyConfig Missing From<TryThunk> and From<TryTrampoline>

The `From<TryThunk>` and `From<TryTrampoline>` conversions are only implemented for `RcLazyConfig`. There are no corresponding `ArcLazyConfig` variants. Adding these would require `Send` bounds on the thunk/trampoline results.

### catch_unwind Only for Rc and Arc Separately

The `catch_unwind` and `catch_unwind_with` methods are implemented separately for `RcLazyConfig` and `ArcLazyConfig` with duplicated logic. This could potentially be unified through a helper, though the different `Send` bounds make a fully generic version awkward.

## 5. Alternative Designs

### Could This Be `Lazy<Result<A, E>>`?

No, not without losing significant ergonomic and type-system benefits (discussed in Section 1). The wrapper is the right choice.

### Could TryLazy Wrap Lazy Internally?

Instead of having `Config::TryLazy` as a separate associated type, `TryLazy` could be defined as `TryLazy<'a, A, E, Config>(Lazy<'a, Result<A, E>, Config>)`, similar to how `TryThunk(Thunk<Result<A, E>>)` works. This would:

- Reduce the `LazyConfig` trait surface (no need for `TryLazy`, `TryThunk`, `try_lazy_new`, `try_evaluate`).
- Let `TryLazy` delegate to `Lazy` for construction and evaluation.
- The `evaluate()` method would call `self.0.evaluate()` (returning `&Result<A, E>`) and then `.as_ref()` to get `Result<&A, &E>`.

This is arguably cleaner and is the pattern used by `TryThunk` and `TryTrampoline`. The current approach of baking fallible variants into `LazyConfig` creates more API surface for the same outcome. However, there may be historical or extensibility reasons for the current design (e.g., allowing a custom `LazyConfig` to use a different backing store for fallible vs. infallible cells).

### RefBifunctor Trait

A `RefBifunctor` trait (analogous to `RefFunctor` but for two type parameters) would allow `TryLazyBrand` to participate in generic HKT code. This does not currently exist in the library but would be the natural extension.

## 6. Documentation Quality

### Strengths

- Every method has `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` macro attributes, consistent with the library's documentation standards.
- Doc examples are runnable and cover both Ok and Err cases.
- The module-level doc comment is concise and informative.
- The HKT representation section in the struct doc comment correctly describes `TryLazyBrand<E, Config>`.

### Weaknesses

- **No panic documentation on `evaluate()`:** If the initializer panics (without `catch_unwind`), subsequent calls to `evaluate()` will also panic due to poisoning. This is documented on `Lazy` but not on `TryLazy`. A `# Panics` section should be added to `TryLazy::evaluate()`.
- **No documentation on Clone requirements for map/map_err:** While the trait bounds enforce correctness, the doc comments for `map` and `map_err` could more explicitly explain *why* `Clone` is needed on the other type.
- **No mention of cache chain behavior:** The docs do not explain that `map` creates a new independent cell that holds an `Rc`/`Arc` reference to the original. For long chains of maps, this creates a chain of pointers. Users might want to know this.
- **Missing comparison with `Lazy`:** A brief note explaining when to use `TryLazy` vs. `Lazy<Result<A, E>>` would help users understand the design choice.

## 7. Test Coverage

Tests are thorough:

- Caching semantics for both Ok and Err paths.
- Clone/sharing behavior.
- `catch_unwind` and `catch_unwind_with` for both Rc and Arc variants.
- `From` conversions: `TryThunk`, `TryTrampoline`, `Lazy` (Rc and Arc), `Result` (Rc and Arc).
- `Deferrable` and `SendDeferrable`.
- Panic poisoning.
- Thread safety (10 threads sharing an `ArcTryLazy`).
- QuickCheck property tests for memoization and deferrable transparency.
- `map` and `map_err` for both Ok and Err inputs, both Rc and Arc variants.

No obvious gaps in test coverage for the implemented functionality.

## Summary of Findings

| Category | Assessment |
|----------|------------|
| Design | Well-justified wrapper; parallels TryThunk/TryTrampoline pattern |
| Correctness | No bugs found; Clone bounds and reference semantics are correct |
| HKT integration | `Kind` instance exists but no type class trait impls (major gap vs. TryThunk) |
| Consistency with Lazy | Missing `RefFunctor` equivalent; missing `Semigroup`/`Monoid` |
| Consistency with TryThunk | No `Functor`, `Bifunctor`, `Semimonad`, or other HKT traits |
| Documentation | Good coverage via macros; missing panic docs and design rationale |
| Tests | Comprehensive |
| Key improvement opportunity | Add `RefFunctor` (and potentially `RefBifunctor`) implementations for `TryLazyBrand` |
