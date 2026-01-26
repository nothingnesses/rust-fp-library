# Step 03: Task (Stack-Safe Computation)

## Goal
Implement `Task` and `TryTask`, the stack-safe computation types. `Task` is built on `Free<ThunkF, A>` and guarantees stack safety for deep recursion and long bind chains.

## Files to Create
- `fp-library/src/types/task.rs`
- `fp-library/src/types/try_task.rs`
- `fp-library/tests/stack_safety.rs`

## Files to Modify
- `fp-library/src/types.rs`

## Implementation Details

### Task
A wrapper around `Free<ThunkF, A>`.
- **Constraint**: `A: 'static + Send` (due to `Free`'s type erasure).
- **Constructors**: `now`, `later`, `always`, `defer`.
- **Combinators**: `flat_map`, `map`, `map2`, `and_then`.
- **Recursion**: `tail_rec_m` (standalone method, not trait impl).

```rust
pub struct Task<A> {
    inner: Free<ThunkF, A>,
}
```

### TryTask
A wrapper around `Task<Result<A, E>>`.
- **Constructors**: `ok`, `err`, `try_later`.
- **Combinators**: `map`, `map_err`, `and_then`, `or_else`.

```rust
pub struct TryTask<A, E> {
    inner: Task<Result<A, E>>,
}
```

## Tests

### Task Tests
1.  **Basic Execution**: `now`, `later`, `run`.
2.  **Defer**: Verify `defer` delays execution.
3.  **FlatMap**: Chain multiple operations.
4.  **Tail Recursion**: Implement factorial or fibonacci using `tail_rec_m`.

### TryTask Tests
1.  **Success/Failure**: Verify `ok` and `err` paths.
2.  **Combinators**: Verify `map` only affects success, `map_err` only affects error.
3.  **Short-circuiting**: Verify `and_then` stops at first error.

### Stack Safety Tests (`tests/stack_safety.rs`)
1.  **Deep Recursion**: `tail_rec_m` with 1,000,000 iterations.
2.  **Deep Bind Chain**: 10,000 left-associated `flat_map` calls.
3.  **Deep Defer**: 10,000 nested `defer` calls.

## Checklist
- [ ] Create `fp-library/src/types/task.rs`
    - [ ] Implement `Task` struct
    - [ ] Implement constructors (`now`, `later`, `always`, `defer`)
    - [ ] Implement combinators (`flat_map`, `map`, etc.)
    - [ ] Implement `run`
    - [ ] Implement `tail_rec_m` and `tail_rec_m_shared`
    - [ ] Add unit tests
- [ ] Create `fp-library/src/types/try_task.rs`
    - [ ] Implement `TryTask` struct
    - [ ] Implement constructors and combinators
    - [ ] Implement `run`
    - [ ] Add unit tests
- [ ] Create `fp-library/tests/stack_safety.rs`
    - [ ] Add deep recursion tests
    - [ ] Add deep bind chain tests
- [ ] Update `fp-library/src/types.rs`
