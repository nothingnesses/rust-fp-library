# Dispatch Expansion: Remediation Plan

Follow-up items identified by the implementation review (see
`analysis/review-1.md`, `analysis/review-2.md`, `analysis/review-3.md`).

None of these are blocking; the dispatch expansion is functionally
complete and all tests pass.

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

Scope: ~40 new test functions. Mechanical; follow the existing test
patterns in `dispatch.rs` (tests for map, bind, lift2, filter_map,
traverse are already there).

## 2. Migrate remaining `ref_*` call sites to dispatch form

**Issue:** ~40+ `ref_*` free function call sites remain in type file doc
examples and tests (e.g., `ref_bimap::<ResultBrand, ...>(f, g, &x)`
instead of `bimap::<ResultBrand, ...>((f, g), &x)`). These are not
bugs but are inconsistent with migrated call sites.

**Action:** Replace all `ref_*` free function calls in type files with
the corresponding dispatch function call. The transformation is:

- `ref_bimap::<Brand, A, B, C, D>(f, g, &x)` becomes
  `bimap::<Brand, A, B, C, D, _, _>((|a: &_| f(a), |c: &_| g(c)), &x)`

Actually, since the dispatch function infers the Ref marker from `&x`
and the closure arg type, the simpler transformation is:

- `ref_bimap::<Brand, _, _, _, _>(f, g, &x)` becomes
  `bimap::<Brand, _, _, _, _, _, _>((f, g), &x)` (for bi-\* functions)
- `ref_filter_map::<Brand, _, _, _>(f, &x)` becomes
  `filter_map::<Brand, _, _, _, _>(f, &x)` (for single-closure functions)
- `ref_traverse::<Brand, FnBrand, _, _, F>(f, &x)` becomes
  `traverse::<FnBrand, Brand, _, _, F, _, _>(f, &x)`

Files affected (non-exhaustive): result.rs, pair.rs, tuple_2.rs,
control_flow.rs, identity.rs, option.rs, vec.rs, cat_list.rs.

Scope: ~40 call sites across ~8 files. Mechanical.

## 3. Hide `ref_*` free functions from public API

**Issue:** `ref_*` free functions that are fully superseded by dispatch
versions are still publicly exported. The plan says they should be
removed from the public API.

**Action:** Add exclusion entries to `generate_function_re_exports!` in
`functions.rs` for all `ref_*` free functions whose functionality is
covered by a dispatch function. This includes:

Bifunctorial:

- `ref_bifunctor::ref_bimap`
- `ref_bifoldable::ref_bi_fold_right`
- `ref_bifoldable::ref_bi_fold_left`
- `ref_bifoldable::ref_bi_fold_map`
- `ref_bitraversable::ref_bi_traverse`

Non-bifunctorial (already not re-exported, but verify):

- `ref_filterable::ref_filter`
- `ref_filterable::ref_filter_map`
- `ref_filterable::ref_partition`
- `ref_filterable::ref_partition_map`
- `ref_functor_with_index::ref_map_with_index`
- `ref_filterable_with_index::ref_filter_with_index`
- `ref_filterable_with_index::ref_filter_map_with_index`
- `ref_filterable_with_index::ref_partition_with_index`
- `ref_filterable_with_index::ref_partition_map_with_index`
- `ref_foldable_with_index::ref_fold_map_with_index`
- `ref_foldable_with_index::ref_fold_right_with_index`
- `ref_foldable_with_index::ref_fold_left_with_index`
- `ref_traversable_with_index::ref_traverse_with_index`
- `ref_witherable::ref_wilt`
- `ref_witherable::ref_wither`

The free functions themselves remain in their trait modules (needed as
dispatch targets and for trait doc examples) but are not re-exported
in `functions.rs`.

Note: check whether the `generate_function_re_exports!` macro supports
exclusion (suppressing auto-export without providing an alias). If not,
alias them with long module-prefixed names (the existing approach) and
consider adding `#[doc(hidden)]` to the aliased re-exports.

## 4. Fix `traverse_with_index.rs` doc example imports

**Issue:** Doc examples in `traverse_with_index.rs` import from
`fp_library::classes::dispatch::traverse_with_index` instead of
`fp_library::functions::*`, inconsistent with all other dispatch files.

**Action:** Update the doc examples to use `use fp_library::functions::*;`
and `use fp_library::brands::*;`.

Scope: 1 file, ~3 doc examples.

## 5. Fix plan turbofish table

**Issue:** The dispatch-expansion plan's turbofish table lists
`traverse_with_index` dispatch as 8 type params; the actual
implementation has 7 (`FnBrand, Brand, A, B, F, FTA, Marker`).

**Action:** Update the table entry in `docs/plans/dispatch-expansion/plan.md`
from 8 to 7, and update the delta from +3 to +3 (delta is correct;
original was 4, dispatch is 7, so +3).

Scope: 1 line in plan.md.

## Implementation Order

Items are independent and can be done in any order or in parallel.

1. Fix plan table (item 5) - trivial, do first.
2. Fix doc example imports (item 4) - trivial.
3. Add routing tests (item 1) - medium effort.
4. Migrate ref\_\* call sites (item 2) - medium effort, mechanical.
5. Hide ref*\* from public API (item 3) - low effort, depends on item 2
   (call sites should use dispatch form before hiding the ref*\* exports).
