# frunk

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/frunk/`

## Purpose

Stage 1 research document: classify `frunk` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`frunk` is the canonical HList / Coproduct crate in Rust and is named
in the effects port-plan as the Option 1 (Peano-indexed coproduct)
reference. The classification should focus on: does frunk itself sort
HLists or Coproducts? If not, what type-level operations does it
provide that could be composed with a sort?

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview of the crate's scope: HList, Coproduct, Generic, etc.)_

### Type-level sorting capability

_(Does frunk sort HList or Coproduct types? Does it provide ordering
primitives? Look for traits like `Plucker`, `Sculptor`,
`CoproductEmbedder`, `CoproductSubsetter`, and assess whether any of
them imply a canonical ordering.)_

### Approach used (or enabled)

_(Which of the eight approaches in README does it implement? Cite the
relevant trait, type, or macro.)_

### Stable or nightly

_(Feature gates required, MSRV if known.)_

### Ergonomics and compile-time profile

_(How does a user invoke the relevant operations? Manual or
macro-driven? Compile-time cost characteristics if documented.)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.
Crates.io download counts.)_

### Applicability to coproduct row canonicalisation

_(Specifically: can frunk make `Coproduct<A, Coproduct<B, Void>>` and
`Coproduct<B, Coproduct<A, Void>>` resolve to the same type? If not,
what is the gap? Note that the port-plan currently relies on
`CoproductSubsetter` for permutation-based mediation rather than
canonicalisation; characterise the difference.)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
