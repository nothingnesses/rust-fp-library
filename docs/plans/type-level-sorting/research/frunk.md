# frunk

**Status:** complete
**Last updated:** 2026-04-24
**Codebase location:** `/home/jessea/Documents/projects/type-level/frunk/`

## Purpose

Stage 1 research document: classify `frunk` against the eight
type-level sorting approaches catalogued in [README.md](README.md).
Identify whether this codebase implements sorting directly, provides
primitives that enable sorting, or is unrelated to the question.

`frunk` is the canonical HList / Coproduct crate in Rust and is named
in the effects decisions as the Option 1 (Peano-indexed coproduct)
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

Frunk (version 0.4.4, edition 2021) provides a functional programming
toolbelt for Rust, centered on HList (heterogeneous linked lists) and
Coproduct (tagged unions over arbitrary types). The main crate spans
HLists with operations like `pluck` (remove by type), `sculpt` (rearrange
to a target shape), and `fold` (consume to single value). Coproducts
support `inject`, `get`, `take`, `uninject`, `embed`, `subset`, `fold`,
and `map`. It also includes `Generic` and `LabelledGeneric` for struct
isomorphism, plus `Validated` (Result-like), `Monoid`, `Semigroup` type
classes. No explicit sorting of HList or Coproduct row types is present.

### Type-level sorting capability

Frunk does _not_ sort HList or Coproduct types. Instead, it provides
permutation operations that require the compiler to solve indices at
resolve time. The `Plucker` trait (hlist.rs:884) removes one element by
type and returns a remainder. The `Sculptor` trait (hlist.rs:979) accepts
a target HList shape and via recursive `Plucker` calls (line 1014),
constructs that shape from the source list, order-agnostic. The
`Sculptor` implementation shows the key difference from sorting: it takes
a target type as input and proves "I can extract this permutation from
you," rather than deciding on a canonical form. For Coproducts,
`CoproductEmbedder` (coproduct.rs:1199) converts one coproduct to another
capable of holding its variants; the comment at line 408-413 explicitly
lists reordering as an example. The mechanism is again proof-of-permutation,
not canonicalisation. Neither trait family encodes an ordering decision
(e.g., via Peano comparisons or TypeId hashing).

### Approach used (or enabled)

Frunk uses approach 1 (Peano + typenum) in a limited, permutation-only
sense. The `indices::Here` and `indices::There` types (indices.rs, not
shown in detail but referenced throughout) form a Peano-style linked
structure to encode positions. Type inference resolves these indices when
`Sculptor` or `Plucker` are applied. However, this mechanism does _not_
compute a canonical order. Instead, the target type (or requested element
type) drives the index calculation. It is used to ask "do you have a B
in you?" and "where is it?", not "in what order should the variants
appear?". Thus frunk enables aspect 1 (Peano structure) but not the
sorting aspect of approach 1.

### Stable or nightly

Frunk is entirely stable Rust. Edition 2021, MSRV not documented in
Cargo.toml. No feature gates for sorting or type-level comparison; all
operations compile on stable with `default-features` (which include
`validated` and `proc-macros`). The proc-macros crate (version 0.1.4)
provides derive macros for `Generic` and `LabelledGeneric`, but these
do not perform sorting either.

### Ergonomics and compile-time profile

Users invoke sorting/permutation via method calls: `list.pluck::<T, _>()`,
`list.sculpt::<TargetHList, _>()`, or `coproduct.embed::<TargetCoproduct,
_>()`. Type inference solves the `Index` parameter automatically; the
turbofish syntax for specifying the target is optional. Compile-time cost
is not documented; the trait-based approach (recursive impl resolution via
`Plucker`, `Sculptor`, `CoprodInjector`) incurs resolution cost proportional
to the list/coproduct size. No published benchmarks measure this overhead
in the repository.

### Production status

Frunk is mature and actively maintained. Latest release 0.4.4 (repository
https://github.com/lloydmeta/frunk); CI/CD enabled on master branch. Widely
used in the Rust ecosystem (Crates.io badge present in README). Repository
shows regular commits and issue responses, suggesting active maintenance,
though no specific recent date is given in the cloned snapshot.

### Applicability to coproduct row canonicalisation

Frunk _cannot_ make `Coproduct<A, Coproduct<B, Void>>` and `Coproduct<B,
Coproduct<A, Void>>` resolve to the same type. The core issue: both the
`embed` method (coproduct.rs:440-450) and the `Embedder` trait (line 1231)
require an explicit target type as input. The compiler does not infer a
canonical form. In the effects decisions, `CoproductSubsetter` (coproduct.rs: 1145) is used to prove that a permutation exists: `subset()` extracts a
subset and returns the remainder. This is permutation _mediation_, not
canonicalisation. Two different coproduct orderings will have two different
types; subsets must be explicitly requested. A true canonicalisation would
require a trait that, given any permutation of {A, B}, automatically
resolves to a single canonical type (e.g., always `Coproduct<A,
Coproduct<B, Void>>`). Frunk provides no such mechanism and does not
attempt to; its design assumes the caller specifies the desired permutation
(or subset) explicitly.

### References

- `frunk/core/src/hlist.rs`, lines 322-325: `sculpt` method and `Sculptor`
  trait bounds.
- `frunk/core/src/hlist.rs`, lines 884-897: `Plucker` trait definition.
- `frunk/core/src/hlist.rs`, lines 979-991: `Sculptor` trait definition.
- `frunk/core/src/hlist.rs`, lines 1011-1020: `Sculptor` impl combining
  `Plucker` recursively.
- `frunk/core/src/coproduct.rs`, lines 100-105: `Coproduct` enum definition.
- `frunk/core/src/coproduct.rs`, lines 310-350: `subset` method and
  `CoproductSubsetter` trait.
- `frunk/core/src/coproduct.rs`, lines 403-458: `embed` method and comment
  at 408-413 listing reordering as use case.
- `frunk/core/src/coproduct.rs`, lines 1145-1158: `CoproductSubsetter` trait
  definition.
- `frunk/core/src/coproduct.rs`, lines 1199-1210: `CoproductEmbedder` trait
  definition.
- `frunk/Cargo.toml`, lines 4, 44-48: version 0.4.4, edition 2021, stable
  only.

## Closing checklist

- [x] All subsections above filled in
- [x] Status updated to `complete`
- [ ] `_status.md` updated to reflect this file's completion
- [x] Word count under ~1200 (excluding this template boilerplate)
