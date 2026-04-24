# type-level-sort

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/type-level/type-level-sort/`

## Purpose

Stage 1 research document: classify `type-level-sort` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

This codebase is a direct implementation of type-level sorting in Rust
(per the discovery survey); the classification should focus on _how_ it
works and whether the technique would scale to coproduct row
canonicalisation.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

This codebase provides a complete type-level bubble sort implementation (file: `/home/jessea/Documents/projects/type-level/type-level-sort/src/main.rs`). The user-facing API is the `BubbleSort<Ls>` type alias (line 253), which takes a heterogeneous list `Ls` constructed via `Cons<Head, Tail>` (lines 30-33) and returns a sorted `Cons` list. The main assertion (lines 5-7) sorts `Cons<N3, Cons<N1, Cons<N2, Nil>>>` to `Cons<N1, Cons<N2, Cons<N3, Nil>>>`, proving it works end-to-end. The implementation uses trait-based dispatch and associated types to simulate recursion within the type system, producing a statically-verified sorted type.

### Type-level sorting capability

It sorts exclusively _Peano numbers_ encoded as type-level structures (`Zero`, `Succ<A>`; lines 19-23). The comparator `ComputeCompareNat<Rhs>` (lines 49-71) reduces two natural numbers to an `Equality` type (`EQ`, `LT`, `GT`; lines 37-45). There is no abstraction over user-defined types with custom orderings: the sort is hardcoded to integer Peano arithmetic. The list structure itself is generic (`Cons<V, C>` accepts any `V`; line 30), but comparison only works on `Nat`-typed heads; this is a strong-typed limitation.

### Approach used

This implements **Approach 1: Peano + typenum comparison**. The implementation uses Peano numbers (`Zero`, `Succ`) and a hand-crafted trait-based comparison system (`ComputeCompareNat`; lines 49-71) to drive a bubble sort kernel (`ComputeBubble` at lines 161-179, and `ComputeBubbleSort` at lines 211-251). The comparison is computed via trait resolution using associated types, not code generation or macros.

### Stable or nightly

Stable Rust 2021 edition only. The `Cargo.toml` (line 7) enables `-Z chalk=true` as a build flag, which is _chalk trace verbosity_ for compiler diagnostics; this is not a feature gate but a debug aid. No nightly features (`adt_const_params`, `specialization`, `const fn`, etc.) are required. The code compiles with `assert-type-eq = 0.1.0` (line 10), a stable-friendly type equality assertion macro.

### Ergonomics and compile-time profile

Users invoke the sort by constructing a `Cons` list of `Nat` types and calling `BubbleSort<...>` in a type annotation or assertion. No procedural macros or derive mechanics are used; every type is written explicitly. The list must be constructed in reverse textual order (manual nesting). Compile-time cost grows with list size and nesting depth due to trait resolution; the code does not document measured compilation time, though the deeply nested `where` clauses (lines 224-247) suggest significant constraint solving overhead. Error messages from type mismatch are standard Rust trait bound failures and will not guide users on sorting failures.

### Production status

This is a proof-of-concept / educational implementation (see README link to dev.to article). There is no Crates.io publication, and the codebase has no recent activity markers in the local Git history. It appears to be a standalone research artifact created to demonstrate type-level bubble sort feasibility. Not suitable for production use without significant hardening (e.g., error messages, performance tuning, documentation on applicability limits).

### Applicability to coproduct row canonicalisation

**Not directly applicable.** The fundamental gap: the sort requires `Nat`-typed list heads for comparison; it cannot sort arbitrary types like `Effect<A>`, `Effect<B>`, etc. To canonicalise `Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B, Coproduct<A, Void>>`, each effect type would need to be wrapped in a Peano tag (e.g., `type EffectA = N0; type EffectB = N1;`), sorted via `BubbleSort`, and then the resulting numeric type would need to be mapped back to effect types via a lookup table. This is cumbersome and defeats the ergonomic goal of transparent canonicalisation. The user would need to: (1) assign each effect a unique Peano number, (2) construct the input coproduct as a `Cons` list of those numbers, (3) sort, and (4) use a downstream type-level map to recover the original effect types. The technique is sound but lossy without additional machinery to preserve type identity through the sort.

### References

- Main implementation: `/home/jessea/Documents/projects/type-level/type-level-sort/src/main.rs`, lines 1-254
- `BubbleSort` type alias: line 253
- Peano structure: lines 19-23
- Comparator trait: lines 49-71
- Bubble pass kernel: lines 161-179
- Full sort trait: lines 211-251
- User-facing assertion: lines 5-7

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion (parent handles)
- [x] Word count under ~1200 (excluding this template boilerplate)
