# Stage 1 classification: aggregated findings

**Status:** complete
**Last updated:** 2026-04-24

## 1. What Stage 1 set out to answer

The originating question was whether type-level sorting in Rust is
feasible enough to inform the effects research's port-plan section 4.1
"ordering mitigations" subsection (specifically workaround 2: tag-based
type-level sorting, currently rejected on speculative complexity and
compile-time grounds). Stage 1 surveyed 11 codebases under
`/home/jessea/Documents/projects/type-level/` against the eight
candidate approaches catalogued in [README.md](README.md):

1. Peano + typenum comparison.
2. Proc-macro textual canonicalisation.
3. Hash-based type tags.
4. `feature(adt_const_params)` with string const parameters.
5. `feature(specialization)` / `min_specialization`.
6. `std::any::TypeId` runtime comparison.
7. Marker-trait inequality via orphan-rule tricks.
8. Const generics + `const fn` comparison.

Each Stage 1 file answered three narrow questions: which approach does
this codebase implement, is it on stable or nightly Rust, and could it
canonicalise a coproduct row of arbitrary effect types? This synthesis
aggregates the verdicts, maps the design space, and decides whether
the port-plan should be revised or any Stage 2 deep dive is justified.

## 2. Classification table

Columns: codebase, primary approach, sort capability, stable or
nightly, applicability to coproduct row canonicalisation.

| Codebase             | Approach        | Sort capability                      | Stable / nightly  | Coproduct canonicalisation?                                           |
| -------------------- | --------------- | ------------------------------------ | ----------------- | --------------------------------------------------------------------- |
| type-level-sort      | 1 (Peano)       | Bubble sort over Peano nats only     | Stable            | No (numeric domain only).                                             |
| typelist             | 1 (Peano)       | Merge sort over `typenum::Const<N>`  | Stable            | No (numeric domain only).                                             |
| typenum              | 1 (enabler)     | Comparison primitives (sealed)       | Stable, MSRV 1.41 | Partial (user must write the sort engine; sealed against user types). |
| frunk                | 1 (permutation) | Permutation proof; no canonical form | Stable, ed 2021   | No (target type required as input).                                   |
| static-assertions-rs | None            | Predicate checks only                | Stable, MSRV 1.37 | No (validation, not transformation).                                  |
| stabby               | 3 (runtime u64) | Hash via const-fn SHA256             | Stable, MSRV 1.61 | No (`const ID: u64` not a type parameter).                            |
| type-uuid            | 3 / 6           | `const UUID: [u8; 16]` per type      | Stable, ed 2018   | No (arrays disallowed as const generic params).                       |
| rust-type-freak      | None            | List ops; no sort, no Cmp            | Stable, ed 2018   | No (no comparison machinery; abandoned 2020).                         |
| typ                  | 2 (DSL)         | Turing-complete; no shipped sort     | Nightly           | Possible but verbose; abandoned 2021.                                 |
| tstr_crates          | 4 (chars)       | `tstr::cmp -> Ordering`; no sort     | Stable, MSRV 1.88 | Possible building block (most credible).                              |
| spidermeme           | 7 (inequality)  | Inequality only; not a total order   | Nightly           | No (symmetric relation cannot order).                                 |

Two cross-cutting findings emerge only from the table. First, **no
surveyed crate solves coproduct canonicalisation**. Of 11 codebases,
zero ship a sort that operates on arbitrary type identities; the
direct sort implementations (type-level-sort, typelist) are confined
to numeric domains, the comparison primitives (typenum, tstr_crates)
ship without sort engines, and the identity systems (stabby,
type-uuid) produce values not types. Second, **the stable-Rust path
narrows to one approach**: tstr_crates on Rust 1.79+ via char
const-generic parameters. Every other live primitive either requires
nightly features (`negative_impls`, `adt_const_params` with
`&'static str`, `specialization`) or has a structural blocker on
stable.

## 3. Approach-by-approach summary

### Approach 1: Peano + typenum comparison

Three codebases cluster here. typenum supplies the comparison
primitive (`Cmp` returning `Greater` / `Less` / `Equal` marker types,
src/type_operators.rs:310-316), but the trait is sealed via a
`Sealed` private trait (src/lib.rs:149-173); only typenum's own
integer types implement it. type-level-sort and typelist write the
sort engine but are pinned to numeric domains: type-level-sort sorts
hand-rolled Peano nats, typelist sorts `typenum::Const<N>`. Neither
extends to type identities.

For the port-plan, the gap is structural rather than missing-engine.
A user wanting to sort a coproduct row would need to (a) assign each
effect a unique typenum tag (manually), (b) write a custom recursive
sort trait that uses `Cmp` as the decision gate, and (c) maintain a
parallel mapping from typenum tags back to effect types. Steps (a)
and (c) are exactly the manual coordination the port-plan's
workaround 2 originally cited as prohibitive. The libraries do not
remove that cost; they validate it as the dominant cost.

### Approach 2: Proc-macro textual canonicalisation

One surveyed codebase: `typ`. It is a proc-macro DSL that compiles
type-level functions to trait-projection chains. Turing-complete in
principle (recursion + ADT pattern matching), but ships no sort and
requires nightly (`hash_set_entry` feature, src/lib.rs:5). Last
commit July 2021; effectively abandoned. The author's more mature
follow-up `rust-type-freak` does not include the sort either.

The effects research already covered macro-based canonicalisation
(corophage's `Effects![...]` macro, which does NOT sort; preserves
user order). No new evidence here changes the workaround-1
recommendation in the port-plan.

### Approach 3: Hash-based type tags

Two codebases: stabby and type-uuid. Both compute a stable
compile-time identifier per type, but both store the result as a
**runtime constant**, not a type parameter. stabby's `const ID: u64`
(stabby-abi/src/istable.rs:30-53, computed via const-fn SHA256 in
report.rs:172-176) is a `u64` associated const; type-uuid's
`const UUID: [u8; 16]` (src/lib.rs:60-62) is a fixed-size array. On
stable Rust, neither shape can be lifted into a const generic
parameter to drive trait dispatch: only scalar types (integer, bool,
char) are valid const generic kinds outside of nightly's
`adt_const_params`.

This is the most decisive negative finding of Stage 1. The
hash-based approach was a serious candidate going in (intuitively,
"give every type a stable u64 and sort by that") but the structural
Rust limitation rules it out cleanly without any complexity argument.
On nightly with `adt_const_params` the design space opens, but the
port-plan rejects nightly.

### Approach 4: adt_const_params with string const parameters

One codebase, and it is the bright spot of the survey: tstr_crates.
On stable Rust 1.79+, char const-generic parameters became
available; tstr_crates encodes type-level strings as
`TStr<...>` over tuples of `char` const-params
(`tstr/src/tstr_impl_with_chars.rs:277-282`) and provides
`tstr::cmp` returning `core::cmp::Ordering` via const-fn evaluation
(`tstr/src/tstr_fns.rs:138-144`). Active maintenance, v0.3.2, MSRV
Rust 1.88.

tstr_crates does not ship a sort engine, but the comparison
primitive is the most credible building block surveyed. A working
coproduct canonicalisation could be assembled as: a proc-macro
assigns each effect a `TStr` name, a custom recursive trait drives
insertion sort using `tstr::cmp` as the decision gate, and the row
is rebuilt in canonical order. This is net-new code (the proc-macro
plus the recursive trait), not glue, but it has a foundation that
typenum's sealed pattern denied.

### Approaches 5, 6, 7, 8

Approach 5 (specialization) was not surveyed because no production
crate uses `min_specialization` for sorting. The feature is brittle
and tied to nightly; ruled out by the port-plan's stable-only
posture independent of this research.

Approach 6 (TypeId runtime comparison) was not directly surveyed but
appears as the runtime fallback inside type-uuid. It is not
type-level proper and falls back to the port-plan's Option 3
(TypeId dispatch) territory in section 4.1.

Approach 7 (marker-trait inequality) is spidermeme. It uses
`negative_impls` (nightly) to prove `X != Y` as a sealed marker
trait. Critical limitation: inequality is symmetric, not a total
order. spidermeme cannot decide whether `A` precedes `B` or vice
versa; it can only assert they differ. As a sort component it would
require pairing with a separate ordering mechanism, which means it
does not move the workaround-2 cost-benefit needle on its own.

Approach 8 (const generics + const fn comparison) was not surveyed
in any crate. The Rust ecosystem has not produced a published
canonicalisation library taking this path. tstr_crates' approach 4
is structurally similar (char const-params plus const-fn comparison)
and is the closest realisation of approach 8's idea.

## 4. Design-space map

Pulling together the per-approach findings:

- **On stable Rust**, the only viable comparison primitive over
  arbitrary user types is tstr_crates' `tstr::cmp` over char
  const-generic strings (approach 4, requires Rust 1.79+ for char
  const-params and 1.88+ for the crate's MSRV). Every other approach
  either requires nightly (5, 7, full string const-params), is
  sealed against user types (typenum, approach 1 enabler), produces
  the wrong shape (3, runtime constant not type-level), or has the
  wrong relation (7, inequality not order).

- **No production library** ships an end-to-end sort over arbitrary
  type identities. Direct sorts work on Peano nats or
  `typenum::Const<N>` only; comparison primitives ship without sort
  engines; identity systems produce runtime values. Building a
  working coproduct sort therefore means composing
  comparison-plus-engine, which is custom development.

- **The most credible custom path** is: proc-macro assigns
  `TStr<...>` names per effect, recursive trait drives a sort using
  `tstr::cmp`, output is the canonicalised coproduct. This would be
  several hundred lines of macro plus recursive impls. It has the
  same shape as the port-plan's workaround 2 description.

## 5. Recommendations for Stage 2

**No Stage 2 deep dives are recommended.**

The Stage 1 survey already produced the answer: the only credible
path is implementation work, not research. A Stage 2 dive on
"prototype the tstr_crates-driven coproduct sort" would be writing
the macro and the trait, which is implementation, not research. The
synthesis above is enough to support a "decline" decision on the
port-plan's workaround 2; if the project ever revisits the decision,
the implementation can begin directly without further research
preamble.

The closest near-Stage-2 candidates and the reasons for declining:

- _Prototype the tstr_crates sort._ This is not research; it is
  implementation. Defer until and unless a use case forces it.
- _Survey approach 8 (const generics + const fn) more deeply._ No
  surveyed crate uses this path; the closest is tstr_crates which we
  have already classified. Additional research would not surface
  new evidence.
- _Investigate whether typenum could be unsealed via a fork._ This
  is also implementation work and would require coordinating with
  the upstream maintainer or maintaining a fork. Out of scope.

## 6. Relevance to port-plan

**Recommendation: no port-plan edits required. Workaround 2 stays
declined on the existing reasoning.**

Section 4.1 of [../../effects/port-plan.md](../../effects/port-plan.md)
currently rejects workaround 2 (tag-based type-level sorting) on
speculative complexity and compile-time grounds. Stage 1 confirms
the speculation: the Rust ecosystem has not produced a working
example of this approach, the only credible building blocks
(tstr_crates) ship comparison primitives without sort engines, and
realising workaround 2 would be a substantial custom library on top
of net-new proc-macro plus recursive trait machinery. The original
complexity argument stands.

One narrow note worth adding to section 4.1's ordering-mitigations
subsection (optional, low priority): **if the project ever revisits
workaround 2, the credible starting point is tstr_crates' char
const-params plus `tstr::cmp` on stable Rust 1.88+, not typenum**.
typenum's sealed comparison rules out the typenum-tag approach the
plan currently sketches. This is a small factual correction; the
verdict on workaround 2 does not change.

No other sections of the port-plan are affected. Section 4.1's
workaround 1 (macro-based canonicalisation) and workaround 3
(`CoproductSubsetter` permutation proofs) remain the recommended
mitigations and are unchanged by Stage 1.

## Closing checklist

- [x] All sections above populated
- [x] Status updated to `complete`
- [x] `_status.md` updated to tick `_classification.md` (parent
      handles this); no Stage 2 dives scheduled
- [x] Word count under ~2500
