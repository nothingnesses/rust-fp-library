# typenum

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/typenum/`

## Purpose

Stage 1 research document: classify `typenum` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`typenum` is the foundational type-level integer crate in the Rust
ecosystem. The classification should focus on the primitives it
provides for ordering / comparison and whether they scale to driving a
type-level sort over arbitrary types.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview: what is the user-facing API, what types does
it expose, what operations.)_

### Type-level sorting capability

_(Does it sort types directly? Does it provide primitives only? What
ordering / comparison traits exist, and what types do they apply to:
only its own integers, or arbitrary types?)_

### Approach used (or enabled)

_(Which of the eight approaches in README does it enable? Cite the
relevant trait, type, or macro.)_

### Stable or nightly

_(Feature gates required, MSRV if known.)_

### Ergonomics and compile-time profile

_(How does a user invoke comparison or sorting? Manual trait impls per
type, or auto-derived? Documented compile-time costs? Error-message
quality?)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.
Crates.io download counts.)_

### Applicability to coproduct row canonicalisation

_(Specifically: can the primitives in this crate be used to sort a
`Coproduct<A, Coproduct<B, Void>>` if A and B carry typenum tags? What
would the user need to do per effect to make it work?)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
