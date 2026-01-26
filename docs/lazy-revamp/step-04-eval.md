# Step 04: Eval (HKT-Compatible Computation)

## Goal
Implement `Eval` and `TryEval`, the closure-based computation types. These types support Higher-Kinded Types (HKT) and borrowed references but are NOT stack-safe for deep recursion.

## Files to Create
- `fp-library/src/types/eval.rs`
- `fp-library/src/types/try_eval.rs`

## Files to Modify
- `fp-library/src/types.rs`

## Implementation Details

### Eval
A wrapper around a boxed closure `Box<dyn FnOnce() -> A + 'a>`.
- **Lifetime**: `'a` (allows borrowing).
- **No 'static**: Unlike `Task`, `Eval` works with non-static data.
- **Constructors**: `new`, `pure`, `defer`.
- **Combinators**: `flat_map`, `map`.
- **Conversions**: `into_try` (converts to `TryEval` that always succeeds).

```rust
pub struct Eval<'a, A> {
    thunk: Box<dyn FnOnce() -> A + 'a>,
}
```

### TryEval
A wrapper around `Box<dyn FnOnce() -> Result<A, E> + 'a>`.
- **Constructors**: `new`, `pure`, `ok`, `err`.
- **Combinators**: `flat_map`, `map`, `map_err`.

```rust
pub struct TryEval<'a, A, E> {
    thunk: Box<dyn FnOnce() -> Result<A, E> + 'a>,
}
```

## Tests

### Eval Tests
1.  **Basic Execution**: `new`, `pure`, `run`.
2.  **Borrowing**: Verify `Eval` can capture references (e.g., `&str`).
3.  **Composition**: Chain `map` and `flat_map`.
4.  **Defer**: Verify `defer` works.

### TryEval Tests
1.  **Success/Failure**: Verify `ok` and `err` paths.
2.  **Combinators**: Verify `map` and `map_err`.
3.  **Borrowing**: Verify capturing references works.

## Checklist
- [ ] Create `fp-library/src/types/eval.rs`
    - [ ] Implement `Eval` struct
    - [ ] Implement constructors (`new`, `pure`, `defer`)
    - [ ] Implement combinators (`flat_map`, `map`)
    - [ ] Implement `into_try<E>` conversion to `TryEval`
    - [ ] Implement `run`
    - [ ] Add unit tests (including borrowing tests)
- [ ] Create `fp-library/src/types/try_eval.rs`
    - [ ] Implement `TryEval` struct
    - [ ] Implement constructors and combinators
    - [ ] Implement `run`
    - [ ] Add unit tests
- [ ] Update `fp-library/src/types.rs`
