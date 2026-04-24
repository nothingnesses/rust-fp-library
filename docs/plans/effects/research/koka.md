# koka

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/effects/koka/`

## Purpose

Stage 1 research document: classify `koka` against the five effect-row
encodings catalogued in [../port-plan.md](../port-plan.md) section 4.1.
Koka has native row polymorphism, so the interesting question is not "is
this one of options 1-5" but "how does Koka lower its row types into a
runtime representation, and could that lowering strategy inform a Rust
encoding?". The second question is where Stage 2 might follow up.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

_(What is the fundamental encoding Koka uses internally to lower its row
types? Evidence passing? Continuation-passing with prompts? Pick the
compiler stage where the encoding crystallises and point at it.)_

### Distinctive contribution relative to baseline

_(What does Koka's lowering do differently from a standard `Free + Coproduct + Member` baseline as used by Haskell effect libraries? The user-facing language feature is different (row polymorphism); focus on what the implementation teaches us.)_

### Classification against port-plan section 4.1

_(Does Koka's lowering strategy inform one of options 1-5, or point at a
new option? Even though Koka's surface is row-polymorphic, the runtime
representation must reduce to something implementable in a
non-row-polymorphic language.)_

### Scoped-operations handling (`local`, `catch`, and similar)

_(How are higher-order scoped operations expressed at the language level
and at the runtime level? Cite source for the mechanism used.)_

### Openness approach

_(How does Koka's implementation achieve extensibility to new effects once
the row has been lowered?)_

### Rust portability assessment

_(Could Koka's lowering strategy be implemented in Rust given the
`fp-library` Brand/Kind machinery? What would block it? What would it
cost? A short feasibility assessment is enough; defer full portability
analysis to a Stage 2 deep dive if one is triggered.)_

### Relevance to port-plan

_(Would findings here change any decision in `port-plan.md`? Which
sections? Answer "no change needed" explicitly if that is the
conclusion.)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1500 (excluding this template boilerplate)
