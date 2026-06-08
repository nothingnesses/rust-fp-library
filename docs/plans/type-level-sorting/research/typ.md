# typ

**Status:** complete
**Last updated:** 2026-04-24
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

`typ` is a procedural macro DSL that allows writing type operators
(functions on types) using Rust-like syntax. Users define functions
with generic type parameters and trait bounds, and the macro expands
them to trait definitions and trait implementations. The DSL integrates
first-class support for `typenum` integer types; operators like `+`,
`-`, `*`, `/`, `%`, and comparisons (`<`, `>`, `<=`, `>=`, `==`, `!=`)
are translated to corresponding `typenum` trait invocations. Control
flow (if/else, match, recursion) and list manipulation (Cons-based
structures) are also supported. Example: a binary GCD function at
`tests/macro/recursion.rs:8-41` demonstrates recursive type operators
with comparisons and arithmetic on typenum `Unsigned` types.

### Type-level sorting capability

No built-in sort macro or function exists in the DSL. However, the DSL
is Turing-complete for type-level computation: it supports recursion,
pattern matching on ADTs, comparisons, and conditional branching.
Users could theoretically write a quicksort or merge-sort in the DSL,
but no canonical example is provided in the codebase. The list append
example at `tests/macro/match_.rs:60-72` shows the pattern: recursive
type operators that process nested structures. A sort would require a
similar approach: match on list structure, recursively compare and
partition elements, then reconstruct. This is expressible but not a
turn-key primitive.

### Approach used (or enabled)

Approach 2 (proc-macro). The macro generates trait dispatch chains:
each typ function becomes a trait (e.g., `trait BinaryGcd<ARG_0, ARG_1>`)
with an associated `Output` type. The macro emits the trait definition
and a single blanket impl that computes the output (`src/trans/fn_.rs:228-297`).
Recursion and control flow are translated to cascading trait projections.
This is not a sorting primitive itself but enables the user to write
sorting algorithms as type-level code that compiles to trait dispatch.

### Stable or nightly

Nightly Rust required. The codebase uses `#![feature(hash_set_entry)]`
(`src/lib.rs:5`), which is unstable. Edition 2018. This blocks use on
stable Rust and limits adoption to projects willing to pin a nightly
toolchain.

### Ergonomics and compile-time profile

Users write functions inside a `typ! { ... }` block with Rust-like syntax.
The DSL feels natural to Rustaceans but is verbose: a sort would require
explicit recursion, pattern matching, and variable binding. Generated
code is a trait and a single blanket impl (`src/trans/fn_.rs:257-296`).
Compile-time cost is proportional to recursion depth and trait resolution.
The macro generates an inner module with the trait and impl, exposing a
type alias (`MyFunctionOp<Args...>`) for public use. No compile-time
introspection or debugging aids are evident.

### Production status

Likely abandoned. Last commit: July 19, 2021 (merge of a single PR).
Repository shows minimal activity and no recent development. Crate
version 0.1.1 on crates.io. No documentation beyond README and inline
examples. The author also created `rust-type-freak` (a more mature
library); `typ` appears to be a redesigned but unmaintained predecessor.

### Applicability to coproduct row canonicalisation

Could a user write a sort over a coproduct row using `typ`? Yes, in
principle. The DSL supports recursive type functions, pattern matching
on nested ADTs, and comparisons. However, it is not ergonomic. A user
would manually write a recursive sorter (e.g., quicksort or merge-sort)
operating on a `Coproduct<A, Coproduct<B, Void>>` structure, extracting
each variant via pattern matching, comparing them with hand-written
comparison logic (or a separate comparison type function), and
reconstructing the sorted coproduct. The result would be a type alias
like `SortCoproductOp<MyCoproduct>` that expands to a sorted nested
coproduct type. This is cumbersome and not recommended: the DSL lacks
automation for extraction, comparison, and permutation generation. In
practice, approach 1 (Peano arithmetic + typenum) or approach 8 (const
generics + const fn with a proc-macro wrapper) would be less tedious.

### References

- Cargo.toml: 0.1.1, edition 2018, proc-macro crate
- src/lib.rs:5: nightly feature gate
- src/trans/fn\_.rs:228-297: trait and impl generation
- src/trans/binop.rs:1-50: binary operator translation (comparisons to typenum traits)
- tests/macro/recursion.rs:8-41: recursive GCD example with comparison operator
- tests/macro/match\_.rs:60-72: list append with pattern matching
- GitHub history: last commit 2021-07-19

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
