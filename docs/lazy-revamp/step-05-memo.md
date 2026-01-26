# Step 05: Memoization

## Goal
Implement `Memo` and `TryMemo`, the memoization layer that caches results using `std::cell::LazyCell` and `std::sync::LazyLock`.

## Files to Create
- `fp-library/src/types/memo.rs`
- `fp-library/src/types/try_memo.rs`

## Files to Modify
- `fp-library/src/types.rs`

## Implementation Details

### MemoConfig
A trait abstracting over `Rc`/`Arc` and `LazyCell`/`LazyLock`.
- **RcMemoConfig**: Uses `Rc<LazyCell<...>>`.
- **ArcMemoConfig**: Uses `Arc<LazyLock<...>>`.

```rust
pub trait MemoConfig: 'static {
    type Lazy<A: 'static>: Clone;
    type TryLazy<A: 'static, E: 'static>: Clone;
    // ... new_lazy, force, etc.
}
```

### Memo
A memoized value.
- **Constructors**: `new`, `from_task`, `from_eval`.
- **Access**: `get(&self) -> &A`.
- **Conversions**: `into_try<E>` (converts to `TryMemo` that always succeeds).

```rust
pub struct Memo<A, Config: MemoConfig = RcMemoConfig> {
    inner: Config::Lazy<A>,
}
```

### TryMemo
A memoized fallible value.
- **Constructors**: `new`, `from_try_task`, `from_try_eval`, `catch_unwind`.
- **Access**: `get(&self) -> Result<&A, &E>`.
- **`catch_unwind`**: Static factory method that wraps a potentially-panicking thunk and converts panics to errors (opt-in panic catching).

```rust
pub struct TryMemo<A, E, Config: MemoConfig = RcMemoConfig> {
    inner: Config::TryLazy<A, E>,
}
```

## Tests

### Memo Tests
1.  **Caching**: Verify computation runs only once.
2.  **Sharing**: Verify clones share the cache.
3.  **Thread Safety**: Verify `ArcMemo` works across threads (compile check + runtime test).
4.  **Conversion**: Verify `from_task` and `from_eval` work.

### TryMemo Tests
1.  **Caching**: Verify result (success or error) is cached.
2.  **Sharing**: Verify clones share the result.

## Checklist
- [ ] Create `fp-library/src/types/memo.rs`
    - [ ] Implement `MemoConfig` trait
    - [ ] Implement `RcMemoConfig` and `ArcMemoConfig`
    - [ ] Implement `Memo` struct
    - [ ] Implement `new`, `get`
    - [ ] Implement `from_task` and `from_eval`
    - [ ] Implement `into_try<E>` conversion to `TryMemo`
    - [ ] Add type aliases `RcMemo`, `ArcMemo`
    - [ ] Add unit tests
- [ ] Create `fp-library/src/types/try_memo.rs`
    - [ ] Implement `TryMemo` struct
    - [ ] Implement `new`, `get`
    - [ ] Implement `from_try_task` and `from_try_eval`
    - [ ] Implement `catch_unwind` static factory method
    - [ ] Add type aliases `RcTryMemo`, `ArcTryMemo`
    - [ ] Add unit tests
- [ ] Update `fp-library/src/types.rs`
