# type-uuid

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/type-uuid/`

## Purpose

Stage 1 research document: classify `type-uuid` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`type-uuid` assigns compile-time stable UUID constants to types. The
classification should determine whether its UUID-as-tag mechanism is
type-level (drivable by trait resolution) or only runtime (drivable by
matching at runtime).

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview: how UUIDs are assigned, what the user-facing
trait is.)_

### Type-level sorting capability

_(Is the UUID a type-level constant (`const UUID: u128 = ...` style),
or only runtime? Can it be lifted into a typenum integer or const
generic for trait dispatch?)_

### Approach used (or enabled)

_(Which of the eight approaches in README does it implement? Likely
approach 3 (hash-based) or approach 6 (TypeId-equivalent runtime).)_

### Stable or nightly

_(Feature gates required, MSRV if known.)_

### Ergonomics and compile-time profile

_(How does a user assign a UUID? Derive macro? Per-type attribute?
What are the documented costs?)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.)_

### Applicability to coproduct row canonicalisation

_(Specifically: could type-uuid's UUIDs drive a type-level sort? Can a
user-supplied UUID be lifted into a typenum tag at compile time and
fed to a sort routine? What is the gap?)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
