# TrySendThunk Analysis

## Overview

`TrySendThunk<'a, A, E>` is the fallible, thread-safe deferred computation type. It wraps `SendThunk<'a, Result<A, E>>` (a newtype), providing ergonomic combinators for error handling while preserving the `Send` invariant. It sits in the lazy evaluation hierarchy as the intersection of "fallible" and "thread-safe, non-memoized."

**File:** `fp-library/src/types/try_send_thunk.rs`

## Design Assessment

### Overall Verdict

The design is sound and consistent with the library's established patterns. The wrapper approach mirrors `TryThunk` (which wraps `Thunk<Result<A, E>>`), adapted for thread safety. The API surface is well-chosen, and the `Send` bounds are applied correctly throughout.

### Wrapper Approach: Consistent with TryThunk

| Aspect | TryThunk | TrySendThunk |
|--------|----------|--------------|
| Inner type | `Thunk<'a, Result<A, E>>` | `SendThunk<'a, Result<A, E>>` |
| Closure bound | `FnOnce() -> ... + 'a` | `FnOnce() -> ... + Send + 'a` |
| HKT support | Full (Functor, Monad, Foldable, etc.) | None (inherent methods only) |
| bimap | Via `Bifunctor` trait on `TryThunkBrand` | Inherent method |

This is the correct design. The wrapper delegates to `SendThunk` operations, and the `Send` requirement propagates naturally.

## Issues and Inconsistencies

### 1. Deferrable::defer Eagerly Evaluates (Correct but Subtle)

The `Deferrable` impl calls `f()` eagerly because the trait signature does not require `Send` on the closure. This is documented and consistent with `SendThunk`'s `Deferrable` impl. The `SendDeferrable` impl correctly defers via `SendThunk::new(move || f().evaluate())`. No issue here; just noting it is intentional.

### 2. TryThunk Has `bimap` Only via Bifunctor; TrySendThunk Has It as Inherent

`TryThunk` exposes `bimap` through the `Bifunctor` HKT trait on `TryThunkBrand`, not as an inherent method. `TrySendThunk` provides `bimap` as an inherent method since it cannot implement the HKT `Bifunctor` trait (closures are not `Send`-bounded in the trait).

This is the correct adaptation, but it creates a minor API asymmetry: users of `TryThunk` call `bimap` through the free function `bimap::<TryThunkBrand, ...>(f, g, thunk)`, while `TrySendThunk` users call `thunk.bimap(f, g)`. This is unavoidable given the `Send` constraint gap in the HKT traits, and the inherent method is strictly more ergonomic.

### 3. Missing `memoize` (Rc-based) Method

`TryThunk` has both `memoize()` (returns `RcTryLazy`) and `memoize_arc()` (returns `ArcTryLazy`). `TrySendThunk` only has `memoize_arc()`. This is correct because `RcTryLazy` is `!Send`, so converting a `Send` thunk to a `!Send` lazy value would lose thread safety. If a user truly wants this, they can evaluate the `TrySendThunk` and construct an `RcTryLazy` manually.

### 4. Semigroup: Both-Sides Eager Evaluation

The `Semigroup::append` impl evaluates both `a` and `b` before pattern matching:

```rust
TrySendThunk::new(move || match (a.evaluate(), b.evaluate()) {
    (Ok(a_val), Ok(b_val)) => Ok(Semigroup::append(a_val, b_val)),
    (Err(e), _) => Err(e),
    (_, Err(e)) => Err(e),
})
```

This evaluates `b` even if `a` fails. This is consistent with `TryThunk`'s `Semigroup::append`, which has the same behavior. It is a deliberate design choice (not short-circuiting), but it differs from `bind`/`then`/`lift2` which do short-circuit. The inconsistency is inherited from `TryThunk` and is worth documenting or reconsidering at the level of the entire hierarchy, not just this file.

**Potential improvement:** Short-circuit in `append` to avoid unnecessary work:

```rust
TrySendThunk::new(move || {
    let a_val = a.evaluate()?;
    let b_val = b.evaluate()?;
    Ok(Semigroup::append(a_val, b_val))
})
```

This would be more efficient and more consistent with the rest of the API. However, changing it for `TrySendThunk` alone would create an inconsistency with `TryThunk`. If changed, both should be changed together.

### 5. bind: Redundant `Send` Bounds on A, B, E

The `bind` method has explicit `where` clauses:

```rust
pub fn bind<B>(
    self,
    f: impl FnOnce(A) -> TrySendThunk<'a, B, E> + Send + 'a,
) -> TrySendThunk<'a, B, E>
where
    A: Send + 'a,
    B: Send + 'a,
    E: Send + 'a,
```

The `A: Send` and `E: Send` bounds are needed because `self.0.bind(...)` moves the `SendThunk` closure which eventually needs to move the `Result<A, E>` across the `Send` boundary. `B: Send` is needed because `f(a)` produces a `TrySendThunk` whose inner `SendThunk` stores `Box<dyn FnOnce() -> Result<B, E> + Send>`. So these bounds are technically correct.

However, `lift2` and `then` inherit these bounds transitively, leading to verbose signatures. This is the cost of not having HKT trait support; it is an inherent limitation, not a bug.

### 6. No `From<ArcTryLazy>` or `From<TryTrampoline>` Conversions

`TryThunk` has conversions from `TryLazy` (any config), `Lazy` (any config), `Thunk`, `Result`, and `TryTrampoline`. `TrySendThunk` has conversions from `TryThunk`, `Result`, and `SendThunk`.

Missing conversions that could be useful:
- `From<ArcTryLazy<'a, A, E>>` for `TrySendThunk<'a, A, E>` where `A: Clone + Send, E: Clone + Send`: Would clone the memoized result, analogous to `TryThunk`'s `From<TryLazy>`.
- `From<TryTrampoline<A, E>>` for `TrySendThunk<'static, A, E>` where `A: Send, E: Send`: `TryTrampoline` evaluates to a `Result`, which is `Send` if `A` and `E` are.

These are not critical but would improve interoperability.

### 7. No `Evaluable` Trait Implementation

Neither `TrySendThunk` nor `SendThunk` implement the `Evaluable` trait (if one exists in the codebase). Both rely on inherent `evaluate` methods. This is consistent across the `Send` types in the hierarchy.

## Documentation Quality

### Strengths

- The module-level doc comment is clear and correctly identifies this as the fallible counterpart to `SendThunk`.
- The struct-level documentation is thorough: it explains the wrapper, the HKT limitations, when to use it, algebraic properties, stack safety, and the `Traversable` limitation.
- Every method has `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` annotations.
- Doc examples are correct and testable.

### Issues

- The algebraic properties section mentions `TrySendThunk::ok` for right identity: `thunk.bind(TrySendThunk::ok).evaluate() == thunk.evaluate()`. This is correct for the specific case where `A: Send`, but `ok` requires `A: Send + 'a, E: Send + 'a`. A reader might not realize the bound requirement just from reading the doc. This is a minor clarity issue.
- The `memoize_arc` documentation correctly states it does NOT evaluate eagerly (unlike `TryThunk::memoize_arc`), which is a key advantage.

## Test Coverage

Test coverage is good. The test suite covers:
- Basic constructors (`ok`, `err`, `new`, `pure`, `defer`).
- Combinators (`map`, `map_err`, `bimap`, `bind`, `catch`, `lift2`, `then`).
- Error propagation in `map`, `bind`.
- Conversions (`From<TryThunk>`, `From<Result>`, `From<SendThunk>`).
- `catch_unwind` and `catch_unwind_with` (both panic and non-panic paths).
- `memoize_arc` with caching verification and thread safety.
- `Semigroup` and `Monoid` (including error case).
- `Send` static assertion and actual cross-thread usage.
- `Deferrable` and `SendDeferrable`.

### Missing Test Cases

- `bimap` on success propagation through error path (only tests Ok and Err individually).
- `Semigroup::append` where the second thunk fails but the first succeeds (only tests first-fails case).
- `catch` where recovery itself fails (returning a new `Err`).
- Chain of multiple `bind` calls (verifying composition).

## Summary of Recommendations

1. **Consider short-circuiting `Semigroup::append`** to avoid evaluating the second thunk on failure. This should be done for both `TryThunk` and `TrySendThunk` together for consistency.
2. **Add `From<ArcTryLazy>` conversion** for interoperability with the memoized layer.
3. **Add a few more edge-case tests**, particularly around `Semigroup` with second-operand failure, and `catch` with re-failure.
4. **No structural changes needed.** The wrapper approach, API surface, `Send` bounds, and documentation are all well-executed.
