# Agent Prompt: Adversarial Review of the Phase 4 Heftia-Inspired Dual-Row Scoped-Effects Design

Copy the section below into the agent. It assumes the agent has read
access to two repositories (the Rust port in `rust-fp-lib`, the Haskell
heftia codebase in a sibling directory) and can use Read, Grep, Glob,
WebFetch (for the heftia paper if needed), and the LSP tool.

---

## Task

You are performing an **adversarial technical review** of a proposed
_design_ (not an implementation) for a Rust scoped-effects subsystem
that is described as "heftia-inspired dual row". The design lives in
the Phase 4 phasing section of the project's plan document; nothing has
been implemented yet. Phases 1-3 (the first-order effect substrate,
interpreters, and standard first-order effects) shipped already and are
the substrate the Phase 4 design extends.

The user wants you to be harsh and specific. They are explicitly **not**
looking for a balanced report. They want a list of every meaningful
problem you can find, with the most important problems being
**fundamental flaws** that the design either cannot deliver as written
or that would require redesigning the system to fix.

**They are also explicit about wanting only valid and relevant
findings**, not padding or red herrings. If you are not sure a finding
is real, gut-check it twice before promoting it. Strawman concerns
("the type system is complex"), aesthetic complaints ("the names are
bad"), and concerns the design has already foreseen and addressed
elsewhere are noise. Suppress them. The bar for inclusion is that the
finding identifies a real obstacle to the design delivering its stated
properties, not just an aesthetic objection.

Do not soften your critique on findings that are real. Do not pad with
positives unless a positive directly bears on whether a negative is a
real problem. Do not invent fixes; the goal is diagnosis, not treatment.

## Files

- **Design under review:** the `### Phase 4: Scoped effects (heftia
dual row)` section of
  `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md`
  plus all surrounding sections that define the substrate it builds on
  (the `Run`/`RcRun`/`ArcRun` family and their Explicit siblings, the
  `effects!` row macro, the `DispatchHandlers` interpreter trait, and
  the existing `Reader`, `State`, `Except`, `Writer`, `Choose`
  first-order effects). Read the plan end to end so you understand
  what claims the design makes about scoped effects, what it claims
  about its continuation handling, what it explicitly puts out of
  scope, and which earlier decisions in the plan it builds on or
  contradicts.
- **Cross-references for the design:**
  `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md`
  (especially section 4.5 on scoped effects and the `Catch` /
  `Local` / `Bracket` / `Span` sub-decisions) and
  `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md`
  (especially anything mentioning the `Node::Scoped` arm, scoped-row
  shape, or dual-row dispatch).
- **Existing implementation:** the `Phase 1-3` substrate that Phase 4
  builds on lives at
  `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/`
  and supporting brand / class definitions. Read enough of it to
  understand what the substrate provides today: the `Node<R, S, A>`
  enum's `Scoped` arm (currently structurally uninhabited via
  `S = CNilBrand`), the `DispatchHandlers` trait shape, how
  `interpret`/`interpret_with`/`interpret_rec` currently dispatch, and
  what `Run<R, S, A>`'s public surface looks like. The gap between
  what exists and what Phase 4 would need is a critical input to your
  review.
- **Heftia reference codebase:** `/home/jessea/Documents/projects/effects/heftia`.
  This is the Haskell library the design names as its inspiration.
  Skim the high-level architecture (`heftia/src/Data/Hefty/...`,
  `heftia/src/Control/Hefty/...` and similar paths), the core data
  type and its handler shape, and at least one or two of the
  scoped-effect modules under
  `heftia-effects/src/Control/Monad/Hefty/`
  (e.g., `Reader.hs`, `Except.hs`, `NonDet.hs`, `Coroutine.hs`,
  `Provider.hs`, `Unlift.hs`). Note specifically: how heftia
  represents the dual row, how it represents continuations inside
  scoped operations, what shape its handler clauses take, and what
  it does about resource safety. This is the primary point of
  comparison; the proposed Rust design claiming "heftia-inspired"
  is only valid to the extent it preserves heftia's load-bearing
  properties.
- **Theoretical rubric:**
  `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/algebraic_effects.md`.
  Defines what algebraic effect systems are, what scoped-effect
  capabilities a complete system should support, and how various
  implementations differ. Use it as the standard the design should
  be measured against. Cite specific section numbers when invoking
  it.
- **Prior review (for tone reference and scope deltas):**
  `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/0_first_order_effects_implementation/review_effects_rs.md`
  reviewed the Phase 1-3 substrate and surfaced findings F1-F5 / M1-M9
  that have since been resolved or deferred (see `resolutions.md`).
  Read it briefly to understand the project's established critique
  style; do not re-litigate findings it already raised that have been
  resolved. Where a Phase 4 finding overlaps with an existing
  unresolved finding, cross-reference it explicitly rather than
  duplicating.

## Surrounding context

- The host project is a Rust functional-programming library that
  encodes higher-kinded types via a Brand pattern and uses heavy
  type-level machinery (see `CLAUDE.md` and `AGENTS.md` at the repo
  root for conventions). The Phase 1-3 effects substrate is one of
  the project's more ambitious subsystems and Phase 4 is the most
  ambitious extension on the roadmap.
- The substrate is **freer-monad-encoded** with two parallel families
  (Erased: `Free`, `RcFree`, `ArcFree`; Explicit: their
  `*FreeExplicit` siblings). Six `Run` wrappers correspond to the
  six Free variants. The substrate's continuation queue was recently
  migrated from value-typed `CatList` to refcounted `RcCatList` /
  `ArcCatList` to support multi-shot dispatch on the Erased family
  (this is finished as of 2026-05-04).
- Phase 4 is **explicitly unimplemented**. Your review evaluates the
  _plan_, not code that does not exist yet. Distinguish "the plan
  describes a path to delivering capability X but the path requires
  Rust features that do not exist or do not compose" from "the plan
  describes a workable path but glosses over implementation details".
  The first is fundamental; the second is major.
- Rust's lack of native delimited continuations, its borrow checker,
  its restricted higher-rank polymorphism, and its no-rank-2-closures
  property are all load-bearing constraints. Pay particular attention
  to whether the design has a credible answer for each, especially
  around scoped operations whose semantics in heftia rely on running
  user-supplied sub-programs under handler control.

## Dimensions to evaluate

Evaluate the design along at least the following axes. For each, decide
whether the design as written succeeds, partially succeeds, or fails,
and explain in concrete terms with file/line citations into the plan
document and into heftia for comparison.

1. **Scoped-effect semantics.** Does the proposed dual-row design
   actually deliver heftia-equivalent semantics, or does it adopt
   heftia's name without heftia's substance?
   - Is a scoped operation's body a first-class program the handler
     can inspect and control (run zero, one, or many times; modify
     the environment under which it runs; abort it part-way)?
   - Or is the body a `Box<dyn FnOnce(...) -> Run<...>>` that the
     handler invokes opaquely?
   - Where does heftia put the body, and what does that buy heftia
     that the proposed encoding does or does not preserve?
2. **Continuation flexibility for scoped operations.** Heftia's
   `local`, `catch`, and `bracket` differ from the first-order
   `Reader`/`Except`/`Bracket`-as-FFI in that the handler can
   intercept the scoped sub-program's effects mid-flight. Does the
   plan's `Box<dyn FnOnce(E) -> E>` / `Box<dyn FnOnce(A) -> Run<...>>`
   shape preserve that, or does it collapse to "run the sub-program
   to completion, then post-process"? If the latter, what scoped-
   effect properties from the rubric and from heftia are lost?
3. **The `Box<dyn FnOnce>` choice.** The plan uses `Box<dyn FnOnce>`
   for the scoped-effect closure types. Single-shot only. What
   scoped effects in heftia or the rubric require multi-shot
   continuations on the body? Does the plan address that, deflect it,
   or silently drop it? If multi-shot bodies are out of scope, is the
   "out of scope" claim load-bearing for the design's correctness?
4. **Val/Ref dispatch for `Bracket` and `Local`.** The plan ships two
   parallel flavours of `Bracket` and `Local`, dispatched by
   closure-type-driven Val/Ref markers. Is this principled or is it
   working around an inability to express a single uniform shape?
   Does the user have to know which flavour they want before they
   start writing the program, or is the choice dispatched
   automatically? What happens if a user mixes flavours within one
   program?
5. **Interaction with the six wrappers.** Each scoped effect must work
   across the six `Run` wrappers (single-shot vs multi-shot,
   thread-safe vs not, Erased vs Explicit). Is the plan precise about
   which scoped effects work on which wrappers? Are there silent
   omissions (e.g., `Bracket`'s Val flavour on multi-shot wrappers)?
   What does heftia do here for comparison; does heftia have a similar
   axis at all?
6. **The `Node::Scoped` arm and the `S = CNilBrand` invariant.** The
   current substrate fixes `S = CNilBrand` so the scoped arm is
   structurally uninhabited via `match cnil {}`. Phase 4 must lift
   that invariant. What changes to the interpreter family does the
   plan describe to handle a populated scoped row? Does the existing
   `DispatchHandlers` trait extend cleanly, or does Phase 4 require a
   parallel `DispatchScopedHandlers` trait? If the latter, what does
   the plan say about its shape and how does that compare to heftia's
   handler form?
7. **Handler composition across the dual row.** Can a user compose a
   first-order handler list with a scoped handler list and get the
   right thing, or does the dual structure introduce a new ordering
   constraint? What happens when a scoped handler's body calls a
   first-order effect handled by a separate handler? Does the plan
   address this composition shape?
8. **Resource safety with single-shot bodies.** `Bracket`'s
   `release` runs after `body`. If the scoped body panics, does
   `release` still run? If `body` consumes the resource and the
   handler's interpreter thread crashes mid-bind, what happens?
   What does the plan say about panic interaction; what does heftia
   guarantee here?
9. **Type inference and ergonomics.** What does a typical caller's
   signature look like with both rows? How verbose is a Run
   signature with two rows of three effects each? Are there
   inference cliffs (cases where the user must annotate
   extensively)? How does this compare to heftia's syntactic story?
10. **Performance.** Each scoped operation allocates a `Box` for the
    body closure (and for the modify / handler closures where
    applicable). Combined with the freer-monad-encoded substrate's
    per-bind allocation, what is the per-scoped-op cost? Are there
    any fusion opportunities the design forecloses? Does heftia have
    fusion the proposed design lacks?
11. **Plan vs. heftia consistency.** Where the plan claims "heftia-
    inspired" or invokes heftia's semantics, check the heftia source.
    Are the claimed properties actually delivered by the proposed
    encoding? Where the plan's encoding diverges from heftia's, is
    the divergence acknowledged with a tradeoff analysis, or
    silent?
12. **Internal consistency with Phases 1-3.** Phase 4 builds on the
    Erased and Explicit substrate families plus the Coyoneda-headed
    row. Does the scoped-effects encoding compose cleanly with
    `RcCatList`/`ArcCatList`? With the `interpret_rec` /
    `tail_rec_m` stack-safety story? With the `Send + Sync` cascade
    on the Arc family? Are there places the existing substrate
    constraints rule out scoped-effect features the plan promises?
13. **Comparison to other scoped-effect designs.** Heftia is one
    point in a design space that includes `polysemy`'s
    `Tactical`/`HFunctor`, `fused-effects`'s
    `Algebra (h :+: r) m`, `freer-simple`'s `Eff` with handler
    interpreters, and the older `extensible-effects` formulations.
    The plan picks heftia. Briefly: which problems of those
    alternative families does the plan pick up by following heftia,
    and which does it inherit? Are there scoped effects from
    heftia-effects that the plan would NOT be able to port (e.g.,
    `Coroutine`, `NonDet`, `Provider`, `Unlift`)? If so, why?

## What counts as a "fundamental flaw"

Mark a problem as **fundamental** only if at least one of the
following is true:

- Fixing it would require changing the core encoding (e.g., switching
  from a `Box<dyn FnOnce>` body to a stored sub-program with a
  separate dispatch mechanism, or moving from freer-monad-encoded
  scoped effects to an HFunctor-encoded substrate).
- It violates a property the plan or `decisions.md` explicitly claims
  to guarantee.
- It precludes a capability listed in the rubric's relevant scoped-
  effect sections that the plan claims to support.
- It is a soundness hole reachable from safe code or a resource-
  safety issue that the design cannot patch without restructuring.
- It silently diverges from heftia in a way that loses heftia's
  load-bearing property while still using the name.

Mark a problem as **major** if it is a serious design problem that
can be fixed by a local change to the plan (a different smart
constructor shape, a different per-effect signature, a different
interpreter trait method) without changing the core encoding.

Mark a problem as **minor** if it is a real defect but routine
(naming, terminology, missing feature flag, gaps in the per-
constructor table). Save these for the bullet list; don't promote.

If you are uncertain whether something is fundamental or major, err
toward fundamental and explain the uncertainty. If you are uncertain
whether something is a real finding at all, err toward suppressing
it. The user has stated "no red herrings" explicitly.

## Output format

Write the report to
`/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/1_scoped_effects_design/review_phase_4_design.md`.
If a file already exists at that path, overwrite it; prior reviews
are tracked through git history.

Use this Markdown structure:

```
# Adversarial Review of the Phase 4 Dual-Row Scoped-Effects Design

## Summary
<3-5 sentences. Your overall verdict, the count of fundamental /
major / minor findings, and the single most important issue. State
explicitly whether the design as written is workable, workable with
revisions, or fundamentally requires a different encoding.>

## Fundamental Flaws
For each:
### F1. <Short title>
- **Where:** <plan.md / decisions.md / heftia file:line citations,
  multiple if needed>
- **What:** <concrete description of the problem>
- **Why fundamental:** <why local plan revisions do not suffice; cite
  which rubric section, plan claim, or heftia property it violates>
- **Worked example:** <a minimal scoped-effect scenario or comparison
  to a heftia program that exhibits the problem; preferably with a
  reference to a heftia-effects module that uses the same shape>

## Major Issues
Same structure, less depth.

## Minor Issues
Bullet list with citation and a one-sentence description each.

## Plan vs. Heftia Drift
Section listing places where the plan invokes heftia's name or claims
heftia-equivalent semantics but the proposed encoding diverges.
Quote both the plan claim and the heftia source for each.

## Comparison to Existing Scoped-Effect Designs
Identify which scoped-effect family the proposed design ends up
closest to (heftia, polysemy `Tactical`, fused-effects, freer-simple,
extensible-effects, none-of-the-above). Name the problems of the
closest family the design inherits, and identify any novel problems
specific to this design.
```

## Rules of engagement

- Cite plan sections by header text and line range; cite heftia files
  by path and line range. Use relative-link Markdown:
  `plan.md:1970-2052`.
- Quote short snippets (~5 lines) from plan.md or heftia when they
  make a point sharper.
- Use ASCII only: no em-dashes, en-dashes, unicode arrows or math, no
  emoji or unicode symbols. Use `->`, `<-`, `>=`, `<=`, `!=`. Use
  commas or semicolons in place of dashes.
- Do not propose redesigns unless flagging that the issue is
  "redesign-only" (which is what makes it fundamental in the first
  place). The follow-up remediation prompt will collect proposals.
- Do not run tests, build, or modify files. This is a read-only
  review.
- Use the LSP tool aggressively for type-level questions about the
  Phase 1-3 substrate the design builds on.
- If the plan is silent on a topic where the design makes a
  consequential commitment by omission, treat that as itself a
  finding (the design has unstated commitments).
- Length budget: thorough, not padded. The user has explicitly said
  they do not want padding. A fundamental finding may warrant a full
  page; a minor finding should be a single line.

## Starting steps

1. Read `algebraic_effects.md` end to end if you have not already,
   focusing on the scoped-effect sections and the heftia comparison
   if present.
2. Read `plan.md`'s Phase 4 section end to end, plus
   `decisions.md` section 4.5 and any related resolutions in
   `resolutions.md`. Note claimed scope, claimed semantics, and
   explicit non-goals.
3. Spend at least 30 minutes in the heftia codebase: read the core
   `Data/Hefty/...` types, read 3-4 scoped-effect modules from
   `heftia-effects/...`, and form a picture of what heftia actually
   does and what its handler clauses look like. Without this, the
   "is it actually heftia-inspired?" question cannot be answered.
4. Read enough of `fp-library/src/types/effects/` to understand the
   substrate the design builds on: especially `node.rs`,
   `interpreter.rs`, and one of the wrapper files (e.g., `run.rs`).
5. Form a preliminary thesis about whether the design's claimed
   "dual row" is heftia-equivalent or a freer-monad lookalike, and
   verify it against the plan and the heftia source.
6. Then, with the rubric, plan, and heftia source in mind, work
   through the dimensions above. For each finding, ask yourself
   twice: is this real? Could the plan reasonably be read to address
   this without restructuring? If yes, suppress it.
7. Write the report.
