# Step 02: Core Types

## Goal
Implement the core types `Step`, `Thunk`, and `Free` that form the building blocks for the stack-safe `Task` monad.

## Files to Create
- `fp-library/src/types/step.rs`
- `fp-library/src/types/thunk.rs`
- `fp-library/src/types/free.rs`

## Files to Modify
- `fp-library/src/types.rs`

## Implementation Details

### Step
Represents a step in a tail-recursive computation.
```rust
pub enum Step<A, B> {
    Loop(A),
    Done(B),
}
```

### Thunk
A wrapper around a boxed closure `Box<dyn FnOnce() -> A + Send>`.
- **ThunkF**: A zero-sized struct representing the Functor for Thunk.
- **Runnable**: A trait for functors that can be "run" to produce a value.

### Free
The Free monad implementation using `CatList` for O(1) binds.
- **Val**: `Box<dyn Any + Send>` (Type erasure).
- **Cont**: `Box<dyn FnOnce(Val) -> Free<F, Val> + Send>`.
- **Variants**:
    - `Pure(A)`
    - `Roll(Apply!(F::Brand, Free<F, A>))`
    - `Bind { head: Box<Free<F, Val>>, conts: CatList<Cont<F>> }`

## Tests

### Step Tests
1.  **Mapping**: `map_loop`, `map_done`, `bimap`.
2.  **State**: `is_loop`, `is_done`.

### Thunk Tests
1.  **Execution**: Create a thunk, force it, verify result.
2.  **Send**: Verify `Thunk` is `Send`.

### Free Tests
1.  **Pure**: `Free::pure(x).run()` returns `x`.
2.  **Roll**: `Free::roll(thunk).run()` executes thunk.
3.  **Bind**: `Free::pure(x).flat_map(f).run()` works.
4.  **Trampoline**: Verify `run()` loop handles `Bind` variants correctly without recursion.

## Checklist
- [ ] Create `fp-library/src/types/step.rs`
    - [ ] Implement `Step` enum
    - [ ] Implement helper methods
    - [ ] Add unit tests
- [ ] Create `fp-library/src/types/thunk.rs`
    - [ ] Implement `Thunk` struct
    - [ ] Implement `ThunkF` struct (marker)
    - [ ] Implement `Runnable` trait
    - [ ] Add unit tests
- [ ] Create `fp-library/src/types/free.rs`
    - [ ] Define `Val` and `Cont` types
    - [ ] Implement `Free` enum
    - [ ] Implement `pure`, `roll`, `flat_map` (with type erasure)
    - [ ] Implement `run` (trampoline loop)
    - [ ] Implement `Drop` to prevent stack overflow on drop
    - [ ] Add unit tests
- [ ] Update `fp-library/src/types.rs`
