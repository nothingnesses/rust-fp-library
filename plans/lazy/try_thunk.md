# TryThunk Analysis

## Overview

`TryThunk<'a, A, E>` is a deferred, non-memoized fallible computation. It is a newtype wrapper around `Thunk<'a, Result<A, E>>` that provides ergonomic combinators for error handling and full higher-kinded type support across three separate brands.

**File:** `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/try_thunk.rs` (3086 lines).

**Definition:**

```rust
pub struct TryThunk<'a, A, E>(Thunk<'a, Result<A, E>>);
```

## 1. Type Design

### Wrapping `Thunk<Result<A, E>>` is the Right Choice

The newtype-over-composition approach is sound for several reasons:

- It reuses `Thunk`'s box-allocated `FnOnce` closure machinery rather than duplicating it.
- It maintains lifetime polymorphism (`'a`) from `Thunk`, allowing borrowed data in closures.
- The newtype boundary provides a place to define monadic `bind` that short-circuits on `Err`, which a raw `Thunk<Result<A, E>>` cannot express through `Thunk`'s own `bind`.
- `evaluate` returns `Result<A, E>` directly, making it natural to use with `?`.

### Alternatives Considered

- **Enum with `Ok`/`Err`/`Deferred` variants:** Would allow inspecting whether a value is already resolved without evaluating. However, this would break the invariant that the computation is always deferred, complicate the API surface, and lose the clean delegation to `Thunk`.
- **Trait-based abstraction (e.g., `Fallible<F>` parameterized over an effect):** Too abstract for a concrete type. The hierarchy already provides `TryTrampoline`, `TrySendThunk`, `TryLazy` variants for different trade-offs.
- **Using `Result` directly in the `Kind` system:** This is already done via `ResultBrand`/`ResultErrAppliedBrand`. `TryThunk` adds the deferred computation aspect that `Result` alone does not provide.

## 2. HKT Support

### Three-Brand Strategy

`TryThunk<'a, A, E>` has two type parameters (besides the lifetime), so it needs multiple brands to participate in different HKT contexts:

| Brand | Kind Mapping | Role |
|-------|-------------|------|
| `TryThunkBrand` | `Of<'a, E, A> = TryThunk<'a, A, E>` | Bifunctor (2-ary kind). Note: parameters are `(E, A)`, not `(A, E)`, following the Haskell `Either e a` convention. |
| `TryThunkErrAppliedBrand<E>` | `Of<'a, A> = TryThunk<'a, A, E>` | Functor/Monad over `Ok` (the success channel). `E` is fixed. |
| `TryThunkOkAppliedBrand<A>` | `Of<'a, E> = TryThunk<'a, A, E>` | Functor/Monad over `Err` (the error channel). `A` is fixed. |

This mirrors the `ResultBrand`/`ResultErrAppliedBrand<E>`/`ResultOkAppliedBrand<A>` pattern already in the library, providing consistency.

### `'static` Constraint on Partially-Applied Brands

Both `TryThunkErrAppliedBrand<E>` and `TryThunkOkAppliedBrand<A>` require their type parameter to be `'static`. This is documented in `brands.rs` and is an inherent limitation of the Brand pattern: the `Kind` trait's `Of<'a, A>` introduces its own lifetime `'a`, so any type parameter baked into the brand struct must outlive all possible `'a`, which means `'static`. This prevents use with borrowed error types (e.g., `TryThunkErrAppliedBrand<&str>`) in HKT contexts. It is a real ergonomic limitation, but there is no known fix within the current Brand encoding.

### Parameter Ordering in `TryThunkBrand`

The `impl_kind!` maps `Of<'a, E, A>` to `TryThunk<'a, A, E>`, placing `E` first in the kind and `A` second. This follows the convention from Haskell's `Either e a` where the success type is the last parameter, matching how `Bifunctor::bimap` expects `(f_for_first, g_for_second)`. The code documents this explicitly with a comment.

## 3. Type Class Implementations

### For `TryThunkErrAppliedBrand<E>` (Functor over Success)

| Trait | Status | Notes |
|-------|--------|-------|
| `Functor` | Implemented | Maps over success value; errors pass through. |
| `Pointed` | Implemented | `pure(a)` produces `TryThunk::ok(a)`. |
| `Lift` | Implemented | `lift2` combines two successes via `bind`. |
| `ApplyFirst` | Implemented | Blanket (default). |
| `ApplySecond` | Implemented | Blanket (default). |
| `Semiapplicative` | Implemented | `apply` via `bind` then `map`. |
| `Semimonad` | Implemented | `bind` short-circuits on `Err`. |
| `MonadRec` | Implemented | Tail-recursive loop; breaks on `Err(e)` or `Ok(Step::Done(b))`. |
| `Foldable` | Implemented | Folds success value; returns initial on `Err`. |
| `WithIndex` | Implemented | `Index = ()`. |
| `FunctorWithIndex` | Implemented | Maps with `()` index. |
| `FoldableWithIndex` | Implemented | Fold-maps with `()` index. |
| `Evaluable` | **Missing** | See section 8. |
| `Traversable` | Not implemented | Documented as impossible (same `FnOnce`/`Clone` conflict as `Thunk`). |

### For `TryThunkOkAppliedBrand<A>` (Functor over Error)

| Trait | Status | Notes |
|-------|--------|-------|
| `Functor` | Implemented | Maps over error value; successes pass through. |
| `Pointed` | Implemented | `pure(e)` produces `TryThunk::err(e)`. |
| `Lift` | Implemented | Uses fail-last semantics (evaluates both sides). |
| `ApplyFirst` | Implemented | Blanket (default). |
| `ApplySecond` | Implemented | Blanket (default). |
| `Semiapplicative` | Implemented | Uses fail-last semantics. |
| `Semimonad` | Implemented | `bind` over error; short-circuits on `Ok`. |
| `MonadRec` | Implemented | Tail-recursive loop over error channel. |
| `Foldable` | Implemented | Folds error value; returns initial on `Ok`. |
| `WithIndex` | Implemented | `Index = ()`. |
| `FunctorWithIndex` | Implemented | Maps error with `()` index. |
| `FoldableWithIndex` | Implemented | Fold-maps error with `()` index. |

### For `TryThunkBrand` (Bifunctor)

| Trait | Status | Notes |
|-------|--------|-------|
| `Bifunctor` | Implemented | Maps both success and error. |
| `Bifoldable` | Implemented | Folds from either channel. |
| `Bitraversable` | **Missing** | Same `Clone`/`FnOnce` issue as `Traversable`. See section 8. |

### Instance-Level Traits

| Trait | Status | Notes |
|-------|--------|-------|
| `Deferrable<'a>` | Implemented | `defer(f)` creates a lazily-produced `TryThunk`. |
| `Semigroup` | Implemented | `append` evaluates both, combines successes, short-circuits on first error. |
| `Monoid` | Implemented | `empty()` produces `Ok(A::empty())`. |
| `Debug` | Implemented | Prints `"TryThunk(<unevaluated>)"` without forcing. |

### Correctness Assessment

The implementations are correct:

- **Monadic `bind` (success channel):** Properly short-circuits on `Err` by matching the `Result` from the inner `Thunk` and only calling the continuation on `Ok`. This is the standard `Result` monad behavior, lifted into the deferred context.
- **Monadic `bind` (error channel, via `OkAppliedBrand`):** Correctly short-circuits on `Ok`, calling the continuation only on `Err`.
- **`MonadRec` (both channels):** Both implementations use an iterative loop with `match`, breaking on the terminal case or the short-circuit case. This provides O(1) stack usage for the recursion itself (though individual step evaluations can still stack-overflow if they build deep `bind` chains internally).
- **`Semiapplicative` (error channel):** Uses fail-last semantics (evaluates both sides before inspecting), which is the correct behavior for an applicative on the error channel (it collects all errors rather than short-circuiting). This is explicitly documented.
- **`Bifunctor`/`Bifoldable`:** Correctly dispatch to the appropriate branch based on `Ok`/`Err`.
- **`Semigroup`:** Uses `?` operator for short-circuiting, which is idiomatic and correct.

## 4. Error Handling

### Ergonomic API

The inherent methods provide a clean error-handling surface:

- `TryThunk::ok(a)` / `TryThunk::err(e)` for constructing success/failure.
- `map` / `map_err` for transforming each channel independently.
- `bimap` for transforming both simultaneously.
- `bind` for monadic chaining with short-circuit on error.
- `catch` for error recovery (same error type).
- `catch_with` for error recovery with error type change.
- `catch_unwind` / `catch_unwind_with` for converting panics to errors.
- `evaluate` returns `Result<A, E>` directly, enabling use with `?`.

### `catch` vs `catch_with`

The distinction between `catch` (same error type) and `catch_with` (different error type) is well-designed:

- `catch` preserves the error type, allowing repeated recovery attempts.
- `catch_with` allows changing the error type during recovery, which is useful for layered error handling.

Both are implemented correctly: `catch` uses `Thunk::bind` internally (keeping the result wrapped in a `Thunk`), while `catch_with` creates a new `Thunk::new` that evaluates and pattern-matches.

### Panic Handling

`catch_unwind` and `catch_unwind_with` are thoughtful additions:

- `catch_unwind_with` is generic over the handler and the error type.
- `catch_unwind` is a convenience that requires `E = String` and uses a utility function to convert panic payloads.
- Both correctly require `UnwindSafe` on the closure.

## 5. Relationship to `Thunk`

### Code Reuse

`TryThunk` achieves excellent reuse of `Thunk`:

- `new` delegates to `Thunk::new`.
- `pure` delegates to `Thunk::pure(Ok(a))`.
- `evaluate` delegates to `self.0.evaluate()`.
- `map` delegates to `self.0.map(|r| r.map(f))`.
- `bind` delegates to `self.0.bind(|r| match r { ... })`.
- `defer` delegates to `Thunk::defer(move || f().0)`.

The only non-trivial logic is the `Result` matching inside `bind`, `catch`, and `catch_with`. This is minimal and appropriate.

### Duplication

There is effectively no code duplication between `Thunk` and `TryThunk`. The HKT trait implementations for `TryThunkErrAppliedBrand` necessarily duplicate the *structure* of `ThunkBrand`'s implementations (since they must implement the same traits), but the *logic* differs because of error handling. This is unavoidable given Rust's trait system, where each brand needs its own impl block.

The `TryThunkOkAppliedBrand` implementations are genuinely new logic (error-channel monad) with no counterpart in `Thunk`.

### Missing `From<TrySendThunk>` Conversion

`Thunk` has `From<SendThunk>` to erase the `Send` bound. However, `TryThunk` has no `From<TrySendThunk>` conversion. This is an omission; the conversion should be straightforward since `TrySendThunk` wraps `SendThunk<'a, Result<A, E>>`, and `SendThunk` can be converted to `Thunk` via the existing `From` impl.

## 6. Relationship to `Result`

### Proper Leveraging of `Result`

The implementation leverages `Result`'s combinators effectively:

- `map` uses `Result::map`.
- `map_err` uses `Result::map_err`.
- `Semigroup::append` uses `?` for short-circuit evaluation.
- `From<Result<A, E>>` wraps the result in `Thunk::pure`.

### Conversions

The type provides a rich set of conversions:

| From | To | Notes |
|------|----|-------|
| `Thunk<'a, A>` | `TryThunk<'a, A, E>` | Wraps in `Ok`. |
| `Result<A, E>` | `TryThunk<'a, A, E>` | Pure/immediate. |
| `Lazy<'a, A, Config>` | `TryThunk<'a, A, E>` | Clones memoized value, wraps in `Ok`. |
| `TryLazy<'a, A, E, Config>` | `TryThunk<'a, A, E>` | Clones memoized result. |
| `TryTrampoline<A, E>` | `TryThunk<'static, A, E>` | Evaluates trampoline when forced. |
| `TryThunk<'static, A, E>` | `TryTrampoline<A, E>` | Defined in `try_trampoline.rs`. |

The `into_inner` method allows extracting the underlying `Thunk<'a, Result<A, E>>` for interop with raw `Thunk`-based code.

## 7. Documentation Quality

### Strengths

- The type-level documentation is thorough, covering HKT representation, when to use, algebraic properties, stack safety, and limitations.
- Every method has `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` attributes.
- The `OkAppliedBrand` `Lift`/`Semiapplicative` implementations explicitly document their "fail-last" evaluation strategy and contrast it with the "fail-fast" monadic `bind` path.
- The `TryThunkBrand` `impl_kind!` includes a comment explaining the `(E, A)` parameter ordering convention.

### Weaknesses

- The inherent `bind` method documentation does not mention that the HKT-level `Semimonad::bind` requires `Fn` while the inherent method accepts `FnOnce`, unlike `Thunk` which documents this distinction explicitly.
- The `catch_with` method's implementation differs from `catch`: it uses `Thunk::new(move || match self.evaluate() { ... })` rather than `self.0.bind(...)`. This is because `catch_with` changes the error type, which requires constructing a new closure. The difference is correct but not documented.

## 8. Issues, Limitations, and Design Flaws

### Issue 1: Missing `Evaluable` Implementation

`Thunk` implements `Evaluable` for `ThunkBrand`, but neither `TryThunkErrAppliedBrand<E>` nor `TryThunkOkAppliedBrand<A>` implements `Evaluable`. This is notable because:

- `TryThunkErrAppliedBrand<E>` could implement `Evaluable` where `evaluate` returns the `Ok` value (panicking on `Err`), but this would be unsound for a fallible type.
- The `Evaluable` trait's signature is `fn evaluate<'a, A: 'a>(fa: ...) -> A`, which extracts a bare `A`, not a `Result<A, E>`. For a fallible computation, this does not fit cleanly.

This is arguably correct as-is. `Evaluable` means "you can always extract a value," which is not true for a computation that may fail. The omission is intentional, not a bug.

### Issue 2: Missing `From<TrySendThunk>` Conversion

As noted in section 5, there is no `From<TrySendThunk<'a, A, E>> for TryThunk<'a, A, E>`. Given that `Thunk` has `From<SendThunk>`, this is an inconsistency.

### Issue 3: No `and_then` Alias

Rust's `Result` uses `and_then` for monadic bind. `TryThunk` uses `bind`. While `bind` is the correct FP term and matches the library's conventions, an `and_then` alias would improve discoverability for Rust users coming from the standard library.

### Issue 4: `Semigroup` Error Semantics

The `Semigroup` implementation short-circuits on the first error:

```rust
fn append(a: Self, b: Self) -> Self {
    TryThunk::new(move || {
        let a_val = a.evaluate()?;
        let b_val = b.evaluate()?;
        Ok(Semigroup::append(a_val, b_val))
    })
}
```

This means `append(err1, err2)` returns `err1`, discarding `err2`. An alternative would be to accumulate errors (using `Semigroup` on `E`), but that would require `E: Semigroup` and change the semantics. The current behavior is the standard choice and matches `Result`'s behavior.

### Issue 5: `Bitraversable` Not Implemented

The `Bitraversable` trait requires the same `Clone` bounds that prevent `Traversable` from being implemented. This is documented for `Traversable` at the type level but not explicitly called out as a missing `Bitraversable`.

### Issue 6: `'static` Bound on Brand Parameters Limits HKT Generality

The `E: 'static` requirement on `TryThunkErrAppliedBrand<E>` means you cannot use HKT functions (like `map`, `bind`, `pure` from the free function API) with borrowed error types. The inherent methods (`.map()`, `.bind()`, `.pure()`) still work with any lifetime, so this is only a limitation for generic, brand-parameterized code. This is a known, documented limitation of the Brand pattern.

### Limitation: Not Stack-Safe

Like `Thunk`, `TryThunk`'s `bind` chains grow the stack. `MonadRec::tail_rec_m` provides an escape hatch for structured recursion, but arbitrary `bind` chains remain unsafe. The documentation correctly directs users to `TryTrampoline` for stack-safe fallible recursion.

## 9. Alternatives and Improvements

### Potential Improvements

1. **Add `From<TrySendThunk>` conversion.** This is a straightforward addition:
   ```rust
   impl<'a, A: 'a, E: 'a> From<TrySendThunk<'a, A, E>> for TryThunk<'a, A, E> {
       fn from(t: TrySendThunk<'a, A, E>) -> Self {
           TryThunk(Thunk::from(t.into_inner()))
       }
   }
   ```
   (Assuming `TrySendThunk` has an `into_inner` that returns `SendThunk<'a, Result<A, E>>`.)

2. **Document the `FnOnce` vs `Fn` distinction on inherent `bind`.** Match `Thunk`'s documentation style.

3. **Consider an `and_then` alias** for the inherent `bind` method to improve discoverability.

4. **Add `unwrap` / `unwrap_or` / `unwrap_or_else` convenience methods.** These would evaluate and then unwrap the `Result`, mirroring `Result`'s API. This is optional; users can already call `.evaluate().unwrap()`.

5. **Consider `flatten` for `TryThunk<'a, TryThunk<'a, A, E>, E>`.** While `bind(id)` achieves this, a dedicated method could be clearer.

### Design Alternatives Not Worth Pursuing

- **Generic error handling via a trait (MonadError-style):** The library does not have a `MonadError` trait. Adding one is a separate concern and not specific to `TryThunk`.
- **Accumulating errors in `Semiapplicative`:** The success-channel `Semiapplicative` uses `bind` semantics (fail-fast), which is standard. The error-channel `Semiapplicative` already uses fail-last. Adding error accumulation to the success channel would require a `Validation`-like type, which is a different abstraction.
- **Replacing the newtype with a trait-based approach:** The current design is simpler and more performant. A trait-based approach would add indirection without clear benefit.

## Summary

`TryThunk` is a well-designed, thoroughly-implemented fallible lazy computation type. Its three-brand HKT strategy provides comprehensive type class coverage on both the success and error channels, plus bifunctor operations. The implementation correctly reuses `Thunk` internally with minimal duplication. The main gaps are the missing `From<TrySendThunk>` conversion and the `FnOnce`/`Fn` documentation asymmetry with `Thunk`. The test suite is comprehensive, covering unit tests, property-based law verification (Functor, Monad, Bifunctor, Semigroup/Monoid laws), error propagation, panic catching, memoization conversions, and thread safety.
