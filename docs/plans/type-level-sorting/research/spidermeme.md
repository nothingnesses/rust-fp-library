# spidermeme

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/spidermeme/`

## Purpose

Stage 1 research document: classify `spidermeme` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`spidermeme` is an experimental crate that uses `negative_impls` to
provide marker-trait inequality. The classification should determine
whether type inequality alone is enough to drive an ordering and what
the limits are.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview of the negative-impls trick used.)_

### Type-level sorting capability

_(Does inequality imply ordering? Can it be turned into a usable sort
without an additional total-order tag? Be explicit about the gap if
inequality alone is insufficient.)_

### Approach used (or enabled)

_(Which of the eight approaches in README does it implement? Likely
approach 7 (marker-trait inequality).)_

### Stable or nightly

_(Feature gates required: `negative_impls` is nightly. Confirm what
else is needed.)_

### Ergonomics and compile-time profile

_(How does a user opt in?)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.)_

### Applicability to coproduct row canonicalisation

_(Specifically: can marker-trait inequality canonicalise a coproduct
row? Probably no, because inequality is not a total order. Be specific
about the gap and any partial result it could give (e.g., proving two
specific orderings differ).)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
