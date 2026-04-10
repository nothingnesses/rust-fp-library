# Plan: Dispatch Expansion

## Motivation

The dispatch system unifies by-value and by-reference function variants
behind a single free function. For example, `map(f, fa)` dispatches to
`Functor::map` when `f: Fn(A) -> B` (Val) or `RefFunctor::ref_map` when
`f: Fn(&A) -> B` (Ref). Dispatch is driven by the closure's argument
type via marker traits (`Val`/`Ref`) and the container type (`FA`
parameter: owned vs borrowed).

Currently, dispatch exists for 12 functions: `map`, `bind`,
`bind_flipped`, `compose_kleisli`, `compose_kleisli_flipped`,
`lift2`-`lift5`, `fold_right`, `fold_left`, `fold_map`, `filter_map`,
`traverse`.

Nine additional function pairs have both Val and Ref variants with
closure arguments suitable for dispatch but lack dispatch traits. This
plan adds dispatch for all of them.

## Scope

### Functions to add dispatch for

**Simple group (same shape as FunctorDispatch/FilterMapDispatch):**

| Pair                                  | Val closure             | Ref closure              |
| ------------------------------------- | ----------------------- | ------------------------ |
| `filter` / `ref_filter`               | `Fn(A) -> bool`         | `Fn(&A) -> bool`         |
| `partition` / `ref_partition`         | `Fn(A) -> bool`         | `Fn(&A) -> bool`         |
| `partition_map` / `ref_partition_map` | `Fn(A) -> Result<O, E>` | `Fn(&A) -> Result<O, E>` |

**WithIndex group (adds `Brand::Index` projection, no extra type param):**

| Pair                                                        | Val closure                  | Ref closure                   |
| ----------------------------------------------------------- | ---------------------------- | ----------------------------- |
| `map_with_index` / `ref_map_with_index`                     | `Fn(Idx, A) -> B`            | `Fn(Idx, &A) -> B`            |
| `filter_with_index` / `ref_filter_with_index`               | `Fn(Idx, A) -> bool`         | `Fn(Idx, &A) -> bool`         |
| `filter_map_with_index` / `ref_filter_map_with_index`       | `Fn(Idx, A) -> Option<B>`    | `Fn(Idx, &A) -> Option<B>`    |
| `partition_with_index` / `ref_partition_with_index`         | `Fn(Idx, A) -> bool`         | `Fn(Idx, &A) -> bool`         |
| `partition_map_with_index` / `ref_partition_map_with_index` | `Fn(Idx, A) -> Result<O, E>` | `Fn(Idx, &A) -> Result<O, E>` |

**Complex group (multiple brand parameters, follows TraverseDispatch):**

| Pair                    | Val closure                    | Ref closure                     | Extra params   |
| ----------------------- | ------------------------------ | ------------------------------- | -------------- |
| `wilt` / `ref_wilt`     | `Fn(A) -> M::Of<Result<O, E>>` | `Fn(&A) -> M::Of<Result<O, E>>` | `FnBrand`, `M` |
| `wither` / `ref_wither` | `Fn(A) -> M::Of<Option<B>>`    | `Fn(&A) -> M::Of<Option<B>>`    | `FnBrand`, `M` |

### Additional candidates (from ref-expansion plan)

After the ref-expansion plan adds new Ref traits, these additional
pairs become candidates for dispatch:

| Pair                                  | Group   | Notes                                                                       |
| ------------------------------------- | ------- | --------------------------------------------------------------------------- |
| `bimap` / `ref_bimap`                 | Simple  | Two closures (`f`, `g`), but dispatch on first closure's arg type suffices. |
| `bi_fold_right` / `ref_bi_fold_right` | Complex | Has `FnBrand`.                                                              |
| `bi_fold_left` / `ref_bi_fold_left`   | Complex | Has `FnBrand`.                                                              |
| `bi_fold_map` / `ref_bi_fold_map`     | Complex | Has `FnBrand`.                                                              |
| `bi_traverse` / `ref_bi_traverse`     | Complex | Has `FnBrand` + effect brand `F`.                                           |

These will be added as part of this plan after the ref-expansion plan
is implemented.

### Functions NOT getting dispatch

Functions without closures cannot use the closure-type-based dispatch
mechanism: `join`/`ref_join`, `alt`/`ref_alt`, `compact`/`ref_compact`,
`separate`/`ref_separate`, `apply_first`/`ref_apply_first`,
`apply_second`/`ref_apply_second`, `if_m`/`ref_if_m`, `when_m`/`unless_m`
and their ref counterparts. These keep separate free functions.

## Design

### Dispatch trait pattern

Each dispatch trait follows the established two-impl pattern:

```
pub trait FilterDispatch<'a, Brand: Kind_..., A: 'a, FA, Marker> {
    fn dispatch(self, fa: FA) -> Apply!(Brand::Of<'a, A>);
}

// Val impl: FA = Apply!(Of<A>), closure Fn(A) -> bool
impl<...> FilterDispatch<'a, Brand, A, Apply!(Of<A>), Val> for F
where Brand: Filterable, F: Fn(A) -> bool + 'a { ... }

// Ref impl: FA = &'b Apply!(Of<A>), closure Fn(&A) -> bool
impl<...> FilterDispatch<'a, Brand, A, &'b Apply!(Of<A>), Ref> for F
where Brand: RefFilterable, F: Fn(&A) -> bool + 'a { ... }
```

The unified free function:

```
pub fn filter<'a, Brand: Kind_..., A: 'a, FA, Marker>(
    f: impl FilterDispatch<'a, Brand, A, FA, Marker>,
    fa: FA,
) -> Apply!(Brand::Of<'a, A>) {
    f.dispatch(fa)
}
```

### WithIndex: Index as a projection

`WithIndex::Index` is an associated type on the brand, not a standalone
type parameter. The dispatch trait uses `Brand::Index` in its bounds:

```
pub trait MapWithIndexDispatch<'a, Brand: Kind_... + WithIndex, A: 'a, B: 'a, FA, Marker> {
    fn dispatch(self, fa: FA) -> Apply!(Brand::Of<'a, B>);
}
```

Val impl: `F: Fn(Brand::Index, A) -> B`.
Ref impl: `F: Fn(Brand::Index, &A) -> B`.

Index does not add a turbofish position. It does not interfere with
Val/Ref dispatch because `Index` is always passed by value (it
implements `Clone`).

### Wilt/wither: multiple brand parameters

Follow the TraverseDispatch pattern. Include `FnBrand` as a type
parameter (unused in Val, passed through in Ref). Include `M`
(applicative brand) as an explicit parameter.

```
pub trait WiltDispatch<'a, FnBrand, Brand: Kind_..., M: Applicative,
    A: 'a, E: 'a, O: 'a, FA, Marker>
{
    fn dispatch(self, fa: FA)
        -> Apply!(M::Of<'a, (Apply!(Brand::Of<'a, E>), Apply!(Brand::Of<'a, O>))>);
}
```

### Partition return types

No special handling. Both Val and Ref paths return identical owned tuple
types. The dispatch trait return type is a tuple of `Brand::Of<...>`.

### E0119 safety

All 9 pairs are safe. The two-impl pattern distinguishes on:

- Closure argument: `A` (Val) vs `&A` (Ref).
- Container type: `Brand::Of<'a, A>` (Val) vs `&Brand::Of<'a, A>` (Ref).

Both axes are structurally distinct. No three-impl patterns are needed.

### Re-export handling

Follow the established pattern in `functions.rs`:

1. Dispatch version gets the canonical name (`filter`, `partition`, etc.).
2. Non-dispatch free functions get aliased names via
   `generate_function_re_exports!` (e.g., `filterable_filter`).
3. Non-dispatch functions remain available for trait doc examples and
   as the dispatch target.

### Turbofish changes

| Function                   | Current (Val) | Current (Ref) | Dispatch | Delta |
| -------------------------- | ------------- | ------------- | -------- | ----- |
| `filter`                   | 2             | 2             | 4        | +2    |
| `partition`                | 2             | 2             | 4        | +2    |
| `partition_map`            | 4             | 4             | 6        | +2    |
| `map_with_index`           | 3             | 3             | 5        | +2    |
| `filter_with_index`        | 2             | 2             | 4        | +2    |
| `filter_map_with_index`    | 3             | 3             | 5        | +2    |
| `partition_with_index`     | 2             | 2             | 4        | +2    |
| `partition_map_with_index` | 4             | 4             | 6        | +2    |
| `wilt`                     | 5             | 6             | 8        | +3    |
| `wither`                   | 4             | 5             | 7        | +3    |

The consistent +2 (FA + Marker) matches existing dispatch traits.
Wilt/wither have +3 because the Val path gains `FnBrand` (unused but
required for uniformity with the Ref path), matching TraverseDispatch.

## Implementation Order

1. **Simple group:** `FilterDispatch`, `PartitionDispatch`,
   `PartitionMapDispatch`. These follow the FilterMapDispatch pattern
   directly with minimal variation.

2. **WithIndex group:** `MapWithIndexDispatch`,
   `FilterWithIndexDispatch`, `FilterMapWithIndexDispatch`,
   `PartitionWithIndexDispatch`, `PartitionMapWithIndexDispatch`.
   These add `Brand: WithIndex` bound and use `Brand::Index` in the
   closure type.

3. **Complex group:** `WiltDispatch`, `WitherDispatch`. These follow
   the TraverseDispatch pattern with `FnBrand` + `M` parameters.

4. **Bifunctorial dispatch** (after ref-expansion plan): `BimapDispatch`,
   `BiFoldRightDispatch`, `BiFoldLeftDispatch`, `BiFoldMapDispatch`,
   `BiTraverseDispatch`. Added after RefBifunctor/RefBifoldable/
   RefBitraversable are available.

5. **Update `functions.rs`:** Add canonical dispatch exports and alias
   the non-dispatch free functions for each new dispatch trait.

6. **Update call sites:** Update all turbofish counts in source, doc
   comments, tests, and benchmarks.

7. **Tests.** Verify dispatch routing for each new trait (Val closure
   routes to by-value, Ref closure routes to by-reference). Property
   tests confirming Val and Ref paths produce identical results.

## Verification

After each step, run `just verify` (fmt, clippy, doc, test) with a
90-second timeout per command.

## References

- Signature survey: `docs/plans/dispatch-expansion/analysis/signature-survey.md`
- Design concerns: `docs/plans/dispatch-expansion/analysis/design-concerns.md`
- Existing dispatch pattern: `fp-library/src/classes/dispatch/functor.rs`
- FilterMap dispatch: `fp-library/src/classes/dispatch/filterable.rs`
- Traverse dispatch: `fp-library/src/classes/dispatch/traversable.rs`
- Re-export mechanism: `fp-library/src/functions.rs`
