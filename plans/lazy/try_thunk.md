# TryThunk Analysis

**File:** `fp-library/src/types/try_thunk.rs` (3089 lines)

## 1. Design: Is the Fallible Wrapper Justified?

`TryThunk<'a, A, E>` is a newtype over `Thunk<'a, Result<A, E>>` (line 100-103). This is explicitly documented at line 55: "This is `Thunk<'a, Result<A, E>>` with ergonomic combinators for error handling."

**Value added beyond raw `Thunk<Result<A, E>>`:**

- **Ergonomic API**: `ok`, `err`, `catch`, `catch_with`, `map_err`, `bimap`, `lift2`, `then` are all purpose-built for fallible semantics and would require manual `Result` matching if using raw `Thunk`.
- **Short-circuiting `bind`** (line 239-243): Automatically propagates errors without requiring the user to pattern-match. This is the core monadic behavior that justifies the wrapper.
- **Error recovery**: `catch` (line 311-319) and `catch_with` (line 350-358) provide monadic error handling that has no analog on plain `Thunk`.
- **Panic capture**: `catch_unwind` / `catch_unwind_with` (lines 578-640) bridge Rust's panic mechanism into the `Result` monad.
- **Triple HKT encoding**: `TryThunkBrand` (bifunctor), `TryThunkErrAppliedBrand<E>` (functor/monad over `Ok`), `TryThunkOkAppliedBrand<A>` (functor/monad over `Err`). This is a significant structural addition that raw `Thunk<Result>` cannot provide.
- **Conversions**: `From<Thunk>`, `From<Result>`, `From<TryTrampoline>`, `From<Lazy>`, `From<TryLazy>`, `into_rc_try_lazy`, `into_arc_try_lazy`, `into_inner`. A rich conversion graph that connects the lazy hierarchy.

**Verdict:** The newtype is well-justified. It is a thin, zero-cost wrapper that adds substantial ergonomic and algebraic value. The `into_inner` method (line 599) provides an escape hatch when users need the raw `Thunk<Result>`.

## 2. Implementation Quality

### Correctness

All core operations delegate to `Thunk` correctly:

- `new` (line 128-130): Wraps `FnOnce() -> Result<A, E>` in `Thunk::new`. Correct.
- `pure` (line 148-150): `Thunk::pure(Ok(a))`. Correct.
- `bind` (line 235-243): Pattern-matches the Result, short-circuits on `Err`. Correct.
- `map` (line 263-268): Delegates to `Thunk::map` with `Result::map`. Correct.
- `map_err` (line 288-293): Delegates to `Thunk::map` with `Result::map_err`. Correct.
- `catch` (line 311-319): Uses `Thunk::bind` to match the Result and call the recovery function on `Err`. Correct.
- `catch_with` (line 350-358): Evaluates `self` eagerly inside a new `Thunk::new`, matches, and calls `f(e).evaluate()`. Correct but note the eager evaluation is necessary because `catch_with` changes the error type, so it cannot be deferred through `Thunk::bind` which preserves the `Result` type parameter.
- `bimap` (line 386-395): Maps both arms of the Result. Correct.
- `evaluate` (line 411-413): Delegates to inner `Thunk::evaluate`. Correct.

### Potential Issues

**No bugs found.** The implementation is straightforward delegation to `Thunk` with `Result` plumbing. All operations preserve the invariant that the inner `Thunk` produces a `Result`.

**Minor observation:** `catch_with` (line 354) calls `self.evaluate()` eagerly inside the closure, which means the original thunk is consumed at construction time of the new thunk. This is actually fine because `Thunk::new` wraps the whole thing in a new closure, so the evaluation is still deferred. The `self.evaluate()` only runs when the outer thunk is forced.

## 3. Type Class Instances

### Implemented (via `TryThunkErrAppliedBrand<E>`, functor over `Ok`):

| Type Class | Lines | Correct? |
|---|---|---|
| `Functor` | 816-851 | Yes, delegates to `fa.map(func)`. |
| `Pointed` | 854-882 | Yes, `TryThunk::ok(a)`. |
| `Lift` | 885-930 | Yes, `fa.bind(move \|a\| fb.map(move \|b\| func(a, b)))`. |
| `ApplyFirst` | 933 | Yes (default impl). |
| `ApplySecond` | 936 | Yes (default impl). |
| `Semiapplicative` | 939-984 | Yes, `ff.bind` then `fa.map`. |
| `Semimonad` | 987-1024 | Yes, delegates to `ma.bind(func)`. |
| `MonadRec` | 1027-1078 | Yes, loop with `Ok(Step::Loop)` / `Ok(Step::Done)` / `Err`. |
| `Foldable` | 1081-1210 | Yes, evaluates and folds `Ok`, returns initial on `Err`. |
| `WithIndex` | 1992-1994 | Yes, `Index = ()`. |
| `FunctorWithIndex` | 1997-2031 | Yes, maps with `()` index. |
| `FoldableWithIndex` | 2034-2072 | Yes, folds with `()` index. |
| `Semigroup` | 1217-1250 | Yes, short-circuits on first `Err`, combines `Ok` values. |
| `Monoid` | 1257-1277 | Yes, `Ok(Monoid::empty())`. |
| `Deferrable` | 776-806 | Yes, delegates to `TryThunk::defer`. |

### Implemented (via `TryThunkOkAppliedBrand<A>`, functor over `Err`):

| Type Class | Lines | Correct? |
|---|---|---|
| `Functor` | 1545-1578 | Yes, delegates to `fa.map_err(func)`. |
| `Pointed` | 1581-1609 | Yes, `TryThunk::err(e)`. |
| `Lift` | 1612-1669 | See below. |
| `ApplyFirst` | 1672 | Yes. |
| `ApplySecond` | 1675 | Yes. |
| `Semiapplicative` | 1678-1730 | See below. |
| `Semimonad` | 1733-1773 | Yes, bind over error channel, short-circuits on `Ok`. |
| `MonadRec` | 1776-1831 | Yes, loop over `Err(Step::Loop)` / `Err(Step::Done)` / `Ok`. |
| `Foldable` | 1834-1963 | Yes, folds `Err` values, returns initial on `Ok`. |
| `WithIndex` | 2075-2077 | Yes, `Index = ()`. |
| `FunctorWithIndex` | 2080-2114 | Yes. |
| `FoldableWithIndex` | 2117-2155 | Yes. |

### Implemented (via `TryThunkBrand`, bifunctor):

| Type Class | Lines | Correct? |
|---|---|---|
| `Bifunctor` | 1291-1338 | Yes. |
| `Bifoldable` | 1340-1524 | Yes, all three methods. |

### Applicative/Monad Consistency for `TryThunkOkAppliedBrand`

The `Lift` impl for `TryThunkOkAppliedBrand<A>` (lines 1654-1668) uses **fail-last** semantics: it evaluates both `fa` and `fb` before inspecting results. The `Semiapplicative::apply` (lines 1720-1729) also uses fail-last.

However, the `Semimonad::bind` (lines 1764-1772) uses **fail-fast**: it evaluates `ma`, and if it is `Ok`, it returns `Ok` immediately without evaluating the continuation.

This is a **known semantic divergence** between the applicative and monadic interfaces on the error channel, and it is clearly documented in the code (lines 1615-1621, 1681-1687). In Haskell's `Validation` type, the same pattern exists: the applicative accumulates errors while the monad short-circuits. This is intentional and well-established in FP, though it does mean that `apply(pure(f), x) /= bind(pure(f), \f -> map(f, x))` when `x` is `Ok`, which violates the standard monad/applicative consistency law.

Whether this is a bug or a feature depends on the intended use case. The documentation acknowledges the divergence. Strictly speaking, this makes `TryThunkOkAppliedBrand` not a lawful monad if the applicative/monad consistency law is required.

### Missing Instances

1. **`Evaluable`**: `Thunk` implements `Evaluable` (line 692 of thunk.rs), but `TryThunkErrAppliedBrand<E>` does not. This makes sense because `Evaluable` extracts the inner value, but `TryThunk::evaluate` returns `Result<A, E>`, not `A`. The type class would need to return `Result<A, E>` which does not match `Evaluable`'s signature of `fn evaluate<A>(fa) -> A`. So this omission is correct.

2. **`Traversable`**: Explicitly documented as impossible (lines 96-99) due to `FnOnce` not being `Clone`. Same limitation as `Thunk`. Correct.

3. **`Bitraversable`**: Not implemented. This would require `TryThunk` to be `Clone`, which it cannot be for the same reason as `Traversable`. Correct omission.

4. **`Alt`/`Plus`**: Not implemented. These could potentially be implemented for `TryThunkErrAppliedBrand<E>` (try first, on failure try second, similar to `catch`). This is a potential addition but not a gap, since `Alt` is not commonly implemented across the hierarchy.

5. **`MonadError`/`MonadThrow`**: There is no `MonadThrow`/`MonadError` class in the library, so this is not applicable, but `catch` and `catch_with` provide the equivalent functionality as inherent methods.

## 4. API Surface

**Well-designed.** The API mirrors `Thunk` closely while adding fallible-specific operations:

| Thunk Method | TryThunk Equivalent | Notes |
|---|---|---|
| `new(f)` | `new(f)` | `f` returns `Result<A, E>` instead of `A`. |
| `pure(a)` | `pure(a)` / `ok(a)` | `ok` is an alias for readability. |
| N/A | `err(e)` | Error constructor. |
| `bind(f)` | `bind(f)` | Short-circuits on `Err`. |
| `map(f)` | `map(f)` | Maps `Ok` value. |
| N/A | `map_err(f)` | Maps `Err` value. |
| N/A | `catch(f)` | Error recovery (same error type). |
| N/A | `catch_with(f)` | Error recovery (different error type). |
| N/A | `bimap(f, g)` | Maps both channels. |
| `evaluate()` | `evaluate()` | Returns `Result<A, E>`. |
| `defer(f)` | `defer(f)` | Deferred construction. |
| `into_rc_lazy()` | `into_rc_try_lazy()` | Memoize. |
| `into_arc_lazy()` | `into_arc_try_lazy()` | Thread-safe memoize. |
| N/A | `into_inner()` | Escape hatch to `Thunk<Result>`. |
| N/A | `lift2(other, f)` | Inherent lift2. |
| N/A | `then(other)` | Sequence, discarding first. |
| N/A | `catch_unwind(f)` | Panic capture (`E = String`). |
| N/A | `catch_unwind_with(f, handler)` | Panic capture (custom `E`). |

**Observations:**

- `lift2` and `then` are convenience methods not present on `Thunk`. They are useful for `TryThunk` because the short-circuiting behavior makes sequencing/combining common.
- `catch_unwind` is restricted to `E = String` (line 608). The `catch_unwind_with` variant is generic. Good layered design.
- There is no `and_then` alias for `bind`, which might be more idiomatic for Rust users familiar with `Result::and_then`. This is a minor observation, not a deficiency.

## 5. Consistency with Thunk and Other Try* Types

### Consistent with `Thunk`:

- Same newtype-over-`Box<dyn FnOnce>` pattern (via delegation to `Thunk`).
- Same documentation structure (type parameters, examples, limitations).
- Same HKT pattern (`impl_kind!` macro).
- Same type class implementations (Functor, Pointed, Lift, Semiapplicative, Semimonad, MonadRec, Foldable, WithIndex, FunctorWithIndex, FoldableWithIndex, Semigroup, Monoid, Deferrable, Debug).
- Same `cannot implement Traversable` limitation documented.
- Same `into_rc_*` and `into_arc_*` conversion pattern, with `into_arc_*` evaluating eagerly due to `!Send` closure.

### Consistent with `TrySendThunk`:

- `TrySendThunk` is the `Send` counterpart, wrapping `SendThunk<Result<A, E>>`.
- `TrySendThunk` lacks HKT trait impls (Functor, Monad, etc.) because the HKT trait signatures do not impose `Send` bounds on closures. This is documented at brands.rs lines 272-275.
- Both share the same API shape (new, ok, err, bind, map, map_err, catch, evaluate, etc.).

### Consistent with `TryTrampoline`:

- `From<TryTrampoline>` is implemented (line 750-769), maintaining the conversion graph.
- `TryTrampoline` requires `'static` while `TryThunk` supports `'a`. This trade-off is documented.

### Consistent with `TryLazy`:

- `From<TryLazy>` is implemented (line 677-701).
- `into_rc_try_lazy` and `into_arc_try_lazy` are provided for the reverse direction.

## 6. Limitations and Issues

### Confirmed Limitations (correctly documented)

1. **Not stack-safe** (lines 88-92): `bind` chains grow the stack. Documented, with `TryTrampoline` as the alternative.
2. **Not memoized** (line 56): Each `evaluate` re-runs the computation. Documented, with `TryLazy` as the alternative.
3. **Cannot implement `Traversable`** (lines 96-99): Due to `FnOnce` not being `Clone`.
4. **`'static` bound on brand type parameters** (documented at brands.rs lines 262-268): `TryThunkErrAppliedBrand<E>` requires `E: 'static`, `TryThunkOkAppliedBrand<A>` requires `A: 'static`. This is an inherent limitation of the Brand pattern.

### Potential Issues

1. **Applicative/Monad inconsistency on `TryThunkOkAppliedBrand`**: As discussed in section 3, `apply` uses fail-last while `bind` uses fail-fast. This is documented but may surprise users who expect the standard consistency law to hold. Whether to split into separate applicative-only and monad-only brands (like Haskell's `Validation` vs `Either`) could be considered.

2. **`Semigroup` short-circuit semantics**: The `Semigroup` impl (lines 1244-1249) uses `?` to short-circuit on the first `Err`. This means `append(err, ok)` returns the first error, and `append(ok, err)` returns the second error. The associativity law holds because `append` on the inner `A: Semigroup` is associative and the `?` operator is left-to-right sequential. This is correct.

3. **Duplicate `impl` block**: There are two `impl<'a, A: 'a, E: 'a> TryThunk<'a, A, E>` blocks (starting at lines 111 and 544). The second block contains `catch_unwind_with` and `into_inner`. This is not a bug but is slightly unusual; it may have been split for organizational reasons (the second block has different `document_*` attributes). This is fine.

4. **No `From<TryThunk> for TryTrampoline`**: There is `From<TryTrampoline> for TryThunk` (line 750) but not the reverse. `Thunk` has bidirectional conversion with `Trampoline` (thunk.rs lines 366-402). The missing direction would require `A: 'static` and `E: 'static`, which is valid. This is a minor asymmetry.

## 7. Documentation Quality

**Excellent.** Documentation is thorough and consistent:

- Module-level doc comment (lines 1-3) is concise and accurate.
- Struct-level doc (lines 53-103) covers: what it is, HKT representations, when to use, algebraic properties, stack safety, and limitations.
- Every public method has `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` attributes following the project convention.
- All doc examples are testable (`cargo test --doc`).
- The `catch_with` documentation (lines 321-335) clearly explains how it differs from `catch`.
- The `OkAppliedBrand` `Lift` and `Semiapplicative` impls document the fail-last semantics explicitly (lines 1615-1621, 1681-1687).
- The `'static` bound rationale is documented on both brand types in brands.rs.

**Minor documentation issue:** In `fold_map` for `TryThunkOkAppliedBrand` (line 1933), the parameter description says "The Thunk to fold" rather than "The TryThunk to fold." Same issue at line 1180 for `TryThunkErrAppliedBrand`. These are copy-paste artifacts from the `Thunk` implementation.

## 8. Test Coverage

**Comprehensive.** The test module (lines 2159-3088) includes:

- Basic success/failure paths (7 tests).
- `map`, `map_err`, `bind` with both success and failure propagation.
- Borrowing test (captures references).
- Conversion tests: `From<Lazy>`, `From<TryLazy>`, `From<Thunk>`, `From<Result>`.
- `defer`, `catch`, `catch_with`.
- HKT tests for both `ErrAppliedBrand` and `OkAppliedBrand` (Functor, Pointed, Semimonad, Foldable).
- Bifunctor and Bifoldable tests.
- MonadRec tests including short-circuit on `Ok`.
- `catch_unwind` and `catch_unwind_with` tests.
- `into_rc_try_lazy` with caching verification.
- `into_arc_try_lazy` with thread safety verification (multi-threaded test).
- `Semigroup` short-circuit tests.

**QuickCheck property tests:**

- Functor identity and composition laws.
- Monad left identity, right identity, and associativity (both `Ok` and `Err` channels).
- Error short-circuit property.
- Bifunctor identity and composition laws (both `Ok` and `Err` paths).
- Semigroup associativity.
- Monoid left and right identity.

This is thorough. No significant gaps in test coverage.

## Summary

`TryThunk` is a well-designed, correctly implemented, and thoroughly tested fallible deferred computation type. It justifies its existence as a newtype over `Thunk<Result>` through ergonomic API, short-circuiting monadic semantics, error recovery combinators, triple HKT encoding, and a rich conversion graph with the rest of the lazy hierarchy.

**Key findings:**

- No correctness bugs found.
- The applicative/monad inconsistency on `TryThunkOkAppliedBrand` is intentional and documented, but may warrant consideration of whether a separate `Validation`-style type would be cleaner.
- Two minor copy-paste documentation issues ("The Thunk to fold" instead of "The TryThunk to fold") at lines 1180 and 1933.
- Missing `From<TryThunk> for TryTrampoline` conversion (asymmetric with the `Thunk`/`Trampoline` relationship).
- Implementation quality is high; the delegation pattern to `Thunk` keeps the code DRY and correct.
