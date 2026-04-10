# Dispatch Expansion: Implementation Review 3

Technical review of all 19 newly added dispatch files, the dispatch module
registration, the `functions.rs` re-exports, and the turbofish updates in
type files.

## Summary

The implementation is structurally sound. All 19 dispatch traits follow the
established two-impl pattern correctly, with no E0119 risks. Trait bound
correctness, delegation correctness, and Clone bound placement are all
correct across the board. FnBrand handling in traversal-family dispatches
matches the established precedent. Documentation attributes are
comprehensively applied. However, several issues were found:

1. **Stale `ref_*` free functions still exported with canonical names** (medium severity).
2. **Numerous un-migrated `ref_*` call sites remain in type files** (low severity, expected per plan phasing).
3. **`bi_fold_left`/`bi_fold_right`/`bi_fold_map` Ref impls have `A: Clone + B: Clone` but the underlying `Bifoldable` trait also requires these** (not a bug, but the Clone bounds look surprising for fold operations; they are inherited from the trait definition, not the dispatch layer's doing).

---

## 1. E0119 safety

All 19 dispatch traits are E0119-safe. Each trait has exactly two impls
distinguished on two independent axes:

- **FA type**: `Brand::Of<'a, A>` (Val) vs `&'b Brand::Of<'a, A>` (Ref).
- **Closure arg type**: `Fn(A)` (Val) vs `Fn(&A)` (Ref).
- **Marker type**: `Val` vs `Ref`.

For bifunctorial tuple dispatches (bimap, bi_fold_left, bi_fold_right,
bi_fold_map, bi_traverse), both closures in the tuple are constrained in
the same impl, preventing mixed Val/Ref. The Self type is `(F, G)` for
both impls, with the Marker and FA parameters providing structural
distinction.

**Verdict: No E0119 risk in any dispatch trait.**

---

## 2. Trait bound correctness

### 2a. Val impls

All Val impls bind on the correct by-value trait:

| Dispatch file               | Val bounds on          | Correct |
| --------------------------- | ---------------------- | ------- |
| filter.rs                   | `Filterable`           | Yes     |
| partition.rs                | `Filterable`           | Yes     |
| partition_map.rs            | `Filterable`           | Yes     |
| filter_with_index.rs        | `FilterableWithIndex`  | Yes     |
| filter_map_with_index.rs    | `FilterableWithIndex`  | Yes     |
| partition_with_index.rs     | `FilterableWithIndex`  | Yes     |
| partition_map_with_index.rs | `FilterableWithIndex`  | Yes     |
| map_with_index.rs           | `FunctorWithIndex`     | Yes     |
| fold_left_with_index.rs     | `FoldableWithIndex`    | Yes     |
| fold_right_with_index.rs    | `FoldableWithIndex`    | Yes     |
| fold_map_with_index.rs      | `FoldableWithIndex`    | Yes     |
| traverse_with_index.rs      | `TraversableWithIndex` | Yes     |
| wilt.rs                     | `Witherable`           | Yes     |
| wither.rs                   | `Witherable`           | Yes     |
| bimap.rs                    | `Bifunctor`            | Yes     |
| bi_fold_left.rs             | `Bifoldable`           | Yes     |
| bi_fold_right.rs            | `Bifoldable`           | Yes     |
| bi_fold_map.rs              | `Bifoldable`           | Yes     |
| bi_traverse.rs              | `Bitraversable`        | Yes     |

### 2b. Ref impls

All Ref impls bind on the correct Ref trait:

| Dispatch file               | Ref bounds on             | Correct |
| --------------------------- | ------------------------- | ------- |
| filter.rs                   | `RefFilterable`           | Yes     |
| partition.rs                | `RefFilterable`           | Yes     |
| partition_map.rs            | `RefFilterable`           | Yes     |
| filter_with_index.rs        | `RefFilterableWithIndex`  | Yes     |
| filter_map_with_index.rs    | `RefFilterableWithIndex`  | Yes     |
| partition_with_index.rs     | `RefFilterableWithIndex`  | Yes     |
| partition_map_with_index.rs | `RefFilterableWithIndex`  | Yes     |
| map_with_index.rs           | `RefFunctorWithIndex`     | Yes     |
| fold_left_with_index.rs     | `RefFoldableWithIndex`    | Yes     |
| fold_right_with_index.rs    | `RefFoldableWithIndex`    | Yes     |
| fold_map_with_index.rs      | `RefFoldableWithIndex`    | Yes     |
| traverse_with_index.rs      | `RefTraversableWithIndex` | Yes     |
| wilt.rs                     | `RefWitherable`           | Yes     |
| wither.rs                   | `RefWitherable`           | Yes     |
| bimap.rs                    | `RefBifunctor`            | Yes     |
| bi_fold_left.rs             | `RefBifoldable`           | Yes     |
| bi_fold_right.rs            | `RefBifoldable`           | Yes     |
| bi_fold_map.rs              | `RefBifoldable`           | Yes     |
| bi_traverse.rs              | `RefBitraversable`        | Yes     |

**Verdict: All trait bounds are correct.**

---

## 3. Delegation correctness

### 3a. Val bodies

All Val impls delegate to the correct by-value trait method:

- `filter.rs` Val -> `Brand::filter(self, fa)` -- correct.
- `partition.rs` Val -> `Brand::partition(self, fa)` -- correct.
- `partition_map.rs` Val -> `Brand::partition_map(self, fa)` -- correct.
- `filter_with_index.rs` Val -> `Brand::filter_with_index(self, fa)` -- correct.
- `filter_map_with_index.rs` Val -> `Brand::filter_map_with_index(self, fa)` -- correct.
- `partition_with_index.rs` Val -> `Brand::partition_with_index(self, fa)` -- correct.
- `partition_map_with_index.rs` Val -> `Brand::partition_map_with_index(self, fa)` -- correct.
- `map_with_index.rs` Val -> `Brand::map_with_index(self, fa)` -- correct.
- `fold_left_with_index.rs` Val -> `Brand::fold_left_with_index::<FnBrand, A, B>(self, initial, fa)` -- correct.
- `fold_right_with_index.rs` Val -> `Brand::fold_right_with_index::<FnBrand, A, B>(self, initial, fa)` -- correct.
- `fold_map_with_index.rs` Val -> `Brand::fold_map_with_index::<FnBrand, A, M>(self, fa)` -- correct.
- `traverse_with_index.rs` Val -> `Brand::traverse_with_index::<A, B, F>(self, ta)` -- correct.
- `wilt.rs` Val -> `Brand::wilt::<M, A, E, O>(self, ta)` -- correct.
- `wither.rs` Val -> `Brand::wither::<M, A, B>(self, ta)` -- correct.
- `bimap.rs` Val -> `Brand::bimap(self.0, self.1, fa)` -- correct.
- `bi_fold_left.rs` Val -> `Brand::bi_fold_left::<FnBrand, A, B, C>(self.0, self.1, z, fa)` -- correct.
- `bi_fold_right.rs` Val -> `Brand::bi_fold_right::<FnBrand, A, B, C>(self.0, self.1, z, fa)` -- correct.
- `bi_fold_map.rs` Val -> `Brand::bi_fold_map::<FnBrand, A, B, M>(self.0, self.1, fa)` -- correct.
- `bi_traverse.rs` Val -> `Brand::bi_traverse::<A, B, C, D, F>(self.0, self.1, fa)` -- correct.

### 3b. Ref bodies

All Ref impls delegate to the correct `ref_*` method:

- `filter.rs` Ref -> `Brand::ref_filter(self, fa)` -- correct.
- `partition.rs` Ref -> `Brand::ref_partition(self, fa)` -- correct.
- `partition_map.rs` Ref -> `Brand::ref_partition_map(self, fa)` -- correct.
- `filter_with_index.rs` Ref -> `Brand::ref_filter_with_index(self, fa)` -- correct.
- `filter_map_with_index.rs` Ref -> `Brand::ref_filter_map_with_index(self, fa)` -- correct.
- `partition_with_index.rs` Ref -> `Brand::ref_partition_with_index(self, fa)` -- correct.
- `partition_map_with_index.rs` Ref -> `Brand::ref_partition_map_with_index(self, fa)` -- correct.
- `map_with_index.rs` Ref -> `Brand::ref_map_with_index(self, fa)` -- correct.
- `fold_left_with_index.rs` Ref -> `Brand::ref_fold_left_with_index::<FnBrand, A, B>(self, initial, fa)` -- correct.
- `fold_right_with_index.rs` Ref -> `Brand::ref_fold_right_with_index::<FnBrand, A, B>(self, initial, fa)` -- correct.
- `fold_map_with_index.rs` Ref -> `Brand::ref_fold_map_with_index::<FnBrand, A, M>(self, fa)` -- correct.
- `traverse_with_index.rs` Ref -> `Brand::ref_traverse_with_index::<A, B, F>(self, ta)` -- correct.
- `wilt.rs` Ref -> `Brand::ref_wilt::<FnBrand, M, A, E, O>(self, ta)` -- correct.
- `wither.rs` Ref -> `Brand::ref_wither::<FnBrand, M, A, B>(self, ta)` -- correct.
- `bimap.rs` Ref -> `Brand::ref_bimap(self.0, self.1, fa)` -- correct.
- `bi_fold_left.rs` Ref -> `Brand::ref_bi_fold_left::<FnBrand, A, B, C>(self.0, self.1, z, fa)` -- correct.
- `bi_fold_right.rs` Ref -> `Brand::ref_bi_fold_right::<FnBrand, A, B, C>(self.0, self.1, z, fa)` -- correct.
- `bi_fold_map.rs` Ref -> `Brand::ref_bi_fold_map::<FnBrand, A, B, M>(self.0, self.1, fa)` -- correct.
- `bi_traverse.rs` Ref -> `Brand::ref_bi_traverse::<FnBrand, A, B, C, D, F>(self.0, self.1, fa)` -- correct.

**Verdict: All delegation is correct.**

---

## 4. Clone bounds

### 4a. Where Clone is required (filter, partition operations)

The underlying traits require `A: Clone` on:

- `Filterable::filter` -- `A: Clone` (value must be cloned for predicate check).
- `Filterable::partition` -- `A: Clone`.
- `RefFilterable::ref_filter` -- `A: Clone`.
- `RefFilterable::ref_partition` -- `A: Clone`.
- `FilterableWithIndex::filter_with_index` -- `A: Clone`.
- `FilterableWithIndex::partition_with_index` -- `A: Clone`.

The dispatch traits correctly include `A: Clone` in the trait definition for:

- `FilterDispatch` -- `A: 'a + Clone` in trait def. Correct.
- `PartitionDispatch` -- `A: 'a + Clone` in trait def. Correct.
- `FilterWithIndexDispatch` -- `A: 'a + Clone` in trait def. Correct.
- `PartitionWithIndexDispatch` -- `A: 'a + Clone` in trait def. Correct.

### 4b. Where Clone is NOT required (filter_map, partition_map operations)

The underlying traits do NOT require `A: Clone` on:

- `Filterable::filter_map` -- `A: 'a` only.
- `Filterable::partition_map` -- `A: 'a` only.
- `FilterableWithIndex::filter_map_with_index` -- `A: 'a` only.
- `FilterableWithIndex::partition_map_with_index` -- `A: 'a` only.

The dispatch traits correctly omit `A: Clone`:

- `FilterMapDispatch` (in filterable.rs) -- `A: 'a`. Correct.
- `PartitionMapDispatch` -- `A: 'a`. Correct.
- `FilterMapWithIndexDispatch` -- `A: 'a`. Correct.
- `PartitionMapWithIndexDispatch` -- `A: 'a`. Correct.

**Verdict: Clone bounds are correctly placed where needed and omitted where not needed.**

---

## 5. FnBrand handling

### 5a. Dispatch traits that include FnBrand

The following dispatch traits include `FnBrand` as a type parameter:

- `fold_left_with_index.rs` -- FnBrand present. Val impl has `FnBrand: LiftFn + 'a` and passes it through to `Brand::fold_left_with_index::<FnBrand, A, B>`. Ref impl also passes it. Both paths use it. Correct.
- `fold_right_with_index.rs` -- Same pattern as fold_left. Correct.
- `fold_map_with_index.rs` -- Same pattern. Correct.
- `traverse_with_index.rs` -- FnBrand present. Val impl has `FnBrand: LiftFn + 'a` but the body calls `Brand::traverse_with_index::<A, B, F>` without FnBrand. This matches the `TraverseDispatch` precedent where FnBrand is accepted for API uniformity but unused by the Val path. Ref impl bounds `FnBrand: LiftFn + 'a` and the body calls `Brand::ref_traverse_with_index::<A, B, F>` (also without FnBrand). Correct.
- `wilt.rs` -- FnBrand present. Val body calls `Brand::wilt::<M, A, E, O>` (no FnBrand). Ref body calls `Brand::ref_wilt::<FnBrand, M, A, E, O>` (passes FnBrand). Matches the wilt trait signatures. Correct.
- `wither.rs` -- Same pattern as wilt. Val doesn't pass FnBrand, Ref passes it. Correct.
- `bi_fold_left.rs` -- FnBrand present. Both Val and Ref pass it. Correct.
- `bi_fold_right.rs` -- Same. Correct.
- `bi_fold_map.rs` -- Same. Correct.
- `bi_traverse.rs` -- FnBrand present. Val body calls `Brand::bi_traverse::<A, B, C, D, F>` (no FnBrand). Ref body calls `Brand::ref_bi_traverse::<FnBrand, A, B, C, D, F>` (passes FnBrand). Matches the Bitraversable/RefBitraversable trait signatures. Correct.

### 5b. Dispatch traits that omit FnBrand

The filterable-family and functor-with-index dispatches correctly omit FnBrand, since their underlying traits do not use it:

- `filter.rs`, `partition.rs`, `partition_map.rs` -- No FnBrand. Correct.
- `filter_with_index.rs`, `filter_map_with_index.rs`, `partition_with_index.rs`, `partition_map_with_index.rs` -- No FnBrand. Correct.
- `map_with_index.rs` -- No FnBrand. Correct.
- `bimap.rs` -- No FnBrand. Correct.

**Verdict: FnBrand handling is correct throughout, matching the TraverseDispatch precedent.**

---

## 6. Bifunctorial tuple pattern

The 5 bi-\* dispatch traits (bimap, bi_fold_left, bi_fold_right, bi_fold_map,
bi_traverse) all use `(F, G)` as the Self type for both impls.

### Checks:

- **Self type is `(F, G)`**: Yes, all 5 traits implement for `(F, G)` (or `(Func1, Func2)` in bi_traverse).
- **Both closures constrained in same impl**: Yes. In each impl block, both F/G (or Func1/Func2) have matching where-clause bounds (e.g., both take owned args in Val, both take refs in Ref). This prevents mixed Val/Ref.
- **Dispatch method destructures correctly**: Yes. All bi-\* Val impls pass `self.0` as the first function and `self.1` as the second. Same for Ref impls.

**Verdict: Bifunctorial tuple pattern is correctly implemented.**

---

## 7. Documentation

All 19 dispatch files include:

- Module-level doc comments with `### Examples` section showing both owned and by-ref usage.
- `#[fp_macros::document_module]` on the inner module.
- `#[document_type_parameters(...)]` on the dispatch trait.
- `#[document_parameters(...)]` on the trait and both impl blocks.
- `#[document_signature]` on both trait method and impl method definitions.
- `#[document_parameters(...)]` on method parameters.
- `#[document_returns(...)]` on method returns.
- `#[document_examples]` with working code examples on trait method, both impl methods, and the free function.
- Free function has comprehensive doc comments describing the dispatch behavior, with `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]`.

**Verdict: Documentation is comprehensive and consistent across all 19 files.**

---

## 8. `dispatch.rs` module registration

All 19 new modules are registered in `fp-library/src/classes/dispatch.rs`:

```
pub mod bi_fold_left;
pub mod bi_fold_map;
pub mod bi_fold_right;
pub mod bi_traverse;
pub mod bimap;
pub mod filter;
pub mod filter_map_with_index;
pub mod filter_with_index;
pub mod fold_left_with_index;
pub mod fold_map_with_index;
pub mod fold_right_with_index;
pub mod map_with_index;
pub mod partition;
pub mod partition_map;
pub mod partition_map_with_index;
pub mod partition_with_index;
pub mod traverse_with_index;
pub mod wilt;
pub mod wither;
```

All dispatch free functions are re-exported at the dispatch module level via
the `pub use` block (lines 122-162), including all 19 new functions.

**Verdict: Module registration and re-exports are complete.**

---

## 9. `functions.rs` re-exports

### 9a. Dispatch functions re-exported

All new dispatch functions are present in the `pub use crate::classes::dispatch::{...}`
block (lines 64-99):

- `bi_fold_left`, `bi_fold_map`, `bi_fold_right` -- present.
- `bi_traverse` -- present.
- `bimap` -- present.
- `filter` -- present.
- `filter_map_with_index` -- present.
- `filter_with_index` -- present.
- `fold_left_with_index`, `fold_map_with_index`, `fold_right_with_index` -- present.
- `map_with_index` -- present.
- `partition`, `partition_map` -- present.
- `partition_map_with_index`, `partition_with_index` -- present.
- `traverse_with_index` -- present.
- `wilt`, `wither` -- present.

### 9b. Conflicting non-dispatch functions aliased

The `generate_function_re_exports!` macro correctly aliases all conflicting names:

- `bifoldable::bi_fold_left` -> `bifoldable_bi_fold_left`
- `bifoldable::bi_fold_map` -> `bifoldable_bi_fold_map`
- `bifoldable::bi_fold_right` -> `bifoldable_bi_fold_right`
- `bifunctor::bimap` -> `bifunctor_bimap`
- `bitraversable::bi_traverse` -> `bitraversable_bi_traverse`
- `filterable::filter` -> `filterable_filter`
- `filterable::filter_map` -> `filterable_filter_map`
- `filterable::partition` -> `filterable_partition`
- `filterable::partition_map` -> `filterable_partition_map`
- `filterable_with_index::filter_map_with_index` -> `filterable_with_index_filter_map_with_index`
- `filterable_with_index::filter_with_index` -> `filterable_with_index_filter_with_index`
- `filterable_with_index::partition_map_with_index` -> `filterable_with_index_partition_map_with_index`
- `filterable_with_index::partition_with_index` -> `filterable_with_index_partition_with_index`
- `functor_with_index::map_with_index` -> `functor_with_index_map_with_index`
- `foldable_with_index::fold_left_with_index` -> `foldable_with_index_fold_left_with_index`
- `foldable_with_index::fold_map_with_index` -> `foldable_with_index_fold_map_with_index`
- `foldable_with_index::fold_right_with_index` -> `foldable_with_index_fold_right_with_index`
- `traversable::traverse` -> `traversable_traverse`
- `traversable_with_index::traverse_with_index` -> `traversable_with_index_traverse_with_index`
- `witherable::wilt` -> `witherable_wilt`
- `witherable::wither` -> `witherable_wither`

### 9c. Issue: `ref_*` functions from Ref trait modules

The `ref_*` free functions from the Ref trait modules (e.g., `ref_bifoldable::ref_bi_fold_left`,
`ref_bifunctor::ref_bimap`, `ref_bitraversable::ref_bi_traverse`) are aliased in the
`generate_function_re_exports!` macro:

- `ref_bifoldable::ref_bi_fold_left` -> `ref_bifoldable_ref_bi_fold_left`
- `ref_bifoldable::ref_bi_fold_map` -> `ref_bifoldable_ref_bi_fold_map`
- `ref_bifoldable::ref_bi_fold_right` -> `ref_bifoldable_ref_bi_fold_right`
- `ref_bifunctor::ref_bimap` -> `ref_bifunctor_ref_bimap`
- `ref_bitraversable::ref_bi_traverse` -> `ref_bitraversable_ref_bi_traverse`

However, the macro scans `src/classes/` and auto-exports all public functions.
This means `ref_bi_fold_left`, `ref_bimap`, `ref_bi_traverse`, etc. are still
exported under their canonical names by the macro (the alias entries only apply
when there is a name conflict with a dispatch function). Since the dispatch
functions have different names (`bi_fold_left`, not `ref_bi_fold_left`), the
`ref_*` functions are exported under BOTH their canonical name AND the alias.

**Per the plan** (section "ref*\* function handling"): "When dispatch unifies two
functions, the ref*\* free function should be removed from the public API, with
the dispatch version fully replacing it."

The following `ref_*` free functions are still publicly exported (non-exhaustive
list of those covered by new dispatch traits):

- `ref_filter`, `ref_filter_map`, `ref_partition`, `ref_partition_map`
- `ref_filter_with_index`, `ref_filter_map_with_index`, `ref_partition_with_index`, `ref_partition_map_with_index`
- `ref_map_with_index`
- `ref_fold_left_with_index`, `ref_fold_right_with_index`, `ref_fold_map_with_index`
- `ref_traverse_with_index`
- `ref_wilt`, `ref_wither`
- `ref_bimap`, `ref_bi_fold_left`, `ref_bi_fold_right`, `ref_bi_fold_map`, `ref_bi_traverse`

These are still callable directly by users, which contradicts the plan's intent
to have the dispatch version be the sole public entry point.

**Approaches:**

1. **Remove the `ref_*` free functions from the Ref trait modules entirely.**
   The dispatch free functions completely replace them. Users calling
   `ref_filter(pred, &x)` can call `filter(pred, &x)` instead.
   - Trade-off: Breaking change for anyone using the `ref_*` functions directly.
   - Risk: Type impl blocks that call `ref_*` free functions in doc examples
     would need updating. There are many such call sites (see section 10).

2. **Add the `ref_*` function names to the alias map so they get non-canonical
   names.** E.g., `"ref_filterable::ref_filter": ref_filterable_ref_filter`.
   - Trade-off: They remain accessible under ugly names, which is
     intentionally discouraging while not breaking.
   - Risk: The macro auto-exports all public functions; adding an alias only
     renames, it does not hide. The canonical `ref_filter` name would still be
     auto-exported unless the macro is also updated to suppress it.

3. **Mark the `ref_*` free functions as `#[doc(hidden)]` or `pub(crate)`.**
   - Trade-off: Effectively hides them from the public API while keeping them
     available for internal use (trait impls may call them).
   - Risk: Trait impls in type modules might use the free function form; changing
     visibility would require checking all internal usage.

**Recommendation:** Option 1 is cleanest long-term but is a large follow-up
change. As an interim step, option 3 (`#[doc(hidden)]`) on the free functions
is the least disruptive. This should be tracked as a follow-up task.

---

## 10. Turbofish updates in type files

The diff shows turbofish updates across `cat_list.rs`, `control_flow.rs`,
`identity.rs`, `option.rs`, `pair.rs`, `result.rs`, `try_thunk.rs`,
`tuple_2.rs`, and `vec.rs`. These updates fall into two categories:

### 10a. Updated call sites (correct)

Call sites that were updated to use dispatch functions with correct turbofish:

- `partition_map::<CatListBrand, _, _, _, _, _>` (was `_, _, _`) -- correct.
- `partition::<CatListBrand, _, _, _>` (was `_`) -- correct.
- `filter::<CatListBrand, _, _, _>` (was `_`) -- correct.
- `bimap::<ControlFlowBrand, _, _, _, _, _, _>` with tuple arg -- correct.
- `bi_fold_right::<RcFnBrand, ...>` with tuple arg -- correct.
- `wilt::<RcFnBrand, ...>` added FnBrand -- correct.
- `wither::<RcFnBrand, ...>` added FnBrand -- correct.
- Various `bi_traverse` calls updated to dispatch form -- correct.

### 10b. Un-migrated `ref_*` call sites (remaining work)

A significant number of `ref_*` free function call sites remain in type files.
These appear in doc examples and test code within type implementation modules:

**In doc examples on trait impl methods:**

- `pair.rs`: `ref_bimap::`, `ref_bi_fold_right::`, `ref_bi_traverse::`,
  `ref_traverse::` (PairFirstAppliedBrand, PairSecondAppliedBrand).
- `control_flow.rs`: `ref_bimap::`.
- `result.rs`: `ref_bimap::`, `ref_traverse::` (ResultErrAppliedBrand, ResultOkAppliedBrand).
- `tuple_2.rs`: `ref_bimap::`, `ref_traverse::` (Tuple2FirstAppliedBrand, Tuple2SecondAppliedBrand).
- `identity.rs`: `ref_traverse::`, `ref_map_with_index::`, `ref_fold_map_with_index::`, `ref_traverse_with_index::`.
- `option.rs`: `ref_filter_map::`, `ref_traverse::`, `ref_map_with_index::`, `ref_fold_map_with_index::`, `ref_traverse_with_index::`.
- `vec.rs`: `ref_filter_map::`, `ref_traverse::`, `ref_map_with_index::`, `ref_fold_map_with_index::`, `ref_filter_map_with_index::`, `ref_traverse_with_index::`.
- `cat_list.rs`: `ref_filter_map::`, `ref_traverse::`, `ref_map_with_index::`, `ref_fold_map_with_index::`, `ref_filter_map_with_index::`, `ref_traverse_with_index::`.

**In test code:**

- `pair.rs` tests: `ref_bimap::`, `ref_bi_fold_map::`.
- `vec.rs` tests: `VecBrand::ref_traverse::` (direct trait method calls).
- `option.rs` tests: `OptionBrand::ref_traverse::` (direct trait method calls).

**Assessment:** These un-migrated sites are not bugs; the old `ref_*` functions
still work. However, they represent inconsistency: some call sites were migrated
to dispatch form (e.g., `bi_fold_right` with tuple args and `&x`) while
analogous sites were not. The remaining `ref_*` calls should be migrated in a
follow-up pass.

**Recommendation:** Create a follow-up task to migrate all remaining `ref_*`
free function calls in type files to use the dispatch versions. This is
straightforward but mechanical: replace `ref_bimap::<Brand, _, _, _, _>(f, g, &x)`
with `bimap::<Brand, _, _, _, _, _, _>((f, g), &x)`, etc.

---

## 11. Missing dispatch for `ref_*`-only functions

Several `ref_*` free functions exist that do NOT have corresponding dispatch
traits. These are convenience/derived functions that the plan does not scope:

- `ref_bi_sequence`, `ref_bi_traverse_left`, `ref_bi_traverse_right`
- `ref_bi_for`, `ref_bi_for_left`, `ref_bi_for_right`
- `ref_apply`, `ref_apply_first`, `ref_apply_second`
- `ref_pure`, `ref_join`, `ref_alt`
- `ref_compact`, `ref_separate`
- `ref_if_m`, `ref_unless_m`

These are out of scope for this review since they are not covered by the
dispatch expansion plan.

---

## 12. Minor observations

### 12a. `fold_map_with_index` free function missing `A: Clone` in signature

The `fold_map_with_index` free function signature (line 259-272 of
`fold_map_with_index.rs`) has `A: 'a` without `Clone`. But the Val impl
bounds `A: 'a + Clone`. This is fine because the impl's where clause is more
restrictive; the free function signature is loose and the impl provides the
actual constraint. Same pattern as other foldable dispatches. Not a bug.

### 12b. `traverse_with_index` doc examples import from `classes::dispatch`

The `traverse_with_index.rs` doc examples use
`use fp_library::classes::dispatch::traverse_with_index;` instead of
`use fp_library::functions::*;`. This is inconsistent with all other dispatch
files which use `functions::*`. Not a bug (both paths work), but it makes
the examples less uniform.

**Recommendation:** Update `traverse_with_index.rs` doc examples to use
`use fp_library::functions::*;` for consistency.

---

## Conclusion

The 19 new dispatch traits are well-implemented. No E0119 risks, no incorrect
trait bounds, no delegation errors, and no Clone bound mismatches. FnBrand
handling follows the established precedent consistently. Documentation is
comprehensive.

The main follow-up items are:

1. **Migrate remaining `ref_*` call sites in type files** to dispatch form
   (approximately 40+ sites across 10 files).
2. **Deprecate or hide `ref_*` free functions** that are fully superseded
   by dispatch versions, per the plan's "ref\_\* function handling" section.
3. **Fix `traverse_with_index.rs` doc examples** to use `functions::*` import.
