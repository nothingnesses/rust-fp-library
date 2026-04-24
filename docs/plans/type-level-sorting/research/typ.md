# typ

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/typ/`

## Purpose

Stage 1 research document: classify `typ` against the eight type-level
sorting approaches catalogued in [README.md](README.md). Identify
whether this codebase implements sorting directly, provides primitives
that enable sorting, or is unrelated to the question.

`typ` is an experimental type-level DSL with type operators in
Rust-like syntax. The classification should determine whether its DSL
includes or could express sorting.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview: what the DSL allows, how it relates to
typenum.)_

### Type-level sorting capability

_(Does the DSL include a sort macro? Can a user write a sort in the
DSL? Cite specific files / examples.)_

### Approach used (or enabled)

_(Which of the eight approaches in README does it implement?)_

### Stable or nightly

_(Feature gates required, MSRV if known.)_

### Ergonomics and compile-time profile

_(How does a user write code in the DSL? What is generated?)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.)_

### Applicability to coproduct row canonicalisation

_(Specifically: could `typ`'s DSL be used to write a sort over a
coproduct row? What would the user write?)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
