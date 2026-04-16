# Plan: Multi-Brand Ergonomics

**Status:** DRAFT

This plan addresses the ergonomic gap left by the existing brand
inference system for multi-brand concrete types (`Result`, `Pair`,
`Tuple2`, `ControlFlow`, `TryThunk`).

## Motivation

Brand inference
([docs/plans/brand-inference/plan.md](../brand-inference/plan.md),
implemented) lets users call free functions like `map(f, value)` without
a turbofish for types with a single canonical brand. It deliberately
refuses inference for multi-brand types: `Result<A, E>` is reachable
through both `ResultErrAppliedBrand<E>` and `ResultOkAppliedBrand<A>` at
arity 1, and the library cannot pick one without risking silently-wrong
semantics. Those types carry `#[no_inferable_brand]` on their
`impl_kind!` invocations, and users must reach for `explicit::` with a
full turbofish.

The friction shows up in routine code:

```rust
// Today
explicit::map::<ResultErrAppliedBrand<String>, _, _, _, _>(
    |x: i32| x + 1,
    Ok::<i32, String>(5),
)

// PureScript
map (_ + 1) (Right 5 :: Either String Int)

// Haskell
fmap (+1) (Right 5 :: Either String Int)
```

This plan closes most of that gap while preserving the library's
"expose both directions as first-class" design.

See
[analysis/multi-brand-evaluation.md](./analysis/multi-brand-evaluation.md)
for the full analysis of alternatives and their tradeoffs.

## Prerequisites

- Brand inference system is implemented (see
  [brand-inference/plan.md](../brand-inference/plan.md)).
- Current multi-brand types carry `#[no_inferable_brand]` at the
  `impl_kind!` invocation sites.
- The existing `explicit::` dispatch path continues to work unchanged.

## Design summary

Three complementary layers, each targeting a distinct user experience
gap:

1. **Named helpers** (alternative 3 in the analysis): concrete
   direction-specific functions (`map_ok`, `map_err`, `map_fst`,
   `map_snd`, `map_break`, `map_continue`). Additive, no HKT machinery
   touched. Matches the PureScript `map` / `lmap` and Rust stdlib
   `Result::map` / `Result::map_err` naming patterns.
2. **Opt-in primary brand** (alternative 1 in the analysis): for types
   with a canonical direction, designate one brand as primary and let it
   generate `InferableBrand`. Bare `map(f, value)` resolves to the
   primary direction. Types without a canonical direction skip this
   layer.
3. **Targeted diagnostics** (alternative 4, revised in the analysis):
   for types that remain in the all-opt-out configuration,
   `#[diagnostic::on_unimplemented]` messages point users directly at
   the named helpers from layer 1 rather than at the raw `explicit::`
   path.

See the analysis doc for why alternatives 2 (newtype wrappers), 5
(closure-directed inference), and 6 (type-only priority without closure)
are not pursued.

## Per-type strategy decisions

Each multi-brand type uses one of two strategies:

- **Strategy A (designated primary):** One brand gets `InferableBrand`
  generation and is the default for `map`. Siblings keep
  `#[no_inferable_brand]`.
- **Strategy B (all-explicit):** No designated primary. All brands keep
  `#[no_inferable_brand]`. `map` still refuses; users reach for named
  helpers or `explicit::`.

Proposed assignment:

| Type               | Strategy | Primary brand                      | Named helpers               |
| ------------------ | -------- | ---------------------------------- | --------------------------- |
| `Result<A, E>`     | A        | `ResultErrAppliedBrand<E>` (Ok)    | `map_ok`, `map_err`         |
| `Pair<A, B>`       | A        | `PairFirstAppliedBrand<A>` (snd)   | `map_fst`, `map_snd`        |
| `(A, B)`           | A        | `Tuple2FirstAppliedBrand<A>` (snd) | `map_fst`, `map_snd`        |
| `TryThunk<A, E>`   | A        | `TryThunkErrAppliedBrand<E>` (ok)  | `map_ok`, `map_err`         |
| `ControlFlow<B,C>` | B        | N/A                                | `map_break`, `map_continue` |

Rationale per choice:

- `Result`, `TryThunk`: success side is canonically primary, matching
  Haskell `Functor (Either e)`, PureScript `Functor (Either e)`, and
  Rust stdlib `Result::map`.
- `Pair`, `(A, B)`: map-over-second is the Haskell convention for
  `Functor ((,) a)`.
- `ControlFlow`: neither `Break` nor `Continue` is canonically primary.
  Rust stdlib's `ControlFlow::map_break` and `map_continue` are
  symmetric peers with no `map`. Forcing a primary here would encode an
  arbitrary choice.

## Detailed design

### Layer 1: Named helpers

Thin wrappers around `explicit::<SpecificBrand>`:

```rust
// In functions::explicit (or a new functions::helpers module; see
// open questions).

pub fn map_ok<'a, T: 'a, E: 'a, B: 'a, R>(
    f: impl FunctorDispatch<'a, ResultErrAppliedBrand<E>, T, B, R, _>,
    r: R,
) -> Apply!(...) where R: ... {
    explicit::map::<ResultErrAppliedBrand<E>, _, _, _, _>(f, r)
}

// Symmetric for map_err, map_fst, map_snd, map_break, map_continue.
```

Exact signature shape depends on resolution of the open questions
around Val/Ref dispatch and closure input annotations.

### Layer 2: Opt-in primary brand

For each Strategy A type, remove `#[no_inferable_brand]` from the primary
brand's `impl_kind!` invocation and leave it on all non-primary brands.
This alone makes the primary brand inferable via the existing
`InferableBrand_{hash}` machinery.

Introducing a dedicated `#[primary_brand]` attribute is optional; its
only value is documentary (it makes the intent visible at the
declaration site). The mechanical behavior is identical to leaving the
brand unattributed. Recommendation: introduce the attribute for
readability, but implement it as a no-op marker in the macro.

Rust coherence catches any accidental configuration where two brands
both claim `InferableBrand` for the same concrete type, so no macro
cross-check is needed.

### Layer 3: Targeted diagnostics

For Strategy B types (currently just `ControlFlow`, plus any downstream
user-defined types), extend the `#[diagnostic::on_unimplemented]`
messages on the relevant `InferableBrand_{hash}` traits to suggest the
named helpers from layer 1.

Proposed message shape for `ControlFlow<B, C>`:

```text
`ControlFlow<B, C>` does not have a canonical brand and cannot use brand inference.
  = help: use `map_break(f, cf)` to transform the Break side
  = help: use `map_continue(f, cf)` to transform the Continue side
  = note: or use the `explicit::` variant with an explicit brand turbofish
```

Implementation likely requires per-brand `on_unimplemented` attributes
(similar to how the current generic "does not have a unique brand"
message is attached), specialized to the helper name for each type.

## Out of scope

- **Alternative 2 (newtype wrappers).** Forces users to wrap and unwrap
  values at API boundaries, conflicting with the library's "pass your
  normal types in" design.
- **Alternative 5 (closure-directed inference).** Feasibility POC at
  [fp-library/tests/closure_directed_inference_poc.rs](../../../fp-library/tests/closure_directed_inference_poc.rs)
  confirms the `Slot<Brand, A>` pattern works on stable for non-diagonal
  cases. Not pursued because: the diagonal `T = T` case remains a
  permanent failure; closure input types must be explicitly annotated;
  and named helpers deliver equivalent or better ergonomics without the
  machinery. Kept as a documented feasibility artifact.
- **Alternative 6 (type-only priority without closure).** Not
  achievable on stable rustc without specialization or negative impls.

## Open questions

1. **Should `#[primary_brand]` be introduced as an explicit attribute?**
   Mechanically unnecessary (absence of `#[no_inferable_brand]` already
   implies it), but potentially valuable for declaration-site
   documentation. Leaning toward yes.
2. **Where do the named helpers live?** Options: inline in
   `functions::explicit`, a new `functions::helpers` module, or as
   inherent methods on concrete types (mimicking `Result::map_err` from
   stdlib). Inherent methods would conflict with the existing stdlib
   ones and are probably not viable.
3. **Helper naming conventions.** `map_err` vs `map_left`,
   `map_fst` vs `map_first`, etc. Leaning toward: match stdlib where
   precedent exists (`map_err`), use short forms otherwise (`map_fst`,
   `map_snd`). Needs audit against existing library naming.
4. **Extension to other operations.** Does this scheme generalize to
   `bind_ok` / `bind_err`, `fold_ok` / `fold_err`,
   `traverse_ok` / `traverse_err`, etc.? Answering "yes" multiplies the
   API surface significantly. Answering "no" creates asymmetry between
   `map` and other operations on the same types. Needs a policy.
5. **Val/Ref dispatch for helpers.** The existing dispatch system
   transparently routes by-value and by-reference paths through a
   single dispatch trait. Do the helpers need separate
   `ref_map_err`-style variants, or does the closure's input type
   annotation (`|x: i32|` vs `|x: &i32|`) flow through correctly?
   Likely transparent but needs verification.
6. **Diagnostic template precision.** How specific should the
   `on_unimplemented` messages be? Full helper path
   (`fp_library::functions::map_err`), short name (`map_err`), or
   prose (`the `map_err` helper`)? Needs a convention check.
7. **Migration and deprecation.** Since layer 2 is strictly additive
   from the user's perspective (previously-ambiguous types gain a new
   successful inference path), no breaking change. Layer 1 is
   additive. Layer 3 is polish. No migration plan needed beyond
   documenting the new helpers in release notes.

## Implementation phasing

Each layer can land as an independent PR in sequence, with each
releasable on its own:

1. **Layer 1 (named helpers).** Implement `map_ok`, `map_err`,
   `map_fst`, `map_snd`, `map_break`, `map_continue`. Add doctests and
   unit tests. No changes to brand-inference machinery. Strictly
   additive.
2. **Layer 2 (opt-in primary brands).** For each Strategy A type,
   remove `#[no_inferable_brand]` from the primary brand's
   `impl_kind!`. Optionally introduce `#[primary_brand]` attribute.
   Add UI tests confirming `map(f, ok)` now succeeds with Ok-bias.
   Keep existing compile-fail tests for non-primary brands and
   Strategy B types.
3. **Layer 3 (diagnostics).** Extend `on_unimplemented` messages on
   Strategy B types. Update existing `.stderr` snapshots in
   `fp-library/tests/ui/`.

Answers to the open questions (especially naming conventions and
operation scope) should precede layer 1 implementation.

## Success criteria

- `map(f, Ok(5))` compiles and maps over `Ok`.
- `map_err(f, Err("fail".into()))` compiles and maps over `Err`.
- `map(f, ControlFlow::Continue(5))` fails to compile with a diagnostic
  naming `map_break` and `map_continue`.
- All existing `explicit::map::<...>(f, value)` calls continue to work
  unchanged.
- No regression in compile-fail or property test suites.
- Library documentation updated with the `map` / `map_err` convention
  explained alongside the existing brand inference docs.
