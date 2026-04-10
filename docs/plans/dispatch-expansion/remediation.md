# Dispatch Expansion: Remediation Plan

Follow-up items identified by the implementation review (see
`analysis/review-1.md`, `analysis/review-2.md`, `analysis/review-3.md`).

None of these are blocking; the dispatch expansion is functionally
complete and all tests pass.

## Status

| Item                                       | Status                                                        |
| ------------------------------------------ | ------------------------------------------------------------- |
| 1. Routing tests                           | Open                                                          |
| 2. Migrate ref\_\* call sites              | Done (44 doc tests migrated to dispatch form)                 |
| 3. Hide ref\_\* from public API            | Done (exclusion feature added, 20 ref\_\* functions excluded) |
| 4. Fix traverse_with_index doc imports     | Done                                                          |
| 5. Fix plan turbofish table                | Done                                                          |
| 6. Exclude by-value non-dispatch functions | Done (21 by-value functions excluded)                         |

## 1. Add dedicated dispatch routing tests

**Issue:** Doc tests in each dispatch module verify basic compilation and
routing, but no dedicated `#[test]` functions verify routing behavior
across multiple types or edge cases (empty containers, None values).

**Action:** Add `#[test]` functions in `fp-library/src/classes/dispatch.rs`
(the existing test module) for each of the 19 new dispatch traits. Each
test should verify:

- Val closure routes to the by-value trait method.
- Ref closure routes to the by-ref trait method.
- At least two types per trait (e.g., Option + Vec for filterable,
  Result + Pair for bifunctorial).

Scope: ~40 new test functions.

## 2. Migrate remaining `ref_*` call sites to dispatch form

**Issue:** ~40+ `ref_*` free function call sites remain in type file doc
examples and tests. These are not bugs but are inconsistent with
migrated call sites.

**Action:** Replace all `ref_*` free function calls in type files with
the corresponding dispatch function call:

- `ref_bimap::<Brand, _, _, _, _>(f, g, &x)` becomes
  `bimap::<Brand, _, _, _, _, _, _>((f, g), &x)`
- `ref_filter_map::<Brand, _, _, _>(f, &x)` becomes
  `filter_map::<Brand, _, _, _, _>(f, &x)`
- `ref_traverse::<Brand, FnBrand, _, _, F>(f, &x)` becomes
  `traverse::<FnBrand, Brand, _, _, F, _, _>(f, &x)`

Files affected: result.rs, pair.rs, tuple_2.rs, control_flow.rs,
identity.rs, option.rs, vec.rs, cat_list.rs.

Scope: ~40 call sites across ~8 files. Mechanical.

## 3. Hide `ref_*` free functions from public API

**Status: Done.**

Added exclusion support to `generate_function_re_exports!` macro
(commit `e3353bc`). 20 `ref_*` free functions are now excluded from
`functions::*` re-export. They remain in their trait modules as
dispatch targets.

## 4. Fix `traverse_with_index.rs` doc example imports

**Issue:** Doc examples in `traverse_with_index.rs` import from
`fp_library::classes::dispatch::traverse_with_index` instead of
`fp_library::functions::*`, inconsistent with all other dispatch files.

**Action:** Update the doc examples to use `use fp_library::functions::*;`.

Scope: 1 file, ~3 doc examples.

## 5. Fix plan turbofish table

**Issue:** The dispatch-expansion plan's turbofish table lists
`traverse_with_index` dispatch as 8 type params; the actual
implementation has 7 (`FnBrand, Brand, A, B, F, FTA, Marker`).

**Action:** Update the table entry in `docs/plans/dispatch-expansion/plan.md`
from 8 to 7.

Scope: 1 line in plan.md.

## 6. Exclude by-value non-dispatch free functions from public API

**Issue:** The by-value non-dispatch free functions (e.g.,
`filterable::filter`, `bifunctor::bimap`, `witherable::wilt`) are
currently aliased to long names (`filterable_filter`,
`bifunctor_bimap`, etc.) in `functions.rs`. These aliases exist only
to avoid name conflicts with the dispatch versions, but they clutter
the public API surface with functions nobody should call directly.

The `ref_*` functions were already excluded (item 3). The by-value
non-dispatch functions are equally superseded by dispatch and should
receive the same treatment for consistency.

**Action:** Move the by-value non-dispatch aliases from the alias map
to the exclusion list. The functions remain in their trait modules
(needed as dispatch targets and for trait doc examples) but are not
re-exported via `functions::*`.

Functions to exclude:

- `bifunctor::bimap`
- `bifoldable::bi_fold_left`
- `bifoldable::bi_fold_map`
- `bifoldable::bi_fold_right`
- `bitraversable::bi_traverse`
- `filterable::filter`
- `filterable::filter_map`
- `filterable::partition`
- `filterable::partition_map`
- `filterable_with_index::filter_map_with_index`
- `filterable_with_index::filter_with_index`
- `filterable_with_index::partition_map_with_index`
- `filterable_with_index::partition_with_index`
- `foldable_with_index::fold_left_with_index`
- `foldable_with_index::fold_map_with_index`
- `foldable_with_index::fold_right_with_index`
- `functor_with_index::map_with_index`
- `traversable::traverse`
- `traversable_with_index::traverse_with_index`
- `witherable::wilt`
- `witherable::wither`

Note: the pre-existing aliases for `filterable::filter_map` and
`traversable::traverse` (which predated this plan) should also move
to the exclusion list.

Scope: ~21 entries moved from alias map to exclusion list in
`functions.rs`.

## Implementation Order

Items are independent and can be done in any order or in parallel.

1. Fix plan table (item 5) and doc imports (item 4) - trivial.
2. Exclude by-value non-dispatch functions (item 6) - low effort.
3. Migrate ref\_\* call sites (item 2) - medium effort, mechanical.
4. Add routing tests (item 1) - medium effort.
