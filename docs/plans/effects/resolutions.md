# Resolved blockers: effects port

This file is the post-write log of blockers and load-bearing
questions that surfaced during implementation of the effects
port and how they were resolved. Each entry is dated and stays
append-only; entries are added when blockers resolve, never
edited or removed.

The file complements [decisions.md](decisions.md) (frozen
design rationale) and [plan.md](plan.md) (active phasing and
status). Use this file when you want context on "why does X
work this way?" or "what alternatives were considered for Y?".

For per-step deviations from the original plan (smaller-grain
implementation differences that didn't require a paused
investigation), see [deviations.md](deviations.md).

## Resolved (2026-05-07): Phase 4 step 3.1.3 Catch action-field layout cycle (B7) closed

**Disposition.** B7 surfaced during Phase 4 step 3.1.3's `Run::catch` smart constructor implementation: the doctest tripped a Rust layout-cycle error tracing through `Free<NodeBrand<R, S>, TypeErasedValue>` -> `<S>::Of<'_, Free<...>>` -> `BoxCatch<..., Free<...>>` -> unboxed `action: Free<...>` field -> `Free<...>` layout (cycle). Adopted **B-thunk with unit-arg `Fn(()) -> A` form** on user confirmation in this session.

### B7. `Catch` action-field layout cycle when embedded in substrate

- **Issue.** [`BoxCatch`](../../../fp-library/src/types/effects/catch.rs) / [`Catch`](../../../fp-library/src/types/effects/catch.rs) / [`SendCatch`](../../../fp-library/src/types/effects/catch.rs) shipped in step 3.1.1 with `action: A` (unboxed) per a literal reading of [decisions.md section 4.5](decisions.md)'s "carry the action and handler payload" sketch. The layout-cycle implication only surfaces at substrate-embedding time (3.1.3 smart constructors), not when the effect type is exercised in isolation (3.1.1 / 3.1.2 doctests). The cycle is broken by any pointer-sized indirection on the `action` field; the question is which.
- **Resolution: B-thunk (per-pointer-brand pointer of unit-arg `Fn(()) -> A` thunk).** `BoxCatch` action becomes `<P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>` (single-shot via `Box<dyn FnOnce>`); `Catch` action becomes `<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>` (multi-shot via `Rc<dyn Fn>`); `SendCatch` action becomes `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>` (thread-safe via `Arc<dyn Fn + Send + Sync>`). The unit-arg `Fn(()) -> A` form (vs zero-arg `Fn() -> A`) lets the existing pointer-abstraction [`ToDynFnOnce::new`](../../../fp-library/src/classes/to_dyn_fn_once.rs) / [`ToDynCloneFn::new`](../../../fp-library/src/classes/to_dyn_clone_fn.rs) / [`ToDynSendFn::new`](../../../fp-library/src/classes/to_dyn_send_fn.rs) family construct both action and handler fields uniformly. The matrix in [`pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md)'s `(pointer-capability, closure-semantic)` table now has the action field's pointer-and-closure-trait combination matching the handler field's existing pointer-and-closure-trait combination per Catch sibling, parallel to the user-API-facing per-pointer-brand pattern. Call sites invoke as `action(())` and construct as `<P as ToDyn*Fn>::new(move |_: ()| value)`. Functor / SendFunctor / RefFunctor impls compose thunks lazily (no `A: Clone` constraint) by wrapping `f` in an `Rc<dyn Fn(A) -> B>` (or `Arc<dyn Fn(A) -> B + Send + Sync>` for SendCatch) shared between the new action and new handler closures so each can call `f` once via Rc/Arc-deref. The catch dispatcher (step 7) calls the thunk to materialise the action program at execution time.
- **Options considered and rejected.** _Option A (uniform `Box<A>`):_ pointer-sized layout breaks the cycle but pays a real perf cost on multi-shot wrappers (deep-clone vs refcount-bump). _Option B-literal (per-pointer-brand pointer for raw `A`: `Rc<A>` / `Arc<A>`):_ faithful per-pointer-brand pattern but `Functor::map` extraction `A` from `Rc<A>` requires `A: Clone` which can't be added to the trait method's signature; multi-shot path's `to_view` clones produce shared Rc layers at every map call site, so try-unwrap-or-panic would fire in `Choose` + `Catch` programs. _Option C (`BoxedAction<A>` wrapper):_ cosmetic-only; adds a public type for a fix that doesn't need one. _Option D (defer + redesign):_ disruptive given 3.1.1 / 3.1.2 already shipped.
- **Plan-text amendment.** [`catch.rs`](../../../fp-library/src/types/effects/catch.rs)'s three Catch sibling enums updated per the B-thunk spec; impl bodies (Functor, SendFunctor, RefFunctor, WrapDrop, Extract, brand-projection helpers, doctests) revised to match. Manual `Clone` impls added for `Catch<'a, P, E, A>` and `SendCatch<'a, P, E, A>` (refcount-bump on action and handler pointers; no `A: Clone` requirement). `BoxCatch` does NOT impl `Clone` (Box<dyn FnOnce> is structurally uncloneable). The brand-projection helper `box_catch_action_ref` from step 3.1.2 was removed (the BoxCatch RefFunctor impl is now stub-everywhere because Box<dyn FnOnce> can't be invoked through a reference); `catch_action_ref` was renamed to `catch_action_thunk_ref` and returns `&Rc<dyn Fn(()) -> A>` (the thunk reference) for the CatchBrand RefFunctor impl's thunk-composition path. ArcRun gains a `make_node_scoped` HRTB-poisoning workaround helper paralleling the existing `make_node_first` (the existing `wrap_first_arc` is reusable for both First and Scoped paths since its body just forwards to `ArcFree::wrap`).

## Resolved (2026-05-07): Phase 4 step 3.1 sub-step splitting; B5 + B6 closed

**Disposition.** Two coupled blockers that surfaced during Phase 4 step 3.1 foundational implementation closed on user confirmation in this session: B5 (`RefFunctor` GAT-normalization on scoped-effect closure-cell brands) adopted Option A (brand-projection helpers); B6 (step 3.1 sub-step splitting) adopted Option B (4-commit split). Step 3.1.1 shipped at `abd3d1a3`; step 3.1.2 lands the `RefFunctor` work in this commit. Steps 3.1.3 (smart constructors) and 3.1.4 (integration tests) follow.

### B5. `RefFunctor` GAT-normalization for scoped-effect closure-cell brands

- **Issue.** The trait method [`RefFunctor::ref_map`](../../../fp-library/src/classes/ref_functor.rs) takes `fa: &<Self as Kind>::Of<'a, A>`. Inside an impl scope, the compiler refuses to unify that reference with the concrete enum type (e.g., `&BoxCatch<'a, BoxBrand, E, A>`) because GAT projections do not normalize through reference parameters in trait method bodies, even with explicit `let fa: &BoxCatch<...> = fa;` annotation. Pattern matching on the variants therefore fails to type-check. [`Identity`](../../../fp-library/src/types/identity.rs) sidesteps this via tuple-struct field access (`fa.0`), which doesn't require a match; sum types like [`Catch`](../../../fp-library/src/types/effects/catch.rs) have no equivalent workaround in stable Rust.
- **Resolution: Option A (brand-projection helper).** Land `#[doc(hidden)]` free functions outside the impl scope whose where-clauses carry only `Kind` bounds; the function body's pattern match normalizes cleanly because no HRTB context is in scope. Mirrors [`arc_run::unwrap_first`](../../../fp-library/src/types/effects/arc_run.rs)'s precedent. Three helpers ship in 3.1.2 (`box_catch_action_ref`, `catch_action_ref`, `catch_handler_ref`); steps 3.2-3.4 will land additional helpers per scoped-effect family. Option B (`unsafe { core::mem::transmute(fa) }`) was rejected: technically equivalent but introduces `unsafe` audit burden for what is structurally safe under [`impl_kind!`](../../../fp-macros/src/hkt/impl_kind.rs)'s expansion guarantees. Option C (substrate redesign dropping `RefFunctor` from the scoped-row trait surface) was rejected: reduces by-reference traversal capability and forces a non-local change to [`scoped.rs`](../../../fp-library/src/types/effects/scoped.rs)'s "all five required" claim. Option D (skip `RefFunctor` impls, accept compile-fail) was rejected: by-reference traversal is more pervasive than the SendCatchBrand-no-Functor case, so user programs would compile-fail with cryptic trait-bound messages. Step 3.1.2 also discovered that `SendCatchBrand` does not need `RefFunctor` because the cascade through [`ArcRunExplicitBrand`](../../../fp-library/src/brands/effects.rs) does not require it (`ArcFreeExplicitBrand: !RefFunctor` per the brand's docs); a hypothetical impl would face the same `Send + Sync` bound mismatch on `func` that prevents `Functor`. Logged at [deviations.md Phase 4 step 3.1.2](deviations.md), mirroring the [`SendCatchBrand`-no-`Functor` precedent](deviations.md). The `BoxCatch` impl uses an `unreachable!`-stub recovery handler (suppressed via `#[expect(clippy::unreachable, reason = "...")]`) because `Box<dyn FnOnce>` cannot be replicated through a reference; the path is structurally unreachable in real programs since [`RunExplicitBrand: RefFunctor`](../../../fp-library/src/types/effects/run_explicit.rs#L1742) is reachable only through synthetic non-Coyoneda rows. The `Catch` impl is faithful (Rc-clone the handler, share `func` via `<RcBrand as ToDynCloneFn>::ref_new`).
- **Plan-text amendment.** Phase 4 step 3.1 gains the per-sub-step labels per B6's Option B; the `RefFunctor` deferral language was removed from the Phase status / Next greenfield work paragraphs once 3.1.2 shipped.

### B6. Step 3.1 sub-step splitting given the B5 deferral

- **Issue.** The original step 3.1 plan committed to one bundled commit (per the 2026-05-06 step-3 splitting resolution) covering the Catch sibling types + brands + 5 trait impls + smart constructors + tests. The B5 deferral surfaced internal sub-step boundaries not visible at planning time: the foundational scaffold (4 of 5 trait impls) is mechanical from Phase 3.5's State template, while the `RefFunctor` work requires new helper-machinery design (per B5 Option A); smart constructors and tests are separate concerns again.
- **Resolution: Option B (4-commit split).** Step 3.1 splits into 3.1.1 (foundational scaffold; shipped at `abd3d1a3`), 3.1.2 (`RefFunctor` impls + brand-projection helpers; this commit), 3.1.3 (smart constructors per wrapper), 3.1.4 (integration tests). Mirrors step 2's empirically-validated splitting clause (step 2 split into 2.1-2.6 when per-wrapper risk surfaced). Option A (single bundled commit) was rejected for context-budget cost and review surface size. Option C (2-commit split with smart constructors and tests bundled into 3.1.1 using only 4 of 5 traits) was rejected because programs using `NodeBrand<R, S>: RefFunctor` with `S` containing Catch brands would fail to compile until the deferred `RefFunctor` follow-up landed; 3.1.1 would ship in a temporarily-incomplete state.
- **Plan-text amendment.** Step 3 sub-step labels updated: `3.1 Catch` becomes `3.1.1 Catch foundational scaffold` / `3.1.2 Catch RefFunctor + brand-projection helpers` / `3.1.3 Catch smart constructors` / `3.1.4 Catch integration tests`. Mirror split available for 3.2 (Local/RefLocal), 3.3 (Bracket/RefBracket), 3.4 (Span) as design risk surfaces, or simpler if it doesn't.

## Resolved (2026-05-06): Phase 4 implementation-kickoff sequencing K1 and K2 (POC 3 standalone commit first; plan.md numbering authoritative for commit boundaries)

**Disposition.** Two implementation-kickoff sequencing decisions surfaced at the Phase 3.5-to-Phase-4 transition (commit `2e97e812`); both adopted Option A on user confirmation in this session. Closes the previously-active `Phase 4 implementation-kickoff sequencing` subsection in plan.md and unblocks substantive Phase 4 implementation work. Q4, R1, R2, R3 mitigations remain inline in plan.md's [Phase 4 implementation prototypes and risk mitigations](plan.md#phase-4-implementation-prototypes-and-risk-mitigations) subsection, scheduled to land during R1 implementation kickoff (Q4 / R1 / R2 prototypes) or alongside the standard scoped-effect rollout (R3 benchmark commit).

### K1. POC 3 (`interpret_with_either`) validation ordering

- **Issue.** Plan.md commits to a new substrate primitive `interpret_with_either<EBrand, Idx>(self, fo_handlers: &impl DispatchHandlers<...>) -> Either<A, EBrand::Op>` on each Run wrapper, used by the `Catch` cons-cell impl in [Phase 4 step 4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row). The primitive's POC validation on `RcRun` (POC 3) "must land before the step that introduces `interpret_with_either` ships generically across all six Run wrappers", but the literal commit ordering relative to other Phase 4 substrate work (steps 1, 2, the `Span` cons-cell) was unspecified.
- **Resolution: Option A.** POC 3 lands as a standalone commit at [`fp-library/tests/poc_rc_run_interpret_with_either.rs`](../../../fp-library/tests/) before any other Phase 4 substrate work. Mirrors POC 1 ([`poc_send_catch_brand.rs`](../../../fp-library/tests/poc_send_catch_brand.rs)) and POC 2 ([`poc_rc_run_interpose.rs`](../../../fp-library/tests/poc_rc_run_interpose.rs)) precedent (each shipped as a standalone validation commit before its generic rollout). The half-day cost is amortised across Phase 4's 1-2-week budget for Sequencing Plan item 3. Option B (mixed-layer paired commit with the Catch cons-cell substrate primitive) was rejected because if POC 3 surfaces a wall, the entire `Catch` cons-cell design is blocked mid-Phase 4 and earlier non-trivial commits (Span cons-cell, dispatcher trait skeleton) would stand against a now-broken design. Option C (skip POC 3, inline rollout) was rejected because it removes the validation step entirely; the [R1 risk](plan.md#r1-explicit-family-interpose-generalisation) of HRTB-poisoning on the Explicit family makes this riskier than the half-day POC investment.
- **Plan-text amendment.** Phase 4 gains a new step 0 before step 1: "POC 3 validation: `interpret_with_either<EBrand, Idx>` substrate primitive on `RcRun` at [`fp-library/tests/poc_rc_run_interpret_with_either.rs`](../../../fp-library/tests/), paralleling POC 1 / POC 2. Mechanical from [`interpret_with`'s body](../../../fp-library/src/types/effects/run.rs#L885-L900) with one branch substitution. Generic rollout across all six Run wrappers ships in step 2a after POC 3 validates."

### K2. Plan.md step numbering vs Sequencing Plan item numbering

- **Issue.** Phase 4 work was described twice with different boundaries. Plan.md numbered the steps 0, 1, 2, 2a, 3, 4, 5, 6, 7, 8 (after K1 adoption); the [Sequencing Plan in the remediation report](review/1_scoped_effects_design/remediation_proposals_phase_4.md#sequencing-plan) numbers items 1-9. The two schemes overlap differently: Sequencing items 1, 2, 4 are pre-implementation doc commits already shipped; item 3 spans plan.md steps 1, 2, 4 (the `Catch`/`Span` cons-cell slice); items 5-9 map to plan.md steps 3-8 with similar non-bijective mapping. The prompt's per-step protocol cited "one step per commit" without specifying which numbering.
- **Resolution: Option A.** Plan.md step numbering is the authoritative commit boundary. Each plan.md step (0, 1, 2, 2a, 3, 4, 5, 6, 7, 8) becomes one commit (or a small bundled-commit group with surfaced split per the per-step protocol's "splitting an oversized step" clause). The Sequencing Plan items group these commits but do not replace them. Mirrors Phase 3 step 5a precedent (sub-step splits within a single plan.md step). Option B (Sequencing Plan numbering authoritative) was rejected because Sequencing items 3, 5, 7 are 1-2-week each per the report's estimate and bundled commits clash with the per-step protocol; Option C (hybrid) was rejected because two parallel numbering schemes confuse future implementors.
- **Plan-text amendment.** No structural revision (semantic-only decision). Sequencing Plan numbering remains in the remediation report as planning context; plan.md is the authoritative source for commit boundaries. Implementer protocol: `git log` shows commits per plan.md step; for time-budget questions the Sequencing Plan in the remediation report is the right reference.

## Resolved (2026-05-06): Phase 3 prior-review F4 closed structurally via Phase 3.5 retrofit (sibling `Box*Brand` family on default Run substrates)

**Question.** The Phase 3 prior-review's
[F4 finding](review/0_first_order_effects_implementation/remediation_proposals.md#f4-the-single-shot-vs-multi-shot-property-promised-per-wrapper-is-not-enforced-at-the-effect-instance-level)
flagged that the "single-shot vs multi-shot" property advertised
per Run wrapper applied only to the Free-spine consumption, not to
per-effect closure cells. Concretely: on
[`Run`](../../../fp-library/src/types/effects/run.rs) and
[`RunExplicit`](../../../fp-library/src/types/effects/run_explicit.rs)
(both single-shot per the wrapper-level guarantee), the smart
constructors `get` / `put` / `ask` stored continuations as
[`StateBrand<RcBrand, A>`](../../../fp-library/src/brands/effects.rs)
/
[`ReaderBrand<RcBrand, A>`](../../../fp-library/src/brands/effects.rs)
whose closure cell is `Rc<dyn Fn(...) -> A>`. The cell was thus
multi-shot-callable at the effect-instance level even though the
surrounding program was single-shot, creating a semantic asymmetry:
a handler could call the State continuation many times even on a
wrapper advertised as single-shot. The review's F4 framing called
this "API claim does not match reality at the effect-instance
level".

**Original Phase 3 disposition (step 8, 2026-05-05):** Phase 3
step 8 adopted F4's recommended **Option A** (documentation-only):
weaken the [Success criteria](plan.md#success-criteria)'s
"single-shot vs multi-shot" claim to apply to Free spine
consumption only, and document at
[`StateBrand`'s rustdoc](../../../fp-library/src/types/effects/state.rs#L24-L29)
that per-effect closures carry their multi-shot property at the
effect-instance level on every wrapper. Phase 3 closed under this
option.

**Phase 3.5 re-opening rationale.** Phase 4 design-question B3 (the
[Phase 4 pre-implementation design questions](#resolved-2026-05-05-phase-4-pre-implementation-design-questions-b1-b4-q1-q3-q5-closed-by-design-adoption-commit-6e960701))
re-examined the F4 framing in the context of user-supplied scoped
handlers. The key observation: Phase 3's State / Reader / Choose
continuations are _substrate-constructed_ (the smart constructor
emits a trivial `|s| s` or `|()| ()` closure), so the mismatch
surfaces only inside the substrate code at handler dispatch.
Phase 4's scoped-effect handlers (the body of `Catch::handler`,
`Local::modify`, `Bracket::body`, etc.) are _user-supplied_, so the
FnOnce-ergonomics friction would land on user code: a natural
recovery handler is `move |e: MyErr| recovery_built_from_captures`
where the captures are consumed in building the recovery program.
That is `FnOnce`, not `Fn`. Forcing it through `Rc<dyn Fn>` would
reject the natural pattern at the type level on default `Run`,
requiring users to wrap captures in `Rc` or constrain captures to
`Clone`. Phase 4 needs the FnOnce ergonomics; landing the same
pattern across Phase 3 + Phase 4 keeps the user-facing
pointer-brand surface uniform.

**Resolution.** Phase 3.5 (pointer-brand-pattern retrofit) closes
F4 structurally rather than as accepted-tradeoff. The retrofit
replaces, on default `Run` / `RunExplicit` only, the
[`StateBrand<RcBrand, S>`](../../../fp-library/src/brands/effects.rs)
/
[`ReaderBrand<RcBrand, E>`](../../../fp-library/src/brands/effects.rs)
threading with parallel sibling brands
[`BoxStateBrand<BoxBrand, S>`](../../../fp-library/src/brands/effects.rs)
/
[`BoxReaderBrand<BoxBrand, E>`](../../../fp-library/src/brands/effects.rs)
whose closure cells are `Box<dyn FnOnce(...) -> A>` via the new
[`ToDynFnOnce`](../../../fp-library/src/classes/to_dyn_fn_once.rs)
trait (sub-step 1). The new sibling effect types
[`BoxState`](../../../fp-library/src/types/effects/state.rs) /
[`BoxReader`](../../../fp-library/src/types/effects/reader.rs) /
[`BoxChoose`](../../../fp-library/src/types/effects/choose.rs)
mirror the structure of
[`State`](../../../fp-library/src/types/effects/state.rs) /
[`Reader`](../../../fp-library/src/types/effects/reader.rs) /
[`Choose`](../../../fp-library/src/types/effects/choose.rs) and
[`SendState`](../../../fp-library/src/types/effects/state.rs) /
[`SendReader`](../../../fp-library/src/types/effects/reader.rs) /
[`SendChoose`](../../../fp-library/src/types/effects/choose.rs) but
with `where P: ToDynFnOnce` bounds restricting instantiation to
`BoxBrand` (the only brand for which `<P as Pointer>::Of<dyn FnOnce>`
is operationally implementable). Multi-shot wrappers
([`RcRun`](../../../fp-library/src/types/effects/rc_run.rs) /
[`RcRunExplicit`](../../../fp-library/src/types/effects/rc_run_explicit.rs)
/ [`ArcRun`](../../../fp-library/src/types/effects/arc_run.rs) /
[`ArcRunExplicit`](../../../fp-library/src/types/effects/arc_run_explicit.rs))
keep their existing `Rc<dyn Fn>` / `Arc<dyn Fn + Send + Sync>`
paths unchanged, because their wrapper-level multi-shot guarantee
makes the multi-shot continuation cell semantically aligned.

After the retrofit, the single-shot-vs-multi-shot property is
enforced at the type level for both the Free spine _and_ the
per-effect closure cell on default Run substrates. A handler on
`Run::interpret` matches `BoxState::Get(k) => k(state)`; the
`k(state)` consumes the box on the single call, statically proving
the continuation is single-shot.

**Trade-off discussion (relative to the F4-recommended Option A).**
The retrofit does what Option A explicitly avoided: it re-opens the
[(3.a-1) "one effect type per operation"
sub-decision](#L209-L240) (locked 2026-05-03). The trade-off is
acceptable because:

1. **Doubling cost is amortised over Phase 4.** Option B's
   "doubles the per-effect type definitions" critique applied when
   only Phase 3 was in scope; with Phase 4 also needing the
   `BoxBrand` path for user-supplied scoped handlers (per Phase 4
   design-question B3), the cost is amortised across both phases.
   Phase 3.5 lands three sibling types
   ([`BoxState`](../../../fp-library/src/types/effects/state.rs) /
   [`BoxReader`](../../../fp-library/src/types/effects/reader.rs) /
   [`BoxChoose`](../../../fp-library/src/types/effects/choose.rs))
   in one commit; Phase 4 reuses the same pattern for
   [`Catch`](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
   / `Local` / `Bracket` / `Span` / `RefLocal` / `RefBracket`
   without adding a parallel brand-family per scoped effect.

2. **Brand surface stays alphabetised, not duplicated.** Each
   `Box*Brand` lives in
   [`fp-library/src/brands/effects.rs`](../../../fp-library/src/brands/effects.rs)
   alongside its `*Brand` and `Send*Brand` siblings; the
   brand-implementations table at
   [`fp-library/docs/pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md)'s
   newly-added
   `(closure-semantic, pointer-capability)` matrix shows that the
   trait family is structurally complete (each pair maps to
   exactly the brands that legitimately implement it).

3. **Doc surface stays single-source.** The retrofit re-uses
   existing rustdoc on
   [`StateBrand`](../../../fp-library/src/brands/effects.rs)'s
   "Coyoneda-fusion at call-site" subsection (which applies
   equally to `BoxStateBrand`); module-level `state.rs` /
   `reader.rs` / `choose.rs` docs gain one new paragraph each
   describing the sibling. The combined incremental doc surface is
   ~30 lines, not the "doubles the documentation surface" Option
   B would have implied at Phase 3 step 5a's smaller scope.

4. **Macro complexity is unaffected.** Phase 3 step 6's
   [`define_effect!` macro](#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit)
   is deferred indefinitely; the per-wrapper smart constructors
   are hand-written. Adding the `BoxBrand` path to the hand-written
   smart constructors is a per-method update on `Run` /
   `RunExplicit` only (six methods total: `get` / `put` / `ask`
   across two wrappers), not a macro extension. If `define_effect!` ever
   ships, the per-pointer-brand split becomes a macro parameter.

The 2026-05-03 (3.a-1) "one effect type per operation" decision
remains valid for the _user-facing_ effect family naming
convention: users see `BoxStateBrand` / `StateBrand` /
`SendStateBrand` as one State family with three pointer-substrate
variants, and the `Run`-family smart constructors thread the
appropriate variant per wrapper without users specifying the
pointer brand explicitly. The "one type per operation" promise
holds at the user-API level (one `get` on `Run`, one `get` on
`RcRun`, one `get` on `ArcRun`); the under-the-hood representation
splits into three siblings to satisfy the structural closure-trait
constraint.

**Why structural is now preferable to documentation-only.** Phase 3
step 8's Option A wrote: "the API claim does not match reality at
the effect-instance level; document the mismatch." After Phase 3.5,
the API claim _does_ match reality: `Run`'s `BoxState::Get(k)` /
`BoxState::Put(s, k)` / `BoxReader::Ask(k)` continuations are
exactly single-shot at the type level. Future-you reading
`Run::get`'s smart constructor signature does not need to consult
the rustdoc for "by the way, the inner closure is multi-shot but
the program is single-shot". The structural property is visible at
the type signature.

**Plan integration.** Phase 3.5 is sequenced between Phase 3 close
and Phase 4 implementation. Sub-steps:

1. `ToDynFnOnce` trait + `BoxBrand` impl (commit `b067f912`,
   2026-05-05).
2. Sibling effect types and brands; smart constructor updates on
   `Run` / `RunExplicit`; integration test updates (commit
   `a762fa27`, 2026-05-05).
3. `pointer-abstraction.md` documentation update (commit
   `89546709`, 2026-05-06).
4. Implicit (covered by sub-step 2's `just verify` clean run).
5. This resolutions.md entry (this commit, 2026-05-06).

Phase 4 implementation then follows, reusing the same
`BoxBrand` + `ToDynFnOnce` pattern for user-supplied scoped-effect
handlers per
[Phase 4 design-question B3's recommendation](#b3-pointer-brand-parameterisation-boxbrand--todynfnonce-on-default-run).

**Cross-references:**

- Phase 3 step 8 disposition (Option A doc-only, 2026-05-05):
  [deviations.md Phase 3 step 8](deviations.md).
- F4 finding's full restatement, root cause, options A/B/C
  evaluation:
  [`review/0_first_order_effects_implementation/remediation_proposals.md`](review/0_first_order_effects_implementation/remediation_proposals.md).
- Phase 4 design-question B3 (the original re-opening rationale):
  [plan.md Phase 4 pre-implementation design questions](#resolved-2026-05-05-phase-4-pre-implementation-design-questions-b1-b4-q1-q3-q5-closed-by-design-adoption-commit-6e960701).
- Three-sibling-types interpretation rationale (why
  `BoxStateBrand` rather than parameterising `StateBrand` over
  `BoxBrand`): [deviations.md Phase 3.5 sub-step 2](deviations.md).
- Trait-family completeness matrix:
  [`fp-library/docs/pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md)'s
  `ToDynFnOnce is BoxBrand-only by structural necessity`
  subsection.

## Resolved (2026-05-05): Phase 4 pre-implementation design questions B1-B4, Q1-Q3, Q5 closed by design-adoption commit `6e960701`

**Disposition.** Eight Phase 4 pre-implementation design questions had their plan-revision-required edits adopted into [plan.md](plan.md) and [decisions.md](decisions.md) by the 2026-05-05 design-adoption commit `6e960701` (B1, B2, B4, Q1, Q2, Q3, Q5) plus subsequent Phase 3.5 retrofit landings (B3 implementation: commits `b067f912`, `a762fa27`, `89546709`, `4471629d`). Each item's original framing (issue, options, recommendation, reasoning) lived in plan.md's `Phase 4 pre-implementation design questions` subsection prior to this 2026-05-06 cleanup; the pre-cleanup framing is recoverable via `git show 2e97e812:docs/plans/effects/plan.md` (last commit retaining the long-form entries). Compact summary below; outstanding follow-ups (POC 3 prerequisite for B4; Q4 / R1 / R2 / R3 prototype-and-risk items pending at R1 implementation kickoff) live in plan.md's [Phase 4 implementation prototypes and risk mitigations](plan.md#phase-4-implementation-prototypes-and-risk-mitigations) and [Phase 4 implementation-kickoff sequencing](plan.md#phase-4-implementation-kickoff-sequencing) subsections.

### B1. `Catch::action` type-parameter interpretation

- **Issue.** Plan and [decisions.md](decisions.md) describe `Catch<'a, P, E, A>` as storing `action: Run<R, S, A>`, but `CatchBrand<P, E>` does not carry `R` or `S` as parameters; the `Run<R, S, A>` rendering was therefore loose notation rather than a literal Rust type. Implementer needed to know whether `action` is genuinely `Run<R, S, A>` (forcing `CatchBrand` to grow `R, S` parameters) or `A` abstract (where `A` becomes "the next program" at the dispatch boundary, mirroring Phase 3 [`State<'a, P, S, A>`](../../../fp-library/src/types/effects/state.rs)).
- **Resolution: Option A.** `action: A` literal; `Run<R, S, A>` is loose notation for "the next program" bound by dispatch context, mirroring Phase 3 `State<'a, P, S, A>`'s use of `A`. Adding `R, S` parameters to every scoped-effect brand contradicts Phase 3 precedent without structural reason; brand parameter surface stays at 2 per scoped-effect type.
- **Plan-text amendments.** [decisions.md section 4.5](decisions.md#45-decision-scoped-effect-representation-via-a-heftia-inspired-dual-row) clarifying paragraph; [plan.md Phase 4 step 3](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) constructor-list paragraph specifying loose-notation semantics.

### B2. Per-scoped-effect-brand `Functor` / `SendFunctor` / `WrapDrop` / `RefFunctor` / `Extract` impls

- **Issue.** The substrate's [`NodeBrand<R, S>`](../../../fp-library/src/types/effects/node.rs) impls require `S: Functor + SendFunctor + WrapDrop + RefFunctor + Extract`. With `S = CNilBrand` (Phase 3) these are vacuously satisfied; with `S = CoproductBrand<CatchBrand<...>, ...>` (Phase 4), each scoped-effect brand must explicitly implement all five traits because [`RcFree::wrap`](../../../fp-library/src/types/rc_free.rs) and similar substrate operations call `<F as Functor>::map` directly. [decisions.md](decisions.md) had stated "the higher-order row does NOT require a Functor instance" which was misleading: the dispatcher trait does not go through Functor, but the substrate's program-traversal machinery still does.
- **Resolution: Option A.** Each scoped-effect brand provides explicit per-trait impls; Phase 3 per-effect impls are the template. Up to 30 trait impls total (5 brands \* ~5 traits, minus Span which has no closure); each is mechanical. Option B (Coyoneda wrapping per scoped effect) was rejected: doubles per-op allocation cost and contradicts the dual-row design's whole point. Option C (substrate redesign of `NodeBrand`'s Functor requirement) was rejected: justification was documentation alignment, not capability.
- **Plan-text amendments.** [decisions.md section 4.5](decisions.md#45-decision-scoped-effect-representation-via-a-heftia-inspired-dual-row) clarifying paragraph distinguishing dispatcher-trait vs program-traversal-trait requirements; [plan.md Phase 4 step 3](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) substrate-required-traits paragraph plus a code-block `impl Functor for CatchBrand<P, E>` template (added 2026-05-06 follow-up cleanup).

### B3. Pointer-brand parameterisation: `BoxBrand` + `ToDynFnOnce` on default Run

- **Issue.** The R2 plan revision specified a `BoxBrand` pointer brand for the default `Run` / `RunExplicit` substrate, with `BoxBrand`'s closure projection differing from `RcBrand` / `ArcBrand`'s (`Box<dyn FnOnce>` vs `Rc<dyn Fn>` vs `Arc<dyn Fn + Send + Sync>`). On cross-checking, Phase 3's State effect did not have a `BoxBrand`: State on `Run` used `RcBrand` per [`Run::get`](../../../fp-library/src/types/effects/run.rs)'s smart constructor signature. The original plan revision invented `BoxBrand` to symmetrise the wrapper-to-pointer-brand mapping, but the symmetry didn't hold in Phase 3 and forcing it into Phase 4 introduced a new brand whose closure-trait-object differed from its siblings.
- **Resolution: Option B (revised).** Use the existing pointer-abstraction infrastructure (`BoxBrand`, `RcBrand`, `ArcBrand` per [`fp-library/docs/pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md)) extended with a new trait `ToDynFnOnce` parallel to `ToDynFn`, implemented only by `BoxBrand`. `Rc<dyn FnOnce>` and `Arc<dyn FnOnce>` are operationally broken because `FnOnce::call_once` consumes `self` (the trait object), which cannot be moved out of a shared pointer without invalidating other clones. Default `Run` users get `Box<dyn FnOnce>` storage (single-shot at the closure level by construction); `RcRun` / `ArcRun` users get `Rc<dyn Fn>` / `Arc<dyn Fn + Send + Sync>` for multi-shot. Ergonomic gain: user-supplied scoped handlers on default `Run` can be `FnOnce`-natural (no Rc-wrapping or Clone-bounded captures). The retrofit applied to Phase 3's State / Reader / Choose closes the [F4 finding structurally](resolutions.md#resolved-2026-05-06-phase-3-prior-review-f4-closed-structurally-via-phase-35-retrofit-sibling-boxbrand-family-on-default-run-substrates) rather than as accepted-tradeoff.
- **Plan-text amendments.** [decisions.md section 4.5](decisions.md#45-decision-scoped-effect-representation-via-a-heftia-inspired-dual-row) trait-family table and `Closure-storage ceiling on default Run` subsection; new [Phase 3.5 sub-section in plan.md](plan.md#phase-35-pointer-brand-pattern-retrofit) with five sub-steps.
- **Implementation status.** Shipped in Phase 3.5 (sub-steps 1-5; commits `b067f912` / `a762fa27` / `89546709` / `4471629d`). The same pattern is reused in Phase 4 step 3 for user-supplied scoped handlers.

### B4. Catch dispatcher's sentinel mechanism

- **Issue.** The R1 plan revision said the catch dispatcher "interposes a `Throw` catcher that returns to a sentinel value, observes the sentinel via the dispatcher's outer `interpret` loop, and routes to `Catch::handler`". The sentinel's TYPE in the program was unspecified. `Run<R, S, A>`'s payload type is `A`; encoding "either A or thrown E" required either changing the program type or using a side channel.
- **Resolution: Option B with POC validation prerequisite.** New substrate primitive `interpret_with_either<EBrand, Idx>(self, fo_handlers: &impl DispatchHandlers<...>) -> Either<A, EBrand::Op>` (specialisation of [`interpret_with`](../../../fp-library/src/types/effects/run.rs#L885-L900) returning the matched effect's payload as `Right` instead of narrowing). The Catch dispatcher pattern-matches the Either; `Left(a)` becomes `Run::pure(a)`, `Right(thrown_e)` calls `Catch::handler`. No interior mutability; the type system structurally distinguishes "completed" from "thrown". Option A's interior-mutability cell with a placeholder-program return was rejected because the placeholder requires either `A: Default`, an unsafe sentinel, or a panic-on-evaluate `Box::leak`-style construct, none of which is clean. Option C (`std::panic::catch_unwind`) was rejected as unsound for non-`UnwindSafe` programs.
- **Plan-text amendments.** New [Phase 4 step 2a](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) substrate primitive entry; [Phase 4 step 4](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) Catch dispatcher description with `match interpret_with_either` body sketch.
- **Outstanding prerequisite.** POC 3 (`interpret_with_either` substrate primitive on `RcRun`) pending validation; commit-ordering decision is documented at [Phase 4 implementation-kickoff sequencing K1](plan.md#k1-poc-3-interpret_with_either-validation-ordering).

### Q1. `scoped_handlers!` macro shape

- **Issue.** Phase 4 step 5 references a `scoped_handlers!{...}` macro as a companion to the existing [`handlers!`](../../../fp-library/src/types/effects/handlers.rs) macro for assembling the second list passed to `interpret`. Syntax and emitted shape were unspecified.
- **Resolution: Option A with DRY factoring.** `scoped_handlers!{CatchBrand<P, E>: |op| ..., ...}` mirrors `handlers!` syntax exactly; both factor through a new helper module [`fp-macros/src/effects/handler_list_emitter.rs`](../../../fp-macros/src/effects/) (new file) parameterised by cell type identifier (`Handler<E, F>` for FO; `ScopedHandler<S, F>` for scoped) and cons-list cell-and-tail type identifiers (`HandlersCons` / `HandlersNil` for FO; `ScopedHandlersCons` / `ScopedHandlersNil` for scoped). The `effects!` macro's lexical-sort helper is consumed inside the new emitter helper as well.
- **Plan-text amendments.** [Phase 4 step 5](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) helper-module reference + entry-point pattern. Deviations.md entry pending at the commit that retrofits `handlers!` through the helper.

### Q2. `define_scoped_effect!` macro fate

- **Issue.** Phase 4 step 5 referenced `define_scoped_effect!` "mirroring section 9's planned `define_effect!` for first-order effects". `define_effect!` was [deferred per the 2026-05-04 resolution](#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit) (Phase 3 step 6) until Phase 4 ships or user demand surfaces; `define_scoped_effect!` had no precedent template.
- **Resolution: Option A.** Defer in parallel with the Phase 3 `define_effect!` deferral; users hand-write each scoped effect's brand + four-to-six trait impls (analog to Phase 3's `State` / `Reader` / etc.). Revisit triggers parallel the existing `define_effect!` deferred-item triggers (a user writing more than two custom scoped effects, or a Phase 6+ revisit of `define_effect!`).
- **Plan-text amendments.** [Phase 4 step 5](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) drops the `define_scoped_effect!` reference; [Phase 6+ deferred items](plan.md#phase-6-deferred-not-in-this-plan) gains a new entry parallel to the existing `define_effect!` entry.

### Q3. `interpret_scoped_with::<EBrand>` row-narrowing primitive

- **Issue.** Phase 4 step 7 references `interpret_scoped_with::<EBrand>` as a row-narrowing primitive on the scoped row paralleling Phase 3's [`interpret_with`](../../../fp-library/src/types/effects/run.rs#L885-L900). Signature and substrate plumbing unspecified.
- **Resolution: Option A.** New per-wrapper method `interpret_scoped_with::<EBrand, Idx, SMinusE>(scoped_handler) -> Run<R, SMinusE, A>`, paralleling Phase 3's `interpret_with` on the scoped row. Mechanically derivable from Phase 3 pattern; users need pipeline-ordering control for non-commuting scoped effects (e.g., `Catch` before `Local` vs after). Option B (reuse `interpret_with` with type-level branching) has worse type-inference characteristics; Option C (no row-narrowing on scoped row) limits expressivity for handler libraries that ship narrowing handlers.
- **Plan-text amendments.** [Phase 4 step 7](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) callers-write-`interpret_scoped_with` paragraph already in canonical text. The per-wrapper method itself ships as part of Phase 4 step 4's interpret-rewrite.

### Q5. `BracketGuard<A, F>` lifecycle

- **Issue.** Phase 4 step 3's Bracket entries said the dispatcher wraps the resource in a `BracketGuard<A, F>` whose `Drop` impl invokes `release` synchronously. Specific lifecycle questions: when is the guard constructed? Does `body` receive the guard by ownership or reference? Does `release`'s `Run<R, S, ()>` get scheduled by Drop or executed synchronously? These small decisions combined into whether panic-during-body actually runs `release`.
- **Resolution: Option A.** RAII guard, ownership-passed to body, synchronous release-on-Drop. Guard constructed inside the bracket dispatcher after `acquire` evaluates; ownership-passed to the body closure; dropped when body returns (running release on drop). Release executes as a synchronous function call (not threaded through interpret) because the interpret loop has already exited the dispatcher's frame. Matches Rust's RAII conventions; release runs deterministically on body completion or panic; no reliance on `catch_unwind`'s `UnwindSafe` constraints. Option B (reference-passed + explicit-drop) is harder to reason about under panic; Option C (`catch_unwind`) is unsound for non-`UnwindSafe` programs.
- **Plan-text amendments.** [Phase 4 step 3 Bracket entries](plan.md#phase-4-scoped-effects-heftia-inspired-dual-row) `BracketGuard` lifecycle sentence specifying ownership-passed-to-body + synchronous-release-on-Drop semantics.

## Resolved (2026-05-04): Phase 3 step 6 (`define_effect!` macro) deferred until Phase 4 ships or user demand surfaces; design research preserved for later revisit

`define_effect!` was scoped as a proc-macro that mechanically
generates an effect enum + brand registration + per-wrapper
smart constructors from a single user declaration like:

```rust
define_effect! {
    Reader<E> {
        fn ask() -> E,
    }
}
```

Per-effect, the macro would emit ~200-400 lines of boilerplate:
the effect enum, `impl_kind!` brand registration, manual `Clone`,
[`Functor`](../../../fp-library/src/classes/functor.rs) /
[`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
impls, conditional Send-aware parallel brand + type pair, and
4 or 6 per-wrapper smart constructors. The macro itself was
estimated at ~1500+ lines of proc-macro code.

### Why deferred

Five reasons, in order of weight:

1. **Phase 4 (scoped effects, heftia dual row) may invalidate
   the codegen target.** Scoped effects use a different brand
   shape and a different per-wrapper rollout pattern than
   first-order effects. A `define_effect!` shipped now for
   first-order effects would either need significant rework or
   become a sibling-not-replacement when Phase 4 lands. Better
   to know the full target before mechanising.
2. **Pre-1.0 API instability bleeds into macros.** The recent
   step 5e substrate fix changed `RcFree`/`ArcFree`'s
   continuation queue from value-typed `CatList` to refcounted
   `RcCatList`/`ArcCatList`. A macro shipped before that would
   have hard-coded the wrong substrate and needed migration.
   The same risk exists for whatever refines next.
3. **No users yet to validate the input syntax.** Five design
   approaches were surveyed (see below) with non-trivial
   ergonomic differences; without real workloads it's not clear
   which is right.
4. **Library already ships 5 standard effects.** The boilerplate
   savings only apply to effects that don't yet exist. Without
   active demand for custom effects, the macro's break-even is
   ~4-8 future effects, which may take a long time to surface.
5. **Nothing in the rest of the plan depends on the macro.**
   Step 7 (`compile_fail` UI tests) and step 8 (review-
   remediation docs) validate / document the existing hand-
   written effects. Phase 4 (scoped) and Phase 5/6+ also do
   not depend on this macro.

### Trigger conditions for revisiting

Revisit when **either** of:

- **(a) Phase 4 ships** (scoped effects via heftia dual row).
  The full effect-shape design space is then settled; the macro
  can target both first-order and scoped effects, or be cleanly
  scoped to first-order if scoped effects are too different to
  share a macro.
- **(b) A real user surfaces concrete demand for custom
  effects** (a workload, not a hypothetical). The use case
  informs which design approach below is right.

If neither trigger fires within the foreseeable future, the
permanent answer "copy
[`reader.rs`](../../../fp-library/src/types/effects/reader.rs)
and adapt for your effect" is also acceptable for a library
that already ships 5 standard effects covering the common
cases. A Phase 6+ HOWTO entry documenting that recipe would
close out the deferral.

### Design research (preserved for later revisit)

Five approaches were surveyed during the deferral discussion.
Listed roughly from most-terse to most-explicit.

#### Approach 1: PureScript-faithful (most terse)

```rust
define_effect! {
    Reader<E> {
        fn ask() -> E,
    }

    State<S> {
        fn get() -> S,
        fn put(s: S),
    }

    Writer<W> {
        fn tell(log: W),
    }

    #[multi_shot]
    Choose {
        fn choose() -> bool,
    }
}
```

The macro infers the variant encoding from the operation
signature:

- `fn op(args...) -> Ret` -> variant carries `args` (if any)
  plus a `dyn Fn(Ret) -> A` continuation.
- `fn op(args...)` (no return arrow) -> variant carries `args`
  plus `A` directly (no continuation;
  `PhantomData<&'a ()>` for the unused lifetime).

Pros: reads exactly like PureScript Run; minimal cognitive
overhead; naturally maps `fn ask() -> E` to "the operation
produces an `E` that the next program consumes".

Cons: the `fn op(...)` (no return) vs `fn op(...) -> ()`
distinction is subtle; users might expect them to mean the
same thing. Variant names are auto-derived from operation
names (`ask` -> `Ask`, `tell` -> `Tell`), which works for
PureScript-style naming but may clash on edge cases.

#### Approach 2: Explicit variant + constructor names

```rust
define_effect! {
    State<S> {
        Get: get() -> S,
        Put: put(s: S) -> (),
    }
}
```

`Variant: smart_constructor(args) -> ContType` syntax pairs
each enum variant with its smart-constructor entry-point.

Pros: variant names are explicit, matching PureScript
convention exactly. Operation-vs-variant distinction is
visible. The `-> ()` makes "this op has a unit continuation"
explicit (Put pattern).

Cons: slightly more verbose. Two names per op (variant +
constructor) when most map 1:1 (`Ask`/`ask`).

#### Approach 3: Plain Rust enum + attribute-driven generation

```rust
#[define_effect(brand = "ReaderBrand", multi_shot = false)]
pub enum Reader<E, A> {
    #[constructor(ask)]
    Ask(Continuation<E, A>),
}
```

Users write a normal Rust enum; marker types like
`Continuation<E, A>` signal what gets generated.

Pros: looks like normal Rust; users can mix custom variants
with macro-generated boilerplate; visible in `rust-analyzer`
even without macro expansion.

Cons: requires marker types in scope; doesn't match the
PureScript Run user-facing surface as closely; pointer-brand
`P` and lifetime `'a` parameters need separate inference rules.

#### Approach 4: Two-tier (separate brand registration from constructors)

```rust
define_effect_type! {
    Reader<E> {
        fn ask() -> E,
    }
}

// Generated: ReaderBrand, Reader<'a, P, E, A>, Functor,
// SendFunctor, SendReaderBrand, SendReader.
// User still hand-writes the smart constructors per wrapper.
```

The macro generates everything except the per-wrapper smart
constructors; users keep those hand-written for explicit
control over bounds.

Pros: smart constructors stay legible in source (where most of
the per-wrapper bound differences live); the macro is smaller
and more focused.

Cons: doesn't eliminate the largest single chunk of boilerplate
(~50-100 lines per smart constructor x 6 wrappers); users
still have to write them.

#### Approach 5: Single-tier with attribute escape hatches

Approach 1 plus attribute hooks for special cases:

```rust
define_effect! {
    State<S> {
        fn get() -> S,
        fn put(s: S),
    }

    #[no_send_aware]    // skip SendReaderBrand / SendReader
    Reader<E> {
        fn ask() -> E,
    }

    #[multi_shot]
    Choose {
        fn choose() -> bool,
    }

    #[wrappers(Run, RunExplicit)]    // override default 6-wrapper rollout
    SingleShotOnly {
        fn op() -> i32,
    }
}
```

Pros: default behavior is what users want 95% of the time;
attributes handle the corner cases without polluting the base
syntax.

Cons: attribute set is open-ended; risk of feature creep over
time.

### Open design questions (deferred along with the macro)

1. **Variant-encoding inference.** How does `fn op(args)` vs
   `fn op(args) -> Ret` distinguish "no continuation,
   `A` is owned directly" (Tell) from "continuation
   `Fn(Ret) -> A`" (Ask/Get/Put/Choose)? Three plausible rules:
   - **(a)** Missing return arrow -> no continuation;
     `-> ()` -> `Fn(()) -> A` continuation;
     `-> T` -> `Fn(T) -> A` continuation. Distinguishes
     Tell/Put/Ask cleanly.
   - **(b)** Always generate a continuation; the user signals
     "no continuation" via a marker
     (`fn tell(log: W) using direct;` or
     `#[direct] fn tell(log: W);`).
   - **(c)** Each declaration explicitly states its
     continuation type.
2. **Send-aware brand: auto or opt-in?** Auto-generate
   `SendXxxBrand` for every effect that has a `dyn Fn`
   continuation (matches existing State/Reader/Choose
   pattern), or require `#[send_aware]` opt-in?
3. **`multi_shot` placement.** Per-effect attribute, per-
   operation attribute, or syntax keyword?
4. **Pointer-brand parameter `P` inference.** Effects with
   `dyn Fn` continuations need a `P: ToDynCloneFn` parameter;
   effects without (Writer, Except) don't. Auto-detect from
   the presence of any continuation-bearing operation, or
   require explicit declaration?
5. **User-extensibility.** Can a user add custom variants
   alongside macro-generated ones? Or is the enum closed by
   the macro? PureScript users sidestep this because their
   data declarations are open by construction. Approach 3
   (proc-macro on a plain enum) handles this naturally;
   approach 1 doesn't without explicit support.
6. **Migration path.** If/when the macro ships, should the
   five existing effects be migrated to dogfood it, or shipped
   untouched? If migrated, the macro must produce
   byte-equivalent code (modulo doc comments) so the test
   suite continues to pass.
7. **Doc comment placement.** Where do user-supplied doc
   comments land? On the variant, the smart constructor, the
   brand, or all three? Per-component override?

### Tentative recommendation if revisited

Pre-deferral, **Approach 1 (PureScript-faithful) with rule
(1.a)** was the leading option: closest match to the existing
user-facing surface, smallest cognitive load, naturally
auto-generates Send-aware parallels for continuation-bearing
effects. The most uncertain question was **(5) user-
extensibility**; if a future workload needs that, approach 3
becomes more attractive.

This recommendation is non-binding; revisit with full Phase 4
context (or user-workload context, depending on which trigger
fires first).

## Resolved (2026-05-04): Phase 3 step 5e Erased Free family multi-shot dispatch via `RcCatList`/`ArcCatList` (option 1c-ii: parallel reference-counted CatList variants)

`Choose` smart constructors on `RcRun` and `ArcRun` were
panicking at runtime with "`RcFree::to_view map called more
than once`" / "`ArcFree::to_view map called more than once`"
because the Erased Free family's continuation queue was held
in a value-typed [`CatList`](../../../fp-library/src/types/cat_list.rs)
whose derived `Clone` is O(N) deep-recursive. To compensate,
[`RcFree::to_view`](../../../fp-library/src/types/rc_free.rs)
captured the queue inside a `Cell<Option<...>>` and consumed it
once via `take()`, making the closure structurally single-shot
even though its outer `Rc<dyn Fn>` wrapping permitted multiple
invocations. The `Choose` handler runs the continuation twice
(once per branch) and tripped the panic on the second call.

PureScript's `CatList` is naturally O(1)-cloneable because the
language is GC-managed and immutable: "cloning" is just copying
a heap reference. The Rust port chose `VecDeque<CatList<A>>`
with a derived `Clone`, which is cache-local and ergonomic for
a regular catenable list but has no structural sharing, so
clones cost O(N). The Cell/Mutex pattern in `to_view` was a
workaround for that ownership friction.

### What landed

Two new substrates: [`RcCatList<A>`](../../../fp-library/src/types/rc_cat_list.rs)
and [`ArcCatList<A>`](../../../fp-library/src/types/arc_cat_list.rs).
Each wraps `Rc<VecDeque<RcCatList<A>>>` (or `Arc<...>` for the
Send + Sync sibling), so `Clone` is a refcount bump. Mutation
methods (`snoc`, `append`, `cons`) use `Rc::make_mut`/`Arc::
make_mut` for copy-on-write; uniquely-owned deques mutate in
place, shared deques clone one level deep (each contained
element clones in O(1)). The `A: Clone` bound is required on
mutation/uncons; the Free family's continuation types
(`RcContinuation`, `ArcContinuation`) already implement Clone
via `Rc::clone`/`Arc::clone`.

`RcFree::to_view` and `ArcFree::to_view` switched to capture
the continuation list by move and clone it (`all_conts.clone()`)
on every invocation. The `Cell<Option<...>>` and
`Mutex<Option<...>>` workarounds are gone, including the per-
layer mutex acquire on the Arc path that was only ever in
place for `Sync` compatibility, never for actual concurrency
control. `ArcCatList` is structurally `Send + Sync` whenever
`A` is, so the closure satisfies the trait-object bounds on
`Arc<dyn Fn(...) + Send + Sync>` without any synchronisation
primitive.

The new substrates implement only the surface needed by the
Free family (`empty`, `is_empty`, `singleton`, `cons`, `snoc`,
`append`, `uncons`, `len`, `Clone`, `Default`, iterative
`Drop`). The full trait soup on
[`CatListBrand`](../../../fp-library/src/brands.rs) (Functor,
Foldable, Traversable, etc.) is intentionally not mirrored;
`RcCatList`/`ArcCatList` are continuation-queue substrates,
not general-purpose lists. Existing `CatList` is unchanged.

### Why option 1c-ii over the alternatives

Five sub-options were surveyed. Approach 1a (clone the
existing `CatList` per call) ran into the O(N) deep-clone
cost on the single-inner hot path (Identity, State, Reader,
Except, Writer). Approach 1b (Rc-wrap the existing CatList
at the use site) didn't actually help, since `append`
consumes the list and would still need a deep clone before
mutation. Approach 1c-i (a single Arc-everywhere CatList)
would penalise `RcFree`'s hot path with atomic refcount ops.
Approach 1c-iii (generic over `RefCountedPointer`) leaks
bound noise through every type signature that touches
CatList. Approach 1d (don't fold continuations into `F::map`
at all) would unwind the Phase 1 stack-safety story by
walking continuations one at a time per Suspend.

1c-ii (parallel `RcCatList`/`ArcCatList` types) mirrors the
codebase's existing Rc/Arc split (`RcCoyoneda`/`ArcCoyoneda`,
`RcFree`/`ArcFree`, `RcBrand`/`ArcBrand`). The duplication is
straightforward (the implementations differ only in `Rc` vs
`Arc` and the resulting auto-trait derivations) and matches
the convention every other type in the split already pays.

### Validation

All 4 [`run_choose.rs`](../../../fp-library/tests/run_choose.rs)
integration tests pass on the new substrate (one per
multi-shot wrapper). The full pre-existing test suite passes
unchanged, confirming no regression on single-inner Free
workloads. Per-Suspend cost on those workloads adds one
`Rc::clone` (or `Arc::clone`), which is a refcount bump
rather than a structural copy.

### What's preserved

The 2026-05-03 wrapper-parameterization resolution Q4=ii
("`Choose` ships on all four multi-shot wrappers") is fully
honored; no demotion needed. The substrate fix unblocks the
same API surface that resolution committed to.

### Future direction

If a future workload surfaces a measurable need for a
fully-featured `RcCatList`/`ArcCatList` (Functor, Foldable,
Traversable, brand-level dispatch), the additional surface
can be added incrementally. The current scope deliberately
ships the minimum needed by the Free family.

## Resolved (2026-05-04): Phase 3 step 5 (`interpret_with_rec`) deferred indefinitely (option (c))

Pipeline row-narrowing combined with `MonadRec`-target stack
safety does not compose cleanly without a richer abstraction
(e.g., `Codensity`, `Eff`-style continuation passing, or
`Traversable` on every row brand). PureScript Run does not
ship the combination either; the gap is structural, not just
an implementation oversight. Phase 3 ships three orthogonal
interpreter primitives instead of four; users who want both
row narrowing and stack safety chain
[`interpret_with`](../../../fp-library/src/types/effects/run.rs)
(narrowing) and
[`interpret_rec`](../../../fp-library/src/types/effects/run.rs)
(stack safety) at the boundary of the pipeline.

### The structural obstacle

The
[prompt.md](prompt.md)
"Step 5 implementation pattern" subsection specified the
handler signature as

```rust
handler: impl Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Wrapper<RMinusE, CNilBrand, A>>>)
    -> M::Of<'_, Wrapper<RMinusE, CNilBrand, A>>
```

i.e. inners arrive at the handler already narrowed
(`Run<RMinusE, ...>`) and M-wrapped. But producing narrowed
inners requires structural recursion through the inner
programs.
[`Run::interpret_with_shared`](../../../fp-library/src/types/effects/run.rs)'s
matched arm uses
`EBrand::Functor::map(|inner: Run<R, ...>| inner.interpret_with_shared(...), lowered)`
to recursively narrow each inner before handing the layer to
the handler. The recursion is structural (one frame per peel)
and lazy for closure-shaped effects (e.g.,
[`State`](../../../fp-library/src/types/effects/state.rs)'s
continuations defer the recursion).

[`Run::interpret_rec`](../../../fp-library/src/types/effects/run.rs)
sidesteps recursion via
[`tail_rec_m`](../../../fp-library/src/classes/monad_rec.rs)
because the row collapses fully (no narrowing): the step
closure produces `M(ControlFlow<A, Run<R, CNilBrand, A>>)`,
where the loop state is the un-narrowed program and the
final result is the plain `A`. There's no per-layer
structural-recursion need.

The conjunction (`interpret_with_rec`) requires per-layer
structural recursion (for narrowing) AND `tail_rec_m`-driven
linear iteration (for stack safety). For multi-inner row
layers (e.g.,
[`VecBrand`](../../../fp-library/src/types/vec.rs)-shaped
effects), the unmatched arm needs to swap
`RMinusE::Of<M::Of<...>>` to `M::Of<RMinusE::Of<...>>`,
which requires
[`Traversable`](../../../fp-library/src/classes/traversable.rs)
on `RMinusE` plus
[`Applicative`](../../../fp-library/src/classes/applicative.rs)
on `M`. Non-rec `interpret_with` sidesteps this by using just
[`Functor::map`](../../../fp-library/src/classes/functor.rs)
(no M to swap with). PureScript Run skips the combination for
the same reason.

### Options surveyed

**(a) Handler keeps `interpret_with`'s narrowed-inner shape;
implementation does inner structural recursion + outer
`tail_rec_m`.** Step closure type
`Run<R, ..., A> -> M(ControlFlow<Run<RMinusE, ..., A>, Run<R, ..., A>>)`.
Matched arm: structurally-recurse into each inner; M-fmap
pure to wrap; hand to handler; M-fmap `Break`. Outer loop
terminates after one peel: `tail_rec_m` is decorative.

- _Pros:_ matches the prompt's handler signature; user
  ergonomics consistent with non-rec `interpret_with`.
- _Cons:_ stack-safety benefit is whatever the matched
  effect's M-bind chain provides, which is independent of
  `tail_rec_m` and already available via plain
  `interpret_with` followed by M-bind composition. ~600
  lines plus ~12 Send + Sync clauses on the Arc family for
  ~zero genuine benefit.

**(b) Handler returns `M::Of<Run<R, CNilBrand, A>>`
(next-state in the un-narrowed row).** Step closure same
type; matched arm: hand original-row inners to handler;
handler returns `M::Of<Run<R, ...>>`; M-fmap `Continue`.
Outer loop iterates per matched-effect occurrence.
Unmatched arm still needs `Traversable` for multi-inner
layers.

- _Pros:_ `tail_rec_m` actually iterates; closure-shaped
  matched effects (`State`) get genuine stack safety.
- _Cons:_ handler signature differs from non-rec
  `interpret_with`: handler can't compose in the narrowed
  row, only the original row. Most non-rec `interpret_with`
  handlers don't translate. Unmatched arm requires
  `Traversable` on the row brand which is too heavy a
  requirement to put on row brands.

**(c) Defer step 5 indefinitely.** Document that the
combination doesn't compose cleanly without a richer
abstraction. Users who want both row narrowing and stack
safety chain `interpret_with` (narrowing, no stack safety)
followed by `interpret_rec` (stack safety on the
all-handlers-at-once form). Phase 3 ships the two
interpreter primitives separately; the combined form
becomes a Phase 6+ concern when a richer abstraction is in
scope.

- _Pros:_ zero implementation cost; honest about the
  abstraction limit; existing primitives are sufficient for
  most use cases.
- _Cons:_ closes off one of the four cells in the
  cognitive-model matrix (M-free pipeline / M-free
  all-handlers / M-target pipeline / M-target all-handlers).

**(d) Re-derive from PureScript Run; no analog exists.**
PS Run doesn't have `interpretWithRec`. Equivalent to (c)
with the explicit "no PS analog" justification.

### Resolution: option (c) (with (d)'s justification merged in)

Locked in. The abstraction tension reflects a real design
limit, not just an implementation gap. Deferring preserves
the option to revisit with a richer encoding (e.g.,
`Codensity`-style transformation) when one is in scope. The
"three interpreter primitives" framing replaces the original
four-cell matrix throughout the docs.

### Workaround for users wanting both

```rust
// Stack-safe pipeline: narrow effects one at a time
// (no stack safety on the narrowing itself), then drive
// the all-handlers stage via `interpret_rec` for stack
// safety on the long-bind-chain effects (e.g., State).
let narrowed: Run<RMinusE, CNilBrand, A> =
    prog.interpret_with::<E1, _, _>(handler1);
let result: ThunkBrand::Of<'static, A> =
    narrowed.interpret_rec::<ThunkBrand>(handlers! { /* ... */ });
```

The intermediate `interpret_with` calls have no stack-safety
guarantee, but the final `interpret_rec` provides
`tail_rec_m`-driven safety for the dispatched effects'
M-bind chains. For programs whose effect rows can be peeled
without unbounded recursion in the narrowing stage (the
common case), this composition is sufficient.

### Forward-looking: Rust-native iteration could make `interpret_with` itself stack-safe

PureScript needs `MonadRec` because the language lacks native
iteration; Rust has `loop`/`while` and unrestricted mutable
stacks. That changes the design space at the implementation
level (not the API level): the structural-recursion concern
in non-rec `interpret_with` (each `Identity`-shaped layer
adds a host stack frame) can in principle be replaced with a
manual work-stack state machine. Maintain a stack of
partially-narrowed layers, process one inner at a time,
accumulate results into the final narrowed program. CESK /
SECD-style; awkward in PS, implementable in Rust at ~500-800
lines.

This is a Rust-specific _implementation_ option for
[`interpret_with`](../../../fp-library/src/types/effects/run.rs)
itself, not a path back to the deferred `interpret_with_rec`
public API. The multi-inner unmatched-arm `Traversable`
obstacle that motivated the public-API deferral is
independent of how the per-inner recursion is driven; native
iteration changes the loop driver but not the M-swap
requirement.

If a user reports stack overflows in deeply-nested
`interpret_with` pipelines (the eager `Identity`-style layer
case), the appropriate response is to land an
internal-iteration deviation against `interpret_with` rather
than to revisit this resolution. The public-API decision (no
fourth primitive) stays.

### Documentation impact

- [decisions.md](decisions.md) updated to record that
  Phase 3 ships three interpreter primitives, not four.
- [plan.md](plan.md)'s phasing list dropped step 5;
  steps 6, 7, 8, 9 renumbered to 5, 6, 7, 8.
- [prompt.md](prompt.md)'s "Step 5 implementation pattern"
  subsection replaced with a "Deferred: combined pipeline +
  MonadRec" note pointing here.

### Cross-references

- [Step 3 `interpret_with`](../../../fp-library/src/types/effects/run.rs):
  the row-narrowing primitive; structural recursion in
  `interpret_with_shared`'s matched arm is the precedent.
- [Step 4 `interpret_rec`](../../../fp-library/src/types/effects/run.rs):
  the MonadRec-target primitive; loop state is un-narrowed
  program, no structural recursion needed.
- [`tail_rec_m`](../../../fp-library/src/classes/monad_rec.rs):
  the stack-safe loop driver; step closure shape is
  `A -> M(ControlFlow<B, A>)`.

## Resolved (2026-05-04): Phase 3 step 6a downstream blocker; `ArcCoyoneda`'s algebra migrated to `SendFunctor` (option (a))

The
[2026-05-03 SendFunctor option-(c) resolution](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified)
ratified a parallel
[`SendStateBrand<P, S>`](../../../fp-library/src/brands.rs)
/
[`SendState<'a, P, S, A>`](../../../fp-library/src/types/effects/state.rs)
type for the Arc family State effect. The 6a.4 + 6a.6 smart
constructors compiled under that resolution, but attempting to
ship `run_state.rs` integration tests surfaced a downstream
gap: the
[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
dispatch path required `EBrand: Functor + SendFunctor`
([`interpreter.rs:337`](../../../fp-library/src/types/effects/interpreter.rs)),
but `SendStateBrand` cannot honestly implement `Functor`
because [`Functor::map`](../../../fp-library/src/classes/functor.rs)'s
signature only requires `f: impl Fn` (no `Send + Sync` bound)
while `SendState`'s variants store
`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> A + Send + Sync>`
(closures must be `Send + Sync` at storage time).

### Three flavours of "Send-aware"

[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
satisfied two of three Send-awareness properties but not the
third:

1. **Storage Send-aware** (yes): the inner
   `ArcCoyonedaLowerRef` trait has `: Send + Sync + 'a` as a
   supertrait; layer fields are `Send + Sync`; the whole value
   can cross thread boundaries.
2. **Operations Send-aware** (yes): `lower_ref` is callable
   from a spawned thread.
3. **Algebra Send-aware** (no, pre-migration): `lower_ref`'s
   body invoked `F::map` (no `Send + Sync` bound on the
   closure parameter).

For brands implementing both `Functor` and `SendFunctor`,
`lower_ref` worked because `Functor::map` happened to produce
a `Send + Sync` result when inputs were. The system implicitly
relied on every `Functor` impl in the project producing
`Send + Sync` results; a coincidence rather than a guarantee.
For brands like `SendStateBrand` whose structure pins
`Send + Sync` on the closure, the gap surfaced as a hard
compile error: the dispatch impl's `Functor` bound was
unsatisfiable.

### Phase 2 step 9d precedent

[Phase 2 step 9d](#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family)
explicitly migrated
[`ArcFree`](../../../fp-library/src/types/arc_free.rs) and
[`ArcFreeExplicit`](../../../fp-library/src/types/arc_free_explicit.rs)
from `F: Functor` to `F: SendFunctor` for the same reason: a
Send-aware substrate's algebra should propagate Send-aware
bounds. `ArcCoyoneda` was not part of that migration because
no Send-only brand existed inside ArcCoyoneda at the time.
`SendStateBrand` was the first.

### Options surveyed

**(a) Migrate `ArcCoyoneda`'s algebra to `SendFunctor`.**
Replace `F: Functor` with `F: SendFunctor` on the inner
`ArcCoyonedaLowerRef` trait method, the three layer impls
(Base, MapLayer, NewLayer), and the public
`ArcCoyoneda::lower_ref` plus all derived methods. Bodies use
[`F::send_map`](../../../fp-library/src/classes/send_functor.rs)
instead of `F::map`. Drop `+ Functor` from the dispatch impl.

**(b) Add a parallel `send_lower_ref` method.** Keep
`lower_ref` bound on `F: Functor`; add a new `send_lower_ref`
method bound on `F: SendFunctor`. Layer impls implement both.
Discovered structurally harder than it sounded:
`MapLayer::send_lower_ref`'s recursion needs `B: Send + Sync`
on the intermediate type, but Rust forbids tightening
trait-method where-clauses on impls beyond what the trait
declares, so the bound has to live at the struct level
(`ArcCoyonedaMapLayer<F, B: Send + Sync, A>`), which
propagates back to `ArcCoyoneda::map<B>` and breaks callers
using non-Send + Sync intermediates.

**(c) Defer 6a.4 + 6a.6 indefinitely.** Document the gap;
leave `SendStateBrand` and the Arc-family smart constructors
in place but un-dispatch-able. Rejected because it abandons
the locked-in "six variants per effect" design from the
[2026-05-03 wrapper-parameterization resolution](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization).

**(c'') Define a parallel `SendArcCoyoneda`.** Additive (no
breaking change), but doubles the Coyoneda type surface and
complicates the per-wrapper Coyoneda variant pairing rule
from Phase 2 step 9h.

### Resolution: option (a)

Locked in. The migration is bounded; the Phase 2 step 9d
precedent is exactly this; the existing implicit "every
Functor produces Send + Sync" reliance is fragile and would
break the next time a Functor-only-but-not-Send-friendly
brand surfaces.

### Bound placement: method-level + impl-block-level

Matching ArcFree's precedent:

- `A: Send + Sync + 'a` lives at the main impl block.
- `F: SendFunctor` lives at method-level where-clauses.
- The struct definition stays minimal.

### Brand-level `Foldable` on `ArcCoyonedaBrand` dropped (deferred to a follow-up)

The brand-level
[`Foldable`](../../../fp-library/src/classes/foldable.rs)
impl on `ArcCoyonedaBrand` was dropped because
`Foldable::fold_map`'s trait method declares `A: Clone` only,
but the post-migration body needs `A: Send + Sync`. Rust
forbids tightening trait method bounds in impls, so the impl
cannot be salvaged without a parallel `SendFoldable` trait.
That trait is a separate follow-up; the inherent
`ArcCoyoneda::fold_map` method (now `F: SendFunctor`-bound)
covers the user-facing surface in the meantime.

### Migration scope

- [`fp-library/src/types/vec.rs`](../../../fp-library/src/types/vec.rs):
  added `SendFunctor` impl for `VecBrand` (byte-identical
  body to `Functor::map`'s, with tighter `Send + Sync` bounds).
- [`fp-library/src/types/arc_coyoneda.rs`](../../../fp-library/src/types/arc_coyoneda.rs):
  inner trait, three layer impls (Base, MapLayer, NewLayer),
  and public methods (`lower_ref`, `collapse`, `hoist`,
  `fold_map`, `bind`, `apply`, `lift2`) migrated;
  `From<ArcCoyoneda> for Coyoneda` bounds tightened to
  `F: SendFunctor` and `A: Send + Sync`; brand-level
  `Foldable` impl on `ArcCoyonedaBrand` dropped.
- [`fp-library/src/types/effects/interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs):
  dropped `+ Functor` from the ArcCoyoneda dispatch impl's
  `EBrand` bound (now just
  `EBrand: Kind_cdc7cd43dac7585f + SendFunctor + 'static`).
- [`fp-library/src/types/effects/arc_run.rs`](../../../fp-library/src/types/effects/arc_run.rs):
  added `A: Send + Sync` to the `lift_node` helper's
  where-clause.
- [`fp-library/tests/ui/arc_coyoneda_requires_send.stderr`](../../../fp-library/tests/ui/arc_coyoneda_requires_send.stderr):
  re-blessed because the `Send + Sync` rejection now points
  at the impl-block-level bound rather than the inner
  `Apply!` clone-bound.

### Cross-references

- [Original (b) ratification](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds):
  the first SendFunctor blocker resolution.
- [Option (c) re-ratification](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified):
  the parallel `SendStateBrand` design that surfaced this
  downstream issue.
- [Phase 2 step 9d resolution](#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family):
  the `ArcFree` migration precedent.
- [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md):
  the project-wide pattern table for Send-awareness gaps.

## Resolved (2026-05-03): Phase 3 step 6a SendFunctor reopened after option (b) unimplementable; option (c) parallel `SendStateBrand` ratified

The
[earlier 2026-05-03 ratification of option (b)](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds)
(commit `4bd1636`) locked in per-method `Send + Sync` bounds
at smart-constructor sites. Implementation of `ArcRun::get` /
`ArcRun::put` under that lock-in failed at compile time; the
blocker was reopened and re-ratified with option (c).

### Why option (b) was unimplementable

[`State<'a, P, S, A>`](../../../fp-library/src/types/effects/state.rs)
holds:

- `Get(<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(S) -> A>)`
- `Put(S, <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>)`

For `P = ArcBrand`, the projection is
`Arc<dyn 'a + Fn(...) -> A>`. The trait object's bounds are
`'a + Fn(...) -> A` only , no `+ Send + Sync` baked in. Since
`Arc<T>: Send + Sync` requires `T: Send + Sync` structurally,
and `dyn Fn(...)` (without `+ Send + Sync`) is structurally
`!Send + !Sync`, the projection
`Arc<dyn Fn(...)>: Send + Sync` is provably false at the type
level.

Concrete rustc error from the attempted implementation:

```
error[E0277]: `(dyn Fn(()) + 'static)` cannot be shared between threads safely
   = help: the trait `Sync` is not implemented for `(dyn Fn(()) + 'static)`
   = note: required for `Arc<(dyn Fn(()) + 'static)>` to implement `Sync`
```

Adding the bound
`<ArcBrand as RefCountedPointer>::Of<'static, dyn 'static + Fn(()) -> ()>: Send + Sync`
to the smart-constructor's where-clause does not satisfy
this: rustc rejects the bound because the underlying type is
structurally `!Send + !Sync`, and use-site bounds cannot
refine a structural type-level fact.

The (b) analysis conflated two superficially-similar cases:
the existing per-method `Send + Sync` bounds on Arc-family
`interpret_with` / `interpret_rec` work because they apply to
projections of _concrete generic structs_ like
`ArcFree<NodeBrand<R, S>, ...>`, where `Send + Sync` _can_ be
true for specific instantiations. `Arc<dyn Fn(...)>` is a
different beast: the dyn trait object's marker-trait bounds
are part of its type identity, so `dyn Fn(...)` and
`dyn Fn(...) + Send + Sync` are different types.

### Decision: option (c) parallel `SendStateBrand` / `SendState`

Add a parallel
[`SendStateBrand<P, S>`](../../../fp-library/src/brands.rs)
brand registration alongside `StateBrand<P, S>`, and a
parallel
[`SendState<'a, P, S, A>`](../../../fp-library/src/types/effects/state.rs)
enum whose variants use the Send-aware projection:

- `Get(<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(S) -> A + Send + Sync>)`
- `Put(S, <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>)`

For `P = ArcBrand`, the projection is
`Arc<dyn 'a + Fn(...) + Send + Sync>` , a different type from
the non-Send `Arc<dyn Fn(...)>` in `State`, and one that IS
`Send + Sync` because the trait object's bounds now include
the marker traits.

Brand-level `SendFunctor` impl on `SendStateBrand<P, S>` is
implementable because `SendRefCountedPointer::Of<'a, T>`'s
trait bound carries `T: ?Sized + Send + Sync + 'a`,
guaranteeing the projection is structurally `Send + Sync` for
any valid `T`. No HRTB-over-types needed.

The non-Arc smart constructors (`Run::get/put` /
`RunExplicit::get/put` / `RcRun::get/put` /
`RcRunExplicit::get/put`) keep using `StateBrand<P, S>` in
their rows; they cannot use `SendStateBrand` because
`RcBrand` does not implement `SendRefCountedPointer` (`Rc` is
`!Send`). The Arc smart constructors (`ArcRun::get/put` /
`ArcRunExplicit::get/put`) use `SendStateBrand<ArcBrand, S>`
in their rows.

### Why other alternatives were rejected (again)

**(a) HRTB-over-types**: still unimplementable on stable Rust.

**(b) Per-method bounds**: discovered unimplementable, see
above. The original (b) ratification stays in
[resolutions.md](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds)
as historical record.

**(d.1) Polymorphic-projection State**: parameterize
`State<'a, P, S, A>` over a "function-pointer-shape" trait
that returns `RefCountedPointer::Of<dyn Fn>` for non-Send
brands and `SendRefCountedPointer::Of<dyn Fn + Send + Sync>`
for Send brands. The "function-pointer-shape" trait would
need a per-`A`-and-`S` associated type to express the
parametric projection, which collapses back into the
HRTB-over-types wall.

**(d.2) Unconditional Send-aware representation**: change
`State<'a, P, S, A>` to use the Send-aware projection
unconditionally. Forces RcBrand users' closures to be
`Send + Sync`, breaking the canonical `Rc<RefCell<S>>`
capture pattern for state threading (`Rc` is `!Send`).
Rejected.

**(d.3) Two struct-level variants `State` and `SendState`**:
effectively option (c) under a different name. The chosen
implementation IS this; calling it (c) keeps continuity with
the original blocker analysis.

**(e) Defer-and-document**: viable, but the implementation
cost of (c) is small (~1 new brand, ~1 new enum, ~4 new impls
plus 4 smart-constructor methods, all parallel to existing
`State` shape), so the cost-benefit favors shipping over
deferring.

### Implementation phasing

1. New
   [`SendStateBrand<P, S>`](../../../fp-library/src/brands.rs)
   registration.
2. New `SendState<'a, P, S, A>` enum in
   [`state.rs`](../../../fp-library/src/types/effects/state.rs)
   alongside `State<'a, P, S, A>`. Or in a new
   `send_state.rs` module if `state.rs` grows too large.
3. `impl_kind!` for `SendStateBrand`.
4. `Functor` impl for `SendStateBrand`.
5. Manual `Clone` impl for `SendState` (gated on `S: Clone`,
   like the existing `State::Clone` impl from 5a.3).
6. `SendFunctor` impl for `SendStateBrand` (the whole point
   of (c) , this works because the projection is structurally
   `Send + Sync`).
7. `ArcRun::get<Idx>()` and `ArcRun::put<StateType, Idx>(s)`
   smart constructors using `SendStateBrand<ArcBrand, A>` in
   the row, plus `<ArcBrand as ToDynSendFn>::new(closure)`
   for continuation construction.
8. `ArcRunExplicit::get/put` same pattern.

The user-facing API surface for State is now:

- Single-thread programs: use `StateBrand<RcBrand, S>` (or
  `StateBrand<ArcBrand, S>` if the substrate happens to be
  Arc but you don't need thread-safety).
- Thread-safe programs that lift state-effect closures into
  Arc-substrate Run programs: use
  `SendStateBrand<ArcBrand, S>`.

Users with mixed programs face two distinct row brands they
must use depending on the substrate. The
[`define_effect!`](../../../fp-macros/src/effects/) macro
(Phase 3 step 7) can generate the per-wrapper smart
constructors that hide this distinction by selecting the
right brand per wrapper.

### Cross-references

- [Original (b) ratification](#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds):
  the superseded resolution.
- [Phase 3 step 5a.1 deviations entry](deviations.md): the
  original `State` design with `Functor`-only impl.
- [Phase 3 step 5a.3 deviations entry](deviations.md): the
  `State::Clone` impl that 6a.4 / 6a.6 cascade requires.
- [`SendRefCountedPointer`](../../../fp-library/src/classes/send_ref_counted_pointer.rs):
  the trait powering `SendState`'s projection.
- [`ToDynSendFn`](../../../fp-library/src/classes/to_dyn_send_fn.rs):
  parallel to `ToDynCloneFn`, used to construct Send-aware
  `dyn Fn + Send + Sync` continuations.

## Resolved (2026-05-03): Phase 3 step 6a `SendFunctor` impl on `StateBrand` for the Arc family (option (b) per-method bounds)

[`StateBrand<P, S>`](../../../fp-library/src/types/effects/state.rs)
shipped in step 6a.1 with the `Functor` impl only; the
`SendFunctor` impl was deferred. Without `SendFunctor`, the
Arc family smart constructors (`ArcRun::get/put` /
`ArcRunExplicit::get/put`, step 6a.4 and 6a.6) could not ship
via the same path that 6a.3 / 6a.5 use, because
`ArcCoyoneda`'s `Member::project` and `lower_ref` paths
require the inner projection to be `Send + Sync` per-`A`. The
bound
`<P as RefCountedPointer>::Of<'_, dyn 'a + Fn(S) -> A>: Send + Sync`
must be expressed for each `A` the smart constructor
produces, which hits stable Rust's HRTB-over-types limit (the
same constraint family that drove the brand-level
`SendFunctor` cascade gaps in Phase 2 step 9d / 9g / 9i).

### Background

[`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
adds `Send + Sync` bounds on the input/output types and the
closure to the `Functor::map` contract. The
[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
dispatch impl in
[`interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs)
requires `EBrand: SendFunctor` and
`<EBrand as Kind>::Of<'a, NextProgram>: Send + Sync + 'a`.

For `StateBrand<P, S>` with `P = ArcBrand`, the projection is
`State<'a, ArcBrand, S, A>`. The Get and Put variants hold
`<ArcBrand as RefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> A>`
= `Arc<dyn 'a + Fn(...) -> A>`. **This `Arc<dyn Fn>` is NOT
`Send + Sync` by default**: `RefCountedPointer::Of` provides
`Clone + Deref + 'a` but no `Send + Sync` guarantee. For the
`Arc<dyn Fn>` to be thread-safe, the inner `dyn Fn` must
carry `+ Send + Sync`.

### Decision: option (b) per-method `Send + Sync` bounds at smart-constructor sites

No brand-level `SendFunctor` impl on `StateBrand<P, S>`.
Instead, `ArcRun::get/put` and `ArcRunExplicit::get/put`
explicitly require
`<ArcBrand as RefCountedPointer>::Of<'_, dyn Fn(...)>: Send + Sync`
in their where-clauses for the specific `A` the constructor
produces. Callers see the bound at use sites; the
`SendFunctor::send_map` cascade through `ArcCoyoneda`
resolves because each `A` instantiation gets its own concrete
bound discharged.

This mirrors `ArcRunExplicit`'s precedent (every Arc-family
`interpret_with` / `interpret_rec` method already carries
~12 per-method `Send + Sync` clauses rather than struct-level
ones). Uniform precedent across the codebase.

### Alternatives considered and rejected

**(a) Brand-level `SendFunctor` impl with HRTB.** Would need

```rust
impl<S> SendFunctor for StateBrand<ArcBrand, S>
where
    S: Send + Sync + 'static,
    for<'a, A: Send + Sync + 'a> <ArcBrand as RefCountedPointer>::Of<
        'a,
        dyn 'a + Fn(S) -> A,
    >: Send + Sync,
{
    ...
}
```

The `for<'a, A>` HRTB-over-types is unsupported on stable
Rust. Same wall as Phase 2 step 9d / 9g / 9i. Unimplementable.

**(c) Parallel `SendStateBrand<P, S>` separate from
`StateBrand<P, S>`.** Define a second brand whose `Of<'a, A>`
is `SendState<'a, P, S, A>` with the inner pointer projection
typed via
[`SendRefCountedPointer::Of`](../../../fp-library/src/classes/ref_counted_pointer.rs)
(which carries `T: Send + Sync` in its bound). The Arc smart
constructors use `SendStateBrand` in the row; non-Arc
constructors use `StateBrand`. Two state types, two row-brand
entries.

Rejected because:

- Doubles the type surface; users with mixed
  single-thread / thread-safe code in the same program face
  two state types they must convert between.
- The current `StateBrand<P, S>` parameterization was chosen
  specifically to give one State type across all six
  wrappers; splitting it would partially undo that design.

**(d) Use `SendRefCountedPointer::Of` directly in `State`'s
representation.** Bake `+ Send + Sync` into the `dyn Fn(...)`
trait object in the State enum's variants. Rejected because:

- Forces RcBrand single-thread users to provide
  `Send + Sync` closures for State, which **breaks the
  canonical `Rc<RefCell<S>>` capture pattern** for state
  threading (`Rc` is `!Send`).
- The workaround is parallel `ToDynCloneFn::new_send`
  machinery, which collapses into option (c) with extra
  steps.

**(e) Defer-and-document.** Don't ship 6a.4 / 6a.6; document
the closure-capture state pattern (the same pattern F1D
codified for `interpret`) as the canonical way to thread
state through `ArcRun` / `ArcRunExplicit`.

Rejected because:

- The locked-in design from the
  [2026-05-03 wrapper-parameterization resolution](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization)
  signaled "six variants per effect" intent; deferring would
  partially unfulfill that lock-in.
- The implementation cost of (b) is small (~80 lines per
  wrapper plus where-clause noise), so deferring buys little
  while delaying a complete State surface.

### Implementation phasing

Under (b):

- `state.rs` keeps the `Functor` impl on `StateBrand<P, S>`;
  no `SendFunctor` impl is added.
- `ArcRun::get/put` and `ArcRunExplicit::get/put` add
  per-method bounds:
  - `<ArcBrand as RefCountedPointer>::Of<'_, dyn 'a + Fn(S) -> A>: Send + Sync`
  - `<ArcBrand as RefCountedPointer>::Of<'_, dyn 'a + Fn(()) -> A>: Send + Sync`
  - `S: Send + Sync` (already required for `ArcCoyoneda::lift`).
- `ArcRun::interpret_with::<StateBrand<ArcBrand, S>>` users
  get these bounds propagated into their where-clauses.
  Documented as a known cost in deviations.md and the `Arc
family` per-wrapper notes.

### Cross-references

- [Per-`A` HRTB-over-types blocks brand-level type-class
  delegation](prompt.md#per-a-hrtb-over-types-blocks-brand-level-type-class-delegation):
  the broader pattern this blocker instantiates.
- [Phase 2 step 9d resolution](#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family):
  the per-method workaround precedent.
- [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md):
  the project-wide table of HRTB-over-types-blocked
  brand-level cascades.
- [Phase 3 step 5 wrapper-parameterization
  resolution](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization):
  the locked-in "six variants per effect" design that this
  resolution implements for the Arc family.

## Resolved (2026-05-03): Adversarial review reversals (delete `run_accum`, ship `interpret_with_rec`, parameterise `interpret_with` over `RefCountedPointer`)

An adversarial review of the WIP effects implementation
(commissioned 2026-05-03;
[review_effects_rs.md](review/0_first_order_effects_implementation/review_effects_rs.md))
filed 5 fundamental, 8 major, and 9 minor findings. The
remediation analysis
([remediation_proposals.md](review/0_first_order_effects_implementation/remediation_proposals.md))
recommended three changes that overturn prior locked-in
decisions in this file. This entry ratifies those three
reversals and pre-records the remaining recommendations as a
Phase 3 cleanup step.

### Reversal 1: delete `run_accum` and `run_accum_rec` entirely (review F1D)

**Prior decision:**
[Q3 (2026-05-02): closure-capture state threading with `init`
parameter](#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading)
locked the `run_accum` family at "accept `init`, thread state
via user-side closure captures, return `A`". The implementation
honours that lock by writing `let _ = init; self.interpret(handlers)`
across all six wrappers.

**Review finding:** F1 (fundamental). With state threading
delegated to user-side captures, the `init` parameter is
vestigial; the function body is byte-equivalent to `interpret`'s.
The signature accepts `init` but does nothing with it. The
function name advertises state threading the implementation
cannot deliver in the mono-in-A return-type encoding.

**Reversal:** Delete `run_accum` and `run_accum_rec` from all six
wrappers. Document the closure-capture state pattern on
[`interpret`](../../../fp-library/src/types/effects/run.rs#L497)'s
rustdoc with the keyword "state" so rustdoc-search picks it up.
The closure-capture state pattern itself is unchanged; only the
vestigial second method name is removed.

**Why the prior reasoning no longer holds:** The Q3 lock-in
recorded "the latter is documented as the convention for
state-threading uses". That role was a documentation hook, not a
semantic distinction. The same hook lives equally well as a
rustdoc paragraph on `interpret`. The PureScript Run parity
argument (literal `runAccum` naming) is a convention rather than
a constraint, and the project has departed from PureScript naming
elsewhere when the Rust shape diverges (e.g., `im_do!` for
inherent monadic do, no PureScript analogue).

**Forward compatibility:** When StateT lands in Phase 6+ per
[plan.md decisions row 1207](plan.md#L1207), the natural entry
point is `interpret_rec::<StateT<S, IdBrand>>`, not a re-purposed
`run_accum` slot. Keeping the slot would invite future confusion;
deleting it forecloses that.

### Reversal 2: ship `interpret_with_rec::<MBrand, EBrand>` (review M2A)

**Prior decision:**
[Decision 4 (2026-04-29): Phase 6+ deferred entry for
`interpret_with<M: Monad>`](#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape)
deferred the pipeline-plus-MonadRec combination on the grounds of
"no current user demand" and "axis 3 alternative branch".

**Review finding:** M2 (major). `interpret_with`'s recursion is
host-stack-frame per peeled layer
([run.rs:1032-1054](../../../fp-library/src/types/effects/run.rs#L1032-L1054)),
so users with deep eager-recursing effect chains who also need
row narrowing have no stack-safe option. The current escape
hatch (flatten into `interpret_rec`) forfeits the row-narrowing
benefit; the two shapes are not interchangeable.

**Reversal:** Ship the pipeline-plus-MonadRec combination as
Phase 3 step 5, sitting between the simple/pipeline/rec
interpreter families (steps 2-4) and the standard first-order
effects step (step 6). Per-wrapper inherent method
`interpret_with_rec::<MBrand, EBrand, Idx, RMinusE>` returning
`M::Of<'_, Run<RMinusE, CNilBrand, A>>`, internally driven by
[`tail_rec_m`](../../../fp-library/src/classes/monad_rec.rs).
Closes the orthogonality grid: simple, pipeline, MonadRec,
pipeline+MonadRec. The renumbering shifts the in-flight
"standard first-order effects" step from 5 to 6 and consequent
sub-step labels from 5a to 6a (per the project's preference for
clean numerical ordering over append-only step preservation).

**Why the prior reasoning no longer holds:** The "no current
user demand" reasoning was driven by the absence of standard
first-order effects (which only Phase 3 step 5 provides). With
Phase 3 step 5 in progress and the standard effects (`State`,
`Reader`, `Except`, `Writer`, `Choose`) about to land, the
demand-floor is no longer "no users"; it is "every user with a
deep program who wants pipeline narrowing". The original
deferral was precautionary; the review made the trade-off
visible.

### Reversal 3: parameterise `interpret_with` over `P: RefCountedPointer` (review M3C)

**Prior decision:** Implicit. The current `interpret_with`
implementation
([run.rs:1003-1063](../../../fp-library/src/types/effects/run.rs#L1003-L1063))
requires the handler closure to be `Fn + Clone + 'static` (plus
`Send + Sync` on Arc wrappers); the per-recursion clone is the
mechanism for sharing the handler across sub-program narrowings.
No prior resolution covered this; the choice landed in the
[Phase 3 step 3 deviations entry](deviations.md#L2059-L2061)
without an alternative survey.

**Review finding:** M3 (major). The `Clone` bound rules out
handler closures that capture unique resources (e.g., a
`BufWriter` acquired in scope). Users hit the bound at
`interpret_with` call sites and must wrap their captures in
`Rc<RefCell<_>>` themselves.

**Reversal:** Parameterise `interpret_with` over a pointer brand
`P: RefCountedPointer` (the existing trait at
[`ref_counted_pointer.rs`](../../../fp-library/src/classes/ref_counted_pointer.rs)).
Wrap the handler in `P::Of<F>` once at entry; clone the pointer
(cheap refcount bump) on each recursion. The four non-Arc
wrappers thread `RcBrand`; the two Arc wrappers thread
`ArcBrand`. The wrapper-level public method fixes `P` so users
see no extra parameter at the call site. Drops the user-facing
`Clone` bound; the bound becomes `Fn + 'static` (plus
`Send + Sync` on Arc wrappers).

**Why the prior reasoning no longer holds:** The implicit choice
predated the
[`RefCountedPointer`](../../../fp-library/src/classes/ref_counted_pointer.rs)
trait's load-bearing role in
[Phase 3 step 5a's State effect parameterisation](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization),
which set the convention "abstract per-effect machinery over
`P: RefCountedPointer` so one definition serves both refcount
families". `interpret_with` is currently the only Phase 3
machinery that does not follow that convention; aligning it
removes a per-wrapper hard-code and a user-facing bound in one
move.

### Locked-in resolution set: F1D + F3A + M3C cleanup, then M2A

The three reversals fold into the plan's Phase 3 step list
through in-place revisions to existing steps (F1D, F3A, M3C
become "what step 2/3/4 say going forward") plus one new step
(M2A becomes step 5). Land order:

1. **Reversal cleanup** (small): F1D + F3A + M3C land together
   as one commit. F1D deletes `run_accum` / `run_accum_rec`
   from steps 2 and 4 (12 method signatures + 12 doctests).
   F3A tightens the `S` bound to `CNilBrand` on the
   `interpret`, `interpret_with`, and `interpret_rec` families
   (18 wrapper-method bodies; removes the
   `clippy::unreachable`-suppressed panic in the
   `Node::Scoped(_)` arms). **Phase 4 addendum (2026-05-05):**
   the `S = CNilBrand` tightening was correct for Phase 3
   closure but assumed Phase 4 would specify a parallel
   dispatch mechanism. Per [decisions.md section 4.5's
   "Dispatcher trait shape and substrate-level interpose
   primitive" sub-decision](decisions.md), Phase 4 lifts the
   `S = CNilBrand` bound on the interpret family and routes
   `Node::Scoped` through the new `DispatchScopedHandlers`
   trait; the F3A invariant is therefore Phase-3-only. M3C parameterises step 3's
   `interpret_with` over `P: RefCountedPointer` (drops the
   user-facing `Clone` bound; pairs naturally with [m9
   interpreter dispatch impl deduplication](review/0_first_order_effects_implementation/remediation_proposals.md#minor-findings)).
   Lands before resuming step 6a.3 (next pending sub-step of
   the standard first-order effects work) so step 6 does not
   inherit the issues.
2. **M2A: new step 5** (medium): six new method bodies
   `interpret_with_rec` plus integration tests. Sequence after
   step 6 completes so the standard first-order effects can
   drive the integration tests.

### Remaining review recommendations: tracked as Phase 3 step 9

The review's other recommendations
([remediation_proposals.md](review/0_first_order_effects_implementation/remediation_proposals.md))
are non-reversals and do not require ratification here. They
bundle into the new Phase 3 step 9 (review-remediation
documentation pass): F2A, F4A, F5A, M4 audit, M6A
async-via-`spawn_blocking` doc, M7A bind/handler asymmetry
note, and all minor m1-m9 items. Lands as one commit before
public release. The
[`SendFunctorAt`](review/0_first_order_effects_implementation/remediation_proposals.md#m5-sendfunctor-for-statebrand-is-deferred-multi-thread-state-is-unimplemented)
spike on State for the Arc family is a sub-task of step 6
(standard first-order effects), not step 9, since it gates the
ArcRun State Success criterion.

### Cross-references

- [`review_effects_rs.md`](review/0_first_order_effects_implementation/review_effects_rs.md): the
  adversarial review report ($9fd2bb8$) that surfaced the
  findings.
- [`remediation_proposals.md`](review/0_first_order_effects_implementation/remediation_proposals.md):
  the per-finding options + recommendations ($1bfb9f0$) that
  proposed these reversals.
- [Q3 (2026-05-02)](#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading):
  the closure-capture-state lock-in that F1D updates.
- [Decision 4 (2026-04-29)](#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape):
  the Phase 6+ deferral that M2A overturns.
- [Phase 3 step 5 (2026-05-03)](#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization):
  the
  [`RefCountedPointer`](../../../fp-library/src/classes/ref_counted_pointer.rs)
  parameterisation convention that M3C aligns with.
- [Phase 3 step 3 deviations
  entry](deviations.md#L2059-L2061): the implicit `Clone`-bound
  decision that M3C overturns.

## Resolved (2026-05-03): Phase 3 step 5 smart-constructor wrapper parameterization

Phase 3 step 4 shipped as `bd540d5` + `fafcfde`. Step 5
(standard first-order effect types and smart constructors:
`State<S>`, `Reader<E>`, `Except<E>`, `Writer<W>`, `Choose`)
is the next work. Five sub-decisions are entangled with the
top-level wrapper-parameterization question; all five shape
the public API and the Phase 3 step 6 `define_effect!` macro
emit shape, and were answered before implementation begins.

### Background

PureScript Run ships `ask :: Run (READER e r) e`, `get :: Run (STATE s r) s`,
etc. , single entry-points, no wrapper choice. PureScript has
one `Run` type, so the question doesn't arise.

fp-library has six Run wrappers
([`Run`](../../../fp-library/src/types/effects/run.rs),
[`RcRun`](../../../fp-library/src/types/effects/rc_run.rs),
[`ArcRun`](../../../fp-library/src/types/effects/arc_run.rs),
[`RunExplicit`](../../../fp-library/src/types/effects/run_explicit.rs),
[`RcRunExplicit`](../../../fp-library/src/types/effects/rc_run_explicit.rs),
[`ArcRunExplicit`](../../../fp-library/src/types/effects/arc_run_explicit.rs))
because Rust requires substrate choices around continuation
function-pointer kind (`Box<dyn FnOnce>` vs `Rc<dyn Fn>` vs
`Arc<dyn Fn + Send + Sync>`), `'static` vs `'a` payload, and
type-erasure vs concrete-recursive-enum representation. The
[Phase 2 step 9h per-wrapper Coyoneda variant pairing
rule](plan.md#earlier-completed-steps-commit-log) locks each
wrapper to one Coyoneda variant. So smart constructors that
produce `Wrapper<R, S, A>` cannot be wrapper-polymorphic
without abstracting over the substrate cascade.

A second consideration: effects must compose. If a user wants
both `Choose` (multi-shot) and `State` (any wrapper) in the
same program, they must pick a multi-shot wrapper for the whole
program. So `State` has to be available in _whichever wrapper
supports `Choose`_ (i.e., `RcRun` / `ArcRun`), not just on a
canonical default like `Run`. This rules out a simple "ship each
effect on one canonical wrapper" approach.

A third consideration: Phase 3 step 6's `define_effect!` macro
generates effect types + smart constructors at user request.
Whatever shape step 5 picks for the hand-rolled standard
effects, step 6's macro must emit the same shape. So the
decision cascades.

### The five questions and their resolutions

**Q1 (top-level wrapper parameterization shape, confirmed (b) six variants per effect).**
Each effect ships as inherent methods on each wrapper (or in
per-wrapper modules), e.g., `Run::ask`, `RcRun::ask`,
`ArcRun::ask`, `RunExplicit::ask`, `RcRunExplicit::ask`,
`ArcRunExplicit::ask`. Verbose at face value, but the
`define_effect!` macro (Phase 3 step 6) mechanically generates
the six variants from one user declaration, hiding the
verbosity at user-code level.

Alternatives considered:

- **(a) Wrapper-parameterized.** One `ask::<W, R, ..., E, Idx>(...)`
  generic over `W: RunWrapper` (a new trait abstracting the
  lift behaviour). All wrappers implement the trait. Pros: one
  entry-point per effect; symmetric across wrappers; mirrors
  PureScript's single-Run shape most closely. Cons: requires
  defining a `RunWrapper` trait that captures the substrate
  cascade (Coyoneda variant choice, `lift` signature,
  where-clause cascade); every effect type must be parameterised
  by the wrapper's function-pointer kind via
  [`FnBrand`](../../../fp-library/src/types/fn_brand.rs);
  user-facing turbofish at call sites in non-inferrable cases;
  trait machinery is non-trivial because the six wrappers have
  meaningfully different bound cascades (`A: Clone` for Rc
  family; `A: Send + Sync` for Arc family; etc.).
- **(c) Canonical wrapper per effect family.** Pick one per
  effect: e.g., State/Reader/Except/Writer ship on `Run`;
  Choose ships on `RcRun`. Users convert between wrappers via
  existing `From` impls when needed. Pros: minimal API surface;
  one entry-point per effect. Cons: broken by the composition
  argument: if a user wants Choose + State, they need
  `RcRun`-shaped State, not `Run`-shaped. So the user would
  have to convert `Run::get()` into an `RcRun` program, but the
  row brand is fixed at the wrapper, so there's no clean
  conversion. (c) requires either ad-hoc conversion machinery
  or just falls back to (b) under the hood for the multi-shot
  case. Not a coherent design as stated.

The principled argument: the precedent of every other Phase 3
step shipping six wrapper-specific inherent methods is strong
(step 2's `interpret`/`run`/`run_accum`, step 3's
`interpret_with`/`extract`, step 4's
`interpret_rec`/`run_rec`/`run_accum_rec`). (a) would be the
first time we introduce a wrapper-abstracting trait; the cost
(substrate-cascade abstraction; complex bounds; turbofish
proliferation) outweighs the cosmetic benefit of one entry-
point. (c) is fundamentally incoherent because of effect
composition.

**Q2 (effect-type representation, confirmed (a) per-effect Functor instance).**
Each effect type holds its continuation directly:

```rust
enum State<FnP: FnBrand, S, A> {
    Get(FnP::Of<dyn FnOnce(S) -> A>),
    Put(S, FnP::Of<dyn FnOnce(()) -> A>),
}
```

Requires per-effect `Functor` impl (and `SendFunctor` for the
Arc family). The function-pointer kind is wrapper-specific
(`Box` for Run/RunExplicit; `Rc` for Rc family; `Arc` for Arc
family) and threaded via `FnBrand` (see Q3).

Alternatives considered:

- **(b) Continuation-free effect type, Coyoneda holds the
  continuation.** Effect types carry only the operation
  discriminator: `enum State<S, A> { Get(PhantomData<A>), Put(S, PhantomData<A>) }`.
  Coyoneda's `lift(fa)` stores `fa: F::Of<'_, A>` plus an
  identity function. After `map`, Coyoneda accumulates
  continuations. The catch: in (b), the "underlying value"
  inside Coyoneda is not actually `A`; it's `S` for Get and
  `()` for Put. Coyoneda's type signature is `Coyoneda<F, A>`
  where `F::Of<A>` is the inner value. With (b),
  `F::Of<A> = State<S, A>` which is just a tag , but Coyoneda's
  stored `f: B -> A` would then be `B = A`, so the function is
  identity, and the handler must manually invoke its own
  continuation by inspecting the variant. Workable but loses
  the Functor abstraction.

The principled argument: matches PureScript's structure
directly. The `Choose<A> = Choose Boolean a` shape inherently
needs the continuation in the type because the handler runs
both branches; (b) would force handlers to track the
continuation outside the effect type, breaking the abstraction
asymmetrically across effects.

**Q3 (function-pointer-kind threading, confirmed (a-1) `FnBrand`-parameterised).**
Single effect type per effect, parameterised by `FnBrand`:

```rust
enum State<FnP: FnBrand, S, A> { ... }
```

Smart constructors thread `FnP` per-wrapper. Matches existing
`FnBrand`-based code (e.g.,
[`RcFree`/`ArcFree`](../../../fp-library/src/types/rc_free.rs)
already use `FnBrand`-shaped continuations).

Alternatives considered:

- **(a-2) Per-wrapper effect types.** `RunState<S, A>`,
  `RcState<S, A>`, `ArcState<S, A>`, etc., each hard-coded to
  its substrate's function-pointer. ~30 named effect types per
  effect = 30+ types per effect. Massive duplication; rejected.
- **(a-3) Single effect type with `Box<dyn FnOnce>` always;
  convert at lift site.** The effect type stores `Box`; for
  Rc/Arc lifts, convert to `Rc`/`Arc` wrapping at lift time.
  Requires conversion machinery; loses the `FnBrand` precedent.
  Rejected as ad-hoc.

**Q4 (Choose's wrapper coverage, confirmed (ii) all four multi-shot wrappers).**
Choose is intrinsically multi-shot (handler runs both branches
of the choice and combines results). Continuation must be
cloneable. So Choose ships on `RcRun`, `RcRunExplicit`,
`ArcRun`, `ArcRunExplicit`; not on `Run` or `RunExplicit`.

Plan text said "Choose (multi-shot, `RcRun`-only)" , overly
narrow. Plan text needs updating to reflect all four
multi-shot wrappers.

**Q5 (row-brand notation, confirmed (b) `effects!` only initially).**
Users compose row brands via the
[`effects!`](../../../fp-macros/src/effects/effects_macro.rs)
macro: `effects!(ReaderBrand<FnP, E>, StateBrand<FnP, S>)`. No
per-effect type aliases (e.g., `type ReaderRow<E, R> = ...`)
ship in step 5; can be added later if users complain.

Alternatives considered:

- **(a) Ship row-alias type aliases per effect.** E.g.,
  `type ReaderRow<'a, E, R> = CoproductBrand<CoyonedaBrand<ReaderBrand<'a, E>>, R>;`
  Multiplies the API surface and locks in naming choices.
- **(c) Both.** Ship aliases for ergonomics; users can also
  use `effects!`. Reasonable but adds maintenance overhead;
  defer until users complain.

The principled argument: the `effects!` macro is the canonical
composition path; per-effect aliases add API surface and
naming-debate exposure for marginal gain.

### Locked-in resolution set: (1.b) + (2.a) + (3.a-1) + (4.ii) + (5.b)

The combination ships step 5 as wrapper-symmetric inherent-
method variants on per-wrapper modules (precedent-matching),
with `FnBrand`-parameterised effect types (substrate-agnostic),
and row-brand composition via the existing `effects!` macro.

### Sub-decisions summary

| #   | Question                        | Resolution                                                                                        |
| --- | ------------------------------- | ------------------------------------------------------------------------------------------------- |
| 1   | Wrapper parameterization        | (b) six variants per effect; precedent-matching; `define_effect!` macro hides verbosity user-side |
| 2   | Effect type representation      | (a) per-effect Functor; matches PureScript; needed for `Choose`'s multi-branch continuation       |
| 3   | Function-pointer-kind threading | (a-1) `FnBrand`-parameterised; matches `RcFree`/`ArcFree`'s precedent; single effect type per     |
| 4   | Choose's wrapper coverage       | (ii) all four multi-shot wrappers; plan text updated                                              |
| 5   | Row-brand notation              | (b) `effects!` only; per-effect aliases deferred until user demand                                |

### Implementation phasing under the locked-in set

- **Step 5 ships 5 effect types** (`State<FnP, S, A>`,
  `Reader<FnP, E, A>`, `Except<E, A>`, `Writer<FnP, W, A>`,
  `Choose<FnP, A>`) parameterised by `FnBrand` where the
  effect carries continuations.
- **Step 5 ships ~34 smart constructors:**
  - State: `get` and `put` on each wrapper. 2 \* 6 = 12.
  - Reader: `ask` on each wrapper. 1 \* 6 = 6.
  - Except: `throw` on each wrapper. 1 \* 6 = 6.
  - Writer: `tell` on each wrapper. 1 \* 6 = 6.
  - Choose: `choose` on each multi-shot wrapper. 1 \* 4 = 4.
  - Total: ~34 named smart constructors, distributed across
    six per-wrapper modules.
- **Step 5 may be split into sub-steps** per the
  [implementation protocol](plan.md#implementation-protocol)'s
  oversized-step rule; one effect per sub-step is the natural
  cut (5a State, 5b Reader, 5c Except, 5d Writer, 5e Choose).
  Surface the split decision to the user before starting.
- **Step 6's `define_effect!` macro** mechanically generates
  the six per-wrapper variants from a single user declaration:
  ```rust
  define_effect! {
      Reader<E> {
          fn ask() -> E,
      }
  }
  ```
  expands to per-effect type + 6 per-wrapper smart-constructor
  bodies.
- **`Choose` opts out of the single-shot wrappers** at the
  `define_effect!` macro level via a `multi_shot` attribute or
  similar. Step 6's design needs to account for this.

### Cross-references

- [Phase 2 step 9h pairing rule](plan.md#earlier-completed-steps-commit-log):
  locks each wrapper to one Coyoneda variant; root cause of
  wrapper proliferation.
- [`FnBrand`](../../../fp-library/src/types/fn_brand.rs):
  precedent for substrate-abstracted function-pointer kinds.
- [decisions.md](decisions.md) section 4.6: covers natural-
  transformation handler shape but does not pre-decide the
  wrapper-parameterization question.
- The lessons-learned section in [prompt.md](prompt.md)'s
  "Coyoneda variant pairing rule" subsection explicitly flags
  this question.

## Resolved (2026-05-02): Phase 3 step 4 interpreter design (handler shape, dispatch-trait reuse, state threading)

Phase 3 step 3 shipped as `ff84f20`. Step 4 (the
`MonadRec`-target interpreter family `interpret_rec` / `run_rec`
/ `run_accum_rec`) is the next work. Three load-bearing design
questions surfaced during step-4 scoping; all three shape the
public API and the dispatch-trait reuse and were answered
before implementation began.

### The three questions and their resolutions

**Q1 (handler input/output shape, confirmed (A) mirror PureScript).**
Handler is
`Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Run<R, S, A>>>) -> M::Of<'_, Run<R, S, A>>`.
The interpreter does
`<R as Functor>::map(M::pure, peel_layer)` to lift the peeled
layer's `Run<R, S, A>`-continuations to
`M::Of<Run<R, S, A>>`-continuations before dispatch. Reuses
step 2's
[`DispatchHandlers<'a, Layer, NextProgram>`](../../../fp-library/src/types/effects/interpreter.rs)
trait unchanged, instantiated with
`NextProgram = M::Of<Run<R, S, A>>`. Handlers can do real
`M::bind`-monadic work between effect dispatches (e.g., short-
circuit on `M = ResultBrand` via `Result::bind`).

Alternatives considered:

- **(B) Rust-flavoured simplification.** Handler input matches
  step 2's raw shape
  `Fn(<EBrand as Kind>::Of<'_, Run<R, S, A>>) -> M::Of<'_, Run<R, S, A>>`.
  Output is M-wrapped only. Requires a new
  `DispatchHandlersRec<'a, Layer, NextProgram, MBrand>` trait
  parallel to `DispatchHandlers`. The user-facing handler shape
  is simpler (no `M::Of<...>` ceremony in the input type), but
  the handler cannot perform M-monadic operations on the
  continuation before dispatch.
- **(C) Hybrid: M-wrapped input, raw Run output.** Handler is
  `Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Run<R, S, A>>>) -> Run<R, S, A>`.
  Loses access to `M::bind` inside handlers, defeating most of
  the reason to ship a MonadRec-target form.

The principled argument: M-monadic work in handlers IS the
load-bearing capability that distinguishes step 4 from step 2.
(B) and (C) lose that capability for a small cosmetic
simplification. (A) preserves it and reuses the existing
dispatch trait without ceremony.

**Q2 (`DispatchHandlers::dispatch` `&mut self` vs `&self`, confirmed (A1) relax to `&self`).**
Step 4's body invokes dispatch inside a `tail_rec_m` step
function whose closure is `Fn` (not `FnMut`); the closure
captures the handler list and calls `dispatch` on each
iteration. With `&mut self`, the captured list cannot be
borrowed mutably across iterations from inside a `Fn`-bound
closure. The relaxation lands as the first commit in step 4
implementation.

Alternatives considered:

- **(A2) Clone handlers per iteration.**
  `HandlersCons<H, T>: Clone` already holds for `H, T: Clone`,
  which `Handler<E, F>: Clone` satisfies for `F: Clone`. The
  step closure clones the handler list each iteration. Per-
  iteration overhead proportional to chain depth (each clone
  is O(N) in handler-list length, so O(N x chain depth)
  total). Acceptable for short handler lists but unbounded.
- **(A3) Wrap in `RefCell` inside the step closure.** The step
  closure captures
  `RefCell::new(handlers)`; each iteration calls
  `handlers.borrow_mut().dispatch(layer)`. Hides the issue at
  the call site rather than fixing it; ergonomically bad and
  not a clean shape.

The principled argument: all current handler closures across
the codebase use interior mutability for state (e.g.,
[`run_interpret.rs`](../../../fp-library/tests/run_interpret.rs)'s
`run_accum`-via-`Rc<RefCell>` tests use closures that Rust
infers as `Fn` because the mutation goes through
`RefCell::borrow_mut(&self)`). Step 2's
[`Handler<E, F>`](../../../fp-library/src/types/effects/handlers.rs)
carrier holds `F` opaquely; the `Fn` constraint is added at
the dispatch trait's impl bounds, not at the carrier. Relaxing
to `&self` matches actual usage and adds zero per-iteration
overhead. The risk (relaxation breaks an existing handler) is
bounded and easily verified via `just verify`. (A2) and (A3)
are workarounds for a constraint that doesn't actually bind.

**Q3 (state threading in `run_accum_rec`, confirmed (A) closure-capture).**
State threading uses
[`Rc<RefCell<S>>`](https://doc.rust-lang.org/std/cell/struct.RefCell.html)
or
[`Arc<Mutex<S>>`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
closure captures at the user level, matching step 2's
[`run_accum`](../../../fp-library/src/types/effects/run.rs)
shape. The `init` parameter is moved into the user's chosen
state cell internally and is otherwise ignored. The only
difference between
`interpret_rec` and `run_accum_rec` is that the latter is
documented as the convention for state-threading uses; no
separate stateful trait machinery.

Alternatives considered:

- **(B) State-via-M (PureScript-mirroring).** Add a separate
  stateful trait or require `M = StateT<S, MInner>` for some
  inner monad. Mirrors PureScript more directly. Requires
  fp-library to ship `StateT` (currently absent) or a similar
  state-monad transformer. Doubles the trait machinery.
  Deferred to Phase 6+; revisit when the demand surfaces.
- **(C) Punt to Phase 6+.** Drop `run_accum_rec` from step 4
  entirely; document it as deferred. Loses per-method API
  parity with step 2.

The principled argument: parity with step 2 minimises
cognitive load and keeps the trait machinery thin. (B) is
genuinely useful for some `MonadRec` instances (especially
when the target M wants to maintain a state that's accessible
from inside `M::bind`-monadic handler code, not just from
between effect dispatches), but requires `StateT` which is a
separate Phase 6+ addition.

### Locked-in resolution set: (A) + (A1) + (A)

The combination ships step 4 with maximum ergonomic and
capability symmetry to step 2, mirrors PureScript Run as the
upstream design intent, and introduces no new trait ceremony.

### Implementation order under the locked-in set

1. (Q2 = A1.) Refactor
   [`DispatchHandlers::dispatch`](../../../fp-library/src/types/effects/interpreter.rs)
   from `&mut self` to `&self` and confirm `Handler::F: Fn`
   bound suffices for the existing dispatch impls. Land as
   the first commit in step 4 (mechanical refactor; should
   not break any existing tests).
2. Add `interpret_rec` / `run_rec` / `run_accum_rec` per-
   wrapper inherent methods (parallel to step 2's
   `interpret`/`run`/`run_accum`). Each wrapper's body uses
   `tail_rec_m::<MBrand, _, _>(step_fn, self.into())`. Step
   function: peel current program, dispatch on `Node::First`
   via the (now `&self`)
   [`DispatchHandlers`](../../../fp-library/src/types/effects/interpreter.rs)
   trait (with `NextProgram = M::Of<Run<R, S, A>>` after
   pre-dispatch lifting via
   `<R as Functor>::map(M::pure, peel_layer)`), fmap
   `M::Of<NextProgram>` to `M::Of<ControlFlow<Continue, Break>>`.
   ArcRun reuses
   [`unwrap_first`](../../../fp-library/src/types/effects/arc_run.rs)
   for the HRTB-poisoning workaround. The
   [`make_node_first`](../../../fp-library/src/types/effects/arc_run.rs)
   /
   [`wrap_first_arc`](../../../fp-library/src/types/effects/arc_run.rs)
   helpers from step 3 are not needed for step 4 (no narrowed
   Run construction in scope; the rec form returns `M::Of<A>`
   directly).
3. Per-wrapper bounds cascade: `MBrand: MonadRec`; for the Arc
   wrappers, also `M::Of<'_, Run<...>>: Send + Sync` and the
   per-projection cascade.
4. Integration tests in
   `fp-library/tests/run_interpret_rec.rs` covering each
   wrapper x several `M` choices (`ThunkBrand`, `OptionBrand`,
   `ResultBrand`); doctests on each method.
5. Update plan.md's `Current progress` (rolling-detail entry
   for step 4; demote step 1 to commit log per the rolling-
   detail trim window of ~3 narratives).
6. Append deviations.md entry for step 4.

### Cross-references

- [decisions.md](decisions.md) section 4.3 ("Ship both
  interpreter families"): the original commitment that frames
  step 4 as a `MonadRec`-target sibling of step 2.
- [`fp-library/src/classes/monad_rec.rs`](../../../fp-library/src/classes/monad_rec.rs):
  fp-library's `MonadRec` trait + `tail_rec_m` free function.
- [`fp-library/src/types/effects/interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs):
  `DispatchHandlers` trait that step 4 reuses (after Q2 = A1
  relaxation).
- [`fp-library/src/types/effects/handlers.rs`](../../../fp-library/src/types/effects/handlers.rs):
  `Handler<E, F>` carrier; `F: Fn` constraint moves from impl
  bounds to the relaxed dispatch signature.
- Phase 3 step 3 commit `ff84f20` (pipeline row-narrowing +
  empty-row extract).
- Phase 3 step 2 commit `d5efe2a` (`interpret` / `run` /
  `run_accum` family) -- the API whose shape step 4 mirrors.
- [Resolved (2026-04-29): Phase 3 step 2/3 interpreter family shape](#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape)
  -- the prior resolution that locked in step 4's role as the
  third orthogonal interpreter primitive (M-target via
  `tail_rec_m`).
- [PureScript Run](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
  source.
- [PureScript MonadRec](https://github.com/purescript/purescript-tailrec/blob/master/src/Control/Monad/Rec/Class.purs)
  source.

## Resolved (2026-04-29): Phase 3 step 2/3 interpreter family shape

Phase 3 step 2 (`d5efe2a`) shipped `interpret` / `run` /
`run_accum` on the six Run wrappers with the target monad
implicit (`M = self Run wrapper`, returns `A` directly via a
while-loop). [decisions.md](decisions.md) section 4.3 frames
both step 2 (`Monad m`) and step 3 (`MonadRec m`) as exposing
the target monad as a parameter, mirroring PureScript Run's
`run` / `runRec`. The blocker question: should step 2 be
reshaped to match decisions.md 4.3, and what shape should
step 3 take?

A subsequent close read of
[PureScript Run](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
plus [heftia](https://github.com/sayo-hs/heftia)'s
interpreter machinery widened the question into three
orthogonal axes: which interpreter functions ship (axis 1),
handler shape algebraic vs return-next-program (axis 2), and
rec/non-rec for the externally-targeted M family (axis 3).
The original blocker was just axis 3; the widened scope made
axes 1 and 2 explicit.

Five decisions ultimately gated the resolution. The full
analysis lives in plan.md across commits `9f9e07b` (initial
blocker), `8e59bb5` (widened-scope analysis), `05539af`
(per-decision approaches and trade-offs), `f3148b2` (clean
rewrite for readability), and `35ceeee` / `ccc66c9` /
`5757796` / `86a544f` / `855f85c` (per-decision lock-ins).

### The five decisions and their resolutions

**Decision 1 (axis 1 widening, confirmed (1.A) Full widen).**
Schedule `extract(self) -> A` (empty-row pure extract) and
`interpret_with::<EBrand>(handler) -> Run<R_minus_E, S, A>`
(single-effect row-narrowing pipeline) as a new Phase 3 step.
Pipeline + extract uniquely enables three capabilities that
neither step 2's all-handlers-at-once form nor the future
MonadRec form can provide:

1. **Partial interpretation.** Pipeline keeps the program in
   Run-land while peeling effects (returns
   `Run<R_minus_E, S, A>`). Step 2 returns `A`; the future
   MonadRec form returns `M::Of<A>` (extracted). Pipeline is
   the only shape that supports "interpret one effect, store
   the result, interpret the rest later".
2. **User-controlled handler ordering for non-commuting
   effects.** Combinations like `NonDet * Except` produce
   different semantics depending on which handler runs
   "outside" which. Pipeline lets users explicitly chain
   `.interpret_with::<Except>(...).interpret_with::<NonDet>(...)`.
3. **Compositional handler libraries.** Library authors can
   ship reusable handlers as
   `fn run_state<R, A>(...) -> Run<R_minus_State, S, A>`.
   Without pipeline, they can only ship handler closures
   meant for inclusion in a user-built `handlers!{}` block.

These three capabilities are real ecosystem needs, not just
ecosystem precedent. Heftia and PureScript ship pipeline as
their primary interpretation mode for these reasons; the
load-bearing argument is the capability set, not the
convention.

**Decision 2 (axis 3 rec/non-rec, confirmed (2.C) Asymmetric).**
Step 2 stays as-shipped (M = self Run, returns A). The
MonadRec step (renumbered to step 4) adds an
externally-targeted `<MBrand: MonadRec>` family alongside.
Three orthogonal cognitive models map to three primitives:

- Simple (step 2): M-free, "give me a value", no engagement
  with MonadRec abstraction.
- Pipeline (step 3): row-narrowing for compositional handler
  chains.
- MonadRec (step 4): external target (`Thunk` / `Option` /
  `Result` etc.) with stack-safety guarantees via
  `tail_rec_m`.

The key principled argument: step 2's M-free shape uniquely
enables value extraction without engaging MonadRec
abstraction. Its `while`-loop is structurally stack-safe by
construction (no `M::bind` or `M::tail_rec_m` in the body),
so there's no need for a `MonadRec` constraint. Under any
alternative , (2.A) symmetric Monad/MonadRec, (2.B) MonadRec
uniform, or (2.D) drop-simple-form , value extraction would
route through `M = IdentityBrand` with turbofish + `.0`
unwrap, forcing users to encounter MonadRec machinery they
don't conceptually need.

The original recommendation reasoning ("preserve `d5efe2a`'s
API") was partly conventional. The principled reasoning that
landed during the discussion is the M-free unique-value
framing.

**Decision 3 (Phase 3 step ordering, confirmed (3.A) Insert + renumber).**
Inserts the new pipeline step at position 3; renumbers
former steps 3-6 to 4-7. The principled reason: atomic
commits + linear readability is a software-engineering
practice with concrete benefits (bisectability, reviewability,
navigability). Reference-sweep cost is bounded (plan.md
"Implementation phasing" + "Current progress" only; existing
deviations.md entries don't need editing because their step
numbers don't change).

**Decision 4 (Phase 6+ deferred entries, confirmed (4.A) Defer all three).**
Three new Phase 6+ deferred-items entries in plan.md:

- `interpret_with<M: Monad>` (Monad-bound externally-targeted
  family, axis 3 alternative branch).
- `run_cont` / `run_accum_cont` family (axis 1
  continuation-passing handlers).
- `interpose` family (heftia hook-without-removing).
- Algebraic-shape FO handlers (axis 2).

The Phase 6+ pattern serves a real institutional-memory
purpose: each entry records what the item is, why it's
deferred, and a trigger for revisitation. Without entries,
deferred items get lost or re-litigated by future agents.

**Decision 5 (decisions.md update, confirmed (5.A) Keep frozen).**
The doc system has separate roles for separate kinds of
content: decisions.md (design-time frozen rationale), plan.md
"Key decisions" (implementation-time choices), resolutions.md
(blocker analyses), deviations.md (per-step deviations).
Editing decisions.md to record implementation-time choices
would merge two roles inappropriately. (5.B) "refine 4.3"
and (5.C) "add 4.7" both violate role separation.

### Rust constraints that shaped the analysis

The Rust port differs from PureScript in two structural ways
that affect the interpreter design:

1. **Step 2's loop is structurally stack-safe regardless of
   M.** PureScript Run's `run` body
   `loop = resume (\a -> loop =<< k a) pure` recurses through
   `m`'s bind, building m-bind frames in the host stack. The
   `runRec` sibling swaps for `tailRecM` to keep host stack
   constant. fp-library step 2's body is
   `while { match peel { Ok(a) => return a, Err(node) => prog = handlers.dispatch(layer) }}`
   , assignment-driven, no `m`, no `bind`, no `tail_rec_m`.
   The PureScript rec/non-rec distinction does not apply to
   step 2's shape; stack-safety is by construction.
2. **Bind-driven recursion with borrowed handler state is
   awkward in Rust.** PureScript's `loop =<< k a` recurses
   inside the bind continuation, which captures `loop`. In
   Rust, the closure passed to `M::bind` must be `Fn`
   (multi-callable for non-deterministic m), capture the
   handler list, and avoid mutably aliasing it. The
   workarounds are real but cheap (`Fn` closures are
   natural; `HandlersCons<H, T>` and `Handler<E, F>` already
   derive `Clone`; `'static` is mostly already required).
   Without the workarounds, only `tail_rec_m`-driven loops
   are tractable.

These constraints meant the original decisions.md 4.3
"ship both families" reasoning ("implementation cost is
mostly mechanical") was weaker in Rust than the
decision-author anticipated. (2.C) accepts this and ships
the rec form alongside step 2's M-free form rather than
mirroring PureScript's bind-driven `run` shape.

### Heftia row architecture clarification

Decisions.md section 4.5 commits to "heftia's dual-row
architecture" for scoped effects. Reading
[heftia source](https://github.com/sayo-hs/heftia) directly:
heftia uses **one** effect list (`es`) where each element
has a `KnownOrder` (FO or HO), with a `FOEs es` constraint
when all members must be first-order. fp-library takes the
**idea** (separate FO vs HO dispatch) and ships a
value-level dual row (`Run<R, S, A>` with
`Node<R, S> = First | Scoped`). So fp-library's
"heftia-inspired" framing is inspiration, not direct port;
the row encoding diverges. This gives latitude on the
interpreter API: we don't need heftia's mixed-order
constraint machinery (`FOEs`, `KnownOrder`); we just
dispatch on `Node::First` vs `Node::Scoped`.

### What ships under the resolution

Phase 3 re-scheduled per (3.A):

1. Step 1 (`82dd7bb`): `handlers!{...}` macro + `nt()` builder.
2. Step 2 (`d5efe2a`): `interpret` / `run` / `run_accum`
   M-free family.
3. Step 3 (NEW, pending): pipeline `interpret_with::<E>` +
   `extract` per wrapper. New `DispatchOneHandler` trait
   variant.
4. Step 4 (was step 3, pending): MonadRec-target
   `interpret_rec` / `run_rec` / `run_accum_rec`.
5. Step 5 (was step 4): standard first-order effects.
6. Step 6 (was step 5): `define_effect!` macro.
7. Step 7 (was step 6): `compile_fail` UI tests.

Phase 6+ deferred-items section gains four new entries
(`interpret_with<M: Monad>`, `run_cont` / `run_accum_cont`,
`interpose` family, algebraic-shape FO handlers) with full
trigger conditions.

decisions.md stays frozen per (5.A); per-step deviations.md
entries land when each Phase 3 step ships, cross-referencing
this resolutions.md entry.

### Cross-references

- [decisions.md](decisions.md) section 4.3 ("Ship both
  interpreter families"): the original commitment that
  shaped the question.
- [decisions.md](decisions.md) section 4.5: heftia-inspired
  dual-row scoped effects (where the heftia inspiration
  framing originated).
- [`fp-library/src/types/free.rs`](../../../fp-library/src/types/free.rs)'s
  `Free::fold_free<G: MonadRec>`: existing Rust precedent
  for the externally-targeted MonadRec pattern.
- [`fp-library/src/classes/natural_transformation.rs`](../../../fp-library/src/classes/natural_transformation.rs):
  rank-2 polymorphic abstraction; complementary to the
  handler-list path. Future `interpret_nt` deferred entry
  references this.
- [`fp-library/src/classes/monad_rec.rs`](../../../fp-library/src/classes/monad_rec.rs):
  fp-library's MonadRec, mirror of PureScript's.
- Phase 3 step 1 commit `82dd7bb` (handlers! macro).
- Phase 3 step 2 commit `d5efe2a` (interpret family) , the
  API `(2.C)` preserves and `(2.A)` / `(2.B)` / `(2.D)` would
  have broken.
- [PureScript Run](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
  source.
- [PureScript MonadRec](https://github.com/purescript/purescript-tailrec/blob/master/src/Control/Monad/Rec/Class.purs)
  source.
- [Heftia](https://github.com/sayo-hs/heftia) interpreter
  machinery.

## Resolved (2026-04-28): Phase 2 step 9 scope is under-specified

Phase 2 step 9's plan-text originally read in full:

> 9. Coyoneda-wrapping smart constructors (`lift_f` analogues for each effect type).

Two plausible interpretations of "smart constructors for each
effect type" existed, and they differed substantially in scope:
the **generic combinator** interpretation (one helper that
takes any effect value plus a `Member` witness, lifts it
through Coyoneda, injects into the row, wraps in `Node::First`,
and `send`s) and the **per-effect helpers** interpretation
(concrete `State<S>` / `Reader<E>` / `Except<E>` / `Writer<W>` /
`Choose` types plus `ask`, `get`, `put`, `modify`, `tell`,
`throw`).

Reading the rest of the plan, the generic-combinator interpretation
was the intended one: Phase 3 step 4 explicitly schedules
_"Standard first-order effect types and their smart
constructors: `State<S>`, `Reader<E>`, `Except<E>`, `Writer<W>`,
`Choose`"_ as a separate Phase 3 deliverable, so doing per-effect
work in Phase 2 step 9 would duplicate it.
[decisions.md](decisions.md) section 6 likewise describes
per-effect smart constructors as **thin wrappers over** the
`inj + liftF`/`send` infrastructure, implying the row-aware
lift combinator is prerequisite infrastructure that ships first
(which is what step 9 lands).

Three sub-questions remained open under the generic-combinator
interpretation:

- **Free function vs per-wrapper inherent method.** The
  established Phase 2 pattern (steps 5, 7a-c) puts user-facing
  Run-program operations on the wrappers as inherent methods
  (`Run::pure`, `RcRun::bind`, etc.), but the combinator's key
  argument is the effect value, not `self`, so a free function
  would also be natural.
- **Exact signature.** The `Member` bounds, the `Coyoneda`
  decode closure (does the user supply it?), the alignment across
  the six wrappers (whose bounds differ: Erased Rc family wants
  `A: 'static`, Explicit family wants `A: 'a`, ArcRunExplicit
  wants `A: 'a + Send + Sync`), and whether `Idx` is turbofished
  or inferred.
- **HRTB-poisoning under `ArcFree`.** Per the prior 2026-04-27
  resolution, constructing a `Node`-projection literal inside
  an HRTB-bearing scope (which `ArcFree`'s struct propagates
  into every `ArcRun`-method context) fails GAT normalization.
  `ArcRun`'s row-aware lift combinator would need to thread
  the same workaround.

- **Naming: `lift` vs `lift_f`.** The combinator does the full
  chain (`Coyoneda::lift` + Member inject + `Node::First` +
  `*Run::send`); functionally it is the direct analog of
  PureScript Run's
  [`lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
  (signature `Proxy sym -> f a -> Run r a`, body
  `Run <<< liftF <<< inj p`), not of
  [`Free.liftF`](https://github.com/purescript/purescript-free/blob/main/src/Control/Monad/Free.purs).
  PureScript explicitly distinguishes the two: `Free.liftF`
  is the Free-only operation; `Run.lift` is the row-aware
  Run-level operation that consumes a row label and runs the
  full inject + liftF chain. fp-library already mirrors the
  Free side as
  [`Free::lift_f`](../../../fp-library/src/types/free.rs)
  (snake_case translation of `liftF`); the Run-level operation
  takes the bare name `lift`. Phase 3's per-effect smart
  constructors (`ask = lift ReaderBrand Reader::Ask`, etc.)
  read consistently with PureScript's
  `liftEffect = lift (Proxy :: "effect")` pattern under this
  naming.

### Resolution

**Generic combinator interpretation, named `lift` (matching
PureScript Run's
[`lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)).
Inherent associated function on each of the six Run wrappers,
mirroring `*Run::send`'s shape. Take the raw effect (an
`EBrand::Of<'a, A>` value) and do the full chain (`Coyoneda::lift`
-> row inject -> `Node::First` -> `*Run::send`) inside the body.
Type-infer `Idx` at call sites where the row is unambiguous;
turbofish only when duplicate effect types make `Idx` ambiguous.
Try the simple inline body first; fall back to a free
`lift_node<R, S, EBrand, Idx, A>(effect)` helper for `ArcRun::lift`
if HRTB-poisoning recurs.**

The signature for `Run` is:

```rust
impl<R: Kind, S: Kind, A: 'static> Run<R, S, A> {
    pub fn lift<EBrand, Idx>(
        effect: Apply!(<EBrand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>),
    ) -> Self
    where
        Apply!(<R as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>):
            Member<Coyoneda<'static, EBrand, A>, Idx>,
        EBrand: Kind_cdc7cd43dac7585f + 'static,
    {
        let coyo: Coyoneda<'static, EBrand, A> = Coyoneda::lift(effect);
        let layer = <Apply!(<R as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>)
            as Member<Coyoneda<'static, EBrand, A>, Idx>>::inject(coyo);
        Self::send(Node::First(layer))
    }
}
```

The `Kind!()` macro can't appear in trait-bound position (per
its doc-comment limitation: invalid in supertrait bounds, type
aliases, and trait aliases on stable Rust); the bound on
`EBrand` uses the generated hash name `Kind_cdc7cd43dac7585f`
directly. The hash is deterministic from the signature
`type Of<'a, T: 'a>: 'a;` and is in scope via fp-library's
existing `kinds::*` re-export.

Per-wrapper deltas (the body shape is identical; only the bounds
change):

| Wrapper          | `'a`         | Extra `A` bound            | Extra row/node bound                                          |
| :--------------- | :----------- | :------------------------- | :------------------------------------------------------------ |
| `Run`            | `'static`    | `A: 'static`               | (none)                                                        |
| `RcRun`          | `'static`    | `A: 'static`               | the `Apply<...>: Clone` bound `RcRun::send` carries           |
| `ArcRun`         | `'static`    | `A: Send + Sync + 'static` | `NodeBrand<R, S>: Functor` plus the `Apply<...>: Clone` bound |
| `RunExplicit`    | `'a` (param) | `A: 'a`                    | (none)                                                        |
| `RcRunExplicit`  | `'a` (param) | `A: 'a`                    | (none)                                                        |
| `ArcRunExplicit` | `'a` (param) | `A: 'a + Send + Sync`      | (none)                                                        |

The Coyoneda decode closure is implicit: `Coyoneda::lift` defaults
to the trivial decode, which is what every smart-constructor case
wants. Users who want a non-trivial decode construct `Coyoneda`
themselves and use `*Run::send` directly.

### Why not a single polymorphic free function

The six wrappers use six different inner constructors with six
different bound shapes (`'static` vs `'a`, `Clone` on the Apply
node-projection, `Send + Sync` for Arc, etc.) and there is no
common trait abstracting "construct from a node-projection".
Inventing one to make `lift` polymorphic would be more code
than just writing six near-identical inherent methods, which is
the same trade-off `*Run::send` already settled on its 2026-04-27
resolution.

### Why raw effect input (not pre-lifted Coyoneda)

Matches PureScript Run's
[`lift :: Row.Cons sym f r1 r2 => Proxy sym -> f a -> Run r2 a`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs),
which takes the raw effect (`f a`) and does the inject + liftF
chain internally. Phase 3's smart constructors then become
one-liners
(`pub fn ask<R, S, Idx>() -> Run<R, S, Env> { Run::lift::<ReaderBrand, _>(Reader::Ask) }`),
mirroring PureScript's
`liftEffect = lift (Proxy :: "effect")` pattern. The Coyoneda
detail stays an implementation concern of the helper rather than a
user-visible step. Users who want to construct a non-trivial
Coyoneda decode bypass `lift` and call `Coyoneda::new` plus
`*Run::send` directly.

### Why "try inline; fall back if needed" for the HRTB workaround

The 2026-04-27 GAT-normalization issue specifically hit
`Apply!(<NodeBrand<R, S> as Kind>::Of<'static, A>)` _normalization_
inside `ArcFree`'s HRTB-bearing scope. The `lift` body builds the
Node-projection by _literal construction_ (`Node::First(Member::inject(coyo))`);
no `Apply!` normalization is required on the result type, only on
the `effect` parameter (which is fine, since it's already
pre-resolved at the function boundary). Plausibly clean for
`ArcRun::lift`. If it does fail, the workaround is mechanical:
factor `Node::First(<_ as Member<_, Idx>>::inject(Coyoneda::lift(effect)))`
into a free helper outside the HRTB scope and have
`ArcRun::lift` call `Self::send(lift_node::<R, S, EBrand, Idx, A>(effect))`.
Pre-baking the `lift_node` helper in all six wrappers
prophylactically would be wasted code if the simple form works
for everything but `ArcRun`.

## Resolved (2026-04-28 implementation expansion): step 9 SendFunctor cascade prerequisites for Arc family

While implementing the original 2026-04-28 resolution above,
[`Run::lift`](../../../fp-library/src/types/effects/run.rs)
landed cleanly at commit `34b6a97`. Extending the same body to
`RunExplicit`, `RcRun`, `RcRunExplicit` worked. But `ArcRun::lift`
and `ArcRunExplicit::lift` hit a structural conflict the original
resolution didn't anticipate.

### Problem

`ArcRun`'s struct-level HRTB
(`Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync`)
forces every variant of the row's projection to be `Send + Sync`.
The bare
[`Coyoneda`](../../../fp-library/src/types/coyoneda.rs) stores
its accumulated continuation in `Box<dyn FnOnce>` (no
`Send + Sync`), so `Coyoneda<'_, EBrand, A>` is not
`Send + Sync` and `ArcRun` rejects `CoyonedaBrand`-headed rows.

The Send-aware companion
[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
exists and is `Send + Sync`. But
[`ArcCoyonedaBrand`](../../../fp-library/src/brands.rs)
deliberately doesn't implement
[`Functor`](../../../fp-library/src/classes/functor.rs) (it
only implements [`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
and [`Foldable`](../../../fp-library/src/classes/foldable.rs)),
because the
[`Functor::map`](../../../fp-library/src/classes/functor.rs)
trait method's signature lacks `Send + Sync` bounds on its
closure parameter; closures stored in Arc-wrapped layers must
be `Send + Sync`. This is a deliberate fp-library design
choice: the Send-aware parallel trait family
([`SendFunctor`](../../../fp-library/src/classes/send_functor.rs),
[`SendPointed`](../../../fp-library/src/classes/send_pointed.rs),
[`SendSemimonad`](../../../fp-library/src/classes/send_semimonad.rs),
[`SendApplicative`](../../../fp-library/src/classes/send_applicative.rs),
etc., plus the
[`SendRef`](../../../fp-library/src/classes/send_ref_functor.rs)
prefix tree) exists to handle Arc-substrate brands; plain
`Functor` deliberately does not impose Send bounds for the
common-case non-thread-crossing brands.

`ArcRun`'s existing `peel` / `send` / `bind` / `map`
implementations route through `<NodeBrand<R, S> as Functor>::map`
on the row brand. `NodeBrand: Functor` cascades to `R: Functor`,
which `ArcCoyonedaBrand` cannot satisfy. So the universal
`Run.lift` shape (Coyoneda lift -> row inject -> `Node::First` ->
`*Run::send`) cannot work for the Arc family without a Send-aware
substrate path.

The substrate
[`ArcFree`](../../../fp-library/src/types/arc_free.rs) compounds
the issue: its internal machinery (`lift_f`, `wrap`, `bind`,
`evaluate`, `fold_free`, `hoist_free`) all bound `F: Functor` and
call `F::map` directly. Switching the Run wrappers to the
Send-aware tree requires switching the substrate too.

### Resolution

**Expand step 9 with a `SendFunctor` cascade as prerequisite
sub-steps before the universal `lift` work. Replace
`F: Functor` bounds with `F: SendFunctor` on the Arc-substrate
machinery (`ArcFree`, `ArcFreeExplicit`); land the missing
`SendFunctor` impls on the row-cascade brands; expand the
brand-level type-class surface on `ArcFreeExplicitBrand` and
`ArcRunExplicitBrand` to absorb newly-reachable Send-aware
impls; then complete `*Run::lift` for all six wrappers under
the now-supported cascade.** Implement `SendRefFunctor` on
`ArcRunExplicitBrand` via inherent-method delegation (calling
the wrapper's `ref_map` / `ref_bind` / `ref_pure` directly,
bypassing the brand-level cascade via the clone-trick).

The expanded sub-step structure lives in plan.md step 9 (9a
through 9i); each lands as a separate commit. The `Run::lift`
implementation already shipped at commit `34b6a97` stays as the
reference design; sub-step 9h fills in the remaining five
wrappers.

### Why "replace Functor with SendFunctor" instead of adding sibling methods

Two paths considered:

- **Replace `F: Functor` with `F: SendFunctor`** on `ArcFree` /
  `ArcFreeExplicit`'s methods (signatures change, internal
  `F::map` calls become `F::send_map`). Breaking change for any
  pre-existing caller passing a non-Send `Functor`-only row
  brand. Cleaner long-term: one method per operation; semantic
  alignment between the substrate's thread-safety bounds and
  the trait surface.
- **Add Send-aware sibling methods** (`ArcFree::send_lift_f`
  alongside `ArcFree::lift_f`). Backwards-compatible but doubles
  the API surface; users have to pick the right method. The cost
  compounds across `ArcFreeExplicit`'s siblings.

Replacement chosen because `ArcFree`'s struct-level Send+Sync
HRTB already restricts concrete callers to row brands that
satisfy `Send + Sync`; adding `SendFunctor` impls to the row-
cascade brands (sub-step 9a) keeps existing concrete callers
working without method-surface duplication.

### Why `SendRefFunctor` via inherent-method delegation

The 2026-04-27
"[brand-level type-class coverage gap on the Explicit Run brands](#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands)"
resolution documented `SendRef`-family hierarchy as unreachable
through brand-level delegation: `ArcFreeExplicitBrand` can't
implement `SendRefFunctor` because the auto-derive of
`Send + Sync` on the closure return type requires a per-`A`
HRTB on the `Kind` projection that stable Rust's trait method
signatures cannot carry.

The unreachability is at the substrate-brand level. The
`ArcRunExplicit` _wrapper_ has inherent
[`ref_map`](../../../fp-library/src/types/effects/arc_run_explicit.rs)
/ `ref_bind` / `ref_pure` methods that work via the clone-trick
(`self.clone().send_map(move |a| f(&a))`); the `O(1)`
`Arc::clone` makes this cheap, and the per-`A` HRTB doesn't
appear at the wrapper-method signature because the closure
constraints are checked against the inherent method's bound
list rather than against the brand-level trait method's. So
`ArcRunExplicitBrand: SendRefFunctor` is reachable if the impl
delegates to the wrapper's inherent `ref_map`, sidestepping
`ArcFreeExplicitBrand` entirely.

This is a different delegation strategy than what step 4b's
resolution considered (substrate-brand delegation). The
inherent-method delegation pattern produces a working
brand-level `SendRefFunctor` impl with the same observable
behavior at the cost of an `O(1)` clone per call. The clone is
acceptable: brand-level dispatch is the path the user opted into
when they wrote `<ArcRunExplicitBrand as SendRefFunctor>::send_ref_map`,
and the alternative is no brand-level coverage at all.

### Why not defer the SendFunctor cascade to a later phase

Three plausible structures considered:

- **Defer to Phase 1.5 follow-up.** The SendFunctor cascade on
  the row-brand types is genuinely substrate-level
  infrastructure, and Phase 1's WrapDrop migration set a
  precedent for landing prerequisite trait-cascade work as a
  follow-up between phases. But Phase 1 has long completed; a
  retroactive "Phase 1.5" is structurally awkward and signals
  bigger drift than the work warrants.
- **Defer to Phase 3.** Phase 3 step 4 lands per-effect smart
  constructors that build on `*Run::lift`. Deferring the
  cascade would push `ArcRun::ask` / `ArcRun::get` etc. behind
  a structural prerequisite, breaking Phase 3's promise of
  thin one-liners over `*Run::lift`.
- **Expand step 9's scope.** Most coherent: the cascade is
  required by step 9's universal-`lift` promise; landing it as
  step 9 sub-steps keeps the prerequisite-and-payoff together,
  visible in one place, and verifiable as a unit. The smaller
  sub-step granularity ensures each is independently
  reviewable.

The third option chosen.

### Reference: scope inventory at start of expansion

Confirmed by code inspection at the time the blocker surfaced:

- `ArcCoyonedaBrand`: has [`SendFunctor`](../../../fp-library/src/types/arc_coyoneda.rs);
  needs [`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs).
- `IdentityBrand`: has `Functor` and `WrapDrop`; needs
  `SendFunctor` (mechanical; `Identity<A>` has no closures, so
  the closure `Send + Sync` requirement is vacuous).
- `NodeBrand`, `CoproductBrand<H, T>`, `CNilBrand`: have
  `Functor` and `WrapDrop`; need `SendFunctor` (recursive
  cascade for the inductive cases; uninhabited base case for
  `CNilBrand`).
- `ArcFree`: bounds `lift_f` / `wrap` / `bind` / `evaluate` /
  `fold_free` / `hoist_free` on `F: Functor`; calls `F::map` at
  three sites; switch to `F: SendFunctor` and `F::send_map`.
- `ArcFreeExplicit`: same shape as `ArcFree`; same migration.
- `ArcRun`: methods route through `<NodeBrand<R, S> as Functor>::map`;
  switch to `<NodeBrand<R, S> as SendFunctor>::send_map` after
  the cascade lands.
- `ArcRunExplicit`: same as `ArcRun`.
- `ArcFreeExplicitBrand`: brand-level coverage limited to
  `SendPointed` per step 4b; expand to `SendFunctor` and
  cascade dependents under the Send-aware machinery.
- `ArcRunExplicitBrand`: same expansion path; plus the
  `SendRefFunctor`-via-inherent-method-delegation impl.

## Resolved (2026-04-27): `*Run::send` takes a `Node`-projection value to sidestep GAT-normalization poisoning under `ArcFree`'s HRTB

Step 5's `send` method on each of the six Run wrappers takes the
[`NodeBrand<R, S>`](../../../fp-library/src/brands.rs)
`Of<'_, A>` projection (already-constructed) rather than the
first-order row variant (constructed internally via
`Node::First(layer)`). This deviates from the natural shape that
mirrors PureScript Run's `send`, but is required because of a
stable-Rust GAT-normalization limit that surfaces in `ArcRun`'s
impl-block context.

### Problem

While implementing `ArcRun::send` with the natural shape (take
the row variant `R::Of<'static, A>`, construct
`Node::First(layer)` internally, pass to `ArcFree::lift_f`),
the compiler refused to unify `Node<'static, R, S, A>` (the
literal value) with
`<NodeBrand<R, S> as Kind_cdc7cd43dac7585f>::Of<'static, A>`
(the projection that `ArcFree::lift_f` expects), even though
`impl_kind!` declares them equal:

```
expected associated type `<NodeBrand<R, S> as kinds::Kind_cdc7cd43dac7585f>::Of<'static, A>`
                  found enum `node::inner::Node<'static, R, S, A>`
```

The same construction succeeds for `Run::send` (over
[`Free`](../../../fp-library/src/types/free.rs)) and
`RcRun::send` (over
[`RcFree`](../../../fp-library/src/types/rc_free.rs)). The
difference is that
[`ArcFree`](../../../fp-library/src/types/arc_free.rs)'s struct
carries a per-`A`-instantiation HRTB
`F: Kind<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>`
(needed so the compiler can auto-derive `Send + Sync` on
`ArcFree<F, A>` when `F`'s `Of` projection is `Send + Sync`).
This HRTB propagates to `ArcRun`'s impl block, and inside that
block stable Rust's normalizer refuses to fire for any other
instantiation of the same `Of` projection.

### Investigation

Eleven experiments at
[`fp-library/tests/arc_run_normalization_probe.rs`](../../../fp-library/tests/arc_run_normalization_probe.rs)
(see history; trimmed in the final commit to the four passing
patterns) isolated the trigger:

- The HRTB itself, not the `ArcFree` field, is the trigger
  (PhantomData-only struct + HRTB still fails).
- The trigger is not impl-block-specific: a free function
  carrying the HRTB also fails.
- The trigger poisons cross-substrate calls: a `RcFree::lift_f`
  call from inside an `ArcFree`-HRTB-bearing impl also fails.
- Workarounds tried that all fail: explicit `Apply!()`-typed
  local; turbofish `Node::<'static, R, S, A>::First(layer)`;
  using `<...as Kind_cdc7cd43dac7585f>::Of` directly bypassing
  `Apply!`; routing through `Functor::map(identity, Node::First(layer))`
  (whose input is also at the projection); restructuring the
  impl block to use direct `R: ... + 'static, S: ... + 'static`
  bounds plus the HRTB.
- The workaround that succeeds: pass an already-projection-typed
  value into the HRTB-scope function, never construct a Node
  literal there. The caller (typically code without HRTB in
  scope, e.g., test code, smart-constructor macro output) builds
  `Node::First(layer)` and passes the result.

The probe file at
[`fp-library/tests/arc_run_normalization_probe.rs`](../../../fp-library/tests/arc_run_normalization_probe.rs)
is the trimmed regression-test version that documents the four
patterns confirmed to work despite the limit.

### Resolution

`*Run::send` on all six wrappers takes the
`Node`-projection value as a parameter, uniform signature:

```rust
pub fn send(
    node: Apply!(<NodeBrand<R, S> as Kind!(...)>::Of<'_, A>),
) -> Self;
```

Smart constructors (Phase 2 step 9) will emit
`Node::First(<R as Member<...>>::inject(coyo))` in their bodies
and pass the result to `send`. User test code does the same.

### Why not work around at a different layer

- **Re-architect `ArcFree` to remove the struct-level HRTB**:
  out of scope for step 5 (would require a Phase 1 follow-up
  commit). The HRTB is load-bearing for `Send + Sync` auto-derive
  on `ArcFree`, which dozens of other code paths depend on.
- **Provide `unsafe impl Send` / `unsafe impl Sync` for
  `ArcRun`** with bounds that don't include the HRTB: the unsafe
  impl's `where` clause would still need to express the
  Send/Sync condition somehow, and any expression of "the
  projection at this instantiation is Send + Sync" is itself an
  HRTB-shaped constraint that re-triggers the issue.
- **Accept the asymmetry between `Run`/`RcRun` (take row
  variant) and `ArcRun` (take Node projection)**: the symmetric
  approach was chosen for design consistency (the two patterns
  diverging across the six wrappers would surface as confusion
  in users of step 7's macros and step 9's smart constructors).

## Resolved (2026-04-27): brand-level type-class coverage gap on the Explicit Run brands

The plan's Phase 2 step 4 specification named a full
`Functor / Pointed / Semimonad / Monad` hierarchy plus a
`RefFunctor / RefPointed / RefSemimonad / RefMonad` hierarchy
for `RunExplicitBrand`, with analogous coverage for
`RcRunExplicitBrand` and `ArcRunExplicitBrand`. Step 4b
landed the achievable subset: `Functor / Pointed / Semimonad`
plus the by-reference equivalents for `RunExplicitBrand`,
`Pointed` plus by-reference equivalents for
`RcRunExplicitBrand`, and `SendPointed` only for
`ArcRunExplicitBrand`.
[`Monad`](../../../fp-library/src/classes/monad.rs) /
[`RefMonad`](../../../fp-library/src/classes/ref_monad.rs) /
[`SendMonad`](../../../fp-library/src/classes/send_monad.rs) and
the [`SendRef`](../../../fp-library/src/classes/send_ref_functor.rs)-family
hierarchy are not reachable through brand-level delegation;
inherent `bind` and `map` methods on `RcRunExplicit` and
`ArcRunExplicit` (mirroring
[`RcFreeExplicit`](../../../fp-library/src/types/rc_free_explicit.rs)'s
inherent surface) cover the by-value monadic surface for
concrete-type call sites.

### Problem

Three independent gaps share the same root cause: stable Rust's
trait method signatures cannot carry per-`A` bounds (no HRTB
over types), and the `*FreeExplicitBrand`s the Run-Explicit
brands delegate to deliberately do not implement the missing
classes for the same reason.

1. **`Monad` blanket impl requires `Applicative`.** The
   project's [`Monad`](../../../fp-library/src/classes/monad.rs)
   trait at line 214 is
   `pub trait Monad: Applicative + Semimonad {}` with a blanket
   `impl<Brand> Monad for Brand where Brand: Applicative + Semimonad {}`
   at line 218. Same shape for
   [`RefMonad`](../../../fp-library/src/classes/ref_monad.rs)
   over `RefApplicative + RefSemimonad`. So a brand cannot be
   `Monad` without first being `Applicative`.
   [`FreeExplicitBrand`](../../../fp-library/src/brands.rs)
   deliberately does not implement
   [`Applicative`](../../../fp-library/src/classes/applicative.rs)
   (its [`Lift`](../../../fp-library/src/classes/lift.rs)
   supertrait's natural definition pattern
   `lift2 = bind(fa, |a| map(fb, |b| f(a, b)))` requires `fb` to
   be reusable across closure invocations, and
   [`FreeExplicit`](../../../fp-library/src/types/free_explicit.rs)
   is not `Clone` per [`free_explicit.rs`](../../../fp-library/src/types/free_explicit.rs)
   lines 369-388). The Run wrapper brands inherit this gap
   through delegation.
2. **`SendRef` hierarchy unreachable on `ArcRunExplicitBrand`.**
   The [`ArcFreeExplicit`](../../../fp-library/src/types/arc_free_explicit.rs)
   substrate auto-derives `Send + Sync` only when its struct
   carries a per-`A` `Kind` HRTB
   (`Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync`).
   That bound's `'a` and `A` are the trait method's per-method
   generics; stable Rust does not support `for<'a, T>` HRTB at
   the impl-block level. So
   [`ArcFreeExplicitBrand`](../../../fp-library/src/brands.rs)
   does not implement
   [`SendRefFunctor`](../../../fp-library/src/classes/send_ref_functor.rs)
   /
   [`SendRefPointed`](../../../fp-library/src/classes/send_ref_pointed.rs)
   /
   [`SendRefSemimonad`](../../../fp-library/src/classes/send_ref_semimonad.rs)
   (see [`arc_free_explicit.rs`](../../../fp-library/src/types/arc_free_explicit.rs)
   lines 730-745). `ArcRunExplicitBrand`'s would-be Send-Ref
   delegation has no target.
3. **Ref hierarchy is bounded by `R: RefFunctor`.** The Ref
   impls on `RunExplicitBrand` and `RcRunExplicitBrand` delegate
   to the corresponding `*FreeExplicitBrand`'s Ref impls, which
   carry `F: WrapDrop + Functor + RefFunctor + 'static`.
   For `Run`, `F = NodeBrand<R, S>`; the cascade requires
   `R: RefFunctor` and `S: RefFunctor`. Step 4b adds
   [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
   impls on `CNilBrand`, `CoproductBrand<H, T>`, and
   `NodeBrand<R, S>`, but
   [`CoyonedaBrand`](../../../fp-library/src/brands.rs) does not
   implement
   [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs).
   Canonical Run rows (`CoproductBrand<CoyonedaBrand<E_i>, ...>`)
   do not satisfy the cascade. The Ref impls are present at the
   brand level but reachable only for synthetic rows whose
   brands carry their own `RefFunctor` impls (e.g.,
   `CoproductBrand<IdentityBrand, CNilBrand>`).

### Resolution

Ship the achievable subset; document gaps as deviations. Future
work that needs the missing coverage either reaches for the
inherent methods on the concrete Run wrapper types or, for
`Coyoneda`-wrapped effect rows, adds `RefFunctor` to
[`CoyonedaBrand`](../../../fp-library/src/types/coyoneda.rs)
(scope-creep beyond step 4b; tracked separately).

### Why not work around

- **Restructuring `Monad`'s supertrait chain:** would require
  editing [`monad.rs`](../../../fp-library/src/classes/monad.rs)
  and similar; out of scope for the effects port and would break
  every existing brand impl.
- **Adding `Applicative` impls with `Clone` bounds at the trait
  signature level:** stable Rust's
  [`Applicative::lift2`](../../../fp-library/src/classes/lift.rs)
  signature can't be augmented; per-method `where` clauses on
  trait impls are restricted to what the trait allows.
- **Adding the SendRef hierarchy directly on
  `ArcRunExplicitBrand`** (bypassing
  `ArcFreeExplicitBrand`): would have the same per-`A` HRTB
  obstacle the underlying brand has.

## Resolved (2026-04-27): row-brand `RefFunctor` and `Extract` cascade impls land in step 4b

Phase 2 step 4a left
[`CNilBrand`](../../../fp-library/src/types/effects/variant_f.rs),
[`CoproductBrand<H, T>`](../../../fp-library/src/types/effects/variant_f.rs),
and
[`NodeBrand<R, S>`](../../../fp-library/src/types/effects/node.rs)
with [`Functor`](../../../fp-library/src/classes/functor.rs)
and [`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs)
impls only. Step 4b added
[`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
and [`Extract`](../../../fp-library/src/classes/extract.rs)
cascade impls on each of the three brands, plus a
[`Clone`] impl for the
[`Node`](../../../fp-library/src/types/effects/node.rs) enum.

### Problem

Three trait gaps surfaced as step 4b's Explicit family was
landed:

1. **`RefFunctor` needed for Ref-hierarchy delegation.**
   `RunExplicitBrand`'s
   [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
   impl delegates to
   [`FreeExplicitBrand`](../../../fp-library/src/brands.rs)'s,
   which carries `F: WrapDrop + Functor + RefFunctor + 'static`.
   For `Run`, `F = NodeBrand<R, S>`; the cascade requires
   `R: RefFunctor` and `S: RefFunctor`, so the row brand chain
   must support it.
2. **`Extract` needed for `evaluate()` on canonical Run
   programs.**
   [`FreeExplicit::evaluate`](../../../fp-library/src/types/free_explicit.rs)
   requires `F: Extract`. For `Run`, `F = NodeBrand<R, S>`; the
   cascade requires `R: Extract` and `S: Extract`.
   [`IdentityBrand`](../../../fp-library/src/types/identity.rs)
   has [`Extract`](../../../fp-library/src/classes/extract.rs);
   the row chain (Coproduct / CNil / Node) did not.
   Without it, brand-level test programs and doctests over
   synthetic rows could not assert evaluation results.
3. **`Clone` needed by Rc/Arc Free's evaluate fallback.**
   [`RcFreeExplicit::evaluate`](../../../fp-library/src/types/rc_free_explicit.rs)
   and
   [`ArcFreeExplicit::evaluate`](../../../fp-library/src/types/arc_free_explicit.rs)
   carry the per-`A` bound
   `Apply!(<F as Kind!(...)>::Of<'a, *FreeExplicit<'a, F, A>>): Clone`.
   For `F = NodeBrand<R, S>`, this expands to
   `Node<'a, R, S, *FreeExplicit<'a, NodeBrand<R, S>, A>>: Clone`.
   `Node` did not implement
   [`Clone`].

### Resolution

Land mechanical cascade impls on the row brands following the
same shape as the existing
[`Functor`](../../../fp-library/src/classes/functor.rs) /
[`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs)
impls:

- [`CNilBrand`](../../../fp-library/src/types/effects/variant_f.rs):
  uninhabited base case for both
  [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
  and
  [`Extract`](../../../fp-library/src/classes/extract.rs).
- [`CoproductBrand<H, T>`](../../../fp-library/src/types/effects/variant_f.rs):
  dispatches by `Inl` / `Inr` recursing into the active brand;
  bounded `H: RefFunctor + 'static, T: RefFunctor + 'static`
  for [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs);
  same shape with [`Extract`](../../../fp-library/src/classes/extract.rs)
  for the Extract impl.
- [`NodeBrand<R, S>`](../../../fp-library/src/types/effects/node.rs):
  dispatches by `First` / `Scoped`; bounded
  `R: RefFunctor + 'static, S: RefFunctor + 'static` for
  [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs);
  same shape for [`Extract`](../../../fp-library/src/classes/extract.rs).
- [`Node<'a, R, S, A>`](../../../fp-library/src/types/effects/node.rs):
  manual [`Clone`] impl bounded on `Apply!(<R as Kind!(...)>::Of<'a, A>): Clone`
  and the `S` projection; clones the active variant's payload.

`SendRefFunctor` cascade is _not_ added because
[`ArcRunExplicitBrand`](../../../fp-library/src/brands.rs)
cannot have a SendRef hierarchy in the first place (see the
adjacent resolution about brand-level coverage gaps).

## Resolved (2026-04-27): re-export pattern for the effects subsystem types follows the optics A+B hybrid

Step 4b adopts the
[`optics`](../../../fp-library/src/types/optics.rs) precedent:
selective top-level re-exports of headline types in
[`crate::types::*`](../../../fp-library/src/types.rs), plus
comprehensive subsystem-scoped re-exports at
[`crate::types::effects::*`](../../../fp-library/src/types/effects.rs).

### Problem

Phase 2 step 4 left re-exports undecided. Three options were
considered:

- **A. Top-level only** (`crate::types::*`): matches the rest
  of the [`types/`](../../../fp-library/src/types/) directory;
  ergonomic; but ~12 names land in the top-level block and the
  effects subsystem stops being visually distinguished.
- **B. Subsystem-scoped only** (`crate::types::effects::*`):
  preserves the top-level namespace shape; matches what
  [`optics`](../../../fp-library/src/types/optics.rs) does for
  non-headline types; but deviates from the Free family's
  surface.
- **C. No re-exports**: zero maintenance, but friction at
  every import site and matches no existing pattern.

The existing
[`optics`](../../../fp-library/src/types/optics.rs) precedent
is neither pure A nor pure B: it re-exports every submodule
symbol via
`pub use submodule::*` at
[`crate::types::optics::*`](../../../fp-library/src/types/optics.rs)
(comprehensive, B), AND surfaces only the three headline types
[`Composed`](../../../fp-library/src/types/optics.rs),
[`Lens`](../../../fp-library/src/types/optics.rs),
[`LensPrime`](../../../fp-library/src/types/optics.rs) at the
top-level (selective, A).

### Resolution

Adopt the optics precedent literally: the six Run wrapper
headline types
(`Run`, `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`,
`ArcRunExplicit`) are headline-class and ship at the top level
([`crate::types::*`](../../../fp-library/src/types.rs)) because
they are the user-facing types most callers will import; the
brands and row machinery (`Node`, `VariantF`,
`*RunExplicitBrand`) are subsystem-scoped and ship at
[`crate::types::effects::*`](../../../fp-library/src/types/effects.rs)
only. Brand types stay in
[`crate::brands::*`](../../../fp-library/src/brands.rs) per the
existing precedent for all brand types in the library.

## Resolved (2026-04-27): introduce `WrapDrop` trait for Free's struct-level Drop concern

A new trait `WrapDrop` lands at the struct level of the Free
family, replacing `Extract` for `Drop`'s iterative-dismantling
purposes while preserving `Extract` as a separate trait for
`evaluate` / `fold_free` / etc. Migration ships as two Phase 1
follow-up commits before Phase 2 step 4 resumes; the actual
step-by-step migration spec lives in
[plan.md](plan.md)'s "Phase 1 follow-up: WrapDrop migration"
section.

### Problem

Phase 2 step 4 (the six concrete `Run` types) commits to
`Run<R, S, A> = Free<NodeBrand<R, S>, A>` per
[decisions.md](decisions.md) section 5.2 and [plan.md](plan.md)'s
"Will change" table entry for
[`fp-library/src/types/effects.rs`](../../../fp-library/src/types/effects.rs).
This requires `Free<NodeBrand<R, S>, A>` to compile for typical
effect rows. It does not, because of a transitively-poisoning
trait bound:

1. [`Free<F, A>`](../../../fp-library/src/types/free.rs) (and
   the other five Free variants) declares its struct with
   `where F: Extract + Functor + 'static`. The `Extract` bound
   is enforced at the type-declaration site, not just on
   inherent methods, so a `Free<NodeBrand<R, S>, A>` instance
   fails to compile when `NodeBrand<R, S>` does not implement
   `Extract`.
2. [`Free::drop`](../../../fp-library/src/types/free.rs) calls
   `<F as Extract>::extract(fa)` to walk deep `Wrap` chains
   iteratively. This is what keeps a 100 000-deep `Wrap` chain
   from stack-overflowing during cleanup; the `Extract` bound
   is load-bearing for the existing `Drop` strategy, which is
   why the bound is on the struct rather than on individual
   methods (Rust requires `Drop` impl bounds to match struct
   bounds exactly).
3. To satisfy `NodeBrand<R, S>: Extract` for typical Run usage,
   the bound recurses into the row brands. For the first-order
   row, `R = CoproductBrand<CoyonedaBrand<E1>, CoproductBrand<...>>`,
   and the recursive bound bottoms out at
   `CoyonedaBrand<E>: Extract`.
4. `CoyonedaBrand<E>::extract` would need to recover an `A` from
   `Coyoneda<E, A>`. The natural implementation lowers the
   Coyoneda (`coyo.lower()` returns `E::Of<A>`, requires
   `E: Functor`) and then calls `<E as Extract>::extract(...)`.
   So the bound transitively requires `E: Extract` for every
   effect type in the row.
5. Effect types (`Reader<E>`, `State<S>`, `Choose`, `Except<E>`,
   `Writer<W>`, etc.) are pure data with no canonical
   "evaluate" semantics: they need a handler to interpret. So
   `Reader<E>: Extract` (and the same for every other effect)
   cannot hold without baking arbitrary semantics into each
   effect type.

The bound is correct for the Free family's general use cases
(`Free<IdentityBrand>` evaluates by unwrapping; `Free<ThunkBrand>`
evaluates by running the thunk). It is over-conservative for the
effects-as-data use case Run needs.

### Investigation: Wrap-depth probe

A probe at
[`fp-library/tests/run_wrap_depth_probe.rs`](../../../fp-library/tests/run_wrap_depth_probe.rs)
(commit `09d676b`) measures `Wrap`-arm depth in Run-shaped
programs over `Free<ThunkBrand, _>` (using `ThunkBrand` because
`Free<IdentityBrand, _>` is layout-cyclic per the Phase 1 step 8
deviation, but the structural behaviour the probe measures is
brand-independent). The probe distinguishes two metrics:

- **Evaluation depth:** how many `Wrap` layers materialise when
  `to_view` applies pending continuations and follows the
  resulting `Wrap` chain via `Extract`. This is what an
  interpreter sees when walking the program.
- **Structural depth:** how many `Wrap` layers exist in the
  original view BEFORE `to_view` applies any continuation.
  This is what `Drop` traverses, because `Drop` dismantles
  the view and continuations in place without applying the
  closures.

Seven tests and their findings:

| Pattern                                                                | Evaluation depth | Structural depth                |
| ---------------------------------------------------------------------- | ---------------- | ------------------------------- |
| `Free::pure(0)`                                                        | 0                | 0                               |
| `pure(0).bind(\|x\| pure(x+1))` chained 1000 times                     | 0                | 0                               |
| `lift_f(eff)` alone                                                    | 1                | 1                               |
| `lift_f(eff).bind(\|x\| pure(x+1))` chained 1000 times                 | 1                | 1                               |
| `pure(0).bind(\|x\| lift_f(eff))` chained 100 times                    | 100              | 0                               |
| `lift_f(eff).bind(\|x\| pure(x+1))` chained 100 000 times, then `drop` | n/a              | succeeds without stack overflow |
| Explicit `Free::wrap(...)` chained 100 times                           | 100              | 100                             |

Bottom-line finding: Run-typical programs (built via `lift_f`
plus a flat `bind` chain) have structural `Wrap` depth at most
1, regardless of bind-chain length. The depth that grows with
sequencing lives in the `CatList` of continuations, which the
existing iterative `Drop` already dismantles without calling
`Extract`. The 100 000-bind drop test passes without stack
overflow even though `Drop` only walks one `Wrap` layer (the
original `lift_f`'s `Wrap`) recursively.

The artificial 100-deep `Free::wrap` chain pattern (last row) is
the case that motivated the existing `Extract`-based iterative
`Drop`. Run-typical usage does not produce this pattern; users
inject effects via `lift_f` (one `Wrap` per call) and chain via
`bind` (no new `Wrap`s). The probe also covers
`nested_lift_f_via_bind_materializes_wraps_at_evaluation_time`,
showing that `bind` closures returning `lift_f` build their
`Wrap`s at _evaluation_ time, not construction time, so they
live in the `CatList` rather than the structural `Wrap` chain.

### Resolution: introduce the `WrapDrop` trait

A new trait `WrapDrop` separates the structural-cleanup question
(what `Drop` needs) from the semantic-interpretation question
(what `Extract` answers). `Extract` continues to mean "given
`F::Of<X>`, give me the `X`" and is used by `evaluate`,
`fold_free`, `resume`, etc. `WrapDrop` instead asks "given
`F::Of<X>`, can you yield the inner `X` without running user
code?", returning `Option<X>`.

#### Trait definition

```rust
pub trait WrapDrop: Kind {
    /// Drop-time decomposition. `Some(x)` means F materially
    /// stores X and the caller can iterate on it. `None` means
    /// F doesn't store X (or storing is closure-captured), so
    /// the caller should let `fa` drop normally.
    fn drop<'a, X: 'a>(fa: Self::Of<'a, X>) -> Option<X>;
}
```

#### Naming rationale

The trait's name reflects that it is the operation `Free`'s
`Wrap` variant performs at drop time. The method name `drop`
does not clash with `std::ops::Drop::drop` because they are
different traits with different receiver shapes
(`std::ops::Drop::drop(&mut self)` is a method;
`WrapDrop::drop(fa: F::Of<'_, X>)` is an associated function).
Call sites use fully-qualified syntax:
`<F as WrapDrop>::drop(fa)`.

#### Free's Drop dispatch

Free's `Drop` impl is rewritten to dispatch on the `Option`:

```rust
match F::drop(layer) {
    Some(inner) => worklist.push(inner.view); // existing iterative path
    None => { /* layer already dropped recursively by the match arm */ }
}
```

#### Per-F policy choices

- **F materially stores the inner X** (e.g., `IdentityBrand`):
  `WrapDrop::drop` returns `Some(<F as Extract>::extract(fa))`,
  preserving the existing iterative path.
- **F's storage runs user code to materialise X but the
  existing test suite relies on iterative dismantling** (e.g.,
  `ThunkBrand`): `WrapDrop::drop` returns
  `Some(<F as Extract>::extract(fa))`. This preserves
  side-effect-on-Drop semantics and the Phase 1
  `deep_drop_does_not_overflow` test. The alternative (return
  `None` to skip closures) was rejected because the closure's
  captures hold inner Frees that would drop recursively for
  100k-deep chains.
- **F does not materially store X at all** (e.g.,
  `CoyonedaBrand<E>`, `CoproductBrand<H, T>`, `CNilBrand`,
  `NodeBrand<R, S>`): `WrapDrop::drop` returns `None`. Drop
  falls through to recursive drop on `fa`; the probe validates
  this is sound for Run-typical patterns because the `F::Of<X>`
  storage doesn't materially recurse on inner Frees (Coyoneda's
  closure would construct a Free if called, but doesn't store
  one; the Coproduct's variants hold Coyonedas which have the
  same property).

#### Documented limitation

Artificial deep `wrap(...)` chains over F's whose
`WrapDrop::drop` returns `None` (e.g., a hand-built 100k-deep
`wrap(Coyoneda(...))` chain) overflow the stack on `Drop`.
Run-typical usage does not generate this pattern, and no
existing test exercises it. The trait's docs warn future
F-authors of the constraint.

### Alternatives considered and rejected

Four resolution paths were evaluated; the chosen path is the
`WrapDrop` introduction described above. The other three are
recorded for design-history transparency:

- **Build a parallel `RunFree`-like substrate without the
  `Extract` bound.** Define six new types in `types/effects/`
  paralleling the six existing Free variants, with relaxed
  bounds and recursive `Wrap` drop. Same insight as the chosen
  path but isolated to Run; Phase 1's Free family would stay
  untouched. Probe-validated as sound for Run usage. Rejected
  because it duplicates the entire substrate (CatList for
  Erased, naive recursive enum for Explicit, custom `Drop`)
  for one architectural concern. `WrapDrop` achieves the same
  expressivity with a single new trait and mechanical-but-
  unified migration.
- **Make Run a newtype struct that internally holds something
  other than a raw `Free<NodeBrand, A>`** (e.g., a
  `Box<dyn ...>` trait object, a custom enum, or a Free over a
  placeholder brand that does implement Extract trivially while
  effect data lives elsewhere). Rejected because it diverges
  from the plan's literal "Run is a Free" model
  ([decisions.md](decisions.md) section 5.2,
  [README of `purescript-run`](https://github.com/natefaubion/purescript-run))
  and the other paths achieve the goal without redesigning
  the relationship.
- **Implement `Extract` for `CoyonedaBrand<E>` /
  `CoproductBrand<H, T>` / `NodeBrand<R, S>` with panic
  semantics** (extract panics with a clear "handler required"
  message; Drop falls back to recursive drop when extract
  panics). Rejected as a footgun: programs that drop unhandled
  Run values panic in legitimate scenarios (program panics in
  user code mid-evaluation, deliberate program discarding,
  test fixtures asserting on Run structure without running it).

## Resolved (2026-04-26): brand-level dispatch for the multi-shot Explicit Free family lands on the by-reference hierarchy

`RcFreeExplicit::bind` requires `A: Clone` (because shared inner
state must clone to recover an owned `A`), and stable Rust does
not admit per-method `where A: Clone` on a `Functor::map` impl.
This is the same constraint that
[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md)
documents under "Unexpressible Bounds in Trait Method Signatures"
for `RcCoyoneda`/`ArcCoyoneda` and addresses under "Memoized Types
Cannot Implement `Functor`" via the by-reference hierarchy
(`RefFunctor`, `RefSemimonad`, `RefMonad` and `SendRef*`
parallels) that `Lazy` already uses. The decision is to follow
`Lazy`'s precedent.

### Brand-level coverage

- `FreeExplicitBrand`: full by-value (`Functor` / `Pointed` /
  `Semimonad` / `Monad`) + full Ref hierarchy.
- `RcFreeExplicitBrand`: `Pointed` on the by-value side; full
  Ref hierarchy (`RefFunctor` / `RefSemimonad` / `RefMonad`,
  plus `RefPointed` and the supporting Ref traits per
  [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md)).
- `ArcFreeExplicitBrand`: `SendPointed` on the by-value side
  (added by step 6 alongside `SendFunctor` etc.); full SendRef
  hierarchy (`SendRefFunctor` / `SendRefSemimonad` /
  `SendRefMonad`, plus the supporting `SendRef*` traits).

### Inherent-method fallback

The remaining by-value operations (`bind`, `map`, etc.) on
`RcFreeExplicit` / `ArcFreeExplicit` ship as inherent methods
with their natural `Clone` bounds, mirroring the
`RcCoyoneda`/`ArcCoyoneda` precedent.

### Alternatives considered and rejected

- Modifying the existing by-value hierarchy to add `Clone`
  bounds taxes the entire ecosystem (`Option`, `Vec`,
  `Identity`, etc.) for one wrapper's storage strategy.
- Adding a parallel `CloneFunctor` / `CloneSemimonad` /
  `CloneMonad` family duplicates the Ref hierarchy's dispatch
  story and adds a third orthogonal trait-and-dispatch axis
  (closure shape, send-ness, Clone-ness). The Ref path is the
  documented library convention and exists today; revisit
  `CloneFunctor` only if Phase 5+ user feedback indicates
  Ref-only brand UX is insufficient for the multi-shot Explicit
  family.

### Plan-level consequences

The decision is reflected in Phase 1 step 7, Phase 2 step 4, the
Motivation section's multi-shot example, and the "Will change"
table's `*RunExplicitBrand` row. Step 7 also schedules an update
to
[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md)'s
"Unexpressible Bounds" classification table to add rows for the
three Explicit Free variants once their impls land.

## Resolved earlier: Erased / Explicit dispatch split for the Free family

The earlier `RcFreeBrand` / `ArcFreeBrand` blocker is resolved by
adopting the Erased/Explicit dispatch split documented in
[decisions.md](decisions.md) section 4.4: the Erased family
(`Free`, `RcFree`, `ArcFree`) is inherent-method only and is not
Brand-dispatched, while the Explicit family (`FreeExplicit`,
`RcFreeExplicit`, `ArcFreeExplicit`) carries the full Brand
hierarchy. Phase 1 grows by three steps to add the two new
Explicit Rc/Arc siblings and the `SendFunctor` trait family;
Phase 2 grows the Run surface to six concrete types (one per Free
variant) plus an `into_explicit` / `from_explicit` conversion
API. See plan.md's resequenced phasing.

## Design-phase blockers (resolved in decisions.md)

All blockers from the design phase are resolved in
[decisions.md](decisions.md):

- Section 4 (six DECISIONs): row encoding, Functor dictionary,
  stack-safety, six-variant Free family with Erased/Explicit
  dispatch split, scoped-effect representation (heftia dual row),
  natural transformations as values.
- Section 9 (nine pre-implementation decisions): target audience,
  partial interpretation, async, IO/Effect story, higher-order
  effects, performance, lifetime constraints, macro
  infrastructure, testing strategy.
