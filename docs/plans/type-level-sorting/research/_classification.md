# Stage 1 classification: aggregated findings

**Status:** pending
**Last updated:** _not yet started_

## Purpose

Aggregated synthesis of every per-codebase Stage 1 file in this directory.
Produced only after every entry under "Stage 1: per-codebase
classification" in [\_status.md](_status.md) is ticked. The purpose is to
turn the 11 individual findings into a decision artefact: which approaches
work in Rust, which crates implement them, and whether any path is
promising enough to warrant a Stage 2 deep dive or a port-plan edit
recommendation for the effects research's section 4.1 workaround 2
question.

This file is intentionally a placeholder until the synthesis runs. The
contract for filling it in mirrors the effects research's
`_classification.md`.

## Required structure (to be populated later)

An agent filling in this document must produce the following sections.

### Intro

_(One paragraph: what Stage 1 set out to answer and what this synthesis
delivers.)_

### Classification table

_(One row per codebase. Columns: codebase name, primary approach, sort
capability (yes / partial / no), stable or nightly, applicability to
coproduct row canonicalisation.)_

### Approach-by-approach summary

_(One subsection per approach (1-8) summarising which codebases cluster
there and the strongest evidence for or against the approach.)_

### Recommendations for Stage 2

_(Which, if any, Stage 2 deep dives are worth scheduling. For each, name
the specific question it should answer.)_

### Relevance to port-plan

_(Would findings here change any decision in
[../../effects/port-plan.md](../../effects/port-plan.md) section 4.1's
ordering-mitigations subsection (workaround 2)? Answer "no changes
recommended" explicitly if that is the conclusion.)_

## Closing checklist

- [ ] All sections above populated
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to tick `_classification.md` and list any
      Stage 2 deep dives under "Stage 2: deep dives"
- [ ] Word count under ~2500
