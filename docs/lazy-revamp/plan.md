# Lazy Evaluation Revamp Plan

This document serves as the entry point for the complete overhaul of the lazy evaluation system in `fp-library`. This is a **breaking change** that replaces the existing `Lazy` implementation with a new dual-type architecture (`Task`/`Eval`) and a separate memoization layer (`Memo`).

## Implementation Steps

| Step | Description                                                                          | Link                                  |
| :--- | :----------------------------------------------------------------------------------- | :------------------------------------ |
| 1    | **Data Structures**<br>Implement `CatQueue` and `CatList` for O(1) operations.       | [Step 1](./step-1-data-structures.md) |
| 2    | **Core Types**<br>Implement `Step`, `Thunk`, and `Free` monad.                       | [Step 2](./step-2-core-types.md)      |
| 3    | **Task (Stack-Safe)**<br>Implement `Task` and `TryTask` for deep recursion.          | [Step 3](./step-3-task.md)            |
| 4    | **Eval (HKT-Compatible)**<br>Implement `Eval` and `TryEval` for generic composition. | [Step 4](./step-4-eval.md)            |
| 5    | **Memoization**<br>Implement `Memo` and `TryMemo` using `LazyCell`/`LazyLock`.       | [Step 5](./step-5-memo.md)            |
| 6    | **HKT Integration**<br>Implement brands, `MonadRec`, and type class instances.       | [Step 6](./step-6-hkt-integration.md) |
| 7    | **Cleanup & Integration**<br>Remove old `Lazy` types and finalize integration.       | [Step 7](./step-7-cleanup.md)         |

## File Operations

### Files to Create

| File Path                               | Purpose                                        |
| :-------------------------------------- | :--------------------------------------------- |
| `fp-library/src/types/cat_queue.rs`     | O(1) amortized double-ended queue              |
| `fp-library/src/types/cat_list.rs`      | O(1) catenable list                            |
| `fp-library/src/types/step.rs`          | `Step` enum for tail recursion                 |
| `fp-library/src/types/thunk.rs`         | `Thunk` and `ThunkF` types                     |
| `fp-library/src/types/free.rs`          | `Free` monad implementation                    |
| `fp-library/src/types/task.rs`          | `Task` implementation                          |
| `fp-library/src/types/try_task.rs`      | `TryTask` implementation                       |
| `fp-library/src/types/eval.rs`          | `Eval` implementation                          |
| `fp-library/src/types/try_eval.rs`      | `TryEval` implementation                       |
| `fp-library/src/types/memo.rs`          | `Memo` and `MemoConfig` implementation         |
| `fp-library/src/types/try_memo.rs`      | `TryMemo` implementation                       |
| `fp-library/src/classes/monad_rec.rs`   | `MonadRec` trait definition                    |
| `fp-library/src/classes/ref_functor.rs` | `RefFunctor` trait for reference-returning map |
| `fp-library/tests/stack_safety.rs`      | Tests for deep recursion and stack safety      |

### Files to Modify

| File Path                   | Purpose                                                                         |
| :-------------------------- | :------------------------------------------------------------------------------ |
| `fp-library/src/types.rs`   | Export new modules, remove `lazy`                                               |
| `fp-library/src/classes.rs` | Export `monad_rec` and `ref_functor`                                            |
| `fp-library/src/brands.rs`  | Add `EvalBrand`, `ThunkFBrand`, `FreeBrand`, `MemoBrand` (note: no `TaskBrand`) |
| `fp-library/src/lib.rs`     | Update documentation and module structure if needed                             |

### Files to Delete

| File Path                      | Purpose                                               |
| :----------------------------- | :---------------------------------------------------- |
| `fp-library/src/types/lazy.rs` | Old implementation (replaced by `Memo`/`Eval`/`Task`) |

## Implementation Log

**Update these sections as implementation progresses, including information about any deviations from the original plans, blockers and open questions**

### Decisions & Rationale for Plan Deviations

| Decision            | Rationale                                                                                                                                    |
| :------------------ | :------------------------------------------------------------------------------------------------------------------------------------------- |
| **Breaking Change** | Backwards compatibility is explicitly NOT a goal. The old `Lazy` type is being removed to allow for a cleaner, more correct design.          |
| **Std Lazy Types**  | Using `std::cell::LazyCell` and `std::sync::LazyLock` (Rust 1.80+) for memoization to leverage standard library correctness and performance. |
| **Two-Type Split**  | Separating `Task` (stack-safe, `'static`) and `Eval` (HKT, borrowed) to resolve the conflict between stack safety and HKT requirements.      |
| **O(1) List Len**   | `CatList` tracks its length in O(1) to support efficient size checks, adding a small memory overhead per node but improving performance.     |
| **Thunk Not Send**  | `Thunk` (and thus `Task`) is NOT `Send`. This resolves a conflict where `Functor::map` cannot enforce `Send` on the closure, but `Thunk` required it. |
| **Free Static**     | `Free` is strictly `'static` (`F: 'static`, `A: 'static`) to allow type erasure using `Box<dyn Any>`.                                        |
| **Free Struct**     | `Free` is implemented as a struct wrapping `Option<FreeInner>` to safely handle `Drop` recursion and destructuring without `unsafe` code, at the cost of small runtime overhead. |
| **Safe Free**       | Refactored `Free` to remove all `unsafe` code, prioritizing safety and auditability over the zero-cost abstraction of `ManuallyDrop`. |
| **Task Not Send**   | `Task` is not `Send` because `Thunk` is not `Send`. Consequently, `Send` bounds were removed from closures in `Task` and `TryTask` combinators (`flat_map`, `map`, etc.) to allow capturing `Task` instances (e.g., in `map2`). `A` still requires `Send` as per original plan (though strictly `Free` only requires `'static`). |
| **Eval Lifetime Bounds** | `Eval` and `TryEval` require explicit lifetime bounds on type parameters (e.g., `A: 'a`) in `impl` blocks to satisfy Rust's borrow checker when using `Box<dyn FnOnce() -> A + 'a>`. |
| **MemoConfig Associated Types** | `MemoConfig` uses associated types `Init` and `TryInit` instead of generic `F` in `new_lazy` to support different bounds (`Send` vs `!Send`) for `RcMemoConfig` and `ArcMemoConfig` while keeping the trait object safe. |
| **Memo Lifetimes**  | `Memo` and `TryMemo` now take a lifetime parameter `'a` (e.g., `Memo<'a, A>`) instead of requiring `A: 'static`. This allows `Memo` to be used with `Eval` (which supports borrowing) and to implement `RefFunctor` correctly. |
| **MemoConfig Lifetimes** | `MemoConfig` associated types (`Lazy`, `TryLazy`, `Init`, `TryInit`) now take a lifetime parameter `'a` to support non-static memoized values. |
| **Free No HKT**     | `Free` does not implement HKT traits (`Functor`, `Monad`, etc.) because it requires `A: 'static` for type erasure, which conflicts with the HKT `Kind` trait requiring support for any lifetime `'a`. |
| **Memo Send Bounds** | `Memo::from_task` and `TryMemo::from_try_task` require `A: Send` because `Task` implementation requires `Send` for its operations. |
| **ArcMemo Send+Sync** | `Memo::into_try` for `ArcMemoConfig` requires `A: Send + Sync` because `Arc<LazyLock<...>>` requires the inner value to be `Send + Sync` to be `Send`. |
| **Loom Removal**    | Removed `loom` dependency and associated tests. Since `Memo` uses `std::sync::LazyLock` (standard library), we rely on its correctness rather than verifying synchronization primitives with `loom`. Basic thread-safety is verified with `std::thread` tests. |
| **Once/Cell Removal** | Removed `Once`, `OnceCell`, and `OnceLock` wrappers and brands. These were largely superseded by `Memo` and the move to standard library types, simplifying the API surface. |
| **TryClass Removal** | Removed `TrySemigroup` and `TryMonoid` traits as they were underutilized and added unnecessary complexity. |
| **Thunk Brand**     | Renamed `ThunkFBrand` to `ThunkBrand` for simplicity, as it's the primary brand for `Thunk`. |
| **Method Names**    | Renamed `flat_map` to `bind`, `now` to `pure`, `later` to `new`, and `force` to `run` to align with the library's standard naming conventions and `Runnable` trait. |
| **Conversions**     | Replaced specific conversion methods (e.g., `from_memo`, `into_try`) with standard `From` trait implementations for better ergonomics and idiomatic Rust. |
| **Step Typeclasses**| Implemented `Functor`, `Bifunctor`, `Foldable`, `Traversable`, etc., for `Step` to make it a first-class citizen in the ecosystem. |
| **Runnable Trait**  | Added `Runnable` trait to abstract over types that can be executed (like `Thunk`, `Task`, `Eval`), replacing ad-hoc `run` or `force` methods. |
| **Use VecDeque for CatList**  | Benchmarks showed better performance using VecDeque over CatQueue, so Catlist has been refactored to use VecDeque and CatQueue has been removed. |
| **Replace Thunk with Eval**  | They basically do the same thing. |

### Blockers

| Blocker    | Status | Resolution |
| :--------- | :----- | :--------- |
| (None yet) |        |            |

### Open Questions

| Question   | Status | Answer |
| :--------- | :----- | :----- |
| (None yet) |        |        |
