# Effects System Remediation Plan (review-1)

An implementation plan derived from [`findings.md`](findings.md). Each
work item states its adopted goal and the concrete steps to reach it.
Reasoning is retained only where a rejected alternative is non-obvious, or
where the item is still gated on an open question; the approach-by-approach
trade-off analysis that produced these decisions has been folded into the
steps rather than preserved as history.

This is still a working plan: several steps are gated on the open
questions below, and the milestone sequencing is a proposal.

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
  implementation plan as concrete steps. This keeps the document from
  accumulating stale decision history while still making the intended
  work clear.
- Open, not-yet-decided items live only in the numbered
  [Open Questions, Decisions, Issues and Blockers](#open-questions-decisions-issues-and-blockers)
  section. When one is decided, fold the decision into the relevant work
  item's concrete steps and remove it from that section.
- The plan is action-oriented. When analysis is converted into steps,
  the analysis is trimmed rather than preserved as history. Historical
  trace belongs in commit messages and git log.

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
only feasibility spikes run ahead of it. The sequencing leaning and its
one contingency (what to do if generation proves far off) are open
question 13.

## Open Questions, Decisions, Issues and Blockers

This numbered list tracks everything not yet decided. When an item is
decided, fold the decision into the relevant work item's concrete steps
and remove it here.

1. Cross-product reduction vs generation (W2). Can a single wrapper
   parameterized by a closure-storage / pointer brand collapse
   Box / Rc / Arc, or do the documented limits
   (`FnOnce::call_once` consuming `self` out of a shared pointer;
   `Send + Sync` baked into trait objects; the per-`A` HRTB limit) force
   the split? The spike outcome decides reduce (D) vs generate (A).
2. `expand` on the Erased `Box<dyn Any>` substrate (W1). Does the boundary
   representation permit structural re-embedding without re-erasing
   types, and at what cost? Confirm before committing the Erased
   `expand`.
3. `expand` scope (W1). Should `expand` widen both the first-order row and
   the scoped row? Recommended: yes, symmetric.
4. Generator and doc integration (W2). Must all generated items pass
   `#[document_module]` validation, and can the generator emit the
   required doc attributes by extending the `documented_helper_impls!`
   precedent?
5. Feature default (W4). Is the `effects` feature on or off by default?
   Trade-off: out-of-the-box availability vs default compile cost.
6. Erased-family branding (W3). The non-explicit Free family is unbranded
   (no `FreeBrand` / `RcFreeBrand` / `ArcFreeBrand`), so branding the
   Erased Run trio would require branding `Free` / `RcFree` / `ArcFree`
   first. Is that worth doing (does a concrete generic use case need it),
   and can the erased `Box<dyn Any>` `Run` carry a clean `Kind` projection
   at all, or is its brand-less status a permanent, documented exception?
7. KVStore map convention (W11). `BTreeMap`, `HashMap`, or a
   user-supplied map type?
8. Fail identity (W12). A distinct `Fail` effect and brand, or an alias /
   newtype over `Except<String>`?
9. Coroutine semantics (W12). Single-shot vs multi-shot resume; the
   concrete `Status` type shape; which wrappers host it.
10. Scoped-dispatch consolidation (W8). Can the boundary / carrier /
    residual traits be unified without breaking Writer `listen` / `censor`
    and `Span`? Requires a semantics-preservation proof; evaluate after
    W2.
11. Generic scoped-row syntax (W9). The macro syntax, lifetime / type /
    where-clause handling, and recursive `Self` replacement under
    generics.
12. Runtime policy contents (W13). The executor or blocking model,
    cancellation semantics, IO embedding, process lifecycle ownership,
    target-monad lifting, continuation-exposure policy, and `Send + Sync`
    requirements. Blocks Shift / CC, Provider, Unlift, and the Concurrent
    family.
13. Sequencing (W1, W11 vs W2). Leaning: generator-first for wrapper-wide
    public work. Do the W1 feasibility spike early, but ship `expand` /
    `weaken` and the effect ports through the generated surface rather
    than hand-writing six copies. This leaning is contingent on the W2
    spike (open question 1): if generation proves far off, reconsider
    landing `expand` by hand sooner, since it is a P0 unblock and the
    rework is bounded. The suggested implementation order reflects this
    leaning.
14. Macro behavior under feature-off (W4). When the `effects` feature is
    off, what do the effect proc-macros do? Options: stay always exported
    and fail through an unresolved generated path, become `cfg`-gated, or
    surface a "requires the `effects` feature" diagnostic via an
    `fp-library` re-export shim. `fp-macros` cannot read `fp-library`'s
    active features directly.
15. Output / Log accumulation convention (W11, W12). Does `Output`
    accumulate into a list, a user-supplied `Monoid`, or both (heftia
    offers `runOutputList` and `runOutputMonoid`)? `Log` is an `Output`
    specialization and should follow this decision rather than introduce
    its own.

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

- Spike first: prove the row-embed is sound, including on the erased
  `Box<dyn Any>` `Run` substrate (open question 2). If it is blocked
  there, ship `expand` on the Explicit / Rc / Arc families and record the
  erased limitation.
- Build the row-embed as shared machinery, not per-wrapper, widening both
  the first-order and scoped rows symmetrically (open question 3).
- Emit `expand` / `weaken` through the generated wrapper surface (W2) so
  no wrapper carries a hand-written copy.
- Test: compose two independently-rowed programs into a shared row;
  round-trip `expand` then `handle`.

Sequencing: run the feasibility spike early; ship the public surface after
the W2 vertical slice (open question 13).

### W2. Code generation for the wrapper x effect cross-product

Finding: section 4, section 11 (P0).

Goal: generate the six wrappers, the per-effect `Box` / `Send` / plain
brand siblings, their class impls, and the smart constructors from one
spec per effect and one per wrapper, replacing the hand-maintained
parallel code and declaring capability rules (such as multi-shot-only
`choose` / `ref_bracket`) instead of letting them drift. Not
`macro_rules!`: its generated public methods are invisible to
`#[document_module]` validation (prior decision D6). Not parity-tests-only:
that leaves the duplication in place.

Steps:

- Reduction spike (open question 1): attempt one wrapper generic over a
  closure-storage / pointer brand and record exactly where Rust blocks it.
  A successful reduction would supersede generation; the documented limits
  suggest it will not.
- Design the effect spec (name, operations, first-order vs scoped, which
  flavours and wrappers, capability rules) and the wrapper spec.
- Implement `define_effect!` and `define_scoped_effect!` to emit brands,
  effect types, class impls, smart constructors, and
  `#[document_module]`-compatible doc attributes, extending the
  `documented_helper_impls!` precedent
  (`fp-macros/src/documentation/item_generators.rs`); see open question 4.
- Implement `define_run_wrapper!` to emit the newtype, the monadic
  surface, and the interpreter entry points.
- Vertical slice: migrate Reader and one wrapper end-to-end, diffing the
  generated output against the current code, then migrate the rest.

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
- Fix the stale `ArcRunExplicitBrand` docs (they claim "limited to
  `SendPointed`" but `SendRefPointed` is implemented).
- Decide each gap: close or document as intentional. Branding the Erased
  trio first requires branding `Free` / `RcFree` / `ArcFree` and proving
  the erased `Box<dyn Any>` `Run` admits a clean `Kind` projection (open
  question 6); do it only if a concrete generic use case needs it.
  Document gaps forced by Rust limits (for example `ArcRunExplicitBrand`
  `SendFunctor` / `SendSemimonad`) rather than chasing them.
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
- Decide and implement macro behavior under feature-off (open question
  14).
- Confirm the rest of the crate builds with the feature off (no
  non-effects code depends on effects).
- Add CI jobs for feature-off and feature-on, including a feature-off
  compile test that invokes a macro.
- Decide and document the default (open question 5).

Sequencing: after the generated exports and macro paths stabilize (after
W2).

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
Defer any consolidation until after W2 and gate it on a
semantics-preservation proof (open question 10), since the split encodes a
real need (keeping `NextProgram` independent from the selected
`ActionProgram`).

Steps:

- Write the design note; enumerate the invariants each trait protects.
- Record consolidation as a post-W2 evaluation requiring the proof.

### W9. Generic scoped rows

Finding: section 9.

Goal: add a separate generic scoped-row item macro with its own syntax,
lifetime / type / where-clause handling, and recursive `Self` replacement
(matches prior decision D3), leaving `define_scoped_row!` concrete-only.

Steps:

- Design the syntax and the recursive-`Self`-under-generics rules (open
  question 11).
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

- Decide the KVStore map convention (open question 7) and the Output
  accumulation convention (open question 15; this also gates Log in W12).
- Ship a named runner and helper constructors per effect family, matching
  the existing State / Except / Writer helper style.
- Add the combined `Choose` + `Empty` runner and the first-success
  helper; do not duplicate `run_choose` / `run_empty`.

Sequencing: after W2 so each effect is a single spec; if done earlier,
implement on the multi-shot wrappers first.

### W12. Port moderate effects: Coroutine, Log, Fail

Finding: section 10.

Goal: port Coroutine, Log, and Fail through the generator, each after its
gating decision. Coroutine needs a `Status` type (`Done | Continue a k`)
and shot semantics; Log follows the Output accumulation convention; Fail
needs the decision on whether to be distinct from `Except`.

Steps:

- Resolve open questions 8, 9, and 15 (open question 15 gates Log via the
  Output accumulation convention).
- Implement each effect and its runner; add tests.

Sequencing: after W11.

### W13. Runtime policy, then async interpreter, then deferred ports

Finding: sections 6 and 10, section 11 (P3).

Goal: write a runtime policy first, then build the async interpreter the
policy allows, then schedule the runtime-sensitive ports (Shift / CC,
Provider, Unlift, the Concurrent family) against it. These ports encode
runtime commitments (executor or blocking model, cancellation, IO
embedding, process lifecycle, target-monad lifting, continuation
exposure, `Send + Sync`) that must be decided once and centrally rather
than per-effect.

Steps:

- Author the runtime policy doc (open question 12).
- Decide the async approach (`MonadRec`-over-`Future` is the leading
  candidate; a dedicated async substrate is the fallback) and implement
  it.
- Schedule Shift / CC, Provider, Unlift, and the Concurrent family, each
  linking to the policy. Shift / CC additionally needs
  answer-type-polymorphic continuation capture, which is beyond the
  current mono-in-`A` model.

Sequencing: last; everything here is policy-gated.

## Suggested implementation order

A proposal, subject to the open questions (especially 1 and 13). It
follows the generator-first thesis in the
[root-cause framing](#root-cause-framing): feasibility spikes first, then
the generator, then wrapper-wide public work on the generated surface.

1. Documentation and coherence with no policy commitment: W6, W7, the W8
   design note, W10 invariant tests and docs, and the W3 capability audit.
2. De-risk the big decisions: the W2 reduction spike (open question 1) and
   the W1 row-embed feasibility spike (open question 2), then the
   generator and spec design.
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
implementation commits should cite the relevant work item (W-number) and
any decided open-question number in their commit bodies or plan updates.
