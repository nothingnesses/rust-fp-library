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

> **Maintenance template** (mirrors plan.md's
> [`Current progress`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#current-progress)
> structure; see plan.md's
> [`Implementation protocol`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#implementation-protocol)
> step 3 for the rule).
> Update this section after every step. Order: **Phase status** ->
> **Next greenfield work** -> **Recent commit log (newest-first)** ->
> **Remaining Phase 3 steps** -> **When you hit something unexpected**.
> Refresh the Phase status block in place; do not append new prose.
> Commit-log entries demote to one-line bullets; do not duplicate
> per-step narratives that already live in plan.md, deviations.md,
> resolutions.md, or commit messages. Cross-cutting decisions
> awaiting user input live in plan.md's
> [`Open decisions`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#open-decisions)
> section, not here.

### Phase status

- **Phase 1** (Free family): complete. Steps 1-9 plus two follow-up commits (`WrapDrop` migration and the `Functor` -> `Kind` relaxation).
- **Phase 2** (Run substrate and first-order effects): complete. All 10 steps; the `poc-effect-row/` workspace was deleted in 10b after its tests migrated to [`fp-library/tests/run_row_canonicalisation.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_row_canonicalisation.rs) in 10a. Two recurring constraints surfaced that shape Phase 3 work: the HRTB-poisoning pattern across `ArcRun`-substrate code (see Lessons below) and the per-`A` HRTB-over-types limit that caps brand-level `SendFunctor` coverage on the Arc family.
- **Phase 3** (first-order effect handlers, interpreters, natural transformations): complete. Steps 1-4 (interpreter family), the [2026-05-03 adversarial-review reversal cleanup](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-03-adversarial-review-reversals-delete-run_accum-ship-interpret_with_rec-parameterise-interpret_with-over-refcountedpointer) (F1D / F3A / M3C), the entire effect-suite rollout (steps 5a-5e: `State`, `Reader`, `Except`, `Writer`, `Choose` smart constructors), step 7 (`compile_fail` UI tests), and step 8 (review-remediation documentation pass) all shipped. Step 5e shipped together with a substrate fix on the Erased Free family: new [`RcCatList`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/rc_cat_list.rs) and [`ArcCatList`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/arc_cat_list.rs) reference-counted catenable list variants making `Clone` O(1) and unblocking multi-shot dispatch (see the [2026-05-04 substrate-fix resolution](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-04-phase-3-step-5e-erased-free-family-multi-shot-dispatch-via-rccatlist--arccatlist-option-1c-ii-parallel-reference-counted-catlist-variants)). Two original Phase 3 steps were deferred indefinitely: the [2026-05-04 `interpret_with_rec` deferral](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-04-phase-3-step-5-interpret_with_rec-deferred-indefinitely-option-c) (Phase 3 ships three interpreter primitives instead of four; users chain `interpret_with` then `interpret_rec` for the workaround) and the [2026-05-04 `define_effect!` macro deferral](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit) (revisit when Phase 4 ships or user demand surfaces; design research for five candidate approaches preserved in resolutions.md).
- **Phase 3.5** (pointer-brand-pattern retrofit): complete. All five sub-steps shipped (sub-step 4 is implicitly covered by sub-step 2's `just verify` clean run; sub-step 5 lands the [F4-closure resolutions entry](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-06-phase-3-prior-review-f4-closed-structurally-via-phase-35-retrofit-sibling-boxbrand-family-on-default-run-substrates)). Sub-step 1 lands the `ToDynFnOnce` trait + `BoxBrand` impl. Sub-step 2 lands three sibling effect types and brands ([`BoxState`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/state.rs) / [`BoxReader`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/reader.rs) / [`BoxChoose`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/choose.rs); `BoxStateBrand` / `BoxReaderBrand` / `BoxChooseBrand`) and switches `Run::get` / `Run::put` / `Run::ask` (and the `RunExplicit` parallels) to thread `Box<dyn FnOnce>` continuations via the new pattern. The three-sibling-types interpretation diverges from plan.md's literal "single brand parametrised over `P`" reading because the closure trait shape (FnOnce vs Fn) differs structurally per pointer brand and cannot be unified in stable Rust; full rationale in [deviations.md Phase 3.5 sub-step 2](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md). `Rc<dyn FnOnce>` and `Arc<dyn FnOnce>` remain operationally broken (moving out of a shared pointer invalidates other clones), so `ToDynFnOnce` is `BoxBrand`-only and the new `Box*Brand`s are `BoxBrand`-only by structural bound. RcRun / ArcRun smart constructors are unchanged. Closes Phase 3 prior-review F4 finding structurally rather than as accepted-tradeoff (the resolutions entry lands in sub-step 5). Phase 4 then uses the same per-pointer-brand pattern.
- **Phase 4** (scoped effects via heftia-inspired dual row): steps 0-1 plus step 2 (sub-steps 2.1-2.6) plus step 2a plus step 3.1 (sub-steps 3.1.1-3.1.4) plus the full Local cycle (Val 3.2.1-3.2.4 plus Ref 3.2.5-3.2.8) plus **steps 3.3.1 + 3.3.2 (Bracket Val foundational scaffold + RefFunctor)** shipped. B14 / B15 / B16 / B17 / B18 / B19 all closed (resolution entries in [resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md)). B19 closed 2026-05-08 via Option C (split into 6 cells per Free family) per the [B19 closure entry](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-08-phase-4-step-3.3.1-foundational-scaffold-cells-hardcode-freesub-_-b19-closed-via-option-c-split-into-6-cells-per-free-family). **Next two enumerated steps**: (1) the B19 closure commit (single `feat(effects)` commit reworking the foundational scaffold: fix existing 3 cells per pointer-brand substrate, add 3 new Explicit-family cell siblings, 3 new brand declarations, parallel trait impls; ~1500-line diff); (2) step 3.3.3 (Bracket Val smart constructors per wrapper, six per-wrapper smart constructors mapping each wrapper to its appropriate Erased or Explicit cell). Step 2a closure delivered the substrate-level `interpret_with_either<EBrand, Idx, RMinusE>` primitive across all six Run wrappers; 24 integration tests plus 6 doctests. The Phase 4 design review ([`review/1_scoped_effects_design/review_phase_4_design.md`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/1_scoped_effects_design/review_phase_4_design.md)) and its remediation report ([`review/1_scoped_effects_design/remediation_proposals_phase_4.md`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/1_scoped_effects_design/remediation_proposals_phase_4.md)) shipped earlier, with three POC validations now complete ([`poc_send_catch_brand.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/poc_send_catch_brand.rs) for the F2 parallel-Send-brand pattern; [`poc_rc_run_interpose.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/poc_rc_run_interpose.rs) for the F1 substrate-level `Run::interpose` primitive; [`poc_rc_run_interpret_with_either.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/poc_rc_run_interpret_with_either.rs) for the B4 substrate-level `interpret_with_either` primitive). The 2026-05-06 K1 / K2 [implementation-kickoff sequencing resolution](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-06-phase-4-implementation-kickoff-sequencing-k1-and-k2-poc-3-standalone-commit-first-planmd-numbering-authoritative-for-commit-boundaries) set POC 3's standalone-commit-first ordering and plan.md step numbering as the authoritative commit boundary. Phase 4 ships `Catch<'a, P, E, A>`, `Local<'a, P, E, A>` / `RefLocal`, `Bracket<'a, P, A, B>` / `RefBracket`, and `Span<'a, Tag>` scoped-effect constructors; a parallel `DispatchScopedHandlers` trait; substrate-level `Run::interpose` and `interpret_with_either` primitives on each Run wrapper.

### Next greenfield work

Phase 4 step 3.1 closed (3.1.1-3.1.4 across `abd3d1a3` /
`5bb2d1ae` / `205eaba4` / `faab175f`); the full Local Val cycle
3.2.1 / 3.2.2 / 3.2.3 / 3.2.4 shipped at `ef9b2eec` / `cbe401a4`
/ `970ad399` / `ba080e44`; the RefLocal Ref cycle 3.2.5 / 3.2.6
/ 3.2.7 / 3.2.8 shipped at `1668b2e5` / `6ca9157a` / `15300200` /
`b65ac467`; Bracket Val foundational scaffold (3.3.1) shipped at
`1be2af3e` under Option A fallback. **Phase 4 step 3.3.2
(Bracket Val `RefFunctor` impls) shipped on this branch**: lands
`RefFunctor` on
[`BoxBracketBrand<BoxBrand, Sub, A, B>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/bracket.rs)
(panicking-stub mirror of the `BoxLocalBrand` precedent) and
[`BracketBrand<RcBrand, Sub, A, B>`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/bracket.rs)
(one-line `fa.clone()` since Option A's GAT projection is
independent of the trait's universal type parameter X);
`SendBracketBrand` deliberately omits `RefFunctor` (mirrors
`SendCatchBrand` / `SendLocalBrand` / `SendRefLocalBrand`
precedents). No brand-projection helpers were introduced:
Catch / Local / RefLocal needed `*_modify_ref` /
`*_action_thunk_ref` helpers because their cells contained the
trait's universal `A` type parameter and the GAT projection
inside the trait impl's HRTB-bearing scope failed to normalize
against the concrete enum. Bracket's cell type doesn't reference
the trait's universal `A` at all (under Option A the cell's
body return is `Free<Sub, (A_brand, B_brand)>` where A\*brand and
B_brand come from the brand, not from the trait's universal
scope); the GAT projection normalizes cleanly. 2 new doctests;
total bracket.rs doctests now 15. Deviation logged at
[deviations.md Phase 4 step 3.3.2](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md).
**Next two enumerated steps**:

1. **B19 closure commit** (foundational-scaffold substrate
   split per Free family): fix existing 3 cells per
   pointer-brand substrate (Bracket switches to RcFree,
   SendBracket switches to ArcFree, BoxBracket unchanged), add
   3 new Explicit-family cell siblings (BoxBracketExplicit,
   BracketExplicit, SendBracketExplicit), 3 new brand
   declarations, parallel trait impls; ~1500-line single
   `feat(effects)` commit.
2. **Step 3.3.3** (Bracket Val smart constructors per wrapper):
   six per-wrapper smart constructors, each pairing the wrapper
   with its appropriate Erased or Explicit cell. Restores
   stashed `Run::bracket` probe (`git stash@{0}`) and adds 5
   more wrapper constructors. Each smart-constructor doctest
   uses the marker-struct row pattern validated by the
   [B18 POC](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/poc_bracket_marker_row.rs).

Full enumerated steps with concrete contents at
[plan.md Next greenfield work](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#next-greenfield-work).
Step 3
sub-splits per the per-step protocol: 3.1 Catch (closed:
3.1.1-3.1.4 per the
[resolved B6 4-commit-split decision](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-07-phase-4-step-3.1-sub-step-splitting-b5--b6-closed)),
3.2 Local + RefLocal (sub-split into 3.2.1-3.2.8 per the
[resolved B8 8-commit-split decision](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-07-phase-4-step-3.2-sub-step-splitting--local-action-layout-cycle-reuse--file-organization-b8--b9--b10-closed)),
3.3 Bracket + RefBracket (sub-split into 3.3.1-3.3.8 per
B14 / B15 / B16, of which 3.3.1 + 3.3.2 have now shipped under
Option A per the closed B17), 3.4 Span. R2
risk (Send+Sync threading through scoped-effect closure cells)
surfaced cleanly during 3.1 via the parallel-Send-brand pattern.
Step 4 introduces the`DispatchScopedHandlers`trait + per-wrapper interpret rewrite
(R1 implementation kickoff). Step 5 lands the`scoped_effects!`and`scoped_handlers!`macros. Step 6 implements the bracket
dispatcher with Drop-guard for panic safety (M3). Step 7 lands
standard scoped-handler implementations consuming`interpret_with_either`(Catch's recovery path) and`interpose`(Local's environment-replacement path). Step 8 closes Phase 4
with the review-remediation documentation pass. Phase 4 then
proceeds through plan.md steps 3.2-3.4, 4, 5, 6, 7, 8 in order
with plan.md numbering as the authoritative commit boundary (per
K2): standard scoped-effect constructors using the Phase 3.5
retrofit's pointer-brand pattern (R2 implementation);`DispatchScopedHandlers` trait + per-wrapper interpret rewrite
(R1 implementation); bracket dispatcher with Drop-guard for panic
safety (M3); standard scoped-effect rollout (`Catch`, `Local`/`RefLocal`, `Bracket`/`RefBracket`, `Span`plus`scoped_effects!`and`scoped_handlers!`macros via the new`handler_list_emitter`helper module shared with`handlers!`);
standard scoped handlers; review-remediation documentation pass
closing Phase 4. The Q4 / R1 / R2 half-day prototypes land during
R1 implementation kickoff (alongside steps 1-4 substrate work);
the R3 benchmark commit lands alongside the standard
scoped-effect rollout (step 6 onward). Sub-step planning lives in
plan.md's
[Phase 4 section](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#phase-4-scoped-effects-heftia-inspired-dual-row).

Two Phase 3 steps were deferred and may revisit during or after
Phase 4: step 6
([`define_effect!` macro](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit),
revisit when Phase 4 settles the codegen target or a user surfaces
concrete demand) and step 5's
[`interpret_with_rec`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-04-phase-3-step-5-interpret_with_rec-deferred-indefinitely-option-c)
(deferred indefinitely; users chain `interpret_with` then
`interpret_rec` for the workaround). Pre-public-release polish
work (m1-m9 minor findings) is also outstanding as a non-phased
follow-up commit.

### Recent commit log (newest-first)

Full per-step narratives live in
[plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
`Most recent steps (rolling detail)` and `Earlier completed steps (commit log)`
subsections; per-step deviations in
[deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md);
resolved blockers in
[resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md).

- **Phase 4 step 3.3.2** (this commit): Bracket Val `RefFunctor` impls on `BoxBracketBrand<BoxBrand, Sub, A, B>` (panicking stub) and `BracketBrand<RcBrand, Sub, A, B>` (one-line `fa.clone()`); `SendBracketBrand` deliberately omits `RefFunctor`. No brand-projection helpers needed (Option A's GAT projection is independent of the trait's universal X, so the WF/normalization issue Catch/Local/RefLocal hit doesn't apply). 2 new doctests. Deviation logged at [deviations.md Phase 4 step 3.3.2](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md).
- **Phase 4 step 3.3.1** (`1be2af3e`): Bracket Val foundational scaffold + B14 closure (acquire B-thunk) under Option A fallback after Option C `FreeShape` HKT-trait failed Rust's WF check at `Functor::map`. Three sibling cells (`BoxBracket` / `Bracket` / `SendBracket`) at [`bracket.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/bracket.rs); 5-param struct `Bracket<'a, P, Sub, A, B>` carries `Sub` explicitly, brand `BracketBrand<P, Sub, A, B>` is 4-param, GAT projection `Of<'a, X>` ignores X. Three brand declarations + four substrate-required trait impls per brand (Functor / SendFunctor identity, WrapDrop None, Extract panic-stub). Closes B17. Deviation logged at [deviations.md Phase 4 step 3.3.1](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md). 13 doctests.
- **Phase 4 step 3.2.8** (`b65ac467`): RefLocal Ref integration tests at [`run_ref_local.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_ref_local.rs); 22 shape-only tests across six Run wrappers (T1-T3 plus T4 multi-shot clone on the four Clone-able wrappers). Mirrors `run_local.rs`'s template; T3 invokes `modify(&10)` instead of `modify(10)` (Ref flavour borrows the input). Closes the entire 3.2 sub-cycle (8 commits across Val + Ref).
- **Phase 4 step 3.2.7** (`15300200`): RefLocal Ref smart constructors per wrapper (`Run::ref_local` / `RcRun::ref_local` / `ArcRun::ref_local` / `RunExplicit::ref_local` / `RcRunExplicit::ref_local` / `ArcRunExplicit::ref_local`); each takes `(modify, action)` where `modify: Fn(&E) -> E` (Box: `FnOnce(&E) -> E`). Mechanically derived from `local` smart constructors with three swaps (parameter signature, `ref_new` instead of `new`, RefLocal cell types). 6 smart-constructor doctests.
- **Phase 4 step 3.2.6** (`6ca9157a`): RefLocal Ref `RefFunctor` impls on `BoxRefLocalBrand<BoxBrand, E>` (stub-everywhere) and `RefLocalBrand<RcBrand, E>` (faithful) via two `#[doc(hidden)]` brand-projection helpers (`ref_local_modify_ref` / `ref_local_action_thunk_ref`). `SendRefLocalBrand` deliberately omits `RefFunctor` (mirrors `SendLocalBrand` precedent). 4 new doctests; total ref_local.rs doctests 17.
- **Phase 4 step 3.2.5** (`1668b2e5`): RefLocal (Ref flavour) foundational scaffold at [`ref_local.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/ref_local.rs) (`BoxRefLocal` / `RefLocal` / `SendRefLocal`; three brands; four of five substrate-required trait impls per brand) plus closure-trait matrix completion: `ToDynFnOnce::ref_new` trait method, `BoxBrand` impl, free-function shim. Closes B11 (matrix gap) + B12 (variant uniformly `Local`). 13 doctests.
- **Phase 4 step 3.2.4** (`ba080e44`): Local Val integration tests at [`run_local.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_local.rs); 22 shape-only tests across six Run wrappers (T1-T3 plus T4 multi-shot clone on the four Clone-able wrappers). Mirrors `run_catch.rs`'s template; T3 adjusted to verify `modify(some_e) == expected_scalar` because modify is `E -> E` (vs Catch's T3 which verified handler(error) produced a wrapper-typed program).
- **Phase 4 step 3.2.3** (`970ad399`): Local Val smart constructors per wrapper (`Run::local` / `RcRun::local` / `ArcRun::local` / `RunExplicit::local` / `RcRunExplicit::local` / `ArcRunExplicit::local`); each takes `(modify, action)` and returns a wrapper-typed program suspended at the scoped Local layer. Per-wrapper bound shapes mirror Catch's. 6 smart-constructor doctests.
- **Phase 4 step 3.2.2** (`cbe401a4`): Local Val `RefFunctor` impls on `BoxLocalBrand<BoxBrand, E>` (stub-everywhere) and `LocalBrand<RcBrand, E>` (faithful) via two `#[doc(hidden)]` brand-projection helpers (`local_modify_ref` / `local_action_thunk_ref`). `SendLocalBrand` deliberately omits `RefFunctor` (mirrors `SendCatchBrand` precedent). 4 new doctests; total local.rs doctests 17.
- **Phase 4 step 3.2.1** (`ef9b2eec`): Local Val foundational scaffold at [`local.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/local.rs). Three sibling effect types (`BoxLocal` / `Local` / `SendLocal`); three brands; four of five substrate-required trait impls per brand. `SendLocalBrand` does not impl `Functor` (mirrors `SendCatchBrand` / `SendStateBrand` precedents). 13 doctests. Closes B8 + B9 + B10.
- **Phase 4 step 3.1.4** (`faab175f`): Catch integration tests; 22 shape-only tests at [`run_catch.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_catch.rs) across six Run wrappers (T1-T3 plus T4 multi-shot clone on the four Clone-able wrappers).
- **Phase 4 step 3.1.3** (`205eaba4`): six per-wrapper `catch` smart constructors plus B-thunk action-representation refactor (per-pointer-brand pointer of unit-arg `FnOnce/Fn(()) -> A` thunk) closing B7. Manual `Clone` impls for `Catch` / `SendCatch`; ArcRun gains `make_node_scoped` HRTB workaround helper.
- **Phase 4 step 3.1.2** (`5bb2d1ae`): `RefFunctor` impls on `BoxCatchBrand<BoxBrand, E>` and `CatchBrand<RcBrand, E>` via three `#[doc(hidden)]` brand-projection helpers (B5 Option A; mirrors `arc_run::unwrap_first` precedent). `SendCatchBrand` deliberately omits `RefFunctor`.
- **Phase 4 step 3.1.1** (`abd3d1a3`): Catch foundational scaffold (three sibling effect types `BoxCatch` / `Catch` / `SendCatch`; three brands; four of five substrate-required trait impls per brand). `SendCatchBrand` does not impl `Functor` (mirrors `SendStateBrand` precedent).
- **Phase 4 step 2a**: substrate-level `interpret_with_either` primitive across all six Run wrappers; generalises POC 3's two-effect-row template. Bundled commit; 24 integration tests + 6 doctests. Step 7's `Catch` cons-cell consumes this primitive.
- **Phase 4 step 2.6**: `ArcRunExplicit::interpose` substrate primitive. Closes Phase 4 step 2 (all six Run wrappers ship `interpose`); R1 again did NOT surface.
- **Phase 4 step 2.5**: `RcRunExplicit::interpose` substrate primitive. Notably simpler than 2.4 (no `Box::new` wrapping); R1 again did NOT surface.
- **Phase 4 step 2.4** (`1d9ac0cc`): `RunExplicit::interpose` substrate primitive. R1 (Explicit-family HRTB-poisoning) did NOT surface; compiled cleanly without ArcRun-style workaround helpers.
- **Phase 4 step 2.3** (`ebe759d3` + `bdf9245d`): `ArcRun::interpose` substrate primitive. R2 (Send+Sync propagation) cleared structurally with no new substrate machinery.
- **Phase 4 step 2.2** (`c75638f4`): `Run::interpose` substrate primitive (mechanical translation of step 2.1's RcRun template).
- **Phase 4 step 2.1** (`082d025e`): `RcRun::interpose` substrate primitive; first per-wrapper interpose. The `EmbedIndices` extra type parameter (frunk's `CoproductEmbedder` indices) is a structural necessity logged at deviations.md.
- **Phase 4 step 1** (`df1fb60b`): `ScopedCoproduct<H, T>` and `ScopedNil` row-encoding aliases at [`scoped.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/scoped.rs); pure naming layer over existing dual-row machinery.
- **Phase 4 step 0** (`f97e5552`): standalone POC 3 validation at [`poc_rc_run_interpret_with_either.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/poc_rc_run_interpret_with_either.rs). Adopted per K1 resolution; generic rollout in step 2a.
- **Phase 3.5 sub-step 5** (`4471629d`): F4 closure resolutions.md entry closing Phase 3.5; re-opens (3.a-1) "one effect type per operation" sub-decision with four-argument justification.
- **Phase 3.5 sub-step 3** (`89546709`): docs-only update to [`pointer-abstraction.md`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/pointer-abstraction.md) adding `ToDynFnOnce` to the trait diagram, table, and BoxBrand row plus the `(closure-semantic, pointer-capability)` matrix.
- **Phase 3.5 sub-step 2** (`a762fa27`): Phase 3 effect retrofit to `BoxBrand` + `ToDynFnOnce` on default Run; three sibling effect types and brands (`BoxState` / `BoxReader` / `BoxChoose`); diverges from plan.md's literal "single brand parametrised over `P`" reading per deviations.md.
- **Phase 3.5 sub-step 1** (`b067f912`): `ToDynFnOnce` trait + `BoxBrand` impl. `RcBrand` / `ArcBrand` deliberately do NOT implement it.
- **Phase 3 step 8** (`5911d579`): review-remediation documentation pass closing Phase 3 (F2A / F4A / F5A / M4 / M6A / M7A).
- **Step 7**: three `compile_fail` UI tests in [`fp-library/tests/ui/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/ui/) for Phase 3 negative cases: `run_choose_not_found.rs` (single-shot wrappers reject `Choose` constructor; E0599), `run_smart_constructor_type_mismatch.rs` (smart-constructor result type bound to row's effect parameterization; E0277 on `CoprodUninjector`), `interpret_missing_handler.rs` (handler list shorter than row; E0277 on `DispatchHandlers`).
- **Step 5e** (`adbde7b` + `9f58492` + `de4d0eb`): `Choose` smart constructors on the four multi-shot wrappers + Erased Free family multi-shot substrate fix (new `RcCatList` / `ArcCatList`, capture-and-clone-per-call replaces `Cell::take` / `Mutex::take` in `*Free::to_view`).
- **Step 5d** (`5905e9b`): `Writer` smart constructors on all six wrappers using a single `WriterBrand<W>` (no parallel `SendWriterBrand` because Writer has no `dyn Fn` continuation).
- **Step 5c** (`66eca99`): `Except` smart constructors on all six wrappers using a single `ExceptBrand<E>` (same shape as Writer).
- **Step 5b** (`4162d20`): `Reader` smart constructors on all six wrappers with a parallel `SendReaderBrand` for the Arc family (motivated by `Arc<dyn Fn>: !Send + !Sync`).
- **Brands reorg** (`72f753e`): extracted effect-specific brands to `crate::brands::effects` while preserving flat re-exports at `crate::brands` via `pub use effects::*;`.
- **Step 5a.4 + 5a.6** (`7a0d04b`): Arc family `get` / `put` smart constructors using a parallel `SendStateBrand` / `SendState` per the 2026-05-03 option-(c) re-ratification (option (b) per-method bounds was discovered structurally unimplementable). Closes step 5a.
- **Step 5a.4 + 5a.6 follow-ups** (`6db4a26` + `690df0f` + `000a732`): `ArcCoyoneda` algebra migrated to `F: SendFunctor` per the 2026-05-04 option-(a) resolution; `SendFoldable` trait introduced; State integration tests landed at [`fp-library/tests/run_state.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_state.rs).
- **Step 5a.5** (`db07a2f`): Explicit non-Arc family `get` / `put` (`RunExplicit` + `RcRunExplicit`) threading `RcBrand`.
- **Step 5a.3** (`619127e`): `RcRun::get` / `RcRun::put` plus a manual `Clone` impl for `State` gated on `S: Clone`.
- **Reversal cleanup** (`05be270` F1D + `f8031c5` F3A + `b8c9b3c` M3C): three commits implementing the 2026-05-03 reversal resolution against steps 2, 3, 4. Deletes `run_accum` / `run_accum_rec`, tightens `S = CNilBrand` on the interpreter family, parameterises `interpret_with` over `P: RefCountedPointer`.
- **Step 5a.1 + 5a.2** (`96bc448` + `f865152`): State effect type machinery (`StateBrand<P, S>` + `State<'a, P, S, A>`) plus `Run::get` / `Run::put` smart constructors.
- **Cross-cutting docs/macros during step 5a** (`4f0e977` + `3a5a0a8`): wrapped `handlers.rs` / `interpreter.rs` / `member.rs` in `#[fp_macros::document_module]`; tightened `#[document_examples]` validation to reject six trivially-true assertion patterns.
- **Step 4** (`bd540d5` + `fafcfde`): `interpret_rec` / `run_rec` MonadRec-target interpreter family. M's lifetime pinned per family because stable Rust closures can't be HRTB-polymorphic over `Thunk<'h, T>`-shaped types.
- **Step 3** (`ff84f20`): pipeline row-narrowing `interpret_with::<EBrand, Idx, RMinusE>` plus empty-row terminal `extract` across all six wrappers.
- **Step 2** (`d5efe2a`): `interpret` / `run` simple all-handlers-at-once interpreter family across all six wrappers + the `DispatchHandlers` trait with three Coyoneda-variant cons-cell impls.
- **Step 1** (`82dd7bb`): [`handlers!{...}`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/handlers.rs) macro plus `nt()` builder fallback for assembling natural transformations; runtime carrier types `Handler<E, F>` / `HandlersNil` / `HandlersCons<H, T>`.

### Remaining Phase 3 steps

- **Step 6 [DEFERRED 2026-05-04]:** [`define_effect!`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/) macro mechanically generating the six per-wrapper variants from one user declaration. Deferred until Phase 4 (scoped effects) ships or a real user surfaces concrete demand for custom effects. Five design approaches and seven open questions preserved in [resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit) for revisit.

### When you hit something unexpected

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
`CNil`, plus three cons-cell impls , one per Coyoneda variant
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
`(VariantF r (Run r a) -> m (Run r a))` , mono in `a`. fp-library
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

With the standard first-order effects shipped (Phase 3 step 5),
non-Identity shapes (`State<S>` whose `Of<NextProgram>` is a
closure `S -> NextProgram`) make the recursive narrowing lazy:
`Functor::map` over a closure composes with the new function,
and the recursion is deferred until the state value is
supplied. This is a property worth exploiting for stack safety
on State-heavy programs.

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
boundaries. State-family effect types (Phase 3 step 5a) use
`RefCountedPointer::Of` for the unified single-thread surface
([`StateBrand` / `State`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/state.rs))
and a parallel `SendRefCountedPointer::Of`-based
`SendStateBrand` / `SendState` for the Arc family (per the
[2026-05-03 option-(c) resolution](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified)).
Trying to bake `Send + Sync` into `RefCountedPointer::Of`'s
projection at use-site bounds was discovered structurally
unimplementable: `Arc<dyn Fn(...)>: Send + Sync` is provably
false because the trait object's bounds don't include
`Send + Sync`, and use-site bounds can't refine a structural
type-level fact.

This is a recurring theme: brand-level `SendFunctor` impls on
types whose projection is `<P as RefCountedPointer>::Of<...>`
must use the `SendRefCountedPointer` projection instead so
the trait object's bounds bake in `Send + Sync` at the type
level. Plan for a parallel `Send*Brand` from the start
whenever an effect type's representation includes
`dyn Fn(...) -> A` continuations.

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

1. Read [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
   `Current progress` section. Phase 3 closed at step 8
   (commit `5911d579`); Phase 3.5 closed at sub-step 5
   (commit `4471629d`). All Phase 4 design questions and
   implementation-kickoff sequencing decisions are resolved:
   B1-B4 + Q1-Q3 + Q5 closed by the 2026-05-05 design-
   adoption commit (`6e960701`) plus the Phase 3.5 retrofit
   landings (B3 implementation); K1 + K2 closed 2026-05-06
   (Option A adopted for both: POC 3 lands as a standalone
   validation commit first; plan.md step numbering is
   authoritative for commit boundaries). The next concrete
   commit is **Phase 4 step 0**: standalone POC 3 validation
   at [`fp-library/tests/poc_rc_run_interpret_with_either.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/),
   mechanical from [`interpret_with`'s body](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/run.rs#L885-L900)
   with one branch substitution; conventional commit prefix
   `test(effects)`. Q4 / R1 / R2 half-day prototypes land
   during R1 implementation kickoff (alongside steps 1-4
   substrate work); R3 benchmark commit lands alongside the
   standard scoped-effect rollout (step 6 onward). Plan.md's
   [Active blockers](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#active-blockers)
   subsection has no active items.
2. **Phase 4 (scoped effects):** see
   [plan.md's Phase 4 section](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#phase-4-scoped-effects-heftia-inspired-dual-row)
   for the full step list and constructor signatures. Phase 4
   follows the
   [remediation report's Sequencing Plan](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/1_scoped_effects_design/remediation_proposals_phase_4.md)
   items 3, 5, 6, 7, 8, 9 with the resolved
   [Phase 4 pre-implementation design questions](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md#resolved-2026-05-05-phase-4-pre-implementation-design-questions-b1-b4-q1-q3-q5-closed-by-design-adoption-commit-6e960701).
   Substrate-level `Run::interpose` is POC-validated at
   [`fp-library/tests/poc_rc_run_interpose.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/poc_rc_run_interpose.rs);
   the parallel-Send-brand pattern is POC-validated at
   [`fp-library/tests/poc_send_catch_brand.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/poc_send_catch_brand.rs).
   POC 3 (`interpret_with_either` substrate primitive on
   `RcRun`) is pending validation; it must land before the
   step that introduces `interpret_with_either` ships
   generically across all six Run wrappers. Reuse Phase 3.5's
   `BoxBrand` + `ToDynFnOnce` pattern for all user-supplied
   scoped-effect handlers.
3. Read [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
   [section 4.5](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md#45-decision-scoped-effect-representation-via-a-heftia-inspired-dual-row)
   (scoped effects) and
   [section 4.6](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md#46-decision-natural-transformations-as-values)
   (natural transformations) for Phase 4 commitment context.
   Section 4.3 (interpreter families) is the reference for
   any work that touches the interpreter primitive surface.
4. If your step touches type-class impls, brand-level dispatch, or
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
5. Per-step doc maintenance follows the per-step protocol below
   and plan.md's
   [Implementation protocol](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#implementation-protocol)
   step 3: refresh plan.md's `Current progress` four required
   subsections in their canonical order in place, mirror the
   same template into prompt.md's `Current resume point`, and
   append a
   [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
   entry for any per-step deviation from plan text. The
   subsection names and ordering rule are restated at the top
   of the resume point above. When the rolling-detail subsection
   grows past 3 entries, demote the oldest narrative to a
   one-line bullet in the commit log; verify any load-bearing
   context is preserved in deviations.md / resolutions.md /
   commit message before demoting.

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
     Follow plan.md's
     [`Implementation protocol`](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md#implementation-protocol)
     step 3: refresh the four required subsections
     (`Phase status`, `Next greenfield work`, `Most recent
steps (rolling detail)`, `Earlier completed steps
(commit log)`) in place. Edit the Phase status block;
     do not append new prose. When the rolling-detail
     subsection grows past 3 entries, demote the oldest
     narrative to a one-line bullet in the commit log;
     verify any load-bearing context is preserved in
     deviations.md / resolutions.md / commit message before
     demoting.
   - This file's `Current resume point` section, mirroring
     the same template (`Phase status` -> `Next greenfield
work` -> `Phase 3 commit log (newest-first)` ->
     `Remaining Phase 3 steps`). Refresh in place; do not
     append.
   - [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
     (append-only) for any per-step deviation from the original
     plan text. Group entries by phase and step, matching the
     existing structure.
   - plan.md's `Open decisions` section if a sub-step split or
     other user-input-pending decision lands or gets surfaced.
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
  `scoped_effects!` (Phase 4 step 4) , all land in the same
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
