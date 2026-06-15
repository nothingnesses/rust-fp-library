# Effects System Remediation Plan (review-2), Draft

Status: draft. The foundation sweep has resolved the plan's foundational decision (item 4, adopt FS-1); no implementation step has been started.

This plan turns the review-2 findings into work items. The findings themselves live in [README.md](README.md), [architecture.md](architecture.md), [organisation-naming-documentation.md](organisation-naming-documentation.md), [coverage-gaps.md](coverage-gaps.md), [refactoring-opportunities.md](refactoring-opportunities.md), [external-ideas.md](external-ideas.md), and [prior-reviews-crosscheck.md](prior-reviews-crosscheck.md); each work item cites the findings it addresses. Where a finding was already analysed by the review-1 remediation work, the item builds on that record (notably `../review-1/w8-consolidation-feasibility.md` and the W13 material in `../review-1/remediation-plan.md`) instead of re-deciding from scratch. The most foundational finding was settled by the [foundation-sweep/](foundation-sweep/) investigation, which prototyped a unified-row rebuild across ten POCs and four decision gates and concluded to adopt it (FS-1); its conclusions are folded in below, with item 4 the rebuild spine.

For every undecided item the plan records the viable approaches, their trade-offs, the recommended approach, and the reasoning, so adoption is a matter of confirming or overriding recommendations rather than reconstructing analysis. Items the foundation sweep settled are compressed to the adopted decision plus a pointer to the evidence, per the Documentation Protocol.

## Project Principles

- API-breaking changes are acceptable when they lead to a better end state.
- The project prioritises correctness, internal coherence, technical debt reduction, maintainability and long-term architecture over compatibility shims for existing implementation details.
- When a local compatibility-preserving fix conflicts with a cleaner architecture, choose the cleaner architecture unless a concrete limitation prevents it.
- If such a limitation forces a fallback, document the limitation, the trade-offs, and the fallback before adopting it.

## Documentation Protocol

This plan intentionally does not use a separate section for previously resolved decisions. Resolved or adopted decisions are folded into the implementation plan as concrete steps. This keeps the document from accumulating stale decision history while still making the intended work clear.

During implementation, this document should be kept up to date by indicating, for each of the implementation steps, what their current status is.

Status convention: each work item carries a `Status:` line (`not started`, `in progress`, `blocked on <item or question>`, `decided`, `complete`, or `rejected with reason`). While an item is in progress, its individual numbered steps are annotated in place (for example `2. (done) ...`). When an item is decided or completes, its recommendation text is compressed to the adopted decision plus a pointer to the evidence or implementing commits.

## Open Questions, Decisions, Issues and Blockers

The foundation sweep ([foundation-sweep/charter.md](foundation-sweep/charter.md)) settled the one strategic decision that organises this plan: the dual-row-versus-unified-row question (item 4) is resolved toward FS-1 (adopt the unified row). That decision is now the plan's foundation (Phase B); see the Sequencing overview. The residues it leaves open are bounded follow-ups, not blockers, and are recorded on their items: per-`Store` closure construction stays generated (item 7); and exponential higher-order effects (CC/Shift, unlift, async) are a later design round bounded by the E5 catalogue (items 18, 19). The Box `FnOnce` one-shot reconciliation that POC-8 had left as a stretch was subsequently built and passed in POC-8b (item 6), so the prefix-scheme fallback is not needed.

## Implementation Steps

### Sequencing overview

The foundation sweep resolved the most foundational decision by prototype: adopt FS-1, a unified effect row with per-brand order markers, elaboration of higher-order effects into first-order ones, brand-keyed dispatch, and a single closure-storage-parameterised substrate, replacing the dual rows, the boundary-frame subsystem, the result-polymorphic protocol traits, positional dispatch, and the six-wrapper duplication. All four decision gates passed across ten POCs (G1 facade viable, G2 elaboration / FS-1, G3 substrate unification go, G4 adopt FS-1). The plan is therefore organised around that rebuild rather than around the open question it used to be:

- Phase A, accuracy and quality, independent of the rebuild (items 1 to 3). These describe and harden the current code and are worth doing now, in parallel, while FS-1 is unbuilt; item 1's self-containment work carries into the FS-1 docs.
- Phase B, the FS-1 foundation (items 4 to 9): the unified-row rebuild (item 4, the spine) and its decided components, substrate unification (item 5), brand unification via `ClosureStorage` (item 6), the residual construction generation (item 7), brand-keyed dispatch and the effect-spec surface (item 8), and tagged effects as label-brands (item 9). This phase deletes the boundary-frame subsystem, the result-polymorphic protocol traits, and the scoped-row machinery.
- Phase C, the FS-1 surface (items 10 to 14): effect-definition codegen and macros retargeted to the unified row (items 10, 11), the API-name cleanups (items 12, 13), and the nondeterminism runners reframed onto FS-1's elaboration (item 14).
- Phase D, ports on FS-1 (items 15 to 17).
- Phase E, the later exponential round, out of the sweep's scope and bounded by the E5 catalogue [foundation-sweep/polynomial-exponential-catalogue.md](foundation-sweep/polynomial-exponential-catalogue.md) (items 18 async, 19 CC/Shift).
- Phase F, hygiene (item 20), most of which the rebuild absorbs.

Hard dependencies: the Phase B rebuild (item 4) precedes the FS-1-shaped Phase C and the Phase D ports; item 17 still depends on item 14's nondeterminism semantics (reframed onto FS-1's elaboration / weave); item 19 (exponential round) depends on item 18's async direction, no longer on the row decision (now settled). Everything in Phase A can start immediately and in parallel.

## Phase A: accuracy and quality (independent of the FS-1 rebuild)

### 1. Documentation accuracy and self-containedness sweep

Findings: organisation-naming-documentation.md sections 3 and 4 (eight drift items, five plan-label violations); refactoring-opportunities.md R6 and R10; prior-reviews-crosscheck.md section 2 items 1, 5, and 8 (missing limitations and legend).

Approaches:

- A. One consolidated documentation commit fixing every listed item. Trade-offs: a single reviewable change, immediate end to actively misleading docs (the `interpreter.rs` text currently denies a shipped feature); the commit touches many files at once, but every change is prose.
- B. Fix opportunistically as each module is next touched. Trade-offs: smaller diffs per commit, but wrong documentation stays live indefinitely, and several items (the `run.md` catalog, the stale async section) sit in files with no scheduled code changes.

Recommendation: A. Documentation in this codebase is enforced product (the doc pipeline rejects drift it can detect; these items are the ones it cannot). Leaving known-false statements live contradicts the correctness principle, and the change carries zero regression risk. Verification for the doc-only commit is `just fmt && just doc`.

Steps:

1. Fix the stale claims: `node.rs` "future work" text and Functor-impl note; `interpreter.rs` async section (describe `Await`/`run_async`, delete the spawn_blocking framing or demote it to a sidebar about effects other than `Await`); the `CatchHandler` "Rc/Arc only" paragraph; the future-tense "W11 runner" docs in `kv_store.rs`, `fresh.rs`, `input.rs`; the `Run::handle` per-layer recursion claim (state the actual recursion budget: scoped nesting depth).
2. Purge plan-workstream labels and external URLs from source docs: `await_future.rs`, `async_interpreter.rs`, `row_embed.rs`, the three effect modules above, and the `run_heftia_semantics.rs` module header (restate ported cases concretely; move provenance URLs into this plans area).
3. Refresh `fp-library/docs/run.md`: complete the effect catalog (fourteen first-order effects, eight scoped handler families), document the generated runner families as the primary API, add the brand-prefix legend (lift the table from organisation-naming-documentation.md section 2.1), describe the Rc/Arc row-construction path, and correct the Handler Order paragraph per item 14 below.
4. Document `expand`/`weaken` cost in `run.md` by lifting the rationale from `../review-1/w1-row-embed-spike.md`, and present Member-generic row-polymorphic authoring as the preferred pattern with one worked example of each style.
5. Extend the limitations lists (`run.md`, `types/effects.rs`): generic scoped rows are deferred (`define_scoped_row!` is concrete-only), the Fn versus FnOnce capture asymmetry between handlers and `bind`, and the single-continuation-hole requirement for Box-family effects (also added to `custom-effects.md`, together with the `SendFunctor`/`RefFunctor`/`Extract` requirements for shared wrappers).
6. Clarify the `define_effect!` situation in `run.md`/`custom-effects.md` (an internal registry-keyed macro exists; the public macro is item 11).

Foundation-sweep impact: none on the work, but note that this fixes the current dual-row docs, which item 4 step 5 will later replace with the FS-1 design rationale; the self-containment and limitations work carries forward regardless.

Status: not started.

### 2. Run-level test hardening: stack safety and laws

Findings: architecture.md section 3.8 (no deep `Run` program tests; `stack_safety.rs` covers other types); coverage-gaps.md section 5 (laws untested).

Approaches:

- A. Targeted suites: deep-program tests (on the order of 100k steps) through `handle`, `handle_rec`, chained `handle_with`, `interpose`, and `expand`, on Box and Rc families; QuickCheck law tests (Functor/Monad laws on small concrete rows; `expand` naturality against handler results). Trade-offs: bounded effort, directly pins the guarantees the docs make.
- B. A random-program generator producing arbitrary well-typed effect programs for property testing. Trade-offs: far stronger coverage in principle; generating well-typed programs over type-level rows is a project in itself.

Recommendation: A now; consider B only if A starts finding classes of bugs that targeted tests miss. Reasoning: the open risks are specific (recursion over program depth hiding in traversals; queue rotation under boundary frames), and targeted tests reach them at a fraction of B's cost.

Steps:

1. Add `run_stack_safety.rs` with deep-bind, deep-handle, deep-`handle_with`-chain, deep-`interpose`, and deep-`expand` cases on `Run` and `RcRun`; include one deep program inside a `Catch` boundary (this doubles as the CatList rotation check flagged in external-ideas.md item 3).
2. Add law property tests for the wrappers on a two-effect row, including `expand` naturality.
3. While writing step 1, verify or refute the corrected recursion-budget claim from item 1 step 1 (host stack grows with scoped nesting only).

Foundation-sweep impact: the deep-program and law tests are substrate-agnostic in intent; the FS-1 rebuild (item 4) will re-point them at the unified row, and the deep-`Catch`-boundary case becomes a deep-elaboration case (no boundary frames). The POC-3 finding (queue-deep programs drop safely; deep `Wrap`-nesting of Coyoneda cells is a shared limitation) is the property these tests should pin.

Status: not started.

### 3. Benchmark expansion and the tail-resumptive investigation

Findings: coverage-gaps.md section 5 (benchmark gaps); refactoring-opportunities.md R11; external-ideas.md items 1 and 6 (EvEff/koka taxonomy; corophage's dispatch-position calibration).

Approaches:

- A. Extend the existing criterion suites (`effect_rows.rs`, `scoped_operations.rs`) with dispatch-position cost, deep-bind throughput versus plain `Free`, `expand` cost versus program size, and boundary-frame overhead; only then decide whether a tail-resumptive fast path is worth designing. Trade-offs: measurement before design; the fast path may turn out unnecessary (corophage's data suggests positional dispatch is cheap and the costs live in allocation and queue traffic).
- B. Design the fused fast path now on the EvEff/koka argument alone. Trade-offs: saves a measurement round if the result is positive; risks engineering against the wrong bottleneck.

Recommendation: A. Reasoning: the architecture already inlines continuations in the common path (handlers call the continuation synchronously); whether the remaining Coyoneda-lowering and queue costs matter is an empirical question, and the project's benchmarking infrastructure makes answering it cheap.

Steps:

1. Add the four benchmark families to the criterion suites.
2. Write up results in `fp-library/docs/benchmarking.md` or a plans note; decide go/no-go on a fused fast path.
3. If go: design the fast path as an internal specialization of the runner loops (no public API change), with before/after numbers.

Foundation-sweep impact: POC-10 ([foundation-sweep/poc-10-findings.md](foundation-sweep/poc-10-findings.md)) captured an FS-0 baseline (first-order row machinery ~1-15 ns; scoped/higher-order dispatch ~497-788 ns from boundary frames) and showed FS-1 is no worse first-order and substantially cheaper higher-order. This item's "boundary-frame overhead" measurement is partly answered (it is the dominant FS-0 cost and FS-1 removes it); after the rebuild, re-target the suites at the unified row and keep the production wall-clock comparison the sweep deferred.

Status: not started.

## Phase B: the FS-1 foundation (the unified-row rebuild and its components)

### 4. Unified row adoption (FS-1) and rebuild roadmap

Findings: architecture.md section 3.1 (divergence from the referenced heftia's unified row; misattribution in docs); refactoring-opportunities.md R12.

Decision (adopted): the foundation sweep prototyped the unified row (the reserve approach over the originally-recommended decision record) and concluded to adopt FS-1: a unified effect row with per-brand order markers, elaboration of higher-order effects into first-order ones, brand-keyed dispatch, and a single closure-storage-parameterised substrate. Evidence: ten POCs across four gates ([foundation-sweep/charter.md](foundation-sweep/charter.md), Decision gates section; per-POC findings docs). The Rust-specific constraints originally thought to motivate dual rows (constraint-kind ergonomics, closure monomorphism) were shown not to force them: order classification, partition, brand-keyed dispatch, mixed first-order/higher-order rows, same-result and result-shape-changing elaboration, row widening, and substrate unification all hold on the real encoding (POC-0 through POC-10).

This item is the FS-1 production-implementation spine. It subsumes items 5, 6, 7 (substrate), 8 (dispatch), and 9 (tagging), and reshapes items 11, 12, 13, and 14.

Steps (FS-1 rebuild):

1. Build the unified row: per-brand order markers (POC-0), the order-directed peel (POC-3), and brand-keyed handler dispatch (POC-2; this is item 8's mechanism and item 9's labels, the brand is the label and a tag is a wrapper that changes it).
2. Elaborate higher-order effects into first-order ones over the unified row (POC-4 same-result, POC-5 result-shape-changing, POC-9 multi-HOE slice): re-express Catch, Local, Listen/Censor, Bracket as in-row cells with interpret-pass elaboration; delete the boundary-frame subsystem, the result-polymorphic protocol traits, and the scoped-row machinery.
3. Build the `ClosureStorage`-parameterised substrate (items 5 and 6 / POC-8, POC-8b): one `Run<Store, A>` over an associated stored-closure type that carries the callable kind (Box `FnOnce`, Rc/Arc `Fn`), bridged by a by-value `call_once`, with the per-`Store` `Clone`/`Send + Sync` bounds on inherent methods; per-`Store` construction, including `bind`/`map`, stays generated (item 7).
4. Re-express the effect catalog and `expand`/`weaken` over the unified row (POC-6); migrate the tests, including the heftia semantics suite. API-breaking, acceptable per the principles.
5. Fold the design rationale into `run.md` (replacing the dual-row design-rationale paragraph) and rewrite architecture.md to the FS-1 as-built once it lands.

Scope: polynomial higher-order effects only; exponential effects (CC/Shift, unlift, async) are the later round (items 18, 19), bounded by the E5 catalogue. Weave (heftia's `Weave`) is held in reserve (FS-2) for any polynomial higher-order effect that resists clean elaboration, and is the principled route to item 14's branch-local-versus-global (R5) semantics.

Status: decided (adopt FS-1); implementation not started.

### 5. Substrate unification: one closure-storage-parameterised substrate

Findings: architecture.md section 3.6 (do all six wrappers earn their keep); prior-reviews-crosscheck.md section 2 item 7 (the W3 capability audit's undecided half).

Decision (adopted): gate G3 / POC-8 ([foundation-sweep/poc-8-findings.md](foundation-sweep/poc-8-findings.md)) shows substrate unification is feasible (go), superseding the earlier "shrink or formalise the wrapper matrix" framing. The six erased wrappers collapse: the substrate type, its interpreter, and its `Clone` instance become one `Run<Store: ClosureStorage, A>` over an associated stored-closure type, with the per-`Store` `Clone`/`Send + Sync` bounds carried on the substrate's inherent methods (which a fixed trait method signature could not carry). Recorded boundary: closure construction stays per-`Store` (the `Store::Stored` projection is not injective, so generic construction cannot infer `Store`), so smart constructors and per-effect injection stay generated (item 7); POC-8b ([foundation-sweep/poc-8b-findings.md](foundation-sweep/poc-8b-findings.md)) confirms `bind`/`map` join that per-`Store` construction surface (their bodies move the captured continuation for Box and clone it for Rc/Arc), without moving the substrate type, interpreter, or `Clone` off their single definitions. The erased-versus-explicit and per-pointer matrix questions are answered by this collapse: one parameterised substrate, with `Store` ranging over the closure-storage brands.

This work is item 4 step 3.

Status: decided (substrate unification go, via FS-1); implementation folded into item 4.

### 6. Brand unification via `ClosureStorage`

Findings: organisation-naming-documentation.md section 2.1; refactoring-opportunities.md R3; prior-reviews-crosscheck.md section 3 (recorded disagreement with review-1's "coherent" assessment).

Decision (adopted): POC-8 is the State spike the original item called for, and it lands the decision in favour of unification. The `ClosureStorage` associated stored-closure type unifies the closure storage (`ToDynFnOnce` for Box, `ToDynCloneFn`/`ToDynSendFn` for Rc/Arc), so one brand per effect and one substrate type carry the per-`Store` bounds, replacing the three sibling brands and the prefix axis. The original 6A risk, the `FnOnce`-versus-`Fn` split (the Box spine's one-shot continuation versus the Rc/Arc reusable `Fn`), is resolved rather than deferred: POC-8b ([foundation-sweep/poc-8b-findings.md](foundation-sweep/poc-8b-findings.md)) built it, one `ClosureStorage` associated type carries the callable kind (Box `FnOnce`, Rc/Arc `Fn`) under one substrate, bridged by a by-value `call_once`, so the prefix-scheme fallback is not needed and this item's primary outcome (one brand per effect, no prefix axis) is demonstrated feasible. The prefix-scheme fallback remains documented in `fp-library/docs/pointer-abstraction.md` as the recorded contingency should the full multi-arm integration in item 4 step 3 surface a blocker, but none is known. The uniform-rename fallback is unnecessary, and the status-quo-plus-legend option is rejected (the legend still ships in item 1; the misleading three-sibling scheme goes away with the rebuild).

This work is item 4 step 3.

Status: decided (unify via the FS-1 `ClosureStorage` substrate; the `FnOnce`/`Fn` reconciliation is built and passed in POC-8b, so one brand per effect is feasible without the prefix-scheme fallback); folded into item 4.

### 7. Residual wrapper-family generation (the W8 trigger, rescoped)

Findings: architecture.md section 3.6 (measured duplication); refactoring-opportunities.md R2; prior-reviews-crosscheck.md section 2 item 2 (the prior audit: 83 percent mechanical, generation deferred behind triggers).

Decision (rescoped by the sweep): under FS-1 most of the W8 mechanical surface ceases to exist rather than being generated, the boundary-frame plumbing, the per-wrapper protocol traits, the raw-scoped machinery, and the scoped-row representation are deleted (item 4 step 2), and the substrate type/interpreter/`Clone` unify into one `Run<Store: ClosureStorage, A>` (POC-8) instead of six hand-written families. What still needs generation is narrower: per-`Store` closure construction, the per-effect smart constructors, and the `bind`/`map` composition sites, because construction cannot be made generic over `Store` (the `Store::Stored` projection is not injective, [foundation-sweep/poc-8-findings.md](foundation-sweep/poc-8-findings.md), and the composition bodies differ by move-versus-clone per `Store`, [foundation-sweep/poc-8b-findings.md](foundation-sweep/poc-8b-findings.md)). The W8 trigger logic still applies to that residual surface: build the generator with a concrete consumer (the first effect-catalog port in item 4 step 4 or the first Phase D port), proving equivalence by `cargo expand` comparison.

Steps:

1. During item 4 step 3, inventory the residual generation surface (per-`Store` construction, per-effect smart constructors), updating the W8 line counts against the now-deleted boundary/protocol/scoped surface.
2. Build the construction generator with the first concrete consumer; prove expansion equivalence before switching effects over.

Status: rescoped (generation only for per-`Store` construction and smart constructors; the rest is deleted by FS-1, not generated).

### 8. Brand-keyed dispatch and one effect-spec surface

Findings: architecture.md section 3.3; refactoring-opportunities.md R1; organisation-naming-documentation.md section 2.3 (row-macro asymmetry); prior-reviews-crosscheck.md section 2 items 9 and 10 (W7's durable fix; W5's Rc/Arc macro question).

Decision (dispatch, adopted): brand-keyed dispatch is validated, POC-2 ([foundation-sweep/poc-2-findings.md](foundation-sweep/poc-2-findings.md)) shows type-level search over the row makes handler-list order irrelevant, with the missing-handler error naming the brand (the error-anchor improvement the review wanted), using the frunk Sculptor index-list pattern. Under FS-1 brand-keyed dispatch is intrinsic to the unified row (item 4 step 1), so the positional-sort footgun (R1) is eliminated, not merely mitigated, and W5's separate Rc/Arc row macros are moot (the substrate is `Store`-parameterised, so there are no per-pointer row flavours).

Remaining work (the surface macro, not yet built): an `effect_spec!`-style entry point taking the effect list once and emitting the unified-row alias, the brand-keyed handler-list type, and an order-insensitive constructor; `effects!`/`handlers!` remain low-level escape hatches.

Steps:

1. Design `effect_spec!` input syntax (effect list, optional visibility and alias names) and emitted items over the unified row.
2. Implement, with tests mirroring the row/handler macro suites plus a regression test for the spelling-mismatch scenario from the `handlers.rs` docs.
3. Update the macro documentation and the `run.md` quick-start to lead with the spec macro.

Status: brand-keyed dispatch decided (adopt, via FS-1, item 4 step 1); the `effect_spec!` surface is not started and retargets to the unified row.

### 9. Tagged (labelled) effects

Findings: coverage-gaps.md sections 2, 3, and 4 candidate 1 (no mechanism for duplicate same-type effects; both references treat labels as table stakes).

Decision (adopted, folded into dispatch): under FS-1 membership is brand-keyed (the brand is the label), which the sweep's grounding confirmed against heftia (`LabelOf`-resolved `:>` membership) and POC-2 realises as type-level brand search. A `TaggedBrand<Label, EBrand>` transparent wrapper is therefore the mechanism, a tag is a wrapper brand that changes the dispatch key, and it composes with brand-keyed dispatch rather than relying on positional-index disambiguation. The heftia-style separate key-membership family is unnecessary, and the turbofished-index status quo is rejected.

Steps:

1. Add `TaggedBrand` with delegating impls over the unified-row cell, and tagged smart-constructor support (item 4 step 1).
2. Extend `effect_spec!`/`define_effect!` and the handler macros with label syntax.
3. Add a two-States worked example to `run.md` and tests covering tagged rows and handler lists.

Status: decided (tagging is a label-brand over brand-keyed dispatch); folds into item 4 step 1 plus the macro work in items 8 and 11.

## Phase C: the FS-1 surface

### 10. Relocate effect codegen; consolidate per-effect homes

Findings: organisation-naming-documentation.md sections 1.1, 1.2, 1.3; refactoring-opportunities.md R7 and R13 (shell merging); prior-reviews-crosscheck.md section 3 (named_helpers placement disagreement).

Approaches:

- A. Mechanical move first: relocate the generator builders to `fp-macros/src/effects/codegen/` with an explicit descriptor table; `document_module` keeps only a hook that invokes effects codegen before validation. Trade-offs: pure code motion, immediately fixes discoverability; the registry design itself is unchanged until item 11 replaces it.
- B. Move and redesign in one step (fold into item 11). Trade-offs: avoids touching the files twice; couples a safe mechanical change to the riskiest macro item.

Recommendation: A, with item 11 landing into the new location. For the fp-library side: merge the by-wrapper `smart_constructors.rs` shells and the by-effect `named_helpers/` modules into single per-effect modules, so each effect has one home per crate. On the recorded disagreement with review-1 (which praised the `named_helpers` split for keeping wrapper files small): the per-effect layout wins once invocation shells are all that remains in fp-library, because locality then costs nothing in file size. Reasoning: discoverability is a maintainability concern the principles rank above incumbent layout; both moves are behaviour-preserving and verifiable by `just verify` plus expansion comparison.

Steps:

1. Move the generator builders and registry to `fp-macros/src/effects/codegen/`; rename the internal macro to `define_builtin_effect!` if item 11 has not yet landed (interim de-collision).
2. Merge per-wrapper smart-constructor shells into per-effect modules; fold `named_helpers/<effect>.rs` content into the same homes; delete the empty axes.
3. Update AGENTS.md key-locations and the architecture doc to point at the new layout.

Foundation-sweep impact: the FS-1 rebuild deletes most of the generator surface this item relocates (item 7), so sequence the codegen-relocation portion with the FS-1 macro work (item 11) rather than ahead of it, to avoid relocating code that is about to be deleted; the per-effect-home consolidation in fp-library is valuable regardless and can proceed independently.

Status: not started (the per-effect-home consolidation is FS-1-independent; the generator relocation sequences with item 11).

### 11. Public `define_effect!` and registry convergence

Findings: organisation-naming-documentation.md section 1.3 (name collision) and section 3 item 8 (custom-effects.md gaps); refactoring-opportunities.md R4; external-ideas.md item 6 (reffect's `#[group]` shape study; corophage's borrowed-resume GAT consideration).

Approaches:

- A. Extend the internal registry to accept user-provided specs in place. Trade-offs: fastest route to a user macro; bakes the documentation-subtree coupling and the name-keyed registry deeper instead of replacing them.
- B. Design a fresh public `define_effect!` taking an operation-enum-like spec and emitting the brand, the kind projection, the impls it needs, and Member-generic smart constructors; reimplement the built-in effects on it, retiring the name-keyed registry. Trade-offs: the clean end state with the built-ins as permanent conformance tests of the public macro; the largest macro work item in this plan; must preserve the documentation-attribute integration that `document_module` validation needs.
- C. Keep custom effects manual and only improve the manual guide. Trade-offs: zero macro work; the eight-step boilerplate stays the price of entry, which the existence of fourteen macro-generated built-ins makes hard to justify.

Recommendation: B, sequenced after (or together with) item 10's relocation so the new macro is born in the right module. Reasoning: the run.md condition for shipping the macro ("after more custom examples prove the generated shape") is met by the built-ins themselves; converging user and built-in paths onto one macro eliminates the collision, the registry's name-keying, and the custom-effects ergonomics gap in one move. The spec syntax should take reffect's trait-like grouping as a shape reference.

Steps:

1. Specify the macro input (operations, continuation positions, payload kinds, drop policy) and output; validate the spec against the three hardest built-ins (State, Choose, Coroutine) and one higher-order effect.
2. Implement; port two built-ins as proof, comparing expansion against the current generators (the cargo-expand equivalence discipline from the W2/W8 record).
3. Port the remaining built-ins; delete the superseded generator builders.
4. Rewrite `custom-effects.md` to lead with the macro, keeping the manual pattern as the explanatory appendix with the item-1 corrections.

Foundation-sweep impact: retarget the public macro to the FS-1 shape, it emits a brand over the unified row plus the `Functor`/`WrapDrop` it needs, brand-keyed membership, and the per-`Store` smart constructors against the `ClosureStorage` substrate (the residual generation surface from item 7), not the dual-row `Run`/`RcRun`/`ArcRun` impl set. The impl set shrinks because the substrate type/interpreter/`Clone` no longer vary per wrapper. The "validate against one higher-order effect" sub-step uses an in-row elaborated cell (item 4 step 2) rather than a scoped-boundary effect. Sequence after item 4.

Status: not started (retarget to the FS-1 effect shape; after item 4).

### 12. Remove the `run`/`run_rec` alias methods

Findings: organisation-naming-documentation.md section 2.2; refactoring-opportunities.md R8.

Decision (adopted): delete the aliases, keep `handle`/`handle_rec`, and put the purescript-run name correspondence in a `run.md` table. The principles reject compatibility shims for an unstable API; `handle` matches the subsystem's own vocabulary. The deprecate-first and keep-both options are rejected (the subsystem is experimental and feature-gated, so a deprecation period protects nobody).

Foundation-sweep impact: the FS-1 rebuild re-authors the run surface on one `Store`-parameterised substrate, so name the methods once on `Run<Store, A>` as part of item 4 step 4, rather than deleting aliases six times on the dual-row wrappers first. If FS-1 is delayed, this remains a valid standalone break on the current wrappers.

Status: decided (delete the aliases); execute on the FS-1 surface (item 4 step 4), or standalone if FS-1 is delayed.

### 13. Rename `handle_with_either`

Findings: organisation-naming-documentation.md section 2.3; refactoring-opportunities.md R9; coverage-gaps.md section 2 (relationship to `run_except`).

Decision (adopted): rename to a driver-style name (for example `drive_or_intercept` or `handle_all_or_intercept`) and document that `run_except` is the runExcept-shaped narrowing API; cross-link the two. The cross-check against the generated runners showed the either-shaped narrowing already exists as `run_except`, so the method's real value is interception (driving the whole program but surrendering the first matched operation with its continuation intact), and only its name is wrong. Reshaping it into a narrowing combinator (which would duplicate `run_except` and lose interception) and deleting it (which loses interception) are rejected.

Foundation-sweep impact: as with item 12, this rename belongs to the FS-1 surface design (item 4 step 4), defined once on `Run<Store, A>`, unless FS-1 is delayed and the standalone break is wanted sooner.

Status: decided (rename, keep interception); execute on the FS-1 surface (item 4 step 4), or standalone if FS-1 is delayed.

### 14. Nondeterminism semantics: threaded-accumulator runners

Findings: architecture.md section 3.7 (cell-based runners fix global-across-branches semantics; branch-local State/Writer under `Choose` is inexpressible; `run.md` claims an ordering distinction nothing implements); coverage-gaps.md section 2 (`runAccum` family absent) and section 5 (missing NonDet zoo cases); prior-reviews-crosscheck.md section 4.

Approaches:

- A. Documentation-only: declare shared-cell semantics the library's single semantics and correct `run.md`. Trade-offs: cheap and immediately honest; permanently narrows expressiveness against both reference systems, contradicts the heftia-parity ambition, and forecloses the scoped-choice port (item 17).
- B. Pure threaded-accumulator interpreters on the multi-shot wrappers (a `handle_accum`-style loop threading `s` through interpretation; the `Choose` handler re-enters per branch so each branch forks the accumulator), with `run_state_threaded`/`run_writer_threaded`-style runners on top, plus the heftia NonDet zoo cases as tests. Trade-offs: the real fix, matching purescript-run's `runAccum` family; moderate effort.
- C. Snapshot-and-restore cells (the MpEff `mpromptIORef` pattern). Trade-offs: smaller change than B; couples correctness to handler discipline rather than to the interpreter.

Recommendation: B, staged, with A's documentation correction done immediately (inside item 1 step 3) so the docs never overpromise, and C recorded as the documented fallback for handlers that cannot be expressed in threaded form. Reasoning: this is the one finding where the implementation cannot express semantics the project's stated inspirations treat as definitional; the correctness and coherence principles put it ahead of every convenience item.

Steps:

1. Design the `handle_accum` signature (accumulator type, step shape, how `Choose`'s per-branch re-entry forks `s`).
2. Implement the threaded `run_state`/`run_writer`/`fold_writer` variants; the cell-based ones keep their names and gain a documented "shared across branches" sentence.
3. Port heftia's NonDet-x-Writer and NonDet-x-State zoo cases (both handler orders) into `run_heftia_semantics.rs`.
4. Add `run_nondet_monoid`-style folding runners (trivial over the threaded loop).

Foundation-sweep impact: the scoped-boundary question that made this risky ("where does `s` live while a selected action runs") is largely retired by FS-1, with the boundary frames gone, a selected action is just a sub-program the interpret pass runs, and POC-9 shows the interpreter shares or scopes its accumulator at the recursive call (a shared cell survives a catch; a fresh local accumulator scopes a censor). The threaded-versus-shared choice is the branch-local-versus-global (R5) axis, for which FS-1's weave (FS-2, held in reserve) is the principled order-dependent route. Re-target the runners onto the unified row; sequence after item 4.

Status: not started (reframed onto FS-1; lands after item 4).

## Phase D: ports on FS-1

### 15. Small parity ports: `transact_state`, `subsume`, `run_cont`

Findings: coverage-gaps.md sections 2, 3, and 4 candidates 3, 6, and 7.

Approaches and recommendations per port:

- `transact_state` (state snapshot/rollback around an action): (a) scoped handler over the existing State effect; (b) interpose-based rewrite. Recommendation: (a); it matches heftia's semantics (snapshot at entry, restore via put on exit). Under item 14's threaded runners the transactional semantics must be specified against both runner families.
- `subsume` (merge a duplicate effect occurrence into an existing row entry): (a) implement as a row traversal now; (b) defer until item 9 lands and reassess, since labels remove the main source of accidental duplicates. Recommendation: (b); implement only if a concrete need survives tagging.
- `run_cont` (CPS interpreter family): (a) full `runCont`/`runAccumCont` family; (b) minimal callback-driver variant first. Recommendation: (b); the minimal variant proves the shape and serves the callback-target use case.

Steps:

1. Implement `transact_state` with tests against both cell-based and threaded State runners. Status: not started.
2. Re-evaluate `subsume` after item 9; record the outcome here. Status: not started.
3. Implement the minimal `run_cont` variant with one callback-style example. Status: not started.

Foundation-sweep impact: under FS-1, `transact_state` is an in-row elaborated cell (the State-with-Catch composition POC-9 exercised), and `subsume` is brand-keyed row narrowing rather than positional traversal. Target the unified row; after item 4.

Status: not started.

### 16. Streaming on Coroutine

Findings: coverage-gaps.md section 4 candidate 4 (upstream purescript-run `Run.Streaming`; heftia `Machinery`).

Approaches:

- A. Port upstream purescript-run's `Run.Streaming` (producer/consumer/transformer over the existing Yield substrate, `connect`/`for`-substitution). Trade-offs: simple, proven upstream, fits the current synchronous interpreter; no concurrency story.
- B. Port heftia's `Machinery` (Arrow-composed Input/Output machines). Trade-offs: richer composition and a concurrency-ready shape; depends on `Parallel`, which item 18 keeps deferred.
- C. Both, layered. Trade-offs: maximal coverage; B's prerequisite still gates it.

Recommendation: A first; revisit B after item 18 resolves the Parallel criteria. Reasoning: A delivers user value on the existing substrate now; B without `Parallel` would be an API without its point.

Steps:

1. Design the streaming row vocabulary over Coroutine (producer/consumer/transformer aliases, connect, for-substitution).
2. Implement with examples and tests.

Foundation-sweep impact: target the unified row; Coroutine is a first-order effect on FS-1, so streaming is row vocabulary plus handlers with no boundary machinery. After item 4.

Status: not started.

### 17. Scoped choice (`ChooseH` analog)

Findings: coverage-gaps.md section 3 and section 4 candidate 5.

Decision-shaping: under FS-1, scoped choice is an in-row higher-order cell elaborated into first-order `Choose` operations (heftia's `runChooseH` shape, the elaboration approach), keeping one source of truth for nondeterminism semantics; the native-second-implementation alternative is rejected as it would duplicate NonDet semantics. This depends on item 14 (its observable semantics in the zoo tests depend on the threaded accumulators) and on item 4 (the elaboration mechanism).

Steps:

1. Define the scoped-choice cell and its elaboration into `Choose` over the unified row (item 4 step 2).
2. Port the heftia zoo cases that involve scoped choice.

Foundation-sweep impact: the per-wrapper-elaboration tax the original framing feared is gone, FS-1 elaborates once over the unified row (POC-5/POC-9), not per wrapper family. After items 4 and 14.

Status: not started.

## Phase E: the later exponential round (bounded by the E5 catalogue)

### 18. Async continuation and runtime-policy extension

Findings: architecture.md section 3.9; coverage-gaps.md sections 3 and 4 candidate 9; prior-reviews-crosscheck.md section 2 item 3 (the adopted W13 policy and its explicitly open items).

This item extends the adopted W13 policy rather than creating a new policy document. The W13 record already establishes: continuation-as-data async driver, no `MonadRec`-over-`Future`, runtime-agnostic public surface.

Approaches per open sub-item:

- Explicit-family async surface: (a) implement now; (b) wait for demand. Recommendation: (a) at the next async touch, on the FS-1 substrate.
- Multi-shot (Rc/Arc) async: (a) cloneable cached (`Shared`-style) futures so multi-shot stores can re-await; (b) document async as single-shot-only. Recommendation: decide (a) versus (b) as a written decision before any implementation.
- Scoped-under-async: needs an async-aware dispatch design. Recommendation: simplified under FS-1 (no scoped boundary frames to await through); design note after item 14 stabilises the runner surface.
- Deferred runtime-sensitive effects (Unlift, Provider, Parallel, Timer, Subprocess) and a general `Io` base-lift effect: extend the policy with written eligibility criteria per effect. On `Io`: keep the captured-cell idiom as the blessed mechanism and revisit `Io` together with Unlift; document the idiom prominently (item 1 already adds it to `run.md`).

Foundation-sweep impact: `Unlift` and the continuation-capturing async surface are exponential higher-order effects per the E5 catalogue ([foundation-sweep/polynomial-exponential-catalogue.md](foundation-sweep/polynomial-exponential-catalogue.md)), so this overlaps the exponential round (item 19). Target the FS-1 unified row; sequence after item 4.

Status: not started (exponential-round-adjacent; target FS-1, after item 4).

### 19. Delimited continuations (CC/Shift) design round

Findings: coverage-gaps.md section 4 candidate 8; prior-reviews-crosscheck.md section 2 item 4 (answer-type-polymorphic capture is the hard part); external-ideas.md items 2 and 5 (MpEff prompts and state-snapshot discipline; switch-resume's one-shot async shape).

Approaches:

- A. heftia-shaped `CC`/`Shift` effects on the multi-shot stores, with the answer type carried in the effect type (`Shift<Ans, Op>`), handlers built on captured cloneable continuations. Trade-offs: closest to the reference; the answer-type management is the open design problem.
- B. One-shot `shift` on the Box store via the async driver (the continuation is the rest of the `run_async` future). Trade-offs: cheap single-prompt capability; single-shot and async-coupled, so it complements rather than replaces A.
- C. Defer entirely. Trade-offs: nothing to maintain; the flagship heftia capability stays absent with no recorded path.

Recommendation: a design note targeting A, evaluating B as a complementary Box-store capability, gated on item 18 (async direction fixed); implementation is out of this plan's scope until that note is adopted. Reasoning: this is the one port that changes the semantic model; the principles' fallback rule applies in advance, so the design note must state what mono-in-A makes impossible and what the chosen encoding gives up.

Steps:

1. Write the design note (answer-type encoding, Box-store impossibility statement, MpEff state-snapshot guidance for handler authors).
2. Adopt or reject; on adoption, plan implementation as its own item list here.

Foundation-sweep impact: CC/Shift are exactly the exponential higher-order effects the sweep scoped out and the E5 catalogue bounds, so this is the later exponential round. It is no longer gated on the row decision (settled, FS-1), and the design note's "interaction with boundary frames" premise is gone (FS-1 has none); target the unified row plus the heftia `Shift`/`CC` continuation machinery (the W8/W13 reference). Still gated on item 18's async direction.

Status: not started (the later exponential round, on FS-1).

## Phase F: hygiene

### 20. Substrate hygiene

Findings: organisation-naming-documentation.md section 1.4 (43 dead_code allowances, in-flight scaffolding); architecture.md section 3.4 (single-shot panic guard; downcast invariant spread over three modules); refactoring-opportunities.md R13.

Approaches and recommendations per sub-item:

- Scaffolding sweep: delete the dead allowances and the `ExplicitBoundaryOf` compatibility alias (the no-shims principle applies); for anything kept, rewrite its reason self-containedly (item 1 covers the W-label instances).
- Single-shot guard: (a) spike a `SingleShotOp` marker trait bounding Box-store `lift`, turning the runtime panic into a compile error for multi-hole effects; (b) document the runtime panic in `custom-effects.md` and keep the guard. Recommendation: spike (a); if the bound proves infectious across generic code, adopt (b) and record the limitation.
- Downcast audit surface: consolidate every constructor that touches the `TypeErasedValue` pairing invariant into the representation module, shrinking the surface a soundness audit must read.

Steps:

1. Sweep scaffolding after item 4. Status: not started.
2. Run the `SingleShotOp` spike; adopt or document. Status: not started.
3. Consolidate or document the downcast-invariant call sites. Status: not started.

Foundation-sweep impact: most of this hygiene is consumed by the FS-1 rebuild rather than done separately, the boundary-carrier scaffolding and the `ExplicitBoundaryOf` alias are deleted with the boundary subsystem (item 4 step 2), and the `TypeErasedValue` downcast surface is re-authored by the `ClosureStorage` substrate (item 4 step 3). The `SingleShotOp` spike survives independently (it bounds single-hole effects regardless of row design). Sweep the residue after item 4.

Status: not started (mostly absorbed by item 4; the `SingleShotOp` spike survives independently).
