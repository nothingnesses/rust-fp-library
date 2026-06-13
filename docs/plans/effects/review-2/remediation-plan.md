# Effects System Remediation Plan (review-2), Draft

Status: draft, awaiting adoption. No step has been started.

This plan turns the review-2 findings into work items. The findings themselves live in [README.md](README.md), [architecture.md](architecture.md), [organisation-naming-documentation.md](organisation-naming-documentation.md), [coverage-gaps.md](coverage-gaps.md), [refactoring-opportunities.md](refactoring-opportunities.md), [external-ideas.md](external-ideas.md), and [prior-reviews-crosscheck.md](prior-reviews-crosscheck.md); each work item cites the findings it addresses. Where a finding was already analysed by the review-1 remediation work, the item builds on that record (notably `../review-1/w8-consolidation-feasibility.md` and the W13 material in `../review-1/remediation-plan.md`) instead of re-deciding from scratch.

For every item the plan records the viable approaches, their trade-offs, the recommended approach, and the reasoning, so adoption is a matter of confirming or overriding recommendations rather than reconstructing analysis.

## Project Principles

- API-breaking changes are acceptable when they lead to a better end state.
- The project prioritises correctness, internal coherence, technical debt reduction, maintainability and long-term architecture over compatibility shims for existing implementation details.
- When a local compatibility-preserving fix conflicts with a cleaner architecture, choose the cleaner architecture unless a concrete limitation prevents it.
- If such a limitation forces a fallback, document the limitation, the trade-offs, and the fallback before adopting it.

## Documentation Protocol

This plan intentionally does not use a separate section for previously resolved decisions. Resolved or adopted decisions should be folded into the implementation plan as concrete steps. This keeps the document from accumulating stale decision history while still making the intended work clear.

During implementation, this document should be kept up to date by indicating, for each of the implementation steps, what their current status is.

Status convention: each work item carries a `Status:` line (`not started`, `in progress`, `blocked on <item or question>`, `complete`, or `rejected with reason`). While an item is in progress, its individual numbered steps are annotated in place (for example `2. (done) ...`). When an item completes, its recommendation text may be compressed to the adopted decision plus a pointer to the implementing commits.

## Open Questions, Decisions, Issues and Blockers

None outstanding (this will be populated with a numbered list of items in the future).

## Implementation Steps

### Sequencing overview

Items are ordered into phases by dependency and risk, not by importance alone.

- Phase A, accuracy first (no design risk): items 1 to 4.
- Phase B, one-time API breaks while the subsystem is still experimental: items 5 to 7.
- Phase C, macro and codegen architecture: items 8 to 11.
- Phase D, strategic decisions: items 12 to 15.
- Phase E, ports (several of which fire the wrapper-generation trigger): items 16 to 19.
- Phase F, hygiene: item 20.

Hard dependencies: item 11 depends on items 8 and 9; item 14 is scheduled to land with the first port from items 17 or 18; item 18 depends on item 2; item 19 depends on items 12 and 15. Everything in Phase A can start immediately and in parallel.

### 1. Documentation accuracy and self-containedness sweep

Findings: organisation-naming-documentation.md sections 3 and 4 (eight drift items, five plan-label violations); refactoring-opportunities.md R6 and R10; prior-reviews-crosscheck.md section 2 items 1, 5, and 8 (missing limitations and legend).

Approaches:

- A. One consolidated documentation commit fixing every listed item. Trade-offs: a single reviewable change, immediate end to actively misleading docs (the `interpreter.rs` text currently denies a shipped feature); the commit touches many files at once, but every change is prose.
- B. Fix opportunistically as each module is next touched. Trade-offs: smaller diffs per commit, but wrong documentation stays live indefinitely, and several items (the `run.md` catalog, the stale async section) sit in files with no scheduled code changes.

Recommendation: A. Documentation in this codebase is enforced product (the doc pipeline rejects drift it can detect; these items are the ones it cannot). Leaving known-false statements live contradicts the correctness principle, and the change carries zero regression risk. Verification for the doc-only commit is `just fmt && just doc`.

Steps:

1. Fix the stale claims: `node.rs` "future work" text and Functor-impl note; `interpreter.rs` async section (describe `Await`/`run_async`, delete the spawn_blocking framing or demote it to a sidebar about effects other than `Await`); the `CatchHandler` "Rc/Arc only" paragraph; the future-tense "W11 runner" docs in `kv_store.rs`, `fresh.rs`, `input.rs`; the `Run::handle` per-layer recursion claim (state the actual recursion budget: scoped nesting depth).
2. Purge plan-workstream labels and external URLs from source docs: `await_future.rs`, `async_interpreter.rs`, `row_embed.rs`, the three effect modules above, and the `run_heftia_semantics.rs` module header (restate ported cases concretely; move provenance URLs into this plans area).
3. Refresh `fp-library/docs/run.md`: complete the effect catalog (fourteen first-order effects, eight scoped handler families), document the generated runner families as the primary API, add the brand-prefix legend (lift the table from organisation-naming-documentation.md section 2.1), describe the Rc/Arc row-construction path, and correct the Handler Order paragraph per item 2 below.
4. Document `expand`/`weaken` cost in `run.md` by lifting the rationale from `../review-1/w1-row-embed-spike.md`, and present Member-generic row-polymorphic authoring as the preferred pattern with one worked example of each style.
5. Extend the limitations lists (`run.md`, `types/effects.rs`): generic scoped rows are deferred (`define_scoped_row!` is concrete-only), the Fn versus FnOnce capture asymmetry between handlers and `bind`, and the single-continuation-hole requirement for Box-family effects (also added to `custom-effects.md`, together with the `SendFunctor`/`RefFunctor`/`Extract` requirements for shared wrappers).
6. Clarify the `define_effect!` situation in `run.md`/`custom-effects.md` (an internal registry-keyed macro exists; the public macro is item 9).

Status: not started.

### 2. Nondeterminism semantics: threaded-accumulator runners

Findings: architecture.md section 3.7 (cell-based runners fix global-across-branches semantics; branch-local State/Writer under `Choose` is inexpressible; `run.md` claims an ordering distinction nothing implements); coverage-gaps.md section 2 (`runAccum` family absent) and section 5 (missing NonDet zoo cases); prior-reviews-crosscheck.md section 4.

Approaches:

- A. Documentation-only: declare shared-cell semantics the library's single semantics and correct `run.md`. Trade-offs: cheap and immediately honest; permanently narrows expressiveness against both reference systems, contradicts the heftia-parity ambition, and forecloses the ChooseH port (item 18) whose semantics depend on threading.
- B. Pure threaded-accumulator interpreters on the multi-shot wrappers (`handle_accum`-style loop threading `s` through interpretation; the `Choose` handler re-enters per branch so each branch forks the accumulator), with `run_state_threaded`/`run_writer_threaded`-style runners built on top, plus the heftia NonDet zoo cases as tests. Trade-offs: the real fix, matching purescript-run's `runAccum` family; moderate effort; one genuine design risk where a scoped boundary interleaves with the threaded accumulator (where does `s` live while a selected action runs).
- C. Snapshot-and-restore cells (the MpEff `mpromptIORef` pattern): keep cell-based handlers but snapshot the cell at capture and restore per resumption. Trade-offs: smaller change than B; semantics match pure threading for State in the common cases; couples correctness to handler discipline rather than to the interpreter, and every cell-using handler must opt in correctly.

Recommendation: B, staged, with A's documentation correction done immediately (inside item 1 step 3) so the docs never overpromise, and C recorded as the documented fallback for handlers that cannot be expressed in threaded form. Reasoning: this is the one finding where the implementation cannot express semantics the project's stated inspirations treat as definitional; the correctness and coherence principles put it ahead of every convenience item. Staging contains the risk: first-order rows on `RcRun` first, zoo tests pinning both orders, then the scoped-boundary interaction as its own design step.

Steps:

1. Design the `handle_accum` signature for `RcRun` (accumulator type, step shape, how `Choose`'s per-branch re-entry forks `s`); record the scoped-boundary question explicitly and, if unresolved, restrict the first delivery to `S = ScopedNil`.
2. Implement on `RcRun`, then `ArcRun`, then the Explicit multi-shot siblings; add threaded `run_state`/`run_writer`/`fold_writer` variants and decide their names against the existing cell-based runners (the cell-based ones keep their names and gain a documented "shared across branches" sentence).
3. Port heftia's NonDet-x-Writer and NonDet-x-State zoo cases (both handler orders) into `run_heftia_semantics.rs`.
4. Add `run_nondet_monoid`-style folding runners (trivial over the threaded loop; closes the corresponding coverage-gaps row).
5. Resolve the scoped-boundary interaction (design note plus implementation or a documented restriction).

Status: not started.

### 3. Run-level test hardening: stack safety and laws

Findings: architecture.md section 3.8 (no deep `Run` program tests; `stack_safety.rs` covers other types); coverage-gaps.md section 5 (laws untested).

Approaches:

- A. Targeted suites: deep-program tests (on the order of 100k steps) through `handle`, `handle_rec`, chained `handle_with`, `interpose`, and `expand`, on Box and Rc families; QuickCheck law tests (Functor/Monad laws on small concrete rows; `expand` naturality against handler results). Trade-offs: bounded effort, directly pins the guarantees the docs make.
- B. A random-program generator producing arbitrary well-typed effect programs for property testing. Trade-offs: far stronger coverage in principle; generating well-typed programs over type-level rows is a project in itself.

Recommendation: A now; consider B only if A starts finding classes of bugs that targeted tests miss. Reasoning: the open risks are specific (recursion over program depth hiding in traversals; queue rotation under boundary frames), and targeted tests reach them at a fraction of B's cost.

Steps:

1. Add `run_stack_safety.rs` with deep-bind, deep-handle, deep-`handle_with`-chain, deep-`interpose`, and deep-`expand` cases on `Run` and `RcRun`; include one deep program inside a `Catch` boundary (this doubles as the CatList rotation check flagged in external-ideas.md item 3).
2. Add law property tests for the wrappers on a two-effect row, including `expand` naturality.
3. While writing step 1, verify or refute the corrected recursion-budget claim from item 1 step 1 (host stack grows with scoped nesting only).

Status: not started.

### 4. Benchmark expansion and the tail-resumptive investigation

Findings: coverage-gaps.md section 5 (benchmark gaps); refactoring-opportunities.md R11; external-ideas.md items 1 and 6 (EvEff/koka taxonomy; corophage's dispatch-position calibration).

Approaches:

- A. Extend the existing criterion suites (`effect_rows.rs`, `scoped_operations.rs`) with dispatch-position cost, deep-bind throughput versus plain `Free`, `expand` cost versus program size, and boundary-frame overhead; only then decide whether a tail-resumptive fast path is worth designing. Trade-offs: measurement before design; the fast path may turn out unnecessary (corophage's data suggests positional dispatch is cheap and the costs live in allocation and queue traffic).
- B. Design the fused fast path now on the EvEff/koka argument alone. Trade-offs: saves a measurement round if the result is positive; risks engineering against the wrong bottleneck.

Recommendation: A. Reasoning: the architecture already inlines continuations in the common path (handlers call the continuation synchronously); whether the remaining Coyoneda-lowering and queue costs matter is an empirical question, and the project's benchmarking infrastructure makes answering it cheap.

Steps:

1. Add the four benchmark families to the criterion suites.
2. Write up results in `fp-library/docs/benchmarking.md` or a plans note; decide go/no-go on a fused fast path.
3. If go: design the fast path as an internal specialization of the runner loops (no public API change), with before/after numbers.

Status: not started.

### 5. Remove the `run`/`run_rec` alias methods

Findings: organisation-naming-documentation.md section 2.2; refactoring-opportunities.md R8.

Approaches:

- A. Delete the aliases now, keep `handle`/`handle_rec`, and put the purescript-run name correspondence in a `run.md` table. Trade-offs: one mechanical breaking change across six wrappers and any internal call sites; API halves in size.
- B. Deprecate first, remove later. Trade-offs: softer landing for hypothetical external users; the subsystem is feature-gated, experimental, and explicitly unstable, so the deprecation period protects nobody and leaves the double surface live.
- C. Keep both. Trade-offs: zero work; permanent doubled documentation and a permanent "which is canonical" question.

Recommendation: A. Reasoning: the principles reject compatibility shims for an unstable API; `handle` matches the subsystem's own vocabulary (handlers, handler lists). The cross-reference value of the PureScript names is fully preserved by a documentation table.

Steps:

1. Delete `run`/`run_rec` from the six wrappers; migrate internal uses and doctests.
2. Add the purescript-run correspondence table to `run.md`.

Status: not started.

### 6. Rename `handle_with_either`

Findings: organisation-naming-documentation.md section 2.3; refactoring-opportunities.md R9; coverage-gaps.md section 2 (relationship to `run_except`).

Approaches:

- A. Rename to a driver-style name (for example `drive_or_intercept` or `handle_all_or_intercept`) and document that `run_except` is the runExcept-shaped narrowing API. Trade-offs: one rename; the genuinely distinct capability (drive the whole program but surrender the first matched operation, continuation intact) keeps an honest name.
- B. Reshape it into a narrowing combinator returning `Run<RMinusE, S, Result<...>>`. Trade-offs: aligns the name and the shape, but duplicates what the generated `run_except` already provides, and loses the interception capability (the returned matched operation with live continuation), which nothing else offers.
- C. Delete it. Trade-offs: smallest surface; loses the interception capability.

Recommendation: A. Reasoning: review-2 initially read this method as a misshapen runExcept; the cross-check against the generated runners shows the either-shaped narrowing already exists as `run_except`, so the method's real value is interception, and only its name is wrong. Renaming preserves a unique capability and removes the misleading overlap.

Steps:

1. Choose the name, rename across the six wrappers, update docs and doctests.
2. Cross-link the method and `run_except` in both directions, stating when to use which.

Status: not started.

### 7. Brand sibling unification or uniform renaming

Findings: organisation-naming-documentation.md section 2.1; refactoring-opportunities.md R3; prior-reviews-crosscheck.md section 3 (recorded disagreement with review-1's "coherent" assessment).

Approaches:

- A. Unify: one brand per effect, `StateBrand<PB, S>` with `PB` in `{BoxBrand, RcBrand, ArcBrand}`, over a single closure-storage class with associated dyn-closure types subsuming `ToDynFnOnce`/`ToDynCloneFn`/`ToDynSendFn`. Trade-offs: the cleanest end state (one type, one set of impls, no prefix axis); the FnOnce-versus-Fn split is a real type-system risk, because the Functor impls must compose onto a consumed `FnOnce` in one instantiation and a shared `Fn` in the others, and a single associated type may not be able to express both call shapes.
- B. Uniform rename: keep three sibling types but fix the scheme to `BoxStateBrand<S>` / `RcStateBrand<S>` / `ArcStateBrand<S>` (pointer named consistently, redundant `P` on the Box flavour dropped). Trade-offs: mechanical, large but safe rename; the three-sibling duplication remains.
- C. Status quo plus the documentation legend (legend ships in item 1 regardless). Trade-offs: no churn; the inconsistencies (prefix naming a pointer in one case and a bound in another; the bare name belonging to the non-default family) stay.

Recommendation: time-boxed spike of A (one effect, State, end to end including row macros and dispatch impls); adopt A if the spike passes `just verify` without contorting the class hierarchy; otherwise adopt B and record the exact limitation that blocked A in `fp-library/docs/pointer-abstraction.md`, per the fallback-documentation principle. C is rejected: it preserves a naming scheme the review found actively misleading for first-time users of the default wrapper.

Steps:

1. Spike A on State: define the unifying class, port the State enum and impls, compile the existing State tests against it.
2. Decide A versus B from the spike; record the decision and (if B) the limitation.
3. Roll the chosen scheme across all effects and brands; update macros, docs, and the legend.

Status: not started.

### 8. One effect-spec surface and order-independent handler dispatch

Findings: architecture.md section 3.3; refactoring-opportunities.md R1; organisation-naming-documentation.md section 2.3 (row-macro asymmetry); prior-reviews-crosscheck.md section 2 items 9 and 10 (W7's durable fix orphaned between workstreams; W5 folded the Rc/Arc macro question into the next macro redesign).

Approaches:

- A. `effect_spec!` macro: one entry point taking the effect list once and emitting the row aliases (default, Rc, Arc, scoped flavours), the handler-list type alias, and an order-insensitive handler-list constructor; `effects!`/`handlers!` remain as low-level escape hatches. Trade-offs: removes the spelling-mismatch footgun for everyone using the macro path; manual handler lists remain order-sensitive; subsumes the missing `rc_effects!`/`arc_effects!` (the spec emits all flavours, resolving W5 without new standalone macros).
- B. Brand-keyed dispatch: a type-level search (`DispatchHandlersFor<EBrand>`-style) so any handler-list order serves a row; positional lock-step remains the macro-guaranteed fast path. Trade-offs: fixes all construction paths including manual ones, and turns misalignment errors into "no handler for brand X" diagnostics (the error-anchor improvement review/2 finding 4 wanted); risks worse inference and overlapping-impl friction, especially with duplicate effect types in a row.
- C. Both, staged: A first, then B prototyped behind small-row tests.

Recommendation: C. Reasoning: A alone is low-risk and closes the documented footgun for the supported path, satisfying the durable fix the handler docs themselves call for; B is the structural fix but its inference risk is real and should be proven on a two-effect row before commitment. The prior reviews' caution applies (try the narrow fix before a broad macro empire): A is deliberately narrow, generating only aliases and a constructor, not programs or handlers.

Steps:

1. Design `effect_spec!` input syntax (effect list, optional scoped list, visibility, names for the emitted aliases) and emitted items; review against `define_effect_row_aliases!` for subsumption or coexistence.
2. Implement, with tests mirroring the row/handler macro suites plus a regression test for the spelling-mismatch scenario from the `handlers.rs` docs.
3. Prototype brand-keyed dispatch on a two-effect row; measure error quality and inference behaviour with duplicate-type rows; adopt or record the limitation.
4. Update the macro documentation and the `run.md` quick-start to lead with the spec macro.

Status: not started.

### 9. Public `define_effect!` and registry convergence

Findings: organisation-naming-documentation.md section 1.3 (name collision) and section 3 item 8 (custom-effects.md gaps); refactoring-opportunities.md R4; external-ideas.md item 6 (reffect's `#[group]` shape study; corophage's borrowed-resume GAT consideration).

Approaches:

- A. Extend the internal registry to accept user-provided specs in place. Trade-offs: fastest route to a user macro; bakes the documentation-subtree coupling and the name-keyed registry deeper instead of replacing them.
- B. Design a fresh public `define_effect!` taking an operation-enum-like spec and emitting brands, `impl_kind!`, the full impl set (`Functor`/`SendFunctor`/`RefFunctor`/`WrapDrop`/`Extract`), and Member-generic smart constructors; reimplement the built-in effects on it, retiring the name-keyed registry. Trade-offs: the clean end state with the built-ins as permanent conformance tests of the public macro; the largest macro work item in this plan; must preserve the documentation-attribute integration that `document_module` validation needs.
- C. Keep custom effects manual and only improve the manual guide. Trade-offs: zero macro work; the eight-step boilerplate stays the price of entry, which the existence of fourteen macro-generated built-ins makes hard to justify.

Recommendation: B, sequenced after (or together with) item 10's relocation so the new macro is born in the right module. Reasoning: the run.md condition for shipping the macro ("after more custom examples prove the generated shape") is met by the built-ins themselves; converging user and built-in paths onto one macro eliminates the collision, the registry's name-keying, and the custom-effects ergonomics gap in one move, which is exactly the coherent end state the principles ask for. The spec syntax should take reffect's trait-like grouping as a shape reference, and should decide explicitly whether continuation positions may borrow (corophage's `Resume<'r>`) on the Explicit family.

Steps:

1. Specify the macro input (operations, continuation positions, payload kinds, pointer-flavour applicability, drop policy) and output; validate the spec against the three hardest built-ins (State, Choose, Coroutine) and one scoped effect to scope a later `define_scoped_effect!`.
2. Implement for first-order effects; port two built-ins as proof, comparing expansion against the current generators (the cargo-expand equivalence discipline from the W2/W8 record).
3. Port the remaining first-order built-ins; delete the superseded generator builders.
4. Rewrite `custom-effects.md` to lead with the macro, keeping the manual pattern as the explanatory appendix with the item-1 corrections.

Status: not started.

### 10. Relocate effect codegen; consolidate per-effect homes

Findings: organisation-naming-documentation.md sections 1.1, 1.2, 1.3; refactoring-opportunities.md R7 and R13 (shell merging); prior-reviews-crosscheck.md section 3 (named_helpers placement disagreement).

Approaches:

- A. Mechanical move first: relocate the generator builders to `fp-macros/src/effects/codegen/` with an explicit descriptor table; `document_module` keeps only a hook that invokes effects codegen before validation. Trade-offs: pure code motion, immediately fixes discoverability; the registry design itself is unchanged until item 9 replaces it.
- B. Move and redesign in one step (fold into item 9). Trade-offs: avoids touching the files twice; couples a safe mechanical change to the riskiest macro item.

Recommendation: A now, with item 9 landing into the new location. For the fp-library side: merge the by-wrapper `smart_constructors.rs` shells and the by-effect `named_helpers/` modules into single per-effect modules, so each effect has one home per crate. On the recorded disagreement with review-1 (which praised the `named_helpers` split for keeping wrapper files small): the per-effect layout wins once invocation shells are all that remains in fp-library, because locality then costs nothing in file size. Reasoning: discoverability is a maintainability concern the principles rank above incumbent layout; both moves are behaviour-preserving and verifiable by `just verify` plus expansion comparison.

Steps:

1. Move the generator builders and registry to `fp-macros/src/effects/codegen/`; rename the internal macro to `define_builtin_effect!` if item 9 has not yet landed (interim de-collision).
2. Merge per-wrapper smart-constructor shells into per-effect modules; fold `named_helpers/<effect>.rs` content into the same homes; delete the empty axes.
3. Update AGENTS.md key-locations and the architecture doc to point at the new layout.

Status: not started.

### 11. Tagged (labelled) effects

Findings: coverage-gaps.md sections 2, 3, and 4 candidate 1 (no mechanism for duplicate same-type effects; both references treat labels as table stakes).

Approaches:

- A. `TaggedBrand<Label, EBrand>` transparent functor wrapper, with smart-constructor and macro support (`effect_spec!` accepting `label: Effect` entries; tagged variants of the per-effect constructors). Trade-offs: small, composes with the existing Member machinery (the tag makes the row entry type unique, so positional indices disambiguate naturally); label types are user-defined ZSTs, mirroring purescript-run's proxies.
- B. heftia-style key membership (a separate membership family alongside positional `Member`). Trade-offs: more faithful to heftia's three membership modes; significantly more trait machinery for the same user-visible outcome in this positional system.
- C. Status quo: duplicate effects via hand-written rows and turbofished indices. Trade-offs: nothing new to build; the documented experience is bad enough that both references invented labels to avoid it.

Recommendation: A, after items 8 and 9 so macro support is designed once. Reasoning: A delivers the user capability with the least machinery and no new membership theory; B's extra generality has no current consumer.

Steps:

1. Add `TaggedBrand` with delegating impls (`Functor`, `WrapDrop`, and companions) and tagged smart-constructor support.
2. Extend `effect_spec!`/`define_effect!` and the handler macros with label syntax.
3. Add a two-States worked example to `run.md` and tests covering tagged rows and handler lists.

Status: not started.

### 12. Dual-row decision record

Findings: architecture.md section 3.1 (divergence from the referenced heftia's unified row; misattribution in docs); refactoring-opportunities.md R12.

Approaches:

- A. Write the decision record comparing dual rows against a unified row with per-brand order markers (`KnownOrder` analog, `FOEs`-style bound on algebraic handling), covering signature noise, duplicated row machinery, diagnostics, and what each design makes impossible; keep the dual-row design unless the record concludes otherwise. Trade-offs: analysis cost only; the architecture keeps its current shape with an examinable justification.
- B. Prototype the unified row first and decide from evidence. Trade-offs: strongest evidence; a substantial speculative build on the most foundational type in the subsystem.

Recommendation: A, with B held in reserve if A finds the dual-row case genuinely weak. Reasoning: the review found no concrete defect caused by dual rows, only unexamined inheritance from an older heftia plus signature noise; that calls for a recorded decision, not a rebuild. The docs misattribution is fixed in item 1 regardless of the outcome.

Steps:

1. Write the record in this plans area, including the Rust-specific constraints (constraint-kind ergonomics, closure monomorphism) that motivated dual rows.
2. Fold the conclusion into `run.md`'s design-rationale paragraph.

Status: not started.

### 13. Brand-capability decision and wrapper-matrix scope

Findings: architecture.md section 3.6 (do all six wrappers earn their keep); prior-reviews-crosscheck.md section 2 item 7 (the W3 capability audit's undecided half: close gaps versus document them; the erased family is unbranded).

Approaches:

- A. Formalise the status quo: the capability matrix in `types/effects.rs` is the contract; gaps (no erased-family brands, partial Explicit class coverage) are documented as intentional with their type-system reasons. Trade-offs: no churn; brand-generic code over erased wrappers stays impossible.
- B. Brand the erased family. Trade-offs: would unify the matrix; requires branding the erased Free family, which its `TypeErasedValue` representation likely cannot support cleanly (the review-1 record reached the same tentative conclusion); high risk, no identified consumer.
- C. Shrink the matrix: evaluate whether the Explicit family can be the only family (erasure as an internal optimisation), or whether low-usage corners (`RcRunExplicit`, `ArcRunExplicit`) can be dropped. Trade-offs: the largest simplification available anywhere in the subsystem if it holds; needs usage evidence and an ergonomics comparison (the erased family exists for O(1) bind and `'static` ergonomics).

Recommendation: run C's evaluation first (it changes everything downstream, including item 14's scope); default to A if C's evidence is weak; treat B as rejected-unless-consumer-appears. Reasoning: deciding matrix scope before investing in generation machinery follows the technical-debt principle (do not automate the maintenance of types nobody needs); A is the honest floor either way.

Steps:

1. Gather evidence: which wrappers do the tests, docs, and any downstream code actually exercise; what would each candidate removal break.
2. Write the decision (kept families, gap policy); update the capability matrix and `run.md` wrapper guidance.

Status: not started.

### 14. Wrapper-family mechanical generation (the W8 trigger)

Findings: architecture.md section 3.6 (measured duplication); refactoring-opportunities.md R2; prior-reviews-crosscheck.md section 2 item 2 (the prior audit: 83 percent mechanical, generation deferred behind triggers: a new scoped effect, a seventh wrapper, or recurring maintenance burden).

Approaches:

- A. Honour the deferral as-is: generate nothing until a trigger fires naturally. Trade-offs: no speculative work; every port in Phase E pays the six-fold tax by hand first, then the generator is built anyway.
- B. Preemptive generation now. Trade-offs: removes the tax before the ports; contradicts the recorded W8 verdict without new evidence, and risks generating for wrappers item 13 may remove.
- C. Scheduled trigger: recognise that items 17 and 18 (new scoped effects) fire the W8 trigger by definition, so plan the generator to land together with the first of them, scoped per the W8 prescription (mechanical plumbing only; semantic handlers stay hand-written; cargo-expand equivalence proof), and after item 13 fixes the matrix scope. Trade-offs: the generator is built exactly once, against the final wrapper set, with a concrete consumer.

Recommendation: C. Reasoning: it follows the prior decision's own logic instead of relitigating it; the review's contribution is noticing that the adopted port roadmap makes the trigger condition imminent rather than hypothetical.

Steps:

1. After item 13, inventory the exact mechanical surface to generate (per-wrapper protocol traits, carriers, raw-scoped plumbing, representation boilerplate), updating the W8 line counts.
2. Build the generator alongside the first Phase E scoped-effect port; prove equivalence by expansion comparison on the existing wrappers before switching them over.
3. Regenerate the remaining families; delete the hand-written duplicates.

Status: not started.

### 15. Async continuation and runtime-policy extension

Findings: architecture.md section 3.9; coverage-gaps.md sections 3 and 4 candidate 9; prior-reviews-crosscheck.md section 2 item 3 (the adopted W13 policy and its explicitly open items).

This item extends the adopted W13 policy rather than creating a new policy document. The W13 record already establishes: continuation-as-data async driver, no `MonadRec`-over-`Future`, runtime-agnostic public surface.

Approaches per open sub-item:

- `RunExplicit` async surface: (a) implement now (recorded as mechanical and unblocked); (b) wait for demand. Recommendation: (a) at the next async touch; it is the only unblocked async increment and keeps the Explicit family at parity.
- Rc/Arc async: (a) cloneable cached (`Shared`-style) futures so multi-shot wrappers can re-await; (b) document async as single-shot-only and exclude the multi-shot families. Trade-offs: (a) is the capability users will expect but imports subtle caching semantics; (b) is honest and cheap but makes `Choose`-plus-`Await` programs impossible forever. Recommendation: decide (a) versus (b) as a written decision before any implementation; do not let an implementation default decide it.
- Scoped-under-async: needs an async-aware scoped dispatch design (the synchronous `dispatch_scoped` path cannot await). Recommendation: design note first, implementation only after items 2 and 14 stabilise the scoped surface.
- Deferred runtime-sensitive effects (Unlift, Provider, Parallel, Timer, Subprocess) and a general `Io` base-lift effect: extend the policy with written eligibility criteria per effect. On `Io` specifically: (a) add an `Io` effect now (parity with `EFFECT`/`Emb IO`); (b) keep the captured-cell idiom as the blessed mechanism and revisit `Io` together with Unlift. Recommendation: (b); an `Io` effect's design is entangled with Unlift and interruption questions, and the idiom covers current needs; document the idiom prominently (item 1 already adds it to `run.md`).

Status: not started.

### 16. Small parity ports: `transact_state`, `subsume`, `run_cont`

Findings: coverage-gaps.md sections 2, 3, and 4 candidates 3, 6, and 7.

Approaches and recommendations per port:

- `transact_state` (state snapshot/rollback around an action): (a) scoped handler over the existing State effect; (b) interpose-based rewrite. Recommendation: (a); it matches heftia's semantics (snapshot at entry, restore via put on exit) and exercises the scoped machinery it would live in. Note: under item 2's threaded runners the transactional semantics must be specified against both runner families.
- `subsume` (merge a duplicate effect occurrence into an existing row entry): (a) implement as a row traversal now; (b) defer until item 11 lands and reassess, since labels remove the main source of accidental duplicates. Recommendation: (b); implement only if a concrete need survives tagging.
- `run_cont` (CPS interpreter family): (a) full `runCont`/`runAccumCont` family; (b) minimal callback-driver variant first. Recommendation: (b); the minimal variant proves the shape and serves the callback-target use case; expand if used.

Steps:

1. Implement `transact_state` with tests against both cell-based and threaded State runners. Status: not started.
2. Re-evaluate `subsume` after item 11; record the outcome here. Status: not started.
3. Implement the minimal `run_cont` variant with one callback-style example. Status: not started.

Status: not started.

### 17. Streaming on Coroutine

Findings: coverage-gaps.md section 4 candidate 4 (upstream purescript-run `Run.Streaming`; heftia `Machinery`).

Approaches:

- A. Port upstream purescript-run's `Run.Streaming` (producer/consumer/transformer over the existing Yield substrate, `connect`/`for`-substitution). Trade-offs: simple, proven upstream, fits the current synchronous interpreter; no concurrency story.
- B. Port heftia's `Machinery` (Arrow-composed Input/Output machines). Trade-offs: richer composition and a concurrency-ready shape; depends on `Parallel`, which item 15 keeps deferred.
- C. Both, layered. Trade-offs: maximal coverage; B's prerequisite still gates it.

Recommendation: A first; revisit B after item 15 resolves the Parallel criteria. Reasoning: A delivers user value on the existing substrate now; B without `Parallel` would be an API without its point.

Steps:

1. Design the streaming row vocabulary over Coroutine (producer/consumer/transformer aliases, connect, for-substitution); confirm multi-shot wrapper requirements.
2. Implement with examples and tests; this is a new scoped-or-first-order effect family and therefore fires item 14's generation trigger; sequence accordingly.

Status: not started.

### 18. Scoped choice (`ChooseH` analog)

Findings: coverage-gaps.md section 3 and section 4 candidate 5.

Approaches:

- A. Elaboration-style: a scoped effect whose handler translates the selected action's scoped choice into first-order `Choose` operations (heftia's `runChooseH` shape). Trade-offs: reuses the existing NonDet substrate and inherits item 2's semantics work; the elaboration must be written per wrapper family until item 14 lands.
- B. Native scoped handler with its own branching execution. Trade-offs: avoids the elaboration step but duplicates NonDet semantics in a second place, which is exactly the incoherence the elaboration approach avoids.

Recommendation: A, strictly after item 2 (its observable semantics in the zoo tests depend on threaded accumulators). Reasoning: matching heftia's elaboration keeps one source of truth for nondeterminism semantics.

Steps:

1. Define the scoped effect and its elaboration handler on the multi-shot wrappers.
2. Port the heftia zoo cases that involve scoped choice.

Status: not started.

### 19. Delimited continuations (CC/Shift) design round

Findings: coverage-gaps.md section 4 candidate 8; prior-reviews-crosscheck.md section 2 item 4 (answer-type-polymorphic capture is the hard part, beyond multi-shot resumption); external-ideas.md items 2 and 5 (MpEff prompts and state-snapshot discipline; switch-resume's one-shot async shape).

Approaches:

- A. heftia-shaped `CC`/`Shift` effects on the multi-shot wrappers, with the answer type carried in the effect type (`Shift<Ans, Op>`), handlers built on captured cloneable continuations. Trade-offs: closest to the reference and to the existing substrate; the answer-type management against mono-in-A dispatch is the open design problem the priors identified.
- B. One-shot `shift` on the Box family via the async driver (the continuation is the rest of the `run_async` future; the switch-resume shape). Trade-offs: cheap single-prompt capability without touching the scoped machinery; single-shot and async-coupled, so it complements rather than replaces A.
- C. Defer entirely. Trade-offs: nothing to maintain; the flagship heftia capability stays absent with no recorded path.

Recommendation: a design note targeting A, evaluating B as a complementary Box-family capability, gated on item 12 (row architecture confirmed) and item 15 (async direction fixed); implementation is explicitly out of this plan's scope until that note is adopted. Reasoning: the priors and this review agree this is the one port that changes the semantic model; the principles' fallback rule applies in advance, so the design note must state what mono-in-A makes impossible and what the chosen encoding gives up.

Steps:

1. Write the design note (answer-type encoding, interaction with boundary frames, Box-family impossibility statement, MpEff state-snapshot guidance for handler authors).
2. Adopt or reject; on adoption, plan implementation as its own item list here.

Status: not started.

### 20. Substrate hygiene

Findings: organisation-naming-documentation.md section 1.4 (43 dead_code allowances, in-flight scaffolding); architecture.md section 3.4 (single-shot panic guard; downcast invariant spread over three modules); refactoring-opportunities.md R13.

Approaches and recommendations per sub-item:

- Scaffolding sweep: once item 9 and the boundary-carrier wiring consume or obsolete the retained pieces, delete the dead allowances and the `ExplicitBoundaryOf` compatibility alias (the no-shims principle applies); for anything kept, rewrite its reason self-containedly (item 1 covers the W-label instances).
- Single-shot guard: (a) spike a `SingleShotOp` marker trait bounding Box-family `lift`, turning the runtime panic into a compile error for multi-hole effects; (b) document the runtime panic in `custom-effects.md` and keep the guard. Recommendation: spike (a); if the bound proves infectious across generic code, adopt (b) and record the limitation. The custom-effects documentation lands in item 1 either way.
- Downcast audit surface: consolidate every constructor that touches the `TypeErasedValue` pairing invariant into the representation module (or document why the boundary and row-embed call sites must stay separate), shrinking the surface a soundness audit must read.

Steps:

1. Sweep scaffolding after item 9. Status: not started.
2. Run the `SingleShotOp` spike; adopt or document. Status: not started.
3. Consolidate or document the downcast-invariant call sites. Status: not started.

Status: not started.
