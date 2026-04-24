# typelist

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/typelist/`

## Purpose

Stage 1 research document: classify `typelist` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

The discovery survey flagged this codebase as a direct merge-sort
implementation at the type level; the classification should focus on
mechanism and applicability to coproduct row canonicalisation.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview: what is the user-facing API, what does it
sort, what does it produce.)_

### Type-level sorting capability

_(Does it sort types directly, or provide primitives only? What can it
sort: only typenum integers, arbitrary types with a tag trait,
something else?)_

### Approach used

_(Which of the eight approaches in README does it implement? Cite the
relevant trait, type, or macro.)_

### Stable or nightly

_(Feature gates required, MSRV if known.)_

### Ergonomics and compile-time profile

_(How does a user invoke the sort? Manual trait impls per type, or
auto-derived? Documented compile-time costs? Error-message quality?)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.
Crates.io presence.)_

### Applicability to coproduct row canonicalisation

_(Specifically: can this make `Coproduct<A, Coproduct<B, Void>>` and
`Coproduct<B, Coproduct<A, Void>>` resolve to the same type? If not,
what is the gap? If yes, what would the user need to do per effect to
make it work?)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
