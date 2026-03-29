# Lazy Evaluation Hierarchy: Implementation Plan

This plan addresses all issues identified in the [consolidated analysis summary](summary.md) and subsequent research conversations. Work is grouped into sequential phases, ordered so that foundational and blocking changes come first. Each phase includes its own testing tasks to ensure the test suite stays green throughout.

**Parallelism:** Tasks within a phase that share no dependencies can be worked on concurrently. These are noted with "(parallelizable)" in the phase description.

---

## Phase 1: Structural Foundations

Foundational changes that unblock or simplify later phases. Tasks 1.1, 1.2, and 1.3 are independent and parallelizable.

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
- **Docs:** Update `MonadRec` trait docs to use `ControlFlow::Continue`/`ControlFlow::Break` terminology. Add the equivalence law: `tail_rec_m(f, a)` is equivalent to `f(a) >>= match { Continue(a') => tail_rec_m(f, a'), Break(b) => pure(b) }`. Add nondeterministic termination caveat for `VecBrand`/`CatListBrand`: if the step function always produces `Continue`, the computation never terminates and consumes unbounded memory.

### Task 1.3: Refactor `Free` to CatList-paired representation

- **Reference:** [PureScript's Free monad](../../../purescript-free/src/Control/Monad/Free.purs)
- **Files to modify:**
  - `fp-library/src/types/free.rs` (rewrite internals)
- **What:** Replace the current three-variant `FreeInner` enum with a two-component structure pairing a `FreeView` with a `CatList<Continuation<F>>`, matching PureScript's design.

  **Current Rust representation (three variants):**

  ```rust
  enum FreeInner<F, A> {
      Pure(A),
      Wrap(F::Of<'static, Free<F, A>>),
      Bind {
          head: Box<Free<F, TypeErasedValue>>,
          continuations: CatList<Continuation<F>>,
          _marker: PhantomData<A>,
      },
  }
  pub struct Free<F, A>(Option<FreeInner<F, A>>);
  ```

  **PureScript representation (view + CatList):**

  ```purescript
  data Free f a = Free (FreeView f Val Val) (CatList (ExpF f))
  data FreeView f a b = Return a | Bind (f b) (b -> Free f a)
  ```

  **Proposed Rust representation:**

  ```rust
  enum FreeView<F> {
      Return(TypeErasedValue),
      Suspend(Apply!(<F as Kind!(...)>::Of<'static, Free<F, TypeErasedValue>>)),
  }
  pub struct Free<F, A> {
      view: FreeView<F>,
      continuations: CatList<Continuation<F>>,
      _marker: PhantomData<A>,
  }
  ```

  Note: PureScript's `Bind(f b, b -> Free f a)` variant stores both a functor value and a continuation in the view. The Rust `Suspend` variant stores only the functor value (equivalent to PureScript's `Bind(f b, identity)`), because the continuation is always in the CatList. Both approaches are valid; the simpler `Suspend`-only view is sufficient since `wrap` and `liftF` can place their continuations in the CatList.

  **Key change in `bind`:**

  Current Rust `bind` has three branches:
  ```rust
  // Current: bind on Pure creates unnecessary nesting
  FreeInner::Pure(a) => {
      let head = Free::from_inner(FreeInner::Pure(Box::new(a)));
      Free::from_inner(FreeInner::Bind {
          head: Box::new(head),        // extra Box
          continuations: CatList::singleton(erased_f),
      })
  }
  // Current: bind on Wrap also creates a Bind node
  FreeInner::Wrap(fa) => {
      let head = Free::wrap(fa).boxed_erase_type();
      Free::from_inner(FreeInner::Bind { head, continuations: CatList::singleton(erased_f) })
  }
  // Current: bind on Bind appends (O(1))
  FreeInner::Bind { head, continuations, .. } => {
      Free::from_inner(FreeInner::Bind { head, continuations: conts.snoc(erased_f) })
  }
  ```

  PureScript's `bind` is a single case:
  ```purescript
  bind (Free v s) k = Free v (snoc s k)
  ```

  Proposed Rust `bind` becomes uniformly O(1) with no branching:
  ```rust
  fn bind(self, f: ...) -> Free<F, B> {
      Free {
          view: self.view,   // view unchanged
          continuations: self.continuations.snoc(erased_f),
          _marker: PhantomData,
      }
  }
  ```

  **Key change in `evaluate`:**

  Current Rust `evaluate` matches three variants:
  ```rust
  loop {
      match current.take_inner() {
          FreeInner::Pure(val) => { /* apply next continuation or return */ }
          FreeInner::Wrap(fa) => { current = F::evaluate(fa); }
          FreeInner::Bind { head, continuations, .. } => {
              current = *head;
              outer_continuations = continuations.append(outer_continuations);
          }
      }
  }
  ```

  Proposed `evaluate` matches two variants, analogous to PureScript's `toView`:
  ```rust
  loop {
      // Merge this node's continuations into the outer queue
      continuations = current.continuations.append(continuations);
      match current.view {
          FreeView::Return(val) => {
              match continuations.uncons() {
                  Some((k, rest)) => { current = k(val); continuations = rest; }
                  None => return downcast(val),
              }
          }
          FreeView::Suspend(fa) => {
              current = F::extract(fa);  // (formerly F::evaluate)
          }
      }
  }
  ```

  The public API (`pure`, `wrap`, `bind`, `map`, `evaluate`, `resume`, `fold_free`, `hoist_free`) is unchanged.

- **Why:** Eliminates the "Bind on Pure creates unnecessary nesting" issue: `pure(a).bind(f1).bind(f2)` produces `(Return(a), CatList[f1, f2])` instead of nested `Bind { head: Bind { head: Pure(a), ... }, ... }`. Reduces per-bind allocations for Pure/Wrap values (no extra `Box` wrapping). Simplifies the evaluate loop (two-way match instead of three-way). Aligns with PureScript's proven "Reflection without Remorse" design.
- **Dependencies:** None, but should be done before other Free-related changes (Tasks 5.1, 5.2, 7.4).

### Task 1.4: Phase 1 verification

- **What:** Run `fmt -> clippy -> doc -> test`. Ensure all existing tests pass after the structural changes. Port existing `Step` tests to use `ControlFlow`. Verify all `MonadRec` implementors produce correct results with the new type.
- **Dependencies:** Tasks 1.1, 1.2, 1.3.

---

## Phase 2: Gap Fills

Fills obvious gaps in the type class hierarchy and conversion web. All tasks are independent and parallelizable.

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

### Task 2.4: Add `Display` for `TryLazy`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs`
- **What:** Implement `Display` mirroring `Lazy`'s implementation.
- **Why:** Restores parity with `Lazy`.
- **Dependencies:** None.

### Task 2.5: Add `Display` for `CatList`

- **Files to modify:**
  - `fp-library/src/types/cat_list.rs`
- **What:** Implement `Display for CatList<A: Display>` showing elements in list notation (e.g., `[1, 2, 3]`).
- **Why:** Natural representation for a list type. Trivial to implement via `self.iter()`.
- **Dependencies:** None.

### Task 2.6: Add cross-config conversions for `TryLazy`

- **Files to modify:**
  - `fp-library/src/types/try_lazy.rs`
- **What:** Add `From<RcTryLazy> for ArcTryLazy` and vice versa, following `Lazy`'s pattern.
- **Why:** Completes the conversion matrix.
- **Dependencies:** None.

### Task 2.7: Phase 2 verification

- **What:** Run `fmt -> clippy -> doc -> test`. Verify new `From` impls, `Display` impls, and `FoldableWithIndex` with unit tests.
- **Dependencies:** Tasks 2.1 through 2.6.

---

## Phase 3: New Type Classes

Renames `Evaluable` to `Extract` and introduces `Extend` and `Comonad` to complete the categorical duality with `Monad`. Tasks 3.1 and 3.2 are independent and parallelizable. Tasks 3.4, 3.5, and 3.6 are independent and parallelizable (after 3.3).

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
- **Docs:** Add the law `extract(pure(x)) == x` to `Extract` docs. Document that `Extract` and `Deferrable` are duals: `Deferrable` constructs lazy values from thunks, `Extract` forces/extracts them. For types implementing both (like `Thunk`), `extract(defer(|| x)) == x` forms a round-trip. Add reciprocal cross-references on `Deferrable`.
- **Why:** `extract` is the standard categorical name (matching PureScript/Haskell). Removing `Functor` from `Extract` allows it to stand independently; `Functor` is a law-level consequence of `Extend`, not of `Extract` alone. The map-extract law (`extract(map(f, fa)) == f(extract(fa))`) belongs to `Comonad` (the combination), not to `Extract` in isolation.
- **Dependencies:** None. This is a rename + constraint adjustment.

### Task 3.2: Implement `Extend` trait

- **Reference:** [PureScript's `Control.Extend`](../../../purescript-control/src/Control/Extend.purs)
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

- **Reference:** [PureScript's `Control.Comonad`](../../../purescript-control/src/Control/Comonad.purs)
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

### Task 3.7: Phase 3 verification

- **What:** Run `fmt -> clippy -> doc -> test`. Add QuickCheck property tests for comonad laws (`extract . duplicate == id`, `fmap extract . duplicate == id`, `duplicate . duplicate == fmap duplicate . duplicate`) for `IdentityBrand`, `ThunkBrand`, and `LazyBrand` (Extend-only laws for Lazy). Verify all existing tests still pass after the `Evaluable` -> `Extract` rename.
- **Dependencies:** Tasks 3.1 through 3.6.

---

## Phase 4: Documentation

Addresses documentation gaps that affect correctness understanding. Text-only changes with no code impact. All tasks are independent and parallelizable.

### Task 4.1: Add algebraic properties and limitations sections to `TryTrampoline`

- **Files to modify:**
  - `fp-library/src/types/try_trampoline.rs` (update doc comments)
- **What:** Add structured documentation for algebraic properties (monad laws, short-circuiting) and limitations (`'static`, no HKT brand, `!Send`). Model after `TryThunk` and `Trampoline` docs.
- **Why:** `TryTrampoline` is the only major type lacking structured property documentation.
- **Dependencies:** None.

### Task 4.2: Phase 4 verification

- **What:** Run `cargo doc --workspace --all-features --no-deps` and verify zero warnings.
- **Dependencies:** Task 4.1.

---

## Phase 5: Free Monad Improvements

Loosens overly broad type constraints on `Free` and adds new operations. Tasks 5.1 and 5.2 are independent and parallelizable.

### Task 5.1: Relax `Extract` constraint on `Free` construction methods

- **Files to modify:**
  - `fp-library/src/types/free.rs` (split `impl` blocks by constraint level)
- **What:** Move `pure`, `bind`, `map`, and `lift_f` into an `impl<F: Functor, A>` block (or unconstrained where possible), reserving `F: Extract + Functor` only for `evaluate`, `resume`, and methods that call `Extract::extract`. Audit each method for its minimal constraint.
- **Why:** Currently all methods require `F: Extract` (formerly `Evaluable`), preventing construction of `Free` values over functors that are not `Extract`. PureScript's Free only requires `Functor` for structural operations.
- **Dependencies:** Tasks 1.3 (CatList-paired refactor), 3.1 (rename).

### Task 5.2: Add `to_view` helper and `substitute_free` method to `Free`

- **Reference:** [PureScript's `toView` and `substFree`](../../../purescript-free/src/Control/Monad/Free.purs)
- **Files to modify:**
  - `fp-library/src/types/free.rs`
- **What:** Factor the iterative collapse logic shared by `evaluate` and `resume` into a `to_view` helper, then implement `substitute_free`.

  **PureScript's `toView`** collapses a `Free` into its outermost step:
  ```purescript
  toView :: forall f a. Free f a -> FreeView f a Val
  toView (Free v s) = case v of
      Return a -> case uncons s of
          Nothing  -> Return a
          Just (h, t) -> toView (concatF (h a) t)    -- apply continuation, loop
      Bind f k -> Bind f (\a -> concatF (k a) s)     -- reattach remaining continuations
  ```

  In the CatList-paired Rust representation (Task 1.3), this becomes:
  ```rust
  enum FreeStep<F, A> {
      Done(A),
      Suspended(Apply!(<F as Kind!(...)>::Of<'static, Free<F, A>>)),
  }

  fn to_view(self) -> FreeStep<F, A> {
      let mut current = self;
      loop {
          let Free { view, continuations, .. } = current;
          match view {
              FreeView::Return(val) => match continuations.uncons() {
                  None => return FreeStep::Done(downcast(val)),
                  Some((k, rest)) => { current = concat_free(k(val), rest); }
              },
              FreeView::Suspend(fa) => {
                  // Reattach continuations to the inner Free
                  let typed_fa = F::map(|inner| concat_free(inner, continuations), fa);
                  return FreeStep::Suspended(typed_fa);
              }
          }
      }
  }
  ```

  Both `evaluate` and `resume` then become thin wrappers over `to_view`.

  **PureScript's `substFree`** folds `Free f` into `Free g` without `MonadRec`:
  ```purescript
  substFree :: forall f g. (f ~> Free g) -> Free f ~> Free g
  substFree k = go where
      go f = case toView f of
          Return a -> pure a
          Bind g i -> k g >>= go <<< i
  ```

  Rust equivalent:
  ```rust
  pub fn substitute_free<G: Extract + Functor + 'static>(
      self,
      nt: impl Fn(Apply!(<F as Kind!(...)>::Of<'static, Free<F, A>>))
              -> Free<G, Free<F, A>> + Clone + 'static,
  ) -> Free<G, A>
  ```

  Note: PureScript's `hoistFree` is implemented as `substFree (liftF <<< k)`, so `substitute_free` is the more general operation.

- **Why:** Reduces code duplication between `evaluate` and `resume`. `substitute_free` enables Free-to-Free transformations without the `MonadRec` overhead of `fold_free`.
- **Dependencies:** Task 1.3 (CatList-paired refactor simplifies the view concept).

### Task 5.3: Phase 5 verification

- **What:** Run `fmt -> clippy -> doc -> test`. Verify that `Free` construction works with functors that only implement `Functor` (not `Extract`). Test `substitute_free` with `Free<ThunkBrand, A>` -> `Free<IdentityBrand, A>` transformations.
- **Dependencies:** Tasks 5.1, 5.2.

---

## Phase 6: New Functionality

Adds new operations and combinators. Grouped by area.

### 6A: Trampoline and Free Operations

#### Task 6.1: Add `resume` method to `Trampoline` and `TryTrampoline`

- **Files to modify:**
  - `fp-library/src/types/trampoline.rs`
  - `fp-library/src/types/try_trampoline.rs`
- **What:** Expose `Free::resume` on `Trampoline` as `fn resume(self) -> Result<A, Thunk<'static, Trampoline<A>>>` and analogously on `TryTrampoline`. This decomposes a computation into one step without full evaluation, enabling introspection.
- **Why:** `resume` is a fundamental Free monad operation that is currently available only on `Free` directly.
- **Dependencies:** Task 1.3 (benefits from the CatList-paired refactor).

#### Task 6.2: Make `hoist_free` stack-safe

- **Files to modify:**
  - `fp-library/src/types/free.rs`
- **What:** Replace the recursive `hoist_free` with an iterative implementation using `resume` or an explicit stack.
- **Why:** The current implementation recurses per `Wrap` layer and can overflow on deep chains.
- **Dependencies:** Tasks 1.3, 5.1.

### 6B: MonadRec Combinators

#### Task 6.3: Add derived `MonadRec` combinators

- **Reference:** [PureScript's `Control.Monad.Rec.Class`](../../../purescript-tailrec/src/Control/Monad/Rec/Class.purs)
- **Files to modify:**
  - `fp-library/src/classes/monad_rec.rs` (add free functions)
  - `fp-library/src/functions.rs` (re-export)
- **What:** Implement derived combinators as free functions. MVP:
  - `forever<Brand: MonadRec>(action) -> m b` : run an action indefinitely (stack-safe).
  - `while_some<Brand: MonadRec, A: Monoid>(computation) -> m A` : loop accumulating via Monoid until `None`.
  - `until_some<Brand: MonadRec>(computation) -> m A` : loop until `Some(x)`.
  - `repeat_m<Brand: MonadRec>(n, f, initial) -> m S` : apply step function n times.
  - `while_m<Brand: MonadRec>(condition, body)` : loop while monadic condition holds.
  - `until_m<Brand: MonadRec>(condition, body)` : loop until monadic condition becomes true.

  Follow-up (can be a separate PR):
  - `fold_m` : stack-safe monadic fold over a `Foldable`.
  - `find_m` : search with monadic predicate, short-circuit on match.
  - `any_m` / `all_m` : monadic predicates with short-circuit.
  - `for_each_m` : side-effect iteration discarding results.
  - `scan_m` : collect intermediate fold values.
  - `do_until_m` : at-least-once iteration.

  All are implemented internally via `tail_rec_m`, ensuring stack safety.
- **Why:** Makes `MonadRec` practically useful beyond manual `tail_rec_m` calls. These combinators are standard in PureScript's `purescript-tailrec` and Haskell's `monad-loops`.
- **Dependencies:** Task 1.2 (uses ControlFlow).

### Task 6.4: Phase 6 verification

- **What:** Run `fmt -> clippy -> doc -> test`. Test `resume` on `Trampoline`/`TryTrampoline`. Verify `hoist_free` stack safety at 100k+ iterations. Test `forever` stack safety, `while_some`/`until_some` correctness, and all other new combinators with unit and property tests.
- **Dependencies:** Tasks 6.1 through 6.3.

---

## Phase 7: Minor Improvements (Low Priority)

Small quality-of-life improvements. Tasks 7.1 through 7.4 are independent and parallelizable.

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

### Task 7.4: Relax `Clone` bound on `SendThunk::tail_rec_m`

- **Files to modify:**
  - `fp-library/src/types/send_thunk.rs`
- **What:** `SendThunk::tail_rec_m` uses an iterative loop (not recursion), so the step function is called by reference and does not need `Clone`. Remove the unnecessary `Clone` bound.
- **Why:** Reduces friction for callers without changing behavior.
- **Dependencies:** None.

### Task 7.5: Phase 7 verification

- **What:** Run `fmt -> clippy -> doc -> test`. Add QuickCheck property tests for `SendDeferrable` laws (transparency, idempotence) for `SendThunk`, `ArcLazy`, `TrySendThunk`, `ArcTryLazy`. Add QuickCheck tests verifying functor and monad laws for `SendThunk`'s inherent `map` and `bind`.
- **Dependencies:** Tasks 7.1 through 7.4.

---

## Final Verification

### Task F.1: Run full verification suite

- **What:** Run `fmt -> clippy -> doc -> test` (using the test caching wrapper). Ensure zero warnings from `cargo doc` and zero clippy lints. Verify all doc examples compile and pass.
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

PureScript's `CatList` Functor instance also flattens during `map`: it calls `foldr link CatNil q` to collapse the internal queue before recursing. The Rust implementation's `into_iter().map(f).collect()` achieves the same flattening. The internal tree structure is a transient artifact of O(1) `append`, not something to preserve. Flattening during `map` normalizes the structure, improving performance of all subsequent operations (iteration, `uncons`, `fold`, further `map`s) at no extra asymptotic cost since `map` is already O(n). Structure-preserving `map` would not win in any operation and would add allocation overhead by rebuilding `VecDeque`s at each tree level.

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
| 1 | `classes.rs`, `types/lazy.rs`, `types/try_lazy.rs`, `brands.rs`, `types/step.rs`, `classes/monad_rec.rs`, all MonadRec implementors, `types/free.rs`, `functions.rs` | `classes/lazy_config.rs`, `classes/try_lazy_config.rs` |
| 2 | `types/try_lazy.rs`, `brands.rs`, `types/try_send_thunk.rs`, `types/try_thunk.rs`, `types/cat_list.rs` | None |
| 3 | `classes/evaluable.rs` (renamed to `classes/extract.rs`), `classes/deferrable.rs`, `types/thunk.rs`, `types/free.rs`, `types/identity.rs`, `types/lazy.rs`, `classes.rs`, `functions.rs`, all files importing `Evaluable` | `classes/extend.rs`, `classes/comonad.rs` |
| 4 | `types/try_trampoline.rs` | None |
| 5 | `types/free.rs` | None |
| 6 | `types/trampoline.rs`, `types/try_trampoline.rs`, `types/free.rs`, `classes/monad_rec.rs`, `functions.rs` | None |
| 7 | `types/lazy.rs`, `types/try_lazy.rs`, `types/send_thunk.rs` | None |
