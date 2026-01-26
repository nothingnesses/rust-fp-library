# Lazy Evaluation Revamp Plan

This document serves as the entry point for the complete overhaul of the lazy evaluation system in `fp-library`. This is a **breaking change** that replaces the existing `Lazy` implementation with a new dual-type architecture (`Task`/`Eval`) and a separate memoization layer (`Memo`).

## Implementation Steps

| Step | Description | Link |
| :--- | :--- | :--- |
| 01 | **Data Structures**<br>Implement `CatQueue` and `CatList` for O(1) operations. | [Step 01](./step-01-data-structures.md) |
| 02 | **Core Types**<br>Implement `Step`, `Thunk`, and `Free` monad. | [Step 02](./step-02-core-types.md) |
| 03 | **Task (Stack-Safe)**<br>Implement `Task` and `TryTask` for deep recursion. | [Step 03](./step-03-task.md) |
| 04 | **Eval (HKT-Compatible)**<br>Implement `Eval` and `TryEval` for generic composition. | [Step 04](./step-04-eval.md) |
| 05 | **Memoization**<br>Implement `Memo` and `TryMemo` using `LazyCell`/`LazyLock`. | [Step 05](./step-05-memo.md) |
| 06 | **HKT Integration**<br>Implement brands, `MonadRec`, and type class instances. | [Step 06](./step-06-hkt-integration.md) |
| 07 | **Cleanup & Integration**<br>Remove old `Lazy` types and finalize integration. | [Step 07](./step-07-cleanup.md) |

## File Operations

### Files to Create

| File Path | Purpose |
| :--- | :--- |
| `fp-library/src/types/cat_queue.rs` | O(1) amortized double-ended queue |
| `fp-library/src/types/cat_list.rs` | O(1) catenable list |
| `fp-library/src/types/step.rs` | `Step` enum for tail recursion |
| `fp-library/src/types/thunk.rs` | `Thunk` and `ThunkF` types |
| `fp-library/src/types/free.rs` | `Free` monad implementation |
| `fp-library/src/types/task.rs` | `Task` implementation |
| `fp-library/src/types/try_task.rs` | `TryTask` implementation |
| `fp-library/src/types/eval.rs` | `Eval` implementation |
| `fp-library/src/types/try_eval.rs` | `TryEval` implementation |
| `fp-library/src/types/memo.rs` | `Memo` and `MemoConfig` implementation |
| `fp-library/src/types/try_memo.rs` | `TryMemo` implementation |
| `fp-library/src/classes/monad_rec.rs` | `MonadRec` trait definition |
| `fp-library/src/classes/ref_functor.rs` | `RefFunctor` trait for reference-returning map |
| `fp-library/tests/stack_safety.rs` | Tests for deep recursion and stack safety |

### Files to Modify

| File Path | Purpose |
| :--- | :--- |
| `fp-library/src/types.rs` | Export new modules, remove `lazy` |
| `fp-library/src/classes.rs` | Export `monad_rec` and `ref_functor` |
| `fp-library/src/brands.rs` | Add `EvalBrand`, `ThunkFBrand`, `FreeBrand`, `MemoBrand` (note: no `TaskBrand`) |
| `fp-library/src/lib.rs` | Update documentation and module structure if needed |

### Files to Delete

| File Path | Purpose |
| :--- | :--- |
| `fp-library/src/types/lazy.rs` | Old implementation (replaced by `Memo`/`Eval`/`Task`) |

## Implementation Log

### Decisions & Rationale

| Decision | Rationale |
| :--- | :--- |
| **Breaking Change** | Backwards compatibility is explicitly NOT a goal. The old `Lazy` type is being removed to allow for a cleaner, more correct design. |
| **Std Lazy Types** | Using `std::cell::LazyCell` and `std::sync::LazyLock` (Rust 1.80+) for memoization to leverage standard library correctness and performance. |
| **Two-Type Split** | Separating `Task` (stack-safe, `'static`) and `Eval` (HKT, borrowed) to resolve the conflict between stack safety and HKT requirements. |

### Blockers

| Blocker | Status | Resolution |
| :--- | :--- | :--- |
| (None yet) | | |

### Open Questions

| Question | Status | Answer |
| :--- | :--- | :--- |
| (None yet) | | |
