# Stage 1 classification: aggregated findings

**Status:** complete
**Last updated:** 2026-04-25

## 1. What Stage 1 set out to answer

The originating question was whether type-level sorting in Rust is
feasible enough to inform the effects research's decisions section
4.1 "ordering mitigations" subsection (specifically workaround 2:
tag-based type-level sorting, currently rejected on speculative
complexity and compile-time grounds).

Stage 1 surveyed 16 codebases under
`/home/jessea/Documents/projects/type-level/` against ten candidate
approaches catalogued in [README.md](README.md). The original 11
codebases plus five added in a later expansion (tyrade, anymap,
fixed-type-id, typemap-meta, rust-typemap) cover both ordering-based
canonicalisation (approaches 1-8) and deduplication-based
canonicalisation (approaches 9 and 10) routes:

1. Peano + typenum comparison.
2. Proc-macro textual canonicalisation.
3. Hash-based type tags (runtime constant).
4. `feature(adt_const_params)` with string const parameters.
5. `feature(specialization)` / `min_specialization`.
6. `std::any::TypeId` runtime comparison.
7. Marker-trait inequality via orphan-rule tricks.
8. Const generics + `const fn` comparison.
9. Type-level hashing with type-level result.
10. Type-level hash-map / hash-set.

Each Stage 1 file answered three narrow questions: which approach
does this codebase implement, is it on stable or nightly Rust, and
could it canonicalise a coproduct row of arbitrary effect types?
This synthesis aggregates the verdicts, maps the design space, and
decides whether the decisions should be revised or any Stage 2 deep
dive is justified.

## 2. Classification table

Columns: codebase, primary approach, sort/dedup capability, stable
or nightly, applicability to coproduct row canonicalisation.

| Codebase             | Approach          | Sort or dedup capability                    | Stable / nightly          | Coproduct canonicalisation?                       |
| -------------------- | ----------------- | ------------------------------------------- | ------------------------- | ------------------------------------------------- |
| type-level-sort      | 1 (Peano)         | Bubble sort over Peano nats only            | Stable                    | No (numeric domain only).                         |
| typelist             | 1 (Peano)         | Merge sort over `typenum::Const<N>`         | Stable                    | No (numeric domain only).                         |
| typenum              | 1 (enabler)       | Comparison primitives (sealed)              | Stable, MSRV 1.41         | Partial (sealed, user writes engine).             |
| frunk                | 1 (permutation)   | Permutation proof; no canonical form        | Stable, ed 2021           | No (target type required).                        |
| static-assertions-rs | None              | Predicate checks only                       | Stable, MSRV 1.37         | No (validation, not transformation).              |
| stabby               | 3 (runtime u64)   | Hash via const-fn SHA256                    | Stable, MSRV 1.61         | No (`const ID: u64` not a type parameter).        |
| type-uuid            | 3 / 6             | `const UUID: [u8; 16]` per type             | Stable, ed 2018           | No (arrays disallowed as const generic).          |
| rust-type-freak      | None              | List ops; no sort, no Cmp                   | Stable, ed 2018           | No (no comparison; abandoned 2020).               |
| typ                  | 2 (DSL)           | Turing-complete; no shipped sort            | Nightly                   | Possible but verbose; abandoned 2021.             |
| tstr_crates          | 4 (chars)         | `tstr::cmp -> Ordering`; no sort            | Stable, MSRV 1.88         | Possible building block (most credible for sort). |
| spidermeme           | 7 (inequality)    | Inequality only; not a total order          | Nightly                   | No (symmetric relation cannot order).             |
| tyrade               | 2 (DSL)           | Recursive functions; no shipped sort        | Nightly (specialization)  | Possible but verbose; abandoned 2018.             |
| anymap               | 10 (runtime)      | Runtime `TypeId` dedup                      | Stable, MSRV 1.36         | No (runtime, not type-level).                     |
| rust-typemap         | 10 (runtime)      | Runtime `TypeId` dedup with `Key` trait     | Stable                    | No (runtime, abandoned 2017).                     |
| fixed-type-id        | 3 (runtime u64)   | Hash via rapidhash; version metadata        | Nightly (4 feature gates) | No (same gap as stabby/type-uuid).                |
| typemap-meta         | 10 (compile-time) | Compile-time trait dispatch on fixed schema | Stable, ed 2021           | No (fixed schema, not dynamic rows).              |

Three cross-cutting findings emerge from the table that no individual
file surfaced. First, **no surveyed crate solves coproduct
canonicalisation**, even after the expansion. Of 16 codebases, zero
ship a mechanism that would make `Coproduct<A, Coproduct<B, Void>>`
and `Coproduct<B, Coproduct<A, Void>>` resolve to the same compile-time
type. Second, **the stable-Rust path remains tstr_crates** (approach 4)
plus a custom proc-macro and recursive sort trait; the expansion did
not surface a more credible building block. Third, **stable Rust does
have type-level dispatch on type identity, but only for fixed schemas**:
typemap-meta proves that compile-time trait-based lookup over a
fixed set of types works cleanly on stable, just not for dynamic
row-typed effects.

## 3. Approach-by-approach summary

### Approach 1: Peano + typenum comparison

Four codebases cluster here. typenum supplies the comparison
primitive (`Cmp` returning `Greater` / `Less` / `Equal` marker
types, src/type_operators.rs:310-316), but the trait is sealed via
a `Sealed` private trait (src/lib.rs:149-173); only typenum's own
integer types implement it. type-level-sort and typelist write the
sort engine but are pinned to numeric domains: type-level-sort sorts
hand-rolled Peano nats, typelist sorts `typenum::Const<N>`. frunk's
Plucker / Sculptor use Peano-style indices but only for proof of
permutation, not canonicalisation. None of these can sort arbitrary
effect type identities; the sealed comparison plus numeric-only
sort engines mean the user must coordinate effect-to-typenum-tag
mappings manually, which is the original cost workaround 2 cited.

### Approach 2: Proc-macro textual canonicalisation

Two surveyed codebases now: `typ` and `tyrade`. Both are proc-macro
DSLs that compile type-level functions to trait-projection chains.
`typ` requires nightly (`hash_set_entry`); `tyrade` requires nightly
(`specialization`). Both are abandoned (typ in 2021, tyrade in 2018).
Both are Turing-complete in principle but ship no sort. tyrade is
more ergonomic (cleaner `fn` syntax, no need for typenum
boilerplate), but neither provides type-level comparison primitives
applicable to arbitrary user types; a user wanting to sort a
coproduct would need to write the comparison machinery outside the
DSL, defeating the DSL's ergonomic benefit. tyrade also has a
documented stack-overflow bug under specialization (tnum.rs:45-51),
which casts doubt on production reliability.

The effects research already covered macro-based canonicalisation
(corophage's `Effects![...]` macro, which does NOT sort; preserves
user order). Two abandoned DSL data points reinforce that a
production-grade implementation does not exist.

### Approach 3: Hash-based type tags (runtime constant)

Three codebases: stabby, type-uuid, fixed-type-id. All compute a
stable compile-time identifier per type, but all store the result
as a **runtime constant**, not a type parameter. stabby's `const
ID: u64` is computed via const-fn SHA256; type-uuid stores `const
UUID: [u8; 16]`; fixed-type-id uses rapidhash to produce
`const TYPE_ID: FixedId(u64)`. On stable Rust, none of these shapes
can be lifted into a const generic parameter to drive trait
dispatch.

A surprise from the expansion: fixed-type-id requires nightly,
contradicting the discovery agent's earlier classification of it
as stable. It uses four feature gates: `generic_const_exprs`,
`str_from_raw_parts`, `nonzero_internals`, and conditionally
`specialization` (src/lib.rs:1-7). The version-tuple metadata is a
distinguishing feature (`#[version((0, 1, 0))]`) but does not move
the canonicalisation needle: the version is hashed into the same
runtime u64.

This is the most decisive negative finding of Stage 1, now
strengthened by a third data point: there is a structural Rust
limitation (only scalars are valid const generic kinds on stable)
that rules out approach 3 cleanly without any complexity argument.

### Approach 4: adt_const_params with string const parameters

One codebase, and it remains the bright spot of the survey:
tstr_crates. On stable Rust 1.79+, char const-generic parameters
became available; tstr_crates encodes type-level strings as
`TStr<...>` over tuples of char const-params and provides
`tstr::cmp` returning `core::cmp::Ordering` via const-fn evaluation.
Active maintenance, v0.3.2, MSRV 1.88. tstr_crates does not ship a
sort engine, but the comparison primitive is the most credible
building block surveyed for a custom coproduct sort.

### Approach 5: specialization

No directly surveyed codebase, but tyrade and fixed-type-id both
opt into specialization at the language level. Neither uses it for
type-level sorting; tyrade uses it for DSL trait dispatch, and
fixed-type-id uses it conditionally for hash-tagged dispatch. Both
are nightly-only. The feature is brittle and has no stabilisation
timeline; ruled out by the decisions's stable-only posture.

### Approach 6: TypeId runtime comparison

Three codebases (anymap, rust-typemap, type-uuid in part) implement
this. anymap uses a custom `TypeIdHasher` that transmutes `TypeId`
to u64 for near-zero overhead lookup; rust-typemap layers a `Key`
trait with associated `Value` type on top of the same mechanism for
ergonomic value-type recovery; type-uuid's runtime trait-object
access falls into the same family. All are stable. All produce
runtime deduplication, not type-level canonicalisation. For an
effect row, the user could store values from either ordering in an
anymap and reconstruct in canonical order, but the original
coproduct types remain distinct compile-time entities. This is the
decisions's Option 3 territory.

### Approach 7: marker-trait inequality

spidermeme uses `negative_impls` (nightly) to prove `X != Y` as a
sealed marker trait. Inequality is symmetric, not a total order;
it cannot decide whether `A` precedes `B` or vice versa. As a sort
component it would require pairing with a separate ordering
mechanism, which means it does not move the workaround 2
cost-benefit needle on its own.

### Approach 8: const generics + const fn comparison

Not directly surveyed in any crate. The Rust ecosystem has not
produced a published canonicalisation library taking this path.
tstr_crates' approach 4 is structurally similar (char const-params
plus const-fn comparison) and is the closest realisation.

### Approach 9: type-level hashing with type-level result

**No surveyed codebase implements this.** stabby, type-uuid, and
fixed-type-id all stop at the runtime const u64 / [u8; 16] step
and never lift the hash into a type-level parameter. The structural
blocker is the same as approach 3: only scalars are valid const
generic kinds on stable; on nightly with `adt_const_params` the
design space opens, but no published crate has built it. This is
the gap the discovery agent flagged most clearly, and the expansion
confirmed it remains a gap.

### Approach 10: type-level hash-map / hash-set

Three codebases here, and one notable distinction:

- **anymap and rust-typemap** are runtime-keyed despite the
  nomenclature. Both use `HashMap<TypeId, Box<dyn Any>>` with
  trait-based ergonomics on top. Approach 10 in form, but reduces
  to approach 6 in execution.
- **typemap-meta** is genuinely type-level. The proc-macro
  generates monomorphic `impl Get<T> for Struct` blocks at expansion
  time; trait resolution selects the correct impl with zero runtime
  dispatch and no `TypeId` machinery. This is the most type-level
  codebase in the entire survey.

But typemap-meta solves a fundamentally different problem.
Its "map" is a fixed schema declared at struct definition: the
field set is known up front, and the macro generates one
`impl Get<T>` per field. There is no way to "merge" two coproducts
with reordered effects into a typemap, because the typemap's keys
are exactly the struct fields. For a dynamic effect row whose
contents are determined by composition, typemap-meta provides no
mechanism. The `Coproduct<A, Coproduct<B, Void>>` and
`Coproduct<B, Coproduct<A, Void>>` types are still distinct, and
typemap-meta cannot make them resolve to the same type.

The takeaway is sharper than expected: **stable Rust does support
genuine type-level dispatch on type identity**, just not for
canonicalising dynamic rows. This is a useful piece of evidence for
the decisions: the canonicalisation gap is not "stable Rust is too
weak", it is "stable Rust's type-level dispatch is closed-world".

## 4. Design-space map

Pulling together the per-approach findings:

- **On stable Rust**, the only viable comparison primitive over
  arbitrary user types remains tstr_crates' `tstr::cmp` over char
  const-generic strings (approach 4, requires Rust 1.79+ for char
  const-params and 1.88+ for the crate's MSRV). The expansion did
  not surface an alternative.

- **Stable Rust does have type-level dispatch on type identity**,
  via proc-macro-generated trait impls (typemap-meta, approach 10).
  However, this works only for fixed schemas declared up front, not
  for dynamic row-typed effects. The closed-world assumption is the
  blocker, not Rust's type system per se.

- **No production library** ships an end-to-end coproduct
  canonicalisation. Direct sorts work on Peano nats or
  `typenum::Const<N>` only; comparison primitives ship without sort
  engines; hash-based identity systems produce runtime values; the
  type-level map family solves a different problem (typemap-meta)
  or reduces to runtime dispatch (anymap, rust-typemap).

- **The most credible custom path** is unchanged: proc-macro
  assigns `TStr<...>` names per effect, recursive trait drives a
  sort using `tstr::cmp`, output is the canonicalised coproduct.
  Several hundred lines of macro plus recursive impls; same shape
  as the decisions's workaround 2 description.

## 5. Recommendations for Stage 2

**No Stage 2 deep dives are recommended.** Unchanged from the
original synthesis. The closest near-Stage-2 candidate
(prototype the tstr_crates-driven coproduct sort) remains
implementation work, not research; defer to whenever a use case
forces it.

The expansion did not surface any approach that warrants further
research:

- The DSL family (typ, tyrade) is a known dead-end (both abandoned,
  both nightly).
- The hash-based family (stabby, type-uuid, fixed-type-id) all hit
  the same const-generic structural blocker.
- The hash-map family (anymap, rust-typemap, typemap-meta) is
  either runtime-keyed or fixed-schema-only.

## 6. Relevance to decisions

**Recommendation: no decisions edits required. Workaround 2 stays
declined on the existing reasoning.** Unchanged from the original
synthesis.

Section 4.1 of [../../effects/decisions.md](../../effects/decisions.md)
currently rejects workaround 2 (tag-based type-level sorting) on
speculative complexity and compile-time grounds. Stage 1 confirmed
the speculation; the expansion strengthens the confirmation with five
more codebases, none of which solves the canonicalisation problem.

One narrow note worth adding (low priority, optional): if the
project ever revisits workaround 2, the credible starting point is
tstr_crates' char const-params plus `tstr::cmp` on stable Rust 1.88+,
not typenum. typenum's sealed comparison rules out the typenum-tag
approach the plan currently sketches.

A second narrow note worth adding (low priority, optional): the
distinction surfaced by typemap-meta is worth recording. Stable
Rust does have type-level dispatch on type identity, but only for
**fixed schemas** declared up front. The canonicalisation gap is
specifically about dynamic rows whose composition is determined at
the call site. This is an analytic distinction, not a new
encoding; it explains why the design space looks empty even though
type-level dispatch is technically possible on stable.

No other sections of the decisions are affected. Section 4.1's
workaround 1 (macro-based canonicalisation) and workaround 3
(`CoproductSubsetter` permutation proofs) remain the recommended
mitigations.

## Closing checklist

- [x] All sections above populated
- [x] Status updated to `complete`
- [x] `_status.md` updated to tick `_classification.md` (parent
      handles this); no Stage 2 dives scheduled
- [x] Word count under ~3000 (synthesis grew with the expansion)
