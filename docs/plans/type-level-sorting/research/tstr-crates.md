# tstr_crates

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/tstr_crates/`

## Purpose

Stage 1 research document: classify `tstr_crates` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`tstr_crates` provides type-level strings (`TStr<...>`) with const
comparison operators. The classification should determine whether the
comparison primitives are usable for ordering arbitrary types and
whether the crate provides a sort routine.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview of the type-level string encoding and the
operators provided.)_

### Type-level sorting capability

_(Does it sort types directly? What ordering primitives does it provide
on TStr? Could those drive a sort over a heterogeneous coproduct?)_

### Approach used (or enabled)

_(Which of the eight approaches in README does it implement? Likely
approach 4 (adt_const_params with strings) or a stable workaround.)_

### Stable or nightly

_(Feature gates required, MSRV if known. tstr_crates has historically
had both stable and nightly variants; confirm.)_

### Ergonomics and compile-time profile

_(How does a user construct a TStr? Macro? const generic? What are
documented compile-time costs?)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.)_

### Applicability to coproduct row canonicalisation

_(Specifically: could TStr-based ordering be used to sort a coproduct
row whose effects each have a type-level string name? What would the
integration look like?)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
