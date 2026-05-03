# Agent Prompt: Propose Remediations for Review Findings

Copy the section below into the agent. It assumes the agent has read
access to the repository and can use Read, Grep, Glob, and the LSP
tool.

---

## Task

The user has previously commissioned an adversarial review of the WIP
algebraic effects implementation in this repository. That review is
already written. Your job is to **take each finding from that review
and propose concrete ways to address it**, including the trade-offs
between alternative approaches and a final recommendation.

The user wants three things from you, in order of importance:

1. **Options.** For each finding, list at least two viable remediations
   wherever the design admits more than one. If only one approach is
   genuinely viable, say so explicitly and explain why alternatives
   were rejected. Surfacing alternatives is the point of this exercise;
   single-option findings should be the exception, not the default.
2. **Trade-offs.** For each option, explain what it costs and what it
   buys. Be specific: name the impact on type inference, on
   performance, on API stability, on plan scope, on the implementation
   effort required, and on which other findings the option also
   resolves or creates.
3. **Recommendation.** State the option you recommend and why. Tie the
   recommendation to concrete criteria (the project's existing
   conventions, Rust's constraints, the plan's stated goals, the
   relative severity of the trade-offs). The recommendation is your
   honest engineering judgment, not a hedge.

You are not implementing anything. This is a planning artefact. Do not
edit code. Do not write tests. Do not modify any file outside the
output report described below.

## Inputs

Read all of the following before producing output. They are listed in
the order you should read them.

1. **Review report (the source of findings):**
   `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/review_effects_rs.md`
   This is your working set. Every finding in it must appear in your
   output. If the report is missing, stop and tell the user; do not
   fabricate findings.
2. **Theoretical rubric:**
   `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/algebraic_effects.md`
   Use this to evaluate whether a proposed remediation actually
   restores a capability the system should have, versus papering over
   a symptom.
3. **Design plan:**
   `/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md`
   Use this to understand what the implementation is _trying_ to be.
   Some remediations may require revising the plan; flag those
   explicitly.
4. **Implementation:**
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects.rs`
   plus the surrounding effects module. Read enough of it to ground
   your remediations in the actual code, especially when proposing
   local refactors. You should be able to point at the file and lines
   a fix would touch.
5. **Project conventions:**
   `CLAUDE.md` and `AGENTS.md` at the repo root, plus the `decisions.md`
   and `deviations.md` already in `docs/plans/effects/`. Recommendations
   should be consistent with established conventions or, if they
   require breaking from convention, must justify the break.

## Per-finding output structure

For every finding in the review report (Fundamental, Major, and Minor),
produce a section in the report you write. The structure depends on
severity.

### For Fundamental and Major findings

```
### F1. <Title copied from the review>

**Restated:** <Two- or three-sentence restatement of the problem in
your own words. This proves you understood it before proposing fixes.>

**Root cause:** <What the underlying cause is, distinct from the
symptom. Multiple findings may share a root cause; note that here.>

**Option A: <Short label>**
- *What:* <Concrete description of the change. Reference files and
  lines where applicable.>
- *Cost:* <Implementation effort, ergonomic impact, performance
  impact, type-inference impact, API breakage, plan revisions
  required.>
- *Benefit:* <What it buys; which other findings it also resolves;
  which capabilities it restores.>
- *Risks:* <What could go wrong; what assumptions it relies on.>

**Option B: <Short label>**
<same fields>

**Option C (if applicable):** <same fields>

**Recommendation:** <One option, named explicitly. Two- to four-sentence
justification grounded in the criteria you used to choose. Cite the
relevant rubric section, plan claim, or convention.>

**Dependencies and ordering:** <List any other findings whose
remediation must precede or follow this one. If standalone, say so.>

**Plan revision required:** <Yes or No. If yes, describe what the
plan needs to say differently after this change.>
```

### For Minor findings

A condensed bullet block is sufficient:

```
- **m1.** <Title>. <Proposed fix in one or two sentences, with
  file:line if applicable.>
```

Skip options and trade-offs for Minor findings unless the fix is
non-obvious. If a Minor finding is non-obvious, promote it to the
Fundamental/Major structure.

## Cross-cutting analysis

After per-finding sections, produce three additional sections:

1. **Root cause clusters.** Group findings by shared root cause.
   Identify which clusters can be resolved together by a single
   remediation. This is often where the highest-leverage work sits:
   one structural change may close five findings at once.
2. **Sequencing plan.** Propose an ordering for the remediations as a
   numbered list, considering dependencies, risk, and the relative
   payoff. Distinguish "must do before continuing the WIP" from "do
   before next milestone" from "defer with rationale". For each item,
   state the rough size: small (a session), medium (a few days),
   large (a redesign).
3. **Findings with no clean fix.** Some findings may have no clean
   remediation in the current design; the only honest answer is to
   accept the limitation, declare it a non-goal in the plan, and
   document the consequence. Call these out explicitly. Do not fudge
   them with hopeful options.

## Quality bar

A good remediation proposal:

- Names the file and lines it would touch, when local.
- States its impact on type inference quantitatively when possible
  (e.g., "adds one type parameter to `Eff`, propagated through ~12
  call sites").
- Distinguishes "fixes the symptom" from "fixes the root cause" and
  prefers the latter when the cost is comparable.
- Notes when it conflicts with the plan, with a project convention,
  or with a prior decision in `decisions.md` / `deviations.md`.
- Is honest about uncertainty. If you do not know whether an option
  is feasible without prototyping, say so and describe what experiment
  would resolve the uncertainty.

A bad remediation proposal:

- Hand-waves about "refactoring this trait" without saying which
  trait, how, or what it would look like after.
- Proposes a remediation that the rubric or plan already considered
  and rejected, without acknowledging the prior rejection.
- Recommends the largest, most ambitious option in every case
  (overcorrection) or the smallest, easiest option in every case
  (under-engineering). Match the option to the severity.
- Treats every finding as independent when several share a root
  cause.

## Anti-patterns to avoid

- **"It depends" recommendations.** If the right answer truly depends
  on context, name the context that decides it and pick a default for
  the most likely case.
- **Padding with low-value options.** Two viable options is better
  than four where two are obviously bad. Strawmen are noise.
- **Plan deference.** If the plan is wrong, say so. Do not refuse to
  recommend a fix just because the plan does not anticipate it. The
  plan can be revised.
- **Silent scope expansion.** If a remediation requires touching code
  outside the effects module, name what gets touched. Do not present
  a "just one trait change" recommendation that quietly metastasizes.

## Output

Write the result to
`/home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/review/remediation_proposals.md`.
If the file exists, overwrite it. Use this top-level structure:

```
# Remediation Proposals for effects.rs Review

## Summary
<3-5 sentences. Total finding count by severity, count of root-cause
clusters, your single highest-leverage recommendation, and any
finding flagged as having no clean fix.>

## Fundamental Findings
<F1, F2, ... with the full per-finding structure>

## Major Findings
<M1, M2, ... with the full per-finding structure>

## Minor Findings
<Bullet list>

## Root Cause Clusters
<Grouping analysis>

## Sequencing Plan
<Numbered ordering with sizing>

## Findings with No Clean Fix
<List with rationale and proposed plan-revision text>
```

## Rules of engagement

- Cite file paths as relative links: `[effects.rs:42](path#L42)`.
- Quote short snippets (~5 lines) when they sharpen a remediation.
- Use ASCII only: no em-dashes, en-dashes, unicode arrows or math, no
  emoji or unicode symbols. Use `->`, `<-`, `>=`, `<=`, `!=`. Use
  commas or semicolons in place of dashes.
- Use the LSP tool aggressively for type-level questions. The Brand
  pattern, the HKT machinery, and the optics layer are nontrivial to
  trace by reading; LSP `hover` and `goToDefinition` will save time
  and reduce errors.
- Do not run tests, build, or modify any file other than the output
  report. This is a planning artefact, not an implementation step.
- Use the project's commit-message and naming conventions when
  describing what a fix would look like, so the reader can map your
  recommendations onto a future commit cleanly.
- Length budget: thorough, not padded. A Fundamental finding may
  warrant a full page; a Minor finding should be a single line.

## Starting steps

1. Read `review_effects_rs.md` end to end. Build a list of every
   finding by ID and severity. If IDs are missing, assign them in
   order.
2. Read `algebraic_effects.md` and `plan.md` to refresh the standards
   you will judge remediations against.
3. Skim `decisions.md` and `deviations.md` to learn what has already
   been settled or explicitly accepted.
4. Read enough of the implementation to ground each remediation in
   real code.
5. For each finding, draft options before forming a recommendation.
   Resist the temptation to commit to the first option that looks
   plausible.
6. Cluster findings by root cause and reconsider whether your
   per-finding recommendations still make sense in light of the
   clustering. Often a cluster-level remediation supersedes per-item
   ones.
7. Sequence the remediations.
8. Write the report.
