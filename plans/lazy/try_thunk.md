# TryThunk Analysis

## Overview

`TryThunk<'a, A, E>` is a deferred, non-memoized fallible computation. It is a newtype wrapper around `Thunk<'a, Result<A, E>>` that provides ergonomic combinators for error handling. File: `fp-library/src/types/try_thunk.rs` (1,836 lines of implementation + ~600 lines of tests).

## 1. Design

### Relationship to Thunk

The design is clean: `TryThunk<'a, A, E>(Thunk<'a, Result<A, E>>)`. This is a textbook newtype pattern. All operations delegate to `Thunk` internally, with the error-handling logic layered on top. This mirrors how `TryTrampoline<A, E>(Trampoline<Result<A, E>>)` wraps `Trampoline`.

### Fallible Variant Design

The type provides three HKT brands, which is thorough:
- `TryThunkBrand`: bifunctor over both `E` and `A`.
- `TryThunkErrAppliedBrand<E>`: functor/monad over `A` (the success channel). This is the primary use case.
- `TryThunkOkAppliedBrand<A>`: functor/monad over `E` (the error channel). Unusual but algebraically complete.

This mirrors how `Result` is handled elsewhere in the library (`ResultBrand`, `ResultErrAppliedBrand`, `ResultOkAppliedBrand`), which is good for consistency.

### API Surface

The inherent methods are well-chosen:
- Constructors: `new`, `pure`, `ok`, `err`, `defer`
- Combinators: `map`, `map_err`, `bind`, `catch`, `lift2`, `then`
- Evaluation: `evaluate`
- Memoization: `memoize`, `memoize_arc`
- Panic capture: `catch_unwind`, `catch_unwind_with`

The `catch` method (error recovery) is a nice addition that `Thunk` does not need.

## 2. Implementation Correctness

### Functor/Monad Implementations

The `Functor` impl for `TryThunkErrAppliedBrand<E>` correctly delegates to `fa.map(func)`, which internally does `self.0.map(|result| result.map(func))`. This is correct: it maps over the success value and leaves errors untouched.

The `Semimonad` impl for `TryThunkErrAppliedBrand<E>` delegates to `ma.bind(func)`, which does:
```rust
TryThunk(self.0.bind(|result| match result {
    Ok(a) => f(a).0,
    Err(e) => Thunk::pure(Err(e)),
}))
```
This correctly short-circuits on errors and chains on success. The monad laws are verified with QuickCheck property tests.

The `Bifunctor` impl is correct, mapping `f` over `Err` and `g` over `Ok`.

### TryThunkOkAppliedBrand Implementations

The error-channel monad is algebraically sound. The `Pointed::pure` wraps in `Err`, `Semimonad::bind` chains on `Err` and short-circuits on `Ok`. This is the dual of the success-channel monad, analogous to how some FP libraries treat the "left monad" of `Either`.

### Potential Issues

**`Semigroup::append` evaluates both sides eagerly.** Lines 1116-1121:
```rust
TryThunk::new(move || match (a.evaluate(), b.evaluate()) {
    (Ok(a_val), Ok(b_val)) => Ok(Semigroup::append(a_val, b_val)),
    (Err(e), _) => Err(e),
    (_, Err(e)) => Err(e),
})
```
When `a` succeeds but `b` fails, `b` is still evaluated. This is not a bug per se (both sides must be evaluated to determine the result), but it is worth noting that this does not short-circuit. The `bind`-based approach would short-circuit, but `Semigroup::append` does not have that guarantee. The match arms are correctly ordered to prefer the left error.

**`lift2` for `TryThunkOkAppliedBrand` (lines 1516-1521) also evaluates both sides eagerly**, returning `Ok(a)` if either side is `Ok`. This correctly mirrors the dual semantics: in the error channel, `Ok` is the "short-circuit" case.

**Test says `pure` is deprecated but it is not.** Line 1855 has `#[allow(deprecated)]` and the comment says "still works but is deprecated," but the `pure` method (line 114) has no `#[deprecated]` attribute. This is either a planned deprecation that was not applied, or a stale test annotation.

**`From<TryLazy>` clones both Ok and Err.** Line 571:
```rust
TryThunk::new(move || memo.evaluate().cloned().map_err(Clone::clone))
```
The `.cloned()` call on `Result<&A, &E>` only clones the `Ok` side. The `.map_err(Clone::clone)` handles the `Err` side. This is correct but slightly redundant in expression; `memo.evaluate().map(|a| a.clone()).map_err(|e| e.clone())` would be equivalent. The current form is fine.

**No `Evaluable` implementation.** `Thunk` implements `Evaluable` but `TryThunkErrAppliedBrand` does not. This makes sense because `Evaluable` expects to produce an `A`, but `TryThunk::evaluate` produces `Result<A, E>`, not `A`. The type mismatch is fundamental.

## 3. Consistency

### With Thunk

`TryThunk` closely mirrors `Thunk` in API shape:
- Both have `new`, `pure`, `defer`, `bind`, `map`, `evaluate`, `memoize`, `memoize_arc`.
- Both use `Box<dyn FnOnce>` under the hood (via `Thunk`).
- Both implement `Deferrable`.
- Both implement `Functor`, `Pointed`, `Lift`, `Semiapplicative`, `Semimonad`, `MonadRec`, `Foldable`.
- Neither implements `Traversable` (for the same `FnOnce` / no-Clone reason).

`TryThunk` adds `map_err`, `catch`, `ok`, `err`, `lift2`, `then`, `catch_unwind`, `catch_unwind_with`, and `Bifunctor`/`Bifoldable`, which are specific to the fallible nature.

### With TryLazy

`TryLazy` does not implement the full HKT typeclass hierarchy (no Functor/Monad), which is consistent with `Lazy` also not implementing them (because `Lazy::evaluate` returns `&A`, not `A`). The `TryThunk`/`TryLazy` relationship mirrors the `Thunk`/`Lazy` relationship well.

### With TryTrampoline

`TryTrampoline<A, E>(Trampoline<Result<A, E>>)` follows the exact same newtype pattern. The consistency across the hierarchy is strong.

### With Result

The brand structure (`TryThunkBrand`, `TryThunkErrAppliedBrand<E>`, `TryThunkOkAppliedBrand<A>`) mirrors `ResultBrand`, `ResultErrAppliedBrand<E>`, `ResultOkAppliedBrand<T>` exactly.

## 4. Limitations

**Not stack-safe.** Like `Thunk`, nested `bind` chains grow the call stack. The `MonadRec` implementation provides an escape hatch via `tail_rec_m`, but arbitrary `bind` chains are not safe. This is documented in `Thunk` but not explicitly restated in `TryThunk`'s module docs or struct docs. It should be.

**Cannot implement `Traversable`.** Same as `Thunk`, because `FnOnce` closures cannot be cloned. This is an inherent limitation.

**No `Evaluable` trait.** As discussed, this is a fundamental type mismatch. Not really addressable without a `TryEvaluable` trait or similar.

**`E: 'static` required for HKT brands.** Both `TryThunkErrAppliedBrand<E>` and `TryThunkOkAppliedBrand<A>` require the fixed type parameter to be `'static` (lines 681, 1399). This is a limitation of the `impl_kind!` macro. It means you cannot use HKT-generic functions with `TryThunk` when the error type borrows from a local scope. The inherent methods (`.map`, `.bind`) still work with any lifetime.

**`Semigroup` requires `A: Semigroup` but not `E: Semigroup`.** This means errors cannot be accumulated. The first error wins. This is the standard behavior for `Result`-like monads and is appropriate for `TryThunk`, but it is worth noting for users who want error accumulation (they would need a `Validation`-style type instead).

## 5. Alternatives

### Could this just be `Thunk<Result<A, E>>`?

Technically, yes. Every operation on `TryThunk` could be expressed using `Thunk<Result<A, E>>` directly. The advantages of the newtype:

1. **Ergonomic API.** Methods like `ok`, `err`, `catch`, `map_err`, `catch_unwind` are domain-specific and would not exist on `Thunk<Result<A, E>>`.
2. **Correct HKT semantics.** `Thunk<Result<A, E>>` via `ThunkBrand` treats the whole `Result<A, E>` as a single type parameter. You cannot get `Functor` over just `A` or just `E` without the separate brands.
3. **Type safety.** The newtype prevents accidentally treating a `TryThunk` as a plain `Thunk` or vice versa.

The current design is the right call. The newtype cost is zero at runtime (it compiles away), and the ergonomic and type-level benefits are substantial.

### Alternative: `ExceptT`-style monad transformer

In Haskell, `ExceptT e m a` is a monad transformer that adds error handling to any monad `m`. `TryThunk` is effectively `ExceptT E Thunk A` specialized. A general `ExceptT` would be more powerful but significantly more complex in Rust's type system, especially with the brand/HKT machinery. The specialized approach is pragmatic and appropriate for this library's current scope.

### Alternative: trait-based error handling

One could define a `TryFunctor` or `MonadError` trait that any brand can implement. This would allow generic error-handling code to work across `TryThunk`, `TryLazy`, `Result`, etc. This is a larger architectural question beyond `TryThunk` itself.

## 6. Documentation

### Strengths

- Every method has `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` annotations.
- The doc examples are all runnable and demonstrate both success and failure paths.
- The HKT brand explanation in the struct docs is clear and lists all three brands.
- The relationship to `Thunk` is stated upfront: "This is `Thunk<'a, Result<A, E>>` with ergonomic combinators for error handling."

### Weaknesses

- **Missing stack safety warning.** The struct-level docs do not mention that `bind` chains are not stack-safe, unlike `Thunk` which documents this prominently. This should be added.
- **Missing `Traversable` limitation note.** `Thunk` documents why it cannot implement `Traversable`. `TryThunk` should do the same.
- **Missing algebraic properties section.** `Thunk` documents its monad laws. `TryThunk` should document the same for its success-channel monad.
- **`pure` deprecation confusion.** The test file references `pure` as deprecated, but the method is not marked `#[deprecated]`. Either add the deprecation attribute or update the test.
- **Module-level docs are thin.** The module doc is a single sentence. It could benefit from a brief comparison table (like `Thunk` has) or a "when to use" section.
- **`catch` method name.** The name `catch` is reasonable, but the docs do not explain the relationship to `bind` on the error channel. The method is essentially `bind` for the error side: `match result { Ok(a) => Ok(a), Err(e) => f(e) }`. Noting this duality would help users understand the algebraic structure.

## 7. Test Coverage

The test suite is thorough:
- Unit tests for all inherent methods (`ok`, `err`, `map`, `map_err`, `bind`, `catch`, `defer`, `lift2`, `then`, `memoize`, `catch_unwind`).
- Conversion tests (`From<Lazy>`, `From<TryLazy>`, `From<Thunk>`, `From<Result>`, `From<TryTrampoline>`).
- HKT trait tests for both `TryThunkErrAppliedBrand` and `TryThunkOkAppliedBrand`.
- Bifunctor and Bifoldable tests.
- QuickCheck property tests for functor laws, monad laws, and error short-circuiting.
- `MonadRec` tests including short-circuit behavior.

**Missing tests:**
- No QuickCheck tests for bifunctor laws (identity, composition).
- No QuickCheck tests for the error-channel monad laws (`TryThunkOkAppliedBrand`).
- No tests for `Semigroup`/`Monoid` laws.
- No test verifying that `memoize_arc` produces a value accessible from multiple threads.

## Summary

`TryThunk` is a well-designed, correctly implemented fallible computation type. The newtype-over-`Thunk<Result>` approach is the right design choice, providing zero-cost abstraction with strong ergonomics. The HKT integration with three brands is thorough and consistent with the rest of the library. The main areas for improvement are: (1) adding stack safety and limitation documentation to match `Thunk`'s standard, (2) resolving the `pure` deprecation confusion, and (3) expanding QuickCheck coverage to bifunctor and error-channel monad laws.
