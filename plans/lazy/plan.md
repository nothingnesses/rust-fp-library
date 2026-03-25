# Lazy Evaluation Hierarchy: Implementation Plan

Based on the [consolidated summary](summary.md). Phases are ordered by priority: correctness, design, missing features, documentation, performance, tests.

## Preserved Areas (Do Not Change)

The following are sound and must not be altered:

- `Deferrable` / `SendDeferrable` trait split (lifetime parameterization, `FnOnce` choice, `Sized` bound).
- `RefFunctor` / `SendRefFunctor` separation from `Functor`.
- `Thunk` as the lightweight HKT-compatible computation type, and all its HKT trait implementations.
- The newtype wrapper pattern for all fallible types (`TryThunk`, `TrySendThunk`, `TryTrampoline`, `TryLazy`).
- `Trampoline` as a newtype over `Free<ThunkBrand, A>`.
- `Free` monad's "Reflection without Remorse" CatList-based algorithm.
- `Evaluable` trait (keep it even though `ThunkBrand` is the sole implementor).
- `CatList` core data structure and its `VecDeque`-based spine.
- `Step<A, B>` naming and design.
- The overall brand hierarchy and dependency ordering (brands -> classes -> types).
- `SendThunk` not implementing HKT traits (correct given missing `Send` bounds on trait signatures).
- `LazyConfig` trait abstraction.

---

## Phase 1: Correctness Fixes

These are bugs or correctness concerns that could cause wrong behavior.

### 1.1 Short-circuit `Semigroup::append` for fallible types

**Summary ref:** 1.1

All three fallible types evaluate both operands before pattern matching, wasting work when the first is `Err`. Change to sequential evaluation so `b` is only evaluated when `a` is `Ok`.

Do all three together since the fix is identical in structure.

- [ ] **Task 1.1a:** Fix `TryThunk` `Semigroup::append`.
  - File: `fp-library/src/types/try_thunk.rs`
  - Change `match (a.evaluate(), b.evaluate())` to evaluate `a` first, short-circuit on `Err`, then evaluate `b`.
  - Complexity: small.
- [ ] **Task 1.1b:** Fix `TrySendThunk` `Semigroup::append`.
  - File: `fp-library/src/types/try_send_thunk.rs`
  - Same pattern as 1.1a.
  - Complexity: small.
- [ ] **Task 1.1c:** Fix `TryLazy` `Semigroup::append`.
  - File: `fp-library/src/types/try_lazy.rs`
  - Same pattern as 1.1a. Note: `TryLazy` returns `&A` from `evaluate()`, so the fix may need `.cloned()` only on success.
  - Complexity: small.

Tasks 1.1a, 1.1b, 1.1c are independent and can be parallelized.

### 1.2 Harden `Free::Drop` for deep continuation chains

**Summary ref:** 1.2

- [ ] **Task 1.2:** Extend the iterative `Drop` implementation to handle `CatList` continuations and `Wrap` variants.
  - File: `fp-library/src/types/free.rs`
  - The existing `Drop` handles `Bind` and `Map` chains but does not iteratively drop continuations (each a `Box<dyn FnOnce>` that may capture `Free` values) or `Wrap` variants.
  - Approach: after extracting the inner chain, also drain the `CatList` iteratively, and for `Wrap` variants, extract and loop rather than recursing.
  - Complexity: medium.
  - Depends on: nothing.

### 1.3 Add `Sync` bound to `ArcLazy::pure`

**Summary ref:** 1.4

- [ ] **Task 1.3:** Add `A: Sync` to the `ArcLazy::pure` signature.
  - File: `fp-library/src/types/lazy.rs`
  - Verify whether the compiler already enforces this via `Arc`'s auto-trait impls. If it does, add the bound anyway for clarity and documentation. If it does not, this is a soundness fix.
  - Complexity: small.

---

## Phase 2: Design Fixes and Inconsistencies

### 2.1 Remove unnecessary `Sync` bound on `SendDeferrable::send_defer`

**Summary ref:** 2.3

- [ ] **Task 2.1:** Remove `Sync` from the closure bound on `SendDeferrable::send_defer` and the corresponding free function.
  - File: `fp-library/src/classes/send_deferrable.rs`
  - Change `impl FnOnce() -> Self + Send + Sync + 'a` to `impl FnOnce() -> Self + Send + 'a`.
  - Verify all four implementations (`SendThunk`, `ArcLazy`, `TrySendThunk`, `ArcTryLazy`) still compile.
  - Complexity: small.

### 2.2 Fix `SendRefFunctor` / `RefFunctor` relationship

**Summary ref:** 2.1

**Decision: Option B (document limitation, keep traits independent).**

Option A (supertrait) is infeasible: `ArcLazy::new` requires `Box<dyn FnOnce() -> A + Send + 'a>`, so any `ref_map` implementation for `LazyBrand<ArcLazyConfig>` must require `Send` on the closure. But `RefFunctor::ref_map` accepts `impl FnOnce(&A) -> B + 'a` with no `Send` bound. There is no way to implement `RefFunctor` for `ArcLazy` without violating the trait contract. The comment at `lazy.rs:776-779` explicitly documents this constraint.

- [ ] **Task 2.2:** Document the limitation prominently in `SendRefFunctor` and `RefFunctor` trait docs. Explain that the traits are independent because `ArcLazy::new` requires `Send` on the closure, which `RefFunctor` cannot guarantee.
  - Files: `fp-library/src/classes/ref_functor.rs`, `fp-library/src/classes/send_ref_functor.rs`
  - Complexity: small.

### 2.3 Fix false documentation claim about `ArcLazy` implementing `RefFunctor`

**Summary ref:** 2.2

- [ ] **Task 2.3:** Correct the `SendRefFunctor` trait docs that claim `ArcLazy` implements both `RefFunctor` and `SendRefFunctor`.
  - File: `fp-library/src/classes/send_ref_functor.rs`
  - Also check `fp-library/src/classes/ref_functor.rs` for similar claims.
  - Depends on: 2.2 (the docs should reflect the decision to keep traits independent).
  - Complexity: small.

### 2.4 Change `rc_lazy_fix` and `arc_lazy_fix` to accept `FnOnce`

**Summary ref:** 2.5

- [ ] **Task 2.4:** Change the bound from `impl Fn(...)` to `impl FnOnce(...)` on both fix combinators.
  - File: `fp-library/src/types/lazy.rs`
  - Complexity: small.

### 2.5 Fix `TryLazy::map_err` unnecessary clone of success side

**Summary ref:** 2.6

- [ ] **Task 2.5:** Replace the `.cloned().map_err(f)` pattern with an explicit `match` that only clones the side that needs transformation.
  - File: `fp-library/src/types/try_lazy.rs`
  - Complexity: small.

### 2.6 Add `Applicative` and `Monad` marker traits to `Step` brands

**Summary ref:** 2.7

- [ ] **Task 2.6:** Implement `Applicative` and `Monad` for `StepLoopAppliedBrand` and `StepDoneAppliedBrand`.
  - File: `fp-library/src/types/step.rs`
  - These are empty marker traits; the component traits are already implemented.
  - Complexity: small.

### 2.7 Remove `E: Clone` bound from `Foldable` for `TryLazyBrand`

**Summary ref:** 2.9

- [ ] **Task 2.7:** Remove the `Clone` bound on `E` from the `Foldable` implementation if the fold methods do not actually clone `E`.
  - File: `fp-library/src/types/try_lazy.rs`
  - Verify by checking every method body in the impl.
  - Complexity: small.

### 2.8 Make `Free`'s inner field private

**Summary ref:** 2.10

- [ ] **Task 2.8:** Change `pub(crate) Option<FreeInner>` to private and add accessor methods as needed.
  - File: `fp-library/src/types/free.rs`
  - Audit all crate-internal access points and refactor them to use the new accessors.
  - Complexity: medium.

### 2.9 Document `Free::resume` Cell invariant and `Free::Map` actual benefit

**Summary ref:** 1.3, 2.8

**Decision: Keep `Free::Map` variant, fix its documentation.**

The `Map` variant adds 7 match arms and complicates `Drop`, but provides a real construction-time optimization (one fewer CatList entry per `map` call). Both `resume()` and `evaluate()` convert `Map` into a continuation during evaluation, so the benefit is in construction, not evaluation. Removal would simplify `Drop` significantly (~180 lines affected), but the semantic clarity of an explicit `map` operation and existing test coverage (6 dedicated tests) justify keeping it.

Note: if Task 1.2 (hardening `Free::Drop`) proves difficult due to `Map` complexity, reconsider removal at that point.

- [ ] **Task 2.9a:** Add documentation to `Free::resume` explaining the `Cell::take` trick and the invariant that `Functor::map` must call the mapping function exactly once.
  - File: `fp-library/src/types/free.rs`
  - Complexity: small.
- [ ] **Task 2.9b:** Fix `Free::Map` variant documentation: the benefit is one fewer CatList continuation entry at construction time, not "avoids type-erasure roundtrip." Both `resume()` and `evaluate()` convert `Map` to a continuation anyway.
  - File: `fp-library/src/types/free.rs`
  - Complexity: small.

Tasks 2.9a and 2.9b can be done together.

### 2.10 Add `LazyConfig` bounds to brand definitions

**Summary ref:** 6.4

- [ ] **Task 2.10:** Add `Config: LazyConfig` bound to `LazyBrand<Config>` and equivalent bound to `TryLazyBrand<E, Config>`.
  - File: `fp-library/src/brands.rs`
  - Complexity: small.

---

## Phase 3: Missing Implementations

### 3.1 Conversions (batch)

**Summary ref:** 3.1

These are independent and can be parallelized.

- [ ] **Task 3.1a:** `From<SendThunk<'a, A>> for Thunk<'a, A>` (drop `Send` bound).
  - File: `fp-library/src/types/send_thunk.rs`
  - Complexity: small.
- [ ] **Task 3.1b:** `From<Thunk<'static, A>> for Trampoline<A>`.
  - File: `fp-library/src/types/thunk.rs`
  - Complexity: small.
- [ ] **Task 3.1c:** `From<TrySendThunk<'a, A, E>> for TryThunk<'a, A, E>`.
  - File: `fp-library/src/types/try_thunk.rs` or `fp-library/src/types/try_send_thunk.rs`
  - Complexity: small.
- [ ] **Task 3.1d:** `From<ArcTryLazy<'a, A, E>> for TrySendThunk<'a, A, E>`.
  - File: `fp-library/src/types/try_send_thunk.rs`
  - Complexity: small.
- [ ] **Task 3.1e:** `TryThunk::into_inner() -> Thunk<'a, Result<A, E>>`.
  - File: `fp-library/src/types/try_thunk.rs`
  - Complexity: small.
- [ ] **Task 3.1f:** `TryTrampoline::into_trampoline() -> Trampoline<Result<A, E>>`.
  - File: `fp-library/src/types/try_trampoline.rs`
  - Complexity: small.
- [ ] **Task 3.1g:** `Step <-> Result` and `Step <-> ControlFlow` conversions.
  - File: `fp-library/src/types/step.rs`
  - Complexity: small.

### 3.2 Missing methods and combinators (batch)

**Summary ref:** 3.2

- [ ] **Task 3.2a:** Add `SendThunk::zip_with` and `SendThunk::apply` inherent methods.
  - File: `fp-library/src/types/send_thunk.rs`
  - Complexity: small.
- [ ] **Task 3.2b:** Add `TryThunk` inherent `bimap` method.
  - File: `fp-library/src/types/try_thunk.rs`
  - Complexity: small.
- [ ] **Task 3.2c:** Add `TryLazy::and_then` and `TryLazy::or_else`.
  - File: `fp-library/src/types/try_lazy.rs`
  - Complexity: small.
- [ ] **Task 3.2d:** Add `TryTrampoline::pure`.
  - File: `fp-library/src/types/try_trampoline.rs`
  - Complexity: small.
- [ ] **Task 3.2e:** Add `Trampoline::ap` and `Trampoline::flatten`.
  - File: `fp-library/src/types/trampoline.rs`
  - Complexity: small.
- [ ] **Task 3.2f:** Add `WithIndex` / `FunctorWithIndex` / `FoldableWithIndex` for `TryThunk` brands.
  - File: `fp-library/src/types/try_thunk.rs`
  - Use `Index = ()` as with `Thunk`.
  - Complexity: small.

### 3.3 Missing trait implementations (batch)

**Summary ref:** 3.3

- [ ] **Task 3.3a:** Implement `Display` for `Lazy` (forces evaluation and displays the value).
  - File: `fp-library/src/types/lazy.rs`
  - Complexity: small.
- [ ] **Task 3.3b:** Implement `Hash` for `Lazy`.
  - File: `fp-library/src/types/lazy.rs`
  - Complexity: small.
- [ ] **Task 3.3c:** Implement `FoldableWithIndex` for `Lazy` (index type `()`).
  - File: `fp-library/src/types/lazy.rs`
  - Complexity: small.
- [ ] **Task 3.3d:** Add `CatListIterator::size_hint` and implement `ExactSizeIterator`.
  - File: `fp-library/src/types/cat_list.rs`
  - Complexity: small.
  - Note: also a performance improvement (Phase 5, 5.1), so doing it here covers both.
- [ ] **Task 3.3e:** Document the `Evaluable` naturality law in the trait's doc comment.
  - File: `fp-library/src/classes/evaluable.rs`
  - Complexity: small.
- [ ] **Task 3.3f:** Add `Applicative` / `Monad` marker impls for `Step` brands.
  - Covered by Task 2.6; skip here.

### 3.4 Larger missing features

#### 3.4a `SendTrampoline` type

**Decision: Deferred.** No concrete use case exists. Parameterizing `Free` with a Send marker is not viable in Rust (trait object bounds cannot be abstracted via generics). The only viable path is a separate `SendFree`/`SendTrampoline` (~650 new lines, ~40% code reuse with `Free`), following the existing `Thunk`/`SendThunk` pattern. The current hierarchy (`SendThunk` + `Trampoline` + `ArcLazy`) covers most real-world patterns; `SendTrampoline` only matters for deep recursion that must cross thread boundaries.

If a concrete use case arises, implement as a separate `SendFree` type with `Send`-bounded continuations.

#### 3.4b `CatList` borrowing iterator

- [ ] **Task 3.4b:** Implement a borrowing iterator `CatListIter<'a, A>` for `CatList<A>`.
  - File: `fp-library/src/types/cat_list.rs`
  - Approach: stack-based depth-first traversal using `Vec<std::collections::vec_deque::Iter<'a, CatList<A>>>`. For each `Cons(a, deque, _)`, yield `&a`, then push `deque.iter()` onto the stack and descend. The deque stores sublists in left-to-right order, so this matches the consuming iterator's element ordering.
  - This requires only immutable borrows; no restructuring or `flatten_deque` needed.
  - Also implement `IntoIterator for &'a CatList<A>`.
  - Remove `Clone` bounds from `PartialEq`, `Hash`, `Ord`, `PartialOrd` impls by using the borrowing iterator instead of `clone().into_iter()`.
  - Add tests comparing borrowing iterator output against consuming iterator for equivalence.
  - Complexity: medium.

#### 3.4c `Lazy: Comonad`

Deferred until the `Comonad` trait is defined in the library.

#### 3.4d `catch_with` variant for fallible types

**Decision: Implement with low priority.** Users can compose `catch` + `map_err` for most cases, but true monadic error recovery (`E -> TryThunk<'a, A, E2>`) is more ergonomic when the recovery itself can fail with a different error type.

- [ ] **Task 3.4d:** Add `catch_with` to `TryThunk`, `TrySendThunk`, and `TryTrampoline`.
  - Signature: `pub fn catch_with<E2>(self, f: impl FnOnce(E) -> TryThunk<'a, A, E2>) -> TryThunk<'a, A, E2>` (and analogous for the other types).
  - Name follows existing `catch_unwind_with` precedent.
  - Files: `fp-library/src/types/try_thunk.rs`, `fp-library/src/types/try_send_thunk.rs`, `fp-library/src/types/try_trampoline.rs`
  - Complexity: small per file.

---

## Phase 4: Documentation

These are documentation-only changes. All are independent and can be parallelized.

### 4.1 Cross-cutting documentation

- [ ] **Task 4.1a:** Add warning to `Deferrable` trait docs that some implementations evaluate eagerly for `Send` types, and that `SendDeferrable` should be preferred for true deferral with thread-safe types.
  - File: `fp-library/src/classes/deferrable.rs`
  - Complexity: small.
- [ ] **Task 4.1b:** Fix `Traversable` limitation explanation on `Thunk` to describe the actual blocker (trait bounds), not just `FnOnce` cloneability.
  - File: `fp-library/src/types/thunk.rs`
  - Complexity: small.
- [ ] **Task 4.1c:** Document `Foldable` error-discarding behavior for `TryLazy` in module-level docs.
  - File: `fp-library/src/types/try_lazy.rs`
  - Complexity: small.
- [ ] **Task 4.1d:** Add guidance on when to use `TryLazy` vs `Lazy<Result<A, E>>` vs `Result<Lazy, E>`.
  - File: `fp-library/src/types/try_lazy.rs`
  - Complexity: small.

### 4.2 File-specific documentation fixes

- [ ] **Task 4.2a:** Fix `evaluate` type parameter docs where `'b` description is wrong (says "lifetime of the computation" instead of "borrow lifetime").
  - File: `fp-library/src/types/lazy.rs` (lines 117, 245, 363, and `TryLazyConfig` equivalents)
  - Complexity: small.
- [ ] **Task 4.2b:** Remove duplicated "Stack Safety" section in `TryThunk` struct doc comment.
  - File: `fp-library/src/types/try_thunk.rs`
  - Complexity: small.
- [ ] **Task 4.2c:** Improve `OkAppliedBrand` doc examples by explaining the dual-channel encoding.
  - File: `fp-library/src/types/try_thunk.rs`
  - Complexity: small.
- [ ] **Task 4.2d:** Document `pure` / `ok` redundancy on `TryThunk`.
  - File: `fp-library/src/types/try_thunk.rs`
  - Complexity: small.
- [ ] **Task 4.2e:** Fix module-level example in `deferrable.rs` that creates a thunk-of-a-thunk; should use `Thunk::pure(42)`.
  - File: `fp-library/src/classes/deferrable.rs`
  - Complexity: small.
- [ ] **Task 4.2f:** Fix `SendCloneableFn` analogy imprecision regarding `FnOnce` vs `Fn` and `Sync`.
  - File: `fp-library/src/classes/send_deferrable.rs`
  - Complexity: small.
- [ ] **Task 4.2g:** State `A: Clone` requirement in `RefFunctor` identity law, and add cross-reference to `SendRefFunctor`.
  - File: `fp-library/src/classes/ref_functor.rs`
  - Complexity: small.
- [ ] **Task 4.2h:** Add explanation of why `FnOnce` is used for `RefFunctor::ref_map`.
  - File: `fp-library/src/classes/ref_functor.rs`
  - Complexity: small.
- [ ] **Task 4.2i:** Rename `memoize` to `into_rc_lazy` and `memoize_arc` to `into_arc_lazy` across all types.
  - **Decision: Rename without deprecation.** The `into_*` convention is neutral on evaluation timing, eliminating the confusion where `memoize_arc` evaluates eagerly on some types (`Thunk`, `Trampoline`, `TryThunk`, `TryTrampoline`) but lazily on others (`SendThunk`, `TrySendThunk`).
  - Files (all methods to rename):
    - `fp-library/src/types/thunk.rs`: `memoize` -> `into_rc_lazy`, `memoize_arc` -> `into_arc_lazy`
    - `fp-library/src/types/send_thunk.rs`: `memoize_arc` -> `into_arc_lazy`
    - `fp-library/src/types/trampoline.rs`: `memoize` -> `into_rc_lazy`, `memoize_arc` -> `into_arc_lazy`
    - `fp-library/src/types/try_thunk.rs`: `memoize` -> `into_rc_try_lazy`, `memoize_arc` -> `into_arc_try_lazy`
    - `fp-library/src/types/try_send_thunk.rs`: `memoize_arc` -> `into_arc_try_lazy`
    - `fp-library/src/types/try_trampoline.rs`: `memoize` -> `into_rc_try_lazy`, `memoize_arc` -> `into_arc_try_lazy`
  - Also update all doc examples, doc references, and tests that call these methods.
  - Complexity: medium (many files, but each change is mechanical).
- [ ] **Task 4.2j:** Simplify module doc memoization example to reference the `into_rc_lazy()` method.
  - File: `fp-library/src/types/trampoline.rs`
  - Complexity: small.
- [ ] **Task 4.2k:** Document that `Thunk`'s inherent `map` accepts `FnOnce` while HKT `Functor::map` requires `Fn`.
  - File: `fp-library/src/types/thunk.rs`
  - Complexity: small.
- [ ] **Task 4.2l:** Clarify `TryLazy::map` vs `Lazy::ref_map` naming inconsistency.
  - File: `fp-library/src/types/try_lazy.rs`
  - Complexity: small.
- [ ] **Task 4.2m:** Add type parameter descriptions to `LazyBrand<Config>` and `TryLazyBrand<E, Config>` doc comments.
  - File: `fp-library/src/brands.rs`
  - Complexity: small.
- [ ] **Task 4.2n:** Document why `TrySendThunk` lacks partially-applied brands (unlike `TryThunk`).
  - File: `fp-library/src/brands.rs`
  - Complexity: small.
- [ ] **Task 4.2o:** Add terminal periods to `Step` variant doc comments.
  - File: `fp-library/src/types/step.rs`
  - Complexity: small.
- [ ] **Task 4.2p:** Improve `Debug` for `Trampoline` to print the value for `Pure` variants when `A: Debug`.
  - File: `fp-library/src/types/trampoline.rs`
  - Note: this is a behavior change, not just a doc fix. The current impl always prints `<unevaluated>`.
  - Complexity: small.
- [ ] **Task 4.2q:** Add note to `send_thunk.rs` that there is no `Send`-capable stack-safe lazy type in the hierarchy.
  - File: `fp-library/src/types/send_thunk.rs`
  - Complexity: small.
- [ ] **Task 4.2r:** Refine CatList docs: "no reversal overhead" is slightly misleading, and document `uncons` amortized complexity nuances.
  - File: `fp-library/src/types/cat_list.rs`
  - Complexity: small.

---

## Phase 5: Performance

### 5.1 `CatListIterator::size_hint` / `ExactSizeIterator`

Covered by Task 3.3d. No additional work needed here.

### 5.2 Reduce double clone in `Trampoline::tail_rec_m`

**Summary ref:** 5.2

- [ ] **Task 5.2:** Restructure `tail_rec_m` to clone `f` only once per iteration instead of twice.
  - File: `fp-library/src/types/trampoline.rs`
  - Complexity: small.

### 5.3 Minor: `SendThunk::into_arc_lazy` closure indirection

**Summary ref:** 5.3

- [ ] **Task 5.3:** Remove the unnecessary wrapper closure in `into_arc_lazy` (formerly `memoize_arc`), passing the inner `Box<dyn FnOnce>` directly if possible.
  - File: `fp-library/src/types/send_thunk.rs`
  - Likely optimized away by the compiler; lowest priority.
  - Depends on: 4.2i (rename must happen first).
  - Complexity: small.

### 5.4 `Free::erase_type` allocation (informational, no action)

**Summary ref:** 5.4

This is inherent to the type-erasure design. No fix planned. Noted for awareness only.

---

## Phase 6: Tests

### 6.1 Tests for Phase 1 correctness fixes (do alongside the fixes)

- [ ] **Task 6.1a:** Test `Semigroup::append` where first operand is `Err` (verify second is not evaluated).
  - Files: `fp-library/src/types/try_thunk.rs`, `fp-library/src/types/try_send_thunk.rs`, `fp-library/src/types/try_lazy.rs`
  - Use a side-effect counter or similar mechanism to verify short-circuiting.
  - Depends on: Tasks 1.1a, 1.1b, 1.1c.
  - Complexity: small.
- [ ] **Task 6.1b:** Test `Semigroup::append` where second operand fails but first succeeds.
  - Same files as 6.1a.
  - Complexity: small.

### 6.2 Standalone test additions

All independent; can be parallelized.

- [ ] **Task 6.2a:** `MonadRec::tail_rec_m` stack safety test with large iteration count (e.g., 100,000+).
  - File: `fp-library/src/types/thunk.rs`
  - Complexity: small.
- [ ] **Task 6.2b:** Cross-thread integration test for `SendThunk` (actually send to another thread and evaluate).
  - File: `fp-library/src/types/send_thunk.rs`
  - Complexity: small.
- [ ] **Task 6.2c:** Test `catch` where recovery itself fails.
  - File: `fp-library/src/types/try_send_thunk.rs`
  - Complexity: small.
- [ ] **Task 6.2d:** `ArcLazy` `Foldable` tests (currently only `RcLazy` is tested).
  - File: `fp-library/src/types/lazy.rs`
  - Complexity: small.
- [ ] **Task 6.2e:** `SendRefFunctor` law tests via QuickCheck.
  - File: property test file or `fp-library/src/types/lazy.rs`
  - Complexity: small.
- [ ] **Task 6.2f:** `rc_lazy_fix` / `arc_lazy_fix` tests where `f` actually uses the self-reference.
  - File: `fp-library/src/types/lazy.rs`
  - Complexity: small.
- [ ] **Task 6.2g:** `into_rc_try_lazy` / `into_arc_try_lazy` unit tests for `TryTrampoline` (currently only in doc tests).
  - File: `fp-library/src/types/try_trampoline.rs`
  - Depends on: 4.2i (rename must happen first).
  - Complexity: small.
- [ ] **Task 6.2h:** Monad law tests for `Free` (left identity, right identity, associativity).
  - File: `fp-library/src/types/free.rs`
  - Complexity: medium.
- [ ] **Task 6.2i:** Mixed deep chain tests for `Free` (interleaved `map`, `bind`, `wrap`, `lift_f`).
  - File: `fp-library/src/types/free.rs`
  - Complexity: medium.
- [ ] **Task 6.2j:** `FunctorWithIndex` / `FoldableWithIndex` tests via HKT free functions for `Thunk`.
  - File: `fp-library/src/types/thunk.rs`
  - Complexity: small.
- [ ] **Task 6.2k:** `bimap` tests on both success and error paths simultaneously for `TrySendThunk`.
  - File: `fp-library/src/types/try_send_thunk.rs`
  - Complexity: small.

---

## Dependency Graph (key ordering constraints)

```
Task 2.2 -> 2.3             (document RefFunctor limitation before fixing false claims)
Task 1.1*  -> 6.1*          (fix append before writing tests for it)
Task 4.2i -> 5.3            (rename memoize methods before optimizing closure indirection)
Task 4.2i -> 6.2g           (rename memoize methods before writing unit tests)
```

All other tasks are independent of each other within their phase.

---

## Design Decisions (Resolved)

| Task | Decision | Rationale |
|------|----------|-----------|
| 2.2 | **Option B:** Document limitation, keep traits independent. | `ArcLazy::new` requires `Send` on closures; `RefFunctor::ref_map` cannot guarantee it. Option A is infeasible. |
| 2.9 | **Keep `Free::Map`, fix docs.** | Construction-time optimization is real. Reconsider removal only if Task 1.2 (`Drop` hardening) proves too complex due to `Map`. |
| 3.4a | **Deferred.** If needed, implement as separate `SendFree` type. | No concrete use case. Parameterization is not viable in Rust. |
| 3.4b | **Implement borrowing iterator.** | Feasible via stack-based depth-first traversal; removes `Clone` bounds from comparison traits. |
| 3.4c | **Deferred.** | Blocked on `Comonad` trait design. |
| 3.4d | **Implement `catch_with`, low priority.** | `catch` + `map_err` covers most cases; `catch_with` adds ergonomic monadic error recovery. |
| 4.2i | **Rename: `memoize` -> `into_rc_lazy`, `memoize_arc` -> `into_arc_lazy`.** | `into_*` convention is neutral on evaluation timing; eliminates eager/lazy naming confusion across types. |
