# Remediation Proposals for the Phase 4 Dual-Row Scoped-Effects Design Review

## Summary

The review filed **2 fundamental, 6 major, and 8 minor** findings against the Phase 4 plan section. Clustered by root cause, those eight non-minor findings collapse into **three root-cause clusters**: (i) the interpret-pipeline / scoped-handler-trait gap (F1, M1, M4, M6 share the same root), (ii) the closure-storage encoding for scoped bodies (F2, partially M3), (iii) Rust-specific Val/Ref + lifetime ergonomics (M2, M5). The single highest-leverage remediation is **R1 (Specify the scoped-dispatch surface explicitly, including handler-list interposition)**, which closes one fundamental and three major findings together. **F2's ceiling has no clean fix without changing the substrate** beyond what Phase 3 currently provides; the principled answer is **R2 (parameterise scoped-effect closure storage by `RefCountedPointer`)**, mirroring Phase 3 step 5a's `StateBrand<P, S>` pattern, accepting that on default `Run`/`RunExplicit` bodies remain single-shot. The report recommends **iterative remediation of the existing freer-monad-encoded design**, not a wholesale switch to an HFunctor / Hefty-Algebras encoding; the heftia agent survey confirmed heftia v0.7 is itself freer-monad-encoded (`Eff = D.Eff Freer` at [`heftia/src/Control/Monad/Hefty/Types.hs:32`](https://github.com/sayo-hs/heftia/blob/master/heftia/src/Control/Monad/Hefty/Types.hs#L32)), so the plan's encoding choice is structurally aligned with heftia and the gaps are at the dispatcher level, not the substrate level.

## Fundamental Findings

### F1. The scoped-handler trait shape is unspecified at the load-bearing decision: how does a scoped handler intercept first-order effects inside its stored action?

**Restated:** Phase 4 stores `action: Run<R, S, A>` and recovery closures inside concrete scoped-effect constructors (`Catch`, `Local`, `Bracket`, `Span`), but the plan only says the interpreter has "Method per constructor; fixed `Run<R, A>` continuation" ([plan.md:2015-2017](../../plan.md#L2015-L2017)) without specifying what data the per-constructor method receives. The load-bearing question is whether the method gets a handle to the surrounding first-order handler list and a way to locally rewrite it; without that, the design cannot deliver heftia's `interposeInWith`-style `Catch` semantics that [decisions.md:470-474](../../decisions.md#L470-L474) actively claims over freer-simple's interposition.

**Root cause:** The Phase 4 plan elided the scoped-dispatch trait signature, treating it as an implementation detail. It is not. The trait's input shape determines whether scoped handlers can do interposition, and interposition is exactly what makes higher-order effects more than first-order effects. Heftia's `handleCatch (Catch action hdl) = action & interposeInWith \(Throw e) _ -> hdl e` ([`heftia-effects/src/Control/Monad/Hefty/Except.hs:50-51`](https://github.com/sayo-hs/heftia/blob/master/heftia-effects/src/Control/Monad/Hefty/Except.hs#L50-L51)) only types because `interposeInWith` is a substrate-level primitive on `Eff es a`, not a per-handler convention.

**Option A: Specify a `DispatchScopedHandlers` trait that receives both rows' handler lists; provide handler-list cell substitution machinery.**

- _What:_ Define a parallel trait at `fp-library/src/types/effects/interpreter.rs` (alongside `DispatchHandlers`):
  ```
  trait DispatchScopedHandlers<'a, ScopedLayer, FOLayer, NextProgram> {
      fn dispatch_scoped<FOH>(
          &self,
          layer: ScopedLayer,
          fo_handlers: &FOH,
      ) -> NextProgram
      where FOH: DispatchHandlers<'a, FOLayer, NextProgram>;
  }
  ```
  Each scoped-effect cons-cell impl gets the FO handler list as `fo_handlers`. The Phase 1-3 `DispatchHandlers` trait already takes `&self` (`interpreter.rs:176-179` per the M3C resolution), so cloning the FO handler reference into the scoped dispatcher is cheap. For `Catch` specifically, the cons-cell impl rewrites the FO list at the type level: locate the `ExceptBrand<E>` cell via frunk's `Plucker`, replace its closure with a "throws to a sentinel" clause, recursively call `Run::interpret` on `Catch::action` with the rewritten list, observe the sentinel, and route to `Catch::handler`.
- _Cost:_ Medium. Requires (a) the new trait, (b) one cons-cell impl per scoped-effect type (4 standard + macro-generated for user-defined), (c) handler-list type-level substitution (already supported by frunk's `Plucker` / `CoproductSubsetter` per [variant_f.rs](../../../../../fp-library/src/types/effects/variant_f.rs)), and (d) rewriting all six Run wrappers' `interpret` methods to dispatch on `Node::Scoped` via the new trait (the M1 work). Touches `interpreter.rs`, `node.rs` (no change needed: `Functor`/`SendFunctor`/`WrapDrop` already dispatch on both arms at `node.rs:84-220`), and the six wrapper files. Estimated 800-1500 lines of new code plus per-wrapper test additions. Adds one trait to the public API; does not affect type inference for users (the trait is an internal dispatch surface, like `DispatchHandlers`).
- _Benefit:_ Closes F1. Closes M1 (the new trait IS the Phase 4 interpret-method spec). Closes M4 (scoped handlers are a parallel impl set, not extensions to FO `run_reader` / `run_except`; the plan bullet rewrites to "Phase 4 ships parallel `LocalDispatcher` / `CatchDispatcher` impls"). Closes M6 (the dispatcher exposes the natural narrowing primitive: `interpret_scoped_with::<EBrand>` mirrors `interpret_with::<EBrand>` but on the scoped row). Heftia parity at the semantic level: `Catch` actually intercepts inner `Throw`. Multi-shot recovery handlers do remain blocked by F2's `Box<dyn FnOnce>` choice, but that is F2's problem, not F1's.
- _Risks:_ The handler-list cell-substitution machinery has not been prototyped; whether `Plucker`-style row updates compose cleanly with the existing `HandlersCons<Handler<E, F>, T>` shape needs verification. The existing per-wrapper `interpret` signature pins `S = CNilBrand` (`run.rs:552-568`); generalising to `S != CNilBrand` interacts with the F3A resolution ([resolutions.md:1232](../../resolutions.md#L1232)) that fixed `S = CNilBrand` to make the `Node::Scoped` arm `match cnil {}`. That resolution is reversed here, deliberately. Stable Rust is sufficient.

**Option B: Add a substrate-level `Run::interpose` primitive on each Run wrapper that walks the program tree replacing a chosen effect's dispatches; scoped handlers call it directly.**

- _What:_ Add a public method on each Run wrapper (six implementations) of approximate shape
  ```
  pub fn interpose<EBrand, Idx>(
      self,
      replacement: impl Fn(<EBrand as Kind>::Of<'_, Self>) -> Self + 'static,
  ) -> Self
  where EBrand: Functor + 'static, ... ;
  ```
  that walks the underlying `Free` tree, finds dispatches against `EBrand`, and substitutes `replacement`. This is the literal Rust analogue of heftia's `interposeInWith`. The catch dispatcher then writes:
  ```
  let result = catch.action.interpose::<ExceptBrand<E>, _>(|throw| {
      // Wrap the thrown value in a sentinel Run that the dispatcher
      // observes and routes to catch.handler.
  });
  ```
  Implementation walks the substrate's Free tree analogously to existing `interpret_with` (at `run.rs:885-900`), but does not narrow the row.
- _Cost:_ Larger than A. Six new public methods, each ~50-100 lines, plus per-wrapper tests. The `interpose` primitive is publicly useful (heftia explicitly ships `interposeBy` and `interposeInWith` as user-facing combinators per [`heftia/src/Control/Monad/Hefty/Interpret.hs`](https://github.com/sayo-hs/heftia/blob/master/heftia/src/Control/Monad/Hefty/Interpret.hs)), so it is not throwaway machinery. Estimated 600-900 lines for the primitive plus ~400-600 for the scoped-dispatch glue that uses it. Same six-wrapper surface as Option A. Stack-safety on `interpose` requires the same care as `interpret_with`'s host-stack recursion (`run.rs:947-954`).
- _Benefit:_ Closes F1 the same way A does, but exposes `interpose` as a user-facing primitive. This adds an [`interpose` deferred entry](../../plan.md#L2321-L2337) item from Phase 6+ to the v1 surface as a side effect; users get heftia-equivalent observability/tracing patterns without rolling their own. Composes cleanly with existing handler-list dispatch (the scoped trait's method just calls `interpose` on the action and inspects the result). Closes the same three majors (M1, M4, M6) as A.
- _Risks:_ **POC-validated** (`fp-library/tests/poc_rc_run_interpose.rs`). The concrete one-effect interpose at `poc_rc_run_interpose.rs:74-110` walks an `RcRun` program via `peel`, transforms inner sub-programs through `<EBrand as Functor>::map`, and re-emits via `RcFree::wrap`; the rebuilt program is structurally indistinguishable from the original at the row level and accepts the same handler list (T3 in the POC). Generalising over `<EBrand, Idx, R>` is mechanical from this template since the constraint surface mirrors `interpret_with_shared` line for line (`rc_run.rs:992-1062`); the only delta is the rebuilt layer's row stays in `R` instead of narrowing to `RMinusE`. Remaining uncertainty: scaling to the Explicit family interacts with the HRTB-poisoning workarounds at `arc_run.rs`; the POC exercises only the Erased substrate. Doubles the API surface compared to A but adds a publicly-useful primitive.

**Option C: Adopt a true HFunctor (Hefty Algebras paper) encoding for the scoped row, with the scoped row's elements parameterised by a higher-kinded type constructor that the elaborator can fold over.**

- _What:_ Replace `ScopedCoproduct<S>` ([decisions.md:599-608](../../decisions.md#L599-L608)) with a row of HFunctor-shaped types where each scoped op carries the body as a structural higher-kinded data slot, e.g., `Catch<m> = Catch (m a) (e -> m a)` with `m` instantiated to `Run<R, S, ?>` only at fold time. Adds a parallel "Hefty" substrate at `fp-library/src/types/effects/hefty.rs` analogous to the existing `Free` family, with hand-written `hmap` (HFunctor map over the program type constructor) operations.
- _Cost:_ x-large. New substrate parallel to the six-variant Free family ([decisions.md:597-614](../../decisions.md#L597-L614)). Touches every Phase 1-3 substrate file: `node.rs` becomes a sum of `First | Hefty` instead of `First | Scoped`; the six `Run` wrappers add hefty-aware `peel` variants; the brand surface needs HFunctor brands paralleling `Functor`/`SendFunctor`. Estimated > 4000 lines of new substrate plus migration of the four standard scoped ops. Brand-pattern integration is unproven: the HFunctor version of `Run<R, S, A>` parameterises every constructor by `Run<R, S, ?>` itself, which creates a recursive type that the existing `impl_kind!` macro and `Apply!` mechanism may not support without extension.
- _Benefit:_ Closes F1 and F2 holistically. Multi-shot bodies become natural (the body is a structural sub-program, not a `FnOnce` closure). User-defined scoped effects (Coroutine, Provider, scoped NonDet with backtracking) become writable. Aligns with the original Hefty Algebras paper. Resolves M3 (resource safety) trivially: the body is data, so the elaborator can wrap it in a Drop-guard.
- _Risks:_ Heftia v0.7 itself **moved away** from HFunctor to a freer-monad encoding (`Eff = D.Eff Freer` per [`heftia/src/Control/Monad/Hefty/Types.hs:32`](https://github.com/sayo-hs/heftia/blob/master/heftia/src/Control/Monad/Hefty/Types.hs#L32)); adopting HFunctor would diverge from the library that sources the "heftia-inspired" naming. The paper-form HFunctor encoding has known compile-time and inference costs that heftia's authors found prohibitive in practice. Rust's lack of higher-kinded types makes the Brand-pattern hosting of HFunctor structurally messy (each scoped op's brand would need a per-row-shape parameterisation). Reproducing a design heftia tried and abandoned.

**Recommendation: Option B.** Three reasons: (1) **Empirically validated**: POC at `fp-library/tests/poc_rc_run_interpose.rs` demonstrates the substrate primitives suffice on `RcRun` for a one-effect concrete row; the generic shape is a mechanical lift from `interpret_with_shared`. Option A's handler-list-Plucker substitution mechanism remains unprototyped; choosing it now would mean preferring the unvalidated path. (2) **Heftia-faithful**: heftia's `interposeInWith` IS Option B's mechanism (walk the `Eff` value, rewrite matched effects per [`heftia-effects/src/Control/Monad/Hefty/Except.hs:50-51`](https://github.com/sayo-hs/heftia/blob/master/heftia-effects/src/Control/Monad/Hefty/Except.hs#L50-L51)); Option A's handler-list-cell substitution is a Rust-flavoured workaround. (3) **Closes a Phase 6+ deferred item**: the existing [`interpose` family entry at plan.md:2321-2337](../../plan.md#L2321-L2337) defers a public `interpose` primitive pending an observability/tracing use case; shipping `Run::interpose` as part of Phase 4's scoped-handler dispatcher closes that deferral as a side effect. The recommendation overturns part of [resolutions.md F3A:1232](../../resolutions.md#L1232) (the `S = CNilBrand` tightening on the interpret family); that resolution was correct for Phase 3 closure but assumed Phase 4 would specify a parallel mechanism, which this option now does. Option A remains a viable fallback if Option B's per-wrapper substrate addition surfaces an obstacle the POC did not catch (likely on the Explicit family's HRTB-poisoning workarounds), but the POC's clean compile across stable Rust suggests no such obstacle. Option C (HFunctor) remains rejected for the reasons given above.

**Dependencies and ordering:** Must precede the rest of Phase 4. F2's remediation (R2 below) is independent and can land in parallel. M3's panic-safety remediation depends on this (the bracket dispatcher in the new trait shape is where the Drop-guard would live).

**Plan revision required:** Yes (large). [plan.md Phase 4 step 3](../../plan.md#L2015-L2017) must expand from one bullet to a full subsection specifying the `DispatchScopedHandlers` trait shape, its required cons-cell impls, and the substrate-level `Run::interpose` primitive each cons-cell impl uses. [decisions.md section 4.5](../../decisions.md#L436-L573) line 481's "fixed `Run<R, A>`" must become "fixed `Run<R, S, A>` (the scoped row stays in scope so that dispatchers can recurse on actions that themselves contain scoped operations)". [resolutions.md F3A](../../resolutions.md#L1232) needs an addendum noting the `S = CNilBrand` bound on the Phase 3 interpret family is lifted in Phase 4 and the new trait subsumes it. The Phase 6+ deferred [`interpose` family entry at plan.md:2321-2337](../../plan.md#L2321-L2337) should be removed (or rewritten to point at the now-shipped Phase 4 primitive) since `Run::interpose` ships as part of Phase 4's substrate.

### F2. `Box<dyn FnOnce>` body closures plus default `Run`/`RunExplicit`'s non-`Clone` action structurally preclude any user-defined scoped effect that needs to run the body or the action more than once.

**Restated:** The plan locks every scoped-effect constructor's closure slot to `Box<dyn FnOnce>` ([plan.md:1982-2014](../../plan.md#L1982-L2014)) and stores actions as `Run<R, S, A>` values. On default `Run`/`RunExplicit`, the action is non-`Clone`, so even structurally re-runnable scoped semantics are unavailable. On `RcRun`/`ArcRun` the action is `Clone` but recovery / body closures are still `FnOnce`. Heftia's [`Example/Continuation/Main.hs:38-41`](https://github.com/sayo-hs/heftia/blob/master/heftia-effects/Example/Continuation/Main.hs#L38-L41) shows `mapM resume [1 .. numberOfFork]` invoking `resume` N times in a user-defined scoped effect; the plan's encoding cannot host that shape.

**Root cause:** The plan picked a single closure-storage shape (`Box<dyn FnOnce>`) for every scoped-effect constructor without parameterising over the underlying Run wrapper's pointer kind. Phase 3 step 5a explicitly solved the analogous problem for first-order State by introducing [`StateBrand<P, S>` parameterised by `P: RefCountedPointer`](../../../../../fp-library/src/brands/effects.rs) and shipping per-wrapper smart constructors. The Phase 4 plan does not apply that pattern; the scoped-effect constructors are fixed-shape, not pointer-kind-parameterised.

**Option A: Parameterise every scoped-effect constructor by `P: RefCountedPointer`, mirroring the Phase 3 `StateBrand<P, S>` pattern.**

- _What:_ Convert each scoped-effect type from `Catch<'a, E>` to `Catch<'a, P, E>` with `handler: <P as RefCountedPointer>::Of<'a, dyn Fn(E) -> Run<R, S, A>>`. Same change for `Local<'a, P, E>`, `Bracket<'a, P, A, B>`, and the existing `RefBracket<'a, P, A, B>` (which then absorbs the role of distinguishing pointer kinds; the Val flavour disappears or unifies). The smart constructors `bracket(...)` and `local(...)` dispatch on closure type to pick `P = BoxBrand` (single-shot) or `P = RcBrand` / `ArcBrand` (multi-shot), reusing the existing `Val`/`Ref<P>` markers. Phase 3 step 5e's [`RcCatList`](../../../../../fp-library/src/types/rc_cat_list.rs) and [`ArcCatList`](../../../../../fp-library/src/types/arc_cat_list.rs) already provide the substrate's multi-shot continuation queue. Touches each scoped-effect file (4 standard + macro), the `define_scoped_effect!` macro, and the dispatch traits at [`fp-library/src/dispatch/`](../../../../../fp-library/src/dispatch/). Adds one new pointer brand `BoxBrand` (analogous to `RcBrand`/`ArcBrand`) for the single-shot case.
- _Cost:_ Medium. Closely mirrors Phase 3 step 5a's effort budget per [plan.md commit log](../../plan.md#L86-L107) (ratified `SendStateBrand<P, S>` plus per-wrapper smart constructors). Estimated 1500-2500 lines across the four standard scoped ops plus dispatch traits. Type inference: identical to Phase 3 State (the closure-shape-driven dispatch resolves `P` automatically). Macro work: `define_scoped_effect!` needs an optional `pointer = ...` parameter or auto-detection from the closure shape.
- _Benefit:_ Closes F2 the same way Phase 3 closed it for State. Multi-shot bodies on `RcRun`/`ArcRun` become possible (`Rc<dyn Fn>` is multi-shot at the cell level). User-defined Coroutine, Provider, Retry effects become writable on the multi-shot wrappers. Default `Run`/`RunExplicit` users keep the single-shot ceiling, which is the encoding's honest answer for non-refcounted substrates. Composes cleanly with R1: the F1 fix references the closure storage but does not depend on its specific shape.
- _Risks:_ **POC-validated** ([`fp-library/tests/poc_send_catch_brand.rs`](../../../../../fp-library/tests/poc_send_catch_brand.rs)). The parallel-Send-brand pattern carries to scoped-effect constructors with `Box<dyn FnOnce>` continuations as cleanly as it carried to State's `Rc<dyn Fn>` continuations: the static `assert_send_sync::<SendCatch<'static, String, i32>>()` in the POC compiles, confirming the Send + Sync auto-traits propagate through the dyn bound when baked in at definition time. No new HRTB-over-types friction surfaces beyond what Phase 3 documented. The Phase 4 rollout to `SendLocalBrand` / `SendBracketBrand` / `SendSpanBrand` follows the same pattern with no expected per-effect novelty. Phase 4 will still need new rows in [`limitations-and-workarounds.md`](../../../../../fp-library/docs/limitations-and-workarounds.md) recording the parallel-brand requirement, but the cost is documentation-mechanical rather than design-discovery.

**Option B: Type-aligned queue continuations for scoped closures, mirroring heftia's FTCQueue.**

- _What:_ Replace the single `Box<dyn FnOnce>` closure cell with a queue of cells. Each scoped operation can store a queue of pending "post-body" continuations, allowing left-bind fusion and natural multi-shot via queue traversal. Heftia's [`Data/FTCQueue.hs:38-40`](https://github.com/sayo-hs/heftia/blob/master/heftia/src/Data/FTCQueue.hs#L38-L40) defines the type-aligned queue exactly:
  ```
  data FTCQueue m a b where
      Leaf :: (a -> m b) -> FTCQueue m a b
      Node :: FTCQueue m a x -> FTCQueue m x b -> FTCQueue m a b
  ```
- _Cost:_ Larger than A. Net-new substrate type per Run wrapper (BoxFTCQueue, RcFTCQueue, ArcFTCQueue). Each scoped-op constructor changes shape to carry the queue. The bind machinery on every Run wrapper must extend the queue rather than allocating a fresh closure. Estimated 2500-4000 lines plus a new test surface paralleling the [Free / RcFree / ArcFree](../../../../../fp-library/src/types/) trio. Type-inference impact: small (queue type is opaque to the user in the same way `dyn Fn` is today).
- _Benefit:_ Closes F2 and a chunk of M3 in one stroke (queue-based continuations let release run after body via queue discipline rather than after-the-fact, simplifying panic safety). Heftia-faithful at the substrate level, not just the constructor level. Inherits heftia's amortised-O(1) bind cost for scoped ops.
- _Risks:_ The substrate's existing [`RcCatList`/`ArcCatList`](../../../../../fp-library/src/types/rc_cat_list.rs) already provides this for the FO Free spine, but adapting the same pattern to per-scoped-constructor closure queues requires either reusing those types (potentially) or introducing parallel structures. Substantial design work to validate the chosen variant. Larger than A while overlapping with the substrate the project already shipped in step 5e.

**Option C: Document multi-shot bodies as out of scope for v1; ship `Box<dyn FnOnce>` for the standard set; explicitly define which scoped effects users can and cannot define via `define_scoped_effect!`.**

- _What:_ Add a "Scoped-effect closure storage" subsection to [decisions.md section 4.5](../../decisions.md#L436-L573) and a corresponding [plan.md "Out of scope"](../../plan.md#L1287-L1304) entry stating that v1's scoped-effect surface ships `Box<dyn FnOnce>` everywhere and that user-defined effects requiring multi-shot bodies (Coroutine, Provider, Retry, scoped NonDet with backtracking) are deferred to a future phase. The `define_scoped_effect!` macro accepts only `FnOnce`-typed closure slots in v1.
- _Cost:_ Negligible (documentation only).
- _Benefit:_ Honest with users: the encoding ceiling is named, not silently inherited. Closes the rubric Section 6.2 #8 expectation gap by acknowledging the limitation.
- _Risks:_ Delays closing F2 indefinitely. The first user who wants Coroutine has to either fork the library or accept their effect cannot exist. Defers the principled answer.

**Recommendation: Option A.** It closes F2 within the encoding the project has already chosen and uses a pattern Phase 3 has already validated for State. The cost is comparable to Phase 3 step 5a (which the project completed in roughly the equivalent budget). Option B is the more aggressive substrate answer; it is a viable v2 path but does not buy enough over A to justify the doubled implementation cost in v1 given the standard scoped set is single-shot already. Option C is the smallest option and exactly the kind of "papering over a fundamental issue" the user has explicitly ruled out for Phase 4. The recommendation does not overturn any prior decision; [decisions.md:468](../../decisions.md#L468) anticipates "exact payload shapes depend on the chosen Free variant", which is the parameterisation A introduces.

**Dependencies and ordering:** Independent of F1's remediation; the two can land concurrently. Should be sequenced after the Phase 3 [m1-m9 polish commit](../../plan.md#L25) so the scoped pattern can mirror the FO pattern's final shape. Adds rows to [`limitations-and-workarounds.md`](../../../../../fp-library/docs/limitations-and-workarounds.md) as the per-pointer impls land.

**Plan revision required:** Yes (medium). [plan.md Phase 4 step 2](../../plan.md#L1975-L2014) must rewrite each constructor's signature to reflect the `P: RefCountedPointer` parameter; [decisions.md:464-465](../../decisions.md#L464-L465) Bracket / RefBracket payload entries collapse into one parameterised entry. Add a `BoxBrand` registration pointer brand for Box-based wrappers if one does not already exist.

## Major Findings

### M1. Phase 4 must rewrite all six Run wrappers' interpret methods to lift the `S = CNilBrand` invariant, but the plan does not sketch the new public signature.

**Restated:** The Phase 3 [F3A resolution](../../resolutions.md#L1232) tightened every wrapper's `interpret` to `Run<R, CNilBrand, A>` so the `Node::Scoped` arm closes via `match cnil {}`. Phase 4 must reverse that on each wrapper. The plan does not specify how (parallel method, generalised method, breaking change).

**Root cause:** Plan-completeness gap; subsumed by F1's R1.

**Option A: Generalise each wrapper's `interpret` to take both an FO handler list and a scoped handler list; remove the `S = CNilBrand` bound.**

- _What:_ Grow each wrapper's `interpret` signature from one handler-list parameter to two. The `Node::First` arm dispatches via the existing `DispatchHandlers`; the `Node::Scoped` arm dispatches via the F1-Option-A `DispatchScopedHandlers` trait. Existing call sites that pass only an FO handler list need a `HandlersNil` placeholder for the scoped slot (or the macro emits one).
- _Cost:_ Medium. Touches all six wrappers' `interpret`, `interpret_rec`, `interpret_with`, `run`, `run_rec`, `extract` signatures. Phase 1-3 has not shipped a public release ([plan.md API stability stance](../../plan.md#L1011)) so the breaking change is in scope. ~600 lines across the wrappers plus doctest updates.
- _Benefit:_ One unified interpret family across both rows. Closes M1. Composes with R1 (F1 fix).
- _Risks:_ Type inference cost: every doctest and example today passes a single `handlers!{...}` list; the new shape requires a two-list call, which the macro can sugar (`handlers!{ FO: ..., Scoped: ... }`) but the doctests need rewriting.

**Option B: Keep the existing `interpret` for `S = CNilBrand` programs; add a parallel `interpret_dual` taking both lists.**

- _What:_ New methods on each wrapper sit beside the existing ones; the existing names stay a single-list `S = CNilBrand`-only shortcut.
- _Cost:_ Smaller than A but doubles the public method surface per wrapper (now 12 methods per wrapper instead of 6 for the interpret family).
- _Benefit:_ No call-site migration for users who never use scoped effects.
- _Risks:_ Two methods doing the same thing for the degenerate case is a strict regression in API clarity; the prior remediation report's [F1 Option D](../0_first_order_effects_implementation/remediation_proposals.md#L111-L137) made this argument against `run_accum` aliasing `interpret`. Reproducing the rejected pattern.

**Recommendation: Option A.** Phase 4 hasn't shipped; no migration to preserve. One method per role, parameterised over both rows. The macro update to `handlers!{...}` is small and absorbs the call-site cost. Subsumed by F1-R1; this finding's recommendation is just "the F1 fix's interpret-method rewrite is intentional and breaking".

**Dependencies and ordering:** Lands as part of F1-R1.

**Plan revision required:** Yes (medium). [plan.md Phase 4](../../plan.md#L1970-L2052) gets a new step (call it 1a) sketching the new interpret signature; the existing step 1 stays as the dual-row coproduct definition.

### M2. The Val/Ref split for `Bracket` and `Local` makes the scoped row carry up to three constructors per logical effect; mixing flavours in one program is a 2x-3x row tax not analysed.

**Restated:** A program using both Val and Ref `bracket` plus both flavours of `local` declares four scoped-row entries instead of two, plus per-pointer-brand splits on `RefBracket<P>`. The plan frames this as "uniform with the existing Val/Ref dispatch pattern" ([decisions.md:483-499](../../decisions.md#L483-L499)) but the existing dispatch pattern routes one trait through two impls of one type; here the row gets two distinct types per logical effect.

**Root cause:** The Val/Ref split was promoted from "ergonomic improvement" to "axis of the row" without analysis of the row-cost. Subsumed by F2-R2 (parameterising scoped constructors by `P: RefCountedPointer` collapses Val/Ref/Pointer-brand into a single axis).

**Option A: Ship Val flavour only for v1 (i.e., as a side-effect of F2-R2 with `P = BoxBrand`); defer Ref to a Phase 5+ promotion when concrete demand surfaces.**

- _What:_ Phase 4 ships only the `P = BoxBrand` case for `Bracket` and `Local`. The plan's [Phase 6+ deferred items](../../plan.md#L2199-L2403) gain entries for "RefBracket / RefLocal" with revisit triggers parallel to the existing `State::modify` Val/Ref split deferral at [plan.md:2218-2230](../../plan.md#L2218-L2230). [decisions.md:483-499](../../decisions.md#L483-L499) gets a "v1 ships Val only; Ref deferred" amendment.
- _Cost:_ Negligible (deletion / non-implementation).
- _Benefit:_ Halves the Phase 4 surface. Closes M2 by removing the row tax. Aligns with the `State::modify` Val/Ref deferral pattern already established. The decisions doc admits at [line 487](../../decisions.md#L487) that PureScript GC-aliases the resource between body and release; Rust users pay one allocation on `RcRun`/`ArcRun` regardless and are not under-served by the Val flavour.
- _Risks:_ Loses the worked-example use case at [decisions.md:501-519](../../decisions.md#L501-L519) (capturing a file handle freely across nested binds). Users who hit that wall must use `Rc<File>` manually or wait. The trigger ([Phase 6+ entries](../../plan.md#L2199-L2403)) keeps the path open.

**Option B: Ship Val and Ref unified via F2-R2's `P` parameterisation, eliminating the Val/Ref dispatch trait split.**

- _What:_ `Bracket<'a, P, A, B>` covers all three cases (`P = BoxBrand` is the Val flavour; `P = RcBrand` / `ArcBrand` are the Ref flavours). The smart constructor `bracket(...)` dispatches on closure shape exactly as today, but the resulting node type is uniform. No extra row entries.
- _Cost:_ Same as F2-R2.
- _Benefit:_ Closes M2 by definition: the row never carries more than one Bracket entry per logical effect.
- _Risks:_ Higher initial Phase 4 surface than A. But the unification reduces total lines compared to "Val + Ref + RefBracket<P>" as separate types.

**Recommendation: Option B (subsumed by F2-R2).** A is a defensible scope cut, but B is the principled answer once F2-R2 lands. The `P` parameterisation IS the unification; M2 disappears as a side effect.

**Dependencies and ordering:** Lands with F2-R2.

**Plan revision required:** Yes (small). [decisions.md Bracket Val/Ref subsection](../../decisions.md#L485-L499) and [plan.md Phase 4 step 2 Bracket entries](../../plan.md#L1992-L2012) consolidate from two constructors plus a pointer-brand split into one parameterised constructor.

### M3. `Box<dyn FnOnce>` for `release` plus the absence of any panic-safety statement means resource cleanup is not guaranteed if `body` panics.

**Restated:** If `body` panics during interpretation, the post-body step that moves the `A` into `release` never runs. `release` lives in the scoped-effect node and is dropped (without invocation) during unwind. Heftia delegates to GHC's `mask`/`onException`; the plan inherits Rust's drop-based model but does not specify a guard.

**Root cause:** Rust unwind semantics and the plan's separation of `body` and `release` into independent closures.

**Option A: Wrap the resource in a Drop-implementing guard so `release` runs in `Drop` regardless of how `body` exits.**

- _What:_ The bracket dispatcher (in F1-R1's new trait) constructs a guard struct holding the resource `A` and the `release` closure. The guard's `Drop` impl invokes `release` with the resource. The dispatcher hands the guard to `body`, observes `body`'s return, and drops the guard normally on success or via unwind on panic. Touches the bracket dispatcher in `interpreter.rs` and adds a `BracketGuard<A, F>` type at `fp-library/src/types/effects/bracket.rs`.
- _Cost:_ Small. ~150 lines plus tests covering panic-during-body and panic-during-release. Tests use `std::panic::catch_unwind` at the test boundary only, not in production code.
- _Benefit:_ Honest panic safety. Resource leaks bounded by the resource's own Drop semantics, which is the Rust convention.
- _Risks:_ `release` inside `Drop` cannot return a `Run<R, S, ()>` for the interpreter to schedule; it must execute synchronously. This means `release` cannot itself perform effects in the row; on panic, side-effectful release becomes a problem (the interpreter is not running). For most resources (file handles, sockets, mutex guards) the Rust-native synchronous-drop model is the right one anyway. The plan should explicitly document that `release` is invoked synchronously on panic; effectful release is best-effort.

**Option B: Wrap body invocation in `std::panic::catch_unwind`.**

- _What:_ The dispatcher catches panics from `body`, runs `release`, then resumes the panic.
- _Cost:_ Small. Adds an `UnwindSafe` bound to the body closure (or uses `AssertUnwindSafe` and accepts the soundness implication).
- _Benefit:_ Same as A.
- _Risks:_ `catch_unwind` is heavyweight and pollutes the type signature with `UnwindSafe`. The Rust convention for resource cleanup is Drop, not catch_unwind; this option swims against the grain.

**Option C: Document bracket as panic-leaky; users who care about panics use Rust's RAII or wrap their own guard.**

- _What:_ A "Panic safety" subsection in the rustdoc for `bracket`'s smart constructor stating that `release` is not run if `body` panics.
- _Cost:_ Negligible.
- _Benefit:_ Honest.
- _Risks:_ Loses a rubric Section 6.2 #9 capability the plan implicitly claims.

**Recommendation: Option A.** Drop-guard is the Rust-conventional answer and aligns with how every resource type in `std` (`File`, `MutexGuard`, `Box`) handles cleanup on unwind. The synchronous-drop limitation is a real but documented constraint; users wanting fully-effectful release on panic should layer their own Drop-impl on top, which the encoding does not preclude.

**Dependencies and ordering:** Lands after F1-R1 (the bracket dispatcher is where the guard goes). Independent of F2-R2.

**Plan revision required:** Yes (small). [plan.md Bracket entries](../../plan.md#L1992-L2012) gain a "Panic safety: release runs in Drop" sentence; [decisions.md:485-489](../../decisions.md#L485-L489) gains a corresponding subsection.

### M4. Bullet 6 ("Standard handlers ... wired through the dual row") obscures whether existing FO handlers gain scoped clauses or whether parallel scoped handlers are introduced; the two are structurally different.

**Restated:** [plan.md:2046-2048](../../plan.md#L2046-L2048) reads as if `run_reader` (an FO handler) gains a `local` clause, but [decisions.md:455](../../decisions.md#L455) commits to "one method per variant" in a separate trait. Two structurally different things.

**Root cause:** Plan-text imprecision. Subsumed by F1-R1.

**Option A: Rewrite bullet 6 to specify parallel scoped-handler implementations; introduce a naming convention.**

- _What:_ "Phase 4 ships `LocalDispatcher`, `RefLocalDispatcher`, `CatchDispatcher`, `BracketDispatcher`, `SpanDispatcher` impls (or analogous trait names) as parallel `DispatchScopedHandlers` cons-cells. They are independent of the FO `run_reader` / `run_except` handlers; both are passed to `interpret` together via the unified two-list form (per F1-R1)." Add a one-paragraph subsection clarifying the FO and scoped handlers may share state via interior-mutability captures (the same convention as Phase 3 closure-capture state-threading) but do not share types.
- _Cost:_ Negligible (plan-text edit).
- _Benefit:_ Closes M4.
- _Risks:_ None.

**Recommendation: Option A.** Trivial.

**Dependencies and ordering:** Lands with F1-R1's plan revision.

**Plan revision required:** Yes (small).

### M5. The `'a` lifetime parameter on every scoped constructor (`Catch<'a, E>`, `Local<'a, E>`, etc.) is structural cost on Erased-family wrappers where it has no observable role.

**Restated:** `'a` parameterises every constructor uniformly. On Erased wrappers `'a = 'static` always; the parameter is signature noise.

**Root cause:** [decisions.md:480](../../decisions.md#L480) chose uniformity over per-family minimalism.

**Option A: Keep the uniform `'a` parameter; document the Erased-family `'static` collapse.**

- _What:_ Add a sentence at the relevant decision sub-bullet noting that on Erased wrappers the `'a` parameter is always `'static` and that the parameter is retained for cross-family uniformity with Explicit wrappers.
- _Cost:_ Negligible.
- _Benefit:_ Clarifies a non-obvious detail; closes M5 with documentation rather than redesign.

**Option B: Drop `'a` from constructor types when used on Erased family; keep on Explicit only.**

- _What:_ Two constructor variants per logical effect; one `'static`-only for Erased, one `'a`-parameterised for Explicit. Doubles the type surface and creates an asymmetry in the standard scoped set.
- _Cost:_ Larger than A; doubles the type surface for the four standard scoped ops.
- _Risks:_ The asymmetry violates the [decisions.md:480](../../decisions.md#L480) commitment to uniformity; maintainability cost.

**Recommendation: Option A.** The documentation cost is trivial; the type-surface doubling in B is not worth the marginal saving.

**Dependencies and ordering:** Standalone.

**Plan revision required:** Yes (small).

### M6. Interaction with `interpret_with` (row-narrowing) is unspecified for the scoped row.

**Restated:** Phase 3's `interpret_with::<EBrand>` narrows one FO effect at a time. Phase 4 needs an equivalent for scoped effects, plus an ordering rule for mixed pipelines.

**Root cause:** Plan-completeness gap; subsumed by F1-R1.

**Option A: Add `interpret_scoped_with::<ScopedEBrand>` paralleling `interpret_with`; pipeline ordering is "narrow whichever row the user names, in any order".**

- _What:_ New per-wrapper method that narrows one scoped-effect entry, returning a `Run<R, SMinusE, A>`. The ordering is user-driven; the plan documents that scoped handlers requiring FO effects (e.g., a `Local` impl needing `Reader::Ask`) must run while their target FO effect is still in the row.
- _Cost:_ Medium. Six new per-wrapper methods plus tests, paralleling `interpret_with`'s shape on the scoped row.
- _Benefit:_ Closes M6. Composes with F1-R1.
- _Risks:_ User foot-gun: narrowing FO before scoped or vice versa silently produces handler-not-found errors at the type level, which the user must read carefully. Mitigation: doctests that show common pipeline orderings.

**Option B: Force scoped narrowing first, then FO; encode the ordering at the type level.**

- _What:_ Provide only `interpret_scoped_with::<ScopedEBrand>` returning a `Run<R, SMinusE, A>`; once `S = CNilBrand`, the existing FO `interpret_with` becomes available. Encode via where-bounds.
- _Cost:_ Smaller surface than A but loses flexibility.
- _Risks:_ Some scoped handlers genuinely need FO effects to remain in scope (Local needs Reader::Ask in scope when interpreted); forcing scoped-first breaks them.

**Recommendation: Option A.** Heftia's ordering is also user-driven; users who need `Local` while `Reader` is still in the row write the handler that way. Don't over-encode.

**Dependencies and ordering:** Lands with F1-R1.

**Plan revision required:** Yes (small).

## Minor Findings

- **m1.** Decisions.md `Run<R, A>` vs plan.md `Run<R, S, A>` inconsistency. Fix: amend [decisions.md:481](../../decisions.md#L481) to read `Run<R, S, A>` so the trait return type matches the constructor types in [plan.md:1982-2014](../../plan.md#L1982-L2014).
- **m2.** Span as scoped is over-specified. Fix: keep Span in the scoped row for uniform dispatch, but note in [decisions.md:495](../../decisions.md#L495) that Span has no closure semantics and the dispatcher is a one-line passthrough. Alternative (small breaking change): move Span to the FO row as a Coyoneda-trivial effect; not recommended because the interpreter would not know to skip the body.
- **m3.** Lexical sort of FO and scoped brand identifiers in the macro. Fix: amend [plan.md Phase 4 step 4](../../plan.md#L2018-L2021) to specify that `effects!` and `scoped_effects!` sort within their own namespaces; cross-namespace collisions are user-resolved (rename one).
- **m4.** "Uniform with Val/Ref dispatch" framing. Fix: subsumed by M2-Option-B (the unified `P` parameterisation).
- **m5.** Bracket Val flavour's `(A, B)` thread-back is unergonomic. Fix: subsumed by F2-R2-Option-A's unification (the `P` parameterisation makes `RcRun`/`ArcRun` callers skip thread-back; default `Run` callers still pay it but only on the non-refcounted path where there is no alternative).
- **m6.** Heftia framing acknowledged in resolutions.md but not in plan.md / decisions.md. Fix: lift the [resolutions.md:1893-1906](../../resolutions.md#L1893-L1906) row-encoding clarification into a one-paragraph note at [decisions.md section 4.5](../../decisions.md#L436) and at [plan.md Phase 4 header](../../plan.md#L1970), so readers of either doc see the divergence note inline.
- **m7.** `Node::Scoped` arm ergonomics during Phase 4 development. Fix: not actionable; Phase 4's first commit lands the F1-R1 change set, after which the arm has a real branch.
- **m8.** Phase 4 step 7 `compile_fail` test enumeration. Fix: amend [plan.md step 7](../../plan.md#L2049-L2052) to enumerate at minimum: scoped operation in an FO-only row, mismatched body / release closure shapes for Bracket dispatch, `RefBracket` with a non-`RefCountedPointer` `P`, scoped handler-list omission of a row entry. Mirrors the [Phase 3 step 7](../../plan.md#L31) `compile_fail` enumeration shape.

## Root Cause Clusters

**Cluster 1 (Interpret-pipeline / scoped-dispatch surface, four findings).** F1, M1, M4, M6 share the same root: Phase 4 elided the dispatcher specification. F1's R1 is the single change that closes all four. This is the highest-leverage remediation in the report. **Recommend: prioritise F1-R1 ahead of all other Phase 4 work.**

**Cluster 2 (Closure-storage encoding, two findings).** F2 and M3 (partially) share root: scoped-effect closures are fixed-shape `Box<dyn FnOnce>` regardless of the underlying Run wrapper's pointer kind. F2-R2 (parameterise by `P: RefCountedPointer`) closes F2 and is independent of M3's drop-guard. M3 is independent and also closes via M3-Option-A.

**Cluster 3 (Rust-specific cost framing, two findings).** M2 and M5 are about the cost of Rust-flavoured choices on the scoped surface. M2 collapses into F2-R2's unification. M5 is documentation-only.

**Cluster 4 (Plan-text imprecision, several minors).** m1, m3, m4, m6, m8 are plan-text edits without semantic implications. They land as a single docs-only commit per the doc-only-verify convention (run `just fmt && just doc`, skip the full test suite).

## Holistic Redesigns

### R1. Specify the scoped-dispatch surface explicitly: parallel `DispatchScopedHandlers` trait, two-list `interpret`, substrate-level `Run::interpose` primitive.

**Sketch:** Two pieces; the first is substrate, the second is dispatch.

**Piece 1: `Run::interpose` substrate primitive on each Run wrapper.** Mirrors heftia's `interposeInWith`. Approximate shape:

```
pub fn interpose<EBrand, Idx>(
    self,
    replacement: impl Fn(<EBrand as Kind>::Of<'_, Self>) -> Self + 'static,
) -> Self
where ...
```

Walks the underlying `Free` tree, finds dispatches against `EBrand`, applies `replacement`, and re-emits in the same row. Validated by `poc_rc_run_interpose.rs:74-110` on `RcRun` for a one-effect concrete row; the implementation pattern is identical to `interpret_with_shared` with the row staying in `R` instead of narrowing to `RMinusE`.

**Piece 2: `DispatchScopedHandlers` trait at `fp-library/src/types/effects/interpreter.rs` parallel to `DispatchHandlers`.**

```
pub trait DispatchScopedHandlers<'a, ScopedLayer, FOLayer, NextProgram> {
    fn dispatch_scoped<FOH: DispatchHandlers<'a, FOLayer, NextProgram>>(
        &self,
        layer: ScopedLayer,
        fo_handlers: &FOH,
    ) -> NextProgram;
}
```

For `Catch`, the cons-cell impl calls `Catch::action.interpose::<ExceptBrand<E>, _>(|throw_op| /* re-emit as catch sentinel */)`, observes the sentinel via the dispatcher's outer `interpret` loop, and routes to `Catch::handler`. The `fo_handlers` argument is threaded through so re-interpretation of `Catch::action` and `Catch::handler`'s recovery program use the same FO handler set.

Each Run wrapper's `interpret` grows a second handler-list parameter; the loop dispatches `Node::First` to the existing `DispatchHandlers::dispatch` and `Node::Scoped` to the new `DispatchScopedHandlers::dispatch_scoped`, passing the FO list along.

**Phase 1-3 invariants preserved:** `Node` enum shape; `Functor`/`SendFunctor`/`WrapDrop`/`RefFunctor`/`Extract` impls on `NodeBrand<R, S>` (`node.rs:84-449`); the six-variant Free family substrate; per-effect `Coyoneda`-wrapping pattern.

**Phase 1-3 invariants broken:** Each wrapper's `interpret` signature breaks (the F3A `S = CNilBrand` tightening is reversed). `handlers!{...}` macro grows a sister `scoped_handlers!{...}` and a combiner. The two-list form is breaking for any caller whose code is already on `git main`; since no public release has shipped per [plan.md API stability stance](../../plan.md#L1011), this is in scope.

**Findings resolved:** F1, M1, M4, M6.

**Phase touch:** Phase 4 only on the trait + cons-cell impls; six-wrapper `interpret` rewrites touch Phase 1-3-shipped code at `run.rs:552-568` (and equivalent in `run_explicit.rs`, `rc_run.rs`, `arc_run.rs`, `rc_run_explicit.rs`, `arc_run_explicit.rs`).

### R2. Parameterise scoped-effect closure storage by `P: RefCountedPointer`, mirroring Phase 3 `StateBrand<P, S>`.

**Sketch:** Each scoped-effect type from `Catch<'a, P, E>` to `Bracket<'a, P, A, B>` carries `P` and stores closures as `<P as RefCountedPointer>::Of<'a, dyn Fn(...) -> Run<R, S, A>>`. Smart constructors dispatch on closure shape via `Val<BoxBrand>` / `Val<RcBrand>` / `Val<ArcBrand>` markers; the per-pointer-brand impl picks the right `P`. The substrate already provides [`RcCatList`](../../../../../fp-library/src/types/rc_cat_list.rs) / [`ArcCatList`](../../../../../fp-library/src/types/arc_cat_list.rs) for multi-shot continuation queues, which scoped operations on `RcRun`/`ArcRun` can consume.

**Phase 1-3 invariants preserved:** Brand pattern; pointer-kind families; the existing dispatch-marker conventions at [`fp-library/src/dispatch/`](../../../../../fp-library/src/dispatch/); the [`StateBrand<P, S>`](../../../../../fp-library/src/brands/effects.rs) per-pointer parameterisation pattern.

**Phase 1-3 invariants broken:** None directly. New pointer brand `BoxBrand` registers parallel to `RcBrand`/`ArcBrand`; this is additive.

**Findings resolved:** F2, M2 (via the unification side effect; M2-Option-B collapses into this).

**Phase touch:** Phase 4 only.

### Why no R3 (HFunctor / Hefty Algebras paper rewrite).

The Hefty Algebras paper proposes an HFunctor encoding where scoped effects are higher-kinded structural types. This was the obvious "wholesale redesign" candidate for the user's invitation. Two reasons not to pursue it:

1. **Heftia v0.7 itself moved to freer-monad encoding** (`Eff = D.Eff Freer` per [`heftia/src/Control/Monad/Hefty/Types.hs:32`](https://github.com/sayo-hs/heftia/blob/master/heftia/src/Control/Monad/Hefty/Types.hs#L32)). Adopting HFunctor would diverge from the library that sources the "heftia-inspired" naming, not converge on it. The plan's freer-monad-encoded scoped row is already structurally aligned with current heftia.
2. **R1 + R2 close the load-bearing findings** within the existing encoding at medium cost. HFunctor closes the same findings at x-large cost without delivering capability beyond what R1+R2 plus a future v2 substrate (FTCQueue-style scoped continuations per F2-Option-B) would give.

If user-defined scoped effects in v2 surface a real need for HFunctor's structural-fold semantics that R1+R2 cannot accommodate, revisit then. For Phase 4, do not pre-pay the cost.

## Sequencing Plan

Both R1 and R2 are POC-validated (`fp-library/tests/poc_rc_run_interpose.rs` for R1's substrate-level `interpose` primitive on `RcRun`; [`fp-library/tests/poc_send_catch_brand.rs`](../../../../../fp-library/tests/poc_send_catch_brand.rs) for R2's parallel-Send-brand pattern on `SendCatchBrand`). The sequencing below assumes the POCs' mechanisms translate to the production-generic shape, which the constraint-surface analysis suggests; substrate work on the Explicit family (HRTB-poisoning workarounds at `arc_run.rs`) remains an unprototyped wrinkle in items 3 and 5.

1. **Plan-text edits closing minors and clarifying decisions (small, ~half a session).** Lift the heftia-divergence note from [resolutions.md:1893-1906](../../resolutions.md#L1893-L1906) into [decisions.md section 4.5](../../decisions.md#L436) and [plan.md Phase 4 header](../../plan.md#L1970). Fix m1 (Run<R, A> vs Run<R, S, A>). Enumerate Phase 4 step 7 compile_fail tests (m8). Add the `Local`/`Reader::Ask` interaction note (M4-Option-A, M6-Option-A). Documentation only; runs `just fmt && just doc` per the doc-only-verify convention. Conventional commit prefix: `docs(effects)`.
2. **R1 specification commit (medium, a few days).** Write the `DispatchScopedHandlers` trait spec into [plan.md Phase 4 step 3](../../plan.md#L2015-L2017); document the substrate-level `Run::interpose` primitive (per the POC reference); revise the per-wrapper interpret-method shape; reverse the F3A `S = CNilBrand` tightening note in [resolutions.md:1232](../../resolutions.md#L1232) with an addendum referencing this remediation; remove or rewrite the Phase 6+ deferred [`interpose` family entry at plan.md:2321-2337](../../plan.md#L2321-L2337) since `Run::interpose` ships as part of Phase 4 substrate. Documentation only; no code yet. Conventional commit prefix: `docs(effects)`.
3. **R1 implementation phase (large, one to two weeks).** Phase 4 step 1: `ScopedCoproduct<S>` and `Node`-based dispatch. Phase 4 step 1a (new): substrate-level `Run::interpose` primitive on each of the six Run wrappers (template in the POC at `poc_rc_run_interpose.rs:74-110`; generalise over `<EBrand, Idx, R>` mirroring `interpret_with_shared`). Phase 4 step 1b (new): `DispatchScopedHandlers` trait and per-wrapper interpret rewrite consuming `interpose`. Lift `S = CNilBrand` from each of the six wrapper interpret-method signatures. Land cons-cell impls for `Catch` and `Span` (the simplest two) plus integration tests. Conventional commit prefix: `feat(effects)`.
4. **R2 specification commit (small, a session).** Plan revision for [plan.md Phase 4 step 2](../../plan.md#L1975-L2014) reflecting `P: RefCountedPointer` parameterisation. Conventional commit prefix: `docs(effects)`.
5. **R2 implementation phase (large, one to two weeks).** Per-pointer-brand parameterisation of the four standard scoped ops, smart-constructor dispatch via Val/Ref<P> markers, register `BoxBrand`. Mirrors the Phase 3 step 5a budget. Conventional commit prefix: `feat(effects)`.
6. **Bracket dispatcher with Drop-guard (medium, a few days).** M3-Option-A: implement `BracketGuard<A, F>` and route the bracket dispatcher through it. Add panic-during-body and panic-during-release tests. Conventional commit prefix: `feat(effects)`.
7. **Phase 4 standard scoped-effect rollout (large, one to two weeks).** Land `Local` / `RefLocal`, full `Bracket` (Val + Ref<RcBrand> + Ref<ArcBrand>), `Span` cons-cell impls plus the `scoped_effects!` and `define_scoped_effect!` macros. Per-step deviation entries. Conventional commit prefix: `feat(effects)`.
8. **Phase 4 standard handlers (medium).** `run_reader_local_dispatcher`, `run_except_catch_dispatcher`, etc. paralleling the FO handlers. Conventional commit prefix: `feat(effects)`.
9. **Phase 4 review-remediation pass (small).** Documentation pass per the Phase 3 step 8 pattern. Conventional commit prefix: `docs(effects)`.

Items 1, 2, 4, 9 are small documentation commits. Items 3, 5, 7 are the substantive implementation phases. Item 6 is the panic-safety remediation. Item 8 is the standard-handler rollout.

## Findings with No Clean Fix

**F2 has a partial-only clean fix.** R2 (`P: RefCountedPointer` parameterisation) lifts the multi-shot ceiling on `RcRun` / `ArcRun` / `RcRunExplicit` / `ArcRunExplicit`. On default `Run` and `RunExplicit` (the non-refcounted Box-based wrappers), bodies and recovery closures remain single-shot because the underlying substrate's `Free` is non-`Clone`. Multi-shot bodies on default `Run` are not fixable without either (a) moving the default Run to a refcounted substrate (which deletes the [decisions.md six-variant Free decision](../../decisions.md#L597-L614) for the default case and is not a Phase 4 question), or (b) introducing FTCQueue-style scoped continuations per F2-Option-B (substrate addition deferred to v2).

**Plan-revision text:** Add a "Closure-storage ceiling on default `Run`" subsection to [decisions.md section 4.5](../../decisions.md#L436-L573) stating: "User-defined scoped effects requiring multi-shot bodies (Coroutine, Provider, Retry, scoped NonDet with backtracking) are not expressible on the default `Run` and `RunExplicit` wrappers; users requiring multi-shot scoped bodies must use `RcRun`/`ArcRun`/`RcRunExplicit`/`ArcRunExplicit`. The standard scoped set (`Catch`, `Local`, `Bracket`, `Span`) is single-shot on every wrapper and is therefore unaffected. Lifting the multi-shot ceiling on the default `Run` family would require introducing FTCQueue-style scoped continuations as a substrate addition; this is deferred to a v2 surface and is not part of Phase 4."

This is honest about what the encoding can and cannot do, names the workaround (`RcRun`/`ArcRun`), and names the future fix (FTCQueue). It is the limitation-acknowledgement the rubric Section 6.2 #8 readers deserve.
