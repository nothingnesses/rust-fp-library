# Lazy Evaluation Hierarchy: Implementation Plan

This plan addresses all issues identified in the [consolidated analysis summary](summary.md) and subsequent research conversations. Work is grouped into sequential phases, ordered so that foundational and blocking changes come first.

---

## Phase 1: Structural Foundations

Foundational changes that unblock or simplify later phases.

### Task 1.1: Move `LazyConfig` and `TryLazyConfig` trait definitions to `classes/`

- **Files to modify:**
  - `fp-library/src/classes.rs` (add two new module exports)
  - `fp-library/src/types/lazy.rs` (remove `LazyConfig` trait definition, keep concrete impls `RcLazyConfig`/`ArcLazyConfig`)
  - `fp-library/src/types/try_lazy.rs` (remove `TryLazyConfig` trait definition, update imports)
  - `fp-library/src/brands.rs` (update imports to point at `classes::` instead of `types::`)
- **Files to create:**
  - `fp-library/src/classes/lazy_config.rs` (new home for the `LazyConfig` trait definition)
  - `fp-library/src/classes/try_lazy_config.rs` (new home for the `TryLazyConfig` trait definition)
- **What:** Extract the `LazyConfig` trait definition into `classes/lazy_config.rs` and `TryLazyConfig` into `classes/try_lazy_config.rs`, following the library's convention of separate files for related-but-distinct traits (e.g., `foldable.rs`/`bifoldable.rs`). The concrete config structs (`RcLazyConfig`, `ArcLazyConfig`) and their trait impls remain in `types/lazy.rs`. Update all imports.
- **Why:** `brands.rs` currently imports from `types`, violating the `brands -> classes -> types` dependency graph. These config traits define behavior (like type classes), not concrete types.
- **Dependencies:** None. Pure refactor with no semantic changes.

### Task 1.2: Replace `Step` with `core::ops::ControlFlow`

- **Files to modify:**
  - `fp-library/src/brands.rs` (replace `StepBrand`, `StepLoopAppliedBrand`, `StepDoneAppliedBrand` with `ControlFlowBrand`, `ControlFlowBreakAppliedBrand`, `ControlFlowContinueAppliedBrand`)
  - `fp-library/src/types/step.rs` (rewrite: remove `Step` enum, add `ControlFlowBrand` with static helper methods and all type class impls reimplemented for `ControlFlow`)
  - `fp-library/src/classes/monad_rec.rs` (update `Step` references to `ControlFlow`)
  - All 16+ `MonadRec` implementor files (update `Step::Loop`/`Step::Done` to `ControlFlow::Continue`/`ControlFlow::Break`)
  - `fp-library/src/types/free.rs` (update `Step` usage in `fold_free`, etc.)
  - `fp-library/src/functions.rs` (update re-exports)
  - All test files referencing `Step`
- **What:** Replace the custom `Step<A, B>` enum with `core::ops::ControlFlow<B, C>`. Define brands with swapped type parameters to preserve HKT semantics:
  ```rust
  pub struct ControlFlowBrand;
  impl_kind! {
      for ControlFlowBrand {
          type Of<'a, C: 'a, B: 'a>: 'a = core::ops::ControlFlow<B, C>;
      }
  }

  // Fixes Break (done/result) type, functor over Continue (loop/state)
  pub struct ControlFlowBreakAppliedBrand<B>(PhantomData<B>);
  impl_kind! {
      impl<B: 'static> for ControlFlowBreakAppliedBrand<B> {
          type Of<'a, C: 'a>: 'a = core::ops::ControlFlow<B, C>;
      }
  }

  // Fixes Continue (loop/state) type, functor over Break (done/result)
  pub struct ControlFlowContinueAppliedBrand<C>(PhantomData<C>);
  impl_kind! {
      impl<C: 'static> for ControlFlowContinueAppliedBrand<C> {
          type Of<'a, B: 'a>: 'a = core::ops::ControlFlow<B, C>;
      }
  }
  ```
  The swapped parameters mirror `ResultBrand`'s pattern (`type Of<'a, E, A> = Result<A, E>`), ensuring the first HKT parameter is the continue/loop value and the second is the break/done value, matching `tail_rec_m` conventions.

  Reimplement all `Step`'s helper methods (`map_loop`/`map_done`/`bind`/`swap`/`bi_fold_*`/etc.) as static methods on `ControlFlowBrand`. Reimplement all type class instances (Bifunctor, Bifoldable, Bitraversable on the base brand; Functor through MonadRec on both applied brands).

  `Hash` and `Copy` are preserved (`ControlFlow` derives both when its fields do). Serde support (`Serialize`/`Deserialize` behind the `serde` feature flag) is lost because serde derives cannot be added to foreign types directly. If serde support is needed in the future, use `#[serde(remote = "ControlFlow")]` on a helper struct or implement `Serialize`/`Deserialize` manually. Remove `From` conversions between `Step` and `ControlFlow` (no longer needed).
- **Why:** Uses a standard library type instead of a custom enum. Provides interop with Rust's `?` operator and `Try` trait. Eliminates `From` conversion boilerplate. The swapped-parameter `impl_kind!` preserves all HKT semantics.
- **Dependencies:** None, but best done early since it touches many files.

### Task 1.3: Refactor `Free` to CatList-paired representation

- **Files to modify:**
  - `fp-library/src/types/free.rs` (rewrite internals)
- **What:** Replace the three-variant `FreeInner` enum (`Pure`, `Wrap`, `Bind`) with a two-component structure pairing a `FreeView` (`Return(value)` or `Suspend(functor)`) with a `CatList<Continuation<F>>`. This matches PureScript's design where every `Free` value is `(FreeView, CatList)`.

  Key changes:
  - `bind` becomes uniformly O(1) `snoc` for all cases (no special-casing for Pure/Wrap).
  - `pure(a).bind(f1).bind(f2)` produces `(Return(a), CatList[f1, f2])` instead of nested `Bind { head: Bind { head: Pure(a), ... }, ... }`.
  - `evaluate` simplifies to a two-way match (Return/Suspend) with continuation merging.
  - The public API (`pure`, `wrap`, `bind`, `map`, `evaluate`, `resume`, `fold_free`, `hoist_free`) is unchanged.

  This eliminates the "Bind on Pure creates unnecessary nesting" issue identified in the analysis: no extra boxing when binding on Pure, fewer branches in evaluate, and closer alignment with PureScript's proven design.
- **Why:** Reduces per-bind allocations for Pure values, simplifies the evaluate loop, eliminates a class of unnecessary nesting, and aligns with the PureScript reference implementation.
- **Dependencies:** None, but should be done before other Free-related changes (Tasks 4.1, 6.4, 8.3).

---

## Phase 2: Missing Trait Implementations (High Priority)

Fills obvious gaps in the type class hierarchy. These are small, self-contained additions.

### Task 2.1: Add `WithIndex` and `FoldableWithIndex` for `TryLazyBrand<E, Config>`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs` (add trait impls)
- **What:** Implement `WithIndex` with `type Index = ()` and `FoldableWithIndex` (delegating to `Foldable` with the unit index) for `TryLazyBrand<E, Config>`, mirroring the existing implementations on `LazyBrand<Config>`.
- **Why:** `LazyBrand` has both traits; `TryLazyBrand` does not. Both are single-element containers where `Index = ()` is the natural choice.
- **Dependencies:** None.

### Task 2.2: Remove `TrySendThunkBrand`

- **Files to modify:**
  - `fp-library/src/brands.rs` (remove `TrySendThunkBrand` definition and `impl_kind!`)
  - `fp-library/src/types/try_send_thunk.rs` (remove any brand references, add a doc comment explaining why no brand exists)
- **What:** Remove `TrySendThunkBrand` and its `impl_kind!`. The brand cannot soundly implement any closure-accepting HKT trait (`Bifunctor`, `Bifoldable`, `Functor`, etc.) because the trait signatures use `impl Fn(A) -> B + 'a` without a `Send` bound, but `TrySendThunk` internally stores a `Send` closure. Composing a non-`Send` closure from the trait with the internal `Send` closure would violate the `Send` invariant. Add a doc comment on `TrySendThunk` explaining this limitation.
- **Why:** The brand has zero type class implementations and cannot gain any. It creates false expectations of HKT support.
- **Dependencies:** None.

### Task 2.3: Add `From<TrySendThunk> for TryThunk`

- **Files to modify:**
  - `fp-library/src/types/try_thunk.rs` (add `From` impl)
- **What:** Implement `From<TrySendThunk<'a, A, E>> for TryThunk<'a, A, E>` using the same unsizing coercion pattern as the existing `From<SendThunk<'a, A>> for Thunk<'a, A>`.
- **Why:** Fills a gap in the conversion web. The infallible pair already exists.
- **Dependencies:** None.

---

## Phase 3: New Type Classes

Renames `Evaluable` to `Extract` and introduces `Extend` and `Comonad` to complete the categorical duality with `Monad`.

### Task 3.1: Rename `Evaluable` to `Extract`

- **Files to modify:**
  - `fp-library/src/classes/evaluable.rs` (rename trait to `Extract`, rename method `evaluate` to `extract`)
  - `fp-library/src/classes.rs` (update module name/export)
  - `fp-library/src/types/thunk.rs` (update `Evaluable` impl to `Extract`, method name)
  - `fp-library/src/types/free.rs` (update all `Evaluable` bounds to `Extract`, all `Evaluable::evaluate` calls to `Extract::extract`)
  - `fp-library/src/functions.rs` (update re-export)
  - All files importing `Evaluable` or calling the free function `evaluate`
- **What:** Rename the `Evaluable` trait to `Extract` and its method from `evaluate` to `extract`. Remove the `Functor` supertrait from `Extract`; the `Functor` constraint belongs on `Extend` instead (see Task 3.2). Update `Free`'s bounds from `F: Evaluable` to `F: Extract + Functor` where both are needed. Rename the free function wrapper from `evaluate` to `extract`. Rename the file from `evaluable.rs` to `extract.rs`.

  Inherent `evaluate` methods on `Thunk`, `Lazy`, `TryLazy`, `Trampoline`, `TryTrampoline`, `Free`, etc. are **not renamed**; they mean "force/run this computation," which is a broader operation than categorical extraction.
- **Why:** `extract` is the standard categorical name (matching PureScript/Haskell). Removing `Functor` from `Extract` allows it to stand independently; `Functor` is a law-level consequence of `Extend`, not of `Extract` alone. The map-extract law (`extract(map(f, fa)) == f(extract(fa))`) belongs to `Comonad` (the combination), not to `Extract` in isolation.
- **Dependencies:** None. This is a rename + constraint adjustment.

### Task 3.2: Implement `Extend` trait

- **Files to create:**
  - `fp-library/src/classes/extend.rs`
- **Files to modify:**
  - `fp-library/src/classes.rs` (add module export)
  - `fp-library/src/functions.rs` (re-export free functions)
- **What:** Define the `Extend` trait with `Functor` as its supertrait:
  ```rust
  pub trait Extend: Functor {
      fn extend<'a, A: 'a, B: 'a>(
          f: impl Fn(Apply!(Self::Of<'a, A>)) -> B + 'a,
          wa: Apply!(Self::Of<'a, A>),
      ) -> Apply!(Self::Of<'a, B>);
  }
  ```
  Provide a default `duplicate` method: `duplicate(wa) = extend(identity, wa)`.
  Add free function wrappers `extend` and `duplicate`.

  The `Functor` supertrait is here (not on `Extract`) because `Extend`'s associativity law (`extend f <<< extend g = extend (f <<< extend g)`) implies a lawful `Functor` must exist, just as `Monad`'s laws imply `Functor`.
- **Why:** Completes the Functor -> Extend -> Comonad tower, dual to Functor -> Monad.
- **Dependencies:** None.

### Task 3.3: Implement `Comonad` trait as `Extend + Extract`

- **Files to create:**
  - `fp-library/src/classes/comonad.rs`
- **Files to modify:**
  - `fp-library/src/classes.rs` (add module export)
- **What:** Define `Comonad` as a blanket impl:
  ```rust
  pub trait Comonad: Extend + Extract {}
  impl<Brand: Extend + Extract> Comonad for Brand {}
  ```
  Document the comonad laws that arise from the combination:
  - Left identity: `extract(extend(f, wa)) == f(wa)`
  - Right identity: `extend(extract, wa) == wa`
  - Map-extract: `extract(map(f, fa)) == f(extract(fa))`

  The hierarchy is:
  ```
  Functor
    |
    +-- Extract              (extract :: F<A> -> A; no Functor constraint)
    |
    +-- Extend: Functor      (extend :: (F<A> -> B) -> F<A> -> F<B>)
    |
    +-- Comonad: Extend + Extract   (blanket impl, no new methods)
  ```
- **Why:** Provides the standard comonadic abstraction. Types implementing both `Extend` and `Extract` automatically get `Comonad`.
- **Dependencies:** Tasks 3.1, 3.2.

### Task 3.4: Implement `Extract` and `Extend` for `IdentityBrand`

- **Files to modify:**
  - `fp-library/src/types/identity.rs` (add `Extract` and `Extend` impls)
- **What:** Implement `Extract` with `extract(Identity(a)) = a` and `Extend` with `extend(f, wa) = Identity(f(wa))`. This makes `IdentityBrand` the first `Comonad` via the blanket impl.
- **Why:** Validates both `Extract` (second implementor after `ThunkBrand`) and the `Comonad` blanket impl. Also enables `Free<IdentityBrand, A>` as a degenerate case.
- **Dependencies:** Tasks 3.1, 3.2, 3.3.

### Task 3.5: Implement `Extend` for `ThunkBrand`

- **Files to modify:**
  - `fp-library/src/types/thunk.rs` (add `Extend` impl)
- **What:** Implement `Extend` for `ThunkBrand` with `extend(f, thunk) = Thunk::new(move || f(thunk))`. This makes `ThunkBrand` a `Comonad` (it already implements `Extract`).
- **Why:** `Thunk` is the primary lazy type with both extraction and contextual mapping capabilities.
- **Dependencies:** Tasks 3.1, 3.2.

### Task 3.6: Implement `Extend` for `LazyBrand<Config>` (without Comonad)

- **Files to modify:**
  - `fp-library/src/types/lazy.rs` (add `Extend` impl)
- **What:** Implement `Extend` for `LazyBrand<Config>` with `extend(f, lazy) = Lazy::new(move || f(lazy))`. Note: `LazyBrand` cannot implement `Extract` (its `evaluate` returns `&A`, not `A`), so it gets `Extend` only, not `Comonad`.
- **Why:** Memoized lazy values support contextual mapping even though they cannot extract owned values.
- **Dependencies:** Task 3.2.

---

## Phase 4: Documentation Fixes

Addresses documentation gaps that affect correctness understanding. Text-only changes with no code impact.

### Task 4.1: Document the unfolding/equivalence law for `MonadRec`

- **Files to modify:**
  - `fp-library/src/classes/monad_rec.rs` (update trait-level doc comment)
- **What:** Add the equivalence law: `tail_rec_m(f, a)` is equivalent to `f(a) >>= match { Continue(a') => tail_rec_m(f, a'), Break(b) => pure(b) }`.
- **Why:** The trait currently states only the identity law. The equivalence law is critical for reasoning about correctness.
- **Dependencies:** Task 1.2 (uses ControlFlow terminology).

### Task 4.2: Document the pure-extract law for `Extract` and cross-reference with `Deferrable`

- **Files to modify:**
  - `fp-library/src/classes/extract.rs` (update trait-level doc comment)
  - `fp-library/src/classes/deferrable.rs` (add cross-reference)
- **What:** Add the law `extract(pure(x)) == x` to `Extract` docs. Document that `Extract` and `Deferrable` are duals: `Deferrable` constructs lazy values from thunks, `Extract` forces/extracts them. For types implementing both (like `Thunk`), `extract(defer(|| x)) == x` forms a round-trip. Add reciprocal cross-references on both traits.
- **Why:** `Free::evaluate` implicitly relies on this law. The duality is a key conceptual relationship that aids understanding.
- **Dependencies:** Task 3.1 (rename must be done first).

### Task 4.3: Add algebraic properties and limitations sections to `TryTrampoline`

- **Files to modify:**
  - `fp-library/src/types/try_trampoline.rs` (update doc comments)
- **What:** Add structured documentation for algebraic properties (monad laws, short-circuiting) and limitations (`'static`, no HKT brand, `!Send`). Model after `TryThunk` and `Trampoline` docs.
- **Why:** `TryTrampoline` is the only major type lacking structured property documentation.
- **Dependencies:** None.

### Task 4.4: Document nondeterministic termination caveat for multi-element `MonadRec`

- **Files to modify:**
  - `fp-library/src/classes/monad_rec.rs` (add note to trait docs or impl docs)
- **What:** Note that for `VecBrand`/`CatListBrand`, if the step function always produces `Continue` values, the computation never terminates and consumes unbounded memory.
- **Dependencies:** Task 1.2 (uses ControlFlow terminology).

---

## Phase 5: Free Monad Constraint Relaxation

Loosens overly broad type constraints on `Free`.

### Task 5.1: Relax `Extract` constraint on `Free` construction methods

- **Files to modify:**
  - `fp-library/src/types/free.rs` (split `impl` blocks by constraint level)
- **What:** Move `pure`, `bind`, `map`, and `lift_f` into an `impl<F: Functor, A>` block (or unconstrained where possible), reserving `F: Extract + Functor` only for `evaluate`, `resume`, and methods that call `Extract::extract`. Audit each method for its minimal constraint.
- **Why:** Currently all methods require `F: Extract` (formerly `Evaluable`), preventing construction of `Free` values over functors that are not `Extract`. PureScript's Free only requires `Functor` for structural operations.
- **Dependencies:** Tasks 1.3 (CatList-paired refactor), 3.1 (rename).

### Task 5.2: Add `to_view` helper and `subst_free` method to `Free`

- **Files to modify:**
  - `fp-library/src/types/free.rs`
- **What:** Factor the iterative loop from `evaluate` and `resume` into a shared `to_view`-like helper that returns either `Return(a)` or `Suspend(fa, continuation)`. Then implement `subst_free` which folds `Free<F, A>` into `Free<G, A>` using a natural transformation `F ~> Free<G>`, without requiring `MonadRec` on the target (unlike `fold_free`).
- **Why:** Reduces code duplication between `evaluate` and `resume`. `subst_free` enables Free-to-Free transformations without the `MonadRec` overhead of `fold_free`.
- **Dependencies:** Task 1.3 (CatList-paired refactor simplifies the view concept).

---

## Phase 6: Additional Trait Implementations and Combinators (Medium Priority)

Expands the hierarchy's utility.

### Task 6.1: Add `Display` for `TryLazy`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs`
- **What:** Implement `Display` mirroring `Lazy`'s implementation.
- **Why:** Restores parity with `Lazy`.
- **Dependencies:** None.

### Task 6.2: Add `Display` for `CatList`

- **Files to modify:**
  - `fp-library/src/types/cat_list.rs`
- **What:** Implement `Display for CatList<A: Display>` showing elements in list notation (e.g., `[1, 2, 3]`).
- **Why:** Natural representation for a list type. Trivial to implement via `self.iter()`.
- **Dependencies:** None.

### Task 6.3: Add cross-config conversions for `TryLazy`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs`
- **What:** Add `From<RcTryLazy> for ArcTryLazy` and vice versa, following `Lazy`'s pattern.
- **Why:** Completes the conversion matrix.
- **Dependencies:** None.

### Task 6.4: Add `resume` method to `Trampoline` and `TryTrampoline`

- **Files to modify:**
  - `fp-library/src/types/trampoline.rs`
  - `fp-library/src/types/try_trampoline.rs`
- **What:** Expose `Free::resume` on `Trampoline` as `fn resume(self) -> Result<A, Thunk<'static, Trampoline<A>>>` and analogously on `TryTrampoline`. This decomposes a computation into one step without full evaluation, enabling introspection.
- **Why:** `resume` is a fundamental Free monad operation that is currently available only on `Free` directly.
- **Dependencies:** Task 1.3 (benefits from the CatList-paired refactor).

### Task 6.5: Add derived `MonadRec` combinators

- **Files to modify:**
  - `fp-library/src/classes/monad_rec.rs` (add free functions)
  - `fp-library/src/functions.rs` (re-export)
- **What:** Implement derived combinators as free functions. Phase 1 (MVP):
  - `forever<Brand: MonadRec>(action) -> m b` : run an action indefinitely (stack-safe).
  - `while_some<Brand: MonadRec, A: Monoid>(computation) -> m A` : loop accumulating via Monoid until `None`.
  - `until_some<Brand: MonadRec>(computation) -> m A` : loop until `Some(x)`.
  - `repeat_m<Brand: MonadRec>(n, f, initial) -> m S` : apply step function n times.
  - `while_m<Brand: MonadRec>(condition, body)` : loop while monadic condition holds.
  - `until_m<Brand: MonadRec>(condition, body)` : loop until monadic condition becomes true.

  Phase 2 (follow-up):
  - `fold_m` : stack-safe monadic fold over a `Foldable`.
  - `find_m` : search with monadic predicate, short-circuit on match.
  - `any_m` / `all_m` : monadic predicates with short-circuit.
  - `for_each_m` : side-effect iteration discarding results.
  - `scan_m` : collect intermediate fold values.
  - `do_until_m` : at-least-once iteration.

  All are implemented internally via `tail_rec_m`, ensuring stack safety.
- **Why:** Makes `MonadRec` practically useful beyond manual `tail_rec_m` calls. These combinators are standard in PureScript's `purescript-tailrec` and Haskell's `monad-loops`.
- **Dependencies:** Task 1.2 (uses ControlFlow).

---

## Phase 7: Minor Improvements (Low Priority)

Small quality-of-life improvements.

### Task 7.1: Use weak references in Lazy fix combinators

- **Files to modify:**
  - `fp-library/src/types/lazy.rs` (update `rc_lazy_fix` and `arc_lazy_fix`)
- **What:** Replace the strong `Rc`/`Arc` clone captured in the fix combinator closures with `Rc::downgrade`/`Arc::downgrade`. The closure captures a `Weak` reference; during evaluation, `weak.upgrade()` succeeds because the outer reference is still alive. If dropped without evaluation, no cycle exists and memory is reclaimed.
- **Why:** Eliminates the documented memory leak hazard when fix-constructed lazy values are dropped without evaluation. No API change, negligible performance cost.
- **Dependencies:** None.

### Task 7.2: Add fix combinators for `TryLazy`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs`
- **What:** Add `rc_try_lazy_fix` and `arc_try_lazy_fix`, analogous to `rc_lazy_fix`/`arc_lazy_fix`. Use weak references from the start (per Task 7.1).
- **Why:** Completes `TryLazy`/`Lazy` parity.
- **Dependencies:** Task 7.1 (use the improved weak-reference pattern).

### Task 7.3: Add `evaluate_owned` convenience method to `Lazy`

- **Files to modify:**
  - `fp-library/src/types/lazy.rs`
- **What:** Add `pub fn evaluate_owned(&self) -> A where A: Clone` returning `self.evaluate().clone()`.
- **Why:** Eliminates the common `.evaluate().clone()` pattern.
- **Dependencies:** None.

### Task 7.4: Make `hoist_free` stack-safe

- **Files to modify:**
  - `fp-library/src/types/free.rs`
- **What:** Replace the recursive `hoist_free` with an iterative implementation using `resume` or an explicit stack.
- **Why:** The current implementation recurses per `Wrap` layer and can overflow on deep chains.
- **Dependencies:** Tasks 1.3, 5.1.

### Task 7.5: Relax `Clone` bound on `SendThunk::tail_rec_m`

- **Files to modify:**
  - `fp-library/src/types/send_thunk.rs`
- **What:** `SendThunk::tail_rec_m` uses an iterative loop (not recursion), so the step function is called by reference and does not need `Clone`. Remove the unnecessary `Clone` bound.
- **Why:** Reduces friction for callers without changing behavior.
- **Dependencies:** None.

---

## Phase 8: Testing and Verification

Validates all changes and fills testing gaps.

### Task 8.1: Add QuickCheck property tests for `SendDeferrable`

- **Files to modify:**
  - `fp-library/src/classes/send_deferrable.rs` (add test module)
- **What:** Property tests verifying `SendDeferrable` laws (transparency, idempotence) for `SendThunk`, `ArcLazy`, `TrySendThunk`, `ArcTryLazy`.
- **Why:** `Deferrable` has property tests but `SendDeferrable` does not.
- **Dependencies:** All prior phases.

### Task 8.2: Add QuickCheck property tests for `SendThunk` inherent methods

- **Files to modify:**
  - `fp-library/src/types/send_thunk.rs`
- **What:** QuickCheck tests verifying functor laws and monad laws for `SendThunk`'s inherent `map` and `bind`.
- **Why:** `Thunk` has law tests through HKT traits; `SendThunk` lacks equivalent coverage.
- **Dependencies:** Task 7.5.

### Task 8.3: Add tests for new ControlFlow brands

- **Files to modify:**
  - `fp-library/src/types/step.rs` (or renamed file)
- **What:** Verify all type class instances (Bifunctor, Bifoldable, Bitraversable, Functor, Monad, MonadRec on both applied brands) with QuickCheck property tests. Port existing Step tests to use ControlFlow.
- **Why:** Ensures the replacement preserves all algebraic properties.
- **Dependencies:** Task 1.2.

### Task 8.4: Add tests for `Extend`/`Comonad` instances

- **Files to modify:**
  - Test files for identity, thunk, lazy
- **What:** QuickCheck property tests for comonad laws: `extract . duplicate == id`, `fmap extract . duplicate == id`, `duplicate . duplicate == fmap duplicate . duplicate`.
- **Why:** Validates the new type class hierarchy.
- **Dependencies:** Phase 3.

### Task 8.5: Add tests for derived `MonadRec` combinators

- **Files to modify:**
  - `fp-library/src/classes/monad_rec.rs` (test module)
- **What:** Unit and property tests for `forever` (verify stack safety at 100k+ iterations), `while_some`, `until_some`, `repeat_m`, `while_m`, `until_m`, and Phase 2 combinators.
- **Why:** Ensures stack safety and correctness of all new combinators.
- **Dependencies:** Task 6.7.

### Task 8.6: Run full verification suite

- **What:** Run `fmt -> clippy -> doc -> test`. Ensure zero warnings from `cargo doc` and zero clippy lints.
- **Dependencies:** All prior phases.

---

## Decisions and Rationale

This section documents design decisions reached during research, for future reference.

### SendFree is not feasible

A `Send` variant of `Free` (and by extension `SendTrampoline`/`SendTryTrampoline`) is not feasible with the current architecture. `Free` uses `Box<dyn Any>` for type erasure and `Box<dyn FnOnce(TypeErasedValue) -> Free<F, TypeErasedValue>>` for continuations. Making these `Send` creates a circular dependency: `Send Free` requires `Send Continuation` which returns `Send Free`. Even if `SendThunkBrand` implemented `Extract`, `Free`'s internals would still be `!Send`. For cross-thread stack-safe computation, use `SendThunk` to defer creation of a `Trampoline`, or evaluate a `Trampoline` on one thread and send the result.

### `Functor` belongs on `Extend`, not on `Extract`

`Extract` (formerly `Evaluable`) does not require `Functor`. The `extract` operation is purely "pull a value out." The map-extract law (`extract(map(f, fa)) == f(extract(fa))`) only applies when both `Extract` and `Functor` are present, so it belongs to `Comonad` (which gets `Functor` via its `Extend` supertrait). `Extend: Functor` is correct because `Extend`'s associativity law implies a lawful `Functor` must exist, mirroring how `Monad`'s laws imply `Functor`. `Free` uses `F: Extract + Functor` in its where clause for `evaluate`/`resume`, making the requirements explicit at the use site.

### `Traversable` for `ThunkBrand` is infeasible

`Traversable::traverse` requires `Apply!(Self::Of<'a, B>): Clone`, i.e., `Thunk<'a, B>: Clone`. `Thunk` wraps `Box<dyn FnOnce() -> A + 'a>`, which is inherently `!Clone` because `FnOnce` closures are consumed on invocation. This is true regardless of whether `A: Clone`. The `!Clone` is an intentional design choice: `Thunk` is a single-shot deferred computation. Cloning it would require either caching the result (becoming `Lazy`) or using `Fn` instead of `FnOnce` (losing move-only captures). This is already documented in `thunk.rs`.

### `CatList::map` should flatten (not preserve structure)

PureScript's `CatList` Functor instance also flattens during `map`: it calls `foldr link CatNil q` to collapse the internal queue before recursing. The Rust implementation's `into_iter().map(f).collect()` achieves the same flattening. Structure-preserving `map` would diverge from the reference implementation and the internal tree structure is an implementation detail of O(1) append, not something users should depend on. Flattening during `map` normalizes the structure, improving subsequent iteration performance.

### `TrySendThunkBrand` cannot implement any HKT traits

`Bifunctor::bimap`, `Bifoldable::bi_fold_*`, and all other closure-accepting HKT traits use `impl Fn(A) -> B + 'a` without a `Send` bound. `TrySendThunk` stores a `Send` closure internally. Composing a non-`Send` closure from the trait with the internal `Send` closure would produce a result that violates the `Send` invariant. This is the same fundamental blocker as `Functor` for `SendThunkBrand`. The brand should be removed.

### `Functor` should not require `Clone` on `A`

Adding `A: Clone` to `Functor` would not enable `Lazy` types to implement `Functor`. The core issue is closure signature compatibility: `Functor::map` takes `Fn(A) -> B` (owned `A`), while `Lazy::evaluate` returns `&A`. Even with `A: Clone`, the implementation would need to silently clone, violating the library's zero-cost abstraction principle. It would also break code using non-`Clone` types and propagate unnecessarily to `Applicative`, `Monad`, etc. The `RefFunctor`/`SendRefFunctor` split correctly reflects genuinely different capabilities.

### `tail_rec_m` `Clone` bound vs `arc_tail_rec_m` `Arc` is intentional

`tail_rec_m` requires `Clone` on the step function because the recursive inner `go` function moves `f` into a `defer` closure while also passing it to the recursive call. `arc_tail_rec_m` wraps `f` in `Arc` first, providing shared ownership without `Clone`. This gives users a zero-overhead path (`Clone`) when possible and a fallback (`Arc`) when the closure captures non-Clone state. Unifying them (always using `Arc`) would add unnecessary reference-counting overhead; always requiring `Clone` would remove the non-Clone closure capability.

---

## Summary of File Changes by Phase

| Phase | Files Modified | Files Created |
|-------|---------------|---------------|
| 1 | `classes.rs`, `types/lazy.rs`, `types/try_lazy.rs`, `brands.rs`, `types/step.rs`, `classes/monad_rec.rs`, all MonadRec implementors, `types/free.rs`, `functions.rs` | `classes/lazy_config.rs` |
| 2 | `types/try_lazy.rs`, `brands.rs`, `types/try_send_thunk.rs`, `types/try_thunk.rs` | None |
| 3 | `classes/evaluable.rs` (renamed to `classes/extract.rs`), `types/thunk.rs`, `types/free.rs`, `types/identity.rs`, `types/lazy.rs`, `classes.rs`, `functions.rs`, all files importing `Evaluable` | `classes/extend.rs`, `classes/comonad.rs` |
| 4 | `classes/monad_rec.rs`, `classes/extract.rs`, `classes/deferrable.rs`, `types/try_trampoline.rs` | None |
| 5 | `types/free.rs` | None |
| 6 | `types/try_lazy.rs`, `types/cat_list.rs`, `types/trampoline.rs`, `types/try_trampoline.rs`, `classes/monad_rec.rs`, `functions.rs` | None |
| 7 | `types/lazy.rs`, `types/try_lazy.rs`, `types/free.rs`, `types/send_thunk.rs` | None |
| 8 | Test files across the crate | None |
