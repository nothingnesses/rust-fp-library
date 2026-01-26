# Step 06: HKT Integration

## Goal
Integrate the new types with the library's Higher-Kinded Types (HKT) system. This involves defining brands, implementing the `MonadRec` trait, and implementing standard type classes (`Functor`, `Monad`, etc.) for `Eval` and `Memo`.

## Important: Task Does NOT Get HKT Integration

**`Task<A>` cannot implement HKT traits** due to a fundamental conflict:

- HKT trait methods require working for any lifetime `'a` (e.g., `fn bind<'a, A: 'a, B: 'a, ...>`)
- Task requires `A: 'static` due to type erasure via `Box<dyn Any>`

These constraints are mutually exclusive. `'static` is a specific lifetime, not "any `'a`". Therefore:

- **No `TaskBrand`** is defined
- Task provides standalone `tail_rec_m` methods instead of implementing the `MonadRec` trait
- Use `Eval<'a, A>` when you need HKT polymorphism
- Use `Task<A>` when you need guaranteed stack safety

## Files to Create
- `fp-library/src/classes/monad_rec.rs`
- `fp-library/src/classes/ref_functor.rs` (new trait for reference-returning functors)

## Files to Modify
- `fp-library/src/brands.rs`
- `fp-library/src/classes.rs`
- `fp-library/src/types/eval.rs` (to add trait impls)
- `fp-library/src/types/memo.rs` (to add trait impls)
- `fp-library/src/types/thunk.rs` (to add trait impls)
- `fp-library/src/types/free.rs` (to add trait impls)

## Implementation Details

### Brands
Define marker types for HKT dispatch.
- `EvalBrand`: For `Eval<'a, A>`.
- `ThunkFBrand`: For `Thunk<A>`.
- `FreeBrand<F>`: For `Free<F, A>`.
- `MemoBrand<Config>`: For `Memo<A, Config>`.

**Note**: No `TaskBrand` or `TryTaskBrand` — Task cannot implement HKT traits.

### RefFunctor Trait
A variant of `Functor` for types where `map` receives/returns references. Required because `Memo::get()` returns `&A`, not `A`.

```rust
pub trait RefFunctor {
    fn map_ref<'a, B: 'a, A: 'a, F>(f: F, fa: Self::Of<'a, A>) -> Self::Of<'a, B>
    where
        F: FnOnce(&A) -> B + 'a;
}
```

### MonadRec
A trait for monads supporting tail recursion.
```rust
pub trait MonadRec: Monad {
    fn tail_rec_m<'a, A: 'a, B: 'a, F>(f: F, a: A) -> Apply!(Self::Brand, B)
    where
        F: Fn(A) -> Apply!(Self::Brand, Step<A, B>) + Clone + 'a;
}
```

#### Clone Bound Rationale

The `Clone` bound on `F` is necessary because:
1. Each recursive step needs to pass `f` to the next iteration
2. In trampolined implementations, `f` must be moved into closures multiple times (once per `defer` or continuation)
3. Most closures naturally implement `Clone` when their captures do

For closures that cannot implement `Clone`, use `tail_rec_m_shared` which wraps `f` in `Arc` internally (with a small performance cost).

### Trait Implementations
- **Eval**: `Functor`, `Pointed`, `Semimonad`, `Monad`, `MonadRec`, `Foldable`.
- **Memo**: `RefFunctor` (since `get` returns reference).
- **ThunkF**: `Functor`.
- **Free**: `Functor`, `Pointed`, `Semimonad`, `Monad`.

**Note**: `Eval`'s `MonadRec` implementation is NOT stack-safe for deep recursion (~8000 call limit). For truly stack-safe deep recursion, use `Task::tail_rec_m` directly.

## Tests

### HKT Tests
1.  **Generic Functions**: Write a function generic over `Monad` and use it with `Eval`.
2.  **MonadRec**: Verify `Eval` implements `MonadRec` (even if not stack-safe for deep recursion, it should work for shallow).
3.  **RefFunctor**: Verify `Memo` works with `RefFunctor`.

## Checklist
- [ ] Update `fp-library/src/brands.rs`
    - [ ] Add `EvalBrand`
    - [ ] Add `ThunkFBrand`
    - [ ] Add `FreeBrand`
    - [ ] Add `MemoBrand`
    - [ ] Document: NO `TaskBrand` (Task cannot implement HKT traits due to `'static` requirement)
- [ ] Create `fp-library/src/classes/ref_functor.rs`
    - [ ] Define `RefFunctor` trait for reference-returning map operations
    - [ ] Add documentation explaining why standard `Functor` doesn't work for `Memo`
- [ ] Create `fp-library/src/classes/monad_rec.rs`
    - [ ] Define `MonadRec` trait with `Clone` bound on `F`
    - [ ] Document Clone bound rationale in doc comments
    - [ ] Implement `tail_rec_m` free function
    - [ ] Implement `tail_rec_m_shared` (Arc-wrapped for non-Clone closures)
- [ ] Update `fp-library/src/classes.rs` to export `monad_rec` and `ref_functor`
- [ ] Implement traits for `Eval` in `src/types/eval.rs`
    - [ ] `Functor`, `Pointed`, `Semimonad`, `Monad`
    - [ ] `MonadRec` (note: NOT stack-safe for deep recursion)
    - [ ] `Foldable` (with `fold_right`, `fold_left`, `fold_map`)
- [ ] Implement traits for `Memo` in `src/types/memo.rs`
    - [ ] `RefFunctor`
- [ ] Implement traits for `Thunk` in `src/types/thunk.rs`
    - [ ] `Functor`
- [ ] Implement traits for `Free` in `src/types/free.rs`
    - [ ] `Functor`, `Pointed`, `Semimonad`, `Monad`

## Scope Notes

The following types intentionally do NOT get HKT brands:
- `Task<A>` / `TryTask<A, E>` — requires `'static`, incompatible with HKT's `for<'a>` bounds
- `TryEval<'a, A, E>` / `TryMemo<A, E, Config>` — fallible variants are out of scope for initial HKT integration
