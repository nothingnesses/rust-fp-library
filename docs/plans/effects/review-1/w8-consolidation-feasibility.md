# W8 Scoped-Dispatch Consolidation Feasibility

This records the proof-first audit that resolves the W8 Scoped-Dispatch
Consolidation Gate. It is the companion to
[`w8-scoped-dispatch-design-note.md`](w8-scoped-dispatch-design-note.md)
and addresses the second recommendation of `findings.md` section 5
(evaluate whether the scoped-dispatch surface can be unified or reduced).

Resolution: preserve the shipped design; partial descriptor-backed
generation of the per-wrapper plumbing is feasible but deferred behind a
concrete trigger; full consolidation is rejected.

## Method

- Read the production dispatch routing: the two ordinary one-slot roles
  (`DispatchScopedHandler` / `DispatchScopedHandlers`) and the three
  around-action roles (boundary, carrier, residual).
- Captured the scoped-surface line-count baseline.
- Analysed the W2 descriptor model in
  `fp-macros/src/documentation/generator_descriptors.rs`.
- Measured the mechanical similarity of the per-wrapper plumbing.

## Baseline

Scoped-surface size at the time of the audit:

- Interpreter core (dispatch traits plus resume vocabulary):
  ~3,050 lines (`interpreter.rs`, `interpreter/scoped_resume.rs`).
- Per-wrapper dispatch machinery: ~8,260 lines across the six wrappers'
  `representation.rs` / `raw_scoped.rs` / `boundary.rs`.
- Per-effect scoped handlers: ~18,600 lines under
  `standard_scoped_handlers/`.
- Behavioural tests: ~16,260 lines across 17 test files.

## Findings

### F1. The W2 descriptor model is first-order by construction.

`EffectName` lists only the 13 first-order effects, `EffectOperationShape`
enumerates only first-order operation shapes (`ReaderEnvironment`,
`StateCell`, `TypedAbort`, `BooleanChoiceContinuation`, ...), and every
builder emits effect-as-functor code dispatched by `Functor::map`. The
scoped surface instead uses case-analysis dispatch (boundary / carrier /
residual), varies per-wrapper along a carrier-strategy axis (erased
raw-step versus explicit boundary) that the model does not have, and each
scoped effect's semantics are bespoke. Reusing the W2 model as-is is not
possible; full scoped generation would require a new scoped-dispatch
generator subsystem.

### F2. The per-wrapper plumbing is about 83 percent mechanical.

The `RcRun` and `ArcRun` raw-scoped modules have identical item skeletons
(the same `*FirstOrderAccumulator` / `PreservingAccumulator` / `Replacer`
/ `Rewriter` traits, the same `RawScopedContinuation` / `ScopedContinuation`
structs, the same `dispatch_*_raw_scoped` head / cons / nil methods, and
the same `resume_*` / `_with_post_action` / `_with_action_transform`
methods). After normalising the pointer type and the `Send + Sync`
bounds, only about 203 of about 1,200 lines differ, and those differences
are import ordering plus the `Send + Sync` bounds that the Arc variant
adds. This is the same Box / Rc / Arc plus Send axis that W2 already
models for first-order effects, so the per-wrapper carrier / resume /
dispatch plumbing is mechanically generatable in principle.

### F3. The per-effect semantic handlers are bespoke.

The Writer pre / post handlers interpose on the first-order `Tell`
operations inside the selected action; `CatchDispatcher` intercepts
`Except`; `BracketDispatcher` runs an acquire / body / release lifecycle;
`Local` / `RefLocal` rewrite `Reader`; `Span` is metadata-only. These are
not mechanical and must stay hand-written under any option.

### F4. The design is complete and tested.

`plan.md` Phases 1 through 5 are complete with no open blockers, and the
scoped surface carries about 16,260 lines of behavioural tests. The W8
finding (`findings.md` section 5) was about conceptual heaviness for a
reader, "a lot of surface to hold in your head," which the design note
addresses directly. The per-wrapper line count is a maintenance cost, not
the architectural defect the finding named.

## Options

- Full consolidation (generate the whole scoped cross-product): requires
  a new generator subsystem (F1); rejected. There is no architectural
  justification, the per-effect semantic handlers cannot be mechanised
  (F3), and merging the distinct dispatch roles would flatten the
  type-level routing the split exists to provide.
- Partial generation (generate the mechanical plumbing, keep the semantic
  handlers and the trait surface): feasible (F2). The cost is a new
  scoped-dispatch generator plus an exact `cargo expand` equivalence proof
  against about 8,260 lines of subtle hand-tuned dispatch; the payoff is
  maintenance reduction only, because the design is complete and tested
  (F4).
- Preserve (keep the shipped design; the design note resolves the
  finding's stated concern): the lowest-risk option and the gate's
  preserve-by-default position.

## Recommendation

Preserve now; defer partial generation behind a concrete trigger; reject
full.

- Preserve the boundary / carrier / residual design and its trait surface
  as shipped. The design note resolves the finding's conceptual-heaviness
  concern.
- Partial descriptor-backed generation is feasible (F2) and is the
  recorded eventual path, but it is deferred and gated on a concrete
  trigger that makes the duplication actually bite: adding a new scoped
  effect, adding a seventh wrapper, or a recurring maintenance burden
  across the Box / Rc / Arc scoped plumbing. When triggered, the target is
  the mechanical plumbing only, proven via the same `cargo expand`
  equivalence discipline W2 used for first-order effects; the per-effect
  semantic handlers (F3) stay hand-written.
- Reject full consolidation.

## Reasoning

This follows the guiding principles. The architecture is already right
(the split is correct, complete, and tested), so the finding is addressed
by documenting the rationale (the design note), not by spending a large,
error-prone new-generator effort to reduce duplication on finished code
with no capability gain. Optimising the per-wrapper line count now would
target a maintenance symptom rather than an architectural defect. The
deferral keeps the proven-mechanical path available for the moment a
concrete need makes the duplication cost real. This decision is
independent of the W13 runtime-policy work; the scoped-dispatch surface
and the deferred runtime-sensitive effects do not interact here.
