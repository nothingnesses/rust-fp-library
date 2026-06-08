# typelist

**Status:** complete
**Last updated:** 2026-04-24
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

`typelist` provides a type-level singly-linked list implemented as recursively nested tuples: `(((), A), B)` for a two-element list. The primary API is a compile-time merge sort (`MergeSorted<T>` at line 171) that orders elements by their underlying numeric value. Users pass lists built via the `typenum_list![5, 3, -2, 1, ...]` macro (lines 343-354), which constructs nested tuples of `typenum::Const<N>` integers. The sort produces a canonicalised type where smaller numbers precede larger ones. Ancillary operations include concatenation, min/max extraction, push, and pop (lines 43-87). The codebase also provides runtime recursively-functional implementations of the same operations to document type-level invariants (lines 376-427).

### Type-level sorting capability

`typelist` sorts types _directly_ at the type level via trait resolution; it does not offer user-facing primitives for custom type ordering. It can sort _only_ `typenum::Const<N>` integers from the typenum crate. The sort is parameterised by `typenum`'s built-in `Cmp` trait, which performs numeric comparison. Users cannot sort arbitrary types or user-defined wrapper types; the mechanism is tightly bound to typenum's integer representation. This severely limits applicability to the coproduct canonicalisation problem, where one would need to sort by type identity rather than numeric value.

### Approach used

This codebase implements **candidate 1: Peano + typenum comparison**. The `MergeSort` trait (line 164) orchestrates a divide-and-conquer algorithm. Split operation (line 273) uses `Len<Array>: Add<P1>` and division by `P2` (two) to find the midpoint, relying on typenum's `Peano` number representation. The merge step (line 215) delegates to `LeftLast: Cmp<RightLast> + IsGreaterPrivate<RightLast, Compare<LeftLast, RightLast>>` (line 232), where `Compare<L, R>` yields `Gr` or `Le` for conditional element swapping via the `CmpSwap` trait (line 240). No other candidate (proc-macro textual canonicalisation, hash-based tags, const parameters, specialization, `TypeId`, marker traits, or const generics) is employed.

### Stable or nightly

This codebase compiles on stable Rust. `Cargo.toml` declares no feature gates (`feature(...)`) and depends only on public crates: `typenum = "1.15.0"` and `typenum_alias` from a public Git repository. MSRV is not explicitly documented, but the code uses standard trait machinery available in Rust 2021 edition. The implementation avoids nightly-only features like `adt_const_params`, `specialization`, or const-generic comparison.

### Ergonomics and compile-time profile

Invocation is declarative: write `type A = MergeSorted<typenum_list![5, 3, -2, 1, 2, 1, 2, 3, 4]>;` and let type inference finish the sort. The `typenum_list!` macro (lines 343-354) reverses input via `apply_args_reverse!` to match the right-associative tuple nesting. No manual trait impls per element are required; sorting is automatic once the list is constructed. Compile-time cost is not documented, but the algorithm is O(n log n) at runtime and likely similar at type resolution time. Error messages inherit typenum's style (lines 10-13 note shorter compiler errors than `typenum::TArr`). Debugging sorted types requires deliberately failing to compile (lines 36-40) to see the resolved structure in compiler output.

### Production status

`typelist` is experimental and undocumented outside its `README.md`. The crate version is 0.1.0 (Cargo.toml, line 2). No crates.io entry is visible; the codebase lives in a local GitHub-like clone. The Git history is not inspected here, but the code shows active iteration (e.g., comments documenting intentional compile failures). The project's primary purpose is as a dependency for `typeunits`, a composite-units library; `typelist` is not a standalone, production-ready library yet.

### Applicability to coproduct row canonicalisation

**Not applicable.** `typelist` cannot make `Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B, Coproduct<A, Void>>` resolve to the same type. The fundamental gap is that it sorts only `typenum::Const<N>` integers by numeric value, not arbitrary types by identity. To sort coproduct variants by identity, one would need a type-level comparison function that assigns stable, ordered tags to types like `A` and `B` independent of numeric constants. `typelist` provides no such mechanism. One would have to either (1) manually assign each variant a unique numeric constant and rewrite the coproduct as a list of `Const<N>` values (losing type safety), or (2) extend `typelist` with a custom comparison trait and canonicalisation layer for coproduct rows (outside the scope of this codebase). The Peano + typenum comparison approach is sound for numeric ordering but does not generalise to type identity.

### References

- Merge sort trait: `/home/jessea/Documents/projects/type-level/typelist/src/lib.rs`, lines 164-213
- Type comparison using `Cmp` and `IsGreaterPrivate`: `/home/jessea/Documents/projects/type-level/typelist/src/lib.rs`, lines 232, 240-258
- `typenum_list!` macro and reversal mechanism: `/home/jessea/Documents/projects/type-level/typelist/src/lib.rs`, lines 343-367
- Length calculation via `Peano`: `/home/jessea/Documents/projects/type-level/typelist/src/lib.rs`, lines 307-322
- Runtime merge and merge sort functions (documenting invariants): `/home/jessea/Documents/projects/type-level/typelist/src/lib.rs`, lines 260-271, 376-398

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion (handled separately)
- [x] Word count under ~1200 (excluding this template boilerplate)
