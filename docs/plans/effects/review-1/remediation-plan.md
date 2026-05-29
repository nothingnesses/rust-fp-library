# Effects System Remediation Plan (review-1)

An implementation plan derived from [`findings.md`](findings.md). Each
work item states its adopted goal and the concrete steps to reach it,
with the justification for each step carried inline. The
[Open Questions, Decisions, Issues and Blockers](#open-questions-decisions-issues-and-blockers)
section holds only the decisions the plan cannot yet act on.

This is still a working plan: the open decisions below gate parts of the
work, and the milestone sequencing is a proposal.

## Guiding principles

- API-breaking changes are acceptable when they lead to a better end
  state. The design, plan, and implementation prioritise correctness,
  internal coherence, and long-term architecture over compatibility
  shims for existing implementation details. `fp-library` is pre-1.0;
  when a local compatibility-preserving fix conflicts with a cleaner
  architecture, choose the cleaner architecture unless a concrete Rust
  type-system, safety, or proc-macro limitation prevents it. If such a
  limitation forces a fallback, document the limitation, the trade-offs,
  and the fallback before adopting it.
- Optimise for the smallest coherent end state, not the smallest diff. A
  finding is "addressed" when the architecture is right, not when the
  symptom is patched.

## Document conventions

- This plan intentionally does not use a separate "Resolved Decisions"
  section. Resolved or adopted decisions are folded into the
  implementation plan as concrete steps.
- The Open Questions, Decisions, Issues and Blockers section lists only
  items the plan does not yet act on: decisions whose sole next step is
  to make a choice. Once an item has concrete plan steps and next steps,
  its reasoning is folded into those steps as the justification for why
  the step exists, and the item is dropped from the list.
- Work items carry that justification inline and refer to any still-open
  decision by its title, never by list position. The Open Questions list
  can therefore be reordered or renumbered freely without breaking any
  reference.
- The plan is action-oriented. When analysis is converted into steps, the
  analysis is trimmed to the residual reasoning that keeps a rejected
  option from being reintroduced. Historical trace belongs in commit
  messages and git log.

## Root-cause framing

Most findings trace back to a single root: the substrate cross-product
(six wrappers, per-effect closure-storage siblings) is materialized by
hand rather than generated. Row-subsumption duplication, brand and impl
consistency, helper drift, and the syntactic row-sort footgun all shrink
or disappear once the cross-product is generated from one spec per effect
and per wrapper. (Generation makes the brand and class matrix consistent
and declared, but it cannot create class impls that Rust's type system
cannot express; those remain a separate capability question, W3.)

This is why the plan is generator-first: W2 (code generation) is the
central lever; wrapper-wide public work such as `expand` rides the
generated surface rather than being hand-built across six wrappers; and
only feasibility spikes run ahead of it.

## Open Questions, Decisions, Issues and Blockers

These are the decisions the plan cannot yet act on, choices with no
concrete next step beyond making the choice, so no work item owns a step
for them yet. Each names the work item it gates. When one is decided, its
reasoning is folded into that work item's steps and it is removed here.

1. Feature default (W4). Should the `effects` feature be on or off by
   default? Trade-off: out-of-the-box availability versus paying the
   subsystem's compile cost by default.
2. KVStore map convention (W11). Interpret `KVStore` over a `BTreeMap`, a
   `HashMap`, or a user-supplied map type? Affects the bound surface and
   whether ordering is guaranteed.
3. Fail identity (W12). Is `Fail` a distinct effect and brand, or an
   alias / newtype over `Except<String>`? Distinct gives clearer intent
   and errors; aliasing avoids another brand.
4. Coroutine semantics (W12). Single-shot versus multi-shot resume, the
   concrete `Status` type shape (`Done | Continue a k`), and which
   wrappers host it. Multi-shot fits the Rc / Arc wrappers; single-shot
   fits the default wrappers.
5. Output / Log accumulation convention (W11, W12). Does `Output`
   accumulate into a list, a user-supplied `Monoid`, or both (heftia
   offers `runOutputList` and `runOutputMonoid`)? `Log` is an `Output`
   specialization and follows this decision.
6. Macro behavior under feature-off (W4). When the `effects` feature is
   off, what do the effect proc-macros do? Options: stay always exported
   and fail through an unresolved generated path; become `cfg`-gated; or
   surface a "requires the `effects` feature" diagnostic via an
   `fp-library` re-export shim. `fp-macros` is a separate crate and cannot
   read `fp-library`'s active features directly.

## Work items

Each item links to the originating finding section. The "Sequencing" note
ties the item to the generator (W2); the milestone view is in
[Suggested implementation order](#suggested-implementation-order).

### W1. Row subsumption: `expand` / `weaken`

Finding: section 9, section 11 (P0).

Goal: add `expand` (widen a program's rows to a superset) and `weaken`
(the single-effect convenience) as a sound O(n) structural re-embed that
walks the program and lifts each `Node` layer's first-order row `R` and
scoped row `S` (through `NodeBrand`) into the larger row via
`CoproductEmbedder`. PureScript's `unsafeCoerce` `expand` has no sound
analog here, because `Coproduct`s of different arity differ in size and
layout.

Steps:

- Spike first to prove the row-embed is sound, including on the erased
  `Box<dyn Any>` `Run` substrate, where it must thread through the
  existing fold / peel machinery and is not yet proven. If it is blocked
  there, ship `expand` on the Explicit / Rc / Arc families and record the
  erased limitation.
- Build the row-embed as shared machinery, not per-wrapper, widening both
  the first-order and scoped rows symmetrically; asymmetric widening
  would leave one row unable to compose, so symmetry is the natural
  contract.
- Expose `expand` / `weaken` through the generated wrapper surface (W2)
  so no wrapper carries a hand-written copy, and make the
  `CoproductEmbedder` evidence inferable by reusing the existing
  `InferableBrand` / `InferableFnBrand` machinery, with a turbofish
  fallback only where inference is ambiguous; `expand` is only worth
  adding if it does not force a per-call index turbofish.
- Test: compose two independently-rowed programs into a shared row;
  round-trip `expand` then `handle`.

Sequencing: run the feasibility spike early; ship the public surface after
the W2 vertical slice.

### W2. Code generation for the wrapper x effect cross-product

Finding: section 4, section 11 (P0).

Goal: generate the six wrappers, the per-effect `Box` / `Send` / plain
brand siblings, their class impls, and the smart constructors from one
spec per effect and one per wrapper, replacing the hand-maintained
parallel code and declaring capability rules (such as multi-shot-only
`choose` / `ref_bracket`) instead of letting them drift.

Steps:

- Run a reduction spike before committing to generation: attempt one
  wrapper generic over a closure-storage / pointer brand and record where
  Rust blocks it. A successful reduction would supersede generation,
  though the documented limits (`FnOnce::call_once` consuming `self` out
  of a shared pointer, `Send + Sync` baked into trait objects, the
  per-`A` HRTB) suggest it cannot.
- Specify each effect and wrapper via a co-located `define_effect!` /
  `define_run_wrapper!` invocation in its module, matching the
  file-per-effect layout and the `#[fp_macros::document_module] mod inner`
  convention, rather than a central manifest or a `build.rs` step, so
  rust-analyzer support and the doc audit keep working and each spec is
  reviewable in isolation.
- Emit `#[document_module]`-compatible doc attributes by extending the
  `documented_helper_impls!` precedent
  (`fp-macros/src/documentation/item_generators.rs`), and carry
  user-facing doctests in the spec (templates only for mechanical items)
  so generated items still pass the `document_examples` audit
  (`scripts/document_examples.rs`); do not exempt generated items, which
  would quietly erode a documentation guarantee the project spends real
  effort to keep. Rule out `macro_rules!` for the public surface, since
  its generated methods are invisible to `#[document_module]` validation
  (prior decision D6).
- Scope the first iteration to the first-order effects, their brand
  siblings, and the six wrappers; defer scoped-effect generation until
  after W8 stabilizes the boundary / carrier / residual model, so the
  headline de-duplication win is not coupled to the riskiest subsystem.
- Migrate incrementally and behavior-preservingly: keep the existing
  effects test suite green against generated wrappers (which requires the
  generated public API to stay path-compatible) and golden-diff macro
  expansion (`cargo expand`) during the vertical slice, accepting brief
  coexistence rather than a big-bang cutover, because the existing tests
  are the cheapest high-fidelity behavioral oracle.
- Vertical slice: migrate Reader and one wrapper end-to-end, diffing the
  generated output against the current code, then migrate the rest,
  encoding the capability rules in the spec so the `choose` /
  `ref_bracket` asymmetry is declared, not drifted.

### W3. Brand and class capability audit, then decide the gaps

Finding: section 7, section 11 (P0).

Goal: replace the (incorrect) assumption that the Erased wrappers can
delegate to existing Free brands with a verified capability matrix and a
per-gap decision. Verified state: `RunExplicitBrand` has Functor /
Pointed / Semimonad plus the Ref trio; `RcRunExplicitBrand` has Pointed
plus the Ref trio (no owned Functor or Semimonad); `ArcRunExplicitBrand`
has SendPointed and SendRefPointed; the Erased trio and the non-explicit
Free family are unbranded (no `FreeBrand` / `RcFreeBrand` /
`ArcFreeBrand`).

Steps:

- Produce the verified per-brand, per-class matrix and fold it into the
  W6 doc.
- Fix the stale `ArcRunExplicitBrand` docs, which claim "limited to
  `SendPointed`" but `SendRefPointed` is implemented.
- Decide each gap, close or document: branding the Erased trio first
  requires branding `Free` / `RcFree` / `ArcFree` and proving the erased
  `Box<dyn Any>` `Run` admits a clean `Kind` projection, so do it only if
  a concrete generic use case needs it; document gaps forced by Rust
  limits (for example `ArcRunExplicitBrand` `SendFunctor` /
  `SendSemimonad`) rather than chasing them.
- Feed the decided matrix into W2's wrapper and effect specs.

Sequencing: before or alongside W2 so the generator emits a consistent,
intentional matrix.

### W4. Feature-gate the subsystem

Finding: section 7, section 11 (P1).

Goal: add a single `effects` cargo feature gating the effects modules and
their public re-exports, with granular sub-features (`effects-arc`,
`effects-explicit`) considered only if compile-cost data later justifies
the `cfg` and CI-matrix complexity. The gating must account for two macro
path families: row macros (`effects!`, `raw_effects!`, `scoped_effects!`,
`define_scoped_row!`, `define_effect_row_aliases!`) emit
`::fp_library::brands::` paths (`CoproductBrand` / `CNilBrand` in
`brands/effects.rs`; the `CoyonedaBrand` family in the general
`brands.rs`), while handler macros (`handlers!`, `scoped_handlers!`) emit
`::fp_library::types::effects::handlers::` paths, so the two fail at
different gated locations when the feature is off.

Steps:

- Gate the effects modules and their public re-exports, not only the
  module declarations: `pub mod effects;` and `pub use effects::*;` in
  `brands.rs`, and `pub mod effects;` and the flat `effects::{...}`
  re-export in `types.rs`.
- Confirm the rest of the crate builds with the feature off (no
  non-effects code depends on effects).
- Add CI jobs for feature-off and feature-on, including a feature-off
  compile test that invokes a macro.

Gated on the feature-default and feature-off-macro-behavior decisions
(Open Questions); W4 ships once both are chosen, and their reasoning
folds into the steps above at that point.

### W5. Row-macro Rc / Arc symmetry

Finding: section 9.

Goal: close the `rc_effects!` / `arc_effects!` gap as part of the W2 macro
redesign rather than adding throwaway macros now. By default, defer and
decide the row-macro surface inside W2; add the two macros early only if
their names are committed to survive W2, or a concrete near-term Rc / Arc
need predates it. When the work does happen, thin sibling macros sharing
the worker beat changing `effects!`'s syntax or leaving the asymmetry a
documentation burden.

Steps (only if done early, ahead of W2):

- Add `rc_effects!` / `arc_effects!` entry points and workers reusing
  `build_coproduct_row` with `RcCoyoneda` / `ArcCoyoneda`.
- Add tests mirroring the `effects!` tests; add docs.

Sequencing: deferred into W2 by default.

### W6. Documentation consolidation

Finding: sections 6, 8, 9.

Goal: one authoritative effects-guide section (module-level in
`types/effects.rs`) covering the substrate axis (Erased vs Explicit), the
shot / sharing axis (Box / Rc / Arc), the prefix legend (Box / Send /
plain), the brand and class capability matrix (from W3), and a
consolidated known-limitations list (mono-in-`A`, `Fn` vs `FnOnce`,
`ArcRunExplicit` coverage, Erased downcasts). Per-module docs link to it
rather than restating it.

Steps:

- Write the guide and the capability matrix.
- Link existing module docs to it.
- Ensure ASCII-only and lychee link checks pass.

### W7. Syntactic row-sort footgun

Finding: section 9.

Goal: sharpen the docs and the downstream diagnostic now; the durable fix
is W2 (generating the row and the handlers from one spec removes the
mismatch entirely). Macro-time detection is impossible because
proc-macros cannot resolve aliases or imports, so a spelling mismatch is
not rejected at expansion and instead surfaces as a trait/type error at
the `handle` call site.

Steps:

- Strengthen the `row_sort` / `handlers` docs with a worked mismatch
  example.
- Improve the "not implemented for `HandlersNil`" guidance.
- Note generation (W2) as the eventual elimination.

### W8. Scoped-dispatch design note and consolidation evaluation

Finding: section 5.

Goal: document the boundary / carrier / residual scoped-dispatch split
with a Writer-`listen` worked example and the invariant each trait
protects, so the rationale is not spread across `pub(crate)` trait docs.

Steps:

- Write the design note; enumerate the invariant each trait protects,
  chiefly keeping `NextProgram` independent from the selected
  `ActionProgram`.
- Evaluate consolidation only after W2, and gate it on a
  semantics-preservation proof that Writer `listen` / `censor` and `Span`
  still behave; the split encodes a real need, so unifying it prematurely
  risks regressions against working code.

### W9. Generic scoped rows

Finding: section 9.

Goal: add a separate generic scoped-row item macro (matches prior decision
D3), leaving `define_scoped_row!` concrete-only.

Steps:

- Design the macro syntax, including lifetime / type / where-clause
  handling and recursive `Self` replacement under generics, before
  implementing; these are the hard parts that distinguish it from the
  concrete `define_scoped_row!`.
- Implement; add tests covering lifetimes, type parameters, and where
  clauses.

Sequencing: when a concrete generic scoped-row need arrives.

### W10. Erased `Run` downcast soundness

Finding: section 9.

Goal: keep the erased default (its `Box<dyn Any>` downcasts are a
deliberate ergonomics choice) and harden it with tests and docs, rather
than removing it (which would mean merging `Run` into `RunExplicit` and
losing the erased default).

Steps:

- Add boundary downcast soundness tests showing a type mismatch cannot
  arise through the safe API.
- Document the invariant near `RunRepresentation`.
- Cross-link from the W6 known-limitations list.

### W11. Port low-risk first-order effects and NonDet aggregation

Finding: section 10, section 11 (P1).

Goal: add thin dedicated effects Fresh, Input, Output, and KVStore, each
with a named runner that reinterprets onto State / Writer (for
discoverability and heftia parity). For nondeterminism, add only the
genuinely-missing pieces, a combined `Choose` + `Empty` runner and a
first-success helper; the per-effect `run_empty` (into `Option`) and
`run_choose` (into `Vec`) already exist in `named_helpers/nondet.rs`.

Steps:

- Ship a named runner and helper constructors per effect family, matching
  the existing State / Except / Writer helper style.
- Add the combined `Choose` + `Empty` runner and the first-success
  helper; do not duplicate the existing `run_choose` / `run_empty`.

Gated on the KVStore-map-convention and Output-accumulation-convention
decisions (Open Questions).

Sequencing: after W2 so each effect is a single spec; if done earlier,
implement on the multi-shot wrappers first.

### W12. Port moderate effects: Coroutine, Log, Fail

Finding: section 10.

Goal: port Coroutine, Log, and Fail through the generator. Coroutine needs
a `Status` type and shot semantics; Log is an Output specialization that
follows the Output accumulation convention; Fail is `Except<String>`-like.

Steps:

- Implement each effect and its runner; add tests.

Gated on the Coroutine-semantics, Fail-identity, and
Output-accumulation-convention decisions (Open Questions); Log follows the
Output accumulation convention.

Sequencing: after W11.

### W13. Runtime policy, then async interpreter, then deferred ports

Finding: sections 6 and 10, section 11 (P3).

Goal: write a runtime policy first, then build the async interpreter the
policy allows, then schedule the runtime-sensitive ports (Shift / CC,
Provider, Unlift, the Concurrent family) against it.

Steps:

- Author the runtime policy doc covering the executor or blocking model,
  cancellation, IO embedding, process lifecycle ownership, target-monad
  lifting, continuation exposure, and `Send + Sync`; these ports encode
  runtime commitments that must be decided once and centrally rather than
  per-effect.
- Decide the async approach (`MonadRec`-over-`Future` is the leading
  candidate; a dedicated async substrate is the fallback) and implement
  it.
- Schedule Shift / CC, Provider, Unlift, and the Concurrent family
  against the policy. Shift / CC additionally needs
  answer-type-polymorphic continuation capture, which is beyond the
  current mono-in-`A` model.

Sequencing: last; everything here is policy-gated.

## Suggested implementation order

A proposal that follows the generator-first thesis in the
[root-cause framing](#root-cause-framing): feasibility spikes first, then
the generator, then wrapper-wide public work on the generated surface. If
the W2 reduction spike shows generation is far off, reconsider landing
`expand` by hand sooner, since it is a P0 unblock and the rework is
bounded.

1. Documentation and coherence with no policy commitment: W6, W7, the W8
   design note, W10 invariant tests and docs, and the W3 capability audit.
   Also record the performance and compile-time baseline now (reusing the
   `benchmarking` plan area and `fp-library/benches/benchmarks.rs`), so
   the plan's O(n) `expand`, runtime-neutral generation, and reduced
   compile-time claims become checkable.
2. De-risk the big decisions: the W2 reduction spike and the W1 row-embed
   feasibility spike, then the generator and spec design.
3. W2 vertical slice: one effect and one wrapper generated end-to-end,
   diffed against the current code.
4. W1 (`expand` / `weaken`) implemented through the generated / shared
   surface, not six hand copies.
5. W4 feature-gating, after the generated exports and macro paths
   stabilize.
6. Effect ports on the generated base: W11, then W12.
7. W5 row macros and W9 generic scoped rows: folded into the macro
   redesign, or done earlier only if a concrete need predates W2.
8. Policy-gated runtime work: W13.

## Traceability

This plan is derived from [`findings.md`](findings.md). Future
implementation commits should cite the relevant work item (W-number) and,
if the change resolves an open decision, that decision by title.
