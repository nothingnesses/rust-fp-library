# static-assertions-rs

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/static-assertions-rs/`

## Purpose

Stage 1 research document: classify `static-assertions-rs` against the
eight type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`static-assertions-rs` provides compile-time-evaluated assertions over
const expressions and trait bounds. The classification should determine
whether any of its primitives could be repurposed for type-level
ordering.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview of the macros it provides.)_

### Type-level sorting capability

_(Does it sort types directly? Does it provide primitives only?
Probably not relevant to sorting, but assess its const-evaluation
machinery for any sort-adjacent capability.)_

### Approach used (or enabled)

_(Which of the eight approaches in README, if any, does it enable?)_

### Stable or nightly

_(Feature gates required, MSRV if known.)_

### Ergonomics and compile-time profile

_(How does a user invoke the relevant macros?)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.)_

### Applicability to coproduct row canonicalisation

_(Specifically: is there any path from this crate's primitives to
canonicalising a coproduct row? Most likely: no. State the gap
explicitly.)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
