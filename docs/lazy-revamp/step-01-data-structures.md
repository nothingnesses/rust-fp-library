# Step 01: Data Structures

## Goal
Implement the foundational data structures `CatQueue` and `CatList` required for the "Reflection without Remorse" optimization in the Free monad. These structures provide O(1) amortized append and uncons operations, enabling stack-safe left-associated binds.

## Files to Create
- `fp-library/src/types/cat_queue.rs`
- `fp-library/src/types/cat_list.rs`

## Files to Modify
- `fp-library/src/types.rs` (to expose the new modules)

## Implementation Details

### CatQueue
A double-ended queue implemented using two `Vec`s ("banker's queue").
- **Front**: Elements ready to be dequeued.
- **Back**: Elements recently enqueued (in reverse order).
- **Invariant**: `front` contains elements in FIFO order (head at end), `back` contains elements in LIFO order.

```rust
pub struct CatQueue<A> {
    front: Vec<A>,
    back: Vec<A>,
}
```

### CatList
A catenable list supporting O(1) concatenation.
- **Nil**: Empty list.
- **Cons**: Head element + Queue of sublists.

```rust
pub enum CatList<A> {
    Nil,
    Cons(A, CatQueue<CatList<A>>),
}
```

## Tests

### CatQueue Tests
1.  **Basic Operations**: `empty`, `is_empty`, `len`, `singleton`.
2.  **FIFO Behavior**: `snoc` multiple items, `uncons` them, verify order.
3.  **Amortization Logic**: `snoc` many items, `uncons` all. Verify `front` is refilled from `back`.
4.  **Double-Ended**: `cons` (push front) and `unsnoc` (pop back) behavior.

### CatList Tests
1.  **Basic Operations**: `empty`, `singleton`, `cons`, `snoc`.
2.  **Concatenation**: `append` two lists, verify order.
3.  **Flattening**: Create a nested structure via multiple appends, `uncons` all elements to verify `flatten_queue` logic works correctly and iteratively.
4.  **Iteration**: Verify `IntoIterator` works correctly.

## Checklist
- [ ] Create `fp-library/src/types/cat_queue.rs`
    - [ ] Implement struct and `new`/`empty`
    - [ ] Implement `snoc`, `cons`
    - [ ] Implement `uncons`, `unsnoc` (with reversal logic)
    - [ ] Implement `IntoIterator`
    - [ ] Add unit tests
- [ ] Create `fp-library/src/types/cat_list.rs`
    - [ ] Implement enum `CatList`
    - [ ] Implement `link` helper
    - [ ] Implement `append`
    - [ ] Implement `uncons` with iterative `flatten_queue`
    - [ ] Implement `IntoIterator`
    - [ ] Add unit tests
- [ ] Update `fp-library/src/types.rs` to export `cat_queue` and `cat_list`
