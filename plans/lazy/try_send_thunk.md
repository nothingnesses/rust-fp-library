# Analysis: `try_send_thunk.rs`

**File:** `fp-library/src/types/try_send_thunk.rs`
**Role:** `TrySendThunk<'a, A, E>`, the thread-safe fallible thunk.

## Design

`TrySendThunk<'a, A, E>` is a newtype over `SendThunk<'a, Result<A, E>>`. It mirrors `TryThunk` but with `Send` on the inner closure.

Like `SendThunk`, `TrySendThunk` has **no HKT brand** because the trait signatures lack `Send` bounds on closures.

## Assessment

### Correct decisions

1. **Newtype over `SendThunk<Result<A, E>>`.** Consistent with `TryThunk` over `Thunk<Result<A, E>>`.
2. **Inherent `tail_rec_m` and `arc_tail_rec_m`.** Since there is no HKT brand, these must be inherent methods. This is the correct approach.
3. **No brand.** Well-documented rationale; the HKT trait closure signatures prevent `Send` guarantees.

### Issues

#### 1. Massive duplication with `try_thunk.rs`

Nearly every method (`new`, `pure`, `ok`, `err`, `defer`, `bind`, `map`, `map_err`, `catch`, `catch_with`, `bimap`, `evaluate`, `lift2`, `then`, `catch_unwind`) is duplicated between the two files, differing only in `Send` bounds and the base type. This is approximately 1600 lines of largely parallel code.

A generic wrapper `TryThunkBase<'a, A, E, Inner>` parameterized over the inner thunk type could reduce this, but Rust's type system makes it difficult to abstract over "closure may or may not need Send." The current duplication is pragmatic but a maintenance burden.

**Impact:** Moderate. Any behavioral change or bugfix must be replicated in both files.

#### 2. `Deferrable::defer` eagerly evaluates (inherited from `SendThunk`)

Same issue as `SendThunk`: the `Deferrable` trait's closure is not `Send`, so `TrySendThunk` evaluates eagerly.

**Impact:** Moderate. Same semantic mismatch as in `SendThunk`.

#### 3. `TrySendThunk::defer` (inherent) flattens eagerly

```rust
pub fn defer(f: impl FnOnce() -> TrySendThunk<'a, A, E> + Send + 'a) -> Self {
    TrySendThunk(SendThunk::new(move || f().evaluate()))
}
```

This differs from `TryThunk::defer(f)` which does `Thunk::defer(move || f().0)` (structural composition). The `TrySendThunk` version evaluates the inner thunk immediately when the outer closure runs, rather than composing them structurally. This is likely forced by the `Send` constraint.

**Impact:** Low. Functionally correct but slightly less efficient for deeply nested deferred constructions.

#### 4. Direct field access to `TryLazy` internals

`into_arc_try_lazy` accesses `TryLazy.0` directly rather than using a constructor. This couples the implementation to `TryLazy`'s internal representation.

**Impact:** Low. Both types are in the same crate, so this is safe but somewhat fragile.

#### 5. `TryThunk` has inherent `tail_rec_m` but `TryThunk` does not (asymmetry)

`TrySendThunk` provides `tail_rec_m` as an inherent method. `TryThunk` does not, relying instead on the HKT `MonadRec` trait. This means `TryThunk` with non-`'static` error types has no access to `tail_rec_m` at all.

**Impact:** Moderate. This is an asymmetry that should be resolved by adding inherent `tail_rec_m` to `TryThunk`.

## Strengths

- Consistent parallel structure with `TryThunk`.
- Inherent `tail_rec_m` and `arc_tail_rec_m` fill the HKT gap.
- Correct `Send`/`!Sync` semantics.
- Well-documented conversion semantics (eager when crossing Send boundary).
