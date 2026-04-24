# rust-type-freak

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/rust-type-freak/`

## Purpose

Stage 1 research document: classify `rust-type-freak` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`rust-type-freak` is a collection of type operators (list ops, map
ops, etc.) used as the foundation for tensor-shape type checking. The
classification should determine whether any of its list operators
include sorting and whether the trait machinery could be repurposed.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview: which operators it provides, what its design
is, what it is used for.)_

### Type-level sorting capability

_(Does it provide a sort operator? Does it provide insert / merge /
split operators that could be composed into a sort? Cite specific
files.)_

### Approach used (or enabled)

_(Which of the eight approaches in README does it implement?)_

### Stable or nightly

_(Feature gates required, MSRV if known.)_

### Ergonomics and compile-time profile

_(How does a user invoke the operators? Trait projections? Macros?)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.)_

### Applicability to coproduct row canonicalisation

_(Specifically: can rust-type-freak's operators sort a coproduct row?
What would the user need to provide per effect (a comparison
function?), and what would the resulting row look like?)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
