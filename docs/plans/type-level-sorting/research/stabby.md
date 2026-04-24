# stabby

**Status:** pending
**Last updated:** _not yet started_
**Codebase location:** `/home/jessea/Documents/projects/type-level/stabby/`

## Purpose

Stage 1 research document: classify `stabby` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`stabby` is an ABI-stability crate that uses compile-time hashing of
type names for FFI safety. The classification should determine whether
its hash-based type identity could drive a type-level sort.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_(One-paragraph overview of the crate's purpose: ABI stability via
type hashing.)_

### Type-level sorting capability

_(Does it sort types directly? Does it provide a stable hash that could
drive ordering? What type does the hash live in (typenum integer,
const generic, runtime u64, etc.)?)_

### Approach used (or enabled)

_(Which of the eight approaches in README does it implement? Most
likely: approach 3 (hash-based type tags) and possibly approach 4
(adt_const_params).)_

### Stable or nightly

_(Feature gates required, MSRV if known. Stabby historically used
nightly features for its const-string handling; confirm current
status.)_

### Ergonomics and compile-time profile

_(How does a user opt a type into stabby's hashing? Derive macro,
manual impl, automatic for all types? Documented compile-time costs?)_

### Production status

_(Active / abandoned / experimental. Last commit date if easy to find.
Crates.io download counts.)_

### Applicability to coproduct row canonicalisation

_(Specifically: could stabby's type-hash machinery be used to sort a
coproduct row? What would the integration look like? What would the
user need to do per effect?)_

### References

_(File paths and line numbers into the codebase that support claims
above.)_

## Closing checklist

- [ ] All subsections above filled in
- [ ] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [ ] Word count under ~1200 (excluding this template boilerplate)
