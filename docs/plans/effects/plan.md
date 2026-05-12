# Plan: Port purescript-run to fp-library

**Status:** Phase 1, Phase 2, Phase 3, and Phase 3.5 are complete.
Phase 4 is in progress; Phase 4 step 7.3 has shipped the complete
standard scoped-dispatcher set (`CatchDispatcher`, `SpanDispatcher`,
`LocalDispatcher`, `RefLocalDispatcher`, `BracketDispatcher`, and
`RefBracketDispatcher`) across all six wrappers. Phase 4 step 8's
Bracket / RefBracket lifecycle test slice has shipped, including the
B20 `ArcRun::bracket` retry through a direct `Coproduct` scoped row;
the subsequent non-Bracket slice fills the missing Rc/Arc Explicit
Catch / Span dispatcher cases and the scoped-row negative UI cases
enumerated for step 8. Step 8a is not needed for the dispatcher test
surface. B30 is resolved via the continuation-aware raw `Run`
scoped-step path for Box-backed dispatchers that must run `interpose`
before reattaching the erased `Free` continuation queue; Rc/Arc
wrappers stay on the ordinary interpose-backed dispatcher path. B31 is
resolved via Option B: add a continuation-aware scoped-handler path for
around-action handlers before completing the remaining Span lifecycle
tests. B32 is resolved via Option D / H1: implement that B31 path by
adding substrate-level continuation insertion first, then revisit the
larger H2 carrier rewrite or H3 protocol-family split only if similar
continuation-boundary issues surface again. B33 is resolved via Option
A: retrofit the Explicit Free substrates with continuation queues /
raw-step decomposition, starting with a small `FreeExplicit` proof
before extending to `RcFreeExplicit` and `ArcFreeExplicit`. B34 is
resolved via Option A with an explicit H2 fallback: first prove a
private existential visitor raw-step protocol for `FreeExplicit`; if
that protocol cannot stay private to the Explicit substrates and
requires wrapper-wide, object-stored, or handler-list-visible carrier
state, stop the proof and reopen the deferred H2 internal
continuation-carrier rewrite. The first proof audit surfaced B35,
which is resolved via Option A: promote H2 into Phase 4 step 7.4 now.
The private visitor proof needs a callback generic over an
existentially hidden intermediate type, which Rust trait objects cannot
provide; the continuation boundary is now handled as an internal
wrapper-owned carrier invariant instead of as a per-substrate visitor
escape hatch.

## Current progress

> **Maintenance template** (see [Implementation protocol](#implementation-protocol) step 3 for the full rule).
> Update this section after every step. Keep it under ~120 lines. Order: **Phase status** -> **Next greenfield work** -> **Recent history lookup**. Do not duplicate per-step history here; use `git log`, `git show`, [deviations.md](deviations.md), [resolutions.md](resolutions.md), and commit messages. Do not append new prose to the intro paragraphs; refresh the Phase status block in place. Cross-cutting decisions awaiting user input live in the dedicated [Open decisions](#open-decisions) section, not here.

### Phase status

- **Phase 1** (Free family, [`fp-library/src/types/`](../../../fp-library/src/types/)): complete. Steps 1-9 plus two follow-up commits (the `WrapDrop` migration and the `Functor` -> `Kind` relaxation).
- **Phase 2** (Run substrate and first-order effects): complete. All 10 steps; the `poc-effect-row/` workspace was deleted in 10b after its tests migrated to [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs) in 10a.
- **Phase 3** (first-order effect handlers, interpreters, natural transformations): complete. Steps 1-4 (interpreter family), the [2026-05-03 adversarial-review reversal cleanup](resolutions.md#resolved-2026-05-03-adversarial-review-reversals-delete-run_accum-ship-interpret_with_rec-parameterise-interpret_with-over-refcountedpointer) (F1D / F3A / M3C), the entire effect-suite rollout (steps 5a-5e: `State`, `Reader`, `Except`, `Writer`, `Choose` smart constructors), step 7 (`compile_fail` UI tests), and step 8 (review-remediation documentation pass) all shipped. Step 5e also delivered a substrate fix on the Erased Free family: new [`RcCatList`](../../../fp-library/src/types/rc_cat_list.rs) and [`ArcCatList`](../../../fp-library/src/types/arc_cat_list.rs) reference-counted catenable list variants making `Clone` O(1) and unblocking multi-shot dispatch (see the [2026-05-04 substrate-fix resolution](resolutions.md#resolved-2026-05-04-phase-3-step-5e-erased-free-family-multi-shot-dispatch-via-rccatlist--arccatlist-option-1c-ii-parallel-reference-counted-catlist-variants)). Two original Phase 3 steps were deferred indefinitely: the [2026-05-04 `interpret_with_rec` deferral](resolutions.md#resolved-2026-05-04-phase-3-step-5-interpret_with_rec-deferred-indefinitely-option-c) (Phase 3 ships three interpreter primitives instead of four) and the [2026-05-04 `define_effect!` macro deferral](resolutions.md#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit) (revisit when Phase 4 ships or user demand for custom effects surfaces).
- **Phase 3.5** (pointer-brand-pattern retrofit): complete. All five sub-steps shipped (sub-step 4 is implicitly covered by sub-step 2's `just verify` clean run; sub-step 5 lands the [F4-closure resolutions entry](resolutions.md#resolved-2026-05-06-phase-3-prior-review-f4-closed-structurally-via-phase-35-retrofit-sibling-boxbrand-family-on-default-run-substrates)). Sub-step 1 lands the [`ToDynFnOnce`](../../../fp-library/src/classes/to_dyn_fn_once.rs) trait + `BoxBrand` impl at [`box_ptr.rs`](../../../fp-library/src/types/box_ptr.rs). Sub-step 2 lands three sibling effect types and brands ([`BoxState`](../../../fp-library/src/types/effects/state.rs) / [`BoxReader`](../../../fp-library/src/types/effects/reader.rs) / [`BoxChoose`](../../../fp-library/src/types/effects/choose.rs); [`BoxStateBrand`](../../../fp-library/src/brands/effects.rs) / [`BoxReaderBrand`](../../../fp-library/src/brands/effects.rs) / [`BoxChooseBrand`](../../../fp-library/src/brands/effects.rs)) and switches `Run::get` / `Run::put` / `Run::ask` (and the `RunExplicit` parallels) to thread `Box<dyn FnOnce>` continuations via the new pattern. The three-sibling-types interpretation diverges from plan.md's literal "single brand parametrised over `P`" reading because the closure trait shape (FnOnce vs Fn) differs structurally per pointer brand and cannot be unified in stable Rust; full rationale in [deviations.md Phase 3.5 sub-step 2](deviations.md). `Rc<dyn FnOnce>` and `Arc<dyn FnOnce>` remain operationally broken (moving out of a shared pointer invalidates other clones), so `ToDynFnOnce` is `BoxBrand`-only and the new `Box*Brand`s are `BoxBrand`-only by structural bound. RcRun / ArcRun smart constructors are unchanged. Closes Phase 3 prior-review F4 finding structurally rather than as accepted-tradeoff (the resolutions entry lands in sub-step 5). Phase 4 then uses the same per-pointer-brand pattern.
- **Phase 4** (scoped effects via heftia-inspired dual row): steps 0-1, step 2 (sub-steps 2.1-2.6), step 2a, Catch 3.1.1-3.1.4, Local / RefLocal 3.2.1-3.2.8, Bracket / RefBracket 3.3.1-3.3.8, Span 3.4.1-3.4.3, step 4 (Q4 method-generic viability prototype, `DispatchScopedHandlers` / scoped-handler carrier scaffold, and wrapper interpreter plumbing), step 5 base scoped row / scoped-handler macros, step 5b `define_scoped_row!` item-position marker-row macro, step 6a scoped-row-preserving primitive retrofit across all six wrappers, the B29 scoped-dispatcher architecture checkpoint, step 7.1 `CatchDispatcher` / `SpanDispatcher`, step 7.2 `LocalDispatcher` / `RefLocalDispatcher`, step 7.3 `BracketDispatcher` / `RefBracketDispatcher`, steps 7.4.2-7.4.4b.1 H2 internal continuation-carrier contract plus default `Run` / `RunExplicit` / `RcRun` / `RcRunExplicit` / `ArcRun` / `ArcRunExplicit` carrier proofs, the B37 family-specific private carrier protocol split, the carrier-aware scoped-handler list path, private raw-step extraction for default and erased shared wrappers, and the Span-first two-slot protocol / macro-spelling proof, the step 8 Bracket / RefBracket lifecycle test slice, and the step 8 Catch / Span explicit-wrapper plus negative-UI slice have shipped. Closed blockers and design decisions are tracked in [resolutions.md](resolutions.md) and [deviations.md](deviations.md). B21 is resolved via Option A: Span remains Val-only at the user-facing level, but the implementation mirrors Catch and Local with Box/Rc/Arc action-thunk substrate cells. B22 is resolved via Option A: Span stores tags by value and adds clone/send bounds only where Rc/Arc substrates require them. Q4 is resolved via Option A: a method-generic scoped dispatch shape can consume a real first-order `DispatchHandlers` cons-list from an `RcRun` scoped-handler prototype; the production trait uses argument-position `impl DispatchHandlers` for the same static-dispatch shape. B23 is resolved via Option B: add a separate item-position `define_scoped_row!` macro for named marker rows while keeping `scoped_effects![...]` as the type-position row macro. B29 is resolved via Option A plus Option B surface polish: standard scoped dispatchers use each wrapper's actual peeled-layer lifetime, interpose-backed dispatchers carry row-removal evidence, helper constructors hide witness-bearing type names where practical, `SpanDispatcher` stays witness-free, and Bracket dispatchers remain result-specific. B30 is resolved via Option C: default `Run` uses continuation-aware raw scoped-step paths for Box-backed dispatchers that must run `interpose` before reattaching the erased `Free` continuation queue; `RunExplicit` and Rc/Arc wrappers use ordinary dispatcher/interpose paths. B31 is resolved via Option B: add a continuation-aware scoped-handler path for around-action handlers before claiming nested Span lifecycle ordering. B32 is resolved via Option D / H1: build the B31 path through substrate-level continuation insertion first, not recursive interpretation through borrowed handler lists. B33 is resolved via Option A: retrofit the Explicit Free substrates with continuation queues / raw-step decomposition, starting with `FreeExplicit`. B34 is resolved via Option A with an explicit H2 fallback: prove a private existential visitor raw-step protocol for `FreeExplicit`, and reopen H2 only if the visitor cannot stay private to the Explicit substrates. B35 is resolved via Option A: promote the H2 internal continuation-carrier rewrite into Phase 4 step 7.4 because the B34 private visitor proof requires a generic callback on an existential bind node, which Rust trait objects cannot express. B36 is resolved via Option A: the typed `RunExplicitScopedContinuation` boundary keeps the action and outer continuation separate without `Any`, unsafe erasure, or dyn-generic callbacks; the H3 fallback did not trigger. B37 is resolved via Option A: `ScopedContinuation` stays the shared handle, `ScopedResumeTypes` carries shared action associated types, and default-erased / single-shot Explicit / Rc-shared / Arc-shared resume methods live on family-specific private traits. H3 otherwise remains deferred to Phase 6+ as a possible public protocol-family facade over the H2 carrier if custom handler APIs later need explicit classes. Step 7.2 also relaxed Arc `interpose` / `interpret_with` effect-brand bounds to `SendFunctor` (not ordinary `Functor`) so `SendReaderBrand` can participate in Arc-family Local dispatch. Step 7.3 completed normal-path effectful release for Bracket / RefBracket dispatch while retaining ordinary Rust `Drop` as the unwind cleanup guarantee. Step 7.4.2 added private `ScopedContinuation` carrier vocabulary in the interpreter substrate for wrapper-owned normal resume and result-preserving post-action continuation insertion before outer continuations are reattached. Step 7.4.2a added `RunScopedContinuation`, proving default `Run` can resume `RawRunFree` normally and append one raw post-action continuation before the erased continuation queue. Steps 7.4.2b.0 and 7.4.2b.1 added `RunExplicitScopedContinuation`, proving typed Explicit normal resume, post-action insertion before the outer continuation, and borrowed action values. Steps 7.4.2c.0 and 7.4.2c.1 split the private carrier protocol by wrapper family and moved the shipped carriers onto `DefaultScopedResume` and `ExplicitScopedResume`; step 7.4.2c.2 added `RcRunScopedContinuation` on `RcScopedResume` with repeated-resume and post-action insertion coverage; step 7.4.2c.3 added `RcRunExplicitScopedContinuation` with repeated-resume, post-action insertion, and borrowed action value coverage; step 7.4.2d added `ArcRunScopedContinuation` and `ArcRunExplicitScopedContinuation` with `Send + Sync` carrier and post-action obligations plus repeated shared-use coverage. Step 7.4.3 added `DispatchScopedCarrierHandler` / `DispatchScopedCarrierHandlers`, preserving ordinary scoped dispatch while adding a private list-walking route from `SBrand::Of<ActionProgram>` plus `ScopedContinuation` to `NextProgram` for around-action handlers. Step 7.4.4a added `RcFreeRawStep` / `ArcFreeRawStep` plus `continue_from_erased` helpers so erased shared substrates can keep `RcCatList` / `ArcCatList` queues outside a selected suspended action. Steps 7.4.4b.0 and 7.4.4b.1 documented and proved the two-slot around-action protocol: `BoxSpanBrand<BoxBrand, Tag>` remains a static row brand while `SBrand::Of<'a, ActionProgram>` can carry borrowed action payloads and `NextProgram` remains distinct; `scoped_effects!` and `define_scoped_row!` preserve that spelling. Step 8 now covers Bracket / RefBracket acquire -> body -> release order across the wrapper families, `ArcRun::bracket` through a direct `Coproduct` scoped row, Catch recovery / recovery-rethrow across all six wrappers, scoped operation in an empty scoped row, scoped handler-list omission for a non-empty Catch scoped row, Bracket closure-shape mismatch, and RefBracket non-refcounted-pointer rejection. R3 remains a non-blocking benchmark follow-up below.

B38 is resolved via Option A for default and erased shared wrappers:
wrapper interpreters first get private carrier raw-step extraction
boundaries, then the carrier-aware scoped-handler list is wired through
those boundaries. B39 is resolved via Option B: the B38 Option C
fallback is now active for the Explicit-family path, so scoped-effect
constructors preserve a runner/carrier boundary before Explicit `bind`
distributes the outer continuation. B40 identified that
carrier-backed scoped effects need to preserve the selected action type
while `Functor::map` changes the final next-program slot. B41 is
resolved via Option D: do not encode the selected action type in the
`'static` row brand; instead split ordinary one-slot scoped rows from a
minimal two-slot around-action protocol whose action boundary remains
in a method/GAT position and whose final-program slot remains the
ordinary mapped result. Steps 7.4.4b.0 and 7.4.4b.1 shipped the
focused protocol and macro-spelling proofs. B42 is resolved via the
Option B-first plan: the scoped operation owns the selected action, the
continuation carrier owns only the outer resume boundary, and the
7.4.4b.2 implementation starts with a narrow `RunExplicit` Span proof
before migrating every carrier. Option C remains the fallback if that
proof shows the Explicit substrate must store the carrier cell directly
inside carrier-backed scoped layers. B43 executed that proof gate and
activated the Option C fallback: the current `RunExplicit::bind` maps
the Span action slot to the final program before interpretation. The
private carrier-cell Span layer shape and focused `RunExplicit` Span
dispatcher proof have shipped. The next implementation step extends the
same carrier-cell proof to the shared Explicit wrappers.

### Next greenfield work

Phase 4 step 5 / 5b macro work is verified:
[`fp-macros/src/effects/effects_macro.rs`](../../../fp-macros/src/effects/effects_macro.rs)
now provides `scoped_effects!` as a public, lexically sorted,
unwrapped `CoproductBrand` scoped-row macro, and
[`fp-macros/src/effects/handlers.rs`](../../../fp-macros/src/effects/handlers.rs)
now provides `scoped_handlers!{...}` as the scoped parallel to
`handlers!`. The two handler macros share the same parser, sort, and
cons-list emission helper. Integration coverage lives in
[`fp-library/tests/effects_macro.rs`](../../../fp-library/tests/effects_macro.rs)
and
[`fp-library/tests/handlers_macro.rs`](../../../fp-library/tests/handlers_macro.rs).
[`fp-macros/src/effects/scoped_row.rs`](../../../fp-macros/src/effects/scoped_row.rs)
now provides `define_scoped_row!` as the public item-position macro
for concrete named marker rows, including structural bare-`Self`
substitution before lexical sorting. Integration coverage lives in
[`fp-library/tests/define_scoped_row_macro.rs`](../../../fp-library/tests/define_scoped_row_macro.rs).

**Next greenfield step: Phase 4 step 7.4.4b.2d, extend the
carrier-cell Span proof to shared Explicit wrappers.** Steps
7.4.4b.2a-2c shipped the proof gate, the private
`RunExplicitSpanCarrierLayer` shape, and the focused `RunExplicit`
Span dispatcher path that consumes the carrier cell, observes the tag,
and inserts post-action work before the outer continuation for borrowed
selected action payloads. Extend that proof to `RcRunExplicit` and
`ArcRunExplicit`, preserving repeated shared resume for Rc, `Send +
Sync` obligations for Arc, and borrowed selected action payload
coverage.
Steps 7.4.2c.0 and 7.4.2c.1 shipped the B37 protocol split:
`ScopedContinuation` remains the shared wrapper-owned handle,
`ScopedResumeTypes` carries the action value/program associated-type
vocabulary, and the resume methods now live on private wrapper-family
traits for default erased, single-shot Explicit, Rc-shared, and
Arc-shared carriers. The shipped `RunScopedContinuation` moved to
`DefaultScopedResume`, `RunExplicitScopedContinuation` moved to
`ExplicitScopedResume`, and step 7.4.2c.2 added
`RcRunScopedContinuation` on `RcScopedResume` with repeated-resume and
post-action insertion coverage. Step 7.4.2c.3 added
`RcRunExplicitScopedContinuation` on the same Rc-specific private trait
with repeated-resume, post-action insertion, and borrowed action value
coverage. Step 7.4.2d added `ArcRunScopedContinuation` and
`ArcRunExplicitScopedContinuation` on the Arc-specific private trait
with explicit `Send + Sync` obligations, repeated shared resume,
repeated post-action insertion, and borrowed Explicit action value
coverage. Step 7.4.3 added the private `DispatchScopedCarrierHandler`
and `DispatchScopedCarrierHandlers` substrate: around-action handlers
can receive a scoped layer mapped to the carrier's `ActionProgram`, a
typed `ScopedContinuation`, and the inherited first-order handlers,
while ordinary `DispatchScopedHandler` / `DispatchScopedHandlers`
remain unchanged for non-around-action handlers. The failed direct
generic `RcRun` / `RcRunExplicit` prototype remains preserved in
`git stash` as
`wip(effects): rc scoped continuation carrier generic bound prototype`
for historical comparison only; the production Rc carriers now compile
through the family-specific trait split.
Step 7.4.4a added private raw-step extraction and reattach helpers for
`RcFree` and `ArcFree`: `RcFreeRawStep` / `ArcFreeRawStep` keep the
`RcCatList` / `ArcCatList` continuation queues outside the suspended
type-erased branch, and focused tests prove the selected branch only
produces the final result after `continue_from_erased` reattaches that
queue. Default `Run` continues to reuse the existing
`Free::into_raw_step` boundary.
Step 7.4.2 shipped the private carrier vocabulary in
[`interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs):
`ScopedContinuation` is the wrapper-owned handle that forwards normal
resume and post-action continuation insertion to concrete
family-specific carriers without trait objects or unsafe non-`'static`
erasure. `ScopedResumeTypes::ActionValue` and `ActionProgram` exist so
default `Run` can append a `TypeErasedValue -> RawRunFree`
continuation before the erased outer continuation queue is reattached,
rather than wrapping the final resumed `Run` too late. Unit tests
cover normal resume, post-action continuation insertion, and returning
the inner carrier for wrapper-local rewrites across the default,
Explicit, and Rc family traits. Step 7.4.2a added
`RunScopedContinuation` and tests proving that default `Run` resumes a
raw action through an outer continuation and inserts a post-action raw
continuation before that outer continuation. Steps 7.4.2b.0 and
7.4.2b.1 added the typed `RunExplicitScopedContinuation` boundary and
carrier; focused tests prove normal resume, post-action insertion
before the outer continuation, and non-`'static` borrowed action
values. Step 7.4.2c.2 added `RcRunScopedContinuation`; focused tests
prove repeated normal resume and post-action insertion before the
outer continuation. Step 7.4.2c.3 added
`RcRunExplicitScopedContinuation`; focused tests prove repeated normal
resume, post-action insertion before the outer continuation, and
non-`'static` borrowed action values. Step 7.4.2d added the Arc-family
carriers; focused tests prove `Send + Sync` carrier bounds, repeated
shared normal resume, repeated post-action insertion before the outer
continuation, and non-`'static` borrowed Explicit action values. The
B36 H3 fallback is not active. B37 does not require public wrapper API
churn; it changes only the private carrier protocol shape used by the
around-action handler path.
Step 7.3 shipped the standard `BracketDispatcher` and
`RefBracketDispatcher` implementations after step 7.2 verified
standard `LocalDispatcher` and `RefLocalDispatcher` across all six
wrappers. Integration coverage lives in
[`fp-library/tests/run_scoped_dispatchers.rs`](../../../fp-library/tests/run_scoped_dispatchers.rs).
Local / RefLocal dispatch uses scoped-row-preserving Reader
interposition: the dispatcher asks the inherited Reader environment
once, computes the modified environment through `E -> E` or `&E -> E`,
and answers repeated Reader asks inside the action with `E: Clone`.
Default `Run` uses the continuation-aware raw scoped-step path and
reboxes erased action results before ordinary `interpose` walks the
raw body; Rc/Arc and Explicit wrappers use the ordinary scoped
dispatcher path. Arc-family Local dispatch relies on `SendFunctor`
only, matching `SendReaderBrand`'s deliberate lack of ordinary
`Functor`.

The standard step 7 dispatcher set is complete, but B31 inserts a step
7.4 substrate extension before the final Span lifecycle tests. The
first B31 prototype surfaced B32: recursive action interpretation
through borrowed handler lists hits an Explicit-family lifetime wall.
B32 initially adopted substrate-level continuation insertion as the
concrete implementation shape. The later B35 decision promotes that
boundary into an H2 wrapper-owned continuation carrier, so step 7.4
now starts by designing the carrier contract rather than adding a
narrow H1 insertion primitive.
The 2026-05-11 H1 audit surfaced B33: the Explicit Free substrates
currently push `bind` continuations recursively into nested suspended
layers, so they have no pending continuation queue for H1 to splice
unless the Explicit substrates are retrofitted. B33 is resolved via
Option A: first prove a continuation-queue / raw-step retrofit on
`FreeExplicit`, then extend the same shape to `RcFreeExplicit` and
`ArcFreeExplicit` if the proof stays bounded. The `FreeExplicit`
proof surfaced B34: the raw-step shape must keep a hidden intermediate
result type available without `Any`, because `FreeExplicit` exists
specifically to support non-`'static` payloads. B34 is resolved via
Option A: attempt a private existential visitor raw-step proof first.
The proof must remain private to the Explicit substrates, preserve the
public wrapper API, avoid unsafe non-`'static` erasure, and avoid
handler-list-visible carrier state. The first proof audit found B35:
this exact private visitor shape requires a generic callback on a
trait object, which Rust's dyn-compatibility rules do not allow. B35 is
resolved via Option A: promote H2 into the current Phase 4 step 7.4
work and implement an internal continuation carrier instead of adding a
Span-specific primitive. Use that carrier to place Span's post-action
hook before the action's outer continuation queue without requiring
owned / clonable handler lists.
`BracketDispatcher` and
`RefBracketDispatcher` sequence acquire -> body -> effectful release on
the normal path and return the body result after release completes; the
public `bracket` constructors return `B`, while the Val body closure
still returns `(A, B)` internally so the dispatcher can pass `A` to
release. B26 is no longer an open blocker: it adopts normal-path
effectful release plus best-effort panic cleanup through ordinary Rust
`Drop`, without rewriting Bracket release into a synchronous-only
closure. Generic scoped rows remain deferred until a concrete
standard-handler or custom-effect use case requires row type
parameters. B20 remains separate and is retried during step 8's bracket
dispatcher tests. The Bracket / RefBracket lifecycle slice has now
retried `ArcRun::bracket` successfully through a direct `Coproduct`
scoped row; no step 8a escalation is needed for dispatcher coverage.
The non-Bracket step 8 slice fills the missing Rc/Arc Explicit
Catch / Span dispatcher coverage and adds compile-fail coverage for
constructing a scoped operation with `S = CNilBrand`, interpreting a
non-empty Catch scoped row with an empty scoped-handler list, Bracket
body/release closure-shape mismatch, and RefBracket non-refcounted
pointer rejection. Remaining step 8 work is Span tag / nesting
lifecycle coverage; it now follows step 7.4 so public Span handlers can
observe stack-like nested enter/exit ordering through the adopted
continuation-aware scoped-handler path.

Two Phase 3 steps were deferred and may revisit during or after Phase 4: step 6 ([`define_effect!` macro](resolutions.md#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit), revisit when Phase 4 settles the codegen target or a user surfaces concrete demand) and step 5's [`interpret_with_rec`](resolutions.md#resolved-2026-05-04-phase-3-step-5-interpret_with_rec-deferred-indefinitely-option-c) (deferred indefinitely; users chain `interpret_with` then `interpret_rec` for the workaround). Pre-public-release polish work (m1-m9 minor findings from [`remediation_proposals.md`](review/0_first_order_effects_implementation/remediation_proposals.md)) is also outstanding as a non-phased follow-up commit.

### Recent history lookup

Do not duplicate per-step history in this plan. Use git and the append-only history docs instead:

- `git log --oneline -12` for recent commits on the current branch.
- `git show --stat <hash>` for a compact implementation summary.
- `git show --name-only <hash>` for changed files.
- [deviations.md](deviations.md) for per-step implementation choices that diverged from the plan text.
- [resolutions.md](resolutions.md) for blockers, investigations, alternatives, and decisions that paused implementation.

Commit messages carry the full implementation summary for each step. If a detail is load-bearing for future work, preserve it in deviations.md or resolutions.md rather than adding another rolling-history paragraph here.

## Open decisions

> **Maintenance template.** Tracks decisions awaiting user input that affect upcoming steps. Each entry: a heading naming the decision, a one-paragraph context, the proposed options, and trade-offs. Once the user picks an option, fold the chosen path into the relevant phasing section, demote the survey to [resolutions.md](resolutions.md) (or [deviations.md](deviations.md) for smaller-grain choices), and remove the entry from this section.

No open decisions awaiting user input.

## Open questions, issues and blockers

This section tracks **active** blockers only. Resolved blockers
are logged in [resolutions.md](resolutions.md) for design
history. Per-step deviations from the plan are logged in
[deviations.md](deviations.md) for code-review context.

### Active blockers

No active blockers.

### Phase 4 implementation follow-ups and risk status

Closed blockers are tracked in [resolutions.md](resolutions.md) and summarized in [Resolved blockers (summary)](#resolved-blockers-summary). B20's step 8 retry has succeeded for dispatcher coverage: `ArcRun::bracket` is exercised with a direct `Coproduct` scoped row, so the conditional step 8a `SendBracketBrand` redesign is not active.

Q4 is resolved by the step 4 prototype recorded in
[resolutions.md](resolutions.md#resolved-2026-05-08-phase-4-step-4-dispatch_scopedfoh-method-generic-viability-q4-closed-via-option-a).
R3 remains pending as a non-blocking benchmark follow-up alongside
the standard scoped-effect rollout. R1 and R2 were addressed by
shipped Phase 4 interpose work: R1 cleared across
`RunExplicit::interpose`, `RcRunExplicit::interpose`, and
`ArcRunExplicit::interpose` in steps 2.4-2.6; R2 cleared during
`ArcRun::interpose` in step 2.3, with the Arc Explicit-family path
also shipping in step 2.6. No new blocker was opened for either risk.
Non-blocking follow-up: the default `Run` raw scoped dispatch path uses
dispatcher-specific raw-head impls instead of a blanket adapter from
`DispatchScopedHandler`. A blanket impl conflicts with the Box-backed
`CatchDispatcher` specialization under Rust's coherence rules. This is
acceptable for the standard dispatcher rollout because each standard
Box-backed dispatcher can provide its own raw-head impl; revisit the
adapter story when custom Box-backed scoped handlers become a concrete
public use case.

Items B1-B4, Q1-Q3, Q5 are design-resolved; full original framing and
resolution summaries live in
[resolutions.md](resolutions.md#resolved-2026-05-05-phase-4-pre-implementation-design-questions-b1-b4-q1-q3-q5-closed-by-design-adoption-commit-6e960701).
The 2026-05-08 code audit found Q3's implementation prerequisite
missing from the wrappers; B24 is resolved by adopting the
scoped-row-preserving primitive retrofit as Phase 4 step 6a. B25 and
B26 are resolved by the 2026-05-09 adoption of the v1 Local / RefLocal
and Bracket dispatcher semantics. B29 is resolved by the scoped
dispatcher architecture checkpoint and converted into step 7 production
guidance. B31 is resolved by adopting the continuation-aware
around-action handler path and converted into Phase 4 step 7.4. B32 is
resolved by adopting substrate-level continuation insertion as the
concrete implementation shape for that path. B33 is resolved via Option
A: retrofit the Explicit substrates with continuation queues / raw-step
decomposition, starting with a `FreeExplicit` proof and then extending
to `RcFreeExplicit` and `ArcFreeExplicit` if the proof stays bounded.
B34 is resolved via Option A with an explicit H2 fallback: first prove
a private existential visitor raw-step protocol for `FreeExplicit`;
if that protocol cannot stay private and bounded, stop and reopen B32
H2 instead of adding unsafe erasure, dropping six-wrapper parity, or
shipping a Span-specific primitive as the main path. B35 is resolved
via Option A: H2 is now the step 7.4 implementation path. B36 is
resolved via Option A: the typed Explicit carrier keeps the action and
outer continuation separate without unsafe erasure. B37 is resolved via
Option A: the private carrier protocol is split by wrapper family so
Rc / Arc bind obligations live where Rust can check them. B38 is
resolved via Option A for default and erased shared wrappers: add
private carrier raw-step extraction boundaries before wiring the
wrapper interpreters. B39 is resolved via Option B: the B38 Option C
fallback is now active for the Explicit-family path, so scoped-effect
constructors store the runner/carrier boundary before Explicit `bind`
distributes the outer continuation. B40 identified that
carrier-backed scoped effects need to preserve the selected action type
while `Functor::map` changes the final next-program slot. B41 is
resolved via Option D: the action type stays out of the `'static` row
brand and moves into a minimal two-slot around-action protocol for
carrier-backed handlers. B42 is resolved via Option B first, with
Option C as a fallback if the `RunExplicit` Span proof shows that
Explicit mapping still requires an action-plus-outer carrier cell in
the scoped layer. B43 executed that proof gate and activated the
Option C fallback. Step 7.4.4b.2b added the private
`RunExplicitSpanCarrierLayer`; step 7.4.4b.2c added the focused
`RunExplicit` Span dispatcher proof. The next implementation step
extends the carrier-cell proof to `RcRunExplicit` and
`ArcRunExplicit`. The only remaining pending non-blocking risk item
here is R3.

#### R3. Scoped-operation allocation cost (pending benchmark follow-up)

**Risk.** Bracket on `RcRun` allocates 3 closure cells (acquire is a Run; body and release are `Rc<dyn Fn>`); plus the `BracketGuard`. No benchmarks exist. The plan's performance characterisation is implicit ("amortised over Coyoneda fusion") but scoped operations don't go through Coyoneda.

**Mitigation:** Add a Phase 4 benchmark commit alongside the standard scoped-effect rollout (sequencing item 7), paralleling the [Phase 1 step 8 per-variant Free benches](#phase-1-complete-the-free-family). Compare scoped-op cost against equivalent FO-only programs that simulate the scoped behaviour through closure capture.

### Open follow-ups (not blocking but worth surfacing)

R3 is tracked in
[Phase 4 implementation follow-ups and risk status](#phase-4-implementation-follow-ups-and-risk-status).
No other open follow-ups are currently tracked here. Resolved history
lives in [resolutions.md](resolutions.md) and in
[Resolved blockers (summary)](#resolved-blockers-summary).

### Procedure for new blockers

If a load-bearing question surfaces during implementation:

1. Add an `### Active blocker (date): <summary>` subsection
   under `### Active blockers` above and pause work.
2. When the blocker resolves, move the entry verbatim (or with
   added resolution detail) to [resolutions.md](resolutions.md)
   as a new top-level entry, dated.
3. Remove the active-blocker subsection.

### Resolved blockers (summary)

For full investigation, alternatives, and rationale on each
resolved blocker, see [resolutions.md](resolutions.md). One-line
summaries:

- [Resolved (2026-05-12): B43 RunExplicit Span proof selects carrier-cell fallback](resolutions.md#resolved-2026-05-12-b43-runexplicit-span-proof-selects-carrier-cell-fallback)
  : B43 records the 7.4.4b.2a proof-gate result. Current
  `RunExplicit::bind` maps the `BoxSpan` action slot to the final
  program before interpretation, so the outer-only continuation model
  cannot recover the selected action. Steps 7.4.4b.2b-2c added the B42
  Option C carrier-cell layer shape and focused `RunExplicit`
  dispatcher proof; the next step extends the proof to shared Explicit
  wrappers.
- [Resolved (2026-05-12): B42 carrier-aware handler protocol duplicates selected actions](resolutions.md#resolved-2026-05-12-b42-carrier-aware-handler-protocol-duplicates-selected-actions)
  : B42 closed via Option B first. The scoped operation owns the
  selected action, the continuation carrier owns only the outer resume
  boundary, and 7.4.4b.2 starts with a narrow `RunExplicit` Span proof.
  Option C remains the fallback if Explicit mapping requires storing a
  carrier cell inside carrier-backed scoped layers.
- [Resolved (2026-05-12): B41 action-indexed carrier brands conflict with static row-brand bounds](resolutions.md#resolved-2026-05-12-b41-action-indexed-carrier-brands-conflict-with-static-row-brand-bounds)
  : B41 closed via Option D. Phase 4 step 7.4.4b now starts by
  defining a minimal two-slot around-action protocol for Span so the
  selected action program/value remains lifetime-indexed, row brands
  remain `'static`, and ordinary one-slot `DispatchScopedHandlers`
  keep their existing route.
- [Resolved (2026-05-12): B40 constructor-stored carriers need an action-type home across Functor map](resolutions.md#resolved-2026-05-12-b40-constructor-stored-carriers-need-an-action-type-home-across-functor-map)
  : B40 found the need to preserve a selected action type while the
  final-program slot changes under `Functor::map`. Its
  action-result-indexed row-brand prototype was superseded by B41
  before implementation because the prototype conflicted with
  non-`'static` Explicit payload support.
- [Resolved (2026-05-12): B39 Explicit-family extraction cannot recover action-outer split from recursively mapped binds](resolutions.md#resolved-2026-05-12-b39-explicit-family-extraction-cannot-recover-action-outer-split-from-recursively-mapped-binds)
  : B39 closed via Option B. Phase 4 step 7.4.4 now activates the B38
  Option C fallback for the Explicit-family path: scoped-effect
  constructors preserve a runner/carrier boundary before Explicit
  `bind` distributes the outer continuation, and wrapper interpreters
  consume that stored shape instead of reconstructing it from the
  substrate.
- [Resolved (2026-05-12): B38 wrapper interpreters need carrier extraction boundaries](resolutions.md#resolved-2026-05-12-b38-wrapper-interpreters-need-carrier-extraction-boundaries)
  : B38 closed via Option A. Phase 4 step 7.4.4 now starts by adding
  private carrier raw-step extraction boundaries for default and
  erased shared wrappers. B39 later activated the constructor-stored
  runner/carrier fallback for the Explicit-family path before the full
  six-wrapper `DispatchScopedCarrierHandlers` wiring proceeds.
- [Resolved (2026-05-12): B37 Rc-family carriers need wrapper-specific bind bounds](resolutions.md#resolved-2026-05-12-b37-rc-family-carriers-need-wrapper-specific-bind-bounds)
  : B37 closed via Option A. Phase 4 step 7.4.2c now starts by
  splitting the private carrier protocol into wrapper-family resume
  traits for default erased, single-shot Explicit, Rc-shared, and
  Arc-shared carriers while keeping `ScopedContinuation` as the shared
  handle. Rc and Arc projection bounds live on their family-specific
  private traits instead of a common under-bounded contract.
- [Resolved (2026-05-11): B36 `RunExplicit` H2 carrier needs an existential-safe continuation boundary](resolutions.md#resolved-2026-05-11-b36-runexplicit-h2-carrier-needs-an-existential-safe-continuation-boundary)
  : B36 closed via Option A. Phase 4 step 7.4.2b proved a typed
  `RunExplicitScopedContinuation` carrier that separates the action
  from its outer continuation; the H3 protocol-family fallback did not
  trigger.
- [Resolved (2026-05-11): B35 private visitor proof requires a generic callback on an existential bind node](resolutions.md#resolved-2026-05-11-b35-private-visitor-proof-requires-a-generic-callback-on-an-existential-bind-node)
  : B35 closed via Option A. Phase 4 step 7.4 promotes B32 H2 into the
  active implementation path: wrappers own an internal continuation
  carrier that can resume normally or insert result-preserving
  post-action behavior before outer continuations.
- [Resolved (2026-05-11): B34 FreeExplicit raw steps need a non-static existential continuation boundary](resolutions.md#resolved-2026-05-11-b34-freeexplicit-raw-steps-need-a-non-static-existential-continuation-boundary)
  : B34 originally closed via Option A with an explicit H2 fallback.
  B35 then promoted that fallback into the active plan after the
  private visitor proof hit Rust's trait-object generic-callback wall.
  Phase 4 step 7.4.2b now starts with the B36 Explicit-boundary
  prototype and proves `RunExplicit` through the H2 carrier only if
  that private boundary succeeds.
- [Resolved (2026-05-11): B33 Explicit substrates lack a continuation queue for H1 insertion](resolutions.md#resolved-2026-05-11-b33-explicit-substrates-lack-a-continuation-queue-for-h1-insertion)
  : B33 closed via Option A. Phase 4 retrofits the Explicit Free
  substrates with continuation queues / raw-step decomposition,
  starting with a `FreeExplicit` proof before extending to
  `RcFreeExplicit` and `ArcFreeExplicit`.
- [Resolved (2026-05-10): B32 Explicit-wrapper continuation-aware action runner lifetime wall](resolutions.md#resolved-2026-05-10-b32-explicit-wrapper-continuation-aware-action-runner-lifetime-wall)
  : B32 closed via Option D / H1. Phase 4 implements B31's
  around-action path by adding substrate-level continuation insertion
  first, avoiding recursive action interpretation through borrowed
  handler lists and preserving six-wrapper parity. H2 and H3 are
  deferred to Phase 6+ revisit criteria.
- [Resolved (2026-05-10): B31 Span nested lifecycle exit ordering under the public scoped-handler shape](resolutions.md#resolved-2026-05-10-b31-span-nested-lifecycle-exit-ordering-under-the-public-scoped-handler-shape)
  : B31 closed via Option B. Phase 4 adds a continuation-aware
  scoped-handler path for around-action handlers before completing
  Span lifecycle tests, so public Span handlers can observe stack-like
  nested enter/exit ordering instead of only action-result propagation.
- [Resolved (2026-05-09): B29 scoped dispatcher architecture checkpoint](resolutions.md#resolved-2026-05-09-b29-scoped-dispatcher-architecture-checkpoint)
  : B29 closed via Option A plus Option B surface polish. Production
  scoped dispatch uses each wrapper's actual peeled-layer lifetime;
  interpose-backed dispatchers carry row-removal evidence; helper
  constructors hide witness-bearing dispatcher type names where
  practical; `SpanDispatcher` stays witness-free; Bracket dispatchers
  remain result-specific.
- [Resolved (2026-05-09): B30 Box-backed CatchDispatcher single-shot continuation](resolutions.md#resolved-2026-05-09-b30-box-backed-catchdispatcher-single-shot-continuation)
  : B30 closed via Option C. Default `Run` uses a raw scoped-step path
  that keeps the erased `Free` continuation queue outside Box-backed
  `Catch` until the active branch is known; `RunExplicit` and Rc/Arc
  wrappers use the ordinary dispatcher/interpose paths.
- [Resolved (2026-05-09): Phase 4 step 6a / 7 scoped-row primitive and dispatcher semantics; B24 / B25 / B26](resolutions.md#resolved-2026-05-09-phase-4-step-6a--7-scoped-row-primitive-and-dispatcher-semantics-b24--b25--b26)
  : B24 closed via Option A: add the scoped-row-preserving primitive
  retrofit before standard scoped dispatchers. B25 closed via Option A:
  implement Local / RefLocal by Reader interposition, with `E: Clone`
  where the current by-value Reader needs repeated environment values,
  and defer borrow-oriented Reader for true no-clone repeated asks. B26
  closed via Option A: keep normal-path release effectful and document
  panic cleanup as ordinary resource `Drop`, with any synchronous
  panic-finalizer hook deferred.
- [Resolved (2026-05-09): B27 `interpret_with_either` scoped-suspension return path](resolutions.md#resolved-2026-05-09-b27-interpret_with_either-scoped-suspension-return-path)
  : keep `interpret_with_either` scoped-row-empty / first-order-only,
  build `CatchDispatcher` on scoped-row-preserving `interpose`, and
  defer a richer scoped-aware short-circuit carrier until a second use
  case appears.
- [Resolved (2026-05-08): Phase 4 step 4 `dispatch_scoped<FOH>` method-generic viability; Q4 closed via Option A](resolutions.md#resolved-2026-05-08-phase-4-step-4-dispatch_scopedfoh-method-generic-viability-q4-closed-via-option-a)
  : Q4 closed via Option A (validate by prototype before dispatcher
  implementation): a local `RcRun` `Span` scoped-handler prototype
  compiles with a method-generic scoped dispatch method and invokes a
  real `handlers!` first-order cons-list through the existing
  `DispatchHandlers` bound, including tail recursion through a
  two-effect row. The production trait spells this as argument-position
  `impl DispatchHandlers`, preserving static dispatch without a named
  `FOH` parameter.
- [Resolved (2026-05-08): Phase 4 step 5b `define_scoped_row!` item macro adopted; B23 closed via Option B](resolutions.md#resolved-2026-05-08-phase-4-step-5b-define_scoped_row-item-macro-adopted-b23-closed-via-option-b)
  : B23 closed via Option B: keep `scoped_effects![...]` as the
  type-position row macro and add a separate `define_scoped_row!`
  item-position macro for named marker rows. The v1 macro is concrete
  only, supports a `Self` placeholder for recursive row references, and
  defers generic row support until a concrete need appears.
- [Resolved (2026-05-08): Phase 4 step 3.4 Span tag storage and clone/send bounds; B22 closed via Option A](resolutions.md#resolved-2026-05-08-phase-4-step-34-span-tag-storage-and-clonesend-bounds-b22-closed-via-option-a)
  : B22 closed via Option A (keep by-value tags and add bounds only
  where required): default Box-backed Span cells keep non-`Clone` tags
  available, Rc paths require `Tag: Clone` where cells must clone, and
  Arc paths require `Tag: Clone + Send + Sync` where cells must be
  cloneable and thread-safe.
- [Resolved (2026-05-08): Phase 4 step 3.4 Span action storage versus no-pointer-brand shorthand; B21 closed via Option A](resolutions.md#resolved-2026-05-08-phase-4-step-34-span-action-storage-versus-no-pointer-brand-shorthand-b21-closed-via-option-a)
  : B21 closed via Option A (mirror Catch and Local at the substrate
  level): Span remains Val-only and has no Ref dispatch split at the
  public API, while the implementation uses Box/Rc/Arc sibling cells
  and unit-argument B-thunks for the action program to avoid recursive
  layout cycles.
- [Resolved (2026-05-07): Phase 4 step 3.2 sub-step splitting + Local action layout cycle reuse + file organization; B8 + B9 + B10 closed](resolutions.md#resolved-2026-05-07-phase-4-step-3.2-sub-step-splitting--local-action-layout-cycle-reuse--file-organization-b8--b9--b10-closed)
  : B8 closed via Option B (8-commit Val + Ref split: 3.2.1-3.2.4 Local Val cycle / 3.2.5-3.2.8 RefLocal Ref cycle); B9 closed via Option A (apply B-thunk uniformly to Local mirroring catch.rs); B10 closed via Option A (separate `local.rs` and `ref_local.rs` files).
- [Resolved (2026-05-07): Phase 4 step 3.1.3 Catch action-field layout cycle (B7) closed](resolutions.md#resolved-2026-05-07-phase-4-step-3.1.3-catch-action-field-layout-cycle-b7-closed)
  : B7 closed via B-thunk with unit-arg `Fn(()) -> A` form: per-pointer-brand pointer of unit-arg closure thunk leveraging the existing pointer-abstraction `ToDyn*Fn::new` family per [`pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md)'s `(pointer-capability, closure-semantic)` matrix.
- [Resolved (2026-05-07): Phase 4 step 3.1 sub-step splitting; B5 + B6 closed](resolutions.md#resolved-2026-05-07-phase-4-step-3.1-sub-step-splitting-b5--b6-closed)
  : B5 closed via Option A (brand-projection helpers escape the
  trait impl's HRTB-bearing scope so the GAT projection normalizes
  against the concrete `BoxCatch` / `Catch` enum, mirroring
  `arc_run::unwrap_first`); B6 closed via Option B (4-commit split
  into 3.1.1 foundational scaffold / 3.1.2 `RefFunctor` + helpers /
  3.1.3 smart constructors / 3.1.4 integration tests).
- [Resolved (2026-05-04): Phase 3 step 5 (`interpret_with_rec`) deferred indefinitely (option (c))](resolutions.md#resolved-2026-05-04-phase-3-step-5-interpret_with_rec-deferred-indefinitely-option-c)
  : pipeline row-narrowing combined with `MonadRec`-target stack
  safety doesn't compose cleanly. Multi-inner row layers in the
  unmatched arm need `Traversable` on the row brand plus
  `Applicative` on `M` to swap `RMinusE::Of<M::Of<...>>` to
  `M::Of<RMinusE::Of<...>>`; non-rec `interpret_with` sidesteps
  this with plain `Functor::map`. PureScript Run skips the
  combination too. Phase 3 ships three interpreter primitives,
  not four; users chain `interpret_with` (narrow) then
  `interpret_rec` (stack-safe) for the workaround.
- [Resolved (2026-05-04): Phase 3 step 6a downstream blocker; `ArcCoyoneda`'s algebra migrated to `SendFunctor` (option (a))](resolutions.md#resolved-2026-05-04-phase-3-step-6a-downstream-blocker-arccoyonedas-algebra-migrated-to-sendfunctor-option-a)
  : the
  [2026-05-03 option-(c) `SendStateBrand` resolution](resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified)
  surfaced a downstream gap, the
  [`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
  dispatch path required `EBrand: Functor + SendFunctor`, but
  `SendStateBrand` cannot honestly implement `Functor`.
  Migrated `ArcCoyoneda`'s inner trait, three layer impls, and
  public methods from `F: Functor` to `F: SendFunctor` (option
  (a), mirroring Phase 2 step 9d's `ArcFree` migration); added
  `SendFunctor` impl for `VecBrand`; dropped `+ Functor` from
  the dispatch impl's `EBrand` bound. Brand-level `Foldable`
  on `ArcCoyonedaBrand` was dropped (Rust forbids tightening
  trait method bounds in impls); a
  [`SendFoldable` follow-up commit](deviations.md#step-5a4--5a6-second-follow-up-2026-05-04-sendfoldable-trait--brand-level-fold-restored-on-arccoyonedabrand)
  introduced the Send-aware parallel trait and restored the
  brand-level fold surface.
- [Resolved (2026-05-03): Phase 3 step 6a SendFunctor reopened after option (b) unimplementable; option (c) parallel `SendStateBrand` ratified](resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-reopened-after-option-b-unimplementable-option-c-parallel-sendstatebrand-ratified)
  : the original (b) ratification (`4bd1636`) was discovered
  unimplementable , `Arc<dyn Fn(...)>` is structurally
  `!Send + !Sync` because the trait object's bounds don't
  include `Send + Sync`, and use-site bounds can't change a
  structural type-level fact. Reopened and re-ratified with
  option (c): a parallel
  [`SendStateBrand<P, S>`](../../../fp-library/src/brands.rs)
  / `SendState<'a, P, S, A>` type whose variants store
  `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> A + Send + Sync>`,
  sidestepping the structural problem. Brand-level
  `SendFunctor` impl on `SendStateBrand<P, S>` is
  implementable because the projection is structurally
  `Send + Sync`. Non-Arc smart constructors keep using
  `StateBrand`; Arc smart constructors use `SendStateBrand`.
- [Resolved (2026-05-03): Phase 3 step 6a `SendFunctor` impl on `StateBrand` for the Arc family (option (b) per-method bounds)](resolutions.md#resolved-2026-05-03-phase-3-step-6a-sendfunctor-impl-on-statebrand-for-the-arc-family-option-b-per-method-bounds)
  (superseded): option (b) per-method `Send + Sync` bounds at
  smart-constructor sites was the original ratification. The
  bound was discovered unsatisfiable during implementation
  because `Arc<dyn Fn(...)>: Send + Sync` is structurally
  false. Superseded by the option (c) ratification above; this
  entry stays as historical record.
- [Resolved (2026-05-03): smart-constructor wrapper parameterization for the standard first-order effects step](resolutions.md#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization)
  : five sub-decisions confirmed: (1.b) six per-wrapper variants
  per effect, with the Phase 3 step 7 `define_effect!` macro
  hiding verbosity user-side; (2.a) per-effect Functor instance
  (matches PureScript directly); (3.a-1) effect types
  parameterised by `FnBrand` for substrate-agnostic continuation
  representation; (4.ii) Choose ships on all four multi-shot
  wrappers (RcRun / RcRunExplicit / ArcRun / ArcRunExplicit),
  not RcRun-only as the prior plan text said; (5.b) row-brand
  composition via the existing `effects!` macro initially; per-
  effect aliases deferred until user demand surfaces.
- [Resolved (2026-05-02): Phase 3 step 4 interpreter design (handler shape, dispatch-trait reuse, state threading)](resolutions.md#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading)
  : three questions confirmed (Q1 = A) mirror PureScript handler
  shape (`Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Run<R, S, A>>>) -> M::Of<'_, Run<R, S, A>>`),
  reusing the existing `DispatchHandlers` trait with
  `NextProgram = M::Of<Run<R, S, A>>`; (Q2 = A1) relax
  `DispatchHandlers::dispatch` from `&mut self` to `&self` (and
  `Handler::F: Fn`) as the first commit in step 4; (Q3 = A)
  continue closure-capture state threading in `run_accum_rec`,
  parity with step 2.
- [Resolved (2026-04-29): Phase 3 step 2/3 interpreter family shape](resolutions.md#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape)
  -- five decisions confirmed: (1.A) ship row-narrowing
  pipeline + `extract`; (2.C) keep step 2's M-free shape and
  add MonadRec extract as a sibling; (3.A) renumber Phase 3
  to insert pipeline at position 3; (4.A) defer
  `interpret_with<M: Monad>` / `runCont` / `interpose` /
  algebraic-FO to Phase 6+; (5.A) keep decisions.md frozen.
  Resolution distinguishes three orthogonal cognitive models
  (simple value extract, pipeline row-narrowing, MonadRec
  external target) and ships all three under axis 1 widening.
- [Resolved (2026-04-28 implementation expansion): step 9 SendFunctor cascade prerequisites for Arc family](resolutions.md#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family)
  -- discovered while implementing the original 2026-04-28
  resolution: `ArcRun::lift` and `ArcRunExplicit::lift` cannot
  use the same `Coyoneda::lift` chain as the other four wrappers
  because `Coyoneda` isn't `Send + Sync` and the Send-aware
  sibling `ArcCoyonedaBrand` doesn't implement `Functor` (a
  deliberate fp-library design choice; the `Functor` trait's
  `map` signature lacks `Send + Sync` bounds on closures).
  Resolution: expand step 9 with sub-steps 9a-9g landing the
  `SendFunctor` cascade prerequisites (replace `F: Functor` with
  `F: SendFunctor` on `ArcFree`/`ArcFreeExplicit`; add missing
  `SendFunctor` impls on the row-cascade brands; expand
  brand-level coverage on `ArcFreeExplicitBrand` /
  `ArcRunExplicitBrand`); then complete `*Run::lift` for all six
  wrappers in 9h; add `SendRefFunctor` on `ArcRunExplicitBrand`
  via inherent-method delegation in 9i.
- [Resolved (2026-04-28): Phase 2 step 9 scope is under-specified](resolutions.md#resolved-2026-04-28-phase-2-step-9-scope-is-under-specified)
  -- generic combinator interpretation locked in, named `lift`
  to match PureScript Run's
  [`Run.lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs);
  inherent associated function on each of the six Run wrappers
  mirroring `*Run::send`'s shape; takes the raw effect (not
  pre-lifted Coyoneda) and does Coyoneda lift -> row inject ->
  `Node::First` -> `*Run::send` inline; falls back to a free
  `lift_node` helper for `ArcRun::lift` only if HRTB-poisoning
  recurs. Followed up by the implementation-expansion entry
  above when the Arc-family Coyoneda/Functor conflict surfaced.
- [Resolved (2026-04-27): `*Run::send` takes a `Node`-projection value to sidestep GAT-normalization poisoning under `ArcFree`'s HRTB](resolutions.md#resolved-2026-04-27-runsend-takes-a-node-projection-value-to-sidestep-gat-normalization-poisoning-under-arcfrees-hrtb)
  -- discovered while implementing `ArcRun::send`: the HRTB at
  `ArcFree`'s struct level poisons `<NodeBrand as Kind>::Of<...>`
  normalization in any scope mentioning it. Workaround: pass
  the `Node`-projection value as a parameter rather than
  constructing it inside the HRTB scope. Applied symmetrically
  to all six Run wrappers' `send` for API uniformity.
- [Resolved (2026-04-27): brand-level type-class coverage gap on the Explicit Run brands](resolutions.md#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands)
  -- ship `Functor / Pointed / Semimonad` plus the by-reference
  equivalents for `RunExplicitBrand`; `Pointed` plus by-reference
  for `RcRunExplicitBrand`; `SendPointed` only for
  `ArcRunExplicitBrand`. `Monad` / `RefMonad` / `SendMonad` and
  the `SendRef`-family hierarchy are unreachable through
  brand-level delegation; inherent `bind` and `map` cover the
  by-value monadic surface.
- [Resolved (2026-04-27): row-brand `RefFunctor` and `Extract` cascade impls land in step 4b](resolutions.md#resolved-2026-04-27-row-brand-reffunctor-and-extract-cascade-impls-land-in-step-4b)
  -- add `RefFunctor` and `Extract` impls to `CNilBrand`,
  `CoproductBrand<H, T>`, and `NodeBrand<R, S>`, plus a `Clone`
  impl on `Node`. Required by the Run-Explicit brand
  Ref-hierarchy delegation and by `Rc`/`Arc`-Free's `evaluate`
  fallback.
- [Resolved (2026-04-27): re-export pattern for the effects subsystem types follows the optics A+B hybrid](resolutions.md#resolved-2026-04-27-re-export-pattern-for-the-effects-subsystem-types-follows-the-optics-ab-hybrid)
  -- six Run wrapper headline types at top-level
  (`crate::types::*`); same six plus `Node` and `VariantF` at
  subsystem-scoped (`crate::types::effects::*`). Mirrors the
  optics precedent.
- [Resolved (2026-04-27): introduce `WrapDrop` trait for Free's struct-level Drop concern](resolutions.md#resolved-2026-04-27-introduce-wrapdrop-trait-for-frees-struct-level-drop-concern)
  -- replace Free's struct-level `Extract` bound with a new
  `WrapDrop` trait that decouples Drop's structural cleanup
  from `Extract`'s semantic interpretation. Two-commit migration
  before Phase 2 step 4 resumes.
- [Resolved (2026-04-26): brand-level dispatch for the multi-shot Explicit Free family lands on the by-reference hierarchy](resolutions.md#resolved-2026-04-26-brand-level-dispatch-for-the-multi-shot-explicit-free-family-lands-on-the-by-reference-hierarchy)
  -- `RcFreeExplicitBrand` and `ArcFreeExplicitBrand` get
  `Pointed`/`SendPointed` on the by-value side and full Ref/SendRef
  hierarchies; remaining by-value operations ship as inherent
  methods.
- [Resolved earlier: Erased / Explicit dispatch split for the Free family](resolutions.md#resolved-earlier-erased--explicit-dispatch-split-for-the-free-family)
  -- Erased family (`Free`, `RcFree`, `ArcFree`) is
  inherent-method only; Explicit family (`FreeExplicit`,
  `RcFreeExplicit`, `ArcFreeExplicit`) is Brand-dispatched.
- [Design-phase blockers (resolved in decisions.md)](resolutions.md#design-phase-blockers-resolved-in-decisionsmd)
  -- pointer aggregating decisions.md sections 4 and 9.

<!-- The full problem statement, investigation, resolution,
migration plan, and alternatives for the WrapDrop blocker live
in resolutions.md per the link above. The phasing-side checklist
lives in "Phase 1 follow-up: WrapDrop migration" below. -->

<!-- old content removed; see resolutions.md -->

## Deviations

Per-step deviations from the original plan text (where the
shipped code or design diverged from what the step description
said) are logged in [deviations.md](deviations.md), grouped by
phase and step. New deviations are appended there as steps land.

## Implementation protocol

After completing each step within a phase:

1. Run verification: `just fmt`, `just check`, `just clippy`,
   `just deny`, `just doc`, `just test` (or `just verify` which
   runs all six in order).
2. If verification passes, update `Current progress`, `Open
questions, issues and blockers`, `Open decisions` (if a
   decision lands or is newly surfaced), and `Deviations`
   sections at the top of this plan to reflect the current
   state.
3. **Refresh `Current progress` per the canonical template.**
   The section has three required subsections, in this order:
   1. **`### Phase status`** holds one short paragraph per phase
      summarising current state. Edit in place; do not append
      new prose.
   2. **`### Next greenfield work`** holds a 1-3 paragraph
      description of the next step, including the example
      syntax / shape if relevant and a cross-link to
      `Open decisions` if a sub-step split is awaiting user
      input.
   3. **`### Recent history lookup`** holds stable instructions
      for using `git log`, `git show`, deviations.md,
      resolutions.md, and commit messages to recover shipped-step
      history. Do not add per-step narratives or commit-log
      bullets here.

   **Anti-pattern (do not do this):** appending new prose to
   the Phase status paragraph each time a step ships, growing
   the intro into a multi-paragraph blob. The Phase status
   block must remain a tight summary; per-step detail belongs
   in commit messages, deviations.md, or resolutions.md.

   Before deleting any stale plan prose, verify its load-bearing
   context lives somewhere persistent: design choices in
   [deviations.md](deviations.md), load-bearing
   investigations in [resolutions.md](resolutions.md),
   "what changed" in the commit message. If a piece of
   context lives only in plan.md, move it to the right home
   first.

   Goal: keep `Current progress` under ~250 lines so a new
   agent reading the plan reaches actionable content quickly.
   Detailed history stays accessible via `git show <hash>`,
   deviations.md, and resolutions.md.

   Do not mirror live progress into [prompt.md](prompt.md).
   prompt.md is a durable handoff entrypoint and should change only
   when the resume workflow, durable lessons, or operational gotchas
   change.

4. Commit the step (including the plan updates and any inline
   trim).

---

Port `purescript-run`'s extensible algebraic effects to
`fp-library`, delivering Rust `Run` types that support
row-polymorphic first-order effects and heftia-style scoped
effects, with macro ergonomics for common cases and a six-variant
`Free` substrate covering single-shot, multi-shot, thread-safe,
non-`'static` payload, and Brand-dispatched-vs-inherent-method
combinations via the Erased/Explicit dispatch split.

## API stability stance

`fp-library` is pre-1.0. API-breaking changes are acceptable when
they lead to a better end state. This plan prioritises design
correctness and internal coherence over preserving compatibility
with any pre-existing user surface for `Run` (there is none yet;
this is an additive port).

## Motivation

PureScript's `purescript-run` ships an extensible algebraic-effect
system shaped around row polymorphism, partial interpretation, and
multi-shot continuations. fp-library has the building blocks
(`Free<F, A>`, `Coyoneda<F>`, the Brand-and-Kind HKT machinery, and
the `MonadRec` interpreter family) but no public `Run` type. This
plan delivers `Run` and the surrounding effect machinery, ported to
match PureScript's user-facing semantics where stable Rust permits
and explicitly diverging where it doesn't (e.g., `pure` takes a
brand turbofish; multi-shot effects require choosing `RcRun` or
`ArcRun` rather than the default `Run`; typeclass-generic dispatch
requires the corresponding Explicit Run variant).

User surface after this plan, fast-path inherent-method version:

```rust
// Declare a row of effects via the macro:
type AppEffects = effects![Reader<Env>, State<Counter>, Logger];

// Build a program with the im_do! macro (inherent monadic do,
// inherent-method-based, O(1) bind, no Brand dispatch):
fn run_program() -> Run<AppEffects, NoScoped, String> {
    im_do! {
        cfg <- ask::<Env>();
        n <- get::<Counter>();
        log(format!("config = {cfg:?}, counter = {n}"));
        pure(format!("got {n}"))
    }
}

// Compose handlers as a pipeline that narrows the row at each step:
let result: String = run_program()
    .handle(run_reader(env))
    .handle(run_state(0))
    .handle(run_logger())
    .extract();
```

For Brand-dispatched typeclass-generic code (or programs with
non-`'static` payloads), use the corresponding Explicit variant.
The single-shot single-thread variant `RunExplicit` (built on
`FreeExplicit`) keeps full by-value brand coverage and is the
ergonomic default:

```rust
fn run_program_explicit<'a>() -> RunExplicit<'a, AppEffects, NoScoped, String> {
    m_do!(RunExplicitBrand {
        cfg <- ask::<Env>();
        n <- get::<Counter>();
        pure(format!("got {n}"))
    })
}
```

The multi-shot variants `RcRunExplicit` / `ArcRunExplicit` get
brand dispatch via the by-reference hierarchy (`RefFunctor` /
`RefSemimonad` / `RefMonad` and their `SendRef*` parallels),
matching `Lazy`'s precedent for the same constraint. The existing
`m_do!` / `a_do!` macros support a `ref` qualifier
(`m_do!(ref Brand { ... })`) that routes through
`RefSemimonad::ref_bind`; closures take `&A`:

```rust
fn run_program_rc_explicit<'a>() -> RcRunExplicit<'a, AppEffects, NoScoped, String> {
    m_do!(ref RcRunExplicitBrand {
        cfg <- ask::<Env>();          // cfg: &Env
        n <- get::<Counter>();         // n: &Counter
        pure(format!("got {n}"))
    })
}
```

For inherent-method calls on multi-shot Explicit Run programs
(e.g., when `A: Clone` is satisfied and consuming continuations
are preferred), the by-value `bind` / `map` ship as inherent
methods on `RcRunExplicit` / `ArcRunExplicit` directly, with their
natural `Clone` bounds, mirroring the
[`RcCoyoneda`/`ArcCoyoneda` precedent](../../../fp-library/docs/limitations-and-workarounds.md).

Convert between Erased and Explicit on demand:
`run_program().into_explicit()` walks the structure once and
returns the corresponding Explicit Run of the same program,
suitable for handing into typeclass-generic consumers.

`runReader: Run<R + READER, S, A> -> Run<R, S, A>`-style row
narrowing matches PureScript Run (the scoped-effect row `S`
threads unchanged through first-order handlers and is narrowed
only by scoped-effect handlers); the macro layer plus
`CoproductSubsetter`-mediated permutation proofs handle the
ordering-mitigation problem (see
[decisions.md](decisions.md) section 4.1).

## Design

The design is recorded in full in
[decisions.md](decisions.md) sections 4 (six core DECISIONs) and
5 (draft architecture). Quick reference:

- **Row encoding (decisions Section 4.1):** Option 4 hybrid (frunk-style
  Peano-indexed `Coproduct<H, T>` plus `effects![...]` macro
  layer). Workaround 1 (macro lexical sort) is primary; workaround
  3 (`CoproductSubsetter` permutation proof) is fallback for
  hand-written rows.
- **Functor dictionary (decisions Section 4.2):** static option via
  `Coyoneda` per effect. Each row variant is `Coyoneda<E, A>`,
  which is a `Functor` for any `E` regardless of `E`'s own shape.
  `Coproduct<H, T>` implements `Functor` via recursive trait
  dispatch (`H: Functor + T: Functor`). The dynamic
  `DynFunctor` option is retained as a fallback only.
- **Stack safety (decisions Section 4.3):** ship both interpreter
  families, mirroring PureScript: `interpret` / `run` (assume
  target stack-safe) and `interpretRec` / `runRec` (require
  `MonadRec` on target). Plus the Rust-specific row-narrowing
  pair `interpret_with` / `interpret_with_rec` per the
  [2026-04-29 widening resolution](resolutions.md#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape)
  and the
  [2026-05-03 reversal](resolutions.md#resolved-2026-05-03-adversarial-review-reversals-delete-run_accum-ship-interpret_with_rec-parameterise-interpret_with-over-refcountedpointer).
  No `run_accum` / `runAccum` companion: state threading is via
  user-side closure captures; PureScript Run's `runAccum` shape
  drops the threaded state on return anyway, and the mono-in-A
  Rust port has no slot for a tupled state result.
- **Free family (decisions Section 4.4):** six variants in two rows.
  Erased family (`Free`, `RcFree`, `ArcFree`) is inherent-method
  only with O(1) bind via `dyn Any` erasure plus CatList; pins
  `A: 'static`. Explicit family (`FreeExplicit<'a, ...>`,
  `RcFreeExplicit<'a, ...>`, `ArcFreeExplicit<'a, ...>`) carries
  Brand dispatch with O(N) bind via concrete recursive enum;
  supports `A: 'a`. The Erased/Explicit split is the dispatch
  story: typeclass-generic code uses the Explicit row, fast-path
  code uses the Erased row, and `into_explicit()` converts between
  them when needed. The `ArcFreeExplicitBrand` `Functor` impl
  lands via the new `SendFunctor` trait family (Phase 1 step 6).
- **Scoped effects (decisions Section 4.5):** heftia-style dual-row
  architecture. `Run` carries a separate higher-order row of
  scoped-effect constructors (`Catch<'a, E>`, `Local<'a, E>`,
  `Bracket<'a, A, B>`, `Span<'a, Tag>`). Day-one `'a` parameter,
  fixed `Run<R, A>` continuation, coproduct-of-constructors
  extension shape. (A `Mask<'a, E>` constructor for duplicated-effect
  masking was considered and deferred to a future revision; see
  [decisions.md](decisions.md) section 4.5's "Deferred to a future
  revision" sub-decision for the four options preserved on the
  shelf.)
- **Natural transformations (decisions Section 4.6):** `handlers!{...}`
  macro DSL primary, builder pattern (`nt().on::<E>(handler)...`)
  as fallback.

`Run` core type:

```text
Run<Effects, ScopedEffects, A> = FreeFamily<Node<Effects, ScopedEffects>, A>

Node<Effects, ScopedEffects>   = First(VariantF<Effects>)
                               | Scoped(ScopedCoproduct<ScopedEffects>)
```

where `FreeFamily` is one of `Free` / `RcFree` / `ArcFree` /
`FreeExplicit`, and `Run<R, S, A>` has matching aliases (`RcRun`,
`ArcRun`, `RunExplicit`).

## Validated via POCs

| POC                                                                                                                                                                         | Findings                                                                                                                                                                                                                                                                                                                                                   |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `poc-effect-row/tests/feasibility.rs` (deleted in 10b; migrated to [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs)) | 17 tests covering workaround 1 (lexical-sort macro) plus workaround 3 (`CoproductSubsetter` fallback), generic-effect handling, lifetime parameters, 5- and 7-effect rows for trait-inference scaling, plus `tstr_crates` Phase 2 refinement (3 tests showing content-addressed naming + `tstr::cmp` compile-time ordering). All pass on stable Rust 1.94. |
| `poc-effect-row/tests/coyoneda.rs` (deleted in 10b; migrated to [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs))    | 8 tests validating static-via-Coyoneda end-to-end: `effects_coyo!` macro emits Coyoneda-wrapped Coproducts canonically; `Coyoneda<F, A>` is `Functor` for any `F`; `Coproduct<H, T>` implements `Functor` via recursive trait dispatch with no specialization or runtime dictionary; row canonicalises across input orderings under wrapping.              |
| [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)                                                                                     | 6 tests validating `FreeExplicit<'a, F, A>` integrates with the Brand-and-Kind machinery, supports non-`'static` payloads, supports two-effect Run-shaped composition. One `#[ignore]`d test documents that naive `Drop` overflows on deep chains; the iterative custom `Drop` ships in Phase 1.                                                           |
| [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs)                                                                   | Criterion bench at depths 10 / 100 / 1000 / 10000 confirming `FreeExplicit`'s per-node cost is approximately 27ns in the linear regime. The Phase-1 baseline for measuring `RcFree` / `ArcFree` regressions.                                                                                                                                               |

The POC code (the `effects!` / `effects_coyo!` macros, the stub
Coyoneda) migrates into production during Phase 2 and Phase 3; the
POC repos remain as reference until then and are deleted once the
production tests cover the same surface.

## Key decisions

The full decision rationale is in [decisions.md](decisions.md).
Quick reference table:

| ID        | Decision                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | Rationale (one-line)                                                                                                                                                                                                                                                                                                                            |
| --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 4.1       | Option 4 hybrid (macro + nested Coproduct) with corophage-style `'a` per effect                                                                                                                                                                                                                                                                                                                                                                                                                         | Most production-credible reference (corophage) and best stable-Rust ergonomics                                                                                                                                                                                                                                                                  |
| 4.1       | Workaround 1 (macro canonicalisation) primary; workaround 3 (`CoproductSubsetter`) fallback                                                                                                                                                                                                                                                                                                                                                                                                             | Macro pays the sort cost once at row construction; Subsetter handles hand-written rows                                                                                                                                                                                                                                                          |
| 4.1       | tstr_crates content-addressed naming as Phase 2 refinement                                                                                                                                                                                                                                                                                                                                                                                                                                              | Stable type-level identity across import paths; the only credible stable-Rust improvement                                                                                                                                                                                                                                                       |
| 4.2       | Static option via `Coyoneda` per effect                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | Each row variant is trivially a Functor; section 5.2 commits to Coyoneda anyway                                                                                                                                                                                                                                                                 |
| 4.3       | Ship both `interpret` and `interpretRec` families                                                                                                                                                                                                                                                                                                                                                                                                                                                       | Documentation parity with PureScript Run; few-percent runtime cost is small                                                                                                                                                                                                                                                                     |
| 4.4       | Six-variant Free: `Free`, `RcFree`, `ArcFree` (Erased) + `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit` (Explicit)                                                                                                                                                                                                                                                                                                                                                                                 | Erased family is inherent-method-only with O(1) bind; Explicit family is Brand-dispatched with O(N) bind; Erased/Explicit split is the dispatch story                                                                                                                                                                                           |
| 4.4       | `SendFunctor` / `SendPointed` / `SendSemimonad` / `SendMonad` trait family for `ArcFreeExplicitBrand`                                                                                                                                                                                                                                                                                                                                                                                                   | By-value parallel of existing `SendRef*` family; closes the same gap that today prevents `ArcCoyonedaBrand` from implementing `Functor`                                                                                                                                                                                                         |
| 4.5       | Heftia dual-row for scoped effects                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | Cleanest higher-order effect encoding surveyed; preserves first-class programs                                                                                                                                                                                                                                                                  |
| 4.5       | `'a` lifetime parameter on every scoped-effect constructor from day one                                                                                                                                                                                                                                                                                                                                                                                                                                 | Avoids breaking-change retrofit when `FreeExplicit` use cases want non-`'static` actions                                                                                                                                                                                                                                                        |
| 4.5       | Fixed `Run<R, A>` interpreter continuation (no associated type)                                                                                                                                                                                                                                                                                                                                                                                                                                         | Matches every Haskell library surveyed; associated type deferred until use case forces it                                                                                                                                                                                                                                                       |
| 4.5       | Coproduct-of-constructors for user-defined scoped effects                                                                                                                                                                                                                                                                                                                                                                                                                                               | Mirrors the first-order row's structure; preserves first-class-programs property                                                                                                                                                                                                                                                                |
| 4.6       | `handlers!{...}` macro DSL primary; builder pattern fallback                                                                                                                                                                                                                                                                                                                                                                                                                                            | Same shape as section 4.1's macro + mechanical-fallback hybrid                                                                                                                                                                                                                                                                                  |
| (impl)    | Phase 3 interpreter family ships three orthogonal primitives: simple `interpret` (M-free, returns `A`), pipeline `interpret_with::<E>` (row-narrowing), MonadRec `interpret_rec<M>` (external target with `tail_rec_m`); see [resolutions.md](resolutions.md#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape)                                                                                                                                                                              | Each shape uniquely enables a use case the others cannot subsume; the M-free simple form sidesteps MonadRec abstraction for basic value extraction; the asymmetric layout avoids the Clone-bound and reshape costs of PureScript-faithful `<M: Monad>`-bound symmetry                                                                           |
| (impl)    | Phase 3 step 4 handler shape mirrors PureScript: input `Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Run<R, S, A>>>) -> M::Of<'_, Run<R, S, A>>`; the interpreter does `<R as Functor>::map(M::pure, peel_layer)` to lift Run-continuations to M-wrapped before dispatch; reuses the existing `DispatchHandlers` trait with `NextProgram = M::Of<Run<R, S, A>>`; see [resolutions.md](resolutions.md#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading) | Lets handlers do real `M::bind`-monadic work between effect dispatches (the load-bearing capability that distinguishes step 4 from step 2); reuses the existing dispatch trait without ceremony; alternatives lose the M-monadic-handler capability for cosmetic simplification                                                                 |
| (impl)    | Phase 3 step 4 relaxes `DispatchHandlers::dispatch` from `&mut self` to `&self` (and `Handler::F: Fn` in the impl bounds); lands as the first commit in step 4; see [resolutions.md](resolutions.md#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading)                                                                                                                                                                                           | Required for `tail_rec_m`'s `Fn`-bound step closure to call dispatch on each iteration; matches actual handler usage across the codebase (interior-mutability via `Rc<RefCell>` / `Arc<Mutex>`) so no test regressions expected; alternatives (clone per iteration, `RefCell` wrap) are workarounds for a constraint that doesn't actually bind |
| (impl)    | Phase 3 step 4 `run_accum_rec` continues closure-capture state threading (parity with step 2's `run_accum`); state-via-`StateT s m` deferred to Phase 6+; see [resolutions.md](resolutions.md#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading)                                                                                                                                                                                                 | Matches step 2's `run_accum` shape; minimises cognitive load and keeps the trait machinery thin; `StateT` is a separate Phase 6+ addition revisitable when the demand surfaces                                                                                                                                                                  |
| 9.3 / 9.4 | Sync interpreters in v1; async (and async IO) via `Future` as a `MonadRec` target in Phase 3                                                                                                                                                                                                                                                                                                                                                                                                            | "User picks the target monad" -- single mechanism, no parallel `AsyncRun` family                                                                                                                                                                                                                                                                |
| 9.8       | All effects-related macros live in `fp-macros`; split off a separate crate only if needed                                                                                                                                                                                                                                                                                                                                                                                                               | One crate, one release cadence, one place to coordinate macro semantics                                                                                                                                                                                                                                                                         |
| 9.9       | TalkF + DinnerF integration test from `purescript-run` as the headline Phase 4 milestone                                                                                                                                                                                                                                                                                                                                                                                                                | Real-world reference; validates the port behaves like `purescript-run` for a worked example                                                                                                                                                                                                                                                     |

## Integration surface

### Will change

| Component                                                                                         | Change                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fp-library/src/types/free.rs`                                                                    | Existing `Free<F, A>` keeps its current shape; inherent-method only (no Brand). Minor adjustments if integration with `Run` requires.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `fp-library/src/types/free_explicit.rs`                                                           | **New module (Phase 1 step 1).** Promote `FreeExplicit<'a, F, A>` from POC, add iterative custom `Drop`, add full by-value `Functor` / `Pointed` / `Semimonad` / `Monad` impls plus full `RefFunctor` / `RefPointed` / `RefSemimonad` / `RefMonad` impls (Phase 1 step 7). The naive recursive enum has no Clone bound on bind, so both hierarchies land cleanly.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `fp-library/src/types/rc_free.rs`                                                                 | **New module (Phase 1 step 2).** `RcFree<F, A>` following the `Free` template with `FnBrand<RcBrand>`-shaped continuations (i.e., `Rc<dyn 'a + Fn(B) -> RcFree<F, A>>` via the unified [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) abstraction). Multi-shot effects (`Choose`, `Amb`). Inherent-method only; no `RcFreeBrand`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `fp-library/src/types/arc_free.rs`                                                                | **New module (Phase 1 step 3).** `ArcFree<F, A>` following the `ArcCoyoneda` template with `FnBrand<ArcBrand>`-shaped continuations (i.e., `Arc<dyn 'a + Fn(B) -> ArcFree<F, A> + Send + Sync>` via [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) parameterised by [`ArcBrand`](../../../fp-library/src/brands.rs#L43)) and the `Send`/`Sync` Kind-trait pattern via `SendRefCountedPointer`. Inherent-method only; no `ArcFreeBrand`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `fp-library/src/types/rc_free_explicit.rs`                                                        | **New module (Phase 1 step 4).** `RcFreeExplicit<'a, F, A>` extending `FreeExplicit`'s concrete recursive enum with an outer `Rc<RcFreeExplicitInner>` wrapper plus `Rc<dyn Fn>` continuations. O(N) bind, multi-shot, `A: 'a`, Brand-compatible (`RcFreeExplicitBrand<F>` registered in step 4). Custom iterative `Drop` via `Extract` + `Rc::try_unwrap`. Brand-level dispatch in step 7: `Pointed` only on by-value (`pure` has no Clone bound); full `RefFunctor` / `RefSemimonad` / `RefMonad` plus supporting Ref traits per [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md). By-value `bind` / `map` ship as inherent methods with their natural `A: Clone` bounds.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `fp-library/src/types/arc_free_explicit.rs`                                                       | **New module (Phase 1 step 5).** `ArcFreeExplicit<'a, F, A>` extending `RcFreeExplicit`'s shape with `Arc<...>` wrapping and `Arc<dyn Fn + Send + Sync>` continuations. Same `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick as `ArcFree`. Brand-compatible (`ArcFreeExplicitBrand<F>` registered in step 5). Brand-level dispatch in step 7: `SendPointed` (added by step 6) on by-value; full `SendRefFunctor` / `SendRefSemimonad` / `SendRefMonad` plus supporting `SendRef*` traits. By-value `bind` / `map` ship as inherent methods with `A: Clone + Send + Sync` bounds.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `fp-library/src/classes/send_functor.rs`, `send_pointed.rs`, `send_semimonad.rs`, `send_monad.rs` | **New trait files (Phase 1 step 6).** By-value parallels of the existing `send_ref_*` family with `Send + Sync` bounds on the closure parameters and on values entering the structure (`SendPointed::pure(a: A)` requires `A: Send + Sync`). `SendPointed` lands as the brand-level `pure` for `ArcCoyonedaBrand` (closing the open gap module docs flag) and `ArcFreeExplicitBrand`. `SendFunctor` / `SendSemimonad` / `SendMonad` carry trait impls for `ArcCoyonedaBrand` (whose by-value path has no Clone bound). The multi-shot Explicit Free family does not implement `SendFunctor` / `SendSemimonad` / `SendMonad` at the brand level (Clone bound on bind makes them unexpressible) and instead routes brand-level dispatch through the existing `SendRef*` hierarchy in step 7.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `fp-library/src/types/effects.rs`                                                                 | **New module (Phase 2 step 4).** Six concrete Run types: `Run<R, S, A>`, `RcRun<R, S, A>`, `ArcRun<R, S, A>` (Erased family, inherent-method only) and `RunExplicit<'a, R, S, A>` (Explicit, full by-value brand-dispatched), `RcRunExplicit<'a, R, S, A>`, `ArcRunExplicit<'a, R, S, A>` (Explicit, Pointed/SendPointed by-value plus full Ref/SendRef brand coverage). `Node<R, S>` enum dispatching first-order vs scoped layers. `into_explicit()` / `from_erased()` conversion API between paired Erased and Explicit Run variants.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `fp-library/src/types/effects/coproduct.rs`                                                       | **New submodule.** Brand-aware adapter layer over `frunk_core::coproduct::{Coproduct, CNil, CoproductSubsetter}`: newtype wrappers, `impl` blocks bridging `frunk_core`'s Plucker / Sculptor / Embedder traits to the project's `Brand` system. Direct (non-newtyped) `Functor` impls on `frunk_core::Coproduct<H, T>` live here too (own-trait + foreign-type, orphan-permitted).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `fp-library/src/types/effects/variant_f.rs`                                                       | **New submodule.** `VariantF<Effects>` first-order coproduct with Coyoneda-wrapped variants and recursive `Functor` impl on `Coproduct<H, T>` (delegating to the adapter in `coproduct.rs`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `fp-library/src/types/effects/scoped.rs`                                                          | **New submodule.** `ScopedCoproduct<ScopedEffects>` higher-order coproduct, standard scoped constructors. `Catch<'a, E>` and `Span<'a, Tag>` ship Val-only. `Local` ships in Val and Ref flavours (`Local<'a, E>` + `RefLocal<'a, E>`); `Bracket` ships in Val and Ref flavours (`Bracket<'a, A, B>` + `RefBracket<'a, P, A, B>`) per [decisions.md](decisions.md) section 4.5 sub-decisions. `Mask` is deferred to a future revision per the same section.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `fp-library/src/dispatch/run/`                                                                    | **New submodule.** Closure-driven Val/Ref dispatch for `bracket` and `local` smart constructors, mirroring the existing layout described in [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md). Files: `bracket.rs` (`BracketDispatch` trait + `Val` impl + `Ref<P>` impls per pointer brand + `bracket` inference wrapper + `explicit::bracket` brand-explicit wrapper); `local.rs` (`LocalDispatch` trait + `Val` and `Ref` impls + `local` inference wrapper + `explicit::local` wrapper). Re-exported from `fp-library/src/functions.rs` alongside `map`, `bind`, etc.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-library/src/types/effects/handler.rs`                                                         | **New submodule.** Handler-pipeline machinery (`Run::handle`), natural-transformation type, `peel` / `send` / `extract`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `fp-library/src/types/effects/interpreter.rs`                                                     | **New submodule.** `interpret` / `run` / `runAccum` (recursive) and `interpretRec` / `runRec` / `runAccumRec` (`MonadRec`-targeted) families.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-macros/src/effects/`                                                                          | **New module tree.** `effects!`, `effects_coyo!`, `handlers!`, `define_effect!`, `define_scoped_effect!`, `scoped_effects!`, and `im_do!` proc-macros (with `ia_do!` planned as a future companion). `im_do!` (Inherent Monadic do) is the inherent-method-based monadic do-notation that desugars to chained `.bind(...)` / `.ref_bind(...)` method calls and works uniformly across all six Run wrappers (the Erased family `Run`/`RcRun`/`ArcRun`, plus the Explicit family `RunExplicit`/`RcRunExplicit`/`ArcRunExplicit` for cases where brand-level dispatch isn't reachable, e.g., canonical Coyoneda-headed rows on `RcRunExplicit` or any use of `ArcRunExplicit`'s by-reference path). The Explicit Run family also supports the existing brand-dispatched `m_do!` / `a_do!` over `RunExplicitBrand` (full by-value brand coverage) and the `ref` qualifier (`m_do!(ref ...)` / `a_do!(ref ...)`) over `RcRunExplicitBrand` for synthetic rows whose row brand satisfies `RefFunctor`; canonical Coyoneda-headed rows route through `im_do!(ref RcRunExplicit { ... })` instead. `ia_do!` (Inherent Applicative do) is the inherent-method-based applicative companion to `im_do!`, deferred to a future phase but named in advance to lock in the convention. Migration from POC for the row-construction macros. |
| `fp-library/src/brands.rs`                                                                        | Add brands for the Brand-dispatched (Explicit) types only: `FreeExplicitBrand<F>`, `RcFreeExplicitBrand<F>`, `ArcFreeExplicitBrand<F>`, `RunExplicitBrand<R, S>`, `RcRunExplicitBrand<R, S>`, `ArcRunExplicitBrand<R, S>`. The Erased family (`Free`, `RcFree`, `ArcFree`, `Run`, `RcRun`, `ArcRun`) does NOT get brands; those types remain inherent-method only. `*FreeExplicitBrand<F>` are single-parameter `PhantomData<F>` structs mirroring [`CoyonedaBrand<F>`](../../../fp-library/src/brands.rs#L155); the three `*RunExplicitBrand<R, S>` variants are two-parameter `PhantomData<(R, S)>` structs mirroring [`CoyonedaExplicitBrand<F, B>`](../../../fp-library/src/brands.rs#L171). For all of them, `'static` bounds live on impls (so the row types `R`, `S` and the payload `'a`, `A` stay out of the brand identity and appear only in `Of<'a, A>` at instantiation, keeping brand types `'static`-clean while admitting non-`'static` payloads via the Explicit family).                                                                                                                                                                                                                                                                                                                                   |
| `fp-library/tests/run_*.rs`                                                                       | **New test files.** Per-Free-variant unit tests for all six variants (Phase 1 step 9, including `compile_fail` cases for Brand-dispatched calls against Erased variants and missing `Send + Sync` on `ArcFreeExplicit::bind` closures), row-canonicalisation regression tests migrated from `poc-effect-row/` (Phase 2), `Run <-> RunExplicit` conversion tests (Phase 2 step 6), TalkF + DinnerF integration test (Phase 4).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-library/benches/benchmarks/run_*.rs`                                                          | **New bench files.** Per-Free-variant Criterion benches for all six variants (bind-deep, bind-wide, peel-and-handle) plus a cross-variant comparison documenting the O(1) vs O(N) bind-cost asymmetry between the Erased and Explicit families. Row-canonicalisation benches (macro vs Subsetter), handler-composition benches, and `Run <-> RunExplicit` conversion benches.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |

### Unchanged

- **`Free<F, A>` core** (`fp-library/src/types/free.rs`): the existing
  `Box<dyn FnOnce>`-based variant stays as-is. New variants are
  added beside it.
- **Coyoneda family** (`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`,
  `CoyonedaExplicit`): used by `Run`'s first-order row but not
  modified.
- **Brand-and-Kind machinery** (`fp-macros` HKT macros, `brands.rs`,
  `kinds.rs`): used by `Run` but not modified beyond the new brand
  registrations above.
- **Optics subsystem** (`Lens`, `Prism`, `Iso`, `Traversal`,
  etc.): unrelated to `Run`.
- **Existing `MonadRec` impls** (`Option`, `Result`, `Thunk`,
  etc.): used as interpretation targets but not modified.
- **Pre-existing dispatch traits and `m_do!` / `a_do!`**: continue
  to work for `RunExplicit` (full by-value brand coverage) once
  the corresponding `RunExplicitBrand` impls from Phase 2 step 4
  ship. `RcRunExplicit` / `ArcRunExplicit` carry only `Pointed` /
  `SendPointed` on the by-value side (the Clone bound on bind
  makes `Functor` / `Semimonad` / `Monad` unexpressible at the
  brand level, per `RcCoyoneda` / `ArcCoyoneda` precedent
  documented in
  [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md));
  brand-dispatched typeclass-generic code over them uses the
  existing `m_do!` / `a_do!` macros with the `ref` qualifier
  (`m_do!(ref RcRunExplicitBrand { ... })`), routing through
  `RefFunctor` / `RefSemimonad` (with the constraint that the
  row brand must implement `RefFunctor` , synthetic rows like
  `CoproductBrand<IdentityBrand, CNilBrand>` qualify, but
  canonical Coyoneda-headed rows generated by the `effects!`
  macro do not, because `CoyonedaBrand` cannot implement
  `RefFunctor` on stable Rust per the HRTB-over-types
  limitation in
  [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md)).
  `ArcRunExplicitBrand`'s `SendRef*` hierarchy is permanently
  unreachable on stable Rust regardless of row shape (the
  `ArcFreeExplicitBrand`-side `SendRefFunctor` impl is
  unimplementable for the same reason). The Erased Run family
  (`Run`, `RcRun`, `ArcRun`) is not Brand-dispatched at all.
  Both gaps route through the new `im_do!` (Inherent Monadic
  do) macro, which uses inherent `.bind(...)` /
  `.ref_bind(...)` method calls and works uniformly across all
  six wrappers.

## Out of scope

Permanently excluded from this plan. Revisit only if design
constraints change.

- **Multi-prompt delimited continuations** (Koka / MpEff style).
  Ruled out by [decisions.md](decisions.md) section 1.2; no Rust
  equivalent of GHC's `prompt#` / `control0#`.
- **Tag-based type-level sorting** (workaround 2 from
  decisions Section 4.1). Surveyed in
  [docs/plans/type-level-sorting/research/](../type-level-sorting/research/);
  the credible building blocks exist (`tstr_crates`) but the full
  sort engine on stable Rust requires the user to write it. The
  workaround-1 macro plus workaround-3 Subsetter hybrid is
  sufficient.
- **Evidence-passing dispatch** (EvEff style). Surveyed in
  [research/deep-dive-evidence-passing.md](research/deep-dive-evidence-passing.md);
  collapses to Option 1 (Peano) or Option 3 (TypeId) once removed
  from Haskell's closed-type-family setting.
- **Coroutine substrate without Free.** Surveyed in
  [research/deep-dive-coroutine-vs-free.md](research/deep-dive-coroutine-vs-free.md);
  loses 4 of 5 first-class-program properties section 4.4
  requires.
- **`mtl`-style trait-bound effect set** (Option 5 from
  decisions Section 4.1). Loses first-class programs.
- **Custom `Effect`-monad analogue.** Section 9.4 commits to
  `Thunk` (v1) and `Future` (Phase 3) as `MonadRec` targets;
  inventing a Rust-specific `Effect` monad is unnecessary.
- **`async fn`-shaped interpreters.** Section 9.3 commits to
  sync interpreters with async-via-target-monad. No parallel
  `AsyncRun` family.
- **Callable continuation primitives in handler clauses
  (Plotkin-Pretnar `k : x -> Result`).** The freer-monad
  encoding stores per-effect closures inside variant
  constructors; handlers fold sub-programs via the
  [`DispatchHandlers`](../../../fp-library/src/types/effects/interpreter.rs)
  trait but do not receive a uniform resumable continuation.
  Multi-shot semantics are achievable via the per-effect
  closure (e.g.,
  [`Choose`](../../../fp-library/src/types/effects/choose.rs)'s
  embedded `dyn Fn(bool) -> NextProgram` can be invoked twice),
  but the shape is per-effect, not per-handler. This is a
  structural property of the freer-monad encoding, not a
  defect of the implementation. Stable Rust has no
  delimited-continuation primitive
  ([`prompt#`](https://gitlab.haskell.org/ghc/ghc/-/wikis/proposal/delimited-continuation-primops)
  / `control0#` GHC equivalents), so a uniform `k` cannot be
  exposed even with library-level effort.
- **Rank-2 natural transformations at the headline
  interpreter API.** `interpret` and friends take handler
  closures with a fixed `NextProgram` type per dispatch step,
  not an `A`-polymorphic shape. PureScript Run's
  `(VariantF r ~> m) -> Run r a -> m a` (a true rank-2 NT)
  cannot be expressed with stable-Rust closures (no
  rank-2-quantification over types). The mono-in-`A` form was
  ratified as
  [Decision 1, Q1](resolutions.md#resolved-2026-04-29-phase-3-step-23-interpreter-family-shape).
  Users who genuinely need rank-2 polymorphism over `A` reach
  for [`NaturalTransformation`](../../../fp-library/src/classes/natural_transformation.rs)
  directly, consumed by
  [`Free::fold_free`](../../../fp-library/src/types/free.rs)
  or similar; that path bypasses the per-effect handler-list
  pattern. The headline `interpret` API trades rank-2 reach
  for closure-fitting ergonomics.

## Implementation phasing

All five phases ship together as one feature release.

### Phase 1: Complete the Free family

Land the five missing Free variants and the `SendFunctor` trait
family. Phases 2-5 treat the choice of variant as a user-level
parameter, so completing the substrate first prevents later
refactor. The Erased family (`Free`, `RcFree`, `ArcFree`) is
inherent-method only; the Explicit family (`FreeExplicit`,
`RcFreeExplicit`, `ArcFreeExplicit`) carries Brand dispatch. See
[decisions.md](decisions.md) section 4.4 for rationale.

1. Promote `FreeExplicit<'a, F, A>` from POC to
   `fp-library/src/types/free_explicit.rs`. Add iterative custom
   `Drop` per [decisions.md](decisions.md) section 4.4 ("What to
   do about `Drop`"). Delete the POC's local copy in
   `fp-library/tests/free_explicit_poc.rs` and replace with a
   `use fp_library::types::FreeExplicit;` import. Move the bench at
   `fp-library/benches/benchmarks/free_explicit.rs` to use the
   imported type. Un-`#[ignore]` the deep-`Drop` test once the
   iterative `Drop` ships.
2. Implement `RcFree<F, A>` at `fp-library/src/types/rc_free.rs`
   following the `Free` template, with continuations expressed via
   [`FnBrand<RcBrand>`](../../../fp-library/src/types/fn_brand.rs)
   (yielding `Rc<dyn 'a + Fn(B) -> RcFree<F, A>>` after `Kind`
   resolution) and the `RcCoyoneda` cloning pattern. Add
   `lower_ref(&self)` / `peel_ref(&self)` for non-consuming
   reinterpretation. The `FnBrand`-based shape is preferred over a
   raw `Rc<dyn Fn>` field so the new module participates in the
   library's unified function-pointer abstraction from day one
   (see [`fn_brand.rs`](../../../fp-library/src/types/fn_brand.rs)).
   Inherent-method only; no `RcFreeBrand` (the `'static` requirement
   from `Rc<dyn Any>` erasure is incompatible with `Kind`'s
   `Of<'a, A: 'a>: 'a` signature).
3. Implement `ArcFree<F, A>` at `fp-library/src/types/arc_free.rs`
   following the `ArcCoyoneda` template, with continuations
   expressed via
   [`FnBrand<ArcBrand>`](../../../fp-library/src/types/fn_brand.rs)
   parameterised by
   [`ArcBrand`](../../../fp-library/src/brands.rs#L43) (yielding
   `Arc<dyn 'a + Fn(B) -> ArcFree<F, A> + Send + Sync>` after
   `Kind` resolution via `SendRefCountedPointer`) and the
   `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick.
   Inherent-method only; no `ArcFreeBrand` (same `'static` reason).
4. Implement `RcFreeExplicit<'a, F, A>` at
   `fp-library/src/types/rc_free_explicit.rs` extending
   `FreeExplicit`'s concrete recursive enum with an outer
   `Rc<RcFreeExplicitInner>` wrapper plus
   [`FnBrand<RcBrand>`](../../../fp-library/src/types/fn_brand.rs)-shaped
   continuations. `A: 'a` (no `'static` requirement) because the
   structure has no `dyn Any` cell; O(N) bind via spine recursion
   through `F::map`. Add `lower_ref(&self)` / `peel_ref(&self)` and
   custom iterative `Drop` (the same `Extract`-driven dismantling
   pattern as `FreeExplicit`, with `Rc::try_unwrap` inside the
   loop). Brand-compatible: this is the multi-shot variant that
   carries Brand dispatch in Phase 1 step 7.
5. Implement `ArcFreeExplicit<'a, F, A>` at
   `fp-library/src/types/arc_free_explicit.rs` extending
   `RcFreeExplicit`'s shape with `Arc<...>` wrapping and
   `Arc<dyn Fn + Send + Sync>` continuations (constructed via
   [`<ArcFnBrand as SendLiftFn>::new`](../../../fp-library/src/classes/send_clone_fn.rs)).
   Same `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick
   as `ArcFree`. `Send + Sync`-capable; Brand-compatible (with the
   `SendFunctor` family from step 6 supplying the missing
   trait-method bounds).
6. Add the `SendFunctor` trait family at
   `fp-library/src/classes/`: `send_functor.rs`,
   `send_pointed.rs`, `send_semimonad.rs`, `send_monad.rs` (the
   by-value parallels of the existing `send_ref_*` files).
   `SendFunctor` / `SendSemimonad` / `SendMonad` take their
   closure parameter as `impl Fn(...) + Send + Sync`;
   `SendPointed::pure(a: A)` requires `A: Send + Sync`. Resolves
   the gap that today prevents `ArcCoyonedaBrand` from
   implementing `Functor` and gives `ArcFreeExplicitBrand` a
   brand-level `pure`. Add `SendFunctor` / `SendPointed` /
   `SendSemimonad` / `SendMonad` implementations for
   `ArcCoyonedaBrand` as a bonus integration, closing the open
   gap that
   [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs)'s
   module docs flag (`ArcCoyoneda`'s by-value path has no Clone
   bound, so the full hierarchy lands).
7. Add by-value and by-reference trait hierarchies for the three
   Explicit Free brands. The brand structs (`FreeExplicitBrand<F>`,
   `RcFreeExplicitBrand<F>`, `ArcFreeExplicitBrand<F>`) and their
   `Kind` registrations land alongside the type definitions in
   steps 1, 4, and 5; this step implements the type-class traits
   on top of them. Brand-level coverage matches the resolved
   open question above:
   - `FreeExplicitBrand`: full by-value (`Functor` / `Pointed` /
     `Semimonad` / `Monad`) plus full by-reference (`RefFunctor`
     / `RefPointed` / `RefSemimonad` / `RefMonad` and supporting
     Ref traits per
     [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md)).
     The naive recursive enum has no Clone bound on bind, so
     both hierarchies land cleanly.
   - `RcFreeExplicitBrand`: `Pointed` only on the by-value side
     (`pure` has no Clone bound); full Ref hierarchy
     (`RefFunctor` / `RefSemimonad` / `RefMonad`, plus
     `RefPointed` and supporting Ref traits). The remaining
     by-value operations (`bind`, `map`, etc.) ship as inherent
     methods on `RcFreeExplicit` with their natural `A: Clone`
     bounds.
   - `ArcFreeExplicitBrand`: `SendPointed` (from step 6) on the
     by-value side; full `SendRef*` hierarchy (`SendRefFunctor`
     / `SendRefSemimonad` / `SendRefMonad` plus supporting
     `SendRef*` traits, which already exist in
     [`fp-library/src/classes/`](../../../fp-library/src/classes/)).
     The remaining by-value operations ship as inherent methods
     on `ArcFreeExplicit` with `A: Clone + Send + Sync` bounds.
   - The Erased family (`Free`, `RcFree`, `ArcFree`) does not get
     brands; those types remain inherent-method only.
   - Both hierarchies are required so `dispatch::map` /
     `dispatch::bind` route correctly over each Brand-dispatched
     Free variant once `Run` and the scoped-effect smart
     constructors land in Phase 2 / Phase 4. The Ref hierarchy
     is the dispatch path for typeclass-generic code over the
     multi-shot Explicit Run variants.
   - Update
     [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md)'s
     "Unexpressible Bounds in Trait Method Signatures"
     classification table to add rows for the three Explicit
     Free variants documenting the brand-level coverage above
     (matching the existing `RcCoyoneda` / `ArcCoyoneda` rows).
8. Per-variant Criterion benches for all six variants (bind-deep
   at depths 10 / 100 / 1000 / 10000, bind-wide, peel-and-handle),
   plus a cross-variant comparison bench documenting the O(1) vs
   O(N) bind-cost asymmetry. The existing
   [`free_explicit.rs`](../../../fp-library/benches/benchmarks/free_explicit.rs)
   POC bench has the `bind-deep` shape only; step 8 extends that
   shape with `bind-wide` (single bind closure mapping over a
   wide-but-shallow chain) and `peel-and-handle` (single-step
   `to_view` / `peel_ref` cost) and replicates the full set
   across all six variants.
9. Per-variant unit tests covering construction, evaluation, and
   the property each variant promises (single-shot vs.
   multi-shot, thread-safe, `'static` vs `'a`, Brand-dispatched
   vs inherent-method-only). The canonical interpretation method
   varies by variant: `Free::fold_free` for `Free` (the only
   variant with that inherent method); `evaluate` for
   `RcFree`/`ArcFree`/`FreeExplicit`/`RcFreeExplicit`/`ArcFreeExplicit`.
   Plus `compile_fail` UI tests under
   [`fp-library/tests/ui/`](../../../fp-library/tests/ui/)
   (registered via the existing
   [`fp-library/tests/compile_fail.rs`](../../../fp-library/tests/compile_fail.rs)
   `trybuild` harness) for the negative cases: multi-shot via
   `Free`, Brand-dispatched call against an Erased variant,
   missing `Send + Sync` on a closure passed to
   `ArcFreeExplicit::bind`, missing `Clone` on a closure passed
   to `RcFree::bind`, etc.

### Phase 1 follow-up: `WrapDrop` migration (resolves Phase 2 step 4 blocker)

These commits land between Phase 1 and Phase 2 step 4. They lift
the Free family's struct-level `Extract` bound so the Phase 2
step 4 architectural commitment
`Run<R, S, A> = Free<NodeBrand<R, S>, A>` can compile over
typical effect rows whose effect types do not implement
`Extract`. Full rationale, problem statement, probe results, and
per-F policy decisions live at
`## Open questions, issues and blockers -> ### Resolved (2026-04-27): introduce WrapDrop trait for Free's struct-level Drop concern`;
this section is the phasing-side checklist.

1. **Introduce the `WrapDrop` trait and migrate the Free family.**
   New trait at `fp-library/src/classes/wrap_drop.rs` with
   signature
   `fn drop<'a, X: 'a>(fa: Self::Of<'a, X>) -> Option<X>`.
   `WrapDrop` impls for the two existing
   `Extract`-implementing brands (`IdentityBrand`,
   `ThunkBrand`), each delegating to their existing
   `Extract` impl by returning
   `Some(<Self as Extract>::extract(fa))`. Replace
   `F: Extract + Functor + 'static` with
   `F: WrapDrop + Functor + 'static` on the struct, `FreeView`,
   `FreeStep`, and `Drop` declarations of all six Free
   variants (`Free`, `RcFree`, `ArcFree`, `FreeExplicit`,
   `RcFreeExplicit`, `ArcFreeExplicit`). Inventory: 71
   occurrences of `F: Extract` across the six variant source
   files, mechanically migrated. Methods that call
   `F::extract` semantically (`evaluate`, `resume`, etc.)
   keep `where F: Extract` on their impl blocks. Rewrite
   Free's `Drop` loop to call `F::drop` and switch on the
   returned `Option`: `Some(inner)` follows the existing
   iterative path; `None` lets the layer drop in place.
   Existing Phase 1 tests (including
   `deep_drop_does_not_overflow` for both `Free<ThunkBrand>`
   and `FreeExplicit<IdentityBrand>`) must all pass.
2. **Relax `Functor` bound to `Kind` on Free's struct.**
   Change the struct, `FreeView`, `FreeStep`, and `Drop`
   bounds from `F: WrapDrop + Functor + 'static` to
   `F: WrapDrop + 'static` (the `Kind` GAT requirement is
   inherited from `WrapDrop`'s `Kind` supertrait). Add
   `where F: Functor` to impl blocks that call `F::map`
   (`wrap`, `lift_f`, `to_view`, and methods that go through
   them transitively such as `evaluate`, `resume`,
   `fold_free`). Methods like `pure`, `bind`, `map` (the
   inherent method, not `Functor::map`) do not need the
   bound. Process: relax the struct bound, run
   `cargo check`, add `where F: Functor` to every impl block
   the compiler flags, repeat until clean. Same six Free
   variants. Same tests must pass.

### Phase 2: Run substrate and first-order effects

1. Add `frunk_core` as a direct dependency of `fp-library`
   (license check via `just deny`, MSRV verification, and
   workspace `Cargo.toml` registration). Introduce a thin
   Brand-aware adapter layer at `fp-library/src/types/effects/coproduct.rs`:
   newtype wrappers around `frunk_core::coproduct::{Coproduct, CNil}`
   plus `impl` blocks bridging `frunk_core`'s Plucker / Sculptor /
   Embedder traits to the project's `Brand` system. Direct `impl`s
   of fp-library's own `Functor` for `frunk_core::Coproduct<H, T>`
   are permitted by the orphan rules; `Brand`-style impls require
   the newtype wrapper. See Implementation note 1 below.
2. `VariantF<Effects>` at `fp-library/src/types/effects/variant_f.rs`:
   Coyoneda-wrapped Coproduct row with recursive `Functor` impl
   on `Coproduct<H, T>` (where `H: Functor + T: Functor`) and base
   case on `CNil`. Migrate the trait-shape from
   `poc-effect-row/src/lib.rs` (workspace deleted in 10b)
   under the production `Functor` trait.
3. `Member<E, Indices>` trait for first-order injection /
   projection, layered on top of `frunk_core::CoproductSubsetter`
   via the adapter from step 1.
4. Six `Run` types at `fp-library/src/types/effects.rs` (and
   sibling files), one per Free variant: `Run<R, S, A>`,
   `RcRun<R, S, A>`, `ArcRun<R, S, A>` (Erased family,
   inherent-method only) and `RunExplicit<'a, R, S, A>`,
   `RcRunExplicit<'a, R, S, A>`, `ArcRunExplicit<'a, R, S, A>`
   (Explicit family, Brand-dispatched). Each is a thin wrapper
   over its Free variant with a shared `Node<R, S>` enum
   dispatching first-order vs scoped layers.
   This step depends on the Phase 1 follow-up commits above
   (the `WrapDrop` migration); without them,
   `Free<NodeBrand<R, S>, A>` does not compile because effect
   types do not implement `Extract`. As part of this step,
   `WrapDrop` impls also land for the row brands that this
   step exercises:
   - `NodeBrand<R, S>`: dispatches by First/Scoped, delegating
     to `R::drop` and `S::drop` respectively. (New brand
     defined in this step.)
   - `CoproductBrand<H, T>` (already exists from Phase 2 step 2):
     dispatches by `Inl`/`Inr`, delegating to `H::drop` and
     `T::drop`.
   - `CNilBrand` (already exists from Phase 2 step 2): the
     uninhabited base case, `match fa {}`.
   - `CoyonedaBrand<E>` (already exists): returns `None`. The
     Coyoneda's stored function would construct a Free if
     called, but does not store one in its environment;
     recursive drop on the Coyoneda is sound for Run-typical
     patterns per the Wrap-depth probe findings recorded in
     the resolution above.
   - `RunExplicitBrand`: full by-value (`Functor` / `Pointed` /
     `Semimonad` / `Monad`) plus full by-reference (`RefFunctor`
     / `RefPointed` / `RefSemimonad` / `RefMonad` and supporting
     Ref traits) by delegating to `FreeExplicitBrand`'s impls
     from Phase 1 step 7.
   - `RcRunExplicitBrand`: `Pointed` only on the by-value side
     (delegating to `RcFreeExplicitBrand::pure`); full Ref
     hierarchy delegating to `RcFreeExplicitBrand`'s `Ref*`
     impls. By-value `bind` / `map` ship as inherent methods on
     `RcRunExplicit`, mirroring `RcFreeExplicit`'s inherent
     surface.
   - `ArcRunExplicitBrand`: `SendPointed` (added by step 6) on
     the by-value side; full `SendRef*` hierarchy delegating to
     `ArcFreeExplicitBrand`'s impls. By-value `bind` / `map`
     ship as inherent methods on `ArcRunExplicit` with
     `A: Clone + Send + Sync` bounds.
   - The three Erased Run types do NOT get brands. They expose
     identical inherent-method APIs (`pure`, `peel`, `send`,
     `bind`, `map`, `lift_f`, `handle`, `extract`, etc.) but
     `m_do!` / `a_do!` do not work over them; `im_do!` from
     step 7 below is the inherent-method-based macro analogue.
   - The hierarchies on the Explicit brands are scoped so
     `dispatch::map` / `dispatch::bind` and the matching
     do-notation macros (`m_do!` / `a_do!` for `RunExplicit`;
     `m_do!(ref ...)` / `a_do!(ref ...)` for `RcRunExplicit` /
     `ArcRunExplicit`) route correctly. Inherent by-value `bind`
     / `map` on the multi-shot Explicit Run variants cover the
     non-generic case where the user has `A: Clone` available.
5. `Run::pure`, `Run::peel`, `Run::send` core operations on
   each of the six Run variants, delegating to the underlying
   Free variant.
6. Conversion methods between paired Erased and Explicit Run
   variants: `Run::into_explicit() -> RunExplicit`,
   `RcRun::into_explicit() -> RcRunExplicit`,
   `ArcRun::into_explicit() -> ArcRunExplicit`, and the reverse
   `RunExplicit::from_erased(...)`, etc. Walks the Free
   structure once via `peel` / `to_view`, rebuilds in the other
   shape; O(N) in the chain depth. Preserves multi-shot /
   `Send + Sync` properties of the underlying substrate
   (`RcRun -> RcRunExplicit` keeps multi-shot via `Rc<dyn Fn>`
   continuations on both sides; `ArcRun -> ArcRunExplicit`
   keeps `Send + Sync`).
7. **Inherent monadic do-notation: `im_do!` macro plus the
   inherent-method scaffolding it desugars against.** Three
   sub-tasks:

   **7a. Inherent `bind` and `map` on the four Run wrappers
   that don't already have them.** `Run`, `RcRun`, `ArcRun`,
   and `RunExplicit` need both inherent methods at the
   wrapper level (delegating to their underlying Free
   variant's `bind` / `map`); `RcRunExplicit` / `ArcRunExplicit`
   already ship them from step 4b. Bounds match the underlying
   substrate's: `Run` / `RunExplicit` have no extra bounds
   beyond their impl block; `RcRun` / `ArcRun` need
   `A: Clone` plus the projection `Clone` bound that their
   `peel` carries; `ArcRun` additionally needs
   `A: Send + Sync` and `NodeBrand<R, S>: Functor` per-method
   (the impl block carries only the `Send + Sync` projection
   HRTB).

   **7b. Inherent `ref_bind` and `ref_map` on the four
   `Clone`-able wrappers.** `RcRun`, `ArcRun`, `RcRunExplicit`,
   `ArcRunExplicit` are all `Clone` (via `Rc`/`Arc`-shared
   substrate); `Run` and `RunExplicit` are not (they wrap an
   unboxed substrate). On the `Clone`-able four, `ref_bind`
   is implementable as `self.clone().bind(move |a| f(&a))`
   and `ref_map` analogously. The clone is `O(1)` (Rc/Arc
   refcount bump), so the by-reference path adds one cheap
   refcount operation per layer.

   The structural reason this matters: it sidesteps the
   `R: RefFunctor` cascade that brand-level
   `RcFreeExplicitBrand: RefSemimonad` / `RcRunExplicitBrand: RefSemimonad`
   require. The inherent `ref_bind` walks the substrate
   by-value (with the wrapping closure converting `A` to
   `&A` for the user-supplied closure), so the row brand
   doesn't need `RefFunctor`. This is the path that lets
   users get by-reference semantics over canonical
   Coyoneda-headed rows generated by `effects!`, where
   brand-level `m_do!(ref ...)` cannot reach because
   `CoyonedaBrand: RefFunctor` is unimplementable on stable
   Rust per
   [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md).

   **7c. `im_do!` macro in `fp-macros/src/effects/im_do.rs`**
   ("Inherent Monadic do"). Inherent-method-based monadic
   do-notation that desugars to chained
   `expr.bind(|x| ...)` calls (or `expr.ref_bind(|x| ...)`
   for the `ref` form). Mirrors `m_do!`'s surface syntax so
   users moving between brand-dispatched and inherent paths
   do not have to re-learn anything; the only differences
   are the macro name (`im_do!` vs `m_do!`) and the type
   the macro takes (concrete wrapper, e.g.,
   `im_do!(RcRun { ... })`, vs brand, e.g.,
   `m_do!(RcRunExplicitBrand { ... })`). Both inferred and
   explicit-wrapper modes ship: `im_do!({ ... })` (inferred,
   monomorphizes against the leading bind expression's type)
   and `im_do!(Wrapper { ... })` (explicit, useful when the
   wrapper type can't be inferred or for `pure(x)` rewriting
   to `Wrapper::pure(x)`).

   The `ref` qualifier (`im_do!(ref Wrapper { ... })`) is
   accepted only for the four `Clone`-able wrappers. Using
   it on `Run` or `RunExplicit` produces a clear "cannot use
   `ref` form on non-`Clone` wrapper" diagnostic, demonstrated
   by a `compile_fail` UI test in
   `fp-library/tests/ui/im_do_ref_on_non_clone_wrapper.rs`.

   The macro coverage matrix:

   | Wrapper          | `im_do!` | `im_do!(ref ...)`             | Brand-dispatched alternative                    |
   | :--------------- | :------- | :---------------------------- | :---------------------------------------------- |
   | `Run`            | works    | not implementable (not Clone) | none (Erased family is not Brand-dispatched)    |
   | `RcRun`          | works    | works                         | none                                            |
   | `ArcRun`         | works    | works                         | none                                            |
   | `RunExplicit`    | works    | not implementable (not Clone) | `m_do!`/`a_do!` (full brand coverage)           |
   | `RcRunExplicit`  | works    | works                         | `m_do!(ref ...)` (synthetic rows only)          |
   | `ArcRunExplicit` | works    | works                         | `m_do!`/`a_do!` (only `pure`; bind unreachable) |

   Naming note: `im_do!` ("Inherent Monadic do") parallels
   `m_do!` ("Monadic do"); a future companion `ia_do!`
   ("Inherent Applicative do") is reserved as the
   inherent-method-based applicative analogue (parallel to
   `a_do!`), to be added when a concrete need arises (e.g.,
   handler-side independent-bind composition over `ArcRun`
   in Phase 3+). The names share a common length so neither
   the monadic nor the applicative form is favored
   typographically; users should prefer `ia_do!` over
   `im_do!` whenever binds are independent, just as they
   should prefer `a_do!` over `m_do!`.

   Both forms route their codegen through a shared input
   parser at `fp-macros/src/support/do_input.rs` (extracted
   from the existing `m_do/input.rs` / `a_do/input.rs`)
   so surface-syntax features (typed binds, `let`-in-bind,
   `pure(x)` rewriting, `ref` qualifier, etc.) stay
   consistent across all four macros.

   Implementation reference: the existing
   [`m_do!`](../../../fp-macros/src/lib.rs) and
   [`a_do!`](../../../fp-macros/src/lib.rs) macros are the
   structural template. The differential against `m_do!` is
   only the codegen target: brand-dispatched
   `<Brand as Semimonad>::bind(expr, f)` becomes inherent
   `expr.bind(f)`. Same input parser, same statement-form
   handling, same `ref`-mode lifetime concerns.

8. `effects!` macro in `fp-macros/src/effects/effects.rs`,
   migrated from `poc-effect-row/macros/src/lib.rs`
   (workspace deleted in 10b).
   Lexical-sort by `quote!{}.to_string()`; emit Coyoneda-wrapped
   variants. The un-wrapped Coproduct form lives at
   `crate::__internal::raw_effects!` for fp-library-internal use
   (test fixtures, lower-level combinators) and is not part of
   the public surface; see [decisions.md](decisions.md) section 4.6.
   Factor the lexical-sort logic into a shared `proc-macro2`
   helper used by both `effects!` and `scoped_effects!` (Phase 4
   step 4) so sort-correctness fixes land in one place.
9. **Generic `lift` combinator** (PureScript Run's
   [`Run.lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
   analog) as an inherent associated function on each of the six
   Run wrappers, mirroring `*Run::send`'s shape. Per the
   [2026-04-28 resolution](resolutions.md#resolved-2026-04-28-phase-2-step-9-scope-is-under-specified)
   and the
   [2026-04-28 expansion](resolutions.md#resolved-2026-04-28-implementation-expansion-step-9-sendfunctor-cascade-prerequisites-for-arc-family):
   take the raw effect (an `EBrand::Of<'a, A>` value, not a
   pre-lifted Coyoneda), do the full chain inside the body
   (Coyoneda lift -> row inject -> `Node::First` -> `*Run::send`),
   and let `Idx` be type-inferred at call sites where the row is
   unambiguous (turbofish only on duplicate-effect-type rows). The
   bare name `lift` matches PureScript Run's `Run.lift`; the `_f`
   suffix is reserved for the Free-only operation
   ([`Free::lift_f`](../../../fp-library/src/types/free.rs), the
   snake_case translation of PureScript's `Free.liftF`).

   The work breaks into nine sub-steps; each lands as a separate
   commit and verifies under `just verify` independently. Sub-steps
   9a-9g establish the `SendFunctor` cascade prerequisites for the
   Arc family (the architectural finding documented in the 2026-04-28
   expansion); sub-steps 9h-9i complete the universal `lift` and
   `SendRefFunctor` work. Already landed: the `Run::lift` reference
   implementation at commit
   [`34b6a97`](../../../fp-library/src/types/effects/run.rs) (which
   names + signs off on the chosen design but doesn't extend to the
   Arc family).

   **9a. Brand-level `SendFunctor` cascade plus missing `WrapDrop`.**
   Add `SendFunctor` impls on the row-cascade brands that don't have
   them: `IdentityBrand`, `CNilBrand`, `CoproductBrand<H, T>` (recursive,
   requiring `H: SendFunctor + T: SendFunctor`), and
   `NodeBrand<R, S>` (delegates to the first-order and scoped row
   brands' `SendFunctor` impls). Add the missing
   [`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs) impl on
   [`ArcCoyonedaBrand`](../../../fp-library/src/types/arc_coyoneda.rs)
   (returns `None`, mirroring the existing
   [`CoyonedaBrand`](../../../fp-library/src/types/coyoneda.rs) /
   [`RcCoyonedaBrand`](../../../fp-library/src/types/rc_coyoneda.rs)
   pattern; the Coyoneda's stored function does not materially store
   an inner Free, so structural-recursive drop is sound). All impls
   are mechanical mirrors of the existing `Functor` / `WrapDrop`
   patterns; no novel algorithm.

   **9b. Replace `F: Functor` with `F: SendFunctor` on `ArcFree`.**
   The substrate at
   [`fp-library/src/types/arc_free.rs`](../../../fp-library/src/types/arc_free.rs)
   currently bounds `lift_f`, `wrap`, `bind`, `evaluate`, `fold_free`,
   `hoist_free`, etc. on `F: Functor` and routes `F::map` calls
   through it. Switch all such bounds to `F: SendFunctor` and replace
   `F::map(...)` calls with `F::send_map(...)`. The closures passed
   at the call sites (`ArcFree::pure`, user-supplied
   `Send + Sync`-bound continuations from `bind`, etc.) are already
   `Send + Sync`, so the migration is mechanical. This is a breaking
   change for any pre-existing caller passing a non-Send `Functor`
   row brand, but `ArcFree`'s struct-level
   `Of<'static, ArcFree<...>>: Send + Sync` HRTB already restricts
   concrete callers to row brands that satisfy `Send + Sync`, so
   adding `SendFunctor` impls (sub-step 9a) keeps existing callers
   working.

   **9c. Replace `F: Functor` with `F: SendFunctor` on
   `ArcFreeExplicit`.** Same migration as 9b for the substrate at
   [`fp-library/src/types/arc_free_explicit.rs`](../../../fp-library/src/types/arc_free_explicit.rs).
   Method signatures and internal `F::map` call sites switch to
   `F::send_map`.

   **9d. Expand brand-level type-class surface on
   `ArcFreeExplicitBrand`.** With `ArcFreeExplicit`'s machinery now
   routed through `SendFunctor`, the brand-level coverage gap
   documented in step 4b (per-`A` HRTB blocking SendRef-family
   impls) shifts. Re-evaluate the cascade and land newly-reachable
   impls: at minimum `SendFunctor`; potentially `SendSemimonad`,
   `SendApplicative`, `SendMonad` if their dependencies are
   satisfiable through the same SendFunctor-aware substrate. Document
   any remaining unreachable subset in
   [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md).
   Don't add `SendRefFunctor` here; that's sub-step 9i.

   **9e. Switch `ArcRun` to `SendFunctor`-routed dispatch.** The
   wrapper at
   [`fp-library/src/types/effects/arc_run.rs`](../../../fp-library/src/types/effects/arc_run.rs)
   currently routes `peel`/`send`/`bind`/`map` through
   `<NodeBrand<R, S> as Functor>::map`. Switch to
   `<NodeBrand<R, S> as SendFunctor>::send_map`, calling the
   Send-aware `ArcFree` siblings from 9b. The struct-level HRTB
   stays (the `Of<'static, ArcFree<...>>: Send + Sync` bound is
   orthogonal to the trait bound on `R`).

   **9f. Switch `ArcRunExplicit` to `SendFunctor`-routed dispatch.**
   Same migration as 9e for
   [`fp-library/src/types/effects/arc_run_explicit.rs`](../../../fp-library/src/types/effects/arc_run_explicit.rs).

   **9g. Expand brand-level type-class surface on
   `ArcRunExplicitBrand`.** Step 4b documented the brand-level
   coverage as `SendPointed` only. With the SendFunctor-aware
   substrate machinery from 9c-9d, this expands to `SendPointed`
   plus whatever cascades from `ArcFreeExplicitBrand`'s expanded
   surface (sub-step 9d). Land the newly-reachable impls; document
   any remaining gap.

   **9h. Add `lift` inherent method to all six Run wrappers.** The
   originally-planned step 9 work, now unblocked for the Arc family
   by the 9a-9g cascade. Reference signature for `Run`:

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

   For the Erased Rc and Explicit families, the wrapper substitutes
   their own Coyoneda variant (`Coyoneda` for non-Arc; `ArcCoyoneda`
   for `ArcRun` / `ArcRunExplicit`) and the corresponding Member
   bound. Per-wrapper delta table:

   | Wrapper          | `'a`         | Coyoneda variant             | Extra `A` bound            |
   | :--------------- | :----------- | :--------------------------- | :------------------------- |
   | `Run`            | `'static`    | `Coyoneda<'static, _, _>`    | `A: 'static`               |
   | `RcRun`          | `'static`    | `Coyoneda<'static, _, _>`    | `A: 'static`               |
   | `ArcRun`         | `'static`    | `ArcCoyoneda<'static, _, _>` | `A: Send + Sync + 'static` |
   | `RunExplicit`    | `'a` (param) | `Coyoneda<'a, _, _>`         | `A: 'a`                    |
   | `RcRunExplicit`  | `'a` (param) | `Coyoneda<'a, _, _>`         | `A: 'a`                    |
   | `ArcRunExplicit` | `'a` (param) | `ArcCoyoneda<'a, _, _>`      | `A: 'a + Send + Sync`      |

   `Run::lift` already landed at commit
   [`34b6a97`](../../../fp-library/src/types/effects/run.rs); 9h
   adds the remaining five. `ArcRun::lift` may need the
   `lift_node` HRTB-fallback helper from the original 2026-04-28
   resolution; defer to the implementer's experience.

   **HRTB-poisoning fallback.** Try the inline body first on every
   wrapper. If `ArcRun::lift` fails to compile due to GAT-normalization
   recurring under `ArcFree`'s HRTB-bearing impl-block scope (the
   2026-04-27 limit), factor the literal-build step into a free
   helper outside the HRTB scope:

   ```rust
   pub fn lift_node<R, S, EBrand, Idx, A>(
       effect: Apply!(<EBrand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>),
   ) -> Apply!(<NodeBrand<R, S> as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>)
   where /* SendFunctor cascade bounds */
   {
       Node::First(<_ as Member<_, Idx>>::inject(ArcCoyoneda::lift(effect)))
   }
   ```

   Then `ArcRun::lift` calls
   `Self::send(lift_node::<R, S, EBrand, Idx, A>(effect))`. Don't
   pre-bake `lift_node` for the other five wrappers prophylactically.

   **Tests.** One integration test per wrapper at
   `fp-library/tests/run_lift.rs` covering: lift -> peel two-step
   round-trip (lower the inner Coyoneda's stored continuation, peel
   that to recover the value) on a single-effect row; second-branch
   injection through a multi-effect row (proves `Member` resolves
   the position correctly); inferred-`Idx` compiles unambiguously;
   `*Run::lift::<EBrand, _>(effect).bind(...)` composition. Erased
   Rc/Arc-family `peel` carries a row-projection `Clone` bound that
   Coyoneda-headed rows don't satisfy; substitute construction-only
   tests for those two wrappers.

   This is the "thin wrapper over `inj + liftF`/`send`"
   infrastructure that [decisions.md](decisions.md) section 6 names
   as the prerequisite for Phase 3's per-effect smart constructors
   (`ask`, `get`, `put`, `modify`, `tell`, `throw`). Each of those
   becomes a one-liner over `*Run::lift`, parallel to PureScript
   Run's `liftEffect = lift (Proxy :: "effect")` pattern.

   **9i. `SendRefFunctor` on `ArcRunExplicitBrand` via inherent-method
   delegation.** Step 4b documented the SendRef-family hierarchy on
   `ArcRunExplicitBrand` as unreachable through brand-level
   delegation because `ArcFreeExplicitBrand` doesn't implement it
   (per-`A` HRTB on `Kind` projection, unexpressible in trait method
   signatures). 9i sidesteps that gap with a different delegation
   strategy: implement `SendRefFunctor` on `ArcRunExplicitBrand` by
   calling the wrapper's inherent
   [`ref_map`](../../../fp-library/src/types/effects/arc_run_explicit.rs)
   directly, which uses the clone-trick
   (`self.clone().map(move |a| f(&a))`) to bypass the brand-level
   cascade. The `O(1)` `Arc::clone` makes this cheap; the inherent
   ref methods already handle the per-`A` constraints at the wrapper
   level.

   Reference shape:

   ```rust
   impl<R, S> SendRefFunctor for ArcRunExplicitBrand<R, S>
   where
       R: WrapDrop + SendFunctor + 'static,
       S: WrapDrop + SendFunctor + 'static,
   {
       fn send_ref_map<'a, A: 'a, B: 'a, Func>(
           f: Func,
           fa: &Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
       ) -> Apply!(<Self as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
       where
           Func: Fn(&A) -> B + Send + Sync + 'a,
       {
           fa.ref_map(f)
       }
   }
   ```

   Applies the same delegation pattern to other reachable
   SendRef-family traits (`SendRefSemimonad` via `ref_bind`,
   `SendRefPointed` via `ref_pure`) where the wrapper's inherent
   counterpart admits direct delegation. Document anything that
   doesn't admit delegation in
   [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md).
   `ArcRun` (the Erased family) has no brand, so its SendRef
   coverage stays inherent-method-only via
   [`im_do!(ref ArcRun { ... })`](../../../fp-macros/src/effects/im_do.rs).

10. Migrate the 25 row-canonicalisation tests from
    `poc-effect-row/tests/` into
    `fp-library/tests/run_row_canonicalisation.rs` as the
    regression baseline. Verify all pass under the production
    types (exercise both Erased and Explicit Run families).
    Delete the POC repository once the migration lands.

### Phase 3: First-order effect handlers, interpreters, natural transformations

1. `handlers!{...}` macro in
   `fp-macros/src/effects/handlers.rs` producing tuple-of-closures
   keyed on the row's type-level structure. Builder fallback
   (`nt().on::<E>(handler)...`) as the non-macro path
   ([decisions.md](decisions.md) section 4.6).
2. `interpret` / `run` simple all-handlers-at-once
   interpreter family on the six Run wrappers
   (`fp-library/src/types/effects/interpreter.rs`). M-free
   `while`-loop shape; handler closures produce `Run<R, CNilBrand, A>`
   directly; returns `A`. The `S` bound is fixed at
   [`CNilBrand`](../../../fp-library/src/brands.rs) so the
   `Node::Scoped(_)` arm is structurally uninhabited
   (`match cnil {}`) rather than a runtime panic; Phase 4
   ships a separate scoped-handler family without this bound.
   State threading is via user-side
   [`Rc<RefCell<S>>`](https://doc.rust-lang.org/std/cell/struct.RefCell.html)
   /
   [`Arc<Mutex<S>>`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
   closure captures applied directly to `interpret`; the
   pattern is documented on `interpret`'s rustdoc with the
   keyword "state" for rustdoc-search discoverability. No
   `run_accum` companion: the mono-in-A return type has no
   slot for threaded state to flow out, so a separate stateful
   interpreter would be a literal alias. State-via-`StateT s m`
   is deferred to Phase 6+; the closure-capture pattern
   minimises cognitive load and keeps the trait machinery thin
   (per the
   [2026-05-03 reversal resolution](resolutions.md#resolved-2026-05-03-adversarial-review-reversals-delete-run_accum-ship-interpret_with_rec-parameterise-interpret_with-over-refcountedpointer)).
3. Pipeline row-narrowing interpreter family:
   `interpret_with::<EBrand>(handler) -> Run<R_minus_E, CNilBrand, A>`
   per wrapper plus `extract(self) -> A` for empty-row Run.
   Enables partial interpretation, user-controlled handler
   ordering for non-commuting effects, and compositional
   handler libraries (heftia's primary mode + PureScript's
   `runPure` / `extract` analogs). The handler closure is
   parameterised over a pointer brand `P: RefCountedPointer`
   (using the existing
   [`ref_counted_pointer.rs`](../../../fp-library/src/classes/ref_counted_pointer.rs)
   trait): wrapped in `P::Of<F>` once at entry, then cloned
   (cheap refcount bump) on each recursive narrowing of a
   layer's content. The four non-Arc wrappers thread
   [`RcBrand`](../../../fp-library/src/brands.rs); the two Arc
   wrappers thread [`ArcBrand`](../../../fp-library/src/brands.rs).
   The user-facing handler bound is `Fn + 'static` (plus
   `Send + Sync` on Arc wrappers); no `Clone` requirement, so
   handlers may capture unique resources like a
   [`BufWriter`](https://doc.rust-lang.org/std/io/struct.BufWriter.html).
   Same `S = CNilBrand` bound as step 2; same rationale.
4. `interpret_rec` `MonadRec`-target interpreter family in the
   same module.
   `interpret_rec<MBrand: MonadRec>(handlers) -> M::Of<'_, A>`
   with `tail_rec_m`-driven loop. Externally-targeted form for
   extracting Run programs to
   [`Thunk`](../../../fp-library/src/types/thunk.rs) /
   [`Option`](../../../fp-library/src/types/option.rs) /
   [`Result`](../../../fp-library/src/types/result.rs) /
   [`Vec`](../../../fp-library/src/types/vec.rs) etc. with
   stack-safety guarantees. Per the
   [2026-05-02 resolution](resolutions.md#resolved-2026-05-02-phase-3-step-4-interpreter-design-handler-shape-dispatch-trait-reuse-state-threading),
   the locked-in design is:
   - **Handler shape mirrors PureScript:** input
     `Fn(<EBrand as Kind>::Of<'_, M::Of<'_, Run<R, CNilBrand, A>>>) -> M::Of<'_, Run<R, CNilBrand, A>>`.
     The interpreter does
     `<R as Functor>::map(M::pure, peel_layer)` to lift the
     peeled layer's `Run<R, CNilBrand, A>`-continuations to
     `M::Of<Run<R, CNilBrand, A>>`-continuations before dispatch.
     Reuses the existing
     [`DispatchHandlers<'a, Layer, NextProgram>`](../../../fp-library/src/types/effects/interpreter.rs)
     trait with `NextProgram = M::Of<Run<R, CNilBrand, A>>`.
     Handlers can do real `M::bind`-monadic work between effect
     dispatches (e.g., short-circuit on `M = ResultBrand` via
     `Result::bind`).
   - **Dispatch-trait relaxation as commit 1:** refactor
     [`DispatchHandlers::dispatch`](../../../fp-library/src/types/effects/interpreter.rs)
     from `&mut self` to `&self` (and `Handler::F: Fn` in the
     impl bounds) so the `tail_rec_m` step closure (`Fn`, not
     `FnMut`) can call dispatch on each iteration without
     cloning the handler list. All current handler closures
     across the codebase use interior mutability for state
     (closures Rust infers as `Fn`); the relaxation matches
     actual usage and adds zero per-iteration overhead. Lands
     as the first commit in this step (mechanical refactor).
   - **State threading via closure capture:** parallels step 2's
     pattern. State cells (`Rc<RefCell<S>>`, `Arc<Mutex<S>>`)
     live at the user level. No `run_accum_rec` companion (per
     the [2026-05-03 reversal resolution](resolutions.md#resolved-2026-05-03-adversarial-review-reversals-delete-run_accum-ship-interpret_with_rec-parameterise-interpret_with-over-refcountedpointer)).
   - **`S` bound is `CNilBrand`** so the `Node::Scoped(_)` arm
     is structurally uninhabited; Phase 4 ships a parallel
     scoped-handler `MonadRec` family without this bound.
   - **ArcRun reuses
     [`unwrap_first`](../../../fp-library/src/types/effects/arc_run.rs)**
     for the HRTB-poisoning workaround. The
     [`make_node_first`](../../../fp-library/src/types/effects/arc_run.rs)
     /
     [`wrap_first_arc`](../../../fp-library/src/types/effects/arc_run.rs)
     helpers from step 3 are not needed (no narrowed Run
     construction in scope; the rec form returns `M::Of<A>`
     directly).
   - Per-wrapper bounds cascade: `MBrand: MonadRec`; for the
     Arc wrappers, also `M::Of<'_, Run<...>>: Send + Sync`
     and the per-projection cascade.
   - Integration tests in
     `fp-library/tests/run_interpret_rec.rs` covering each
     wrapper x several `M` choices (`ThunkBrand`,
     `OptionBrand`, `ResultBrand`).
5. Standard first-order effect types and their smart
   constructors: `State<FnP, S>`, `Reader<FnP, E>`,
   `Except<E>`, `Writer<FnP, W>`, `Choose<FnP>`. Per the
   [2026-05-03 resolution](resolutions.md#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization),
   the locked-in design is:
   - **Effect types parameterised by
     [`FnBrand`](../../../fp-library/src/types/fn_brand.rs)**
     so a single effect type works across the `Box`/`Rc`/`Arc`
     substrate continuations. Each effect implements
     [`Functor`](../../../fp-library/src/classes/functor.rs)
     directly (and
     [`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
     for the Arc family) over its continuation parameter.
   - **Six per-wrapper smart-constructor variants per effect**
     (matching the precedent set by the per-wrapper interpreter
     families in steps 2-5). Smart constructors live in
     per-wrapper module paths (e.g.,
     `fp_library::types::effects::run::ask`,
     `fp_library::types::effects::rc_run::ask`, etc.) or as
     inherent methods on each wrapper.
   - **Choose ships on all four multi-shot wrappers** (`RcRun`,
     `RcRunExplicit`, `ArcRun`, `ArcRunExplicit`). Single-shot
     wrappers (`Run`, `RunExplicit`) cannot host Choose because
     the handler runs both branches and the continuation must
     be cloneable.
   - **Row-brand composition via the existing
     [`effects!`](../../../fp-macros/src/effects/effects_macro.rs)
     macro.** No per-effect type aliases (`type ReaderRow<E, R> = ...`)
     ship in this step; deferred until user demand surfaces.
   - Per-effect smart-constructor counts: State (`get` + `put`)
     - 6 wrappers = 12; Reader (`ask`) _ 6 = 6; Except
       (`throw`) _ 6 = 6; Writer (`tell`) _ 6 = 6; Choose
       (`choose`) _ 4 multi-shot wrappers = 4. Total ~34 named
       entry-points across six per-wrapper modules.
   - **`SendFunctorAt` spike for ArcRun State as a sub-task**
     (per the
     [2026-05-03 reversal resolution](resolutions.md#resolved-2026-05-03-adversarial-review-reversals-delete-run_accum-ship-interpret_with_rec-parameterise-interpret_with-over-refcountedpointer))
     to determine whether ArcRun State can ship without
     HRTB-over-types support. If the spike succeeds,
     `ArcRun::get` / `ArcRun::put` / `ArcRunExplicit::get` /
     `ArcRunExplicit::put` ship as part of this step. If the
     spike fails, demote the
     [Success criteria](#success-criteria) "State on every
     wrapper" claim to "State on every wrapper except Arc family
     (deferred pending HRTB-over-types in stable Rust)" and add
     a Phase 6+ deferred entry.
   - **This step may be split into sub-steps** per the
     [Implementation protocol](#implementation-protocol)'s
     oversized-step rule (~1500+ new lines, 7+ new files, or
     multiple new public types with mixed concerns); one
     effect per sub-step is the natural cut (5a State, 5b
     Reader, 5c Except, 5d Writer, 5e Choose). Surface the
     split decision to the user before starting.
6. **[DEFERRED 2026-05-04]** `define_effect!` macro at
   `fp-macros/src/effects/define_effect.rs` generating effect
   enum + smart constructors + label / brand registration.
   Mechanically would emit the six per-wrapper variants from a
   single user declaration:
   ```rust
   define_effect! {
       Reader<E> {
           fn ask() -> E,
       }
   }
   ```
   Deferred until Phase 4 (scoped effects) ships or a real user
   surfaces concrete demand for custom effects. Phase 4's brand
   shape may invalidate the codegen target; pre-1.0 substrate
   churn (e.g., the recent `RcCatList`/`ArcCatList` migration)
   would have forced an early-shipped macro to be migrated; and
   the library already covers the common cases with five hand-
   written effects, so the boilerplate savings only apply to
   effects that don't yet exist. Five design approaches and
   seven open questions are preserved in
   [resolutions.md](resolutions.md#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit)
   for revisit when either trigger fires.
7. `compile_fail` UI tests for negative cases (handler missing
   an effect, wrong type ascription, multi-shot via single-shot
   `Run`, `Choose` on single-shot wrappers).
8. Review-remediation documentation pass: bundle the docs-only
   items from
   [remediation_proposals.md](review/0_first_order_effects_implementation/remediation_proposals.md)
   into one commit. Lands after the substantive code work
   above so the docs reflect the settled state. Includes:
   - Add "callable continuation primitives in handler clauses
     (Plotkin-Pretnar `k`)" and "rank-2 natural transformations
     at the headline interpreter API" to the
     [Out of scope](#out-of-scope) section, with the
     freer-monad-encoding rationale and cross-links to
     [`NaturalTransformation`](../../../fp-library/src/classes/natural_transformation.rs)
     - [`Free::fold_free`](../../../fp-library/src/types/free.rs)
       as the rank-2 escape hatch.
   - Weaken the [Success criteria](#success-criteria)
     "single-shot vs. multi-shot" claim to apply to the Free
     wrapper's spine consumption only; per-effect closures
     (e.g., State's `dyn Fn` continuations) carry the
     multi-shot property at the effect-instance level on every
     wrapper.
   - Audit `<StateBrand as Functor>::map` call sites: verify
     it is only called once per dispatch (via Coyoneda
     lowering) and document Coyoneda fusion at the call site.
   - Add async-via-`spawn_blocking` workaround paragraph to
     interpreter rustdoc, cross-linking the Phase 6+
     Future-as-MonadRec deferred entry.
   - One-line rustdoc note on `bind` and on
     `DispatchHandlers::dispatch` mentioning the FnOnce-vs-Fn
     asymmetry.
   - All minor (m1-m9) findings per
     [remediation_proposals.md "Minor Findings"](review/0_first_order_effects_implementation/remediation_proposals.md#minor-findings).

### Phase 3.5: Pointer-brand-pattern retrofit

Lands between Phase 3 close and Phase 4 implementation kickoff. Introduces `ToDynFnOnce` to the existing pointer-abstraction trait family at [`fp-library/docs/pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md), retrofits the Phase 3 closure-bearing first-order effects (`State`, `Reader`, `Choose`) to use the new `BoxBrand` + `ToDynFnOnce` path on default `Run` / `RunExplicit`, and updates the documentation. Driven by Phase 4 design-question B3 (see [Phase 4 pre-implementation design questions](resolutions.md#resolved-2026-05-05-phase-4-pre-implementation-design-questions-b1-b4-q1-q3-q5-closed-by-design-adoption-commit-6e960701)) which surfaced that the user-supplied scoped-effect handlers in Phase 4 need FnOnce ergonomics on default `Run`, and that doing the retrofit before Phase 4 ships keeps Phase 3 + Phase 4 on a single consistent pointer-brand pattern.

1. `ToDynFnOnce` trait at [`fp-library/src/classes/to_dyn_fn_once.rs`](../../../fp-library/src/classes/) (new file) paralleling [`ToDynFn`](../../../fp-library/src/classes/to_dyn_fn.rs):
   ```rust,ignore
   pub trait ToDynFnOnce: Pointer {
       fn new<F, A, B>(f: F) -> <Self as Pointer>::Of<'_, dyn FnOnce(A) -> B>
       where F: FnOnce(A) -> B + 'static;
   }
   ```
   Implemented by [`BoxBrand`](../../../fp-library/src/brands.rs) only. `RcBrand` and `ArcBrand` do NOT implement it because `Rc<dyn FnOnce>` and `Arc<dyn FnOnce>` are operationally broken (`FnOnce::call_once` consumes `self`, which cannot be moved out of a shared pointer without invalidating other clones). The trait-family completion is therefore: `ToDynFn` (all brands), `ToDynFnOnce` (BoxBrand only), `ToDynCloneFn` (Rc, Arc), `ToDynSendFn` (Arc only).
2. Retrofit Phase 3 effects to use the new pattern. Each effect's per-wrapper smart constructors thread the appropriate pointer brand:
   - `State` ([`state.rs`](../../../fp-library/src/types/effects/state.rs)): `Run::get` / `Run::put` / `RunExplicit::get` / `RunExplicit::put` switch from `StateBrand<RcBrand, S>` to `StateBrand<BoxBrand, S>`. `RcRun` / `RcRunExplicit` smart constructors keep `StateBrand<RcBrand, S>`. `ArcRun` / `ArcRunExplicit` smart constructors keep `SendStateBrand<ArcBrand, S>` (`Arc<dyn Fn + Send + Sync>` via `ToDynSendFn`). The closure-storage cell type changes per-P: `BoxBrand` projects to `Box<dyn FnOnce>` via the new `ToDynFnOnce`; `RcBrand` projects to `Rc<dyn Fn>` via existing `ToDynCloneFn`; `ArcBrand` projects to `Arc<dyn Fn + Send + Sync>` via existing `ToDynSendFn`. `State`'s `Clone` impl becomes conditional: `State<BoxBrand, S, A>` is NOT `Clone` (used only on non-Clone `Run` substrates); `State<RcBrand, S, A>` and `SendState<ArcBrand, S, A>` are `Clone` (used on Clone-capable `Rc`/`Arc` substrates).
   - `Reader` ([`reader.rs`](../../../fp-library/src/types/effects/reader.rs)): same pattern, applied to `Reader::Ask`'s closure storage.
   - `Choose` ([`choose.rs`](../../../fp-library/src/types/effects/choose.rs)): same pattern, applied to `Choose::Alt`'s closure storage. Note: `Choose` smart constructors only ship on the four multi-shot wrappers per the [2026-05-03 wrapper-parameterization resolution](resolutions.md#resolved-2026-05-03-phase-3-step-5-smart-constructor-wrapper-parameterization), so `Choose<BoxBrand, A>` is structurally not used; the retrofit still defines it for substrate uniformity but no smart constructor exposes it.
   - `Writer` and `Except`: no closure cells; no retrofit needed.
3. Update [`fp-library/docs/pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md): add `ToDynFnOnce` to the trait diagram (under `Pointer` parallel to `ToDynFn`) and to the trait table; update the `BoxBrand` row in the brand-implementations table to add `ToDynFnOnce`. Add a brief section on FnOnce vs Fn variation documenting why `ToDynFnOnce` is `BoxBrand`-only.
4. Re-run integration tests for `State`, `Reader`, `Choose` to verify the retrofit preserves observable behaviour. Existing tests:
   [`fp-library/tests/run_state.rs`](../../../fp-library/tests/run_state.rs) (18 tests),
   [`fp-library/tests/run_reader.rs`](../../../fp-library/tests/run_reader.rs) (12 tests),
   [`fp-library/tests/run_choose.rs`](../../../fp-library/tests/run_choose.rs) (4 tests).
5. Phase 3 prior-review F4 finding (`dyn Fn` continuation on single-shot wrappers) is closed structurally rather than as accepted-tradeoff. Add a [resolutions.md](resolutions.md) entry describing the F4 closure plus the retrofit's design rationale.

Estimated total scope: ~500-700 lines across the three Phase 3 effect modules, smart-constructor signatures across the six Run wrappers per affected effect, the new `ToDynFnOnce` trait + `BoxBrand` impl, the documentation update, and the resolutions entry. Mechanical refactor; existing per-wrapper test coverage catches behaviour regressions.

### Phase 4: Scoped effects (heftia-inspired dual row)

> Heftia v0.7 itself ships a single effect list `es` with per-element `KnownOrder` markers, not literal dual rows; fp-library takes the **idea** (separate FO vs HO dispatch) and ships a value-level dual row (`Run<R, S, A>` with `Node<R, S> = First | Scoped`). The substrate-level mechanism (action stored as a Free-monad-encoded sub-program, handlers walk it via interpose-style rewrites) IS heftia v0.7's mechanism. See [decisions.md section 4.5](decisions.md#45-decision-scoped-effect-representation-via-a-heftia-inspired-dual-row) for the divergence framing.

0. **POC 3 validation: `interpret_with_either<EBrand, Idx>` substrate primitive on `RcRun`.**
   Standalone validation commit at
   [`fp-library/tests/poc_rc_run_interpret_with_either.rs`](../../../fp-library/tests/),
   paralleling POC 1
   ([`poc_send_catch_brand.rs`](../../../fp-library/tests/poc_send_catch_brand.rs))
   and POC 2
   ([`poc_rc_run_interpose.rs`](../../../fp-library/tests/poc_rc_run_interpose.rs)).
   Mechanical from
   [`interpret_with`'s body](../../../fp-library/src/types/effects/run.rs#L885-L900)
   with one branch substitution: the matched-effect arm
   short-circuits to `Right(op_value)` instead of calling a
   user handler. Test shape: build a program emitting an
   Identity effect followed by a Throw; call
   `interpret_with_either::<ExceptBrand, _>`; confirm the
   result is `Right(thrown_value)` rather than running
   through the FO Throw handler. Generic rollout across all
   six Run wrappers ships in step 2a after POC 3 validates;
   the `Catch` cons-cell impl in step 4 consumes the
   validated primitive. Adopted per the [K1 resolution](resolutions.md#resolved-2026-05-06-phase-4-implementation-kickoff-sequencing-k1-and-k2-poc-3-standalone-commit-first-planmd-numbering-authoritative-for-commit-boundaries).
   Conventional commit prefix: `test(effects)`. If POC 3
   surfaces a wall (e.g., HRTB-over-types friction at the
   Either-shaped return type), pause and revisit B4's Option
   A path with the placeholder-program-subtlety mitigation
   that surfaces (see [resolutions.md B4](resolutions.md#b4-catch-dispatchers-sentinel-mechanism)).

1. `ScopedCoproduct<ScopedEffects>` at
   `fp-library/src/types/effects/scoped.rs` with the dual-row
   integration into `Run<Effects, ScopedEffects, A>`.
2. Substrate-level `Run::interpose<EBrand, Idx>` primitive on
   each of the six Run wrappers. The Rust analogue of heftia's
   [`interposeInWith`](https://github.com/sayo-hs/heftia/blob/master/heftia/src/Control/Monad/Hefty/Interpret.hs):
   walks the Free tree, finds dispatches against `EBrand`,
   substitutes a user-supplied replacement, and re-emits in
   the same row (no row narrowing). Approximate signature:

   ```rust,ignore
   pub fn interpose<EBrand, Idx>(
       self,
       replacement: impl Fn(<EBrand as Kind>::Of<'_, Self>) -> Self + 'static,
   ) -> Self
   ```

   Implementation mirrors
   [`RcRun::interpret_with_shared`](../../../fp-library/src/types/effects/rc_run.rs)
   line for line, with the rebuilt layer's row staying in `R`
   instead of narrowing to `RMinusE`. POC-validated for the
   one-effect concrete row at
   [`fp-library/tests/poc_rc_run_interpose.rs`](../../../fp-library/tests/poc_rc_run_interpose.rs);
   the generic shape is mechanical from the POC template.
   `Run::interpose` is one of two substrate-level primitives
   that the `Catch` scoped-handler dispatcher composes. Shipping
   `Run::interpose` here closes the [Phase 6+ deferred `interpose`
   family entry](#phase-6-deferred-not-in-this-plan); the
   deferred entry is removed from Phase 6+ once Phase 4 step 2
   lands.

   2a. **`interpret_with_either<EBrand, Idx>` substrate
   primitive** on each Run wrapper, paralleling step 2's
   `Run::interpose`. Specialisation of
   [`interpret_with`](../../../fp-library/src/types/effects/run.rs#L885-L900)
   that returns `Either<A, EBrand::Op>` instead of narrowing
   the row, short-circuiting at the matched effect:

   ```rust,ignore
   pub fn interpret_with_either<EBrand, Idx>(
       self,
       fo_handlers: &impl DispatchHandlers<...>,
   ) -> Either<A, <EBrand as Kind>::Of<'_, Self>>
   ```

   Used by the `Catch` scoped-handler dispatcher (step 4) to
   structurally distinguish "action completed with `A`" from
   "action threw `EBrand::Op`" without resorting to interior
   mutability or a placeholder-program sentinel. Implementation
   mirrors `interpret_with` line for line with one branch
   substitution: the matched-effect arm returns `Right(op)`
   instead of calling a user handler. To be POC-validated
   ahead of R1 implementation as POC 3 at
   [`fp-library/tests/poc_rc_run_interpret_with_either.rs`](../../../fp-library/tests/),
   following the POC 1 / POC 2 conventions.

3. Standard scoped-effect constructors. Each closure-bearing
   constructor is parameterised by a pointer brand `P`
   selecting the closure storage shape, where `P` is one of
   the three established pointer brands at
   [`fp-library/docs/pointer-abstraction.md`](../../../fp-library/docs/pointer-abstraction.md):
   - `BoxBrand` (default substrate, single-shot): closure cell
     projects to `Box<dyn 'a + FnOnce(...)>` via the new
     [`ToDynFnOnce`](../../../fp-library/src/classes/) trait
     introduced in Phase 3.5; user-supplied scoped handlers
     can be FnOnce-natural.
   - `RcBrand` (Rc family, multi-shot non-thread-safe):
     closure cell projects to `Rc<dyn 'a + Fn(...)>` via
     existing
     [`ToDynCloneFn`](../../../fp-library/src/classes/to_dyn_clone_fn.rs);
     `Clone` is required because the Rc family's program is
     cloneable and cell sharing is needed.
   - `ArcBrand` (Arc family, multi-shot thread-safe): closure
     cell projects to `Arc<dyn 'a + Fn(...) + Send + Sync>`
     via existing
     [`ToDynSendFn`](../../../fp-library/src/classes/to_dyn_send_fn.rs).

   The parameterisation follows the Phase 3 `StateBrand<P, S>`
   pattern (now retrofitted in Phase 3.5 to use the same
   per-P closure-trait split). The closure-trait projection
   varies per `P`: `dyn FnOnce` for `BoxBrand`, `dyn Fn` for
   `RcBrand`, `dyn Fn + Send + Sync` for `ArcBrand`. Per
   [decisions.md](decisions.md) section 4.5 sub-decisions,
   `Bracket` and `Local` additionally ship two flavours each
   distinguished by the closure-argument shape (Val takes
   `A` by value; Ref takes `P::Of<A>` for refcounted
   sharing); `Catch` and `Span` ship Val-only (Ref flavours
   rejected per the sub-decision). For `ArcBrand`-family use
   each scoped-effect type gets a parallel `Send*Brand` sibling
   that bakes `+ Send + Sync` into the dyn bound at definition
   time, mirroring the Phase 3 [`SendStateBrand`](../../../fp-library/src/brands/effects.rs)
   pattern; POC-validated at [`fp-library/tests/poc_send_catch_brand.rs`](../../../fp-library/tests/poc_send_catch_brand.rs).
   The `Run<R, S, A>` rendering in the constructor signatures
   below is loose notation: the underlying Rust type is `A`
   (the next-program type bound by the dispatch context),
   mirroring how Phase 3 [`State<'a, P, S, A>`](../../../fp-library/src/types/effects/state.rs)
   uses `A` for the next-program type rather than a literal
   `Run<R, S, ?>`.

   Each closure-bearing scoped-effect type also implements the
   substrate-required traits `Functor` / `SendFunctor` /
   `WrapDrop` / `RefFunctor` / `Extract` (mirroring Phase 3's
   per-effect impls) so the substrate's `NodeBrand<R, S>`
   composition at [node.rs:84-449](../../../fp-library/src/types/effects/node.rs#L84-L449)
   works through `S` containing scoped-effect brands. The
   impls are mechanical: `Functor::map<X, Y>(f, fa)` walks the
   constructor's payload and applies `f` to the next-program
   slots, composing `f` onto stored closures via the brand's
   per-P trait (`ToDynFnOnce::new` for BoxBrand, etc.).
   `Catch` is the implementation template; subsequent scoped-
   effect impls follow the same shape:

   ```rust,ignore
   impl<E: 'static> Functor for CatchBrand<BoxBrand, E> {
       fn map<'a, A: 'a, B: 'a>(
           f: impl 'a + FnOnce(A) -> B,
           fa: Catch<'a, BoxBrand, E, A>,
       ) -> Catch<'a, BoxBrand, E, B> {
           let Catch { action, handler } = fa;
           Catch {
               action: f(action),
               handler: <BoxBrand as ToDynFnOnce>::new(
                   move |e: E| f(handler(e)),
               ),
           }
       }
   }
   ```

   `RcBrand` and `ArcBrand` parallels swap `BoxBrand` /
   `ToDynFnOnce` for `RcBrand` / `ToDynCloneFn` and
   `ArcBrand` / `ToDynSendFn` respectively, with `Fn` / `Fn +
Send + Sync` closure shapes; the body is structurally
   identical because the brand's per-P trait abstracts the
   coercion. `Local` / `RefLocal` map the `action` slot only
   (the `modify` closure has shape `E -> E` and is untouched
   by `f`); `Bracket` / `RefBracket` map the `acquire` slot
   plus the `body`'s return-program post-composition; `Span`
   maps `action` only.
   - `Catch<'a, P, E, A>` for `Error.catch`, with
     `action: Run<R, S, A>`,
     `handler: <P>::Of<'a, dyn 'a + ClosureTrait(E) -> Run<R, S, A>>`
     where `ClosureTrait` is `FnOnce` for `P = BoxBrand`,
     `Fn` for `P = RcBrand`, `Fn + Send + Sync` for
     `P = ArcBrand`. Val only.
   - `Local<'a, P, E, A>` (Val flavour) for `Reader.local`
     with a consuming modify, holding
     `modify: <P>::Of<'a, dyn 'a + ClosureTrait(E) -> E>`,
     `action: Run<R, S, A>`.
   - `RefLocal<'a, P, E, A>` (Ref flavour) for `Reader.local`
     with a borrowing modify, holding
     `modify: <P>::Of<'a, dyn 'a + ClosureTrait(&E) -> E>`,
     `action: Run<R, S, A>`. Removes the need to consume or clone
     the parent environment while deriving the sub-scope
     environment; repeated by-value `Reader::Ask` operations may
     still require `E: Clone` until a borrow-oriented Reader ships.
   - `Bracket<'a, P, A, B>` (Val flavour), with
     `acquire: Run<R, S, A>`,
     `body: <P>::Of<'a, dyn 'a + ClosureTrait(A) -> Run<R, S, (A, B)>>`,
     `release: <P>::Of<'a, dyn 'a + ClosureTrait(A) -> Run<R, S, ()>>`.
     The body consumes `A`, threads it back to the interpreter
     via `(A, B)`, and the interpreter moves the returned `A`
     into `release`. **Panic safety:** the bracket dispatcher
     runs the effectful `release` program on the normal path after
     `body` returns. During panic/unwind, the library only promises
     ordinary Rust resource `Drop` cleanup for the acquired resource
     and does not claim to interpret the effectful release program
     from `Drop`.
   - `RefBracket<'a, P, A, B>` (Ref flavour), with
     `acquire: Run<R, S, A>`,
     `body: <P>::Of<'a, dyn 'a + ClosureTrait(P::Of<A>) -> Run<R, S, B>>`,
     `release: <P>::Of<'a, dyn 'a + ClosureTrait(P::Of<A>) -> Run<R, S, ()>>`.
     Body and release both receive a pointer clone; the
     resource lives until the last clone drops, mirroring
     PureScript's GC-aliased `bracket` semantics
     ([`Aff.purs:308`](https://github.com/purescript-contrib/purescript-aff/blob/master/src/Effect/Aff.purs#L308)).
     Same normal-path effectful release and panic-path resource
     `Drop` semantics as the Val flavour.
     `RefBracket` requires `P` to be a refcounted brand
     (`RcBrand` for `RcRun` / `RcRunExplicit`, `ArcBrand` for
     `ArcRun` / `ArcRunExplicit`); `BoxBrand` does not satisfy
     `P::Of<A>: Clone` and is rejected at the type level.
   - `Span<'a, Tag>`, with `tag: Tag`, `action: Run<R, S, A>`.
     Val only (no closure to dispatch over; no `P` parameter
     needed).

   **Closure-storage ceiling on default `Run`.** User-defined
   scoped effects requiring multi-shot bodies (Coroutine,
   Provider, Retry, scoped NonDet with backtracking) are not
   expressible on the default `Run` and `RunExplicit`
   wrappers because `BoxBrand`'s `Box<dyn FnOnce>` storage is
   single-shot by construction. Users requiring multi-shot
   scoped bodies must use `RcRun` / `ArcRun` / `RcRunExplicit`
   / `ArcRunExplicit`, whose `RcBrand` / `ArcBrand` storage
   projects to multi-shot `Rc<dyn Fn>` / `Arc<dyn Fn + Send + Sync>`
   cells via `ToDynCloneFn` / `ToDynSendFn`. The standard
   scoped set (`Catch`, `Local`, `Bracket`, `Span`) is
   single-shot on every wrapper and is therefore unaffected.
   Lifting the multi-shot ceiling on the default `Run` family
   would require introducing FTCQueue-style scoped
   continuations as a substrate addition; this is deferred to
   a v2 surface and is not part of Phase 4.

4. `DispatchScopedHandlers` trait at
   [`fp-library/src/types/effects/interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs)
   parallel to the Phase 3
   [`DispatchHandlers`](../../../fp-library/src/types/effects/interpreter.rs)
   trait. Approximate signature:

   ```rust,ignore
   pub trait DispatchScopedHandlers<'a, ScopedLayer, FOLayer, NextProgram> {
       fn dispatch_scoped(
           &self,
           layer: ScopedLayer,
           fo_handlers: &impl DispatchHandlers<'a, FOLayer, NextProgram>,
       ) -> NextProgram;
   }
   ```

   The production API uses argument-position `impl DispatchHandlers`
   rather than a named `FOH` parameter. This is still a method-generic,
   statically-dispatched shape; it just follows the repository's
   documented preference for anonymous generics when the type parameter
   is not referenced elsewhere in the signature.

   Each scoped-effect cons-cell impl receives the FO handler list
   `fo_handlers`. For `Catch` specifically, the original step 4 design
   used the `interpret_with_either` substrate primitive from step 2a.
   B27 later kept that primitive first-order-only for `S = CNilBrand`
   and adopted scoped-row-preserving `interpose` for the standard
   scoped Catch dispatcher, because `interpose` returns a program and
   can therefore carry preserved `Node::Scoped` layers:

   ```rust,ignore
   fn dispatch_scoped(
       &self,
       catch: Catch<...>,
       _fo_handlers: &impl DispatchHandlers<'a, FOLayer, NextProgram>,
   ) -> NextProgram {
       catch.action.interpose::<ExceptBrand<E>, _, _, _>(
           |Except::Throw(e, _)| (catch.handler)(e),
       )
   }
   ```

   The first-order-only `interpret_with_either` primitive remains
   POC-validated via POC 3 (see step 2a), but it is not the standard
   scoped Catch path. Each Run wrapper's `interpret` grows a second
   handler-list parameter; the loop dispatches `Node::First` to the existing
   `DispatchHandlers::dispatch` and `Node::Scoped` to the new
   `DispatchScopedHandlers::dispatch_scoped`, threading the FO
   handler list along. The Phase 3 `S = CNilBrand` tightening
   on each wrapper's `interpret` family (per the
   [2026-05-03 F3A reversal](resolutions.md#resolved-2026-05-03-adversarial-review-reversals-delete-run_accum-ship-interpret_with_rec-parameterise-interpret_with-over-refcountedpointer))
   is lifted; the new trait subsumes the role.

5. `scoped_effects!` macro for assembling the scoped row at
   the type level, sharing the lexical-sort canonicalisation
   helper with Phase 2's `effects!` (one helper, thin
   entry-point macros, distinct output shapes: Coyoneda-wrapped
   `CoproductBrand` for FO vs unwrapped `CoproductBrand` for
   scoped rows). Plus a
   `scoped_handlers!{...}` companion to the Phase 3
   [`handlers!`](../../../fp-library/src/types/effects/handlers.rs)
   macro for assembling the second list passed to `interpret`.
   `scoped_handlers!` and the existing `handlers!` macro share
   their parser, lexical sort, and cons-list emission helper,
   parameterised by cell type identifier (`Handler<E, F>` for
   FO; `ScopedHandler<S, F>` for scoped) and cons-list cell-and-
   tail type identifiers (`HandlersCons` / `HandlersNil` for FO;
   `ScopedHandlersCons` / `ScopedHandlersNil` for scoped).
   Both macros become thin entry-point wrappers over the
   helper. Row macros share the `effects!` lexical-sort helper;
   handler macros carry expressions, so they share the same
   lexical-sort strategy inside the handler-list parser/emitter.

5b. `define_scoped_row!` item-position macro for named scoped
marker rows, adopted by B23. This macro is separate from
`scoped_effects![...]` because the latter expands in type
position and cannot emit the named marker struct plus trait
impls required by the B18 Bracket-row workaround. Initial
syntax is concrete only:

```rust,ignore
define_scoped_row! {
    pub struct MyScopedRow;
    [
        BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, Self>, i32, i32>,
        BoxCatchBrand<BoxBrand, MyError>,
    ]
}
```

The macro substitutes bare `Self` type placeholders with the
generated row marker before lexical sorting, then generates
the marker struct plus delegating `Kind`, `WrapDrop`,
`Functor`, `SendFunctor`, and `RefFunctor` impls. The shipped
by-value row-trait delegates (`WrapDrop`, `Functor`,
`SendFunctor`) intentionally avoid explicit underlying-row
where-clauses so recursive marker rows keep the B18 POC's
accepted proof shape; `RefFunctor` remains guarded by the
underlying row's `RefFunctor` bound so Send-only scoped rows are
not forced to provide by-reference mapping. Generic scoped
rows are deferred to a later step and should be revisited only
when a concrete standard-handler or custom-effect use case
requires row type parameters. B20 remains tracked separately:
this macro removes B18/B23 marker boilerplate but does not
itself redesign the `ArcRun::bracket` Send+Sync overflow path.

`define_scoped_effect!` macro is **deferred to Phase 6+**
in parallel with the [Phase 3 `define_effect!` deferral](resolutions.md#resolved-2026-05-04-phase-3-step-6-define_effect-macro-deferred-until-phase-4-ships-or-user-demand-surfaces-design-research-preserved-for-later-revisit)
per design-question Q2 (see [Phase 4 pre-implementation
design questions](resolutions.md#resolved-2026-05-05-phase-4-pre-implementation-design-questions-b1-b4-q1-q3-q5-closed-by-design-adoption-commit-6e960701)).
Users defining their own scoped effects hand-write the
constructor type plus four to six trait impls (mirroring
Phase 3's standard effect definitions). Revisit triggers
parallel the existing `define_effect!` deferred-item
triggers.

6. Smart constructors: `catch`, `span` (single-flavour
   wrappers); `bracket` and `local` (closure-driven dispatch over
   Val and Ref flavours, reusing the existing `Val` / `Ref`
   markers and dispatch machinery from
   [`fp-library/src/dispatch/`](../../../fp-library/src/dispatch/);
   `bracket`'s Ref impl additionally carries the pointer brand
   `P` so `Ref<RcBrand>` and `Ref<ArcBrand>` resolve to distinct
   `RefBracket` node types). Concretely:
   - `BracketDispatch<R, S, A, B, Marker>` trait with `Val` impl
     (closures of shape `FnOnce(A) -> Run<R, S, (A, B)>` plus
     `FnOnce(A) -> Run<R, S, ()>`) and `Ref<P>` impls for each
     `P: RefCountedPointer` (closures of shape
     `FnOnce(P::Of<A>) -> Run<R, S, B>` plus
     `FnOnce(P::Of<A>) -> Run<R, S, ()>`).
   - `LocalDispatch<R, S, E, A, Marker>` trait with `Val` impl
     (`FnOnce(E) -> E`) and `Ref` impl (`FnOnce(&E) -> E`).
   - The dispatch traits and their impls live at
     `fp-library/src/dispatch/run/bracket.rs` and
     `fp-library/src/dispatch/run/local.rs`, mirroring the
     existing layout described in
     [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md).
     No `mask` smart constructor in v1; the `Mask` constructor is
     deferred per [decisions.md](decisions.md) section 4.5
     sub-decisions.

6a. Scoped-row-preserving primitive retrofit (B24), required before
standard scoped dispatchers:

- Implement per-wrapper
  `interpret_scoped_with::<EBrand, Idx, SMinusE>(scoped_handler)`
  as the scoped-row analogue of `interpret_with`.
- Generalise `interpret_with` and `interpose` to preserve non-empty
  scoped rows by recursively mapping `Node::Scoped`, or ship
  explicitly named scoped-aware sibling methods if generalising the
  existing method names creates inference regressions.
- Leave `interpret_with_either` as the first-order-only primitive for
  `S = CNilBrand` from Phase 4 step 2a. Do not generalise it in step
  6a; the richer scoped-aware short-circuit carrier is deferred to
  Phase 6+ per B27.
- Land a small proof commit on `Run` and `RcRun` first, then
  fan out to `ArcRun`, `RunExplicit`, `RcRunExplicit`, and
  `ArcRunExplicit`, reusing the established Arc-family
  HRTB-poisoning helper pattern where needed.
- Add focused tests showing that first-order row narrowing and
  interpose-style replacement preserve nested scoped operations
  rather than requiring `S = CNilBrand`.

7.  Standard scoped-handler implementations as a parallel
    set of `DispatchScopedHandlers` cons-cell impls, NOT as
    extensions to the existing FO `run_reader` / `run_except`
    handlers. Phase 4 ships `LocalDispatcher`,
    `RefLocalDispatcher`, `CatchDispatcher`, `BracketDispatcher`
    (Val and Ref<P>), `SpanDispatcher` impls. Both the FO and
    scoped handler lists are passed to `interpret` together via
    the unified two-list form (per Phase 4 step 4's
    `DispatchScopedHandlers` trait spec). The FO and scoped
    handlers may share state via interior-mutability captures
    (the Phase 3 closure-capture convention) but do not share
    types. Pipeline ordering (which row to narrow first) is
    user-driven: callers writing `interpret_scoped_with::<EBrand>`
    sequence FO and scoped narrowing as their handler interactions
    require.

    B29 production architecture guidance:
    - Adjust wrapper `interpret` / `run` scoped-handler bounds to each
      wrapper's actual peeled-layer lifetime before standard dispatcher
      impls land: `'static` for erased wrappers and the wrapper lifetime
      `'a` for Explicit wrappers. Keep first-order handler bounds
      higher-ranked unless implementation proves they also need
      narrowing.
    - Interpose-backed dispatchers carry row-removal evidence:
      `CatchDispatcher<Idx, RMinusE, EmbedIndices>`,
      `LocalDispatcher<Idx, RMinusE, EmbedIndices>`, and
      `RefLocalDispatcher<Idx, RMinusE, EmbedIndices>`.
    - Add helper constructors (for example
      `catch_dispatcher::<Idx, RMinusE, EmbedIndices>()`) so examples
      and user code do not need to name witness-bearing dispatcher
      structs directly.
    - `SpanDispatcher` remains witness-free. `BracketDispatcher` and
      `RefBracketDispatcher` remain result-specific because their brands
      include `A` / `B`.

    Adopted step 7 implementation split:
    - **7.1 CatchDispatcher and SpanDispatcher.** Implement
      `CatchDispatcher` with the scoped-row-preserving
      `interpose::<ExceptBrand<_>, _, _, _>` path from step 6a for
      Rc/Arc wrappers and the B30 continuation-aware raw scoped-step
      path for default `Run`. The dispatcher replaces each matched
      `Except::Throw(e, _)` in the action with the catch handler result
      and preserves nested scoped operations. Tests must confirm nested
      scoped actions survive and throws from the recovery handler are
      not caught by the same `Catch` frame. Implement `SpanDispatcher`
      as an around-action dispatcher that observes the by-value tag and
      returns the action result unchanged.
    - **7.2 LocalDispatcher and RefLocalDispatcher (B25 Option A).**
      Implement Local / RefLocal by scoped-row-preserving Reader
      interposition. The dispatcher obtains the current environment,
      computes the modified environment through `E -> E` or
      `&E -> E`, and answers `Reader::Ask` inside the action with the
      modified environment. Because the existing first-order Reader
      operation is by-value, repeated asks require `E: Clone`; this is
      a v1 constraint of the current Reader shape, not a RefLocal
      `modify` constraint. RefLocal's guarantee is that deriving the
      modified environment borrows the parent `E` instead of consuming
      or cloning it. True no-clone environment access across repeated
      asks is deferred as the borrow-oriented Reader follow-up in
      [Phase 6+](#phase-6-deferred-not-in-this-plan). Shipped
      implementation note: default `Run` must rebox the raw-erased
      action before ordinary `interpose` walks it, then use
      `Free::continue_from_reboxed_erased` when reattaching the
      caller's continuation queue. Arc-family `interpose` /
      `interpret_with` now require `SendFunctor` on the targeted
      effect brand rather than both `Functor` and `SendFunctor`, which
      is what lets `SendReaderBrand` participate in Local dispatch.
    - **7.3 BracketDispatcher and RefBracketDispatcher (B26 Option
      A).** Implement normal-path sequencing as acquire -> body ->
      effectful release, returning the body result after release runs.
      Shipped implementation note: the public `bracket` constructors
      now return `B`; the Val body closure still returns `(A, B)`
      internally so the dispatcher can hand `A` to `release` before
      returning `B`.
      On panic/unwind, guarantee only ordinary Rust resource `Drop`
      behavior for the acquired resource and any synchronous cleanup
      encoded in the resource itself; do not claim to interpret the
      effectful release program from `Drop`. This keeps effectful
      release in the public Bracket API and avoids the B26B rewrite to
      synchronous-only release closures. A separate synchronous
      panic-finalizer hook is deferred to [Phase 6+](#phase-6-deferred-not-in-this-plan)
      if users need more than resource `Drop` during unwind.
    - **7.4 Internal continuation carrier for around-action scoped
      handlers (B31 Option B, B32 H2 via B35 Option A).** Add a
      scoped-handler path for handlers that must observe an action
      before and after the action is interpreted, rather than merely
      returning a next program that still contains the action's nested
      scoped suspensions. B35 promotes H2 into this Phase 4 step:
      wrapper interpreters own a private continuation carrier when they
      peel a scoped suspension, and handlers use that carrier to resume
      the active branch normally or insert result-preserving post-action
      behavior before the branch's outer continuation. H3 (separate
      public protocol families) remains deferred to Phase 6+ as a
      possible facade over the H2 carrier if custom handler APIs later
      need explicit handler classes.

           - **7.4.1 Prototype the continuation boundary.** Reapply or
           recreate the preserved B31 nested-Span experiment from the named
           stash `wip(effects): span nested lifecycle ordering experiment`.
           Reduce it to the smallest proof that a handler can record
           `enter outer, enter inner, exit inner, exit outer` while
           preserving the action result.

           - **7.4.2 Design the H2 carrier contracts (shipped).** Added the
           private carrier vocabulary in the scoped interpreter substrate:
           `ScopedContinuation` for the wrapper-owned handle and
           family-specific private resume traits for the static-dispatch
           resume contracts. The scaffold supports normal resume,
           result-preserving post-action continuation insertion before the
           branch's outer continuation is reattached, and carrier
           extraction for wrapper-local rewrites while retaining
           first-order handler access without owning or cloning handler
           lists. `ScopedResumeTypes::ActionValue` is the action result
           type passed to the post-action continuation, and
           `ScopedResumeTypes::ActionProgram` is the program returned by
           that continuation (`TypeErasedValue -> RawRunFree` for default
           `Run`, typed result -> typed program for wrapper families that
           do not erase the branch at this boundary). The invariants
           remain: no unsafe non-`'static` erasure, no public wrapper API
           change unless a compiler proof shows it is unavoidable,
           ordinary handlers remain on the existing static-dispatch path,
           and H3-style public facades are deferred.

           - **7.4.2a Prove the carrier on default `Run` (shipped).** Added
           `RunScopedContinuation` over the existing erased `Free`
           raw-step / continuation-queue machinery. Focused tests prove
           ordinary resume through an outer continuation and post-action
           raw-continuation insertion before the outer continuation queue.

           - **7.4.2b Prove the carrier on `RunExplicit` (B36 Option A
           adopted).** Do not implement this as `action.bind(post_action)`:
           current `FreeExplicit::bind` has already pushed the outer
           continuation into scoped action branches by the time
           interpretation sees the scoped layer.

           - **7.4.2b.0 Prototype an existential-safe Explicit boundary
           (shipped).**
           Build the smallest private `FreeExplicit` / `RunExplicit`
           proof that exposes the action value and the action's outer
           continuation separately without `Any`, unsafe erasure, or
           dyn-generic callbacks. Preserve existing public behavior and
           non-`'static` payload support. The prototype may be private
           test code or a narrowly scoped internal helper, but it must
           prove post-action insertion occurs before the outer
           continuation.

           - **7.4.2b.1 Implement `RunExplicit` carrier on the proven
           boundary (shipped).** Add the private carrier type implementing
           `ExplicitScopedResume` for `RunExplicit`. Cover ordinary resume,
           post-action insertion before outer continuation, result
           propagation, and a borrowed-payload regression.

           - **7.4.2b.2 Fallback trigger (not active).** If 7.4.2b.0
           still requires a callback generic over a hidden intermediate
           type, stop this implementation line and promote the H3
           protocol-family path as the next concrete step. Do not ship a
           `bind`-based carrier as a temporary compatibility shim.

           - **7.4.2c Split the private carrier protocol by wrapper family
           and extend the carrier to `RcRun` / `RcRunExplicit` (B37
           Option A adopted).** Preserve multi-shot semantics, O(1)
           program clone assumptions, and existing `A: Clone` /
           layer-`Clone` bounds. Do not normalize Rc-specific projection
           obligations through dynamic-dispatch closure storage unless the
           family-indexed static protocol proves impossible to thread
           through the private dispatcher path.

           - **7.4.2c.0 Add family-specific private resume traits
           (shipped).** Keep
           `ScopedContinuation` as the shared wrapper-owned handle, but
           split the resume implementation contract into private traits
           for default erased, single-shot Explicit, Rc-shared, and
           Arc-shared carriers. Each trait carries the bounds required by
           its substrate, including Rc row-projection `Clone` bounds and
           later Arc `Send + Sync` projection bounds. The split must
           remain private to the interpreter substrate and must not
           change public wrapper APIs.

           - **7.4.2c.1 Migrate shipped carriers to the family traits
           (shipped).**
           Move `RunScopedContinuation` onto the default-erased trait
           and `RunExplicitScopedContinuation` onto the single-shot
           Explicit trait. Preserve the already-shipped tests for normal
           resume, post-action insertion before the outer continuation,
           and borrowed Explicit action values.

           - **7.4.2c.2 Implement the `RcRun` carrier on the Rc-specific
           trait (shipped).** Reapply or recreate the useful pieces of the
           preserved B37 prototype stash, but put the Rc projection
           `Clone` bounds on the Rc-family trait/impl boundary instead
           of the shared carrier contract. Add tests for repeated resume,
           repeated post-action insertion, and nested continuation
           insertion.

           - **7.4.2c.3 Implement the `RcRunExplicit` carrier on the
           Rc-specific Explicit trait (shipped).** Preserve non-`'static` borrowed
           payload support, cloneable shared-program semantics, and
           post-action insertion before the outer continuation. Add a
           borrowed-payload regression plus repeated-use coverage.

           - **7.4.2c.4 Document any fallback before adopting it.** If the
           family-indexed static protocol still cannot express the Rc
           obligations without public API churn, pause and record the
           concrete compiler error. Option B's pre-bound closure carrier
           is the fallback only after that proof fails.

           - **7.4.2d Extend the carrier to `ArcRun` and `ArcRunExplicit`
           using the Arc-specific private trait (shipped).** Preserved
           `Send + Sync` propagation and the existing `SendFunctor` mapping
           path. Arc projection obligations live on the Arc-family
           trait/impl boundary rather than the Rc trait or the old common
           resume contract. Tests cover `Send + Sync` bounds, repeated
           shared resume/post-action use, and continuation insertion before
           outer continuations.

           - **7.4.3 Add the carrier-aware scoped-handler path (shipped).** Extend
           the scoped-handler substrate in
           [`interpreter.rs`](../../../fp-library/src/types/effects/interpreter.rs)
           with the private H2 carrier route for around-action handlers.
           The path lets standard handlers request normal resume or
           result-preserving post-action insertion without recursively
           interpreting actions through borrowed handler lists. It walks
           `SBrand::Of<ActionProgram>` layers with a typed
           `ScopedContinuation` and keeps the ordinary
           `DispatchScopedHandler` / `DispatchScopedHandlers` path intact
           for non-around-action handlers.

           - **7.4.4 Wire the wrapper interpreters (B38 Option C activated
           for Explicit-family wrappers by B39).** Thread the carrier path
           through `Run`, `RunExplicit`, `RcRun`, `ArcRun`,
           `RcRunExplicit`, and `ArcRunExplicit` without weakening
           existing `DispatchScopedHandlers` support for ordinary scoped
           handlers. Default `Run` and erased shared wrappers use the
           shipped private raw-step extraction boundaries. Explicit-family
           wrappers use constructor-stored carrier shapes because their
           substrates recursively map `bind` continuations into suspended
           layers before interpretation can recover an action/outer split.
           If a wrapper cannot host the carrier without public API churn,
           document the concrete compiler error before changing the public
           surface.

           - **7.4.4a Add private carrier raw-step extraction for default
           and erased shared wrappers (shipped).** Reused default `Run`'s
           existing `Free::into_raw_step`; added `RcFreeRawStep` /
           `ArcFreeRawStep` plus `continue_from_erased` helpers so
           Rc/Arc raw-step views keep `RcCatList` / `ArcCatList`
           continuation queues outside the selected scoped action instead
           of reattaching them inside `to_view` / `resume`.

           - **7.4.4b Add the carrier-backed ownership shape for scoped
           effects (B39 Option B, B42 Option B first).** Preserve the
           typed action/outer boundary before Explicit `bind`
           distributes the outer continuation. B40 found that the
           carrier path must preserve the selected action type while
           `Functor::map` changes the final next-program slot. B41
           adopts Option D: do not move the selected action type into the
           `'static` row brand. B42 refines the production ownership
           model: the scoped operation owns the selected action and the
           continuation carrier owns only the outer resume boundary,
           with a carrier-cell scoped-layer fallback if Explicit mapping
           proves that split impossible.

           - **7.4.4b.0 Define the Span-first two-slot around-action
           protocol (shipped).** Added the private protocol vocabulary needed to
           pass the selected action program/value and the final
           next-program as separate typed boundaries. The action
           boundary must remain in a method/GAT position so borrowed
           Explicit payloads stay valid; the final-program boundary
           remains the ordinary mapped result slot. Keep the protocol
           separate from the one-slot `DispatchScopedHandlers` route.

           - **7.4.4b.1 Specify the row and macro spelling for
           around-action handlers (shipped).** Exercised how `Span` appears
           through `scoped_effects!` and `define_scoped_row!` when the
           handler needs the two-slot carrier protocol. The ordinary
           static `BoxSpanBrand<BoxBrand, Tag>` spelling projects to
           the borrowed action slot through the
           `SBrand::Of<'a, ActionProgram>` projection, so the B41
           static-witness fallback did not trigger at the
           macro-spelling checkpoint.

           - **7.4.4b.2 Prove the Span stored-carrier interpreter path
           (B42 Option C fallback active via B43).** The proof gate
           showed that current `RunExplicit::bind` maps the `BoxSpan`
           action slot to the final program before interpretation. The
           preferred outer-only continuation model cannot recover the
           selected action from that mapped layer. Continue with a
           carrier-cell scoped-layer shape for the Explicit-family Span
           path.

           - **7.4.4b.2a Prove the outer-only continuation carrier on
           `RunExplicit` Span (shipped; fallback selected).** Focused
           coverage now proves that `RunExplicit::span(...).bind(...)`
           exposes a mapped `BoxSpan` action slot returning the final
           program, not the selected action program. This selects the
           carrier-cell fallback rather than the outer-only carrier.

           - **7.4.4b.2b Define the private carrier-cell Span layer
           shape for `RunExplicit` (shipped).** Added the private
           `RunExplicitSpanCarrierLayer` helper shape that stores the
           Span tag plus the `RunExplicit` carrier cell before
           dispatcher wiring consumes it. Focused tests prove the
           shape stores the tag, preserves borrowed selected action
           values, and resumes post-action work before the outer
           continuation. Ordinary `BoxSpan` and ordinary
           `DispatchScopedHandlers` behavior remain unchanged.

           - **7.4.4b.2c Wire `RunExplicit` Span through the
           carrier-cell dispatcher path (shipped).** Added the focused
           private `SpanDispatcher` path that consumes
           `RunExplicitSpanCarrierLayer`, observes the Span tag around
           the selected action, inserts post-action work before the
           outer continuation, preserves borrowed selected payloads,
           and returns the final next-program type.

           - **7.4.4b.2d Extend the carrier-cell Span proof to shared
           Explicit wrappers.** After the single-shot `RunExplicit`
           proof holds, extend the same shape to `RcRunExplicit` and
           `ArcRunExplicit`, preserving repeated shared resume for Rc,
           `Send + Sync` obligations for Arc, and borrowed selected
           action payload coverage.

           - **7.4.4b.3 Retrofit the remaining standard around-action
           effects.** Apply the approved two-slot protocol to the
           carrier-backed forms of `Catch`, `Local`, `RefLocal`,
           `Bracket`, and `RefBracket` only after the Span proof holds.
           Preserve ordinary `DispatchScopedHandlers` support for
           handlers that do not need an around-action carrier. Keep
           existing public smart-constructor inputs stable where
           possible; document any required row-brand spelling change as
           a deviations entry.

           - **7.4.4b.4 Add focused coverage.** Cover non-`'static`
           Explicit payloads, borrowed action values, repeated shared
           resume for Rc/Arc Explicit wrappers, `Functor::map` changing
           the final-program slot without losing the selected action
           boundary, the two-slot macro spelling, and preservation of
           the ordinary `DispatchScopedHandlers` route for handlers
           that do not need an around-action carrier.

           - **7.4.4c Wire the six wrapper interpreters.** Dispatch scoped
           layers through `DispatchScopedCarrierHandlers` when an
           around-action carrier is required, using private raw-step
           extraction for default/erased shared wrappers and
           the approved carrier-backed shape for Explicit-family wrappers,
           while preserving the existing ordinary
           `DispatchScopedHandlers` route for handlers that simply
           produce the next program.

           - **7.4.4d Revisit substrate cleanup only if needed.** If the
           approved carrier-backed shape leaks wrapper-specific detail into
           public APIs or forces broad macro churn, pause and evaluate a
           deliberate Explicit substrate rewrite with delayed typed
           continuation frames. Do not try to recover an action/outer
           split from already-recursively-mapped Explicit binds via
           unsafe erasure or ad-hoc downcasting.

           - **7.4.5 Migrate Span to the carrier path.**
           `SpanDispatcher` should use the H2 carrier so it observes tags
           around the interpreted action and returns the action result
           unchanged. Preserve ordinary scoped-dispatcher behavior for
           `CatchDispatcher`, `LocalDispatcher`, `RefLocalDispatcher`,
           `BracketDispatcher`, and `RefBracketDispatcher`.

           - **7.4.6 Add focused regression tests.** Cover all six wrappers.
           Tests must include nested Span ordering, result propagation,
           coexistence with first-order handlers, borrowed payloads on
           Explicit wrappers, multi-shot Rc behavior, and Send/Sync Arc
           behavior.

8.  Tests: scoped-effect unit tests covering each of the four
    standard constructors (`Catch`, `Local`, `Bracket`, `Span`)
    plus `compile_fail` cases. Negative-case enumeration:
    - End-to-end `Catch` recovery semantics after the scoped
      dispatcher lands: a protected action that throws through the
      FO `Except` handler short-circuits into the scoped catch
      handler, the recovery program runs, and non-throwing actions
      return unchanged.
    - End-to-end `Local` / `RefLocal` environment-modification
      semantics after the scoped dispatcher lands: the dispatcher
      applies `modify` before invoking the action, FO `Reader`
      asks inside the action observe the modified environment,
      the outer environment is restored afterward, and the Ref
      flavour computes the modified environment from `&E` without
      consuming the parent environment. Tests that issue repeated
      by-value `Reader::Ask` operations use an `E: Clone` environment;
      no test should claim current Reader provides no-clone repeated
      environment access.
    - End-to-end `Bracket` / `RefBracket` resource lifecycle
      tests after the scoped dispatcher lands: acquire runs
      first, body receives the acquired resource, release runs
      after body, release observes the same resource semantics
      as the constructor flavour (Val-owned or Ref pointer clone),
      body result is returned after normal-path effectful release
      completes, and panic/drop cleanup tests assert only ordinary
      resource `Drop` behavior rather than interpreted effectful
      release during unwinding.
    - End-to-end `Span` semantics after the scoped dispatcher
      lands: the span tag is observed by the scoped handler around
      the action, nested spans preserve ordering, action results
      propagate unchanged, and handler-list omission is reported
      consistently with the other scoped effects.
      The tag-observable around-action portion depends on step 7.4's
      B31 continuation-aware scoped-handler path; do not narrow this to
      result propagation only.
    - Scoped operation in an FO-only row (program declares
      `S = CNilBrand` but constructs a scoped op).
    - Mismatched body / release closure shapes for the
      `bracket` smart constructor (closure types that resolve
      to neither `Val` nor any `Ref<P>` impl).
    - `RefBracket` instantiated with a `P` that does not
      implement [`RefCountedPointer`](../../../fp-library/src/classes/ref_counted_pointer.rs).
    - Scoped handler-list omission (a row containing
      `CatchBrand<E>` interpreted with a scoped handler list
      missing the `Catch` cell).
      Reformulate relevant Phase 3 tests to use scoped operations
      where appropriate.

    **B20 retry within step 8**: when the bracket dispatcher's
    tests are written, retry `ArcRun::bracket` end-to-end
    exercise (deferred from step 3.3.4 per the [B20 closure
    entry](resolutions.md#resolved-2026-05-08-phase-4-step-3.3.4-arcrunbracket-integration-tests-blocked-by-rustc-sendsync-overflow-b20-closed-via-option-a-skip-arcrunbracket-integration-tests-defer-to-step-8-bracket-dispatcher-tests-escalate-to-option-d-sendbracketbrand-redesign-if-step-8-still-cannot-exercise-it)).
    The dispatcher's API may not require constructing a
    user-facing scoped row containing `SendBracketBrand` (the
    dispatcher provides its own dispatch entry point), in which
    case the rustc Send+Sync overflow does not trigger and the
    test compiles cleanly. If so, add a tracking-resolved
    comment in `tests/run_bracket.rs` and remove the step 3.3.4
    skip for `ArcRun::bracket`.

    If the marker-struct cycle still triggers at step 8 testing
    time (i.e., the dispatcher's test surface still needs the
    user to construct a scoped row that exposes
    `SendBracketBrand` to the type system in the cycling way),
    escalate to step 8a below.

8a. **B20 escalation (conditional, only fires if step 8's
`ArcRun::bracket` retry is still blocked): redesign
`SendBracketBrand` to drop the GAT-Send-Sync bound on
`Sub`.** Single `feat(effects)` commit scoped to
`SendBracketBrand` and any cascading changes; the rest of
the Arc family stays unchanged (the GAT-Send-Sync bound is
preserved where it works, only relaxed on the bracket cell
where it cycles). Two implementation paths to investigate
at that time: - **Path D1: Unsafe `Send + Sync` impl on `ArcFree<F, A>`.**
Currently `ArcFree<F, A>: Send + Sync` is auto-derived
through the GAT-Send-Sync bound on `F`. Replace with an
explicit `unsafe impl<F, A> Send for ArcFree<F, A> where
      F: WrapDrop, A: Send` (similar for `Sync`). The
auto-derive would be removed; the unsafe impl asserts the
property is preserved by the substrate's structural
invariants. Soundness audit required at the unsafe-impl
site.

     - **Path D2: New substrate trait `SendSyncFreeShape`.**

Propagates `Send + Sync` through `ArcFree<F, A>` without
the recursive GAT bound. Higher-cost than D1; requires a
trait redesign.
Pick the lower-cost path that compiles cleanly. After step
8a lands, re-attempt the `ArcRun::bracket` marker-struct
test from step 3.3.4 (now should compile since the cycle
root is removed); add the test back to
`tests/run_bracket.rs` and remove the skip-tracking
comment. Resolutions.md follow-up entry references this
B20 entry. Deviation entry at deviations.md.

### Phase 5: Integration test, deferred items as needed

1. Port the canonical TalkF + DinnerF example from
   [`purescript-run/test/Examples.purs`](https://github.com/natefaubion/purescript-run/blob/master/test/Examples.purs#L13-L106)
   into
   `fp-library/tests/run_talkf_dinnerf_integration.rs`.
   Multi-effect program demonstrating Reader, State, Talk, and
   Dinner effects composed and handled in turn. Faithful port
   from PureScript's source.
2. Add row-canonicalisation Criterion benches (macro path vs
   `CoproductSubsetter` permutation-proof fallback path) and
   handler-composition benches per
   [decisions.md](decisions.md) section 9 item 6.
3. (Phase 3 deferred items, scheduled here so they're not lost):
   - Optional `tstr_crates` content-addressed-naming refinement
     for the macro layer
     ([decisions.md](decisions.md) section 4.1's Phase 2 note).
     Add only if real-world usage shows import-path-sensitive
     sorting causes confusion.
   - Compile-time index-table refinement (Koka-inspired). Add
     only if a benchmark shows Coproduct pattern-match dispatch
     is a measurable bottleneck.
4. Write `fp-library/docs/run.md` documenting the effects
   subsystem for users. Cross-link to
   [decisions.md](decisions.md) for design rationale.
5. **Documentation finalization.** Update the documents listed
   below so they reflect the production state of the effects
   subsystem once Phases 1-5 are complete.

   **Living step:** Each implementation phase, on completion,
   must review the bullets here and add any new public items,
   behavioural surprises, or constraints that surfaced during
   that phase's work, under the relevant document. The goal is
   that when this step finally runs, every documentation change
   it lists is accurate and nothing has been forgotten. Treat
   the per-document bullets as a checklist that grows over time;
   do not rely on memory or `git log` to reconstruct the change
   set at the end. If a phase finds that a planned doc update
   is no longer needed (e.g., a feature was deferred), strike
   it through with rationale rather than deleting it.

   Documents and what to add:
   - **[fp-library/docs/features.md](../../../fp-library/docs/features.md):**
     Add a "Free family" table parallel to the existing "Free
     functors" Coyoneda table (currently around lines 199-204),
     listing the six variants (`Free`, `RcFree`, `ArcFree`,
     `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`) with
     columns for Family (Erased / Explicit), Clone, Send,
     `'a`-payload, Bind cost (O(1) vs O(N)). Add a "Run
     subsystem" section listing the six concrete Run types,
     the dual-row structure, the Erased/Explicit dispatch
     split, and the `into_explicit` / `from_erased` API.
   - **[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md):**
     The "Unexpressible Bounds in Trait Method Signatures"
     classification table already has rows for the three
     Explicit Free variants (added by Phase 1 step 7). Append
     rows for any `*RunExplicitBrand` (Phase 2 step 4) or
     scoped-effect dispatch (Phase 4) impls that hit further
     HRTB-over-types or per-`A` Clone-bound walls.
   - **[fp-library/CHANGELOG.md](../../../fp-library/CHANGELOG.md):**
     Populate the `[Unreleased]` section under `Added` with the
     new public items: six-variant Free family (one promoted
     from POC, five new), `SendFunctor` trait family, six
     `Run` types, `Node` / `VariantF` / `ScopedCoproduct`,
     standard first-order effects (`State`, `Reader`, `Except`,
     `Writer`, `Choose`), standard scoped effects (`Catch`,
     `Local` / `RefLocal`, `Bracket` / `RefBracket`, `Span`),
     the macro family (`effects!`, `effects_coyo!`,
     `handlers!`, `define_effect!`, `define_scoped_effect!`,
     `scoped_effects!`, `im_do!`), the
     `interpret`/`interpretRec`/`run*` interpreter pair, and
     the natural-transformation builder. If any pre-existing
     public API changed shape during the port, record it under
     `Changed`. Match the categorization style established in
     0.17.x entries.
   - **[README.md](../../../README.md):** Add a brief
     "Effects" entry alongside the existing "Dispatch System"
     summary, pointing at `fp-library/docs/run.md` (created by
     step 4) for details.
   - **[docs/todo.md](../../../docs/todo.md):** Strike through
     or remove the "Algebraic effects/effect system" bullet
     (and its sub-bullets pointing at
     [plans/effects/effects.md](effects.md) and external Eff
     references); the work it tracks is now landed.
   - **[fp-library/docs/architecture.md](../../../fp-library/docs/architecture.md):**
     If the effects subsystem warrants top-level architectural
     description (parallel to existing "Free Functions" /
     "Dispatch" sections), add one summarising the
     six-variant Free substrate, the Erased/Explicit dispatch
     split, the dual-row Run shape, and the heftia-style
     scoped-effect encoding. Skip if `run.md` (step 4) already
     covers this depth and an architecture-level summary
     would duplicate.
   - **[fp-library/docs/dispatch.md](../../../fp-library/docs/dispatch.md):**
     If Phase 4's `BracketDispatch` / `LocalDispatch` Val/Ref
     dispatch introduces a pattern that doesn't follow the
     existing convention this doc describes, add a section
     covering the new shape. Skip if the new dispatch is a
     direct application of the existing pattern.

   Per-phase records (append as phases complete):
   - **Phase 1 (complete).** Six Free variants land
     (`FreeExplicit` promoted, `RcFree` / `ArcFree` /
     `RcFreeExplicit` / `ArcFreeExplicit` new), `SendFunctor`
     trait family lands across nine files, brand impls for the
     three Explicit Free brands land. The
     `limitations-and-workarounds.md` "Unexpressible Bounds"
     table gained six new rows (three by-value, three
     by-reference) for the Explicit Free family;
     `features.md` and `CHANGELOG.md` are not yet updated for
     these (waiting for this finalization step). The Phase 1
     step 8 finding that `Free<IdentityBrand, A>` is
     layout-cyclic should be mentioned in `run.md` (step 4)'s
     "When to use which" section because it constrains
     concrete-`F` choices.
   - **Phase 2 (in progress).** Phase 2 ships the `WrapDrop`
     trait at
     `fp-library/src/classes/wrap_drop.rs`
     as a Phase 1 retroactive refinement before step 4 resumes
     (see Open questions resolution above for the rationale).
     `WrapDrop` is a new public trait that needs to land in
     `features.md` (effects subsystem section) and the
     `CHANGELOG.md` `[Unreleased]` Added list. Free's struct
     bound migrates from
     `F: Extract + Functor + 'static` to
     `F: WrapDrop + 'static`; this is technically a breaking
     change to the bound but is purely-additive in practice
     because every existing F that implements `Extract` gains
     a paired `WrapDrop` impl.
     Append remaining findings here when Phase 2 completes:
     new public items in `Run`, `VariantF`, `Node`, the
     `effects!` macro, the `im_do!` macro, the conversion
     API, plus any unexpressible-bound rows that surface in
     the `*RunExplicitBrand` impls.
   - **Phase 3 (TBD).** Append findings here when complete:
     handler-pipeline machinery, interpreter family, standard
     first-order effects, `handlers!` and `define_effect!`
     macros, plus negative-case `compile_fail` UI tests.
   - **Phase 4 (TBD).** Append findings here when complete:
     scoped-effect coproduct, dual-row integration, the four
     standard scoped-effect constructors and their Val/Ref
     flavours where applicable, dispatch additions for
     `bracket` / `local`, plus any new dispatch.md /
     limitations-and-workarounds.md material.

### Phase 6+ (deferred, not in this plan)

These items arrive when concrete need surfaces. Each one names
the artifact, what it would deliver, why it is deferred, and a
revisit trigger; entries are ordered roughly from substrate
outward to user surface.

- **Cargo feature gating for the Free family.** Cargo feature
  gates that let downstream crates opt out of compiling
  individual Free variants if their compile cost becomes
  uncomfortable ([decisions.md](decisions.md) section 4.4
  "Open questions left after this decision"). _Why deferred:_
  the compile cost of shipping six variants plus the
  `SendFunctor` trait family is unverified; a real
  feature-gating design needs benchmark or downstream-feedback
  evidence to be motivated. _Trigger:_ benchmark or compile-time
  evidence that the unified compile cost is meaningfully painful
  for downstream crates.
- **`State::modify` Val/Ref split.** Add a `RefState<S>`
  first-order effect alongside `State<S>` whose `modify`
  operation takes `FnOnce(&S) -> S` instead of `FnOnce(S) -> S`,
  with a unified `modify(...)` smart constructor dispatching
  over closure shape via the same `Val` / `Ref` markers used by
  `Local` / `RefLocal` in Phase 4. _What this is for:_ users who
  want to derive a new state from the old without owning it,
  avoiding an `S: Clone` requirement. _Why deferred:_ Phase 3's
  standard first-order effect set ships Val-only to keep the
  surface small; the Run-brand by-ref hierarchy from Phase 2
  step 4 already supplies the trait routing this would build
  on. _Trigger:_ first user who hits the `S: Clone` wall on the
  Val flavour, or the first integration test that benefits from
  `&S` in `modify`.
- **Borrow-oriented Reader effect for no-clone local environments.**
  Add a first-order Reader variant or companion operation that lets
  handlers answer environment requests by borrow-derived values rather
  than by repeatedly cloning a by-value `E`. _What this is for:_
  true no-`E: Clone` end-to-end Local / RefLocal semantics when an
  action issues multiple environment reads. Phase 4 v1 only guarantees
  that RefLocal computes the modified environment from `&E`; repeated
  asks still use the existing by-value Reader shape and therefore may
  require `E: Clone`. _Why deferred:_ introducing a borrow-oriented
  Reader before standard scoped dispatchers would expand the
  first-order effect surface, handler API, smart constructors, docs,
  and tests. The current Reader semantics remain coherent and unblock
  v1 scoped dispatchers. _Trigger:_ first user or standard-library test
  that needs repeated environment reads without cloning `E`.
- **`Writer::censor` Val/Ref split.** Add `censor` to the
  standard `Writer<W>` set (currently only `tell` ships in
  Phase 3 step 4), then ship a `RefWriter<W>` extension whose
  `censor` takes `FnOnce(&W) -> W` instead of
  `FnOnce(W) -> W`. _What this is for:_ deriving a transformed
  log without consuming the parent, the writer-log analogue of
  `State::modify`'s ergonomic story. _Why deferred:_ `censor`
  itself is not in v1's standard Writer set, and the Val/Ref
  split is a follow-up to adding it. _Trigger:_ a real
  log-censoring use case, plus the same `W: Clone` ergonomic
  wall.
- **`handlers!{...}` macro Val/Ref variants.** Extend the
  Phase 3 macro so each per-effect handler entry can be emitted
  as Val or Ref based on the user's closure type, reusing the
  same `Val` / `Ref` markers as the rest of the dispatch
  system. Each entry is conceptually a closure of shape
  `FnOnce(E::Operation) -> Run<R', S, A>`; the Ref variant
  takes `FnOnce(&E::Operation) -> Run<R', S, A>`. _What this is
  for:_ handlers that inspect operations without consuming them
  (e.g., a logging handler that records and then forwards),
  avoiding a `Clone` bound on operation payloads. _Why
  deferred:_ the macro is already non-trivial in v1; shipping it
  Val-only first and extending it once a real handler benefits
  is a smaller initial bite, and the extension is non-breaking.
  _Trigger:_ first handler (in the standard library or
  downstream) that needs inspect-without-consuming on an
  operation payload.
- **`interpret_nt`-style entry-point taking [`NaturalTransformation`](../../../fp-library/src/classes/natural_transformation.rs)
  directly.** Companion to the handler-list-driven
  [`interpret`](../../../fp-library/src/types/effects/interpreter.rs)
  shipped in Phase 3 step 2. The trait-based form lets users
  bypass the per-effect closure pattern entirely, supplying a
  rank-2-polymorphic struct impl
  (`fn transform<A>(&self, fa: <RowBrand as Kind>::Of<'_, A>) -> <MBrand as Kind>::Of<'_, A>`)
  as a single value; [`Free::fold_free`](../../../fp-library/src/types/free.rs)
  already consumes the same trait, so the internal machinery
  is largely shared. _What this is for:_ users whose
  transformation genuinely doesn't depend on `A` (e.g., a
  cross-row hoist, a logging shim that ignores the program's
  result type, or programmatic composition of transformations
  outside the per-effect handler pattern); the existing
  closure-based path is mono-in-`A` per [decisions.md](decisions.md)
  section 4.6's resolution and Phase 3 step 2's deviations
  entry, so users who want true rank-2 polymorphism currently
  have to reach for `Free::fold_free` directly with their own
  loop. _Why deferred:_ the closure-based path covers the
  common-case ergonomic surface, and shipping a parallel
  `interpret_nt` per Run wrapper would have doubled Phase 3
  step 2's interpreter surface (six wrappers x three methods x
  two paths). _Trigger:_ first user request for an
  A-polymorphic transformation that the closure path's
  mono-in-`A` constraint blocks; or a benchmark / DX motivation
  for offering trait-impl handlers as a first-class user-facing
  alternative.
- **Scoped-aware short-circuit primitive.** Add a richer successor to
  `interpret_with_either` whose return carrier can represent pure
  completion, a matched first-order operation, and suspended scoped
  operations plus their continuations. _What this is for:_ future
  handler authors who need to observe the first matching first-order
  operation while preserving non-empty scoped rows, rather than
  returning a transformed program through `interpose`. _Why deferred:_
  Phase 4's only concrete user is `CatchDispatcher`, which is better
  served by scoped-row-preserving `interpose`; designing a new carrier
  across all six wrappers before a second user appears would expand the
  substrate API and proof surface prematurely. _Trigger:_ a second
  standard handler or downstream handler-builder use case that needs
  short-circuit observation rather than interpose-style replacement.
- **Scoped protocol-family public facade (B32 H3 revisit; H2 promoted
  to Phase 4).** H2 is no longer deferred: B35 promotes the internal
  wrapper-owned continuation carrier into Phase 4 step 7.4. Revisit
  H3 only if custom scoped-handler APIs need separate public protocol
  families after the H2 carrier exists. H3 would keep the current
  `DispatchScopedHandler` path for ordinary "produce the next program"
  handlers and add a separate around-action protocol family for
  Span-like handlers. H3 can document the semantic split clearly, but
  it permanently increases trait, macro, and mixed-row routing surface.
  _Why deferred:_ H2 gives the implementation one internal
  continuation-boundary invariant now; public protocol-family classes
  can be layered on top later only if downstream custom handlers need
  them. _Trigger:_ a custom-handler API or downstream use case needs
  user-visible ordinary-vs-around-action handler classes that the H2
  carrier plus standard dispatcher helpers cannot express cleanly.
- **`interpret_with<M: Monad>` (Monad-bound externally-targeted
  family).** Companion to Phase 3 step 4's
  `interpret_rec<M: MonadRec>` family that drops the
  stack-safety bound for callers with a `Monad`-but-not-`MonadRec`
  target. _What this is for:_ users with a target `m` that
  implements `Monad` but not `MonadRec` (rare in fp-library;
  possibly user-defined custom monads). _Why deferred:_
  requires `Fn` closures and handler-list `Clone` bound to make
  bind-driven recursion work in Rust (per the workarounds in
  the resolved 2026-04-29 blocker); these constraints don't
  appear in the other interpreter families. The closure-based
  MonadRec path (now including Phase 3 step 10's
  `interpret_with_rec`) covers the common-case ergonomic
  surface for most fp-library brands, which already implement
  `MonadRec`. _Trigger:_ first real user with a non-MonadRec
  Monad target, or a benchmark / audit showing the
  closure-recursion-with-Clone path is desirable for some
  specific m.
- **`run_cont` / `run_accum_cont` (continuation-passing
  interpreter family).** PureScript Run's `runCont` and
  `runAccumCont` ([`Run.purs:224-275`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs#L224-L275))
  use an algebraic-shape handler that takes the continuation
  explicitly: `(VariantF r (m b) -> m b) -> (a -> m b) -> Run r a -> m b`.
  Different shape from the closure-returns-next-program form
  Phase 3 ships. _What this is for:_ handlers that need fine
  control over continuation invocation (e.g., custom
  control-flow effects, non-trivial yield/resume semantics).
  _Why deferred:_ rare use case in practice; users with
  continuation-passing needs can reach for
  [`Free::fold_free`](../../../fp-library/src/types/free.rs)
  directly with a custom natural transformation. Shipping this
  family would require a new handler-closure shape, a parallel
  `cont_handlers!` macro, and per-wrapper methods. _Trigger:_
  first user request for explicit-continuation handlers that
  the Phase 3 closure-returns-next-program shape can't
  accommodate.
- **Algebraic-shape FO handlers (axis 2 widening).** Rework
  the `handlers!{}` macro and `Handler<E, F>` so each FO
  handler closure receives the operation AND a continuation
  explicitly: `Fn(EOp, (X -> Run<R, S, A>)) -> Run<R, S, A>`.
  Heftia's `AlgHandler` shape; matches PureScript Run's
  `runCont` form. fp-library currently ships
  implicit-continuation FO handlers (closure returns the next
  program directly) plus algebraic-shape HO handlers (scoped
  constructors carry body / release closures internally per
  decisions.md section 4.5). _What this is for:_ FO handlers
  that need to decide whether / when / how-many-times to
  invoke the continuation. E.g., a non-standard control-flow
  effect requiring explicit yield/resume. _Why deferred:_
  forces every FO handler to take a continuation; ergonomic
  regression for the typical case (most FO handlers just
  return the next program). Shipping algebraic-shape would
  require reshaping `handlers!{}` macro, all
  `DispatchHandlers` impls, and all FO doctests across the
  six Run wrappers. _Trigger:_ first user with a real need for
  explicit-continuation FO handlers (non-standard control flow)
  that the implicit-continuation form can't express; or
  Phase 4 / Phase 5 finding that scoped HO and FO handler
  shapes diverge enough to warrant unification.
- **`generalBracket` and `BracketConditions`.** Port the
  more general bracket from PureScript Aff at
  [`Aff.purs:364-373`](https://github.com/purescript-contrib/purescript-aff/blob/master/src/Effect/Aff.purs#L364-L373):
  `generalBracket` accepts a `BracketConditions` record with
  separate `killed`, `failed`, and `completed` handlers, each
  receiving the resource. _What this is for:_ observing how a
  bracketed action terminated (success, failure, cancellation)
  and running different cleanup per outcome, instead of v1's
  single uniform `release`. _Why deferred:_ v1's interpreter is
  sync and has no cancellation event, so `killed` has no
  semantics until the async target monad lands; without async,
  `generalBracket` collapses to a more verbose `bracket` with
  two unused branches. The Ref-flavour `RefBracket<'a, P, A, B>`
  already shows that multiple closures can each receive
  `P::Of<A>` without contention, so the dispatch design extends
  cleanly when the time comes. _Trigger:_ the async target
  monad lands (next item), at which point cancellation becomes
  a real event handlers want to observe.
- **Synchronous Bracket panic-finalizer hook.** Add an optional
  synchronous cleanup hook for Bracket-style resources that runs from
  ordinary Rust `Drop` during unwind, separate from the existing
  effectful `release` program. _What this is for:_ users who need more
  than resource-owned `Drop` cleanup during panic but still want to keep
  normal-path release effectful. _Why deferred:_ Phase 4 v1 keeps the
  public Bracket API effectful on the normal path and documents that
  effect interpretation is not available from `Drop`; replacing release
  with a synchronous-only closure would be a semantic downgrade. A
  companion hook needs a separate API design. _Trigger:_ first real use
  case requiring panic-time cleanup that cannot be encoded in the
  resource's own `Drop` implementation.
- **`MonadRec` impl for `Future` as an async target monad**
  ([decisions.md](decisions.md) section 9 items 3 + 4). _What
  this is for:_ asynchronous interpretation of `Run` programs
  via the same target-monad mechanism that already lets users
  pick `Identity` or `Thunk` for sync interpretation; no
  parallel `AsyncRun` family. _Why deferred:_ v1 ships sync
  interpreters and async users wrap calls in `spawn_blocking`
  or similar; adding `Future` as a `MonadRec` target requires
  designing around pinned futures, executor coupling, and
  multi-shot continuation friction, which is a separate body of
  work. _Trigger:_ first user request for async interpretation
  that cannot be satisfied by `spawn_blocking` around a sync
  interpreter call.
- **Split `fp-macros` into `fp-effects-macros`**
  ([decisions.md](decisions.md) section 9 item 8). _What this
  is for:_ a separate crate housing the effects-related
  proc-macros (`effects!`, `effects_coyo!`, `handlers!`,
  `define_effect!`, `define_scoped_effect!`,
  `scoped_effects!`) so their release cadence is independent of
  the HKT-system macros and do-notation macros that share
  `fp-macros` today. _Why deferred:_ v1 keeps everything in one
  crate to avoid multiplying release coordination and adding a
  parallel macro-resolution path. _Trigger:_ `fp-macros`
  compile time grows uncomfortably, or effects-related macro
  changes start blocking unrelated macro releases.

## Implementation notes

1. **`Coproduct` choice (Phase 2).** The POC depends on
   `frunk_core::coproduct::{Coproduct, CNil, CoproductSubsetter}`,
   and the production port adopts the same dependency. Phase 2
   step 1 adds `frunk_core` to `fp-library`'s `Cargo.toml`,
   confirms the license is permitted by `just deny`, and
   introduces a thin Brand-aware adapter layer at
   `fp-library/src/types/effects/coproduct.rs` (newtypes plus `impl`
   blocks that bridge `frunk_core`'s Plucker / Sculptor / Embedder
   traits to the project's `Brand` system). Implementing
   fp-library's own `Functor` for `frunk_core::Coproduct<H, T>`
   directly is permitted by the orphan rules (own-trait +
   foreign-type) and is the preferred shape for the recursive
   `Functor` dispatch; `Brand`-style impls on the foreign type
   require the newtype wrapper. If the adapter ever exceeds
   approximately 200 lines, that signals real impedance mismatch
   and a fork to an in-house reimplementation should be
   considered, but the default is to stay on `frunk_core`.
2. **POC-to-production migration (Phases 2 + 3).** Historical
   strategy note: when this plan was written, the POC at
   `poc-effect-row/` was a separate Cargo workspace not
   integrated with fp-library's `Brand` system. The production
   migration was expected to be mostly mechanical (swap the
   stub Coyoneda for fp-library's, swap the raw Coproduct types
   for branded equivalents) with surface-area changes around the
   macro output. The actual migration landed in Phase 2 step 8
   (`effects!` / `raw_effects!` macros) and Phase 2 step 10a
   (regression baseline at
   [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs));
   the POC workspace was deleted in step 10b.
3. **`Drop` correctness (Phase 1).** `RcFree` and `ArcFree`
   inherit `Free`'s iterative `Drop` strategy via the underlying
   `CatList`; `FreeExplicit` requires its own iterative `Drop`
   per the POC findings. Test deep-`Drop` for all four variants
   in Phase 1 unit tests.
4. **Async-via-target-monad gating (Phase 5+).** The interpreter
   functions stay sync; an async target monad arrives in Phase 6+.
   Until then, async users wrap the interpreter call in
   `spawn_blocking` or similar.

## Success criteria

The plan is complete when all of the following hold:

- All six Run types are publicly exported from `fp-library`:
  `Run`, `RcRun`, `ArcRun` (Erased family, inherent-method only)
  and `RunExplicit`, `RcRunExplicit`, `ArcRunExplicit` (Explicit
  family, Brand-dispatched). Conversion methods
  (`into_explicit()` / `from_erased(...)`) link paired Erased
  and Explicit variants.
- The `effects!` macro accepts `effects![A, B, C]` over arbitrary
  effect types and produces a canonical row across input
  orderings; the same row composes with `CoproductSubsetter`
  permutation proofs for hand-written cases.
- `m_do!` and `a_do!` work over the three Explicit Run brands
  (`RunExplicitBrand`, `RcRunExplicitBrand`,
  `ArcRunExplicitBrand`) for first-order effect programs.
  `im_do!` (Inherent Monadic do) provides the equivalent
  monadic do-notation for the three Erased Run types via
  inherent methods, plus by-reference do-notation over
  canonical Coyoneda-headed rows on the four `Clone`-able
  wrappers (`RcRun`, `ArcRun`, `RcRunExplicit`,
  `ArcRunExplicit`) where brand-level `m_do!(ref ...)` cannot
  reach.
- Each of the six Free variants supports its promised property
  (the Free wrapper's spine consumption is single-shot vs.
  multi-shot, thread-safe, `'static` vs `'a`, Brand-dispatched
  vs inherent-method-only) with per-variant unit tests passing.
  Note: the single-shot vs. multi-shot property applies to the
  Free wrapper's spine consumption only; per-effect closures
  (e.g., [`State`](../../../fp-library/src/types/effects/state.rs)'s
  `dyn Fn` continuations,
  [`Choose`](../../../fp-library/src/types/effects/choose.rs)'s
  `dyn Fn(bool) -> A` branch) carry their own multi-shot
  property at the effect-instance level on every wrapper that
  hosts the effect, independent of the wrapper's spine
  semantics.
- The `SendFunctor` / `SendPointed` / `SendSemimonad` /
  `SendMonad` trait family ships and is used by
  `ArcFreeExplicitBrand` and `ArcRunExplicitBrand` for their
  by-value Brand impls. `ArcCoyonedaBrand` also gains
  `SendFunctor` (and downstream) impls, retroactively closing
  the gap that
  [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs)'s
  module docs flag.
- `Reader`, `State`, `Except`, `Writer`, `Choose` ship as standard
  first-order effects with smart constructors.
- `Catch<'a, E>` and `Span<'a, Tag>` ship as Val-only
  scoped-effect constructors; `Local` ships in Val and Ref
  flavours (`Local<'a, E>` + `RefLocal<'a, E>`); `Bracket`
  ships in Val and Ref flavours (`Bracket<'a, A, B>` +
  `RefBracket<'a, P, A, B>` parameterised over
  `P: RefCountedPointer`), each with scoped-handler
  interpreters. The `bracket` and `local` smart constructors
  use closure-driven Val/Ref dispatch (mirroring
  [`dispatch.md`](../../../fp-library/docs/dispatch.md)) so the
  user picks the flavour by closure type, not by turbofish.
  (`Mask` deferred per [decisions.md](decisions.md) section 4.5
  sub-decisions.)
- The by-value hierarchy (`Functor` / `Pointed` / `Semimonad` /
  `Monad`, with `SendFunctor` / etc. for the `Arc`-affected
  brand) and by-reference hierarchy
  (`RefFunctor` / `RefSemimonad` / `RefMonad`, etc.) are both
  implemented for every Brand-dispatched Free variant
  (`FreeExplicitBrand<F>`, `RcFreeExplicitBrand<F>`,
  `ArcFreeExplicitBrand<F>`) and every Brand-dispatched Run
  variant (`RunExplicitBrand`, `RcRunExplicitBrand`,
  `ArcRunExplicitBrand`); `dispatch::map` / `dispatch::bind`
  route correctly for both consuming and borrowing closures over
  these brands. The Erased family (`Free`, `RcFree`, `ArcFree`,
  `Run`, `RcRun`, `ArcRun`) does NOT participate in dispatch by
  design; users access those types via inherent methods or
  convert to the corresponding Explicit variant.
- The TalkF + DinnerF integration test passes.
- All 25 row-canonicalisation tests migrated from
  `poc-effect-row/` pass under the production types.
- Per-Free-variant Criterion benches show no regression beyond
  ~50% of the `FreeExplicit` POC baseline (~27ns / node in the
  linear regime).
- `just verify` passes (fmt, check, clippy, deny, doc, test).

## Reference material

- Design and decisions: [decisions.md](decisions.md).
- Effects research arc:
  [research/](research/) (13 codebase classifications,
  `_classification.md` synthesis, 3 Stage 2 deep dives).
- Type-level-sorting research arc:
  [../type-level-sorting/research/](../type-level-sorting/research/)
  (16 codebase classifications, `_classification.md` synthesis).
- POC validation:
  - [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
    -- findings from the (now-deleted) `poc-effect-row/`
    workspace; the workspace was migrated to
    [`fp-library/tests/run_row_canonicalisation.rs`](../../../fp-library/tests/run_row_canonicalisation.rs)
    and deleted in Phase 2 step 10b.
  - [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)
    -- `FreeExplicit` POC.
  - [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs)
    -- `FreeExplicit` Criterion bench.
- PureScript Run reference:
  [`purescript-run`](https://github.com/natefaubion/purescript-run).
- Comparison table for the Rust port versus PureScript Run and
  Hasura's `eff` is in [decisions.md](decisions.md) section 10.
