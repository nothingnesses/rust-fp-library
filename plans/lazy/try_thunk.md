# Analysis: `try_thunk.rs`

**File:** `fp-library/src/types/try_thunk.rs`
**Role:** `TryThunk<'a, A, E>`, a fallible variant of `Thunk` wrapping `Result<A, E>`.

## Design

`TryThunk<'a, A, E>` is a newtype over `Thunk<'a, Result<A, E>>`. It provides ergonomic combinators for fallible deferred computations: `map`, `map_err`, `bind`, `catch`, `catch_with`, `bimap`, `lift2`, `then`, `catch_unwind`.

### Bifunctor Encoding

Three brands encode different HKT views:

- **`TryThunkBrand`**: 2-arity, `Of<'a, E, A> = TryThunk<'a, A, E>`. Implements `Bifunctor`, `Bifoldable`.
- **`TryThunkErrAppliedBrand<E>`**: Error fixed, polymorphic over success. Full Functor/Pointed/Semiapplicative/Semimonad/MonadRec/Foldable/FunctorWithIndex/FoldableWithIndex tower.
- **`TryThunkOkAppliedBrand<A>`**: Success fixed, polymorphic over error. Same trait tower but operating on the error channel.

## Assessment

### Correct decisions

1. **Newtype over `Thunk<'a, Result<A, E>>`.** Zero overhead; delegates storage and evaluation to the base type.
2. **Three-brand encoding.** Comprehensive HKT coverage for both success and error channels.
3. **Bifunctor + partial application.** Standard approach matching how `Result` is encoded in the library.

### Issues

#### 1. Missing inherent `tail_rec_m`

`TrySendThunk` has `tail_rec_m` and `arc_tail_rec_m` as inherent methods. `TryThunk` has none; it relies on `MonadRec for TryThunkErrAppliedBrand<E>`. This means a user with `TryThunk<'a, A, &'a str>` (non-`'static` error) has no way to use `tail_rec_m` because the brand requires `E: 'static`.

**Impact:** Moderate. Creates a gap for non-`'static` error types.

#### 2. `'static` constraint on brand type parameters

`TryThunkErrAppliedBrand<E>` requires `E: 'static`, and `TryThunkOkAppliedBrand<A>` requires `A: 'static`. These are inherent limitations of the HKT brand pattern but restrict usage with borrowed types in generic contexts.

**Impact:** Moderate. Prevents HKT-polymorphic code with borrowed error/success types.

#### 3. `TryThunkOkAppliedBrand` Lift/Semiapplicative uses "fail-last" semantics

`Lift::lift2` for the error channel evaluates both thunks and returns `Ok` if either succeeds. `Semimonad::bind` short-circuits on the first `Ok`. This means `Lift::lift2` and `bind` have different semantics on the error channel. The documentation warns about this but it is a potential source of confusion and may violate the law `liftA2 f x y = x >>= \a -> fmap (f a) y`.

**Impact:** Moderate. Deliberately chosen semantics, but the law compliance situation needs formal verification.

#### 4. No `Alternative`/`MonadPlus` for the error channel

The error channel brand (`TryThunkOkAppliedBrand`) does not implement `Alternative` or `MonadPlus`, which could provide useful choice/recovery semantics.

**Impact:** Low.

## Strengths

- Clean newtype wrapper with zero overhead.
- Comprehensive trait tower on both success and error channels.
- Thorough QuickCheck law verification.
- Well-documented limitations.
