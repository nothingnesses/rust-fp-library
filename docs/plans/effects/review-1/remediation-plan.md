# Effects System Remediation Plan (draft, review-1)

A draft plan that converts each item in
[`findings.md`](findings.md) into concrete work. For every item it lists
the candidate approaches, their trade-offs, a recommended approach, and
the reasoning behind the recommendation, then states the concrete steps
that the recommendation implies.

This is a draft for discussion. It is not yet sequenced into committed
milestones, and several steps depend on the open questions in the final
section.

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
hand rather than generated. Row subsumption duplication, brand and impl
consistency, helper drift, and the syntactic row-sort footgun all shrink
or disappear once the cross-product is generated from one spec per effect
and per wrapper. (Generation makes the brand and class matrix consistent
and declared, but it does not by itself create class impls that Rust's
type system cannot express; those remain a separate capability question,
see W3.) The central sequencing question (see open question 13) is
therefore whether to build the generator first and make everything else
cheap, or to land high-value point fixes now and fold them into the
generator later.

## Work items

Each item links to the originating finding section.

### W1. Row subsumption: `expand` / `weaken`

Finding: section 9 (missing combinator) and section 11 (P0).

Problem: there is no Run-level `expand` / `weaken`, so programs written
against different-but-compatible rows cannot be composed without manual
plumbing. `CoproductEmbedder` / `CoproductSubsetter` are surfaced but
unused at the wrapper level.

Key constraint: PureScript's `expand` is `unsafeCoerce` because
`Free (VariantF r1)` and `Free (VariantF r2)` share a representation. In
Rust, `Coproduct`s of different arity differ in size and layout, so
`expand` cannot be a coerce; it must be a real O(n) structural re-embed
that walks the program and lifts each `Node` layer's first-order row
(and scoped row) into the larger row via `CoproductEmbedder::embed`.

Approaches:

- A. Structural re-embed via a fold over the Free-family tree, mapping
  every layer's row through `CoproductEmbedder`. Sound for all
  substrates; O(n) at the composition boundary; for the Erased default
  `Run` it must thread through the existing fold / peel machinery.
- B. Representation-sharing via `unsafe` transmute or coerce, mimicking
  PureScript. Unsound: differing `Coproduct` arities have different
  layout and size. Reject.
- C. Require programs to pre-declare the maximal row and use `Member`
  injection throughout. Avoids the feature but defeats modular,
  independently-rowed composition.

Recommendation: A. It is the only sound option in Rust, the cost is
acceptable (one pass where rows are widened), and the embedder evidence
already exists. Expose `expand` / `weaken` as inherent methods on the
wrappers (matching the existing `send` / `peel` / `lift` surface), and
once W2 lands, emit them from the generator instead of hand-writing six
copies.

Concrete steps:

- Define a row-embed operation that lifts a `VariantF` over a sub-row
  into a super-row via `CoproductEmbedder`, for both the first-order row
  `R` and the scoped row `S` (through `NodeBrand`).
- Implement `expand` (widen to a superset row) on each wrapper, and
  `weaken` as the single-effect convenience. Make `expand` widen both
  rows symmetrically (see open question 3).
- Add tests: compose two independently-rowed programs into a shared row;
  round-trip `expand` then `handle`.
- Confirm feasibility on the Erased `Box<dyn Any>` substrate first (open
  question 2); if blocked there, ship `expand` on the Explicit / Rc /
  Arc families and record the Erased limitation.

### W2. Code generation for the wrapper x effect cross-product

Finding: section 4 and section 11 (P0).

Problem: the six wrappers, the per-effect `Box` / `Send` / plain brand
siblings, and the smart constructors are hand-maintained parallel code
(the dominant share of the subsystem by item count). Drift is already
visible (`choose` / `ref_bracket` exist only on multi-shot wrappers,
encoded only in prose).

Approaches:

- A. Proc-macros that generate per-effect items (brand siblings, effect
  enums, `Functor` / `SendFunctor` / `Clone` impls, smart constructors)
  and per-wrapper items (newtype, monadic surface, `handle` / `run` /
  `peel` entry points) from one spec each, emitting
  `#[document_module]`-compatible doc attributes.
- B. Declarative `macro_rules!` for the repetitive bodies. Simpler to
  write, but public generated methods are invisible to
  `#[document_module]` validation (decision D6 in the prior remediation
  plan explicitly warns against `macro_rules!` for public helper
  methods), and `macro_rules!` handles the per-flavour bound differences
  poorly.
- C. Keep hand-written code, add cross-wrapper parity tests or a custom
  lint to detect drift. Cheapest, but does not reduce the code or the
  maintenance burden.
- D. Reduce the cross-product itself: parameterize a single wrapper by a
  closure-storage / pointer brand so `Box` / `Rc` / `Arc` collapse. The
  deepest fix, but the brand docs show each split is forced by concrete
  Rust limitations (`FnOnce::call_once` consumes `self` out of a shared
  pointer; `Send + Sync` must be baked into the trait object; the per-`A`
  HRTB limit on `Send`-deriving projections). Likely blocked, but a
  spike would confirm.

Recommendation: run the D spike first, then do A. If the cross-product
can be reduced (D), that beats generating it. The documented constraints
strongly suggest it largely cannot, so the realistic win is A: generate
the cross-product, extending the `documented_helper_impls!` precedent
(`fp-macros/src/documentation/item_generators.rs`) so generated items
still pass doc validation. Reject B for the doc-validation gap; treat C
as a stopgap only if A is deferred.

Concrete steps:

- Spike D: attempt one wrapper generic over a closure-storage / pointer
  brand and record exactly where Rust blocks it. This resolves open
  question 1 (reduce vs generate).
- Design the effect spec (name, operations, first-order vs scoped, which
  flavours and wrappers, capability rules such as multi-shot-only) and
  the wrapper spec.
- Implement `define_effect!` and `define_scoped_effect!` to emit brands,
  effect types, class impls, smart constructors, and doc attributes.
- Implement `define_run_wrapper!` to emit the newtype, the monadic
  surface, and interpreter entry points.
- Migrate one effect (Reader) and one wrapper end-to-end behind tests,
  diffing generated output against the current code, then migrate the
  rest.
- Encode the `choose` / `ref_bracket` capability rules in the spec so
  the asymmetry is declared, not drifted.

### W3. Brand and class capability audit, then decide the gaps

Finding: section 7 (corrected) and section 11 (P0).

Problem: brand-level class participation is partial and uneven. An
earlier framing of it as a simple "branded Explicit vs unbranded Erased"
split, with the assumption that `RcRun` / `ArcRun` could delegate to
existing `RcFree` / `ArcFree` brands, was inaccurate. The verified state
is:

- Only the Explicit trio has dedicated brands, and their coverage is not
  uniform. `RunExplicitBrand` implements `Functor`, `Pointed`,
  `Semimonad`, `RefFunctor`, `RefPointed`, `RefSemimonad`;
  `RcRunExplicitBrand` implements `Pointed`, `RefFunctor`, `RefPointed`,
  `RefSemimonad` (no owned `Functor` or `Semimonad`);
  `ArcRunExplicitBrand` implements only `SendPointed` and `SendRefPointed`.
- The Erased trio (`Run` / `RcRun` / `ArcRun`) has no brand, and the
  non-explicit Free family is itself unbranded: only `FreeExplicitBrand` /
  `RcFreeExplicitBrand` / `ArcFreeExplicitBrand` exist, with no
  `FreeBrand` / `RcFreeBrand` / `ArcFreeBrand`. There is therefore no
  underlying brand for an Erased Run brand to delegate to.
- The `ArcRunExplicitBrand` source docs are stale: they claim coverage is
  "limited to `SendPointed`" and that the `SendRef` hierarchy is
  unreachable, but `SendRefPointed` is implemented.

Approaches:

- A. Audit first, then decide per gap. Produce the verified per-brand
  class matrix, fix the stale `ArcRunExplicitBrand` docs, then decide for
  each gap whether to close it or document it as intentional.
- B. Aggressively close gaps: brand the non-explicit Free family
  (`FreeBrand` / `RcFreeBrand` / `ArcFreeBrand`), brand the Erased Run
  trio by delegation, and fill the missing Explicit-trio class impls.
  Maximises generic usability but is a large undertaking and may hit the
  same Rust limits that produced the gaps in the first place.
- C. Document the matrix as intentional (Explicit is the brand-level
  generic surface; Erased is the inherent-method ergonomic surface) and
  close nothing.

Recommendation: A. Because the easy-delegation assumption was wrong (the
non-explicit Free brands do not exist), this must begin as an audit and
an explicit decision, not a mechanical delegation. Fix the stale docs
immediately (cheap and unambiguous). Then decide each gap on its merits:
branding the non-explicit Free family and the Erased Run trio (a step
toward B) is worth doing only if a concrete generic use case needs it,
and only after confirming the erased `Box<dyn Any>` representation admits
a clean `Kind` projection (open question 6). Some Explicit-trio gaps are
forced by Rust limits (for example `ArcRunExplicitBrand` cannot carry
`SendFunctor` / `SendSemimonad` without a per-`A` HRTB or `Send`-closure
bound the trait method signatures cannot express); document those rather
than chasing them. Sequence the audit before or alongside W2 so the
generator, if built, emits a consistent and intentional matrix.

Concrete steps:

- Produce the verified per-brand, per-class matrix (fold into the W6
  doc).
- Fix the stale `ArcRunExplicitBrand` documentation.
- For each gap, record a decision: close (noting the concrete branding
  work it requires) or document as intentional with the reason.
- Feed the decided matrix into W2's wrapper and effect specs.

### W4. Feature-gate the subsystem

Finding: section 7 and section 11 (P1).

Problem: `types.rs` exposes the subsystem as a plain `pub mod effects;`,
so the entire heavy subsystem compiles for every downstream user.

Gating must also define macro and export behavior, not only the type and
brand modules. The effect proc-macros (`effects!`, `handlers!`,
`scoped_effects!`, etc.) live in `fp-macros` and expand to absolute paths
into `fp-library`'s effects modules (for example
`::fp_library::brands::CoproductBrand` and
`::fp_library::types::effects::handlers::HandlersCons`). With the feature
off, that generated code would reference items that do not exist, so the
gating scheme must decide what these macros do in that configuration.

Approaches:

- A. A single `effects` cargo feature.
- B. Granular features (`effects`, `effects-arc`, `effects-explicit`).
- C. Status quo (no gating).

Recommendation: A first, then measure compile-time impact, then consider
B only if the Arc / Explicit families dominate the cost. A single feature
is the simplest correct step; granular features multiply `cfg`
complexity and the CI matrix, so the granularity decision should follow
data. Decide the default (open question 5) and the macro behavior (open
question 14).

Concrete steps:

- Gate `types::effects` and `brands::effects` behind `feature = "effects"`.
- Decide macro behavior under feature-off: whether the effect macros stay
  always exported (failing with a clear "requires the `effects` feature"
  diagnostic when used), become `cfg`-gated themselves, or emit
  feature-specific errors. Note that `fp-macros` is a separate crate and
  cannot read `fp-library`'s active features directly, so the diagnostic
  likely has to come from the generated path failing to resolve, or from
  a re-export shim in `fp-library`.
- Confirm the rest of the crate builds with the feature off (no
  non-effects code depends on effects).
- Add CI jobs for feature-off and feature-on, including a feature-off
  compile test that exercises a macro invocation to lock in the chosen
  behavior.
- Decide and document the default.

### W5. Row-macro Rc / Arc symmetry

Finding: section 9.

Problem: `RowHeadWrap` has `RcCoyoneda` / `ArcCoyoneda` variants, but only
`define_effect_row_aliases!` reaches them; there is no `rc_effects!` /
`arc_effects!` paralleling `effects!`.

Approaches:

- A. Add `rc_effects!` and `arc_effects!` as thin siblings sharing the
  worker (mirrors `scoped_effects!`). Cheap; the worker already supports
  the Rc / Arc wraps.
- B. Parameterize `effects!` with a sharing-mode argument (for example
  `effects!(rc; ...)`). One macro, but changes the established `effects!`
  syntax.
- C. Document `define_effect_row_aliases!` as the canonical Rc / Arc path
  and add no macros.

Recommendation: A. Lowest friction, matches the existing `scoped_effects!`
precedent and a one-macro-per-row-flavour mental model. B churns a stable
macro's syntax; C leaves the asymmetry as a documentation burden. If W2
later subsumes row construction, revisit.

Concrete steps:

- Add `rc_effects!` / `arc_effects!` entry points and workers reusing
  `build_coproduct_row` with `RcCoyoneda` / `ArcCoyoneda`.
- Add tests mirroring the `effects!` tests; add docs.

### W6. Documentation consolidation

Finding: sections 6, 8, 9.

Problem: substrate axes, prefix semantics, the capability matrix, and the
known limitations (mono-in-`A`, `Fn` vs `FnOnce`, `ArcRunExplicit`
coverage, Erased downcasts) are scattered or implicit.

Approaches:

- A. One authoritative effects-guide section (module-level in
  `types/effects.rs`) covering: the substrate axis (Erased vs Explicit),
  the shot / sharing axis (Box / Rc / Arc), the prefix legend
  (Box / Send / plain), a capability matrix (which wrapper supports which
  effects and classes), and a consolidated known-limitations list.
- B. Scatter improvements across the relevant module docs only.

Recommendation: A. One place reduces drift and onboarding cost; per-module
prose links to it. Keep it in-tree so doctests and links are checked by
`just doc`.

Concrete steps:

- Write the guide and the capability matrix.
- Link existing module docs to it.
- Ensure ASCII-only and lychee link checks pass.

### W7. Syntactic row-sort footgun

Finding: section 9.

Problem: the row-sort key is structural over parsed syntax, not semantic
Rust identity, so mismatched brand spellings between `effects!` and
`handlers!` silently misalign.

Approaches:

- A. Docs emphasis plus improved missing-handler diagnostic wording (the
  trait error is the real signal today).
- B. A debug-time or test helper that asserts a row and a handler list
  align.
- C. Rely on W2: generating row and handlers from one spec removes the
  footgun entirely.

Recommendation: A now, C as the durable fix. Proc-macros fundamentally
cannot resolve aliases or imports, so semantic identity is impossible at
macro time; the lasting fix is generation. Until then, sharpen docs and
diagnostics. B is optional and only helps tests.

Concrete steps:

- Strengthen the `row_sort` / `handlers` docs with a worked mismatch
  example.
- Improve the "not implemented for `HandlersNil`" guidance.
- Note generation (W2) as the eventual elimination.

### W8. Scoped-dispatch design note and consolidation evaluation

Finding: section 5.

Problem: the boundary / carrier / residual scoped-dispatch trait family is
powerful but hard to hold in mind, and several traits are `pub(crate)`
with `skip_call_check` doctests.

Approaches:

- A. Write a design note that explains the split with a Writer-`listen`
  worked example and enumerates the invariant each trait protects.
- B. A plus attempt to unify the boundary traits into fewer, gated by a
  proof that Writer `listen` / `censor` and `Span` semantics are
  preserved.
- C. Redesign the scoped interpreter around a single abstraction.

Recommendation: A now, B after W2. The split encodes a real semantic need
(keeping `NextProgram` independent from the selected `ActionProgram`), so
unifying it now is high-risk against working code. Document first;
revisit consolidation once generation may reshape it. C is premature.

Concrete steps:

- Write the design note; enumerate the invariants.
- Mark consolidation as a post-W2 evaluation that requires a
  semantics-preservation proof (open question 10).

### W9. Generic scoped rows

Finding: section 9.

Problem: `define_scoped_row!` rejects generic scoped rows, which limits
reusable environment / error / log-parameterized scoped stacks.

Approaches:

- A. A separate generic scoped-row item macro with its own syntax,
  lifetime / type / where-clause handling, and recursive `Self`
  replacement (matches the prior decision D3).
- B. Extend `define_scoped_row!` to accept generics.
- C. Leave it deferred.

Recommendation: A. It keeps the concrete macro simple while giving
parameterized rows a designed home. Design the syntax and tests before
implementing (open question 11).

Concrete steps:

- Design the macro syntax and the recursive-`Self`-under-generics rules.
- Implement; add tests covering lifetimes, type params, and where
  clauses.

### W10. Erased `Run` downcast soundness

Finding: section 9.

Problem: the default Erased `Run` stores values as `Box<dyn Any>` and
downcasts on the way out (for example at `run/representation.rs:403`).
The `expect` paths are internal invariants, not user-facing errors.

Approaches:

- A. Keep the erased default; add focused soundness tests around the
  boundary bind; document the invariant and the Erased / Explicit
  separation.
- B. Remove the downcast by making the default `Run` type-directed like
  `RunExplicit` (effectively merging the two). Loses the ergonomic erased
  default.

Recommendation: A. The erased default is a deliberate ergonomics choice;
the right mitigation is test coverage of the invariant plus clear docs,
not removing the feature.

Concrete steps:

- Add boundary downcast soundness tests showing a type mismatch cannot
  arise through the safe API.
- Document the invariant near `RunRepresentation`.
- Cross-link from the consolidated known-limitations list (W6).

### W11. Port low-risk first-order effects and NonDet aggregation

Finding: section 10 and section 11 (P1).

Problem: commonly-needed first-order effects are missing (Fresh, Input,
Output, KVStore). For nondeterminism, basic aggregation already exists:
`named_helpers/nondet.rs` provides `run_empty` (into `Option<A>`) on all
six wrappers and `run_choose` (into `Vec<A>`, concatenating branches) on
the four multi-shot wrappers. The genuinely-missing nondeterminism pieces
are narrower: a combined `Choose` + `Empty` runner that interprets both
in one pass into a single `Alternative`-style collection (heftia's
`runNonDet` / `runNonDetMonoid`), and a first-success / short-circuit
helper.

Approach choice per effect: a thin dedicated effect plus a named runner
that reinterprets onto State / Writer, versus documenting "use State /
Writer directly."

Recommendation: add thin dedicated effects (Fresh, Input, Output,
KVStore), each with a named runner that reinterprets onto State / Writer,
for discoverability and parity with heftia; for nondeterminism, add only
the combined `Choose` + `Empty` runner and a first-success helper, not
another `Vec` aggregator (the per-effect `run_choose` / `run_empty`
already cover that). A dedicated effect costs more brands (mitigated once
W2 lands) but gives clearer intent and better errors; reuse-only adds
zero types but has poor discoverability.

Concrete steps:

- Prefer to do these after W2 so each effect is a single spec; if done
  before, implement on the multi-shot wrappers first.
- Decide the KVStore map convention (open question 7).
- Ship a named runner and helper constructors per effect family,
  matching the existing State / Except / Writer helper style.
- Add a combined `Choose` + `Empty` runner and a first-success helper; do
  not duplicate the existing `run_choose` / `run_empty`.

### W12. Port moderate effects: Coroutine, Log, Fail

Finding: section 10.

Problem: these are valuable but each needs a decision first.

- Coroutine (`Yield a b`): needs a `Status` type (`Done | Continue a k`)
  and a shot-semantics decision; fits the multi-shot wrappers.
- Log: an Output specialization; should follow the Output / Writer
  decisions rather than lead them.
- Fail: `Except<String>`-like; needs a decision on whether a distinct
  identity from `Except` is warranted.

Recommendation: schedule after W11 and after the relevant decisions
(Status shape and shot semantics, Fail identity, Output / Writer
convention), and implement via the generator.

Concrete steps:

- Resolve open questions 8 and 9.
- Implement each effect and its runner; add tests.

### W13. Runtime policy, then async interpreter, then deferred ports

Finding: sections 6 and 10, and section 11 (P3).

Problem: there is no async interpreter, and Shift / CC, Provider, Unlift,
and the Concurrent family are blocked on undecided runtime semantics.

Approaches for the async step:

- A. `MonadRec`-over-`Future` so the existing stack-safe interpreter
  shape extends to async targets.
- B. A dedicated async substrate (additional wrapper families).
- C. External `spawn_blocking` only (status quo).

Recommendation: write the runtime policy first, then implement A if the
policy allows, then schedule the deferred ports against that policy.
These ports encode runtime commitments (executor model, cancellation, IO
embedding, process lifecycle, target-monad lifting, continuation
exposure, `Send + Sync`) that must be decided once and centrally rather
than per-effect.

Concrete steps:

- Author the runtime policy doc (open question 12).
- Decide the async approach; implement.
- Then schedule Shift / CC, Provider, Unlift, and the Concurrent family,
  each linking to the policy. Shift / CC additionally requires
  answer-type-polymorphic continuation capture, which is beyond the
  current mono-in-`A` model and must be designed against the policy.

## Suggested implementation order

This is a starting proposal, subject to the open questions (especially 1
and 13).

1. Documentation and small coherence fixes with no policy commitment:
   W6, W7, W10, W5, and the W8 design note.
2. Composability: W1 (`expand` / `weaken`).
3. Brand coherence: W3 (brand / class capability audit, fix stale docs,
   decide gaps).
4. The big lever: W2 (code generation), with W4 (feature-gating)
   alongside.
5. Effect ports on the generated base: W11, then W12.
6. Generic scoped rows: W9, when a concrete need arrives.
7. Policy-gated runtime work: W13.

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
13. Sequencing (W1, W11 vs W2). Do `expand` and the low-risk effect ports
    wait for the generator (W2) to avoid writing six hand copies, or land
    first and get folded into the generator later? Trade-off:
    time-to-value vs rework.
14. Macro behavior under feature-off (W4). When the `effects` feature is
    off, what do the effect proc-macros do? Options: stay always exported
    and fail through an unresolved generated path, become `cfg`-gated, or
    surface a "requires the `effects` feature" diagnostic via an
    `fp-library` re-export shim. `fp-macros` cannot read `fp-library`'s
    active features directly.

## Traceability

This plan is derived from [`findings.md`](findings.md). Future
implementation commits should cite the relevant work item (W-number) and
any decided open-question number in their commit bodies or plan updates.
