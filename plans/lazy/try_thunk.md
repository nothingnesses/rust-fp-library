# TryThunk Analysis

## Overview

`TryThunk<'a, A, E>` is a deferred, non-memoized, fallible computation. It is a newtype wrapper around `Thunk<'a, Result<A, E>>`, providing ergonomic combinators for error handling while delegating core mechanics to `Thunk`. It participates in HKT via three brands:

- `TryThunkBrand` (bifunctor: polymorphic over both `E` and `A`).
- `TryThunkErrAppliedBrand<E>` (functor/monad over the success channel, `E` fixed).
- `TryThunkOkAppliedBrand<A>` (functor/monad over the error channel, `A` fixed).

## 1. Wrapping Thunk vs Dedicated Implementation

The wrapper approach is sound. `TryThunk` needs exactly one `Box<dyn FnOnce() -> Result<A, E>>`, which is what `Thunk<'a, Result<A, E>>` provides. A dedicated implementation would duplicate the boxing logic with no benefit. The delegation is clean (e.g., `self.0.map(...)`, `self.0.bind(...)`) and adds negligible overhead.

The only concern: `Thunk`'s inherent `bind` accepts `FnOnce`, but `TryThunk::bind` (lines 240-248) also accepts `FnOnce`. This is correct since `TryThunk` calls `self.0.bind(...)` with a single-use closure that wraps the `Result` matching. The delegation works because `Thunk::bind` is `FnOnce`-based for inherent methods.

**Verdict:** The wrapper approach is appropriate and well-executed.

## 2. HKT Trait Implementations

### 2.1 TryThunkErrAppliedBrand (Functor/Monad over Ok)

All implementations are correct:

- **Functor:** Delegates to `fa.map(func)`, which maps over the `Ok` value only. Correct.
- **Pointed:** `TryThunk::ok(a)` wraps in `Ok`. Correct.
- **Semimonad:** `ma.bind(func)` short-circuits on `Err`. Correct.
- **Semiapplicative:** Uses `ff.bind(|f| fa.map(|a| f(a)))`, sequential applicative. Correct.
- **Lift:** Uses `fa.bind(|a| fb.map(|b| func(a, b)))`, sequential lift. Correct.
- **MonadRec:** Implements a loop that evaluates the step function, breaking on `Ok(Step::Done(_))` or `Err(_)`. Correct.
- **Foldable:** Evaluates the thunk, folds on `Ok`, returns initial on `Err`. Correct.

### 2.2 TryThunkOkAppliedBrand (Functor/Monad over Err)

This is the more unusual set: it treats `Err` as the "value" channel.

- **Functor:** `fa.map_err(func)`. Correct.
- **Pointed:** `TryThunk::err(e)`. Correct, wraps in `Err`.
- **Semimonad:** On `Err(e)`, calls `func(e).evaluate()`; on `Ok(a)`, passes through. Correct.
- **Semiapplicative:** Both `ff` and `fa` are evaluated; applies only when both are `Err`. When either is `Ok`, short-circuits to `Ok`. Correct (mirrors `Either`'s right-biased behavior on the left channel).
- **Lift:** Same pattern: combines when both are `Err`, short-circuits on `Ok`. Correct.
- **MonadRec:** Loop on `Err(Step::Loop(...))`, break on `Err(Step::Done(...))`, short-circuit on `Ok`. Correct.
- **Foldable:** Folds on `Err`, returns initial on `Ok`. Correct.

### 2.3 TryThunkBrand (Bifunctor/Bifoldable)

- **Bifunctor `bimap`:** Applies `f` to `Err` and `g` to `Ok`. The parameter ordering in the signature is `(f, g, p)` where `f` maps the first type parameter (error) and `g` maps the second (success). This follows the `Kind::Of<'a, E, A>` ordering where `E` comes first, matching `Either`/`Result` conventions. Correct.

- **Bifoldable:** `bi_fold_right`, `bi_fold_left`, and `bi_fold_map` all correctly dispatch `f` for `Err` and `g` for `Ok`. Correct.

### 2.4 'static Constraint on Brand Type Parameters

All `impl_kind!` blocks and trait implementations require `E: 'static` (for `TryThunkErrAppliedBrand<E>`) or `A: 'static` (for `TryThunkOkAppliedBrand<A>`). This is a limitation: while `TryThunk<'a, A, E>` supports non-`'static` types for both `A` and `E`, the HKT brands cannot be used with borrowed error/success types in the "fixed" position.

This is a known constraint of the Brand pattern: the brand type itself must be `'static` to work as a type-level witness. The concrete `TryThunk` methods (`map`, `bind`, `map_err`, etc.) still work with borrowed types, so the limitation only affects generic HKT code.

**Verdict:** Acceptable, consistent with how other brands in the library handle this.

### 2.5 Missing `Evaluable` Implementation

`Thunk` implements `Evaluable` for `ThunkBrand`, but neither `TryThunkErrAppliedBrand<E>` nor `TryThunkOkAppliedBrand<A>` implements `Evaluable`. This makes sense: `Evaluable` produces an owned `A`, but `TryThunk::evaluate` produces `Result<A, E>`, not a plain `A`. The `Evaluable` trait's signature does not accommodate fallible evaluation. This is not a flaw; it is a design mismatch that correctly avoids an incorrect implementation.

### 2.6 Missing `WithIndex`, `FunctorWithIndex`, `FoldableWithIndex`

`Thunk` implements `WithIndex` (with `Index = ()`), `FunctorWithIndex`, and `FoldableWithIndex`. `TryThunk` does not implement any of these. Since `TryThunk` is conceptually a single-element container (like `Thunk`), these could be implemented with `Index = ()` for both `TryThunkErrAppliedBrand<E>` and `TryThunkOkAppliedBrand<A>`. This is a gap compared to `Thunk`.

## 3. Inherent Methods

### 3.1 Correctness

All inherent methods are correct:

- `new`, `pure`, `ok`, `err`: Simple constructors; delegate to `Thunk::new`/`Thunk::pure`.
- `defer`: Flattens a `TryThunk`-returning closure via `Thunk::defer`.
- `bind`: Short-circuits on `Err`, chains on `Ok`. Uses `Thunk::bind` internally.
- `map`: Maps the `Ok` value via `Result::map`.
- `map_err`: Maps the `Err` value via `Result::map_err`.
- `catch`: Recovers from `Err` by calling a handler, passes through `Ok`.
- `evaluate`: Delegates to `self.0.evaluate()`.
- `lift2`, `then`: Implemented via `bind`/`map`, correct.
- `memoize`, `memoize_arc`: Conversion to lazy variants, correct.

### 3.2 `pure` vs `ok` Redundancy

`TryThunk::pure` and `TryThunk::ok` are identical in behavior (both produce `Thunk::pure(Ok(a))`). `pure` aligns with the Pointed typeclass naming; `ok` provides a domain-specific alias. This is intentional but should perhaps be documented more explicitly (e.g., "`pure` is an alias for `ok`" or vice versa).

### 3.3 `catch` Error Type Inflexibility

`catch` requires the recovery function to return `TryThunk<'a, A, E>` (same error type). This prevents changing the error type during recovery. A `catch_with` variant that allows `E -> TryThunk<'a, A, E2>` would be more flexible, though it would change the type and could not return `Self`. This is a minor ergonomic limitation.

### 3.4 Missing Combinators

Compared to `Result`'s standard library API, the following combinators are absent:

- `and_then` (equivalent to `bind`, so not needed).
- `or_else` (equivalent to `catch`, so not needed).
- `unwrap_or` / `unwrap_or_else`: Evaluate and extract, returning a default on error. These would be convenient.
- `flatten`: For `TryThunk<'a, TryThunk<'a, A, E>, E>` to `TryThunk<'a, A, E>`. Available via `bind(id)` but a named method would improve discoverability.
- `map_both` / `bimap`: An inherent `bimap` method (not just the HKT `Bifunctor` impl) would be ergonomic. `TrySendThunk` has an inherent `bimap`; `TryThunk` does not.

## 4. Semigroup/Monoid Implementation

### 4.1 Eagerness of `append`

`Semigroup::append` (line 1154) evaluates both thunks inside a new `Thunk::new` closure, which is correct (evaluation is deferred). However, the pattern `match (a.evaluate(), b.evaluate())` eagerly evaluates `b` even if `a` fails. A short-circuiting version would skip evaluating `b` on error:

```rust
fn append(a: Self, b: Self) -> Self {
    TryThunk::new(move || {
        let a_val = a.evaluate()?;
        let b_val = b.evaluate()?;
        Ok(Semigroup::append(a_val, b_val))
    })
}
```

This would be more efficient and consistent with the short-circuiting semantics used elsewhere (`bind`, `lift2`, `then`). The current implementation evaluates `b` unnecessarily when `a` fails. This is arguably a bug, as it violates the short-circuit convention used by every other combinator on this type. `TrySendThunk` has the same issue.

### 4.2 Error Accumulation

`Semigroup::append` discards the second error when both fail (the `(Err(e), _)` arm matches first). This means `append(err(e1), err(e2))` yields `Err(e1)`. This is consistent with `Result`'s convention but means there is no error accumulation. This is acceptable for a monad-based type (monads are inherently sequential/short-circuiting), but worth noting for users who might expect `Semigroup` on errors.

## 5. Conversions

### 5.1 From Implementations

All `From` implementations are correct:

- `From<Lazy<'a, A, Config>>`: Clones the memoized value and wraps in `Ok`. Requires `A: Clone`.
- `From<TryLazy<'a, A, E, Config>>`: Clones both `A` and `E`. Requires `A: Clone + E: Clone`.
- `From<Thunk<'a, A>>`: Wraps the infallible thunk result in `Ok`. Clean, no cloning.
- `From<Result<A, E>>`: Wraps the result in a pure thunk. Clean.
- `From<TryTrampoline<A, E>>`: Wraps trampoline evaluation in a thunk. Requires `'static`.

### 5.2 Missing Conversions

- **`From<TrySendThunk>`:** Missing. A `TrySendThunk<'a, A, E>` (which is `Send`) could trivially be converted to a `TryThunk<'a, A, E>` (which is not `Send`), since dropping the `Send` bound is always safe. This would be a natural widening conversion.
- **`Into<Thunk<'a, Result<A, E>>>`:** No way to unwrap back to the inner `Thunk`. Adding `into_inner` or `From<TryThunk> for Thunk<Result<A, E>>` would allow interop with the infallible `Thunk` API when treating the `Result` as a plain value.

## 6. Documentation

### 6.1 Quality

Documentation is thorough. Every public method has:
- Description.
- Type parameter documentation.
- Parameter documentation.
- Return value documentation.
- Working examples with assertions.

### 6.2 Duplicated Stack Safety Section

The struct-level doc comment contains the "Stack Safety" section twice: once at lines 56-61 and again at lines 92-96. The content is nearly identical. One should be removed.

### 6.3 `OkAppliedBrand` Functor Example Confusion

The doc example for `Functor for TryThunkOkAppliedBrand<A>` (line 1467-1470) is potentially confusing:

```rust
let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
let mapped = map::<TryThunkOkAppliedBrand<i32>, _, _>(|x| x * 2, try_thunk);
assert_eq!(mapped.evaluate(), Err(20));
```

This is technically correct (the brand treats `Err` as the value channel, so `pure` puts `10` in `Err` and `map` transforms the error), but the naming is counterintuitive. A reader unfamiliar with the dual-channel HKT encoding might think `map` over "Ok applied" would map the `Ok` value. A brief explanatory comment in the doc example would help.

## 7. Edge Cases

### 7.1 Panic Safety in `catch_unwind`

`catch_unwind` uses `std::panic::catch_unwind`, which requires the closure to be `UnwindSafe`. This is correctly enforced by the trait bound. The `catch_unwind` convenience method hardcodes `E = String`, which is reasonable for quick prototyping but limits composability. The `catch_unwind_with` variant handles arbitrary error types well.

### 7.2 Double Evaluation

Since `TryThunk` consumes `self` on `evaluate`, double evaluation is prevented at the type level. This is a strength of the design.

### 7.3 Side Effects in `Semigroup::append`

If the thunk closures have side effects, `append` will execute both (as noted in 4.1). For a type that is explicitly about deferred side-effectful computation, this could be surprising when the first thunk fails.

## 8. Comparison with `TrySendThunk`

`TrySendThunk` mirrors `TryThunk` closely but:
- Cannot implement HKT traits (because `Functor`/`Semimonad` signatures do not require `Send` on closures).
- Provides inherent `bimap` method; `TryThunk` does not.
- Has the same `Semigroup::append` eagerness issue.

The two types are structurally identical except for the `Send` bound, which is the correct factoring.

## 9. Summary of Issues

### Bugs / Correctness Issues

1. **`Semigroup::append` does not short-circuit:** Evaluates the second thunk even when the first fails. Inconsistent with every other combinator on the type.

### Documentation Issues

2. **Duplicated "Stack Safety" section** in the struct doc comment (lines 56-61 and 92-96).
3. **`OkAppliedBrand` examples could use explanatory comments** to clarify the dual-channel encoding.

### Missing Implementations

4. **No `WithIndex` / `FunctorWithIndex` / `FoldableWithIndex`** for either applied brand (would be trivial with `Index = ()`).
5. **No inherent `bimap` method**, unlike `TrySendThunk`.
6. **No `From<TrySendThunk>` conversion** (natural widening).
7. **No `into_inner()` or equivalent** to unwrap back to `Thunk<'a, Result<A, E>>`.

### Minor Ergonomic Gaps

8. **`pure` and `ok` redundancy** is undocumented.
9. **`catch` cannot change the error type.**
10. **Missing convenience combinators:** `unwrap_or_else`, `flatten`.
