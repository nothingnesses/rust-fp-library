# Agent Prompt: Propose Remediations for the Phase 4 Design Review

Copy the section below into the agent. It assumes the agent has read
access to the repository and the heftia reference codebase, and can
use Read, Grep, Glob, WebFetch, and the LSP tool.

---

## Task

The user has previously commissioned an adversarial review of the
proposed Phase 4 dual-row scoped-effects design. That review is
already written. Your job is to **take each finding from that review
and propose concrete ways to address it**, including the trade-offs
between alternative approaches and a final recommendation.

The user is **especially interested in correct approaches that
holistically tackle the fundamental issues**, even if the correct
approach causes API breakages, replaces the proposed encoding
wholesale, or invalidates earlier design decisions in `decisions.md`
or already-shipped Phase 1-3 code. Phase 4 has not been implemented,
so the cost of choosing a different encoding now is much lower than
it will ever be again. Optimise for getting Phase 4 right, not for
preserving the current plan text.

The user wants three things from you, in order of importance:

1. **Options.** For each fundamental finding, list at least two viable
   remediations wherever the design admits more than one. If only one
   approach is genuinely viable, say so explicitly and explain why
   alternatives were rejected. Surfacing alternatives is the point
   of this exercise; single-option findings should be the exception,
   not the default. For major findings, two options where they exist;
   one is acceptable when alternatives are clearly inferior. For
   minor findings, a one-sentence fix.
2. **Trade-offs.** For each option, explain what it costs and what it
   buys. Be specific: name the impact on type inference, on
   performance, on API stability, on plan scope, on the implementation
   effort required, on which other findings the option also resolves
   or creates, and on which Phase 1-3 invariants the option preserves
   or breaks. **Holistic options that touch Phase 1-3 substrate code
   are explicitly welcome**; do not artificially constrain options to
   "Phase 4 only" if a deeper change is the principled answer.
3. **Recommendation.** State the option you recommend and why. Tie
   the recommendation to concrete criteria (the rubric's
   capabilities, what heftia actually delivers, the project's
   established conventions where they are not part of the problem,
   Rust's hard constraints, the relative severity of the trade-offs).
   The recommendation is your honest engineering judgment, not a
   hedge. **Do not pick the smallest option by default.** If the
   smallest option papers over a fundamental issue, name that and
   recommend a deeper option. The user has been explicit that
   correctness beats minimal-blast-radius for this review.

You are not implementing anything. This is a planning artefact. Do
not edit code. Do not write tests. Do not modify any file outside the
output report described below.

## Inputs

Read all of the following before producing output. They are listed in
the order you should read them.

1. **Review report (the source of findings):**
   `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/1_scoped_effects_design/review_phase_4_design.md`.
   This is your working set. Every finding in it must appear in your
   output. If the report is missing, stop and tell the user; do not
   fabricate findings.
2. **Theoretical rubric:**
   `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/algebraic_effects.md`.
   Use this to evaluate whether a proposed remediation actually
   restores a capability the system should have, versus papering
   over a symptom.
3. **Heftia reference codebase:**
   `/home/jessea/Documents/projects/effects/heftia`. Read enough of
   the core types in `heftia/src/Data/Hefty/...` and at least 3-4
   scoped-effect modules in
   `heftia-effects/src/Control/Monad/Hefty/...` (e.g.,
   `Reader.hs`, `Except.hs`, `NonDet.hs`, `Coroutine.hs`,
   `Provider.hs`) to ground each remediation in what heftia
   actually does. When a remediation invokes heftia's approach,
   cite the heftia file and shape; do not handwave.
4. **Design plan:**
   `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md`.
   Use the Phase 4 section as the as-written design and the rest
   of the plan to understand what Phase 4 must compose with.
   Some remediations will require revising the plan substantially;
   describe the revision text where applicable.
5. **Substrate the design builds on:**
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/`,
   especially `node.rs`, `interpreter.rs`, the six `Run` wrappers,
   and the existing first-order effects. When a remediation would
   touch substrate code, name the file and rough line region.
6. **Project decisions and deviations:**
   `decisions.md` (especially section 4.5 on scoped effects),
   `resolutions.md` (everything mentioning scoped row, `Node::Scoped`,
   or dual-row), and `deviations.md`. Recommendations should
   acknowledge prior decisions; **prior decisions are not sacrosanct
   for Phase 4** since Phase 4 has not shipped, but a recommendation
   that overturns a prior decision must say so explicitly and justify
   the reversal.
7. **Project conventions:** `CLAUDE.md` and `AGENTS.md` at the repo
   root. Recommendations should be consistent with established
   conventions or, if they require breaking from convention, must
   justify the break.
8. **Prior remediation report (for tone reference and continuity):**
   `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/0_first_order_effects_implementation/remediation_proposals.md`.
   This is the remediation report for the Phase 1-3 substrate. Read
   it briefly to understand the established remediation style and to
   avoid duplicating sequencing-plan items it already covers. Where
   a Phase 4 remediation interacts with a still-outstanding item from
   that report (e.g., the `interpret_with_rec` deferral), cross-
   reference it.

## Per-finding output structure

For every finding in the review report (Fundamental, Major, and
Minor), produce a section in the report you write. The structure
depends on severity.

### For Fundamental and Major findings

```
### F1. <Title copied from the review>

**Restated:** <Two- or three-sentence restatement of the problem in
your own words. This proves you understood it before proposing fixes.
If your restatement reveals the problem is wider or narrower than
the review claimed, say so explicitly.>

**Root cause:** <What the underlying cause is, distinct from the
symptom. Multiple findings may share a root cause; note that here.
If the root cause is "the wrong encoding was chosen", state that
explicitly; do not soften it.>

**Option A: <Short label>**
- *What:* <Concrete description of the change. Reference plan
  sections, decisions sections, substrate files and line regions
  where applicable. Reference heftia source where the option mirrors
  heftia.>
- *Cost:* <Implementation effort, ergonomic impact, performance
  impact, type-inference impact, API breakage versus the current
  plan, plan revisions required, Phase 1-3 substrate code touched.>
- *Benefit:* <What it buys; which other findings it also resolves;
  which capabilities it restores; which heftia properties it
  preserves.>
- *Risks:* <What could go wrong; what assumptions it relies on;
  whether stable Rust is sufficient or whether nightly features
  would be required.>

**Option B: <Short label>**
<same fields>

**Option C (if applicable):** <same fields>

**Recommendation:** <One option, named explicitly. Two- to four-
sentence justification grounded in the criteria you used to choose.
Cite the relevant rubric section, plan claim, heftia property, or
convention. If the recommendation overturns a prior decision in
`decisions.md` or `resolutions.md`, name the decision and explain
why it should be reversed.>

**Dependencies and ordering:** <List any other findings whose
remediation must precede or follow this one. If standalone, say so.
If the recommendation requires Phase 1-3 substrate changes that have
their own ordering implications, name them.>

**Plan revision required:** <Yes or No. If yes, describe what the
plan needs to say differently after this change. If yes and large,
the recommendation may include "Phase 4 is rewritten" as a
plan-revision outcome.>
```

### For Minor findings

A condensed bullet block is sufficient:

```
- **m1.** <Title>. <Proposed fix in one or two sentences, with
  citation if applicable.>
```

Skip options and trade-offs for Minor findings unless the fix is
non-obvious. If a Minor finding is non-obvious, promote it to the
Fundamental/Major structure.

## Cross-cutting analysis

After per-finding sections, produce four additional sections (one
more than the prior remediation prompt; the new one is "Holistic
redesigns" because the user explicitly invited them):

1. **Root cause clusters.** Group findings by shared root cause.
   Identify which clusters can be resolved together by a single
   remediation. This is often where the highest-leverage work sits:
   one structural change may close five findings at once.
2. **Holistic redesigns.** If multiple fundamental findings share
   a root cause that is structural (e.g., "the encoding cannot
   represent inspectable scoped bodies"), name the wholesale
   redesigns that would resolve the cluster. Each redesign should
   be its own subsection: name it, sketch the encoding (citing
   heftia or another reference where possible), describe what
   Phase 1-3 invariants it preserves or breaks, and identify which
   findings it resolves. **Be candid: if the right answer is "the
   freer-monad-encoded scoped-effects design should be replaced
   wholesale with an HFunctor-style encoding before Phase 4 ships",
   say so.** A redesign recommendation here can be the report's
   single most important conclusion.
3. **Sequencing plan.** Propose an ordering for the remediations as
   a numbered list, considering dependencies, risk, and the relative
   payoff. Distinguish "must do before Phase 4 starts" from "do
   during Phase 4" from "defer with rationale". For each item,
   state the rough size: small (a session), medium (a few days),
   large (a redesign that touches substrate code), x-large (rewrite
   the Phase 4 plan).
4. **Findings with no clean fix.** Some findings may have no clean
   remediation in the freer-monad-encoded substrate the project
   has chosen; the only honest answer is to accept the limitation,
   declare it a non-goal in the plan, and document the consequence.
   Call these out explicitly. Do not fudge them with hopeful options.
   Distinguish "not fixable in this encoding" from "not fixable in
   stable Rust" from "not fixable until upstream change X lands".

## Quality bar

A good remediation proposal:

- Names the file and line regions it would touch, when local.
- Names the heftia file/shape it mirrors, when proposing an
  approach inspired by heftia.
- States its impact on type inference quantitatively when possible
  ("adds one type parameter to `Run`, propagated through ~12 call
  sites").
- Distinguishes "fixes the symptom" from "fixes the root cause" and
  prefers the latter when the cost is comparable. **Prefers the
  latter even when the cost is meaningfully higher**, given the
  user's stated priorities for Phase 4.
- Notes when it conflicts with the plan, with a project convention,
  or with a prior decision in `decisions.md` / `deviations.md` /
  `resolutions.md`, and explicitly justifies the reversal.
- Is honest about uncertainty. If you do not know whether an option
  is feasible without prototyping, say so and describe what
  experiment would resolve the uncertainty.

A bad remediation proposal:

- Hand-waves about "refactoring this trait" without saying which
  trait, how, or what it would look like after.
- Proposes a remediation that the rubric, plan, or prior review
  already considered and rejected, without acknowledging the prior
  rejection.
- Recommends the smallest, easiest option in every case
  (under-engineering) when the user has been explicit about
  preferring correct holistic options.
- Treats every finding as independent when several share a root
  cause and a single redesign closes them all.

## Anti-patterns to avoid

- **Plan deference.** If the plan is wrong, say so. Do not refuse
  to recommend a fix just because the plan does not anticipate it.
  The plan can be revised; in fact, for Phase 4, the plan probably
  _should_ be revised.
- **"It depends" recommendations.** If the right answer truly
  depends on context, name the context that decides it and pick a
  default for the most likely case.
- **Padding with low-value options.** Two viable options is better
  than four where two are obviously bad. Strawmen are noise.
- **Silent scope expansion.** If a remediation requires touching
  Phase 1-3 substrate code, name what gets touched. Do not present
  a "just one trait change" recommendation that quietly metastasizes.
- **Pretending API breakage is not on the table.** The user has
  said it is. Do not soften "this requires a breaking change to the
  Run wrapper signatures" into "this requires a careful migration".
  Just say it requires a breaking change and identify the migration.

## Output

Write the result to
`/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/1_scoped_effects_design/remediation_proposals_phase_4.md`.
If the file exists, overwrite it. Use this top-level structure:

```
# Remediation Proposals for the Phase 4 Dual-Row Scoped-Effects Design Review

## Summary
<3-5 sentences. Total finding count by severity, count of root-cause
clusters, your single highest-leverage recommendation, and any
finding flagged as having no clean fix. State explicitly whether the
report is recommending iterative remediation of the existing plan
or a wholesale redesign of Phase 4.>

## Fundamental Findings
<F1, F2, ... with the full per-finding structure>

## Major Findings
<M1, M2, ... with the full per-finding structure>

## Minor Findings
<Bullet list>

## Root Cause Clusters
<Grouping analysis>

## Holistic Redesigns
<Subsection per redesign candidate; see structure above>

## Sequencing Plan
<Numbered ordering with sizing>

## Findings with No Clean Fix
<List with rationale and proposed plan-revision text>
```

## Rules of engagement

- Cite plan sections by header text and line region; cite heftia
  files by path and line region; cite substrate files by path and
  line region. Use relative-link Markdown.
- Quote short snippets (~5 lines) from plan.md, heftia, or the
  substrate when they sharpen a remediation.
- Use ASCII only: no em-dashes, en-dashes, unicode arrows or math,
  no emoji or unicode symbols. Use `->`, `<-`, `>=`, `<=`, `!=`.
  Use commas or semicolons in place of dashes.
- Use the LSP tool aggressively for type-level questions about the
  Phase 1-3 substrate. The Brand pattern, the HKT machinery, and
  the optics layer are nontrivial to trace by reading; LSP `hover`
  and `goToDefinition` will save time and reduce errors.
- Do not run tests, build, or modify any file other than the output
  report. This is a planning artefact, not an implementation step.
- Use the project's commit-message and naming conventions when
  describing what a fix would look like.
- Length budget: thorough, not padded. A fundamental finding may
  warrant a full page, especially if its remediation is a holistic
  redesign; a minor finding should be a single line.

## Starting steps

1. Read `review_phase_4_design.md` end to end. Build a list of every
   finding by ID and severity. If IDs are missing, assign them in
   order.
2. Read `algebraic_effects.md` end to end (or refresh) to refresh
   the standards you will judge remediations against.
3. Spend at least 30 minutes in the heftia codebase reading the
   core types and 3-4 scoped-effect modules. Without this, the
   remediation's "what does heftia actually do here?" question
   cannot be answered.
4. Read `plan.md` Phase 4 section and `decisions.md` section 4.5
   to refresh what the plan is committed to. Read `resolutions.md`
   for any pre-existing scoped-effect decisions.
5. Read enough of `fp-library/src/types/effects/` substrate code
   to ground each remediation that touches substrate.
6. For each finding, draft options before forming a recommendation.
   Resist the temptation to commit to the first option that looks
   plausible. Resist the temptation to commit to the smallest option
   by default; the user has explicitly told you not to.
7. Cluster findings by root cause and reconsider whether your
   per-finding recommendations still make sense in light of the
   clustering. Often a cluster-level remediation supersedes
   per-item ones. **For Phase 4, a cluster-level redesign is a
   real possibility**; do not artificially break it into per-item
   patches.
8. If the cluster analysis surfaces a holistic redesign, write its
   subsection in "Holistic Redesigns" before re-running the per-
   finding recommendations. Per-finding recommendations may then
   reduce to "see redesign R1" plus a one-sentence note about how
   the redesign closes the finding.
9. Sequence the remediations.
10. Write the report.
