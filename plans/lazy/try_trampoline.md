# TryTrampoline Analysis

**File:** `fp-library/src/types/try_trampoline.rs` (1795 lines)

## 1. Design: Wrapping `Trampoline<Result<A, E>>`

The newtype wrapper pattern `TryTrampoline<A, E>(Trampoline<Result<A, E>>)` (line 51-54) is sound and consistent with the rest of the hierarchy:

- `TryThunk<'a, A, E>` wraps `Thunk<'a, Result<A, E>>` (try_thunk.rs:100-103).
- `TryTrampoline<A, E>` wraps `Trampoline<Result<A, E>>` (line 51-54).

This is the correct approach. `Trampoline` already provides stack-safe monadic chaining via `Free<ThunkBrand, A>`. Wrapping `Result` inside it gives stack-safe error-propagating computation without reimplementing the Free monad machinery. The delegation pattern keeps the implementation small and correct by reusing battle-tested `Trampoline` internals.

The `'static` requirement is inherited from `Trampoline` (which requires `'static` due to `Box<dyn Any>` in `Free`). This is documented accurately at lines 45-46.

**Verdict:** Sound design, well-motivated.

## 2. Implementation Quality

### 2.1 Correctness of Core Operations

All core operations correctly delegate to `Trampoline` internals:

- **`ok`/`err`** (lines 75, 135): Use `Trampoline::pure(Ok(a))` / `Trampoline::pure(Err(e))`. Correct.
- **`new`** (line 155-157): `Trampoline::new(f)` where `f: FnOnce() -> Result<A, E>`. Correct.
- **`defer`** (line 199-200): `Trampoline::defer(move || f().0)`. Correctly unwraps the inner `TryTrampoline` and re-wraps.
- **`map`** (lines 221-226): `self.0.map(|result| result.map(func))`. Correctly maps over the `Ok` variant only.
- **`map_err`** (lines 247-252): `self.0.map(|result| result.map_err(func))`. Correct.
- **`bimap`** (lines 283-292): Correctly maps both variants.
- **`bind`** (lines 312-320): Delegates to `Trampoline::bind`, pattern-matches on the `Result`, and short-circuits on `Err`. This is the standard `EitherT` bind implementation. Correct.
- **`catch`** (lines 339-347): Inverts `bind` by continuing on `Err` and short-circuiting on `Ok`. Correct.
- **`catch_with`** (lines 379-387): Allows error type to change during recovery. Correct.
- **`evaluate`** (lines 597-598): Delegates to `self.0.evaluate()`. Correct.

### 2.2 `tail_rec_m` Implementation

The `tail_rec_m` (lines 503-521) uses an inner `go` function that wraps each iteration in `Trampoline::defer`, ensuring stack safety. The pattern match on `Ok(Step::Loop(next))`, `Ok(Step::Done(a))`, `Err(e)` is correct and mirrors Trampoline's own `tail_rec_m` with the added error short-circuit.

The `arc_tail_rec_m` (lines 570-581) correctly wraps in `Arc` and delegates to `tail_rec_m`.

### 2.3 Potential Issues

**No bugs found.** The implementation is straightforward delegation with correct `Result`-aware short-circuiting at each layer.

**Minor note:** The `catch` method (line 339) constrains the recovery function to return the same error type `E`. The `catch_with` method (line 379) allows a different error type `E2`. Both are correct and complementary.

## 3. Type Class Instances

### 3.1 Implemented

| Trait | Lines | Correct? |
|-------|-------|----------|
| `Deferrable<'static>` | 922-951 | Yes, delegates to `Trampoline::defer`. |
| `Semigroup` (where `A: Semigroup`) | 955-993 | Yes, uses `lift2` with `Semigroup::append`. |
| `Monoid` (where `A: Monoid`) | 997-1021 | Yes, wraps `A::empty()` in `Ok`. |
| `Debug` | 1025-1043 | Yes, displays `"TryTrampoline(<unevaluated>)"` without forcing. |

### 3.2 Conversions (`From` impls)

| From | Lines | Notes |
|------|-------|-------|
| `Trampoline<A>` | 785-803 | Maps `A` to `Ok(A)`. Correct. |
| `Lazy<'static, A, Config>` | 811-836 | Defers evaluation, clones from memoized. Correct. |
| `TryLazy<'static, A, E, Config>` | 843-873 | Defers evaluation, clones both Ok/Err. Correct; `Clone` bounds justified. |
| `TryThunk<'static, A, E>` | 876-895 | Evaluates thunk inside trampoline. Correct. |
| `Result<A, E>` | 898-919 | Wraps in `Trampoline::pure`. Correct. |

### 3.3 Missing Instances

**No HKT brand exists for `TryTrampoline`.** This is intentional and documented (line 45-46): `Trampoline` requires `'static`, which conflicts with the `Kind` trait's lifetime polymorphism (`Of<'a, A>`). This is consistent with `Trampoline` itself lacking a brand.

The following are absent but would be reasonable to consider:

1. **No `Bifunctor` trait impl.** `TryTrampoline` has `bimap`, `map`, and `map_err` as inherent methods, which covers the functionality. Since there is no HKT brand, a formal `Bifunctor` impl is impossible anyway.

2. **No `MonadError`-style trait.** The library does not appear to have a `MonadError` class. If one were added, `TryTrampoline` should implement it.

3. **No `Semigroup`/`Monoid` for the error side.** The current `Semigroup` for `TryTrampoline` requires `A: Semigroup` and short-circuits on the first error. An alternative `Semigroup` that accumulates errors (a la `Validation`) is not provided, but this is a different type semantically, not a missing instance.

**Verdict:** All instances that make sense given the `'static` constraint and lack of HKT brand are present.

## 4. API Surface

### 4.1 Methods Provided

| Category | Methods |
|----------|---------|
| Construction | `ok`, `pure`, `err`, `new`, `defer` |
| Transformation | `map`, `map_err`, `bimap` |
| Chaining | `bind`, `catch`, `catch_with` |
| Combining | `lift2`, `then`, `append`, `empty` |
| Recursion | `tail_rec_m`, `arc_tail_rec_m` |
| Conversion | `evaluate`, `into_trampoline`, `into_rc_try_lazy`, `into_arc_try_lazy` |
| Panic safety | `catch_unwind`, `catch_unwind_with` |

### 4.2 Comparison with `TryThunk`

`TryThunk` has an additional method `into_inner` (try_thunk.rs:599) that returns the inner `Thunk`. `TryTrampoline` has the equivalent `into_trampoline` (line 115). The naming is inconsistent: `TryThunk` uses `into_inner` while `TryTrampoline` uses `into_trampoline`. Both expose the wrapped type, but the naming diverges. This is a minor inconsistency; `into_inner` is the more generic/conventional Rust name.

`TryThunk` does NOT have `tail_rec_m` or `arc_tail_rec_m` as inherent methods (it gets these via HKT `MonadRec` trait impls on its brands). `TryTrampoline` provides them as inherent methods since it lacks brands. This is the correct approach.

`TryThunk` does NOT have `catch_unwind` / `catch_unwind_with`. `TryTrampoline` does (lines 740-781). This is a nice addition unique to `TryTrampoline`.

### 4.3 Assessment

The API is well-designed and comprehensive. The `catch_with` method (lines 379-387) allowing error type transformation is a thoughtful addition that goes beyond a basic `catch`. The `catch_unwind` family provides pragmatic integration with Rust's panic system.

## 5. Consistency

### 5.1 With `Trampoline`

`TryTrampoline` mirrors `Trampoline`'s API surface faithfully, adding error-specific operations (`err`, `map_err`, `bimap`, `catch`, `catch_with`, `catch_unwind`). The method signatures follow the same patterns. Conversion methods (`into_rc_try_lazy`, `into_arc_try_lazy`) mirror Trampoline's (`into_rc_lazy`, `into_arc_lazy`).

### 5.2 With Other `Try*` Types

The pattern is consistent across the hierarchy:
- `TryThunk<'a, A, E>` wraps `Thunk<'a, Result<A, E>>`.
- `TryTrampoline<A, E>` wraps `Trampoline<Result<A, E>>`.
- Both provide `ok`, `err`, `new`, `defer`, `map`, `map_err`, `bimap`, `bind`, `catch`, `catch_with`, `evaluate`, `lift2`, `then`, `append`, `empty`.

### 5.3 Naming Inconsistency

As noted, `into_inner` (TryThunk) vs `into_trampoline` (TryTrampoline). Minor but worth standardizing. Options:
- Rename `TryTrampoline::into_trampoline` to `into_inner` for consistency.
- Or keep as-is since `into_trampoline` is more descriptive of the returned type.

### 5.4 `TryThunk` Has `Semigroup` via Eager Evaluation; `TryTrampoline` via `lift2`

`TryThunk`'s `Semigroup::append` (try_thunk.rs:1240-1249) eagerly evaluates both sides with `?` operator. `TryTrampoline`'s `Semigroup::append` (line 988-993) uses `lift2`, which delegates to `bind`. Both short-circuit on the first error. The `TryTrampoline` approach is more consistent with its lazy/deferred nature.

## 6. Limitations

1. **`'static` requirement.** Both `A` and `E` must be `'static` (line 51). This prevents using borrowed error types or values. Documented at line 45.

2. **No HKT brand.** Cannot participate in generic HKT-polymorphic code. Documented at lines 45-46.

3. **Not `Send`.** The underlying `Free` monad uses `Box<dyn FnOnce>` (not `+ Send`), so `TryTrampoline` is `!Send`. The `into_arc_try_lazy` method (lines 697-702) works around this by eagerly evaluating before wrapping in an `Arc`-based lazy. This is the same pattern as `Trampoline::into_arc_lazy`.

4. **No memoization.** Like `Trampoline`, each `evaluate()` call re-runs the computation. The `into_rc_try_lazy`/`into_arc_try_lazy` methods provide an escape hatch. Documented implicitly by the conversion methods' existence.

5. **No parallel execution.** Single-threaded only due to `!Send`. Consistent with `Trampoline`.

6. **`tail_rec_m` requires `Clone` on `f`.** This is a known limitation inherited from `Trampoline::tail_rec_m`, with `arc_tail_rec_m` as the workaround. Well-documented at lines 458-465.

## 7. Documentation

### 7.1 Module-Level Documentation

The module doc (lines 1-14) is concise and accurate. The example demonstrates construction, chaining, and evaluation.

### 7.2 Struct Documentation

The struct doc (lines 38-48) correctly describes the type, explains when to use it vs alternatives, and notes the `'static` constraint. The cross-references to `TryThunk` and `TryLazy` are helpful.

### 7.3 Method Documentation

All methods follow the project's documentation template with `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]`. Examples compile and contain assertions. The `defer` method (lines 159-200) has two example blocks: one showing stack-safe recursion and one showing basic usage, which is thorough.

### 7.4 Minor Documentation Issues

1. **`#[document_examples]` placement on `defer`** (line 190): The `#[document_examples]` attribute appears after the first code block (the factorial example, lines 170-189) and before the second code block (lines 192-197). This is unusual; other methods place `#[document_examples]` before the first example block. The first example block (factorial) appears to be a standalone doc section ("Stack-safe recursion:") not gated by `#[document_examples]`. This may render oddly depending on how the `document_examples` macro works.

2. **`tail_rec_m` doc** (lines 453-456): The doc says "The step function returns `TryTrampoline<Step<S, A>, E>`" which correctly describes the signature. Good.

3. **`catch_with` doc** (lines 349-354): Thorough explanation of how it differs from `catch`. Good.

## 8. Test Coverage

The test suite (lines 1047-1795) is comprehensive:

- Basic operations: `ok`, `err`, `map`, `map_err`, `bind`, `catch`, `catch_with`, `new`, `pure`, `into_trampoline`.
- Conversions: `From<Trampoline>`, `From<Lazy>`, `From<TryLazy>`, `From<TryThunk>`, `From<Result>`.
- Combining: `lift2` (success, first error, second error), `then` (success, first error, second error).
- Recursion: `tail_rec_m` (success, error, stack safety 100k), `arc_tail_rec_m` (success, error with counter).
- Panic safety: `catch_unwind`, `catch_unwind_with` (both panic and success paths).
- `!Send` types: Tests with `Rc<T>` for `ok`, `bind`, and `tail_rec_m`.
- QuickCheck laws: Functor identity, functor composition, monad left identity, monad right identity, monad associativity, error short-circuit.
- Semigroup/Monoid: append (both ok, first err, second err), associativity (QuickCheck), empty, left/right identity (QuickCheck).
- Bimap: ok path, err path, consistency with map/map_err (QuickCheck).
- Conversion to lazy: `into_rc_try_lazy` and `into_arc_try_lazy` (ok and err paths, memoization verification).
- Stack safety: `catch_with` chained 100k times (both error and success paths).

**Verdict:** Excellent test coverage. No significant gaps.

## 9. Summary

`TryTrampoline` is a well-implemented, well-documented, and well-tested type. The newtype delegation pattern is the right approach, the API is comprehensive and consistent with both `Trampoline` and `TryThunk`, and the limitations are inherent to the design and clearly documented.

### Issues Found

| Severity | Issue | Location |
|----------|-------|----------|
| Minor | Naming inconsistency: `into_trampoline` vs `TryThunk::into_inner`. | Line 115 |
| Minor | `#[document_examples]` placement on `defer` is after the first code block rather than before it. | Line 190 |

### Recommendations

1. Consider renaming `into_trampoline` to `into_inner` for consistency with `TryThunk`, or vice versa. If keeping divergent names, document why.
2. Verify that the `defer` method's documentation renders correctly given the `#[document_examples]` placement.
