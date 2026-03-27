# Lazy Evaluation Hierarchy: Implementation Plan

This plan addresses every issue identified in `summary.md`, grouped into four phases ordered by priority. Tasks within each phase are numbered for reference, and parallelization opportunities are noted at the end of each phase.

---

## Phase 1: Correctness and Safety

These tasks fix bugs, prevent undefined behavior, and correct incorrect documentation that could mislead users.

### Task 1.1: Iterative `Drop` for `Free` `Wrap` chains

- **Files:** `fp-library/src/types/free.rs`
- **What:** The existing `Drop` impl handles `Bind` chains iteratively but delegates `Wrap` variants to the default recursive drop. Add an iterative loop that also handles deeply nested `Wrap`-only chains, converting them into a heap-allocated worklist that drains without growing the stack.
- **Why:** Issue #1 (high severity). Deeply nested `Wrap` chains overflow the stack on drop.
- **Dependencies:** None.
- **Complexity:** Medium. Requires careful handling of the `Free` enum variants during teardown without violating ownership rules.
- **Testing:** Add a test in `fp-library/tests/stack_safety.rs` that creates a deeply nested `Wrap`-only `Free` (e.g., 100,000 levels) and drops it without stack overflow.

### Task 1.2: Iterative `Drop` for `CatList`

- **Files:** `fp-library/src/types/cat_list.rs`
- **What:** Add a custom `Drop` implementation that iteratively drains the `CatList` tree structure instead of recursing through nested `Append` nodes.
- **Why:** Issue #2 (high severity). Deeply nested `CatList` trees can overflow the stack when dropped. Although `flatten_deque` mitigates this during iteration, standalone `CatList` values that are dropped without being iterated are vulnerable.
- **Dependencies:** None.
- **Complexity:** Medium. Similar pattern to Task 1.1; drain nested nodes into a `Vec` or `VecDeque` worklist.
- **Testing:** Add a test in `fp-library/tests/stack_safety.rs` that creates a deeply nested `CatList` and drops it safely.

### Task 1.3: Fix `Evaluable` naturality law

- **Files:** `fp-library/src/classes/evaluable.rs`
- **What:** Replace the incorrectly stated naturality law with the correct map-extract law: `evaluate(map(f, fa)) == f(evaluate(fa))`. Update the doc comment and any doc examples that demonstrate the law.
- **Why:** Issue #3 (high severity). The current law constrains natural transformations rather than `evaluate` itself; it is misleading and mathematically incorrect for this trait.
- **Dependencies:** None.
- **Complexity:** Small. Documentation-only change, but the law statement must be precisely correct.

### Task 1.4: Add `tail_rec_m` and `arc_tail_rec_m` to `TrySendThunk`

- **Files:** `fp-library/src/types/try_send_thunk.rs`
- **What:** Implement `tail_rec_m` (using `Rc`-based loop) and `arc_tail_rec_m` (using `Arc`-based loop) as inherent methods, following the same pattern used by `TryThunk::tail_rec_m` and `SendThunk::arc_tail_rec_m`. The step function and result closures must be `Send`.
- **Why:** Issue #4 (medium severity). This is the only thunk/trampoline variant without stack-safe recursion support, leaving no path for fallible + thread-safe deferred recursion.
- **Dependencies:** None. Can reference `TryThunk::tail_rec_m` and `SendThunk::arc_tail_rec_m` as templates.
- **Complexity:** Medium. The logic is well-established in sibling types, but `Send` bounds on closures require careful threading.
- **Testing:** Add unit tests demonstrating stack-safe fallible recursion with `Send` closures, including both success and error paths.

### Task 1.5: Add `Sync` bound to `SendRefFunctor::send_ref_map` output type

- **Files:** `fp-library/src/classes/send_ref_functor.rs`
- **What:** Add `Sync` to the `B` bound in `SendRefFunctor::send_ref_map` (and the free function wrapper if one exists). The resulting `ArcLazy<B>` requires `B: Send + Sync` to itself be `Send`, so omitting `Sync` silently produces a `!Send` result, defeating the purpose.
- **Why:** Issue #10 (medium severity). Without `Sync`, the trait's contract is subtly broken for its primary use case. Types that are `Send + !Sync` (e.g., `Cell<T>`, `MutexGuard`) are rare and wrapping them in `ArcLazy` is not a useful pattern. The whole point of `SendRefFunctor` is thread-safe mapping; producing a `!Send` result is a bug, not a feature.
- **Dependencies:** None.
- **Complexity:** Small. Adding one trait bound; verify all call sites and implementors still compile.

**Parallelization:** All five tasks in Phase 1 are independent and touch different files. They can all be worked on simultaneously.

---

## Phase 2: Completeness

These tasks fill gaps in type class coverage, conversions, and operations.

### Task 2.1: Implement `MonadRec` for standard types

- **Files:** `fp-library/src/types/option.rs`, `fp-library/src/types/vec.rs`, `fp-library/src/types/result.rs`, `fp-library/src/types/identity.rs`, `fp-library/src/classes/monad_rec.rs` (if trait needs updating)
- **What:** Implement `MonadRec` for `OptionBrand`, `VecBrand`, `ResultErrAppliedBrand<E>`, and `IdentityBrand`. Each implementation uses a simple loop (no actual trampolining needed since `bind` is inherently stack-safe for these types).
- **Why:** Issue #5 (medium severity). Without these, generic `MonadRec`-polymorphic code is limited to `Thunk`-family types only. Also a prerequisite for Task 2.6 (making `fold_free` require `MonadRec`).
- **Dependencies:** None.
- **Complexity:** Small. Each implementation is a straightforward loop over `Step::Loop`/`Step::Done`.
- **Testing:** Add QuickCheck property tests verifying the `MonadRec` identity law for each type.

### Task 2.2: Add `From<SendThunk<'a, A>> for Thunk<'a, A>`

- **Files:** `fp-library/src/types/send_thunk.rs` or `fp-library/src/types/thunk.rs` (whichever holds conversion impls)
- **What:** Implement `From<SendThunk<'a, A>> for Thunk<'a, A>` by erasing the `Send` bound on the inner `Box<dyn FnOnce() -> A + Send + 'a>`, coercing it to `Box<dyn FnOnce() -> A + 'a>`. This is a zero-cost unsizing coercion.
- **Why:** Issue #9 (medium severity). A natural subtyping relationship that should be expressible.
- **Dependencies:** None.
- **Complexity:** Small.
- **Testing:** Unit test demonstrating the conversion compiles and produces the expected value.

### Task 2.3: Add `From<TryThunk<'a, A, E>> for TryTrampoline<A, E>`

- **Files:** `fp-library/src/types/try_thunk.rs` or `fp-library/src/types/try_trampoline.rs`
- **What:** Implement the conversion from `TryThunk` to `TryTrampoline` (requires `A: 'static, E: 'static`). This completes the bidirectional conversion; the reverse (`From<TryTrampoline> for TryThunk`) already exists.
- **Why:** Issue #7 (medium severity). Asymmetric conversions are surprising.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 2.4: Implement `Foldable` and `FoldableWithIndex` for `SendThunkBrand`

- **Files:** `fp-library/src/types/send_thunk.rs`
- **What:** Implement `Foldable` and `FoldableWithIndex` (index type `()`) for `SendThunkBrand`. The fold functions themselves do not need `Send` bounds since they run after the thunk has been evaluated.
- **Why:** Issue #8 (medium severity). These are feasible and useful instances that are currently missing.
- **Dependencies:** None.
- **Complexity:** Small to medium. Need to verify that the `Foldable` trait method signatures do not require any bounds incompatible with `SendThunk`.

### Task 2.5: Implement `FoldableWithIndex` for `LazyBrand<Config>`

- **Files:** `fp-library/src/types/lazy.rs`
- **What:** Implement `FoldableWithIndex` with index type `()` for `LazyBrand<Config>` where `Foldable` is already implemented.
- **Why:** Summary section 4, "Missing Type Class Instances." Straightforward completion.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 2.6: Replace `fold_free` with stack-safe `MonadRec` version

- **Files:** `fp-library/src/types/free.rs`
- **What:** Replace the current recursive `fold_free` with an iterative version that requires `G: MonadRec` and uses `G::tail_rec_m` with `resume` as the step function. This mirrors PureScript's `foldFree` exactly:
  ```
  fn fold_free<G: MonadRec>(self, nt) -> G<A> {
      G::tail_rec_m(|free| match free.resume() {
          Ok(a) => G::pure(Step::Done(a)),
          Err(fa) => G::map(|inner| Step::Loop(inner), nt.transform(fa)),
      }, self)
  }
  ```
  Do not keep the old recursive implementation. In a strict language, every well-behaved monad can trivially implement `MonadRec` with a loop, so the stronger bound is not restrictive. If a user defines a custom monad without `MonadRec`, implementing it is a 5-line loop; it does not need a library escape hatch.
- **Why:** The current `fold_free` is recursive in the number of `Wrap` layers, which can overflow the stack for deep effect chains. PureScript's `foldFree` requires `MonadRec` for this reason. After Task 2.1, every monad in the library implements `MonadRec`, so the stronger bound costs nothing in practice.
- **Dependencies:** Task 2.1 (all standard monads must implement `MonadRec` first).
- **Complexity:** Medium. The approach is well-understood (direct translation of PureScript's implementation), but the HKT encoding and lifetime constraints require care.
- **Testing:** Verify existing `fold_free` tests pass with the new implementation. Add a stack-safety test with a deeply nested `Wrap` chain folded into a `MonadRec` target.

### Task 2.7: Add `hoist_free` to `Free`

- **Files:** `fp-library/src/types/free.rs`
- **What:** Add a stack-safe `hoist_free` that transforms `Free<F, A>` into `Free<G, A>` by applying a natural transformation to each functor layer. Use an iterative approach (explicit worklist or loop with `resume`) rather than recursion, consistent with the library's stack-safety goals for `Free`.
- **Why:** Standard `Free` monad operation already acknowledged as missing in the docs. Completes the `Free` monad API.
- **Dependencies:** None.
- **Complexity:** Medium. Requires careful handling of the `Bind` case (the continuation must also be hoisted).
- **Testing:** Unit tests demonstrating natural transformation application, and a stack-safety test with deeply nested structures.

### Task 2.8: Add `From<TrySendThunk> for ArcTryLazy`

- **Files:** `fp-library/src/types/try_send_thunk.rs` or `fp-library/src/types/try_lazy.rs`
- **What:** Implement `From<TrySendThunk<'a, A, E>> for ArcTryLazy<'a, A, E>` as a proper `From` trait impl, complementing the existing `into_arc_try_lazy` inherent method.
- **Why:** Issue #21. Completes the conversion graph and enables generic `From`/`Into` usage.
- **Dependencies:** None.
- **Complexity:** Small. The logic already exists in the inherent method; this just wraps it in a trait impl.

### Task 2.9: Relax `MonadRec` `Clone` bound on step function

- **Files:** `fp-library/src/classes/monad_rec.rs`
- **What:** Change the `Clone` bound on the step function parameter of `MonadRec::tail_rec_m` to `Fn`. All current HKT implementors (after Task 2.1) use simple loops where `Fn` suffices. The `Clone` bound was originally added for `Trampoline`'s recursive `go` pattern, but `Trampoline` cannot implement the HKT trait anyway due to `'static` constraints.
- **Why:** Issue #11 (low severity). `Clone` on closures is an unusual and restrictive bound that limits usability. Relaxing it is backwards-compatible (every `Clone` closure is also `Fn`). Validate with the full test suite.
- **Dependencies:** Task 2.1 (to verify all implementors work with the relaxed bound).
- **Complexity:** Small.

**Parallelization:** Tasks 2.1 through 2.5, 2.7, and 2.8 are all independent and can proceed in parallel. Task 2.6 depends on Task 2.1. Task 2.9 depends on Task 2.1.

---

## Phase 3: Documentation and Testing

These tasks fix documentation errors, add missing doc sections, improve law statements, and add property-based tests.

### Task 3.1: Rewrite `MonadRec` laws section

- **Files:** `fp-library/src/classes/monad_rec.rs`
- **What:** Replace the current "Equivalence" law (which is a tautology) and "Safety varies" (which is an implementation note) with the proper PureScript identity law: `tail_rec_m(|a| pure(Step::Done(a)), x) == pure(x)`. Reframe stack safety as a class invariant rather than a law. Add the expected doc example demonstrating the law.
- **Why:** Issue #6 (medium severity). The current laws are mathematically vacuous.
- **Dependencies:** Task 2.1 (so the law examples can use standard types like `Option`).
- **Complexity:** Small.

### Task 3.2: Add QuickCheck property tests for trait laws

- **Files:** New file or additions to existing test files (e.g., `fp-library/src/classes/deferrable.rs` inline tests, or a new `fp-library/tests/property_laws.rs`)
- **What:** Add QuickCheck property tests for:
  - `Deferrable` transparency law: `evaluate(defer(|| x)) == x`
  - `Deferrable` nesting law: `evaluate(defer(|| defer(|| x))) == evaluate(defer(|| x))`
  - `Evaluable` map-extract law (after Task 1.3): `evaluate(map(f, fa)) == f(evaluate(fa))`
  - `MonadRec` identity law (after Task 3.1): `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`
  - `RefFunctor` identity and composition laws
  - `SendRefFunctor` identity and composition laws (via trait, not just inherent methods)
- **Why:** Issue #12 in recommendations. Doc tests demonstrate correctness but do not systematically verify algebraic properties.
- **Dependencies:** Task 1.3 (for `Evaluable` law), Task 3.1 (for `MonadRec` law).
- **Complexity:** Medium. Requires implementing `Arbitrary` for relevant types (if not already done) and writing the property test harness.

### Task 3.3: Add missing `SendRefFunctor` doc sections

- **Files:** `fp-library/src/classes/send_ref_functor.rs`
- **What:** Add "Cache chain behavior" and "Why `FnOnce`?" documentation sections, mirroring the equivalent sections in `RefFunctor`. These sections explain that chained `send_ref_map` calls create a chain of `ArcLazy` cells (each depending on the previous), and that `FnOnce` is used because the mapping function is consumed when the lazy cell is first forced.
- **Why:** Issue #13, #16 in recommendations. Documentation parity between paired traits.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 3.4: Fix `TryThunk` copy-paste doc errors

- **Files:** `fp-library/src/types/try_thunk.rs`
- **What:** Change "The Thunk to fold" to "The TryThunk to fold" in the `fold_map` parameter descriptions (the summary references lines 1180 and 1933, but verify exact locations).
- **Why:** Issue #13 (low severity). Minor but affects documentation accuracy.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 3.5: Rename `into_trampoline` to `into_inner`

- **Files:** `fp-library/src/types/try_trampoline.rs`
- **What:** Rename `TryTrampoline::into_trampoline` to `into_inner` to match `TryThunk::into_inner`. This is a direct rename with no deprecation alias.
- **Why:** Issue #14 (low severity). Naming consistency across the hierarchy.
- **Dependencies:** None.
- **Complexity:** Small. Update the method name and all call sites (tests, doc examples, any internal uses).

### Task 3.6: Add `TrySendThunk::ok` doc note

- **Files:** `fp-library/src/types/try_send_thunk.rs`
- **What:** Add an "Alias for `pure`" note to the `TrySendThunk::ok` method documentation, matching what `TryThunk::ok` already says.
- **Why:** Issue #15 (low severity). Documentation consistency.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 3.7: Add `SendThunk` comparison table and monad law docs

- **Files:** `fp-library/src/types/send_thunk.rs`
- **What:** Add a comparison table (similar to the one in `Thunk` docs) showing `SendThunk` vs other lazy types. Add monad law documentation for `SendThunk`'s inherent `pure`, `bind`, and `map` methods, documenting that they satisfy the monad laws despite not being expressible via the HKT `Monad` trait.
- **Why:** Issue #16 in recommendations. `Thunk` has significantly richer documentation than `SendThunk`.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 3.8: Fix minor doc issues

- **Files:** `fp-library/src/classes/send_ref_functor.rs`, `fp-library/src/classes/evaluable.rs`, `fp-library/src/types/try_trampoline.rs`
- **What:**
  - `send_ref_functor.rs`: Fix "returning references" to "receiving references" (line 26 per summary).
  - `evaluable.rs`: Remove or generalize "Currently only ThunkBrand implements this trait" (fragile statement).
  - `try_trampoline.rs`: Fix unusual `#[document_examples]` placement on `defer`.
- **Why:** Issues #16, #17 (minor doc issues from summary section 5).
- **Dependencies:** None.
- **Complexity:** Small.

### Task 3.9: Implement `PartialEq`/`Eq`/`Hash`/`Ord` for `TryLazy`

- **Files:** `fp-library/src/types/try_lazy.rs`
- **What:** Implement `PartialEq`, `Eq`, `Hash`, and `Ord` for `TryLazy`, using the same approach `Lazy` uses. The bounds will require both `A` and `E` to satisfy the respective trait. This is almost certainly an oversight since `Lazy` already implements all of these and `Result<A, E>` supports them when both type parameters do.
- **Why:** Issue #19 (low severity). Parity with `Lazy`.
- **Dependencies:** None.
- **Complexity:** Small to medium.

### Task 3.10: Improve brand documentation

- **Files:** `fp-library/src/brands.rs`
- **What:** Add brief doc notes to:
  - `CatListBrand`: Mention its role as the backbone of `Free` monad evaluation.
  - `StepBrand` (and its applied variants): Mention its role in `MonadRec`.
  - `SendThunkBrand`: Note HKT trait limitations due to `Send` bounds.
- **Why:** Summary section 5, "Missing Documentation Sections."
- **Dependencies:** None.
- **Complexity:** Small.

### Task 3.11: Fix `CatList` doc wording about `flatten_deque`

- **Files:** `fp-library/src/types/cat_list.rs`
- **What:** Change "iterative approach" to something more precise like "stack-safe approach using `rfold`" in the `flatten_deque` documentation.
- **Why:** Issue #18 (low severity). The current wording is slightly misleading.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 3.12: Fix `RefFunctor` composition law variable naming

- **Files:** `fp-library/src/classes/ref_functor.rs`
- **What:** Align the variable naming between the abstract composition law statement and the concrete example so they match.
- **Why:** Issue #17 (low severity). Readability improvement.
- **Dependencies:** None.
- **Complexity:** Small.

**Parallelization:** Tasks 3.3 through 3.12 are all independent and can proceed in parallel. Task 3.1 depends on Task 2.1. Task 3.2 depends on Tasks 1.3 and 3.1.

---

## Phase 4: Nice-to-Have Enhancements

These tasks improve the library's API surface and ergonomics but are not blocking any correctness or completeness goals.

### Task 4.1: Add `RcLazy` <-> `ArcLazy` conversions

- **Files:** `fp-library/src/types/lazy.rs`
- **What:** Add `From<RcLazy<'a, A>> for ArcLazy<'a, A>` and `From<ArcLazy<'a, A>> for RcLazy<'a, A>`, both requiring `A: Clone` and eagerly evaluating the source to extract the value.
- **Why:** Issue #18. Completes the conversion graph between lazy types.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 4.2: Add `bimap` to `TryLazy`

- **Files:** `fp-library/src/types/try_lazy.rs`
- **What:** Add a `bimap` method that maps both the success value (`A -> B`) and error value (`E -> F`) in a single operation, producing a new `TryLazy<'a, B, F>`.
- **Why:** Issue #20. Convenience combinator for fallible lazy values.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 4.3: Add non-panicking extractors and `swap` to `Step`

- **Files:** `fp-library/src/types/step.rs`
- **What:** Add `done() -> Option<B>`, `loop_val() -> Option<A>`, and `swap() -> Step<B, A>` methods to `Step`.
- **Why:** Summary section 4, "Missing Operations." These are standard enum utility methods.
- **Dependencies:** None.
- **Complexity:** Small.

### Task 4.4: Fix `SendThunk::into_arc_lazy` abstraction boundary

- **Files:** `fp-library/src/types/send_thunk.rs`
- **What:** Refactor `SendThunk::into_arc_lazy` to use a `From` impl or public constructor on `Lazy` instead of directly constructing `Lazy` via its tuple struct field.
- **Why:** Issue #12 (low severity). The current approach bypasses the `Lazy` type's abstraction boundary.
- **Dependencies:** May require adding a constructor or `From` impl to `Lazy`.
- **Complexity:** Small.

**Parallelization:** All Phase 4 tasks are independent and can proceed in parallel.

---

## Controversial or Discussion-Worthy Items

These items involve design trade-offs that are resolved but worth noting:

1. **`TryThunkOkAppliedBrand` applicative/monad inconsistency:** `apply` uses fail-last semantics while `bind` uses fail-fast, violating the standard consistency law. This is documented as intentional (mirrors `Validation` vs `Either`). No task is created because the current design is deliberate, but it should be revisited if the library wants strict law compliance. Changing `apply` to fail-fast would be a behavioral breaking change.

---

## Summary Table

| Task | Phase | Complexity | Files | Dependencies |
|------|-------|-----------|-------|--------------|
| 1.1 Iterative `Drop` for `Free` `Wrap` | 1 | Medium | `free.rs` | None |
| 1.2 Iterative `Drop` for `CatList` | 1 | Medium | `cat_list.rs` | None |
| 1.3 Fix `Evaluable` law | 1 | Small | `evaluable.rs` | None |
| 1.4 `TrySendThunk` `tail_rec_m` | 1 | Medium | `try_send_thunk.rs` | None |
| 1.5 `SendRefFunctor` `Sync` bound | 1 | Small | `send_ref_functor.rs` | None |
| 2.1 `MonadRec` for standard types | 2 | Small | `option.rs`, `vec.rs`, `result.rs`, `identity.rs` | None |
| 2.2 `From<SendThunk> for Thunk` | 2 | Small | `send_thunk.rs` or `thunk.rs` | None |
| 2.3 `From<TryThunk> for TryTrampoline` | 2 | Small | `try_thunk.rs` or `try_trampoline.rs` | None |
| 2.4 `Foldable` for `SendThunkBrand` | 2 | Small-Med | `send_thunk.rs` | None |
| 2.5 `FoldableWithIndex` for `LazyBrand` | 2 | Small | `lazy.rs` | None |
| 2.6 Replace `fold_free` with `MonadRec` version | 2 | Medium | `free.rs` | 2.1 |
| 2.7 `hoist_free` (stack-safe) | 2 | Medium | `free.rs` | None |
| 2.8 `From<TrySendThunk> for ArcTryLazy` | 2 | Small | `try_send_thunk.rs` or `try_lazy.rs` | None |
| 2.9 Relax `MonadRec` `Clone` bound | 2 | Small | `monad_rec.rs` | 2.1 |
| 3.1 Rewrite `MonadRec` laws | 3 | Small | `monad_rec.rs` | 2.1 |
| 3.2 QuickCheck property tests | 3 | Medium | New or existing test files | 1.3, 3.1 |
| 3.3 `SendRefFunctor` doc sections | 3 | Small | `send_ref_functor.rs` | None |
| 3.4 `TryThunk` doc fix | 3 | Small | `try_thunk.rs` | None |
| 3.5 Rename `into_trampoline` to `into_inner` | 3 | Small | `try_trampoline.rs` | None |
| 3.6 `TrySendThunk::ok` doc | 3 | Small | `try_send_thunk.rs` | None |
| 3.7 `SendThunk` comparison table | 3 | Small | `send_thunk.rs` | None |
| 3.8 Minor doc fixes | 3 | Small | `send_ref_functor.rs`, `evaluable.rs`, `try_trampoline.rs` | None |
| 3.9 Implement `PartialEq`/`Eq`/`Hash`/`Ord` for `TryLazy` | 3 | Small-Med | `try_lazy.rs` | None |
| 3.10 Brand docs | 3 | Small | `brands.rs` | None |
| 3.11 `CatList` doc wording | 3 | Small | `cat_list.rs` | None |
| 3.12 `RefFunctor` variable naming | 3 | Small | `ref_functor.rs` | None |
| 4.1 `RcLazy` <-> `ArcLazy` conversions | 4 | Small | `lazy.rs` | None |
| 4.2 `TryLazy` `bimap` | 4 | Small | `try_lazy.rs` | None |
| 4.3 `Step` utility methods | 4 | Small | `step.rs` | None |
| 4.4 Fix `into_arc_lazy` abstraction | 4 | Small | `send_thunk.rs`, `lazy.rs` | None |
