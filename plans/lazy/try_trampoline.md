# TryTrampoline Analysis

## Overview

`TryTrampoline<A, E>` is a newtype wrapper around `Trampoline<Result<A, E>>` that provides ergonomic combinators for stack-safe fallible monadic recursion. It lives at `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/try_trampoline.rs` (1819 lines including tests).

```rust
pub struct TryTrampoline<A: 'static, E: 'static>(Trampoline<Result<A, E>>);
```

## 1. Type Design

### Wrapping Trampoline<Result<A, E>>

The newtype approach is the correct design for this library. The reasons are:

- **Zero implementation cost.** All stack safety comes directly from the underlying `Free<ThunkBrand, Result<A, E>>` CatList machinery. No duplication of the trampoline loop or CatList logic.
- **Ergonomic separation.** Without the newtype, users would have to manually thread `Result` through every `map` and `bind` call on a bare `Trampoline<Result<A, E>>`. The wrapper lifts error handling into the API surface: `map` operates on the `Ok` channel only, `bind` short-circuits on `Err`, and dedicated `map_err`, `bimap`, `catch`, and `catch_with` methods handle the error channel.
- **Consistent with TryThunk.** `TryThunk<'a, A, E>` wraps `Thunk<'a, Result<A, E>>` with the same pattern. The parallel is clean.

The alternative, a dedicated `TryFree<F, A, E>` with error handling baked into the Free monad itself, would duplicate the CatList and evaluation machinery for minimal benefit. The current design keeps the abstraction tower narrow: `Free` handles stack-safe binding, `Result` handles fallibility, and `TryTrampoline` provides the ergonomic surface.

### Constructor API

The constructor set is comprehensive and mirrors the infallible `Trampoline` API with fallible additions:

| Method | Purpose |
|--------|---------|
| `ok(a)` / `pure(a)` | Lift a success value. |
| `err(e)` | Lift an error value. |
| `new(f)` | Lazy computation returning `Result<A, E>`. |
| `defer(f)` | Defer construction of a `TryTrampoline` (critical for stack safety in recursive functions). |
| `from(Result)` | Convert from a `Result`. |
| `from(Trampoline)` | Wrap an infallible trampoline in `Ok`. |
| `from(TryThunk)` | Lift a non-stack-safe thunk into the stack-safe model. |
| `from(Lazy)` / `from(TryLazy)` | Convert from memoized values. |

Having both `ok` and `pure` is intentional: `ok` mirrors `Result::Ok` for readability in concrete code, while `pure` matches the type-class convention used throughout the library.

## 2. HKT Support

TryTrampoline has **no brand** and **no HKT type class implementations** (no `Functor`, `Monad`, `Foldable`, etc. trait impls through brands). Neither does its base type `Trampoline`.

### Why no brand

The fundamental obstacle is the `'static` requirement. The library's HKT machinery uses:

```rust
type Of<'a, A: 'a>: 'a = SomeType<'a, A>;
```

For `Trampoline`, the mapping would be:

```rust
type Of<'a, A: 'a>: 'a = Trampoline<A>; // but A must be 'static
```

This means `Of<'a, A>` only works when `A: 'static`, which is more restrictive than the `A: 'a` bound the HKT machinery expects. The brand would be technically definable (since `'static` implies `'a` for any `'a`), but it would create a "lie" at the type level: generic code parameterized over any `Kind` could try to instantiate `Of<'a, A>` with non-`'static` `A` and hit a wall.

For TryTrampoline specifically, the situation is even more complex because it has two type parameters (`A` and `E`), so it would need the same bifunctor brand treatment as `TryThunk` (three brands: fully polymorphic, error-applied, success-applied), each with the `'static` restriction compounding the problem.

### Should it have one

Probably not in the near term, for the following reasons:

1. **The `'static` restriction makes generic HKT code awkward.** Any function generic over a `Functor` brand would silently require `'static` when instantiated with a hypothetical `TryTrampolineBrand`, which is surprising.
2. **Trampoline itself lacks a brand.** Adding one for TryTrampoline but not Trampoline would be inconsistent.
3. **The primary use case is terminal.** TryTrampoline values are typically built up via inherent methods and then `evaluate()`d. They rarely need to participate in HKT-polymorphic pipelines. When HKT integration is needed, `TryThunk` is the appropriate choice, and conversions exist in both directions.

A future version could explore a `StaticKind` trait variant that enforces `A: 'static` at the type level, making the brand honest. Until then, the current design is pragmatic.

## 3. Stack Safety

### Core guarantee

TryTrampoline inherits Trampoline's stack safety, which comes from the `Free<ThunkBrand, A>` CatList-based "Reflection without Remorse" implementation. The key properties are:

- **`bind` is O(1).** It appends to a CatList of continuations rather than nesting closures.
- **`evaluate` is iterative.** It pops continuations from the CatList in a loop, never growing the Rust call stack.
- **`defer` converts stack growth into heap allocation.** Each deferred step is a `Thunk` on the heap.

### Where stack safety is maintained

- **`bind` chains**: O(1) append, safe for any depth.
- **`tail_rec_m` / `arc_tail_rec_m`**: Uses `Trampoline::defer` internally, fully stack-safe. Tested with 100,000+ iterations.
- **`map`**: Implemented via `bind` on the inner Trampoline, inherits safety.
- **`map_err`**: Also via Trampoline's `map`, safe.
- **`catch`**: Uses `bind` on the inner Trampoline, safe.
- **`catch_with`**: Uses `bind` on the inner Trampoline, safe. Stack-safety is verified by tests (`test_catch_with_stack_safety`, `test_catch_with_stack_safety_ok`) with 100,000 iterations.
- **`bimap`**: Via Trampoline's `map`, safe.
- **`lift2` / `then`**: Defined in terms of `bind`, safe.
- **`append`**: Defined in terms of `lift2`, safe.

### Notable difference from TryThunk

`TryThunk::catch_with` is implemented via eager evaluation:

```rust
pub fn catch_with<E2: 'a>(...) -> TryThunk<'a, A, E2> {
    TryThunk(Thunk::new(move || match self.evaluate() {
        Ok(a) => Ok(a),
        Err(e) => f(e).evaluate(),
    }))
}
```

In contrast, `TryTrampoline::catch_with` uses `bind`, preserving laziness and stack safety:

```rust
pub fn catch_with<E2: 'static>(...) -> TryTrampoline<A, E2> {
    TryTrampoline(self.0.bind(move |result| match result {
        Ok(a) => Trampoline::pure(Ok(a)),
        Err(e) => f(e).0,
    }))
}
```

This is a meaningful design advantage for TryTrampoline.

## 4. Type Class Implementations

### What TryTrampoline implements

| Trait | Notes |
|-------|-------|
| `Deferrable<'static>` | Delegates to `Trampoline::defer`. |
| `Semigroup` | Where `A: Semigroup + 'static, E: 'static`. Via `lift2(other, Semigroup::append)`. |
| `Monoid` | Where `A: Monoid + 'static, E: 'static`. Returns `ok(A::empty())`. |
| `Debug` | Prints `"TryTrampoline(<unevaluated>)"` without forcing. |

### What TryTrampoline does NOT implement (but TryThunk does via brands)

The following are all implemented for `TryThunkErrAppliedBrand<E>`:

- `Functor`
- `Pointed`
- `Lift`
- `ApplyFirst` / `ApplySecond`
- `Semiapplicative`
- `Semimonad`
- `MonadRec`
- `Foldable`
- `WithIndex` / `FunctorWithIndex` / `FoldableWithIndex`

And for `TryThunkBrand`:

- `Bifunctor`
- `Bifoldable`

TryTrampoline provides all of these capabilities through inherent methods (`map`, `bind`, `bimap`, `map_err`, `lift2`, `tail_rec_m`, etc.) but not through the HKT trait system. This is a direct consequence of having no brand.

### Correctness of existing implementations

The existing trait implementations are correct:

- **Semigroup** delegates to `lift2`, which uses `bind` + `map`. This correctly short-circuits on the first error, matching the "first error wins" semantics.
- **Monoid** provides `Ok(A::empty())`, which is the correct identity for the Result-lifted semigroup.
- **Deferrable** correctly delegates to `Trampoline::defer`, wrapping/unwrapping the newtype.

### Gap: no `From<Thunk<'static, A>>` for `TryTrampoline`

`TryThunk` has `From<Thunk<'a, A>>`, but TryTrampoline has no direct `From<Thunk<'static, A>>`. This is a minor gap; users can go through `Trampoline` first (`TryTrampoline::from(Trampoline::from(thunk))` would require a Trampoline::from(Thunk) which also does not exist, but the path through `TryThunk::from(thunk)` then `TryTrampoline::from(try_thunk)` works).

## 5. Error Handling Integration

### How errors compose with stack-safe recursion

The integration is well designed. The key insight is that the error channel is handled at the `TryTrampoline` wrapper level while stack safety is handled at the `Trampoline`/`Free` level. These concerns compose cleanly because:

1. **`bind` short-circuits on `Err`.** When the inner `Trampoline<Result<A, E>>` produces `Err(e)`, the continuation `f` is never called. Instead, `Trampoline::pure(Err(e))` is returned. This is an O(1) operation that does not accumulate unused continuations.

2. **`tail_rec_m` integrates both stepping and errors.** The step function returns `TryTrampoline<Step<S, A>, E>`, so errors can occur at any iteration and the loop terminates cleanly. The implementation uses the inner Trampoline's bind:
   ```rust
   result.0.bind(move |r| match r {
       Ok(Step::Loop(next)) => go(f, next),
       Ok(Step::Done(a)) => Trampoline::pure(Ok(a)),
       Err(e) => Trampoline::pure(Err(e)),
   })
   ```

3. **`catch` and `catch_with` are stack-safe.** Both use `Trampoline::bind` under the hood, so recovery from errors does not consume stack frames. The `catch_with` method is particularly notable because it allows changing the error type during recovery while remaining stack-safe.

### Potential concern: error accumulation

The `Semigroup` implementation uses first-error semantics (short-circuits on `self`'s error, ignoring `other`). There is no built-in mechanism for accumulating errors (e.g., `Validation`-style). This is a deliberate design choice matching `Result`'s standard behavior, but users needing error accumulation must handle it manually.

## 6. Relationship to TryThunk and Trampoline

### Code duplication

There is significant structural duplication between TryTrampoline and TryThunk. Both implement the same set of inherent methods with nearly identical signatures:

| Method | TryTrampoline | TryThunk |
|--------|---------------|----------|
| `ok` / `pure` / `err` | Yes | Yes |
| `new` / `defer` | Yes | Yes |
| `map` / `map_err` / `bimap` | Yes | Yes |
| `bind` / `catch` / `catch_with` | Yes | Yes |
| `lift2` / `then` | Yes | Yes |
| `evaluate` | Yes | Yes |
| `append` / `empty` | Yes | Yes |
| `into_inner` | Yes | Yes |
| `catch_unwind` / `catch_unwind_with` | Yes | Yes |
| `into_rc_try_lazy` / `into_arc_try_lazy` | Yes | Yes |

The implementations differ only in:
- The underlying type (`Trampoline` vs `Thunk`).
- Lifetime bounds (`'static` vs `'a`).
- Stack safety of the resulting operations.
- TryThunk has HKT implementations; TryTrampoline does not.

This duplication is a recurring pattern across the library's lazy evaluation hierarchy (Thunk/Trampoline, SendThunk, TryThunk/TryTrampoline, TrySendThunk). It is the cost of providing ergonomic wrapper types without HKT unification.

### Conversion web

The types form a complete conversion graph (for `'static` lifetimes):

```
TryThunk<'static, A, E>  <-->  TryTrampoline<A, E>
         ^                           ^
         |                           |
   Thunk<'static, A>          Trampoline<A>
         ^                           ^
         |                           |
      Result<A, E>    Lazy/TryLazy (various configs)
```

Both directions exist for TryThunk/TryTrampoline conversion:
- `TryTrampoline::from(TryThunk)` wraps the thunk evaluation in a Trampoline.
- `TryThunk::from(TryTrampoline)` evaluates the trampoline when the thunk is forced.

## 7. `'static` Requirement

### Origin

The `'static` bound is inherited from `Trampoline<A>`, which requires `A: 'static`, which in turn is inherited from `Free<ThunkBrand, A>`. The `Free` monad uses `Box<dyn Any>` for type erasure in its CatList of continuations. Since `Any` requires `'static`, every type flowing through `Free`'s bind chain must be `'static`.

### Impact on TryTrampoline

- Both `A` and `E` must be `'static`. This means no borrowed references can appear in either the success or error type.
- All closures passed to `map`, `bind`, `defer`, `catch`, etc. must be `'static`.
- This contrasts with `TryThunk<'a, A, E>`, which supports arbitrary lifetimes.

### Practical implications

The `'static` requirement is the primary reason to prefer `TryThunk` over `TryTrampoline` in contexts where:
- Borrowed data needs to flow through the computation.
- The computation is part of a larger structure with a limited lifetime.

Conversely, TryTrampoline is the right choice when:
- Stack safety is needed (deep recursion).
- All data is owned (common in recursive algorithms).
- The computation will be consumed in the same scope it is created (typical for trampolined recursion).

## 8. Documentation Quality

### Strengths

- **Module-level documentation** provides a clear overview with a working example.
- **Type-level documentation** clearly states the relationship to `Trampoline<Result<A, E>>`.
- **"When to Use" section** correctly directs users to TryThunk for HKT support and TryLazy for memoization.
- **Every public method has:**
  - Documentation attribute macros (`#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`).
  - Working doc-test examples.
- **`tail_rec_m`** has an especially good example showing the factorial pattern with error handling.
- **`catch_with`** documentation clearly explains how it differs from `catch` (different error type).
- **`into_arc_try_lazy`** documents why eager evaluation is necessary (inner closures are `!Send`).
- **Clone bound on `tail_rec_m`** is documented, with `arc_tail_rec_m` offered as the alternative.

### Weaknesses

- **No explicit algebraic properties section.** TryThunk's doc comment includes a section documenting monad laws and error short-circuit semantics. TryTrampoline lacks this. While the QuickCheck tests verify these laws, having them in the documentation would be valuable.
- **No "Limitations" section.** TryThunk documents that `Traversable` cannot be implemented; TryTrampoline lacks a similar section noting the absence of HKT support and the `'static` constraint.
- **No "Stack Safety" section.** Unlike TryThunk, which explicitly warns that bind chains are not stack-safe, TryTrampoline's documentation does not explicitly state that it IS stack-safe. This is mentioned in the module doc and the type doc, but a dedicated section analogous to TryThunk's "Stack Safety" heading would make the guarantee more prominent.
- **`catch` method documentation** says "Recovers from an error" but does not mention stack safety implications. Since `catch` is implemented via `Trampoline::bind`, it is stack-safe, and this should be noted.
- **The `bimap` method** lacks `#[inline]` unlike some other methods, though this is a minor inconsistency (it does have `#[inline]`; confirmed on line 282).

## 9. Issues, Limitations, and Design Flaws

### Issue 1: No `and_then` alias

In Rust's ecosystem, `and_then` is the conventional name for monadic bind on `Result`. Having only `bind` may be surprising to users coming from standard library idioms. TryThunk has the same gap, so this is at least consistent.

### Issue 2: No `unwrap_or` / `unwrap_or_else` convenience methods

Standard `Result` methods like `unwrap_or`, `unwrap_or_else`, `ok()`, `err()` are missing. While users can call `evaluate()` and use Result methods on the output, having these on TryTrampoline would reduce boilerplate for common patterns.

### Issue 3: No `flatten` method

There is no `flatten: TryTrampoline<TryTrampoline<A, E>, E> -> TryTrampoline<A, E>`. While this can be achieved via `bind(|x| x)`, an explicit `flatten` method improves discoverability.

### Issue 4: `Semigroup::append` does not short-circuit on the second operand's error

The current implementation is:

```rust
fn append(a: Self, b: Self) -> Self {
    a.lift2(b, Semigroup::append)
}
```

This correctly short-circuits on `a`'s error (since `lift2` uses `bind`), but if `a` succeeds and `b` fails, the error is propagated. This is correct behavior, but the doc comment says "Both computations are evaluated" which is only true when `a` succeeds.

### Issue 5: No `SendDeferrable` implementation

`TryTrampoline` is `!Send` (because the underlying `Free` monad uses `Box<dyn FnOnce>` without a `Send` bound). This is correctly not implementing `SendDeferrable`. However, there is no documentation explaining WHY it is `!Send`, which would help users understand when to reach for `TrySendThunk` instead.

### Issue 6: No `Evaluable` trait integration

If the library has an `Evaluable` trait (for types that can be forced to produce a value), TryTrampoline should implement it. The current `evaluate` method is inherent only.

### Limitation 1: No error type coercion

There is no built-in `map_into<E2>()` or similar method for cheaply converting error types when `E: Into<E2>`. Users must use `map_err(Into::into)` explicitly.

### Limitation 2: Cannot represent computations that diverge on purpose

Because `evaluate()` always returns `Result<A, E>`, there is no way to represent a computation that intentionally loops forever (e.g., for streaming or server loops). This is inherent to the design and not really a flaw, but worth noting.

## 10. Alternatives and Improvements

### Alternative 1: `EitherT`-style monad transformer

In Haskell, the equivalent would be `ExceptT e (Trampoline) a`. The library could define a generic `EitherT<F, A, E>` transformer that wraps `Kind<F, Result<A, E>>` and provides short-circuiting bind. This would:
- Eliminate the code duplication between TryThunk and TryTrampoline.
- Automatically provide HKT support when the base functor has it.
- Allow stacking with other transformers.

The downside is that monad transformers in Rust are verbose and have significant compile-time cost due to the HKT encoding. The current concrete newtype approach trades generality for usability.

### Alternative 2: Dedicated `TryFree<F, A, E>` with error baked in

Instead of wrapping `Free<F, Result<A, E>>`, define a `TryFree` that tracks the error type at the free monad level. This would allow the `bind` implementation to avoid wrapping/unwrapping `Result` at each step, potentially improving performance. However, it would require duplicating significant `Free` machinery and the performance difference would be negligible since the `Result` match is O(1).

### Improvement 1: Add algebraic properties documentation

Add a section documenting:
- Monad laws (left identity, right identity, associativity).
- Error short-circuit semantics.
- Semigroup/Monoid laws with Result lifting.

This is already verified by QuickCheck tests; it just needs documentation.

### Improvement 2: Add a "Limitations" section to the type doc

Document:
- No HKT brand (cannot participate in generic type-class-polymorphic code).
- `'static` restriction on both `A` and `E`.
- `!Send` (cannot cross thread boundaries).
- Not memoized (each `evaluate()` re-runs).
- Cannot implement `Traversable` (same reason as TryThunk: `FnOnce` is consumed).

### Improvement 3: Consider `or_else` alias for `catch`

The method `catch` is named to evoke exception-handling semantics. Adding `or_else` as an alias would connect it to Rust's `Result::or_else` convention, improving discoverability for users coming from std.

### Improvement 4: Add `try_map` for fallible transformations

A `try_map(f: FnOnce(A) -> Result<B, E>) -> TryTrampoline<B, E>` would avoid the pattern of `bind(|a| match f(a) { Ok(b) => TryTrampoline::ok(b), Err(e) => TryTrampoline::err(e) })`.

## 11. Test Coverage Assessment

The test suite is thorough (approximately 35 tests + 7 QuickCheck properties):

**Covered:**
- All constructors (`ok`, `err`, `new`, `pure`, `defer`).
- All combinators (`map`, `map_err`, `bimap`, `bind`, `catch`, `catch_with`, `lift2`, `then`).
- `tail_rec_m` and `arc_tail_rec_m` (success, error, stack safety).
- All `From` conversions (Trampoline, Lazy, TryLazy, TryThunk, Result).
- Round-trip conversions (TryThunk <-> TryTrampoline).
- `into_inner`.
- `Semigroup` / `Monoid` trait implementations.
- `catch_unwind` / `catch_unwind_with`.
- `into_rc_try_lazy` / `into_arc_try_lazy`.
- Functor laws (identity, composition) via QuickCheck.
- Monad laws (left identity, right identity, associativity) via QuickCheck.
- Error short-circuit via QuickCheck.
- Semigroup associativity via QuickCheck.
- Monoid identity laws via QuickCheck.
- `!Send` type compatibility (Rc).
- Stack safety with 100,000+ iterations for both `tail_rec_m` and `catch_with`.

**Not covered (minor gaps):**
- `Deferrable` trait usage (tested only through inherent `defer` method).
- `bimap` identity law via QuickCheck (only consistency with map/map_err is tested).
- Error case for `map` (that `map` on an `Err` value passes through unchanged).
- Error case for `bimap` functor laws via QuickCheck.
- Debug output test exists but only for `Ok`; missing `Err` case (both print `"TryTrampoline(<unevaluated>)"` so this is trivial).

## Summary

TryTrampoline is a well-designed, well-tested type that correctly fulfills its role as the stack-safe fallible computation type in the lazy evaluation hierarchy. Its main design trade-off, no HKT support in exchange for stack safety via `Free`/`'static`, is inherent to the approach and clearly documented (though it could be more explicit). The code duplication with TryThunk is the cost of providing ergonomic concrete types rather than abstract monad transformers. The primary areas for improvement are documentation (algebraic properties section, limitations section) and a few missing convenience methods. The implementation itself is correct, with all operations properly delegating to the underlying Trampoline to maintain stack safety.
