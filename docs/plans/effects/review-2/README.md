# Effects System Review 2

Review of the `Run` effects subsystem as it exists on branch `feat/effects` at commit `2e0a7417` (June 2026). This review was produced without consulting the earlier review documents under `docs/plans/effects/review` and `docs/plans/effects/review-1`; a separate cross-check against those documents is recorded in [prior-reviews-crosscheck.md](prior-reviews-crosscheck.md).

Foundation sweep outcome. After this review, the [foundation-sweep/](foundation-sweep/) investigation prototyped the most foundational decision (remediation item 12, dual rows versus a unified row) and concluded, across ten POCs and four decision gates, to adopt FS-1: a unified effect row with per-brand order markers, elaboration of higher-order effects, brand-keyed dispatch, and a single closure-storage-parameterised substrate, replacing the dual rows, boundary frames, positional dispatch, and six-wrapper duplication this review describes. The descriptive documents here (architecture.md, refactoring-opportunities.md) remain accurate as the as-built `2e0a7417` snapshot; the adopted direction and its consequences are folded into [remediation-plan.md](remediation-plan.md) (the FS-1 spine is item 12) and recorded in [foundation-sweep/charter.md](foundation-sweep/charter.md).

## Scope and method

Reviewed code and documents:

- `fp-library/docs/run.md` and `fp-library/docs/custom-effects.md`.
- `fp-library/src/brands/effects.rs`.
- `fp-library/src/types/effects.rs` and all submodules (six wrapper families, interpreter family, scoped-handler subsystem, effect modules, named helpers, row machinery).
- `fp-macros/src/effects.rs` and submodules (`effects_macro`, `handlers`, `im_do`, `row_aliases`, `row_sort`, `scoped_row`), plus the effect code generators under `fp-macros/src/documentation/generator_builders/`.
- Integration tests under `fp-library/tests/` and inline test modules.

Method: direct reading of the above, the `just item-inventory` output over the effects paths, line-level diff comparisons between wrapper families, and comparative study of the reference codebases at `~/Documents/projects/effects/` (heftia, purescript-run, rust-effects-denful, effect-rs, effect-lite, and a survey of the remaining twelve projects).

## Documents

- [architecture.md](architecture.md): assessment of the core design; substrate, dual rows, dispatch model, boundary frames, wrapper matrix, semantics, stack safety, async.
- [organisation-naming-documentation.md](organisation-naming-documentation.md): code organisation, naming critique, and a concrete list of documentation drift.
- [coverage-gaps.md](coverage-gaps.md): catalog of what exists, coverage matrices against purescript-run and heftia, and prioritized port candidates.
- [refactoring-opportunities.md](refactoring-opportunities.md): prioritized refactorings with rationale, expected payoff, and risk.
- [external-ideas.md](external-ideas.md): ideas worth adopting (or explicitly avoiding) from rust-effects-denful, effect-rs, effect-lite, and the wider effects-implementation survey.
- [prior-reviews-crosscheck.md](prior-reviews-crosscheck.md): fact-checked items from the earlier reviews that remain relevant and were not independently rediscovered above.

## Executive summary

The subsystem is a serious, well-tested, and largely faithful port of purescript-run's first-order core, extended with an action-scoped subset of heftia's higher-order effects and an async base-lift effect. The headline findings:

1. The single largest cost is the six-fold wrapper matrix. `run`, `rc_run`, `arc_run`, and their three Explicit siblings are six hand-maintained, 60 to 75 percent mutually identical module families totalling roughly 35k lines with their submodules. Every effect, runner, protocol trait, and scoped carrier multiplies by six. See [architecture.md](architecture.md) section 4 and [refactoring-opportunities.md](refactoring-opportunities.md) R2.
2. Handler dispatch is positional, which forces the row and handler macros to sort entries by a syntactic structural key. The sort is spelling-sensitive (a row written with `crate::brands::Foo` and a handler written with `Foo` sort differently), and the handler docs themselves call this a footgun whose durable fix is a single effect-spec macro. That fix should be promoted to a concrete plan item. See [refactoring-opportunities.md](refactoring-opportunities.md) R1.
3. The dual-row design (`R` first-order, `S` scoped) matches heftia's older architecture, not the heftia checkout in the references directory, which uses one unified row with `KnownOrder`/`FOEs` constraints. The divergence may still be the right call for Rust, but the docs claim "heftia's pattern" without qualification, and the trade-off deserves an explicit decision record. See [architecture.md](architecture.md) section 3.1.
4. The standard `State`/`Writer` runners are implemented with shared interior-mutability cells, so the branch-local (purely threaded) semantics that heftia and purescript-run obtain by reordering handlers around NonDet is currently inexpressible; `run.md`'s claim about Writer-inside-NonDet ordering is not backed by an implementation mechanism or a test. See [architecture.md](architecture.md) section 3.7.
5. Documentation drift is widespread: stale "future work" claims in `node.rs`, an `interpreter.rs` module doc that denies the async interpreter exists, a stale "Rc/Arc only" claim on the Catch handler, future-tense "W11 runner" docs for runners that already shipped, and a `run.md` effect catalog that lists six of the fourteen shipped first-order effects. Plan-workstream labels (W1, W11, W13) appear in source docs despite the project's self-containedness rule. See [organisation-naming-documentation.md](organisation-naming-documentation.md) section 3.
6. Effect definitions and wrapper methods are generated by internal `define_effect!`/`define_run_wrapper!` macros whose generator code lives under `fp-macros/src/documentation/`, which makes the de facto effect codegen system hard to discover and overloads the documentation subtree. The internal `define_effect!` also collides with the documented future user-facing macro of the same name. See [organisation-naming-documentation.md](organisation-naming-documentation.md) section 1.
7. Brand naming is inconsistent along the pointer axis: `BoxStateBrand<P, S>` carries a redundant `P` that can only be `BoxBrand`, the unprefixed `StateBrand` means the Rc flavour, and `Send` prefixes mean Arc. A single brand per effect parameterized by a closure-storage brand is the cleaner end state if the type system permits it. See [refactoring-opportunities.md](refactoring-opportunities.md) R3.
8. Coverage is broader than the docs admit (fourteen first-order effects, eight scoped handler families, full runner sets, a ported heftia semantics suite), but the genuinely missing pieces relative to the references are: named/tagged duplicate effects, delimited-continuation effects (CC/Shift), `ChooseH`-style scoped choice, `transactState`, Unlift/Provider, pure accumulator runners, `runCont`, and upstream purescript-run's `Run.Streaming`. See [coverage-gaps.md](coverage-gaps.md).
9. `expand`/`weaken` are deep structural traversals of the whole program, where purescript-run's `expand` is a zero-cost coercion. The cost is inherent to the typed-coproduct representation but is undocumented; the row-polymorphic authoring style that avoids it (writing programs against `Member` bounds) should be the documented default. See [architecture.md](architecture.md) section 3.5.
10. From the newer Rust effect crates, the genuinely valuable imports are denful's explicit Resume/Abort handler-result split and condition/restart effect, and effect-rs's `Cause<E>` error tree and interruption-masking discipline (relevant when async and parallel arrive); effect-lite is dependency injection, not algebraic effects. The wider survey points at EvEff/koka's tail-resumptive fast path as the most actionable performance idea. See [external-ideas.md](external-ideas.md).

A point worth stating explicitly because the rest of this review is critical in tone: the engineering quality of what exists is high. The boundary-frame mechanism for single-shot scoped handlers is a real contribution not present in any of the surveyed Rust libraries; the drop-safety story (`WrapDrop`), the feature-gate diagnostics, the canonical-ordering macros, the ported heftia semantics tests, and the breadth of runner coverage are all signs of a system built deliberately rather than accreted.
