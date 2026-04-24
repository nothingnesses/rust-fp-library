# static-assertions-rs

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/type-level/static-assertions-rs/`

## Purpose

Stage 1 research document: classify `static-assertions-rs` against the
eight type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`static-assertions-rs` provides compile-time-evaluated assertions over
const expressions and trait bounds. The classification should determine
whether any of its primitives could be repurposed for type-level
ordering.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

_static-assertions-rs_ is a compile-time assertion library providing macros to validate assumptions about constants, types, and traits. Core macros include `const_assert!`, `const_assert_eq!`, `const_assert_ne!` for const-context assertions (src/const_assert.rs), `assert_type_eq_all!` and `assert_type_ne_all!` for type equality checks (src/assert_type.rs), and `assert_impl_all!`, `assert_impl_any!`, `assert_not_impl_all!` for trait bound validation (src/assert_impl.rs). Additional macros check alignment, size, field existence, object safety, and trait relationships. All checks run at compile-time with zero runtime cost. The library is actively maintained (Cargo.toml declares "actively-maintained" status).

### Type-level sorting capability

_Not relevant._ The codebase provides type equality and trait bound testing only. `assert_type_eq_all!` (src/assert*type.rs, lines 47-67) uses a trait-based mechanism to check if types unify under the type checker, but provides no ordering relation and no mechanism to compare or reorder types. It asserts \_identity* of two types, not relative position. No functionality supports sorting, canonicalizing, or reordering type-level collections. The const-evaluation machinery in `const_assert!` evaluates boolean expressions at compile-time but operates on runtime values (e.g., `const DATA: &str`), not type-level computations. No generic const types, type families, or rewriting rules are present.

### Approach used (or enabled)

_Not applicable._ This codebase does not enable any of the eight approaches enumerated in the sorting research README. It provides compile-time assertion infrastructure only; it does not manipulate, reorder, or canonicalize type-level data structures.

### Stable or nightly

_Stable Rust (MSRV 1.37+)._ The library requires no nightly features for core functionality. Cargo.toml declares a `nightly` feature flag (line 24) with no associated cfg requirements, suggesting support for nightly-gated assertions if future versions add them. The crate works on stable because const-in-trait mechanisms it relies on (const functions in trait impls) were stabilized in Rust 1.37.

### Ergonomics and compile-time profile

_Macros are invoked declaratively._ Example: `assert_type_eq_all!(T, U, V)` expands to const functions that instantiate trait bounds at compile-time, forcing the type checker to unify types (src/assert*type.rs). Macros produce no code in the binary; they consume minimal compile-time overhead. Users typically invoke them at the module scope behind `const *` to create unnamed constants (README.md, lines 140-149), allowing global use without unique labels. Failure is a compile error with clear message pointing to the assertion.

### Production status

_Actively maintained._ The crate is published on crates.io as version 1.1.0 with an "actively-maintained" badge in Cargo.toml. Last significant commit was 2020-02-11 (badge update); the library is mature and stable with no recent activity needed because functionality is complete and well-tested. Documentation on docs.rs reflects the stable API.

### Applicability to coproduct row canonicalisation

_None._ The gap is fundamental: `static-assertions-rs` provides _predicate checking_ at compile-time (does type A equal type B? does T implement Trait?), while type-level sorting requires _term rewriting_ or _constraint solving_. Coproduct row canonicalization requires a system that can (1) pattern-match on type structure, (2) extract type arguments, (3) compare and reorder them, and (4) reconstruct the type. This crate has no reflection capability, no term rewriting, and no ordering semantics for types beyond equality. Its use case is validation, not transformation. To canonicalize `Coproduct<A, Coproduct<B, Void>>`, you would need either generic associated types with recursive type families, or a proc-macro system that manipulates the AST itself; static-assertions offers neither.

### References

- src/const_assert.rs: const-evaluation assertion macros
- src/assert_type.rs: type equality checks (lines 47-101)
- src/assert_impl.rs: trait bound validation
- Cargo.toml: version 1.1.0, feature flags, maintenance status
- README.md: use cases (lines 69-134), const context explanation (lines 140-149)
- GitHub (nvzqz/static-assertions-rs): last commit 2020-02-11

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
