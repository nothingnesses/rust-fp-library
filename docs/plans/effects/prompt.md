# Agent prompt: implement the effects port

Use this prompt to start or continue implementation work on the
purescript-run port. It is self-contained: an agent given this prompt
plus a working tree at the repo root has everything it needs.

## Your role

You are a software engineer implementing the multi-phase port of
`purescript-run` into `/home/jessea/Documents/projects/rust-fp-lib/fp-library`.
The design is fixed; your job is to land code, tests, and benches
against the phased steps in
[plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md),
one step per commit, until the phase is complete or you hit a blocker.

## Current resume point

Phase 1 complete; Phase 1 follow-up both commits landed
(`WrapDrop` migration plus the `Functor` -> `Kind` relaxation);
Phase 2 complete (all 10 steps). The `poc-effect-row/` workspace
was deleted in step 10b after its tests migrated to
[`fp-library/tests/run_row_canonicalisation.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_row_canonicalisation.rs)
in step 10a.

**Phase 3 (first-order effect handlers, interpreters, natural
transformations) is the active phase.** Steps 1, 2, 3, 4
shipped. The adversarial-review reversal cleanup is complete:
F1D (`05be270`, delete `run_accum` / `run_accum_rec`), F3A
(`f8031c5`, tighten `S = CNilBrand` on the interpreter family),
and M3C (`b8c9b3c`, parameterise `interpret_with` over
`P: RefCountedPointer`). All six wrappers' State smart
constructors landed: 6a.1 + 6a.2 (`96bc448` + `f865152`), 6a.3
(`619127e`, `RcRun::get/put`), 6a.5 (`db07a2f`, Explicit non-
Arc family), and 6a.4 + 6a.6 (`7a0d04b`, Arc family using a
parallel
[`SendStateBrand`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/brands.rs)
/
[`SendState`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/state.rs)
type per the
[2026-05-03 SendFunctor option-(c) resolution](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified)).
**Smart constructors compile but the Arc family doesn't
dispatch end-to-end;** see active blocker below.

**Active blocker (2026-05-04):** attempting to ship
`run_state.rs` integration tests surfaced a downstream issue
on the Arc family. `ArcCoyoneda`'s dispatch impl bounds
`EBrand: Functor + SendFunctor`
([`interpreter.rs:337`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/interpreter.rs)),
but `SendStateBrand` cannot honestly implement `Functor`
because [`Functor::map`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/functor.rs)'s
signature only requires `f: impl Fn` (no `Send + Sync` bound)
while `SendState`'s variants store
`<P as SendRefCountedPointer>::Of<'a, dyn Fn(...) -> A + Send + Sync>`
(closures must be `Send + Sync` at storage time).
`ArcRun::interpret` on a `SendStateBrand`-headed row hits
this and fails. Four design options surveyed in
[plan.md's Active blockers](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#active-blockers):
(α) migrate `ArcCoyoneda`'s algebra to `SendFunctor`, the
principled extension of Phase 2 step 9's `ArcFree` migration;
(β) parallel `send_lower_ref` method, structurally harder
than it sounds; (γ) defer 6a.4 + 6a.6 indefinitely; (c'')
parallel `SendArcCoyoneda` variant. **No recommendation
locked in; user decision pending.** Working draft of
`run_state.rs` covering all six wrappers preserved in
`git stash@{0}`.

**Immediate pending task (after blocker resolves):** complete
`run_state.rs` integration tests (pop `git stash@{0}`, fix
`State<'_, ...>` lifetime annotations, adjust Arc family
tests per chosen option). Then step 5 (`interpret_with_rec`
pipeline-plus-MonadRec family) is the next greenfield step,
unaffected by the blocker.

### Resolution path (after user decides on α / β / γ / c'')

- **(α) ArcCoyoneda algebra migration:**
  1. Replace `F: Functor` with `F: SendFunctor` in
     [`arc_coyoneda.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/arc_coyoneda.rs):
     `ArcCoyonedaLowerRef::lower_ref` trait method, three
     layer impls (Base, MapLayer, NewLayer), public
     `ArcCoyoneda::lower_ref`, and downstream methods that
     call `self.lower_ref()` (collapse, hoist, fold, bind,
     apply).
  2. Update bodies to use
     [`F::send_map`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/send_functor.rs)
     instead of `F::map`.
  3. Add `SendFunctor` impl for
     [`VecBrand`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/vec.rs)
     (currently only implements `Functor`; the
     `SendFunctor::send_map` body is byte-identical to
     `Functor::map`'s, just with tighter bounds). Iterate
     `just check` for any other brand surfaced by compile
     errors and add SendFunctor impls similarly.
  4. Drop `+ Functor` from the
     [`ArcCoyoneda` dispatch impl](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/interpreter.rs)
     in `interpreter.rs:337`.
  5. Run `just verify`. Existing `arc_coyoneda` doctests
     using `VecBrand`, `OptionBrand`, etc. should pass
     (those brands have or will have SendFunctor).
  6. Pop `git stash@{0}`, fix `State<'_, ...>` lifetimes
     (the FnOnce-not-general-enough error in the original
     draft was caused by pinning the State projection
     lifetime to `'static` instead of letting it be HRTB-
     polymorphic).
  7. Run `just verify` again. Commit migration + run_state
     tests.
  8. Move active-blocker entry to resolutions.md as a new
     dated resolution.

- **(β) parallel `send_lower_ref`:** see plan.md's blocker
  entry for the structural problem (`B: Send + Sync` on
  layer impls forces `ArcCoyoneda::map`'s `B` parameter
  too); not recommended.

- **(γ) defer 6a.4 + 6a.6:** ship the four non-Arc
  `run_state.rs` tests; document the Arc family as an open
  gap in deviations.md; revert the `SendStateBrand` /
  `SendState` types if cleaner; or leave them as
  scaffolding.

- **(c'') parallel `SendArcCoyoneda`:** new module
  `send_arc_coyoneda.rs`; brand registration; rewrite Arc
  family smart constructors to target the new brand; large
  refactor.

### Step 5 implementation pattern (next greenfield work)

`interpret_with_rec` combines step 3's pipeline row-narrowing
shape with step 4's `MonadRec`-target stack-safety. Per-wrapper
inherent method:

```rust
pub fn interpret_with_rec<MBrand, EBrand, Idx, RMinusE>(
    self,
    handler: impl Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Wrapper<RMinusE, CNilBrand, A>>>)
        -> M::Of<'_, Wrapper<RMinusE, CNilBrand, A>>
        + 'static,
    // (plus Send + Sync on Arc wrappers)
) -> M::Of<'_, Wrapper<RMinusE, CNilBrand, A>>
where MBrand: MonadRec + ...,
```

Implementation:

1. Wrap the handler in
   `<P as RefCountedPointer>::Of<'_, F>` once at entry (per
   M3C's pattern, drops the `Clone` bound).
2. Drive the loop via
   [`tail_rec_m`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/monad_rec.rs)
   with a step closure that:
   - Peels the program.
   - On `Ok(a)`: returns `M::pure(ControlFlow::Break(M::pure(Wrapper::pure(a))))`
     (or similar shape; consult step 4's body for exact
     shape).
   - On `Err(Node::First(layer))`: projects via
     [`Member::project`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/member.rs).
     - Matched arm: lower the Coyoneda; M-fmap the recursive
       continuation; hand to handler; wrap in
       `ControlFlow::Continue` for tail_rec_m.
     - Unmatched arm: M-fmap the recursive continuation; re-
       emit via the substrate's `wrap` operation; wrap in
       `ControlFlow::Continue`.
3. Pin M's lifetime per family: `'static` for the Erased
   trio, `'a` for the Explicit trio (matches step 4).
4. Reuse the
   [`DispatchHandlers`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/interpreter.rs)
   trait if applicable; step 4 reuses it unchanged with
   `NextProgram = M::Of<...>`. Step 5 may need different
   dispatch since it walks one effect at a time rather than
   the full handler list.

Step 5's tests in `fp-library/tests/run_interpret_with_rec.rs`
should cover each wrapper x M-target combination viable for
that wrapper's substrate constraints (Erased non-Arc +
ThunkBrand + OptionBrand; Arc family + OptionBrand only
because Thunk is `!Send`). Use State (now available on all
six wrappers) as one of the test scenarios to exercise the
pipeline-plus-MonadRec interaction with a real first-order
effect.

Watch out for: HRTB poisoning on ArcRun (continues to apply
in step 5's body); the
[`unwrap_first`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/arc_run.rs)
/
[`make_node_first`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/arc_run.rs)
/
[`wrap_first_arc`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/arc_run.rs)
helpers from step 3's body are likely needed verbatim. The
Arc family's per-method `Send + Sync` cascade is large (~12
clauses); step 5's signatures will be the largest yet.

**Phase 2 (complete; all 10 steps).** Built the Run-wrapper
foundation: `frunk_core`-based Coproduct adapter,
`VariantF<Effects>` Coyoneda-wrapped row,
[`Member<E, Idx>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/member.rs)
trait, six Run wrappers (`Run` / `RcRun` / `ArcRun` /
`RunExplicit` / `RcRunExplicit` / `ArcRunExplicit`) with their
`pure` / `peel` / `send` / `bind` / `map` / `ref_*` / `lift`
inherent methods, the
[`effects!`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/effects_macro.rs)
/ `raw_effects!` row macros, the `im_do!` proc-macro, and the
row-canonicalisation regression baseline at
[`fp-library/tests/run_row_canonicalisation.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_row_canonicalisation.rs).
Phase 2 surfaced two recurring constraints that shape Phase 3
work: the HRTB-poisoning pattern across `ArcRun`-substrate code
(see Lessons below) and the per-`A` HRTB-over-types limit that
caps brand-level `SendFunctor` coverage on the Arc family.
Per-step detail lives in
[plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
`Earlier completed steps (commit log)` section.

What Phase 3 has shipped (commit-hash + one-line summary;
full per-step narratives in
[plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)
and per-step deviations in
[deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)):

- **Step 1** (`82dd7bb`):
  [`handlers!{...}`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/handlers.rs)
  macro plus `nt()` builder fallback for assembling natural
  transformations; runtime carrier types `Handler<E, F>` /
  `HandlersNil` / `HandlersCons<H, T>` at
  [`handlers.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/handlers.rs).
- **Step 2** (`d5efe2a`, then F1D `05be270` and F3A
  `f8031c5` cleanup): `interpret` / `run` inherent methods on
  all six Run wrappers + the
  [`DispatchHandlers`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/interpreter.rs)
  trait with three Coyoneda-variant cons-cell impls. M-free
  while-loop shape; one of three orthogonal interpreter
  primitives Phase 3 ships (see lesson "Three orthogonal
  interpreter primitives" below). State threading is via
  user-side closure captures applied to `interpret` directly;
  no separate `run_accum` companion (F1D removed it). The
  `S` bound is fixed to `CNilBrand` so the `Node::Scoped` arm
  is structurally uninhabited via `match cnil {}` rather than
  a runtime panic (F3A); Phase 4 will add a parallel
  scoped-handler family without this bound.
- **Step 3** (`ff84f20`): pipeline row-narrowing
  `interpret_with::<EBrand, Idx, RMinusE>(handler) -> Wrapper<RMinusE, S, A>`
  plus empty-dual-row terminal `extract(self) -> A` on all six
  wrappers. ArcRun gains three HRTB-free helpers
  (`make_node_first`, `wrap_first_arc`, `unwrap_pure_node`).
- **Step 4** (`bd540d5` + `fafcfde`, then F1D `05be270` and
  F3A `f8031c5` cleanup): MonadRec-target interpreter family
  `interpret_rec` / `run_rec`. Two-commit split: relax
  `DispatchHandlers::dispatch` from `&mut self` to `&self` +
  `Fn`, then add the rec methods using
  [`tail_rec_m`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/monad_rec.rs).
  Implementation deviation (lessons section below): M's
  lifetime pinned per family rather than HRTB-quantified
  because stable Rust closures can't be HRTB-polymorphic over
  `Thunk<'h, T>`-shaped types. Arc family uses `OptionBrand`
  rather than `ThunkBrand` in doctests (Thunk is `!Send`).
- **Step 6a.1** (`96bc448`): State effect type machinery.
  [`StateBrand<P, S>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/brands.rs)
  parameterised by `P: ToDynCloneFn` (typically `RcBrand` /
  `ArcBrand`) and `S: 'static`;
  [`State<'a, P, S, A>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/state.rs)
  enum with Get / Put variants holding
  `<P as RefCountedPointer>::Of<'_, dyn Fn(...) -> A>`
  continuations. `Functor` impl shipped; `SendFunctor`
  deferred (active blocker — see Resume point above).
- **Step 6a.2** (`f865152`): `Run::get` / `Run::put` smart
  constructors. Threads `RcBrand` as the pointer kind;
  continuations via
  [`<RcBrand as ToDynCloneFn>::new(closure)`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/to_dyn_clone_fn.rs).
- **Reversal cleanup** (`05be270` F1D + `f8031c5` F3A +
  `b8c9b3c` M3C): three commits implement the
  [2026-05-03 reversal resolution](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-03-adversarial-review-reversals-delete-run_accum-ship-interpret_with_rec-parameterise-interpret_with-over-refcountedpointer)
  in-place against steps 2, 3, and 4. F1D deletes
  `run_accum` / `run_accum_rec` (state-threading via closure
  capture). F3A tightens `S = CNilBrand` on the interpreter
  family's impl block (removes six `clippy::unreachable`
  suppressions). M3C parameterises `interpret_with` over
  `P: RefCountedPointer`: each public outer method wraps the
  user handler in `Of<'_, F>` once at entry and delegates to
  a private inner `interpret_with_shared`; recursive
  narrowing clones the pointer (refcount bump) instead of
  the closure, dropping the `Fn + Clone + 'static` bound to
  `Fn + 'static` (plus `Send + Sync` on Arc).
- **Step 6a.3** (`619127e`): `RcRun::get` / `RcRun::put`
  smart constructors plus a manual `Clone` impl for `State`
  gated on `S: Clone`. `RcRun::lift`'s
  `Apply!(<EBrand>::Of<'static, A>): Clone` bound forced the
  `State::Clone` impl; the bound cascades to all four
  shared-substrate wrappers' smart constructors.
- **Step 6a.5** (`db07a2f`): Explicit non-Arc family
  (`RunExplicit::get/put` + `RcRunExplicit::get/put`). One
  commit covering both wrappers because the
  Explicit-vs-Erased axis is orthogonal to single-thread-vs-
  thread-safe; both thread `RcBrand`. `A: 'static` required
  even on Explicit wrappers (driven by `StateBrand<P, S>`'s
  `impl_kind!` `S: 'static`).
- **Step 6a.4 + 6a.6** (`7a0d04b`): Arc family
  (`ArcRun::get/put` + `ArcRunExplicit::get/put`) plus a
  parallel
  [`SendStateBrand<P, S>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/brands.rs)
  /
  [`SendState<'a, P, S, A>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/state.rs)
  type. Closes step 6a (all six wrappers covered). The
  earlier `4bd1636` ratification of option (b) per-method
  bounds was discovered structurally unimplementable
  (`Arc<dyn Fn>: Send + Sync` is provably false because the
  trait object's bounds don't include `Send + Sync`); see
  the
  [option (c) re-ratification](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified)
  for the (b) discovery details and (c) implementation. The
  Arc smart constructors use `SendStateBrand<ArcBrand, S>`
  in the row; non-Arc constructors keep using
  `StateBrand<P, S>`.

Cross-cutting commits during step 6a (shipped alongside, no
phase-step number):

- **`4f0e977`** (`docs(effects):`): wrapped
  [`handlers.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/handlers.rs),
  [`interpreter.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/interpreter.rs),
  and
  [`member.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/member.rs)
  in `#[fp_macros::document_module]`.
- **`3a5a0a8`** (`fix(macros):`): tightened
  `#[document_examples]` validation to reject six trivially-
  true assertion patterns; refactored 24 existing trivial-
  assertion doctests across fp-library + fp-macros.

**Remaining Phase 3 steps:**

- **Step 6a integration tests in `run_state.rs` (open
  follow-up; recommended before step 5):** see "Open
  follow-up" subsection in the resume point above.
- Step 5 (`interpret_with_rec`, immediate next greenfield
  step): per-wrapper inherent method combining the pipeline
  shape (step 3) with `tail_rec_m` (step 4). Six new method
  bodies + integration tests in
  `fp-library/tests/run_interpret_with_rec.rs`. State (now
  available on all six wrappers) drives one of the test
  scenarios.
- Step 6b-6e: `Reader`, `Except`, `Writer`, `Choose` effects
  - their smart constructors (one effect per sub-step;
    `Choose` ships on the four multi-shot wrappers per the
    2026-05-03 resolution's Q4=ii).
- Step 7: `define_effect!` macro at
  `fp-macros/src/effects/define_effect.rs` mechanically
  generating the six per-wrapper variants from one user
  declaration.
- Step 8: `compile_fail` UI tests for negative cases (handler
  missing an effect, wrong type ascription, multi-shot via
  single-shot `Run`, `Choose` on single-shot wrappers).
- Step 9: review-remediation documentation pass — bundle the
  docs-only items from
  [`remediation_proposals.md`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/remediation_proposals.md)
  (F2A, F4A, F5A, M4 audit, M6A async-via-`spawn_blocking`,
  M7A bind/handler asymmetry note, all minor m1-m9) into one
  commit. Lands after the substantive code work above so the
  docs reflect the settled state.

If you encounter unexpected behaviour during Phase 3
implementation, plan.md's `Active blockers` section is the
place to record load-bearing questions; entries should cite
concrete file paths and line numbers so the next implementor
(or you in a future session) can verify claims without
conversational context.

## Lessons learned in Phase 2 (load-bearing for Phase 3)

These were learned during Phase 2 sub-steps 5, 9a-9i, and 10a.
Reading them up front saves re-discovery cycles in Phase 3.

### Coyoneda variant pairing rule (load-bearing for Phase 3 step 4)

Each Run wrapper's `lift` uses the Coyoneda variant whose
pointer kind matches its substrate:

- `Run` / `RunExplicit` -> bare `Coyoneda`.
- `RcRun` / `RcRunExplicit` -> `RcCoyoneda`.
- `ArcRun` / `ArcRunExplicit` -> `ArcCoyoneda`.

Driver: `*Run::send` and `*Run::peel` carry per-method
`Of<'_, *Free<..., *TypeErasedValue>>: Clone` bounds intrinsic
to the shared-`Rc`/`Arc` substrate state. Bare `Coyoneda`'s
`Box<dyn FnOnce>` continuation is not `Clone`, so only the
matching shared-pointer Coyoneda variant satisfies the bound.
The plan's per-wrapper delta table for step 9h initially
specified bare `Coyoneda` for `RcRun`/`RcRunExplicit`; that
proved unsatisfiable and was corrected in step 9h's deviations
entry.

When Phase 3 step 4's smart constructors (`ask`, `get`, `put`,
`tell`, `throw`) define one-liners over `*Run::lift`, they need
to carry the same pairing in their effect-row parameterisation.
A user-facing `ask` likely needs to either (a) be parameterised
over the Run wrapper, (b) ship six variants, or (c) pick one
canonical wrapper per effect family. Resolve this design
question before writing per-effect smart constructors.

### HRTB-poisoning workaround pattern (relevant whenever Arc-substrate code is touched)

`ArcRun`'s struct-level HRTB on the `Kind` projection
(`Of<'static, ArcFree<..., ArcTypeErasedValue>>: Send + Sync`)
poisons GAT normalization in any scope mentioning the HRTB.
Constructing a `Node::First(layer)` literal inside an
`ArcRun`-impl-block scope fails to unify with
`<NodeBrand<R, S> as Kind>::Of<'_, A>` even though `impl_kind!`
declares them equal.

**Workaround**: receive projection-typed values as parameters;
never construct projection-typed values inside an HRTB-bearing
scope. The probe at
[`fp-library/tests/arc_run_normalization_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/arc_run_normalization_probe.rs)
documents four passing patterns and is the regression-test home
for this limit. The free
[`lift_node`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/arc_run.rs)
helper (used by `ArcRun::lift`) is the precedent fallback. If
Phase 3 handlers / smart constructors need to construct
projection-typed values in HRTB-bearing scopes, use the same
pattern.

### Per-`A` HRTB-over-types blocks brand-level type-class delegation

Stable Rust does not support `for<T>` HRTBs. When a brand-level
type-class trait requires per-`A` bounds (`A: Clone`, per-`A`
`F::Of<...>: Clone + Send + Sync`), the bounds cannot be
expressed in the trait method signature. This is the same wall
that blocked:

- Brand-level `Functor`/`Semimonad` on `RcFreeExplicitBrand`
  (step 4b).
- Brand-level `SendFunctor`/`SendSemimonad` on
  `ArcFreeExplicitBrand` (step 9d) and `ArcRunExplicitBrand`
  (step 9g).
- Brand-level `SendRefFunctor`/`SendRefSemimonad` cascade on
  `ArcRunExplicitBrand` (step 9i).

Workaround pattern: implement the operation on the concrete
type as an inherent method (where per-`A` bounds work in the
where-clause) and document the brand-level gap in
[`fp-library/docs/limitations-and-workarounds.md`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/limitations-and-workarounds.md).
The `im_do!(ref ...)` macro form already routes around
brand-level gaps via inherent-method delegation; if Phase 3
brand-level handler dispatch on Arc-substrate types hits the
same wall, use the same pattern.

### `effects!` vs `raw_effects!` distinction (relevant for Phase 3 steps 1, 6)

[`effects!`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/effects_macro.rs)
is the public macro that produces Coyoneda-wrapped Coproduct
brand rows (each variant satisfies the row-Functor requirement
because Coyoneda is unconditionally Functor regardless of its
inner).
[`raw_effects!`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/lib.rs)
at `fp_library::__internal` is the internal macro that produces
raw Coproduct brand rows (no Coyoneda wrap), used in test
fixtures and lower-level combinators.

Phase 3 handlers should generally consume rows produced by
`effects!`; Phase 3 step 8's compile_fail UI tests may use
`raw_effects!` to construct test-only edge cases.

### Bundle substrate/wrapper migrations by default (load-bearing for any future cascade work)

`ArcFree`'s bound replacement in step 9b broke `ArcRun`'s
methods at every call site; 9b had to bundle with 9e to keep
the workspace compiling. Same coupling drove the 9c+9f bundle.
**Bundle migrations across substrate/wrapper pairs by default**;
surface the bundling decision to the user as a deviations.md
entry but don't try to land the substrate half independently.
Phase 3 is unlikely to surface substrate-vs-wrapper migrations
of this shape, but the principle generalises to any
cascade-coupled refactor.

### Mechanical operational gotchas

- **`compile_fail` `.stderr` files are sensitive to
  bound-changes.** Phase 2 migrations shifted error-message
  line numbers in the
  `arc_free_explicit_bind_requires_send` UI test; regenerate
  with `TRYBUILD=overwrite cargo test --test compile_fail`
  (raw `cargo`, not `just test`, to avoid `wip/` artifacts).
  Phase 3 step 8's compile_fail UI tests will need the same
  regeneration treatment whenever bounds shift.
- **Clippy's `type_repetition_in_bounds` lint requires a
  type's bounds to be in one place.** If you split bounds
  between the generic-param-list and the where-clause (e.g.,
  `<B: 'a>` plus `B: Send + Sync` in where), clippy fails.
  Consolidate: `<B>` plus `B: Send + Sync + 'a` in where.

## Lessons learned in Phase 3 (load-bearing for remaining Phase 3 + Phase 4)

These were learned during Phase 3 step 1 (`82dd7bb`) and
step 2 (`d5efe2a`) plus the active-blocker design analysis.

### Handler-list runtime carrier shape

`HandlersNil` / `HandlersCons<H, T>` is fp-library's own
cons-list, distinct from `frunk_core::hlist::{HCons, HNil}`
already re-exported under
[`crate::types::effects::coproduct`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/coproduct.rs).
The two are intentionally different types: frunk's HList
provides type-level position witnesses for row-membership
proofs (`Here` / `There`, `CoprodInjector`); fp-library's
`HandlersCons` carries runtime handler closures aligned with
the row's value-level shape. The distinction lets inherent
`.on::<E, F>(...)` builder methods live on the handler-list
types directly without an extension-trait dance. Don't conflate
them.

`Handler<E, F>` uses `PhantomData<fn() -> E>` (variance-neutral)
rather than `PhantomData<E>`. The brand `E` is a zero-sized
marker type used purely for tagging; `fn() -> E` keeps the
newtype free of variance and `Send`/`Sync` concerns inherited
from `E` itself. Standard "phantom for tagging" idiom.

The handler list's builder uses **prepend semantics**:
`nt().on::<A, _>(ha).on::<B, _>(hb)` produces
`HandlersCons<Handler<B>, HandlersCons<Handler<A>, HandlersNil>>`
(B at head). The macro sorts lexically, so users wanting
macro-equivalent ordering should call `.on()` in
reverse-lexical order. Documented at the module level in
[`handlers.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/handlers.rs).

### `DispatchHandlers` trait + per-Coyoneda-variant impls

The
[`DispatchHandlers<'a, Layer, NextProgram>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/interpreter.rs)
trait walks a `HandlersCons` / `HandlersNil` against the
row's value-level `Coproduct` chain in lock-step. It has
**four impls**: a base case for `HandlersNil` paired with
`CNil`, plus three cons-cell impls — one per Coyoneda variant
(`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`). The duplication is
mechanical: identical body, different `lower*` method (bare
`Coyoneda::lower` consumes self; the Rc/Arc variants ship
`lower_ref(&self)` only).

Step 3 (`ff84f20`) shipped row-narrowing without adding a
parallel `DispatchOneHandler` trait: the existing
[`Member::project`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/member.rs)
already does the chain walking, and the per-Coyoneda-variant
`lower` choice is one line of wrapper-local code; abstracting
into a trait would have added ceremony without enabling shared
code paths. See
[deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
under Phase 3 step 3 for the alternatives considered (a
`DispatchOneHandler` trait keyed on the Coyoneda variant or on
the chain shape, both rejected). The "trait" wording in the
original plan text was a design pattern, not a contract on the
Rust artifact name.

Step 4's MonadRec-target family is expected to **reuse the
same `DispatchHandlers` trait unchanged**, just instantiated
with `NextProgram = M::Of<Run<R, S, A>>` (the M-wrapped
continuation type) per the active-blocker recommendation Q1.A.
The interpreter does
`<R as Functor>::map(M::pure, peel_layer)` to lift Run-
continuations to M-wrapped before dispatch.

### Closure-mono-in-`A` constraint matches PureScript Run runtime

PureScript Run's
[`interpret`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
is the rank-2 polymorphic API; its actual implementation is
`run` (literally aliased). The `run` form's handler is
`(VariantF r (Run r a) -> m (Run r a))` — mono in `a`. fp-library
adopts the mono-in-`a` form so handler closures fit Rust's
non-generic-closure constraint. Each `Handler<E, F>` cell
carries a closure of shape
`FnOnce(<EBrand as Kind>::Of<'_, NextProgram>) -> NextProgram`
where `NextProgram` is the Run wrapper specialized to the
program's result type `A`.

Rank-2 polymorphic targets reach for
[`NaturalTransformation`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/natural_transformation.rs)
directly via
[`Free::fold_free`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/free.rs).
A future `interpret_nt`-style companion entry-point is recorded
in plan.md's Phase 6+ deferred items.

### HRTB-poisoning workarounds expanded across `ArcRun`

Phase 2 step 5's HRTB-poisoning workaround (`*Run::send`
takes a Node-projection value rather than constructing one
inside the impl-block scope) **recurs** in any `ArcRun`-impl-
block code that pattern-matches `Node` literals or constructs
`Node` projections. The projection equality declared by
[`impl_kind!`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/lib.rs)
won't normalize under the struct-level HRTB
(`<NodeBrand<R, S> as Kind>::Of<'static, ArcFree<...>>: Send + Sync`).

`ArcRun` ships **five HRTB-free helpers** at module scope in
[`arc_run.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/arc_run.rs),
each addressing a specific pattern that the struct-level HRTB
would otherwise poison:

- `lift_node<R, S, EBrand, Idx, A>(effect)`: builds the
  `Node::First(<R as Member>::inject(Coyoneda::lift(effect)))`
  projection outside the HRTB scope. Used by `ArcRun::lift`.
- `unwrap_first<R, S, A>(node)`: pattern-matches a
  `Node::First` projection outside the HRTB scope, returning
  the row-level layer. Used by `ArcRun::interpret` and
  `ArcRun::interpret_with`.
- `make_node_first<R, S, A>(layer)`: HRTB-free `Node::First`
  literal builder. Used by `ArcRun::interpret_with`'s
  unmatched arm.
- `wrap_first_arc<RMinusE, S, A>(node)`: forwards a pre-built
  `Node` projection to `ArcFree::wrap` without constructing
  any `Node` literal inside its own HRTB-bearing scope. Used
  by `ArcRun::interpret_with`'s unmatched arm.
- `unwrap_pure_node<Inner, Ret>(node)`: statically eliminates
  a `Node` over an empty dual row. Both `Node` arms carry
  uninhabited `CNil` payloads, so the body diverges to `!`,
  which coerces to the caller's `Ret` without runtime panic.
  Used by `ArcRun::extract`.

**Other five Run wrappers do not need these helpers**: only
`ArcRun` (Erased Arc) has the struct-level HRTB;
`ArcRunExplicit` carries the `Send + Sync` bounds per-method,
not at the struct level, so it pattern-matches `Node` literals
inline.

If a future step (Phase 3 step 5 / step 6; Phase 4 scoped
effects) needs new `Node`-construction or `ArcFree::wrap`-call
sites inside `ArcRun`'s impl block, expect to add another
helper following the same naming convention. Step 4's
MonadRec-target loop is unlikely to need new helpers (it
returns `M::Of<A>` directly; no narrowed Run construction in
scope), but Phase 4's scoped effects will likely add at least
a `Node::Scoped` analog of `make_node_first` / `unwrap_first`.

### Recursive structural narrowing pattern (load-bearing for any future row-walking step)

Phase 3 step 3's `interpret_with` shipped the canonical
pattern for "walk a Run program, narrowing each layer's
content recursively":

1. `peel` the program; on `Ok(a)` return
   `Wrapper::pure(a)`; on `Err(Node::First(layer))` continue.
2. Project the target effect from the layer via
   [`Member::project`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/member.rs).
3. Matched arm: `coyo.lower()` (or `lower_ref` for shared-
   pointer Coyoneda variants), then map a recursive call to
   the same operation over each inner sub-program via
   `<EBrand as Functor>::map(narrow, lowered)`, then hand to
   the user's handler.
4. Unmatched arm: map the recursive call over each inner
   sub-program in the remainder via
   `<R_minus_E as Functor>::map(narrow, rest)`, then re-emit
   via the substrate's `wrap` operation.

The recursion is **structural** (via `Functor::map`) rather
than iterative (via a `loop`). Host-stack-frame depth equals
the chain depth of the program (NOT the structural Wrap depth,
which is bounded at most 1 per the
[WrapDrop probe](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs)).
This is acceptable for typical user programs but unbounded for
deep Identity-shaped chains; Phase 3 step 4's
`tail_rec_m`-driven loop is the stack-safe alternative for
external-M extraction.

The recursive narrowing closure clones the handler before each
inner recursive call (since `Functor::map`'s closure is `Fn`
and the recursive `interpret_with` consumes a handler by
value). This drives the `F: Fn + Clone + 'static` (plus
`Send + Sync` on Arc) handler bound. Users who want
cheap-to-clone handlers wrap captured state in `Rc` / `Arc`,
which Rust infers as `Fn` automatically due to interior
mutability.

When Phase 3 step 6 ships standard first-order effects with
non-Identity shapes (`State<S>` whose `Of<NextProgram>` is a
closure `S -> NextProgram`), the recursive narrowing for those
effects becomes lazy: `Functor::map` over a closure composes
with the new function, and the recursion is deferred until the
state value is supplied. This is a property worth exploiting
for stack safety on State-heavy programs.

### Panic-free `extract` via empty dual row

Phase 3 step 3's `extract` ships with the where-bound tightened
to `Wrapper<CNilBrand, CNilBrand, A>` (both first-order and
scoped rows empty). Both
[`Node`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/node.rs)
arms carry uninhabited
[`CNil`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/coproduct.rs)
payloads, so the body's exhaustive `match cnil {}` on each
side diverges to type `!`, statically proving no runtime
panic. This is **stronger** than the `interpret` family's
panic-on-Scoped pattern (gated by `#[expect(clippy::unreachable, ...)]`)
and is the design pattern any future "fully reduce a Run program
to its result" entry point should follow.

The trade-off: `extract` is no longer callable on programs
whose first-order row is empty but whose scoped row carries
layers. Phase 4's scoped-effect interpreter will introduce a
separate elimination operation for non-empty scoped rows,
keeping `extract`'s remit fully-pure-programs only. When
designing Phase 4's API, mirror `extract`'s panic-free shape
where possible: tighten the where-bound until the body
diverges via uninhabited match, rather than panicking on
unreachable arms.

### Per-wrapper Coyoneda-variant brand in test rows

Phase 3 step 2's integration tests in
[`run_interpret.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_interpret.rs)
use the wrapper-appropriate Coyoneda-variant brand at the row
level:

- `Run` / `RunExplicit`: `CoproductBrand<CoyonedaBrand<...>, CNilBrand>`.
- `RcRun` / `RcRunExplicit`: `CoproductBrand<RcCoyonedaBrand<...>, CNilBrand>`.
- `ArcRun` / `ArcRunExplicit`: `CoproductBrand<ArcCoyonedaBrand<...>, CNilBrand>`.

Per the per-wrapper Coyoneda variant pairing rule from Phase
2 step 9h. Mismatching the brand to the wrapper triggers
peel-bound or Member-bound failures (not always with clear
error messages). When writing new tests, always use the
matching brand.

The `handlers!{}` macro takes the **inner brand** (e.g.,
`IdentityBrand`) regardless of which wrapper the row targets.
The DispatchHandlers impls bind the inner brand and dispatch
on the relevant Coyoneda value variant; users don't need to
reach into `CoyonedaBrand<IdentityBrand>` syntactic form for
the macro key.

### Three orthogonal interpreter primitives (load-bearing context for step 4)

Phase 3 ships three orthogonal interpreter primitives, one per
cognitive model, per the 2026-04-29 resolution
([resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape)):

- **Simple value extraction (step 2, M-free):** `interpret`
  / `run` return `A` directly via a `while`-loop;
  no engagement with `MonadRec` abstraction. Stack-safe by
  construction (no `M::bind` or `M::tail_rec_m` in the body).
  State threading is via user-side closure captures applied
  to `interpret` directly.
- **Pipeline row-narrowing (step 3, partial interpretation):**
  `interpret_with::<EBrand>` returns
  `Wrapper<RMinusE, CNilBrand, A>`; enables compositional
  handler chains and user-controlled handler ordering for
  non-commuting effects. Recursion is host-stack-frame per
  peeled layer (not stack-safe on long Identity-shaped chains;
  step 5's `interpret_with_rec` is the stack-safe pipeline
  variant).
- **MonadRec extraction (step 4, external M target):**
  `interpret_rec` / `run_rec` return `M::Of<A>` for
  `MBrand: MonadRec`; uses `tail_rec_m` for stack-safety on
  external M targets like `Thunk` / `Option` / `Result` /
  `Vec`. State threading parallels step 2's pattern.
- **Pipeline + MonadRec (step 5, pending; per the 2026-05-03
  reversal resolution):** `interpret_with_rec` combines step 3
  and step 4. Closes the orthogonality grid.

Each shape uniquely enables a use case the others cannot
subsume. Step 4's three load-bearing design questions are
resolved per the
[2026-05-02 resolution](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading)
and shipped (see Phase 3 progress above).

### M-lifetime pinning in step 4 (load-bearing for any future MBrand-target work)

Step 4's `interpret_rec` / `run_rec` pin
M's lifetime per Run-wrapper family rather than HRTB-quantifying
it: `'static` for the Erased family (`Run`, `RcRun`, `ArcRun`;
the wrapper struct already requires `'static` everywhere) and
`'a` for the Explicit family (`RunExplicit`, `RcRunExplicit`,
`ArcRunExplicit`; matches the wrapper's struct-level `'a`). The
HRTB `for<'h>` only quantifies over the row brand's projection,
not M's.

**Why pinning is required:** stable Rust closures cannot be
HRTB-polymorphic over a type parameter that contains the
lifetime in non-reference position (e.g., `Thunk<'h, T>`'s `'h`
lives inside the `Box<dyn FnOnce>`'s payload, not behind a
`&'h` reference). Without pinning, a user handler closure over
`Identity<Thunk<'h, ...>>` cannot satisfy `for<'h>`, and the
trait HRTB fails to be discharged. Pinning collapses M's
lifetime to a single concrete value at the call site, sidestepping
the HRTB-over-types limit. This is the same family of
constraints documented in
[Per-`A` HRTB-over-types blocks brand-level type-class
delegation](#per-a-hrtb-over-types-blocks-brand-level-type-class-delegation);
it appears in any setting where a closure's type must be
HRTB-polymorphic over a type with an embedded lifetime.

When a future Phase introduces another MBrand-target trait (e.g.,
a `tail_rec_m_with_state` companion), apply the same pinning
strategy: pin M's lifetime at the call-site family level, keep
HRTB only on row projections.

### `Thunk<'a, T>` is not `Send + Sync` (load-bearing for Arc family + ThunkBrand combo)

`Thunk<'a, T>` is `Box<dyn FnOnce() -> T + 'a>`. The inner trait
object lacks `Send + Sync` by default, so `Thunk: !Send + !Sync`.
This rules out `ThunkBrand` as the M target for the Arc-family's
`interpret_rec` (`ArcRun`, `ArcRunExplicit`), whose substrate
`SendFunctor::send_map` requires `M::Of<...>: Send + Sync`. Use
`OptionBrand` / `ResultBrand` (or any other brand whose `Of` is
`Send + Sync`-by-default) for Arc family rec doctests; reserve
`ThunkBrand` for the Erased non-Arc and Explicit non-Arc
families.

If a future user genuinely needs stack-safe + thread-safe
interpretation, fp-library may add a `SendThunkBrand` or similar
that wraps the closure in `Arc<dyn Fn + Send + Sync>`. Not yet
shipped; not yet requested.

### `<P as RefCountedPointer>::Of<...>` is not `Send + Sync` by default (load-bearing for State + Arc family)

`<ArcBrand as RefCountedPointer>::Of<'a, T> = Arc<T>` for any
`T: ?Sized + 'a`. The bound has no `Send + Sync` clause, so
`Arc<dyn Fn(...) -> A>` from this projection is **not**
`Send + Sync`. The
[`SendRefCountedPointer`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/ref_counted_pointer.rs)
parallel trait carries `T: ?Sized + Send + Sync + 'a` and is
the projection to use when the inner type must cross thread
boundaries. State-family effect types (Phase 3 step 6a) use
`RefCountedPointer::Of` for the unified single-thread / multi-
shot type surface; the Arc family then needs per-method `Send

- Sync`bounds at smart-constructor sites (the
[2026-05-03 active blocker](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#active-blockers))
to satisfy`ArcCoyoneda`'s dispatch impl bounds.

This is a recurring theme: brand-level `SendFunctor` impls on
types whose projection is `<P as RefCountedPointer>::Of<...>`
hit the per-`A` HRTB-over-types wall on stable Rust. The
established workaround is to push `Send + Sync` bounds to per-
method use sites (matching `ArcRunExplicit`'s precedent).

### `<P as ToDynCloneFn>::new(closure)` is the construction path for `<P as RefCountedPointer>::Of<dyn Fn>` (load-bearing for FnBrand-parameterised effect types)

`<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(A) -> B>` is the
projection for cloneable function pointers parameterised by
the substrate brand `P` (e.g., `RcBrand`, `ArcBrand`). To
construct a value of this type from a sized closure, **use
[`<P as ToDynCloneFn>::new(closure)`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/classes/to_dyn_clone_fn.rs)**,
not `Rc::new(closure)` / `Arc::new(closure)` directly. The
direct constructor returns `Rc<{closure_type}>` (a sized
inner) which does NOT match the `Rc<dyn Fn>` projection;
`ToDynCloneFn::new` performs the unsized coercion.

This pattern is load-bearing for Phase 3 step 6+'s smart-
constructor implementations: any effect type carrying
`<P as RefCountedPointer>::Of<'_, dyn Fn(...) -> ...>`
continuations must be constructed via `ToDynCloneFn::new`.
Direct pointer construction will produce a type-mismatch
error.

### `#[document_examples]` rejects trivial assertions (post-`3a5a0a8`)

The `#[document_examples]` macro now rejects six trivially-
true assertion patterns: `assert!(true)`, `debug_assert!(true)`,
`assert_eq!(true, true)`, `assert_eq!((), ())`,
`assert_ne!(true, false)`, `assert_ne!(false, true)`. Even if a
code block contains a meaningful assertion alongside a trivial
one, the trivial form is treated as noise and rejected.

When writing new doctests, every code block must contain at
least one **meaningful** assertion that verifies the example's
expected output. For Drop tests, construct a post-drop value
and assert via `resume()` / `evaluate()`. For uninhabited types,
use `core::mem::size_of` (size-0). For dispatch-internal
methods, exercise via the user-facing wrapper method
(e.g., `*Run::interpret`) rather than calling the internal
method directly.

### `#[document_parameters("...")]` cannot annotate impl blocks with no methods that take a receiver (load-bearing for new constructor-only impl blocks)

The `document_parameters` attribute requires the impl block to
contain at least one method with a `&self` / `&mut self` /
`self` receiver. Constructor-only impl blocks (containing only
associated functions like `Type::new`, `Type::pure`, etc.) must
omit `document_parameters`; use `document_type_parameters`
alone. The macro errors with "document_parameters cannot be
used on impl blocks with no methods that have receiver
parameters" when this constraint is violated.

When adding a new impl block for smart constructors that take
no `self`, omit `document_parameters` from the impl-block
attributes. The methods themselves can still carry
`document_parameters` for their non-`self` parameters.

### `mod inner { ... }` brings inner items out of scope for module-level (`//!`) doc-link references

When wrapping a module's items in `#[fp_macros::document_module]`

- `mod inner { ... }`, the items defined inside become
  inaccessible to module-level (`//!`) doc-comment references via
  their bare names. Module docs that previously referenced
  `[`Foo`]` need to be rewritten with full crate paths
  (e.g., `[`Foo`](crate::types::effects::foo::Foo)`). Per-item
  docs inside the inner module continue to work without paths
  since they're inside the same scope as the items they
  reference.

This is a one-time migration cost when adopting
`document_module` for an existing module. Worth checking module
docs for bare-name doc-links before / after the wrapping.

## Where to start

**Active blocker (2026-05-04):** `ArcCoyoneda`'s algebra-Send-
awareness gap. `ArcCoyoneda` dispatch requires
`EBrand: Functor + SendFunctor` but `SendStateBrand` cannot
honestly implement `Functor`. Four design options surveyed
in [plan.md's Active blockers](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#active-blockers);
no recommendation locked in. **The Arc family integration
tests in `run_state.rs` cannot ship until this resolves.**
The non-Arc family tests (4 of 6 wrappers) and step 5
(`interpret_with_rec`) are unaffected by the blocker.

1. Read [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
   `Current progress` section and the
   [2026-05-04 active blocker](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#active-blockers)
   subsection. Read the "Resolution path" subsection of this
   prompt's resume point above for the implementation paths
   under each of the four options (α / β / γ / c'').
2. **Decide the design.** User decision needed on (α) /
   (β) / (γ) / (c''). My analysis recommends (α): the
   principled extension of Phase 2 step 9's `ArcFree`
   migration; but the user has not yet locked in a choice.
   Once the design is settled, move the active-blocker entry
   to resolutions.md as a new dated resolution with the
   chosen option's rationale.
3. **(Pre-step before step 5):** ship `run_state.rs` per the
   chosen option's "Resolution path" guidance. The working
   draft in `git stash@{0}` covers all six wrappers (~430
   lines, 18 tests). Pop the stash, fix `State<'_, ...>`
   lifetime annotations (the FnOnce-not-general-enough
   error in the original draft was caused by pinning to
   `'static`), and adjust the Arc family tests per the
   chosen option (delete them under γ; rewrite per c''; or
   keep as-is under α/β). Land as a `test(effects):`
   commit.
4. **Step 5 (`interpret_with_rec`):** see "Step 5
   implementation pattern" subsection in the resume point
   above for the full shape. Six per-wrapper inherent methods
   in `run.rs` / `run_explicit.rs` / `rc_run.rs` /
   `rc_run_explicit.rs` / `arc_run.rs` /
   `arc_run_explicit.rs`. Reuse step 4's M-lifetime pinning
   (`'static` for Erased, `'a` for Explicit) and step 3's
   inline per-wrapper dispatch pattern (no
   `DispatchOneHandler` trait). Integration tests in
   `fp-library/tests/run_interpret_with_rec.rs`. Use State
   (end-to-end on the four non-Arc wrappers; on the Arc
   family iff the active blocker resolves) as one of the
   test scenarios.
5. **Steps 6b-6e (`Reader`, `Except`, `Writer`, `Choose`)**
   follow 6a's per-effect / per-wrapper rollout pattern.
   Note: any effect type whose representation includes
   `dyn Fn(...) -> A` continuations (likely `Reader` and
   `Except`) will hit the same `dyn Fn: !Send + !Sync`
   structural problem that drove 6a.4 + 6a.6's
   `SendStateBrand` parallel-brand design. Plan for parallel
   `SendReaderBrand` / `SendExceptBrand` from the start;
   don't try option (b) per-method bounds first. `Choose`
   ships only on the four multi-shot wrappers per the
   2026-05-03 wrapper-parameterization resolution's Q4=ii.
6. Read [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
   section 4.3 (interpreter families) only if you need the
   original commitment context. Sections 4.5 (scoped effects)
   and 4.6 (natural transformations) become relevant for
   Phase 4 / future work.
7. If your step touches type-class impls, brand-level dispatch, or
   `Send + Sync` auto-derive, also skim
   [fp-library/docs/limitations-and-workarounds.md](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/limitations-and-workarounds.md)'s
   "Unexpressible Bounds in Trait Method Signatures" table. Phase
   1 step 7 added rows for the Explicit Free family that record
   where stable Rust's lack of `for<T>` HRTB caps brand coverage.
   The pattern (Pointed at the brand level; `bind`/`map`
   inherent-only; Ref hierarchy as the by-reference dispatch
   path) is the precedent any new wrapper type with shared
   internal state will end up following. Saves rediscovering the
   constraint mid-implementation.
8. Update plan.md's `Current progress` (rolling-detail entry
   for step 5; demote oldest step from rolling-detail to commit
   log per the rolling-detail trim window of ~3 narratives).
   Append deviations.md entry for step 5. Standard end-of-step
   doc maintenance.

## Per-step protocol

For each step you implement:

1. Implement the code, tests, benches, or docs the step requires.
   Use the LSP tool (`rust-analyzer` is wired through MCP, see the
   project's [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md)
   for usage) for type info, go-to-definition, and find-references.
   The Brand-and-Kind machinery and the existing four-variant
   `Coyoneda` family are the long-standing templates the new code
   follows. The recently committed `Free`, `RcFree`, `ArcFree`, and
   `FreeExplicit` modules in
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/`
   are direct structural templates for subsequent variants in the
   Free family (e.g., the outer `Rc<Inner>` wrapping pattern in
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/rc_free.rs`
   and the concrete recursive enum body in
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/free_explicit.rs`
   together inform `RcFreeExplicit`).
2. Run `just verify` (or the individual sub-recipes: `just fmt`,
   `just check`, `just clippy`, `just deny`, `just doc`, `just test`).
3. If verification fails, fix the underlying issue. Do not bypass
   hooks (`--no-verify`, `--no-gpg-sign`) and do not silence
   warnings without addressing them.
4. Update the docs that capture state and history:
   - [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
     `Current progress` section to reflect what now exists.
     Trim older entries per plan.md's
     [`Implementation protocol`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)
     step 3 when the rolling-detail window has grown past
     ~3 narratives (demote the oldest narrative to a
     one-line bullet in the "Earlier completed steps" commit
     log; verify any load-bearing context is preserved in
     deviations.md / resolutions.md / commit message before
     demoting).
   - [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
     (append-only) for any per-step deviation from the original
     plan text. Group entries by phase and step, matching the
     existing structure.
   - If you encounter a blocker, add an entry to plan.md's
     `Open questions, issues and blockers -> Active blockers`
     subsection (see "When you hit something unexpected" below).
     Once the blocker resolves, move the entry to
     [resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md)
     as a new top-level entry, dated; replace the active-blocker
     subsection in plan.md with a one-line summary plus an
     anchor link to resolutions.md.
5. Commit. One step per commit; the commit message describes the
   step. Use conventional-commit prefixes (`feat`, `fix`, `refactor`,
   `test`, `bench`, `docs`, `chore`). Never include `Co-Authored-By`
   trailers.

Do not skip the protocol to "batch" steps; a step is the commit
boundary, even when two steps look small.

**Splitting an oversized step is permitted but exceptional.** If
a numbered step in plan.md is large enough that landing it as
one commit would risk leaving the working tree mid-step on
context exhaustion (rough rule of thumb: ~1500+ new lines, 7+
new files, or multiple new public types with mixed concerns),
you may split it into sub-commits (e.g., 4a foundation, 4b
follow-on) under the following conditions:

1. Surface the scope to the user before starting. Explain what's
   bundled and offer the split as an option; do not split
   unilaterally.
2. The split must be coherent: each sub-commit must compile and
   pass `just verify` independently, and each must be
   independently reviewable.
3. Record the split in
   [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
   under the step's heading, explaining the scope rationale and
   what each sub-commit lands. Phase 2 step 4's split into 4a
   (foundation) and 4b (Explicit family) is the existing
   precedent.

The default remains "one step per commit"; splitting is for
genuinely outsized steps, not for convenience.

## When you hit something unexpected

The plan and decisions are frozen. You do not have authority to
change them unilaterally. If you encounter:

- **A step that doesn't make sense given the current code state.**
  Stop. Add an entry under
  `Open questions, issues and blockers -> Active blockers` in
  [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)
  describing what's unclear, commit that single edit, and report
  back to the user. Do not invent an interpretation.
- **A genuine design conflict** (a decision in
  [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
  is incompatible with what stable Rust permits, with the existing
  fp-library code, or with another decision). Same protocol: record
  it under
  `Open questions, issues and blockers -> Active blockers` in
  plan.md, commit, report back. Do not edit
  [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
  yourself.
- **A simpler way to do something** (refactor opportunity, missing
  abstraction, etc.). If it is in scope for the step, do it inline.
  If it would expand the step's scope or touch unrelated code, note
  it under
  [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
  or as a follow-up `chore:` commit; do not silently expand the step.
- **Unexpected files, branches, or in-progress work.** Investigate
  before deleting or overwriting. The user's local state is real and
  may be load-bearing; ask before discarding it.

## Boundaries

- **`/home/jessea/Documents/projects/rust-fp-lib/fp-library/` is the
  production crate.** Code, tests, and benches go here.
- **`/home/jessea/Documents/projects/rust-fp-lib/fp-macros/` holds
  proc-macros.** The effects-subsystem macros live in
  `/home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/`.
  Already shipped: `im_do!` ("Inherent Monadic do") at
  [`im_do/codegen.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/im_do/codegen.rs)
  (Phase 2 step 7c.2b); `effects!` (public, Coyoneda-wrapped
  row) and `raw_effects!` (internal, un-wrapped row) at
  [`effects_macro.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/effects_macro.rs)
  with the shared lexical-sort helper at
  [`row_sort.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/row_sort.rs)
  (Phase 2 step 8); `handlers!` at
  [`handlers.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/handlers.rs)
  (Phase 3 step 1, commit `82dd7bb`). Pending:
  `define_effect!` (Phase 3 step 6 / 7 depending on blocker
  resolution), `define_scoped_effect!` (Phase 4),
  `scoped_effects!` (Phase 4 step 4) — all land in the same
  directory. `ia_do!` ("Inherent Applicative do") is
  forward-reserved as a future applicative companion to
  `im_do!`. The shared `DoInput` parser used by all four
  do-notation macros (`m_do!`, `a_do!`, `im_do!`, future
  `ia_do!`) lives at
  `/home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/support/do_input.rs`
  (Phase 2 step 7c.2a).
- **Documentation lives in
  `/home/jessea/Documents/projects/rust-fp-lib/docs/`.** Do not
  invent new top-level docs without an explicit step asking for
  them. Phase 5 step 4 schedules
  `/home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/run.md`.
- **Out-of-scope items in
  [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
  `Out of scope` section** are off-limits. Surveying alternatives,
  prototyping evidence-passing, exploring tag-based type-level
  sorting, etc. are not part of this implementation effort.

## Project conventions

- **Hard tabs for Rust indentation.** The project's
  `/home/jessea/Documents/projects/rust-fp-lib/rustfmt.toml` uses
  hard tabs. When using the Edit tool, the `old_string` must match
  the file's tab characters exactly. Do not fall back to `sed`,
  `awk`, or `python` to edit whitespace.
- **No em-dashes, en-dashes, or `--` as a dash substitute.** Use
  commas or semicolons. Hyphenated words are fine.
- **No emoji or unicode symbols** in code, comments, or docs. ASCII
  only: `->`, `<-`, `>=`, `!=`, plain dashes for dividers.
- **Always end bullet points with proper punctuation.**
- **Conventional commit prefixes** (`feat`, `fix`, `docs`,
  `refactor`, `bench`, `test`, `chore`). No `Co-Authored-By`
  trailers.
- **Default to writing no comments.** Comment only when the _why_
  is non-obvious (a hidden invariant, a workaround for a specific
  bug, behavior that would surprise a reader). Never reference the
  current task, fix, or callers in comments.
- **No backwards-compatibility shims, dead code preservation, or
  removed-code comments.** Delete what is no longer used.

## Common gotchas from prior steps

These bit prior steps repeatedly. Internalising them up front saves
debug cycles.

- **Stage new files before `just verify`.** Untracked files do not
  invalidate the test-output cache, so a green verify on untracked
  code is not trustworthy. After creating new files, run `git add`
  before `just verify`. If verify reformats existing files (via
  `treefmt`), `git status` will show `MM` on the staged file; re-stage
  with `git add` before retrying the commit.
- **`#[document_examples]` requires a real Rust code block.** It
  rejects `\`\`\`ignore`, `\`\`\`text`, and other non-Rust fences.
If no working example exists for a method whose impl depends on
scaffolding from a later step, options are: (a) add a working
example that uses an existing brand which already supports the
trait (e.g., `OptionBrand`for the`Send\*`family); (b) provide
a small one-off impl alongside so the example compiles; (c)
remove the macro and use plain`# Examples`markdown, but the
resulting deprecation warning is escalated by`-D warnings`in`just clippy`, so this only works after careful suppression.
- **Inherent-method bounds do not propagate into trait impl
  bodies.** When implementing a brand-level type-class trait by
  delegating to an inherent method (e.g.,
  `RcFreeExplicitBrand::bind` -> `RcFreeExplicit::bind`), the
  inherent method's `where A: Clone, F::Of<...>: Clone` bounds are
  not in scope inside the trait method body. Stable Rust does not
  let you add per-method `where` bounds beyond what the trait
  declares (no HRTB-over-types). When this hits, the right move
  is usually documenting the brand-level coverage gap (see the
  [`limitations-and-workarounds.md`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/limitations-and-workarounds.md)
  precedent) and routing through the Ref hierarchy where possible,
  not fighting the constraint.
- **`Free<IdentityBrand, A>` is layout-cyclic.** `Free`'s `Wrap`
  arm holds `F::Of<Free<F, TypeErasedValue>>` where
  `TypeErasedValue = Box<dyn Any>`. For `IdentityBrand`,
  `Identity<T>` is `T` with no indirection, so layout recursion
  has no termination and rustc rejects with
  `error[E0391]: cycle detected when computing layout`. Tests
  and benches that wrap Free over an identity-shaped functor
  must use `ThunkBrand` instead (`Thunk<A>` holds a boxed
  closure, providing the indirection). The Rc/Arc Erased family
  escapes via outer `Rc<Inner>` / `Arc<Inner>` wrapping; the
  Explicit family escapes via `Box<...>` in `FreeExplicit`'s
  `Wrap` arm or the same outer wrapping for the Rc/Arc Explicit
  variants. See deviations.md's Phase 1 step 8 entry.
  **For Run-shaped programs**: `Run<R, S, A>` (over `Free`) hits
  this cycle when `R` has a no-indirection head (e.g.,
  `IdentityBrand`); use `CoyonedaBrand`-headed rows for `Run`'s
  doctests/tests. `RcRun` / `ArcRun` / all three Explicit
  variants escape via their respective outer-pointer or
  `Box`-in-Wrap indirection, so `IdentityBrand`-headed rows
  work for them.
- **HRTB on a GAT projection poisons normalization in scope.**
  Discovered while implementing `ArcRun::send` in Phase 2 step 5. When a struct, impl block, or function's where-clause
  carries an HRTB on a generic associated type at a specific
  instantiation (e.g.,
  `NodeBrand<R, S>: Kind<Of<'static, ArcFree<...>>: Send + Sync>`,
  which `ArcFree`'s struct propagates to every `ArcRun`
  impl-block context), rustc refuses to normalize that GAT at
  _other_ instantiations in the same scope. So a literal
  `Node::First(layer)` cannot be unified with
  `<NodeBrand<R, S> as Kind>::Of<'_, A>` even though
  `impl_kind!` declares them equal. The trigger is the HRTB
  itself, not the substrate: PhantomData-only structs with the
  HRTB hit it; free functions carrying the HRTB hit it;
  cross-substrate calls (e.g., `RcFree::lift_f` from inside an
  `ArcFree`-HRTB-bearing impl) hit it. **Workaround**: receive
  projection-typed values as parameters; never construct
  projection-typed values inside an HRTB-bearing scope. The
  caller (typically test code, smart-constructor macro output,
  or top-level concrete-type code with no HRTB in scope) builds
  the projection literal and passes it in. The probe file
  [`fp-library/tests/arc_run_normalization_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/arc_run_normalization_probe.rs)
  documents four passing patterns and is the regression-test
  home for this limit. This is the design driver for
  `*Run::send` taking the `Node`-projection value (rather than
  the row-variant layer) symmetrically across all six Run
  wrappers.
- **The Wrap-depth probe at
  [`fp-library/tests/run_wrap_depth_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs)
  is a regression test guarding the `WrapDrop` resolution.** It
  measures structural Wrap depth across Run-shaped Free
  programs and documents that Run-typical patterns have
  structural depth at most 1, which is the property the
  `WrapDrop::None` policy relies on for soundness (effect-row
  brands like `CoyonedaBrand` / `CoproductBrand` / `NodeBrand`
  all return `None` from `WrapDrop::drop` because they do not
  materially store the inner Free; `Drop` then falls through to
  recursive drop on the layer, which is sound only as long as
  the structural Wrap depth stays bounded). If a future Phase
  2-4 change appears to invalidate the probe (e.g., new patterns
  emit deeper structural Wrap chains), pause and re-evaluate
  before shipping; the probe finding is load-bearing for the
  no-`Extract`-bound semantics.
- **`Kind!(...)` macro invocations inside `Apply!(...)` do not
  require `use fp_macros::Kind;` to be in scope.** `Apply!` is a
  procedural macro that parses the inner `Kind!(...)` syntax
  itself; the inner macro never gets invoked as a real macro,
  so rustc's unused-import analysis flags the import as dead.
  Some older test files used to carry an
  `#[expect(unused_imports)] use fp_macros::Kind;` shim to
  suppress the warning; that shim is no longer needed and was
  removed during Phase 2 step 4a (commits `9adabd5` and
  `c3712f6`). When writing a test file that uses
  `Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)`
  patterns, do not import `Kind` from `fp_macros`. If you
  separately call `<F as Kind!(...)>::Of<...>` outside an
  `Apply!` (rare; the explicit form bypasses `Apply!`), then
  the import is needed.

## Bench and compile_fail test infrastructure

Many phases add benches or `compile_fail` UI tests. The
infrastructure pointers below apply across phases; reach for
them whenever a step asks for benchmarking or negative-case
testing.

- **Criterion benches** go in
  [`fp-library/benches/benchmarks/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks/).
  Existing per-variant Free benches
  ([`free.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks/free.rs),
  [`free_explicit.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks/free_explicit.rs),
  etc.) are the baseline shape. Wire new bench files into the
  `criterion_group!` registration in
  [`fp-library/benches/benchmarks.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks.rs).
- **`compile_fail` UI tests** go in
  [`fp-library/tests/ui/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/ui/).
  The
  [`fp-library/tests/compile_fail.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/compile_fail.rs)
  driver registers them via `trybuild::TestCases::new().compile_fail("tests/ui/*.rs")`,
  and `trybuild = "1.0"` is already in
  `fp-library/Cargo.toml`. Each negative case is one `.rs` file
  plus a sibling `.stderr` capturing expected error output;
  `.stderr` files are auto-generated on first run via
  `TRYBUILD=overwrite cargo test --test compile_fail` (use raw
  `cargo`, not `just test`, when bootstrapping `.stderr` files
  so the wip files do not persist under `fp-library/wip/`).
- **Probe / investigation tests** can also live in
  [`fp-library/tests/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/).
  Existing examples include
  [`run_wrap_depth_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs)
  (regression-guards a property load-bearing for the WrapDrop
  resolution) and
  [`free_explicit_poc.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/free_explicit_poc.rs)
  (integration-tests `FreeExplicit` against the questions the
  POC originally asked). Use the same shape when a step's work
  benefits from a self-documenting investigation as a test.

## Tooling

- All build / test / lint commands go through `just` (the project
  has a
  [justfile](file:///home/jessea/Documents/projects/rust-fp-lib/justfile)
  that handles the Nix environment). Examples: `just verify`,
  `just test`, `just clippy`, `just doc`.
- For one-off `cargo` commands not in the justfile, prefix with
  `direnv allow && eval "$(direnv export bash)" && cargo ...` so
  the project's Nix toolchain is used. Do not silence direnv errors
  with `2>/dev/null`.
- The LSP tool (`rust-analyzer` via MCP) is the right tool for type
  info on generic-heavy code: `LSP` with `operation: "hover"`,
  `"goToDefinition"`, `"findReferences"`, `"goToImplementation"`,
  etc. See the project's
  [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md)
  for examples. Reach for it whenever you would otherwise be tracing
  trait bounds by hand across multiple files.

## Done condition for one run

You can either:

- **Complete one phase end-to-end** (every numbered step ticked,
  `just verify` clean, `Current progress` reflects the new state)
  and stop. The user reviews and starts the next phase.
- **Complete a focused follow-up commit set** (e.g., the Phase 1
  follow-up `WrapDrop` migration's two commits, or the
  Phase 2 step 4a/4b split's two commits) and stop. The user
  reviews before proceeding to the next phase step that the
  follow-up unblocks.
- **Stop at the first blocker** you cannot resolve under the
  protocol above. Commit the active-blocker entry under
  plan.md's
  `Open questions, issues and blockers -> Active blockers`,
  summarise the blocker, and exit.

Do not work through multiple phases unprompted. Phases ship together
as a single feature release, but they review separately.

## Reference map

The four-corner doc taxonomy:

- [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md):
  the active working spec. Phased steps, current progress, active
  blockers, success criteria. The authoritative answer to "what do
  I do next."
- [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md):
  frozen design rationale. The authoritative answer to "why this
  way." Do not edit.
- [resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md):
  append-only post-write log of resolved blockers. Holds full
  problem statements, investigations, alternatives considered,
  and rationale for each load-bearing question that paused
  implementation. Read this when plan.md's `Active blockers`
  section points at it for context, or when "why does X work this
  way?" cannot be answered from decisions.md alone.
- [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md):
  append-only post-write log of per-step implementation choices
  that diverged from the plan text. Grouped by phase and step.
  Read this when "the code doesn't match the step description"
  needs explanation; append a new entry when your own work
  diverges.

Other reference material:

- [research/](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/research/):
  per-codebase classifications, three Stage 2 deep dives, and a
  synthesis. Source material for the decisions.
- [type-level-sorting/research/](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/type-level-sorting/research/):
  the parallel research arc on type-level sorting. Cited from
  decisions section 4.1.
- `poc-effect-row/`: standalone Cargo workspace with the
  row-encoding hybrid POC. Migrated to
  [`fp-library/tests/run_row_canonicalisation.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_row_canonicalisation.rs)
  in Phase 2 step 10a; workspace deleted in step 10b. The
  preserved findings live in
  [`docs/plans/effects/poc-effect-row-canonicalisation.md`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/poc-effect-row-canonicalisation.md).
- [fp-library/tests/free_explicit_poc.rs](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/free_explicit_poc.rs):
  import-based integration tests for the production `FreeExplicit`.
  The POC promotion is complete (Phase 1 step 1); the file now
  exercises the type imported from
  `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/free_explicit.rs`.
- [fp-library/tests/run_wrap_depth_probe.rs](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs):
  regression test for the property the WrapDrop resolution relies
  on (Run-typical structural Wrap depth at most 1). Background
  investigation, see resolutions.md's "Resolved (2026-04-27): introduce WrapDrop trait..."
  entry.
- [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md):
  project-wide agent instructions including LSP usage.
- [AGENTS.md](file:///home/jessea/Documents/projects/rust-fp-lib/AGENTS.md):
  broader agent contract for this repo.
