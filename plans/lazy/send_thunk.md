# SendThunk Analysis

File: `fp-library/src/types/send_thunk.rs` (544 lines including tests)

## Overview

`SendThunk<'a, A>` wraps `Box<dyn FnOnce() -> A + Send + 'a>`, providing a thread-safe counterpart to `Thunk<'a, A>` (which wraps `Box<dyn FnOnce() -> A + 'a>`). The sole difference is the `Send` bound on the inner closure, enabling cross-thread transfer of deferred computations.

## 1. Design

### Trade-offs

The core trade-off is sound and well-motivated: adding `Send` to the closure enables genuine lazy conversion to `ArcLazy` (line 360-362), whereas `Thunk::into_arc_lazy` must evaluate eagerly (thunk.rs line 330). This is the primary reason `SendThunk` exists, and the design delivers on that promise.

The decision to forgo HKT trait implementations (`Functor`, `Semimonad`, etc.) is correct. The HKT trait signatures do not carry `Send` bounds on their closure parameters, so implementing them would require accepting non-`Send` closures and breaking the `Send` invariant on the composed result. The module-level doc (lines 7-9) and the struct-level doc (lines 56-66) both explain this clearly.

### Relationship to PureScript

PureScript's `Data.Lazy` is a memoized type (comparable to `RcLazy`/`ArcLazy`), not a non-memoized thunk. `SendThunk` has no direct PureScript counterpart; it is a Rust-specific adaptation that exists because Rust's type system requires explicit `Send` bounds. This is a reasonable design choice for the library.

## 2. Implementation Quality

### Correctness

All inherent methods are straightforward and correct:

- `new` (line 108): Boxes a `Send` closure. Sound.
- `pure` (line 128): Wraps a value in a closure. The `A: Send` bound (line 130) is necessary and correctly placed.
- `defer` (line 150): Flattens a `SendThunk<SendThunk<A>>` by evaluating the outer layer. The `Send` bound on `f` is correct.
- `bind` (line 175-184): Standard monadic bind via closure composition. Correct.
- `map` (line 204-209): Standard functor map. Correct.
- `evaluate` (line 225-227): Trivial unwrap. Correct.
- `into_arc_lazy` (line 360-362): Passes the `Send` closure directly into `ArcLazy::new` via `ArcLazyConfig::lazy_new`. This is the key advantage over `Thunk::into_arc_lazy`, and it works because the closure is already `Send`.

### `tail_rec_m` (lines 267-282)

The implementation is a standard iterative loop, which is stack-safe by construction. The `Clone + Send` bounds on `f` are necessary (each iteration needs a fresh call to `f`, which requires `Fn`, and the resulting `SendThunk` closure must be `Send`). Correct.

### `arc_tail_rec_m` (lines 326-338)

Wraps the closure in `Arc` to provide `Clone` for non-cloneable closures. The additional `Sync` bound (line 327) is required because `Arc<T>: Clone` requires `T: Send + Sync`. This is correct.

### `From<Thunk<'a, A>>` (lines 366-388)

Eagerly evaluates the `Thunk` and wraps the result in `SendThunk::pure`. This is the only sound approach because `Thunk`'s inner closure is not `Send`. The `A: Send` bound is correct. The documentation (lines 371-373) accurately explains the eager evaluation.

### Edge Cases

No bugs found. The implementation is minimal and each method does exactly one thing.

## 3. Type Class Instances

### Implemented

| Instance | Lines | Correct? | Notes |
|----------|-------|----------|-------|
| `Deferrable<'a>` | 400-427 | Mostly | See issue below |
| `SendDeferrable<'a>` | 433-457 | Yes | Delegates to `SendThunk::defer` |
| `Semigroup` | 463-491 | Yes | Lifts `Semigroup::append` into deferred context |
| `Monoid` | 497-517 | Yes | Lifts `Monoid::empty` into deferred context |
| `Debug` | 524-542 | Yes | Static string, does not force evaluation |
| `Kind` (via `impl_kind!`) | 390-394 | Yes | Maps `SendThunkBrand` to `SendThunk<'a, A>` |

### Deferrable Issue (lines 422-426)

The `Deferrable::defer` implementation evaluates the thunk eagerly:

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self
where
    Self: Sized, {
    f()
}
```

This satisfies the transparency law ("defer(|| x) is equivalent to x"), but it is **not** truly deferred. The `Thunk` version (thunk.rs line 436-440) uses `Thunk::defer(f)`, which wraps the computation in a new closure and delays evaluation.

The eager evaluation here is a deliberate choice: since the `Deferrable` trait does not require `Send` on its closure parameter, and `SendThunk::defer` (the inherent method at line 150) requires `Send`, the trait implementation cannot delegate to the inherent method. The documentation at line 403-404 explains this. This is a correct and well-documented concession.

However, this means that `Deferrable::defer` for `SendThunk` has different performance characteristics than for `Thunk`. Generic code using `Deferrable` will get eager evaluation for `SendThunk` but deferred evaluation for `Thunk`. This is documented but could surprise users.

### `SendDeferrable` Implementation (lines 452-456)

```rust
fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self {
    SendThunk::defer(f)
}
```

This delegates to the inherent `SendThunk::defer`, which calls `SendThunk::new(move || f().evaluate())`. This is truly deferred. The implementation is correct.

However, there is a subtle issue: `send_defer` calls `SendThunk::defer(f)`, and `SendThunk::defer` (line 150-152) calls `SendThunk::new(move || f().evaluate())`. This means the outer thunk `f` is deferred, but when it produces a `SendThunk`, that inner thunk is immediately evaluated. This is the expected "join" semantics for `defer`, equivalent to Thunk's version.

### Missing Instances

Compared to `Thunk`, `SendThunk` is missing the following HKT trait implementations:

- **`Functor` for `SendThunkBrand`**: Cannot be implemented (HKT `map` signature lacks `Send`).
- **`Pointed` for `SendThunkBrand`**: Could potentially be implemented, since `Pointed::pure` takes a value, not a closure. The value just needs to be `Send`. However, implementing `Pointed` without `Functor` would be unusual and might be confusing.
- **`Semimonad`/`Semiapplicative`**: Cannot be implemented (closure parameters lack `Send`).
- **`Foldable` for `SendThunkBrand`**: Could potentially be implemented. `Foldable::fold_right` takes an `Fn(A, B) -> B` which does not need to be `Send` because it is called during evaluation (which happens on a single thread). The fold functions consume the `SendThunk` by evaluating it and then applying the fold function. This would work because the fold function does not need to be stored inside a `Send` closure; it is used immediately after the thunk is evaluated.
- **`Evaluable` for `SendThunkBrand`**: Requires `Functor` as a supertrait, so cannot be implemented.
- **`MonadRec` for `SendThunkBrand`**: Cannot be implemented (requires `Semimonad` or at minimum `Functor`).
- **`WithIndex`/`FunctorWithIndex`/`FoldableWithIndex`**: `FunctorWithIndex` requires `Functor`, so no. `FoldableWithIndex` requires `Foldable`, which could be implemented (see above).

**Recommendation**: Consider implementing `Foldable` and `FoldableWithIndex` for `SendThunkBrand`. The fold functions are not stored inside the thunk; they are applied after evaluation. The `Send` bound on the thunk's closure is not violated because the fold function runs on whatever thread calls `evaluate`. This would give `SendThunk` parity with the fold capabilities of `Thunk`.

### Missing Trait Implementations (Non-HKT)

- **`Eq`/`PartialEq`**: Cannot be implemented without evaluating. `Thunk` does not implement these either. Consistent.
- **`Display`**: Not implemented by `Thunk` either. Could be useful but would require forcing evaluation. Consistent.

## 4. API Surface

### Well-Designed

The API mirrors `Thunk`'s inherent methods closely:

| `Thunk` | `SendThunk` | Notes |
|---------|-------------|-------|
| `new(f)` | `new(f)` | `f` requires `Send` in `SendThunk` |
| `pure(a)` | `pure(a)` | `a` requires `Send` in `SendThunk` |
| `defer(f)` | `defer(f)` | `f` requires `Send` in `SendThunk` |
| `bind(f)` | `bind(f)` | `f` requires `Send` in `SendThunk` |
| `map(f)` | `map(f)` | `f` requires `Send` in `SendThunk` |
| `evaluate()` | `evaluate()` | Identical |
| `into_rc_lazy()` | (missing) | See below |
| `into_arc_lazy()` | `into_arc_lazy()` | `Thunk` evaluates eagerly; `SendThunk` defers |
| `tail_rec_m(f, s)` | `tail_rec_m(f, s)` | Identical structure; `f` requires `Send` |
| (missing) | `arc_tail_rec_m(f, s)` | `SendThunk` adds this for non-Clone closures |

### Missing Methods

- **`into_rc_lazy`**: Not provided. A `SendThunk` could be converted to an `RcLazy` by passing the closure (which is `Send`, and `Send` is not required for `RcLazy`). This conversion would be trivially sound and would allow users to go from thread-safe deferred to single-threaded memoized. Low priority since users can just use `Thunk` if they want `RcLazy`.

- **`apply` / `lift2`**: `Thunk` has these as HKT trait implementations, but `SendThunk` cannot have them at the HKT level. Inherent `apply` and `lift2` methods that accept `Send` closures could be added for ergonomic parity, but the use case is niche. Low priority.

- **`From<SendThunk> for Thunk`**: A `SendThunk` could be trivially converted to a `Thunk` by erasing the `Send` bound on the inner closure (the `Box<dyn FnOnce() -> A + Send>` is a subtype of `Box<dyn FnOnce() -> A>`). This conversion is currently missing. It would be zero-cost and useful for interop.

- **`rc_tail_rec_m`**: `Thunk` has `arc_tail_rec_m` via `MonadRec` HKT. `SendThunk` has its own `arc_tail_rec_m` (line 326). An `rc_tail_rec_m` analog is not needed since `Rc` is not `Send`.

### Unnecessary Methods

None found. The API is lean and purposeful.

## 5. Consistency

### Consistent With Thunk

The implementation closely mirrors `Thunk` in structure, naming, documentation style, and behavior. The `Send` bounds are applied consistently across all methods that store closures.

### Consistent With TrySendThunk

`TrySendThunk` wraps `SendThunk<'a, Result<A, E>>` and follows the same pattern of inherent methods with `Send` bounds. The relationship between `SendThunk`/`TrySendThunk` mirrors `Thunk`/`TryThunk`.

### Minor Inconsistency

- `Thunk`'s `Deferrable` implementation delegates to the inherent `Thunk::defer` (truly deferred), while `SendThunk`'s `Deferrable` evaluates eagerly. This is documented and justified, but the behavioral difference could surprise generic code. Not a bug, but worth keeping in mind.

## 6. Limitations and Issues

### Fundamental Limitations

1. **No HKT traits**: The core limitation. `SendThunk` cannot participate in generic HKT-polymorphic code (e.g., `map::<Brand, _, _>(f, x)` does not work). Users must use inherent methods directly. This is an inherent consequence of Rust's type system and the library's HKT encoding.

2. **Not stack-safe**: Like `Thunk`, `bind` chains grow the stack. `tail_rec_m` provides an escape hatch. This is documented (line 70-71).

3. **Not memoized**: Evaluation re-runs the closure each time (though `evaluate` takes `self`, so in practice it can only be called once). This is by design.

### Potential Issues

1. **`Deferrable` eager evaluation**: As discussed in section 3. Not a bug, but a surprising semantic difference from `Thunk`'s `Deferrable` implementation.

2. **`into_arc_lazy` accesses `Lazy` internals**: Line 361 calls `Lazy(ArcLazyConfig::lazy_new(self.0))`, constructing a `Lazy` directly via its tuple struct field. This creates a coupling to `Lazy`'s internal representation. If `Lazy`'s fields change, this code breaks. `Thunk::into_rc_lazy` (thunk.rs line 303) uses `Lazy::from(self)`, which is more robust. Consider whether `SendThunk` could use a similar conversion trait or a dedicated constructor on `Lazy`.

## 7. Documentation

### Quality

Documentation is thorough and accurate:

- Module-level doc (lines 1-9) concisely explains the purpose and limitations.
- Struct-level doc (lines 40-71) covers HKT representation, trait limitations, stack safety, and comparison with `Thunk`.
- All methods have `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` attributes.
- The `Deferrable` implementation's eager evaluation is documented (line 401-404).
- The `From<Thunk>` conversion's eager evaluation is documented (lines 371-373).

### Missing Documentation

- The struct doc does not include a comparison table like `Thunk` has (thunk.rs lines 59-67). A table comparing `SendThunk` vs `Thunk` would help users choose between them.
- No mention of the algebraic properties (monad laws) like `Thunk` has (thunk.rs lines 69-74). While `SendThunk` cannot implement the HKT `Semimonad` trait, its inherent `bind`/`pure` still satisfy the monad laws.
- `arc_tail_rec_m` does not document why `Sync` is required in addition to `Send` (line 327). This is because `Arc::clone` requires `T: Send + Sync` for the `Arc<T>` to be `Send`.

## Summary of Recommendations

1. **Consider implementing `Foldable` for `SendThunkBrand`**: The fold functions do not need `Send` bounds, so this should be feasible and would improve HKT interop.
2. **Consider adding `From<SendThunk> for Thunk`**: Zero-cost conversion that erases the `Send` bound.
3. **Address `into_arc_lazy` coupling**: The direct construction of `Lazy(ArcLazyConfig::lazy_new(...))` bypasses any abstraction boundary. Consider adding a constructor or `From` impl on `Lazy`/`ArcLazy` that accepts `Box<dyn FnOnce() -> A + Send>`.
4. **Add monad law documentation**: Document that `pure`/`bind`/`map` satisfy the functor and monad laws via inherent methods.
5. **Add comparison table**: Similar to `Thunk`'s table, compare `SendThunk` vs `Thunk` in the struct docs.
6. **Document `Sync` requirement on `arc_tail_rec_m`**: Explain why the closure needs `Sync` in addition to `Send`.
