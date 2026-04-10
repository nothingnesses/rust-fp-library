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

**WithIndex + FnBrand group (indexed folds and traversals):**

| Pair                                                  | Val closure              | Ref closure               | Extra params   |
| ----------------------------------------------------- | ------------------------ | ------------------------- | -------------- |
| `fold_map_with_index` / `ref_fold_map_with_index`     | `Fn(Idx, A) -> M`        | `Fn(Idx, &A) -> M`        | `FnBrand`      |
| `fold_right_with_index` / `ref_fold_right_with_index` | `Fn(Idx, A, B) -> B`     | `Fn(Idx, &A, B) -> B`     | `FnBrand`      |
| `fold_left_with_index` / `ref_fold_left_with_index`   | `Fn(Idx, B, A) -> B`     | `Fn(Idx, B, &A) -> B`     | `FnBrand`      |
| `traverse_with_index` / `ref_traverse_with_index`     | `Fn(Idx, A) -> F::Of<B>` | `Fn(Idx, &A) -> F::Of<B>` | `FnBrand`, `F` |

Note: `ref_fold_left_with_index` currently uses `Fn(B, Idx, &A) -> B`,
which does not match the Val ordering `Fn(Idx, B, A) -> B` or the
PureScript convention `(i -> b -> a -> b)`. The Ref signature must be
fixed to `Fn(Idx, B, &A) -> B` before adding dispatch, so that the
only difference between Val and Ref closures is `A` vs `&A`. See the
prerequisite fix section below.

**Complex group (multiple brand parameters, follows TraverseDispatch):**

| Pair                    | Val closure                    | Ref closure                     | Extra params   |
| ----------------------- | ------------------------------ | ------------------------------- | -------------- |
| `wilt` / `ref_wilt`     | `Fn(A) -> M::Of<Result<O, E>>` | `Fn(&A) -> M::Of<Result<O, E>>` | `FnBrand`, `M` |
| `wither` / `ref_wither` | `Fn(A) -> M::Of<Option<B>>`    | `Fn(&A) -> M::Of<Option<B>>`    | `FnBrand`, `M` |

### Additional candidates (from ref-expansion plan)

After the ref-expansion plan adds new Ref traits, these additional
pairs become candidates for dispatch:

| Pair                                  | Group   | Notes                                                           |
| ------------------------------------- | ------- | --------------------------------------------------------------- |
| `bimap` / `ref_bimap`                 | Simple  | Dispatch on closure tuple `(f, g)`. Both must agree on Val/Ref. |
| `bi_fold_right` / `ref_bi_fold_right` | Complex | Has `FnBrand`.                                                  |
| `bi_fold_left` / `ref_bi_fold_left`   | Complex | Has `FnBrand`.                                                  |
| `bi_fold_map` / `ref_bi_fold_map`     | Complex | Has `FnBrand`.                                                  |
| `bi_traverse` / `ref_bi_traverse`     | Complex | Has `FnBrand` + effect brand `F`.                               |

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

### Method naming convention

Use bare `dispatch` for all new dispatch traits. Each dispatch trait has
exactly one method, and the method is only ever called from inside the
unified free function (never by users directly), so there is no
ambiguity. As part of this plan, also rename the existing qualified
methods (`dispatch_bind`, `dispatch_filter_map`, `dispatch_traverse`,
`dispatch_lift2`-`dispatch_lift5`) to bare `dispatch` for consistency
with the foldable and functor dispatch traits that already use it.

### Clone bounds on FilterDispatch and PartitionDispatch

`FilterDispatch` and `PartitionDispatch` must include `A: Clone` in
their trait bounds, matching the underlying `Filterable::filter` and
`Filterable::partition` methods. The predicate `Fn(A) -> bool` consumes
the element, but the element may need to be kept. The existing
`FilterMapDispatch` does NOT require `A: Clone` (the closure returns
`Option<B>`, a new value). This distinction must be preserved.

### Bifunctorial dispatch uses two-parameter Kind hash

`BimapDispatch` and other bi-\* dispatch traits use `Kind_266801a817966495`
(two type parameters: `type Of<'a, A: 'a, B: 'a>: 'a`) instead of the
standard `Kind_cdc7cd43dac7585f` (one type parameter). The `Apply!` macro
invocations must use the two-param form. Model on the existing `bimap`
free function in `bifunctor.rs`.

### `ref_*` function handling

When dispatch unifies two functions, the `ref_*` free function should be
removed from the public API, with the dispatch version fully replacing
it. This follows the precedent of `ref_map`, which was removed when
`FunctorDispatch` was added. Note: `ref_filter_map` and `ref_traverse`
were inconsistently kept when their dispatch versions were added; this
should be cleaned up during dispatch expansion.

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
| `fold_map_with_index`      | 4             | 4             | 6        | +2    |
| `fold_right_with_index`    | 4             | 4             | 6        | +2    |
| `fold_left_with_index`     | 4             | 4             | 6        | +2    |
| `traverse_with_index`      | 5             | 6             | 8        | +3    |
| `wilt`                     | 5             | 6             | 8        | +3    |
| `wither`                   | 4             | 5             | 7        | +3    |

The consistent +2 (FA + Marker) matches existing dispatch traits.
Functions with `FnBrand` (foldable/traversable with index, wilt, wither)
have +3 on the Val side because the Val path gains `FnBrand` (unused but
required for uniformity with the Ref path), matching TraverseDispatch.

## Prerequisite Fix

### `ref_fold_left_with_index` argument order

The Ref signature `Fn(B, Self::Index, &A) -> B` does not match the Val
signature `Fn(Self::Index, B, A) -> B`. The PureScript convention is
`(i -> b -> a -> b)`, which matches the Val ordering. The Ref signature
must be fixed to `Fn(Self::Index, B, &A) -> B` before adding dispatch,
so that the only difference between Val and Ref closures is `A` vs `&A`.

This is a breaking change to both `RefFoldableWithIndex::ref_fold_left_with_index`
and `SendRefFoldableWithIndex::send_ref_fold_left_with_index`. All trait
definitions, default impl bodies, doc tests, concrete implementations,
and call sites must be updated. The blast radius is small: 2 trait
definitions, 2 default impl bodies, 2 doc tests, 0 concrete overrides,
0 external call sites.

## Implementation Order

0. **Prerequisite: Fix `ref_fold_left_with_index` argument order.**
   Change `Fn(B, Self::Index, &A) -> B` to `Fn(Self::Index, B, &A) -> B`
   in both `RefFoldableWithIndex` and `SendRefFoldableWithIndex` trait
   definitions, default impl bodies, and doc tests.

0b. **Prerequisite: Rename existing dispatch methods to bare `dispatch`.**
Rename `dispatch_bind`, `dispatch_filter_map`, `dispatch_traverse`,
`dispatch_lift2`-`dispatch_lift5` to `dispatch` in the trait
definitions, all impls, and all free function call sites. This is a
mechanical rename within `fp-library/src/classes/dispatch/`.

1. **Simple group:** `FilterDispatch`, `PartitionDispatch`,
   `PartitionMapDispatch`. These follow the FilterMapDispatch pattern
   directly with minimal variation.

2. **WithIndex group (filterable/functor):** `MapWithIndexDispatch`,
   `FilterWithIndexDispatch`, `FilterMapWithIndexDispatch`,
   `PartitionWithIndexDispatch`, `PartitionMapWithIndexDispatch`.
   These add `Brand: WithIndex` bound and use `Brand::Index` in the
   closure type.

3. **WithIndex group (foldable/traversable):**
   `FoldMapWithIndexDispatch`, `FoldRightWithIndexDispatch`,
   `FoldLeftWithIndexDispatch`, `TraverseWithIndexDispatch`. These
   combine the WithIndex pattern with the FnBrand parameter from the
   existing foldable/traversable dispatch traits.

4. **Complex group:** `WiltDispatch`, `WitherDispatch`. These follow
   the TraverseDispatch pattern with `FnBrand` + `M` parameters.

5. **Bifunctorial dispatch** (after ref-expansion plan):
   `BimapDispatch`, `BiFoldRightDispatch`, `BiFoldLeftDispatch`,
   `BiFoldMapDispatch`, `BiTraverseDispatch`. Added after
   RefBifunctor/RefBifoldable/RefBitraversable are available.
   All bi-\* dispatch traits use the closure-tuple pattern: the trait
   is implemented for `(F, G)`, enforcing that both closures agree on
   Val (both owned) or Ref (both borrowed). Calling convention is
   `bimap((f, g), p)`.

6. **Update `functions.rs`:** Add canonical dispatch exports and alias
   the non-dispatch free functions for each new dispatch trait.

7. **Update call sites:** Update all turbofish counts in source, doc
   comments, tests, and benchmarks.

8. **Tests.** Verify dispatch routing for each new trait (Val closure
   routes to by-value, Ref closure routes to by-reference). Property
   tests confirming Val and Ref paths produce identical results.

## Design Decisions

### Bimap dispatch: closure-tuple pattern

`bimap` has two closure parameters. The dispatch trait is implemented
for `(F, G)` as a unit, with exactly two impls:

```
// Val: both closures take owned args
impl BimapDispatch<..., FA, Val> for (F, G)
where F: Fn(A) -> B, G: Fn(C) -> D { ... }

// Ref: both closures take borrowed args
impl BimapDispatch<..., &FA, Ref> for (F, G)
where F: Fn(&A) -> B, G: Fn(&C) -> D { ... }
```

Mixed combinations (one owned, one borrowed) do not match either impl
and fail to compile. The calling convention is `bimap((f, g), p)`.

This pattern also applies to `bi_fold_*` and `bi_traverse`, which
similarly have two closures that must agree on Val/Ref. It follows the
precedent of `compose_kleisli`, which already dispatches on a closure
tuple `(f, g)`.

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
