# Dispatch Expansion: Implementation Review 1

Reviewing implementation against every item in the dispatch-expansion plan.
Base commit: `269712aa5a38527ef81dec8e6742601104a0976c`.

## Step-by-Step Compliance

### Step 0: Fix `ref_fold_left_with_index` argument order

**Status: Done (pre-existing)**

The signature in `RefFoldableWithIndex::ref_fold_left_with_index` already uses
`Fn(Self::Index, B, &A) -> B` at the base commit. No changes were needed in
this diff range.

`SendRefFoldableWithIndex::send_ref_fold_left_with_index` also already uses
the correct ordering. No diff to either file.

### Step 0b: Rename existing dispatch methods to bare `dispatch`

**Status: Done**

All qualified method names were renamed:

- `dispatch_filter_map` -> `dispatch` in `filterable.rs` (trait def, 2 impls, free function call site).
- `dispatch_bind` -> `dispatch` in `semimonad.rs` (trait def, 2 impls, 2 free function call sites for `bind` and `bind_flipped`).
- `dispatch_traverse` -> `dispatch` in `traversable.rs` (trait def, 2 impls, free function call site).
- `dispatch_lift2` through `dispatch_lift5` -> `dispatch` in `lift.rs` (4 trait defs, 8 impls, 4 free function call sites).

All call sites within the dispatch free functions updated accordingly.

### Step 1: Simple group (FilterDispatch, PartitionDispatch, PartitionMapDispatch)

**Status: Done**

All three dispatch traits implemented in new files:

- `fp-library/src/classes/dispatch/filter.rs` - `FilterDispatch` with Val and Ref impls.
- `fp-library/src/classes/dispatch/partition.rs` - `PartitionDispatch` with Val and Ref impls.
- `fp-library/src/classes/dispatch/partition_map.rs` - `PartitionMapDispatch` with Val and Ref impls.

Each has a unified free function that calls `self.dispatch(fa)`.

**Clone bounds:** `FilterDispatch` and `PartitionDispatch` correctly include
`A: 'a + Clone` in their trait bounds. `PartitionMapDispatch` correctly uses
`A: 'a` without `Clone`. This matches the plan.

**Return types:** Partition dispatch returns a tuple
`(Brand::Of<'a, A>, Brand::Of<'a, A>)` as specified. PartitionMap returns
`(Brand::Of<'a, E>, Brand::Of<'a, O>)`.

### Step 2: WithIndex group (filterable/functor) - 5 dispatch traits

**Status: Done**

All five dispatch traits implemented:

- `fp-library/src/classes/dispatch/map_with_index.rs` - `MapWithIndexDispatch`
- `fp-library/src/classes/dispatch/filter_with_index.rs` - `FilterWithIndexDispatch`
- `fp-library/src/classes/dispatch/filter_map_with_index.rs` - `FilterMapWithIndexDispatch`
- `fp-library/src/classes/dispatch/partition_with_index.rs` - `PartitionWithIndexDispatch`
- `fp-library/src/classes/dispatch/partition_map_with_index.rs` - `PartitionMapWithIndexDispatch`

All use `Brand: Kind_cdc7cd43dac7585f + WithIndex` bound and project
`Brand::Index` in closure types, matching the plan. Index does not add a
turbofish position.

`FilterWithIndexDispatch` and `PartitionWithIndexDispatch` have `A: Clone`.
`FilterMapWithIndexDispatch` and `PartitionMapWithIndexDispatch` do not.
This matches the underlying trait semantics.

### Step 3: WithIndex group (foldable/traversable) - 4 dispatch traits

**Status: Done**

All four dispatch traits implemented:

- `fp-library/src/classes/dispatch/fold_map_with_index.rs` - `FoldMapWithIndexDispatch`
- `fp-library/src/classes/dispatch/fold_right_with_index.rs` - `FoldRightWithIndexDispatch`
- `fp-library/src/classes/dispatch/fold_left_with_index.rs` - `FoldLeftWithIndexDispatch`
- `fp-library/src/classes/dispatch/traverse_with_index.rs` - `TraverseWithIndexDispatch`

All include `FnBrand` as a type parameter (unused in Val, used in Ref for
`LiftFn` trait bounds). All use `Brand: WithIndex` bound.

### Step 4: WiltDispatch, WitherDispatch

**Status: Done**

Both dispatch traits implemented:

- `fp-library/src/classes/dispatch/wilt.rs` - `WiltDispatch`
- `fp-library/src/classes/dispatch/wither.rs` - `WitherDispatch`

Both follow the TraverseDispatch pattern with `FnBrand` + `M` (applicative
brand) parameters. `WiltDispatch` has `A, E, O` type params for the
`Result<O, E>` return. `WitherDispatch` has `A, B` type params for the
`Option<B>` return.

### Step 5: Bifunctorial dispatch - 5 traits with closure-tuple pattern

**Status: Done**

All five bifunctorial dispatch traits implemented:

- `fp-library/src/classes/dispatch/bimap.rs` - `BimapDispatch`
- `fp-library/src/classes/dispatch/bi_fold_right.rs` - `BiFoldRightDispatch`
- `fp-library/src/classes/dispatch/bi_fold_left.rs` - `BiFoldLeftDispatch`
- `fp-library/src/classes/dispatch/bi_fold_map.rs` - `BiFoldMapDispatch`
- `fp-library/src/classes/dispatch/bi_traverse.rs` - `BiTraverseDispatch`

**Closure-tuple pattern:** All are implemented for `(F, G)` with two impls
(Val and Ref). Val impl requires both closures to take owned args; Ref impl
requires both to take references. Mixed combinations fail to compile. Calling
convention is `bimap((f, g), p)` as specified.

**Kind hash:** All bifunctorial traits use `Kind_266801a817966495` (two-param
Kind) as specified.

**FnBrand:** `BiFoldRightDispatch`, `BiFoldLeftDispatch`, `BiFoldMapDispatch`,
and `BiTraverseDispatch` include `FnBrand` as a type parameter. `BimapDispatch`
does not (it does not need one, matching `FunctorDispatch`).

### Step 6: functions.rs updates

**Status: Done**

All 20 new dispatch free functions are re-exported in `functions.rs` via the
`pub use crate::classes::dispatch::{...}` block.

The non-dispatch free functions are aliased via `generate_function_re_exports!`:

- `filterable::filter` -> `filterable_filter`
- `filterable::partition` -> `filterable_partition`
- `filterable::partition_map` -> `filterable_partition_map`
- `filterable_with_index::filter_with_index` -> `filterable_with_index_filter_with_index`
- `filterable_with_index::filter_map_with_index` -> `filterable_with_index_filter_map_with_index`
- `filterable_with_index::partition_with_index` -> `filterable_with_index_partition_with_index`
- `filterable_with_index::partition_map_with_index` -> `filterable_with_index_partition_map_with_index`
- `functor_with_index::map_with_index` -> `functor_with_index_map_with_index`
- `foldable_with_index::fold_left_with_index` -> `foldable_with_index_fold_left_with_index`
- `foldable_with_index::fold_map_with_index` -> `foldable_with_index_fold_map_with_index`
- `foldable_with_index::fold_right_with_index` -> `foldable_with_index_fold_right_with_index`
- `traversable_with_index::traverse_with_index` -> `traversable_with_index_traverse_with_index`
- `witherable::wilt` -> `witherable_wilt`
- `witherable::wither` -> `witherable_wither`
- `bifunctor::bimap` -> `bifunctor_bimap`
- `bifoldable::bi_fold_left` -> `bifoldable_bi_fold_left`
- `bifoldable::bi_fold_map` -> `bifoldable_bi_fold_map`
- `bifoldable::bi_fold_right` -> `bifoldable_bi_fold_right`
- `bitraversable::bi_traverse` -> `bitraversable_bi_traverse`

### Step 7: Call site turbofish updates

**Status: Done**

All turbofish counts updated across source files, doc comments, tests, and
benchmarks. The diff shows 50 files changed with turbofish adjustments in:

- `fp-library/src/classes/filterable.rs` (filter, partition, partition_map)
- `fp-library/src/classes/filterable_with_index.rs`
- `fp-library/src/classes/foldable_with_index.rs`
- `fp-library/src/classes/functor_with_index.rs`
- `fp-library/src/classes/traversable_with_index.rs`
- `fp-library/src/classes/witherable.rs`
- `fp-library/src/classes/bifunctor.rs`
- `fp-library/src/classes/bifoldable.rs`
- `fp-library/src/classes/bitraversable.rs`
- `fp-library/src/classes/ref_bifunctor.rs`
- `fp-library/src/classes/ref_bifoldable.rs`
- `fp-library/src/classes/ref_bitraversable.rs`
- `fp-library/src/types/` (option, vec, result, pair, tuple_2, identity, cat_list, control_flow, try_thunk)
- `fp-library/benches/benchmarks/` (option, vec, cat_list)

### Step 8: Tests

**Status: Partially done**

Doc tests exist in all 20 new dispatch modules, demonstrating both Val and Ref
dispatch paths for at least one type (typically `Option` or `Vec` for Val, and
`Lazy` or borrowed `Vec` for Ref). These serve as compile-time verification
that dispatch routing works.

However, the plan calls for dedicated routing tests ("Verify dispatch routing
for each new trait") and property tests ("Val and Ref paths produce identical
results"). No new `#[test]` functions or property tests were added. The
existing tests in `dispatch.rs` were not extended for the new traits.

All existing tests pass (1154 tests, cached output confirms ok).

## Design Decisions Compliance

### Method naming (bare `dispatch`)

**Status: Done**

All new dispatch traits use bare `dispatch`. All existing dispatch traits
renamed from qualified names to bare `dispatch` (Step 0b).

### A: Clone bounds

**Status: Done**

`FilterDispatch` and `PartitionDispatch` have `A: Clone`.
`FilterWithIndexDispatch` and `PartitionWithIndexDispatch` have `A: Clone`.
`PartitionMapDispatch`, `FilterMapDispatch`, and their WithIndex variants
do not have `A: Clone`. This matches the plan.

### Kind hash for bifunctorial dispatch

**Status: Done**

All bifunctorial dispatch traits use `Kind_266801a817966495` (two-param Kind).

### ref\_\* function removal

**Status: Partially done**

The ref\_\* free functions still exist in their respective trait modules
(e.g., `ref_filter` in `ref_filterable.rs`, `ref_wilt` in `ref_witherable.rs`).
This is acceptable since they are needed as dispatch targets and for trait
doc examples.

However, the non-bifunctorial ref*\* functions are correctly NOT re-exported
in `functions.rs`. The bifunctorial ref*\_ functions ARE still re-exported
under aliased names (`ref_bifoldable_ref_bi_fold_left`, etc.). This is an
inconsistency; the plan says ref\_\_ free functions should be removed from
the public API when dispatch unifies them.

Additionally, the plan notes that `ref_filter_map` and `ref_traverse` were
"inconsistently kept" when their dispatch versions were added, and should be
cleaned up. These functions still exist in their trait modules but are not
re-exported in `functions.rs`, so they are effectively internal.

### Partition return types

**Status: Done**

Both Val and Ref paths return identical owned tuple types.

### E0119 safety

**Status: Done**

All dispatch traits use the two-impl pattern distinguishing on closure argument
type (`A` vs `&A`) and container type (owned vs borrowed). No coherence issues.

## Deviations from the Plan

### 1. Bimap calling convention change in non-dispatch free function

The non-dispatch `bimap` free function in `bifunctor.rs` retains its original
signature `bimap(f, g, p)` with separate closure arguments. The dispatch
version uses `bimap((f, g), p)` (tuple). All doc tests and call sites were
updated to use the dispatch version's tuple convention. The non-dispatch
function is aliased as `bifunctor_bimap` and still takes separate args.

Similarly, the non-dispatch `bi_fold_left`, `bi_fold_right`, `bi_fold_map`,
and `bi_traverse` free functions retain separate closure arguments, while
their dispatch versions use closure tuples. Call sites in doc examples were
updated to use the dispatch calling convention.

This is not strictly a deviation, as the plan specifies the tuple pattern
for the dispatch traits. The non-dispatch functions are internal targets and
were correctly left unchanged.

### 2. traverse_with_index turbofish count

The plan's turbofish table says traverse_with_index dispatch should have 8
params. The actual implementation has 7 non-lifetime type params:
`FnBrand, Brand, A, B, F, FTA, Marker`. The plan likely miscounted, as the
delta of +3 from the original (4 params) is correct and consistent with other
FnBrand-carrying dispatch traits. The implementation is internally consistent.

### 3. Bifunctorial ref\_\* re-exports still present

The `ref_bimap`, `ref_bi_fold_left`, `ref_bi_fold_map`, `ref_bi_fold_right`,
and `ref_bi_traverse` free functions are still re-exported under aliased names
in `functions.rs` via `generate_function_re_exports!`. These should either be
removed from re-export or excluded from the macro scan, since dispatch now
covers their functionality.

## Items Not Implemented

### 1. Dedicated routing tests (Step 8)

No new `#[test]` functions were added to verify dispatch routing for the new
traits. The doc tests provide basic coverage, but the plan calls for:

- Dedicated routing tests verifying Val closure routes to by-value path.
- Dedicated routing tests verifying Ref closure routes to by-reference path.
- Property tests confirming Val and Ref paths produce identical results.

### 2. Cleanup of ref_filter_map and ref_traverse free functions

The plan mentions these were "inconsistently kept" and should be cleaned up.
They still exist as free functions in `ref_filterable.rs` and
`ref_traversable.rs` respectively, but they are not re-exported.

### 3. Removal of bifunctorial ref\_\* re-exports

The 5 bifunctorial ref\_\* free functions remain re-exported (under aliased
names) in `functions.rs`.

## Recommendations

### Issue 1: Missing dedicated tests

**Approaches:**

A. Add `#[test]` functions in `dispatch.rs` for each new dispatch trait,
testing both Val and Ref paths with at least two types (e.g., Option, Vec).
~40 new test functions.

B. Add property-based tests (QuickCheck) verifying that dispatch produces
the same results as direct trait method calls.

C. Rely on the existing doc tests as sufficient coverage.

**Recommendation:** Option A. The doc tests verify compilation and basic
routing, but dedicated tests would catch edge cases (e.g., empty containers,
None values) and serve as regression tests. Property tests (B) are a
nice-to-have but lower priority.

### Issue 2: Bifunctorial ref\_\* re-exports

**Approaches:**

A. Add exclusion entries to `generate_function_re_exports!` for the 5
bifunctorial ref\_\* functions.

B. Remove the ref\_\* free functions from the ref trait modules entirely
(breaking if any external code uses them directly).

C. Leave as-is; the aliased names are unlikely to cause confusion since
they are namespaced (e.g., `ref_bifoldable_ref_bi_fold_left`).

**Recommendation:** Option A. Adding exclusion entries is low-risk and
consistent with how non-bifunctorial ref\_\* functions are handled.

### Issue 3: ref_filter_map and ref_traverse cleanup

**Approaches:**

A. Remove the free functions from `ref_filterable.rs` and `ref_traversable.rs`.
Update any doc examples that call them to use the dispatch version.

B. Leave them as internal functions, since they are not re-exported.

**Recommendation:** Option B for now. They are not part of the public API
and serve as useful dispatch targets. Removing them requires updating
doc examples and could break internal usage patterns.

## Summary

The implementation is comprehensive and closely follows the plan. All 20
dispatch traits are implemented (Steps 1-5), method names are unified to bare
`dispatch` (Step 0b), `functions.rs` is properly updated (Step 6), and all
turbofish counts are updated across the codebase (Step 7). The prerequisite
argument order fix (Step 0) was already in place. The code compiles and all
tests pass.

The main gaps are: (1) no dedicated routing or property tests beyond doc tests,
(2) bifunctorial ref\_\* functions still re-exported under aliased names, and
(3) minor plan discrepancy in the traverse_with_index turbofish count table.
None of these are blocking issues.
