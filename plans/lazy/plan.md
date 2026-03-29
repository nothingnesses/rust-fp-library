# Lazy Evaluation Hierarchy: Implementation Plan

This plan addresses all issues identified in the [consolidated analysis summary](summary.md). Work is grouped into sequential phases, ordered so that foundational and blocking changes come first.

---

## Phase 1: Dependency Ordering Fix

Restores the stated `brands -> classes -> types` dependency ordering. This is foundational because later phases add new trait impls that should follow the corrected layering.

### Task 1.1: Move `LazyConfig` and `TryLazyConfig` trait definitions to `classes/`

- **Files to modify:**
  - `fp-library/src/classes.rs` (add new module export)
  - `fp-library/src/types/lazy.rs` (remove trait definitions, keep concrete impls `RcLazyConfig`/`ArcLazyConfig`)
  - `fp-library/src/types/try_lazy.rs` (update imports)
  - `fp-library/src/brands.rs` (update imports to point at `classes::` instead of `types::`)
- **Files to create:**
  - `fp-library/src/classes/lazy_config.rs` (new home for `LazyConfig` and `TryLazyConfig` trait definitions)
- **What:** Extract the `LazyConfig` and `TryLazyConfig` trait definitions (not their concrete impls) from `types/lazy.rs` into a new `classes/lazy_config.rs` module. Update all imports across the crate. The concrete config structs (`RcLazyConfig`, `ArcLazyConfig`) and their trait impls remain in `types/lazy.rs` since they are type-level implementations.
- **Why:** `brands.rs` currently imports from `types`, violating the dependency graph. These config traits define behavior (like type classes), not concrete types, so they belong in `classes/`.
- **Dependencies:** None. This is a pure refactor with no semantic changes.

---

## Phase 2: Missing Trait Implementations (High Priority)

Fills obvious gaps in the type class hierarchy. These are small, self-contained additions.

### Task 2.1: Add `WithIndex` and `FoldableWithIndex` for `TryLazyBrand<E, Config>`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs` (add trait impls)
- **What:** Implement `WithIndex` with `type Index = ()` and `FoldableWithIndex` (delegating to `Foldable` with the unit index) for `TryLazyBrand<E, Config>`, mirroring the existing implementations on `LazyBrand<Config>`.
- **Why:** `LazyBrand` has both traits; `TryLazyBrand` does not. Both are single-element containers where `Index = ()` is the natural choice. This restores parity.
- **Dependencies:** None.

### Task 2.2: Resolve `TrySendThunkBrand` (implement or remove)

- **Files to modify:**
  - `fp-library/src/brands.rs` (potentially remove brand or add partially-applied brands)
  - `fp-library/src/types/try_thunk.rs` or a new `fp-library/src/types/try_send_thunk.rs` (add type class impls if keeping the brand)
- **What:** Investigate whether `Bifunctor`/`Bifoldable` can be implemented for `TrySendThunkBrand`. The bifunctor traits accept two separate mapping closures, so the `Send` constraint may be enforceable differently than for `Functor`. If implementable, add `Bifunctor`, `Bifoldable`, and `Bitraversable`. If blocked by `Send` constraints, remove the brand and document why in a code comment where it was defined.
- **Why:** The brand currently has zero type class implementations, making it a dead definition that creates false expectations of symmetry with `TryThunkBrand`.
- **Dependencies:** None.

### Task 2.3: Add `From<TrySendThunk> for TryThunk`

- **Files to modify:**
  - `fp-library/src/types/try_thunk.rs` (add `From` impl)
- **What:** Implement `From<TrySendThunk<'a, A, E>> for TryThunk<'a, A, E>` using the same unsizing coercion pattern as the existing `From<SendThunk<'a, A>> for Thunk<'a, A>`.
- **Why:** Fills a gap in the conversion web. The infallible pair (`SendThunk -> Thunk`) already exists; the fallible pair should too.
- **Dependencies:** None.

---

## Phase 3: Documentation Fixes

Addresses documentation gaps that affect correctness understanding. These are text-only changes with no code impact.

### Task 3.1: Document the unfolding/equivalence law for `MonadRec`

- **Files to modify:**
  - `fp-library/src/classes/monad_rec.rs` (update trait-level doc comment)
- **What:** Add the equivalence law to the trait documentation: `tail_rec_m(f, a)` is equivalent to `f(a) >>= match { Loop(a') => tail_rec_m(f, a'), Done(b) => pure(b) }`. This is the defining semantic of `MonadRec` and complements the existing identity law.
- **Why:** The trait currently states only the identity law. The equivalence law is critical for users to reason about correctness.
- **Dependencies:** None.

### Task 3.2: Document the pure-extract law for `Evaluable`

- **Files to modify:**
  - `fp-library/src/classes/evaluable.rs` (update trait-level doc comment and `ThunkBrand` impl doc)
- **What:** Add the law `evaluate(pure(x)) == x` to the trait documentation. Also fix the `ThunkBrand` impl's parameter docs, which use "eval" as a noun ("The eval to run") instead of the consistent "evaluate" terminology.
- **Why:** `Free::evaluate` implicitly relies on this law but it is not documented. The naming inconsistency is a minor polish item.
- **Dependencies:** None.

### Task 3.3: Add algebraic properties and limitations sections to `TryTrampoline`

- **Files to modify:**
  - `fp-library/src/types/trampoline.rs` (update `TryTrampoline` doc comments)
- **What:** Add structured documentation sections for algebraic properties (monad laws hold for `map`/`bind`/`pure`, short-circuiting behavior for errors) and limitations (`'static` constraint, no HKT brand, error memoization in `TryTrampoline` when used with `Trampoline<Result>`). Model after the documentation on `TryThunk` and `Trampoline`.
- **Why:** `TryTrampoline` is the only major type in the hierarchy lacking structured documentation of its algebraic properties.
- **Dependencies:** None.

### Task 3.4: Document nondeterministic termination caveat for multi-element `MonadRec`

- **Files to modify:**
  - `fp-library/src/classes/monad_rec.rs` (add note to trait docs or to `VecBrand`/`CatListBrand` impl docs)
- **What:** Add a note that for multi-element containers (`VecBrand`, `CatListBrand`), if the step function always produces `Loop` values, the computation never terminates and consumes unbounded memory. Single-element containers (`ThunkBrand`, `Trampoline`) do not have this issue.
- **Why:** Users reaching for `MonadRec` on collection types may not realize the termination behavior differs from single-element types.
- **Dependencies:** None.

---

## Phase 4: Free Monad Constraint Relaxation

Loosens overly broad type constraints on `Free`.

### Task 4.1: Relax `Evaluable` constraint on `Free` construction methods

- **Files to modify:**
  - `fp-library/src/types/free.rs` (split `impl` blocks by constraint level)
- **What:** Move `pure`, `bind`, `map`, and `lift_f` into an `impl<F, A>` block (or `impl<F: Functor, A>` where needed), reserving `F: Evaluable` only for `evaluate`, `resume`, and `hoist_free`. The `erase_type` helper may need `Functor` for `Wrap` variant handling. Audit each method to determine its minimal constraint.
- **Why:** Currently all methods on `Free<F, A>` require `F: Evaluable`, which prevents constructing `Free` values over functors that are not `Evaluable`. PureScript's Free only requires `Functor` for structural operations. This change increases flexibility and validates the `Evaluable` abstraction by making it required only where it is actually used.
- **Dependencies:** None, but should be done after Phase 2 (task 2.2) to ensure any brand changes are settled.

---

## Phase 5: Additional Trait Implementations (Medium Priority)

Adds trait implementations that expand the hierarchy's utility.

### Task 5.1: Add `Traversable` for `ThunkBrand`

- **Files to modify:**
  - `fp-library/src/types/thunk.rs` (add `Traversable` impl)
- **What:** Implement `Traversable` for `ThunkBrand`. For a single-element container, `traverse(f, thunk)` evaluates the thunk, applies `f` to get an `F<B>`, and maps `pure` over the result to wrap `B` back in a `Thunk`. Investigate whether `Thunk`'s `!Clone` nature blocks this; if so, document the blocker and skip.
- **Why:** `CatListBrand` already has `Traversable`. A single-element `Traversable` is well-defined and useful for generic programming (e.g., `sequence` on a `Thunk` inside an applicative).
- **Dependencies:** None.

### Task 5.2: Add `Evaluable` for `IdentityBrand`

- **Files to modify:**
  - `fp-library/src/types/identity.rs` (or wherever `IdentityBrand` is implemented; add `Evaluable` impl)
- **What:** Implement `Evaluable` for `IdentityBrand` with `evaluate(Identity(a)) = a`. This makes `Free<IdentityBrand, A>` a valid (though degenerate) instantiation.
- **Why:** `Evaluable` currently has only one implementor (`ThunkBrand`), making the trait's genericity purely theoretical. A second implementor validates the abstraction.
- **Dependencies:** Depends on Phase 4 (task 4.1) if `Evaluable` constraints are being relaxed on `Free`.

### Task 5.3: Add `Display` for `TryLazy`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs` (add `Display` impl)
- **What:** Implement `Display` for `TryLazy` that forces evaluation and renders `Ok(value)` or `Err(error)`, mirroring `Lazy`'s `Display` implementation.
- **Why:** Restores parity with `Lazy`.
- **Dependencies:** None.

### Task 5.4: Add cross-config conversions for `TryLazy`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs` (add `From` impls)
- **What:** Add `From<RcTryLazy<'a, A, E>> for ArcTryLazy<'a, A, E>` and `From<ArcTryLazy<'a, A, E>> for RcTryLazy<'a, A, E>`, following the same eager-evaluation-and-clone pattern used by `Lazy`'s cross-config conversions.
- **Why:** `Lazy` has these conversions; `TryLazy` does not. Completes the conversion matrix.
- **Dependencies:** None.

---

## Phase 6: Minor Improvements (Low Priority)

Small quality-of-life improvements.

### Task 6.1: Relax `Clone` bound on `SendThunk::tail_rec_m`

- **Files to modify:**
  - `fp-library/src/types/send_thunk.rs` (update `tail_rec_m` signature and impl)
- **What:** Remove the `Clone` bound on the step function parameter of `SendThunk::tail_rec_m`. The function is `Fn` (not `FnOnce`), so it can be called by reference in the loop without cloning.
- **Why:** Unnecessary `Clone` bound forces callers to wrap closures in `Arc` or use clonable types when a simple `Fn` reference suffices.
- **Dependencies:** None.

### Task 6.2: Add `evaluate_owned` convenience method to `Lazy`

- **Files to modify:**
  - `fp-library/src/types/lazy.rs` (add method)
- **What:** Add `pub fn evaluate_owned(&self) -> A where A: Clone` that returns `self.evaluate().clone()`. This eliminates the `.evaluate().clone()` pattern that appears throughout user code.
- **Why:** Ergonomic improvement; the pattern is common enough to warrant a dedicated method.
- **Dependencies:** None.

### Task 6.3: Add fix combinators for `TryLazy`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs` (add `rc_try_lazy_fix` and `arc_try_lazy_fix` functions)
- **What:** Add fix combinators for self-referential fallible lazy values, analogous to `rc_lazy_fix`/`arc_lazy_fix` in `lazy.rs`.
- **Why:** Completes the `TryLazy`/`Lazy` parity. Use case is niche but the implementation is straightforward given the existing infallible versions.
- **Dependencies:** None.

### Task 6.4: Make `hoist_free` stack-safe

- **Files to modify:**
  - `fp-library/src/types/free.rs` (rewrite `hoist_free`)
- **What:** Replace the recursive `hoist_free` implementation with an iterative one using `resume` to step through `Wrap` layers, or use an explicit stack. The existing `fold_free` is already stack-safe and can serve as a reference.
- **Why:** The current implementation recurses once per `Wrap` layer and can overflow on deep chains. While deep `Wrap` chains are uncommon, making it stack-safe eliminates a potential footgun.
- **Dependencies:** Depends on Phase 4 (task 4.1) if `Evaluable` constraints are being relaxed.

---

## Phase 7: Testing and Verification

Validates all changes and fills testing gaps identified in the analysis.

### Task 7.1: Add QuickCheck property tests for `SendDeferrable`

- **Files to modify:**
  - `fp-library/src/classes/send_deferrable.rs` (add test module) or the relevant property test file
- **What:** Add property-based tests verifying the `SendDeferrable` laws (transparency, idempotence) for all implementors: `SendThunk`, `ArcLazy`, `TrySendThunk`, `ArcTryLazy`.
- **Why:** `Deferrable` has property tests but `SendDeferrable` does not. The eager evaluation behavior of `Deferrable` for Send types makes testing the truly-deferred `SendDeferrable` especially important.
- **Dependencies:** All prior phases.

### Task 7.2: Add QuickCheck property tests for `SendThunk` inherent methods

- **Files to modify:**
  - `fp-library/src/types/send_thunk.rs` (add/expand test module)
- **What:** Add QuickCheck tests verifying functor laws (identity, composition) and monad laws (left identity, right identity, associativity) for `SendThunk`'s inherent `map` and `bind` methods.
- **Why:** `Thunk` has QuickCheck law tests through its HKT trait impls; `SendThunk` provides the same operations as inherent methods but lacks equivalent law tests.
- **Dependencies:** Task 6.1 (if `Clone` bound is relaxed, tests should verify the relaxed signature).

### Task 7.3: Run full verification suite

- **What:** Run the standard verification sequence: `fmt -> clippy -> doc -> test` (using the test caching wrapper). Ensure zero warnings from `cargo doc` and zero clippy lints. Verify all new doc examples compile and pass.
- **Why:** Catches regressions from all prior phases.
- **Dependencies:** All prior phases.

---

## Summary of File Changes by Phase

| Phase | Files Modified | Files Created |
|-------|---------------|---------------|
| 1 | `classes.rs`, `types/lazy.rs`, `types/try_lazy.rs`, `brands.rs` | `classes/lazy_config.rs` |
| 2 | `types/try_lazy.rs`, `brands.rs`, `types/try_thunk.rs` (or `types/try_send_thunk.rs`) | None |
| 3 | `classes/monad_rec.rs`, `classes/evaluable.rs`, `types/trampoline.rs` | None |
| 4 | `types/free.rs` | None |
| 5 | `types/thunk.rs`, `types/identity.rs`, `types/try_lazy.rs` | None |
| 6 | `types/send_thunk.rs`, `types/lazy.rs`, `types/try_lazy.rs`, `types/free.rs` | None |
| 7 | `classes/send_deferrable.rs`, `types/send_thunk.rs` | None |
