# effekt

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/effects/effekt/`

## Purpose

Stage 1 research document: classify `effekt` against the five effect-row
encodings catalogued in [../port-plan.md](../port-plan.md) section 4.1.
Identify whether this codebase (which is a language, not a library)
represents a genuinely novel encoding worth deeper investigation in Stage 2. The relevant question is whether effekt's capability-based approach
could inform a Rust encoding rather than whether effekt's implementation
is itself portable.

Scope is deliberately narrow. This is a skim, not a thorough read. For
deep investigation of any novelty surfaced here, create a
`deep-dive-<topic>.md` file in this directory.

## Required findings

An agent completing this document must fill every subsection below with at
least one paragraph grounded in actual code (cite paths and line numbers
where relevant). Say "not applicable" or "not documented in source"
explicitly if a section does not apply; do not leave blank headers.

### Core substrate

_(What is the fundamental encoding? Free monad + coproduct? Evidence
passing? Capability? Trait/typeclass dispatch? Something else? For effekt
specifically: how does its capability system work at the type-system and
runtime level? Point at documentation, language-manual sections, or
compiler source as appropriate.)_

### Distinctive contribution relative to baseline

_(What does effekt do differently from a standard `Free + Coproduct + Member` baseline?)_

### Classification against port-plan section 4.1

_(Is this a variant of options 1-5 in the plan's encoding list, or
genuinely novel? If a variant, which one and how close? If novel, briefly
sketch the difference.)_

### Scoped-operations handling (`local`, `catch`, and similar)

_(How are higher-order scoped operations expressed, if at all? Cite source
for the mechanism used.)_

### Openness approach

_(How does this codebase achieve extensibility to new effects? If it does
not, note that explicitly.)_

### Rust portability assessment

_(Specific to languages rather than libraries: could effekt's approach be
simulated in Rust given the `fp-library` Brand/Kind machinery and Rust's
existing lexical borrow checker? What would block it? What would it cost?
A short feasibility assessment is enough; defer full portability analysis
to a Stage 2 deep dive if one is triggered.)_

### Relevance to port-plan

_(Would findings here change any decision in `port-plan.md`? Which
sections? Answer "no change needed" explicitly if that is the
conclusion.)_

### References

_(File paths and line numbers into the codebase or its documentation that
support claims above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1500 (excluding this template boilerplate)
