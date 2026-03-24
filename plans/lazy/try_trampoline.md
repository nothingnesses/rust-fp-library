# TryTrampoline Analysis

## Overview

`TryTrampoline<A, E>` is a newtype wrapper around `Trampoline<Result<A, E>>`, providing stack-safe fallible computation with ergonomic combinators for error handling. It lives at `fp-library/src/types/try_trampoline.rs` (903 lines of implementation, ~630 lines of tests).

## Design Assessment

### Wrapping `Trampoline<Result<A, E>>` is the right approach

The newtype pattern here is well-chosen for several reasons:

1. **Stack safety comes for free.** `Trampoline` is built on `Free<ThunkBrand, A>` with CatList-based bind, giving O(1) bind and unlimited recursion depth. Wrapping `Trampoline<Result<A, E>>` inherits all of this without reimplementation.
2. **The wrapper is zero-cost at the type level.** The newtype is erased at compile time; the only overhead is the `Result` discrimination at evaluation boundaries.
3. **Consistent with the library's pattern.** `TryThunk` wraps `Thunk<'a, Result<A, E>>` in the same way. This parallelism makes the hierarchy predictable.
4. **No HKT brand needed.** Both `Trampoline` and `TryTrampoline` intentionally lack brands because `Free`'s type erasure (`Box<dyn Any>`) makes HKT encoding impractical. The documentation correctly states this.

### Short-circuiting behavior is correct

- **`bind`**: On `Err(e)`, returns `Trampoline::pure(Err(e))` without calling `f`. The user's closure is never invoked. Correct.
- **`catch`**: Mirror of `bind` for the error path. On `Ok(a)`, wraps it immediately. Correct.
- **`tail_rec_m`**: Wraps the state in `Result<S, E>`. On `Err(e)`, returns `Step::Done(Err(e))`, terminating the loop immediately. On success, unwraps the `Step` and re-wraps appropriately. Correct.
- **`lift2` and `then`**: Implemented via `bind`, so they inherit short-circuiting. Correct.
- **`map` and `map_err`**: These delegate to `Trampoline::map`, which adds a single O(1) `Map` node in the `Free` structure. The `Result::map`/`Result::map_err` only runs at evaluation time. There is no short-circuiting in `map` because there is nothing to skip; the function is simply not applied to the wrong variant. Correct behavior.

### The `tail_rec_m` encoding is clever but has a subtle cost

The implementation wraps state as `Result<S, E>` and delegates to `Trampoline::tail_rec_m`:

```rust
move |state: Result<S, E>| match state {
    Err(e) => Trampoline::pure(Step::Done(Err(e))),
    Ok(s) => f(s).0.map(|result| match result {
        Ok(Step::Loop(next)) => Step::Loop(Ok(next)),
        Ok(Step::Done(a)) => Step::Done(Ok(a)),
        Err(e) => Step::Done(Err(e)),
    }),
}
```

This is correct and stack-safe. However, every successful iteration wraps and unwraps the state in `Ok(...)`, adding one allocation for the `Result` enum per step. In practice this is negligible compared to the `Free` node allocations that `Trampoline::tail_rec_m` already performs, but it is worth noting.

## Issues and Concerns

### 1. No `pure` constructor (naming inconsistency)

`Trampoline` has `Trampoline::pure(a)` for wrapping a value. `TryTrampoline` uses `TryTrampoline::ok(a)` instead. While `ok`/`err` naming mirrors `Result`, the lack of a `pure` alias means `TryTrampoline` breaks the pattern set by `Trampoline` and by `TryThunk` (which has both `pure` and `ok`/`err`).

**Recommendation:** Add `pub fn pure(a: A) -> Self` as an alias for `ok`, matching `TryThunk::pure` and the general FP convention.

### 2. No `bimap` on `TryThunk` (but present on `TryTrampoline`)

`TryTrampoline` has an inherent `bimap` method. `TryThunk` only has `bimap` via its `Bifunctor` HKT implementation (not as an inherent method). This is not a bug in `TryTrampoline`, but an inconsistency in the other direction: `TryTrampoline` is actually *more* ergonomic than `TryThunk` here because users can call `.bimap(f, g)` directly without going through the `Bifunctor` free function. No change needed for this file, but `TryThunk` could benefit from an inherent `bimap`.

### 3. Missing `and_then` alias

Rust convention uses `and_then` for monadic bind on `Result`/`Option`. `TryTrampoline` only offers `bind`. An `and_then` alias would improve discoverability for Rust users who are not steeped in FP terminology. This is a minor ergonomic point, not a correctness issue.

### 4. No `into_inner` or `into_trampoline` accessor

There is no way to unwrap a `TryTrampoline<A, E>` back into the underlying `Trampoline<Result<A, E>>`. The inner field `self.0` is private. Users who want to compose with `Trampoline` directly (e.g., to avoid the `Result` wrapping overhead in a known-infallible section) cannot do so.

**Recommendation:** Consider adding `pub fn into_trampoline(self) -> Trampoline<Result<A, E>>`.

### 5. `memoize_arc` eagerly evaluates (correctly documented)

`memoize_arc` evaluates the trampoline eagerly because `Trampoline`'s inner closures are `!Send`. The doc comment correctly explains this. The same pattern exists on `Trampoline::memoize_arc`. No issue here.

### 6. `From<Lazy>` requires `Clone` (correctly bounded)

The `From<Lazy<'static, A, Config>>` impl requires `A: Clone` because `Lazy::evaluate` returns `&A`. The `From<TryLazy>` impl requires both `A: Clone` and `E: Clone`. These bounds are correct and necessary.

### 7. `catch` does not allow changing the error type

`catch` has signature `fn catch(self, f: FnOnce(E) -> TryTrampoline<A, E>) -> Self`. The error type `E` is fixed. This is intentional (it mirrors `MonadError`'s `catchError`), but users who want to recover into a different error type must use `map_err` first. This is fine and consistent.

### 8. No `flatten` method

There is no `flatten` for `TryTrampoline<TryTrampoline<A, E>, E> -> TryTrampoline<A, E>`. This is derivable from `bind(id)` but could be a convenience. Low priority.

## Documentation Quality

The documentation is thorough and accurate:

- Every public method has `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` attributes.
- The module-level doc comment clearly explains the type's purpose and shows a complete example.
- The struct-level doc comment explains when to use `TryTrampoline` vs `TryThunk` vs `TryLazy`.
- The `tail_rec_m` documentation includes a factorial example with both success and error paths.
- The `Clone` bound on `tail_rec_m`'s closure is explained, with a pointer to `arc_tail_rec_m`.

One minor doc issue: the `#[document_examples]` attribute on `defer` appears *after* the main example block (the factorial), which means the macro-generated "Examples" heading may appear between the two example blocks. This could produce slightly confusing rendered docs depending on how `document_examples` works.

## Test Coverage

The test suite is comprehensive (approximately 630 lines, 30+ tests):

- **Basic operations**: `ok`, `err`, `new`, `map`, `map_err`, `bimap`, `bind`, `catch`, `lift2`, `then`.
- **Conversions**: `From<Trampoline>`, `From<Lazy>`, `From<TryLazy>`, `From<TryThunk>`, `From<Result>`.
- **Stack safety**: 100,000-iteration tests for both success and error paths in `tail_rec_m`.
- **`!Send` types**: Tests with `Rc<T>` verifying that `TryTrampoline` works in single-threaded contexts.
- **Panic catching**: Tests for both `catch_unwind` and `catch_unwind_with`.
- **Law tests (QuickCheck)**:
  - Functor identity and composition.
  - Monad left identity, right identity, and associativity.
  - Error short-circuiting.
  - Semigroup associativity.
  - Monoid left and right identity.
  - Bifunctor consistency with `map`/`map_err`.

**Missing test coverage:**
- `Deferrable` trait impl is not directly tested (though it is exercised via `defer`).
- `Debug` formatting test exists (via doc test) but there is no unit test for it in the test module.
- No test for `memoize` or `memoize_arc` in the test module (only in doc tests).
- No QuickCheck test for bimap identity law (only consistency tests).

## Comparison with Peer Types

| Feature | TryTrampoline | TryThunk | TrySendThunk |
|---------|--------------|----------|--------------|
| Stack safe | Yes | Partial (via `tail_rec_m`) | No |
| HKT brand | No | Yes (`TryThunkBrand`) | No |
| Lifetimes | `'static` only | `'a` | `'a` |
| `Send` | No | No | Yes |
| `pure` constructor | No (only `ok`) | Yes | N/A |
| Inherent `bimap` | Yes | No (trait only) | N/A |
| `tail_rec_m` | Yes (inherent) | Yes (trait) | No |
| `Semigroup`/`Monoid` | Yes | Yes (trait) | N/A |
| `catch_unwind` | Yes | Yes | N/A |

## Summary of Recommendations

1. **Add `pure` alias** for `ok` to maintain consistency with `TryThunk` and `Trampoline`.
2. **Consider `into_trampoline`** to expose the inner `Trampoline<Result<A, E>>` for advanced users.
3. **Consider `and_then` alias** for `bind` to match Rust idiom (low priority).
4. **Add unit tests** for `memoize`, `memoize_arc`, and `Deferrable` impl.
5. **No structural changes needed.** The newtype-over-`Trampoline<Result<A, E>>` design is sound, the short-circuiting is correct, the documentation is accurate, and the test coverage is strong.
