# spidermeme

**Status:** complete
**Last updated:** 2026-04-24
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

spidermeme provides two marker traits: `SameTypeAs<T>` and `NotSameTypeAs<T>`. The inequality mechanism uses nightly-only `negative_impls` and `auto_traits` to construct a sealed proof that two types are distinct. The core trick (lib.rs:8-27) wraps both compared types in a private `TypePair<A, B>` struct, then defines an auto-trait `DifferentTypes` with a negative impl `impl<T> !DifferentTypes for TypePair<T, T>`. This negative impl prevents `DifferentTypes` from being implemented when both type parameters are identical. The public `NotSameTypeAs<T1>` trait is then implemented (lib.rs:27) only for pairs where `TypePair<T1, T2>: DifferentTypes` holds. Because auto-traits propagate negatively through compound types, wrapping in `TypePair` ensures that `((i32, i32), (f64, f64))` still proves its inner elements differ (lib.rs:40-42 test confirms this).

### Type-level sorting capability

Inequality is not a total order, so it cannot directly drive a sort. spidermeme proves only that X != Y; it provides no mechanism to determine whether X < Y or Y < X. To canonicalise `Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B, Coproduct<A, Void>>` into the same type, a sorting algorithm must know not just that A != B, but also which should come first. spidermeme offers no total-order tag, no numeric comparison, and no way to make that decision. The gap is fundamental: inequality is a symmetric relation (if A != B then B != A), while a sort order is asymmetric. A user would need a secondary mechanism such as typenum's numeric comparison, const generics, or proc-macro-driven tags to lift inequality into an ordering. spidermeme alone provides a building block, not a complete solution.

### Approach used (or enabled)

This is approach 7 (marker-trait inequality via orphan rules) from the eight-approach catalogue. It does not implement sorting; it provides a reusable inequality marker that _could_ be combined with another approach to enable sorting.

### Stable or nightly

spidermeme requires three unstable features: `negative_impls`, `auto_traits`, and `extended_key_value_attributes` (lib.rs:1-2, README.markdown:150-156). All three are nightly-only and subject to stabilization timelines not under user control. It is not usable on stable Rust. The crate uses Rust edition 2018 (Cargo.toml:5).

### Ergonomics and compile-time profile

Users opt in via `where T1: NotSameTypeAs<T2>` bounds in trait impls or struct definitions (README.markdown:70-93 example). The implementation requires only trait resolution; no recursive elaboration or specialization applies, so compile time is proportional to the number of distinct type pairs checked. However, the sealed design (private module, lib.rs:8-27) means downstream crates cannot define their own negative impls, limiting composition with other type-level systems. Tests use `static_assertions::assert_impl_all!` and `assert_not_impl_any!` (lib.rs:35-43).

### Production status

spidermeme is a small, maintained experimental library. The git history shows a single recent commit (6739d12, "Add codecov token"), suggesting active CI and polish but no active feature development. The crate is published on crates.io with CI badges and coverage tracking. It is suitable for small-scale experiments but carries nightly-feature risk; any stabilization of the underlying features could shift correctness or semantics.

### Applicability to coproduct row canonicalisation

spidermeme alone cannot canonicalise a coproduct row. To reorder `Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B, Coproduct<A, Void>>` to the same canonical form, the compiler must decide the relative ordering of A and B. spidermeme can prove A != B, but not whether A should come before or after B in the canonical form. The precise gap: inequality is a symmetric relation (it makes no distinction between direction), while a sort requires an asymmetric total order. To use spidermeme as part of a sorting system, one would need to pair it with a second mechanism that defines a canonical ordering (e.g., a numeric tag from typenum, a string comparison via proc-macro, or const generics). By itself, spidermeme does not narrow the search space of possible reorderings and thus does not solve the canonicalisation problem.

### References

- `/home/jessea/Documents/projects/type-level/spidermeme/src/lib.rs` lines 1-2: feature gate declarations.
- `/home/jessea/Documents/projects/type-level/spidermeme/src/lib.rs` lines 8-27: `TypePair` and `DifferentTypes` sealed trait design.
- `/home/jessea/Documents/projects/type-level/spidermeme/src/lib.rs` lines 40-42: test confirming nested-pair inequality.
- `/home/jessea/Documents/projects/type-level/spidermeme/src/lib.rs` line 181: blanket `NotSameTypeAs` impl.
- `/home/jessea/Documents/projects/type-level/spidermeme/README.markdown` lines 122-140: explanation of negative-impl and auto-trait mechanics.
- `/home/jessea/Documents/projects/type-level/spidermeme/README.markdown` lines 150-156: unstable features list.
- `/home/jessea/Documents/projects/type-level/spidermeme/Cargo.toml` line 5: edition 2018.

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] Word count under ~1200 (excluding this template boilerplate): approximately 650 words
