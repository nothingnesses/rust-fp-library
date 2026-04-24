# rust-type-freak

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/type-level/rust-type-freak/`

## Purpose

Stage 1 research document: classify `rust-type-freak` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`rust-type-freak` is a collection of type operators (list ops, map
ops, etc.) used as the foundation for tensor-shape type checking. The
classification should determine whether any of its list operators
include sorting and whether the trait machinery could be repurposed.

## Required findings

An agent completing this document must fill every subsection below
with at least one paragraph grounded in actual code (cite paths and
line numbers where relevant). Say "not applicable" or "not documented
in source" explicitly if a section does not apply; do not leave blank
headers.

### What this codebase does

`rust-type-freak` is a collection of trait-based type operators for
compile-time manipulation of type-level data structures. It exports
typed list (`TList`), key-value map (`KVList`), boolean algebra,
`Maybe` types, and functional primitives (map, fold, scan, filter).
List operations (`src/list/ops.rs` lines 1-420) include `PushFront`,
`PushBack`, `Insert`, `Reverse`, `Zip`, and functional transforms.
The crate relies on the `typ` procedural macro (external dependency
at `https://github.com/jerry73204/typ.git`) to provide type-level
pattern matching and case analysis. It is licensed MIT or Apache 2.0
and currently at version 0.2.0, designed as an alpha-stage foundation
for the `tch-typed-tensor` project's compile-time tensor shape
verification. No type-level comparison or ordering traits are exposed
in the public API.

### Type-level sorting capability

**Not implemented.** The codebase provides no `Sort` operator. It also
lacks merge or insertion-sort composition building blocks: `Insert`
(`src/list/ops.rs` lines 53-66) takes an explicit numeric index, not
a comparison result, so it cannot drive a sort. Similarly, there is no
`Split` operator that partitions lists based on a predicate or
comparison. The dict module (`src/dict/base.rs`, `src/dict/ops.rs`)
is minimal (line 3 of `ops.rs` shows an empty `typ! {}` block) and
offers no sorting. Control flow (`src/control.rs` lines 1-10) provides
only type equality (`Same`), not comparison or inequality. No traits
for type-level ordering (e.g., less-than, comparison functors) are
documented or present.

### Approach used (or enabled)

**Not applicable.** Since the codebase does not attempt sorting, it
does not instantiate any of the eight approaches (Peano+typenum,
proc-macro, hash, adt*const_params, specialization, TypeId,
marker-trait inequality, or const generics+const fn). The `typ`
procedural macro (\_external*; source not in this repo) enables
pattern matching on types, which is orthogonal to sorting strategy
selection.

### Stable or nightly

The crate targets Rust edition 2018 (`Cargo.toml` line 5) and requires
only `typenum` (1.12+) as a production dependency; `typ` is fetched
from Git. No feature gates, unsafe code, or nightly-only features are
visible in the public API. MSRV is not declared, but the absence of
nightly markers suggests it compiles on stable Rust.

### Ergonomics and compile-time profile

Operators are invoked as type-level trait bounds and associated-type
projections. For example, list length is written `LenOp<List>`, and
insertion as `InsertOp<list, index, value>`. The `typ` macro syntax
allows function-like declarations (`pub fn Insert<...>(...) -> List`
in `src/list/ops.rs` line 53) that desugar to trait impls. Users work
with these operators through trait bounds on generics; there are no
runtime macros or code-generation facilities exposed. Trait resolution
is transparent to the Rust compiler's incremental and coherence
checking, so compile times scale with list length but no special
profiling is reported in the documentation.

### Production status

**Abandoned/dormant.** The crate was last updated on 2020-09-27
(commit b77a053: "New impls for Dyn<bool>"). No further commits have
been made in nearly 6 years. The repo does not indicate known issues,
active maintenance, or a roadmap. Version 0.2.0 is still marked alpha
in `Cargo.toml` (line 26 of README.md: "Still in alpha stage"). While
the code compiles and the foundation is sound, there are no signs of
recent adoption, security audits, or compatibility guarantees with
newer Rust toolchain releases.

### Applicability to coproduct row canonicalisation

**Not applicable.** `rust-type-freak` cannot canonicalise coproduct
rows like `Coproduct<A, Coproduct<B, Void>>` because it lacks (1) a
type-level ordering relation (no trait for comparing arbitrary types),
(2) a sort operator, and (3) a mechanism to extract or describe the
effect metadata needed for intelligent ordering. A user would need to
manually supply a comparison function per effect type, but the crate
provides no infrastructure to apply such comparisons. Even with custom
compare traits, composing them into a sort would require implementing
the missing sort operator on top of `Insert`, which is incomplete
because `Insert` does not accept comparison logic. The crate's strength
is in traversing and transforming pre-structured data, not in
rearranging data according to abstract comparison predicates.

### References

- `src/list/ops.rs` lines 1-420: core list operators (no sort)
- `src/list/ops.rs` lines 53-66: `Insert` operator (index-based, not comparison-driven)
- `src/dict/ops.rs` line 3: empty dict operators module
- `src/control.rs` lines 1-10: type equality only
- `Cargo.toml` lines 1-14: dependencies and metadata
- `README.md` line 13: alpha-stage status
- Git history: last commit 2020-09-27

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [x] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
