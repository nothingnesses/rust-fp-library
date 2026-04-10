# Dispatch Expansion: Open Questions Investigation 2

Focus: Re-export mechanism, naming conventions, and API surface changes.

---

## Question 1: How does `generate_function_re_exports!` work, and what must change when new dispatch modules are added?

### Research findings

The macro lives in `fp-macros/src/codegen/re_export.rs`. Its operation:

1. It receives a directory path (e.g., `"src/classes"`) and an alias map.
2. It scans all `.rs` files in that directory (not subdirectories), skipping
   `mod.rs`.
3. For each file, it parses the AST and collects all `pub fn` items. If the
   file uses a `pub use inner::*` re-export pattern, it looks inside the
   `inner` module instead.
4. For each discovered function, it checks the alias map using the key
   `"module_name::function_name"` (e.g., `"filterable::filter_map"`). If an
   alias exists, the function is re-exported under the alias name. Otherwise
   it is re-exported under its original name.
5. All re-exports are combined into a single `pub use crate::classes::{...}`
   statement.

The macro does NOT scan subdirectories. The `dispatch/` subdirectory is
invisible to it. Dispatch free functions are manually re-exported in a
separate `pub use crate::classes::dispatch::{...}` block in `functions.rs`.

### What must change for each new dispatch function

For each new dispatch function (e.g., `filter`):

1. Create the dispatch module file (e.g., `dispatch/filterable_filter.rs` or
   add to existing `dispatch/filterable.rs`).
2. Add the module and re-export in `dispatch.rs`.
3. Add the canonical name to the manual `pub use` block in `functions.rs`.
4. Add an alias entry in the `generate_function_re_exports!` alias map for
   the non-dispatch version (e.g., `"filterable::filter": filterable_filter`).

The `ref_*` functions from `ref_*.rs` files need separate consideration (see
Question 3 below).

### Issue: the macro has no exclusion mechanism

The macro auto-discovers every `pub fn` in every `.rs` file under
`src/classes/`. There is no way to exclude a function without either:

- Removing it from the source file.
- Adding an alias for it (which renames but still exports it).

This means when `filter` gets a dispatch version:

- The non-dispatch `filter` in `filterable.rs` must be aliased (e.g.,
  `"filterable::filter": filterable_filter`). If it is not aliased, the
  macro will export `filter` from `filterable.rs`, which will conflict with
  the manually exported dispatch `filter` from the `pub use` block.
- The `ref_filter` in `ref_filterable.rs` will be auto-exported under its
  original name `ref_filter`. Since the dispatch version uses the name
  `filter` (not `ref_filter`), there is no name collision. But `ref_filter`
  remains in the public API as a redundant function.

### Recommendation

For each new dispatch function:

- Add the non-dispatch Val function to the alias map.
- Decide whether to alias, remove, or keep the `ref_*` function (see
  Question 3).
- No changes to the macro itself are needed.

---

## Question 2: Name collisions for `filter` and other common names

### Research findings

Searching for `pub fn filter` across the codebase found exactly one
definition in the class hierarchy:

- `filterable.rs` line 443: `pub fn filter<'a, Brand: Filterable, A: 'a + Clone>(...)`

There is no other `pub fn filter` in any other module under `src/classes/`,
`src/types/`, or `src/functions.rs`.

Related names that exist but are distinct:

- `par_filter` in `par_filterable.rs`.
- `par_ref_filter` in `par_ref_filterable.rs`.
- `ref_filter` in `ref_filterable.rs`.
- `filter_with_index` in `filterable_with_index.rs`.
- `ref_filter_with_index` in `ref_filterable_with_index.rs`.
- `filter_map` (already dispatched, in `dispatch/filterable.rs`).

None of these collide with a dispatch `filter` because they have different
names.

For `partition`, `partition_map`, `wilt`, `wither`, and all `*_with_index`
variants, there are similarly no collisions. Each name exists exactly once as
a non-dispatch free function in its respective trait module, plus once as a
`ref_*` variant.

### Potential collision risk: `filter` method on Iterator

Rust's `Iterator::filter` is a method, not a free function, so it does not
collide with `fp_library::functions::filter`. However, users who `use
fp_library::functions::*` alongside `use std::iter::*` should not encounter
ambiguity because `Iterator::filter` is a method called via dot syntax, while
`fp_library::filter` is a free function.

### Recommendation

No name collision issues found. The dispatch `filter` can safely take the
canonical name. The only action needed is aliasing the non-dispatch
`filterable::filter` to `filterable_filter` in the re-export macro.

---

## Question 3: The fate of `ref_*` free functions when dispatch unifies them

### Research findings

The existing dispatch functions show two different strategies:

**Strategy A: Remove both non-dispatch functions (functor, foldable).**

For `map`, `fold_left`, `fold_right`, `fold_map`:

- The non-dispatch `pub fn map` was removed from `functor.rs`.
- The non-dispatch `pub fn ref_map` was removed from `ref_functor.rs`
  (the file has zero `pub fn` items).
- No aliases needed in the re-export macro.
- The dispatch version in `dispatch/functor.rs` is the only `map`.
- Doc examples on the trait definitions themselves call the dispatch
  version directly (e.g., `map::<OptionBrand, _, _, _, _>(...)`).

**Strategy B: Keep non-dispatch functions, alias them (filterable,
traversable).**

For `filter_map`:

- The non-dispatch `pub fn filter_map` remains in `filterable.rs`.
- It is aliased as `filterable_filter_map` in the re-export macro.
- The `ref_filter_map` remains in `ref_filterable.rs` with no alias.
- It is auto-exported under its original name `ref_filter_map`.

For `traverse`:

- The non-dispatch `pub fn traverse` remains in `traversable.rs`.
- It is aliased as `traversable_traverse`.
- The `ref_traverse` remains in `ref_traversable.rs` with no alias.
- It is auto-exported under its original name `ref_traverse`.

**Why the inconsistency?**

The `filterable.rs` and `traversable.rs` modules contain additional
non-dispatched free functions (`filter`, `partition`, `partition_map`,
`sequence`) alongside the dispatched one (`filter_map`, `traverse`). These
modules cannot have all their free functions removed. The non-dispatch
`filter_map` and `traverse` were kept rather than removed because they serve
as documentation targets in their trait's doc examples and as the underlying
implementation that the dispatch trait calls.

For `functor.rs` and `foldable.rs`, the only free functions were the ones
being dispatched (`map`, `fold_left`, `fold_right`, `fold_map`), so removing
them entirely was clean.

**Consequence for `ref_*` functions:**

When dispatch is added for `filter`, `partition`, etc., the `ref_*` versions
(`ref_filter`, `ref_partition`, etc.) become redundant. Currently, the
existing `ref_filter_map` and `ref_traverse` are still auto-exported by the
macro. They remain in the public API even though users should prefer the
dispatch version.

### Approaches

**Approach A: Keep `ref_*` functions, no alias.**

The `ref_*` free functions remain exported under their original names.
Users who have existing code calling `ref_filter::<VecBrand, _>(...)` can
continue using it. The dispatch `filter` subsumes it but does not replace it.

Trade-off: API surface grows. Users see both `filter` and `ref_filter` in
the docs and may be confused about which to use.

**Approach B: Alias `ref_*` functions with a `ref_filterable_ref_filter`
style name.**

This hides them from the obvious namespace but keeps them accessible.

Trade-off: The aliased names are very long and ugly. The `ref_*` names are
already clear about their purpose (unlike `filter_map` which needed
disambiguation from the dispatch `filter_map`).

**Approach C: Remove the `ref_*` free functions entirely.**

Delete `ref_filter`, `ref_partition`, etc. from `ref_filterable.rs`. The
dispatch version handles all use cases. Doc examples on the `RefFilterable`
trait would call the dispatch version with a `Fn(&A)` closure.

Trade-off: Breaking change for callers using `ref_filter`. Requires updating
doc examples in the `RefFilterable` trait to use the dispatch function. This
is the cleanest long-term approach, matching Strategy A used for
`functor.rs`.

**Approach D: Mark `ref_*` with `#[deprecated]`.**

Keep them but add deprecation warnings pointing users to the dispatch
version.

Trade-off: Generates warnings in downstream code. Requires eventual removal
in a future version.

### Recommendation

Use Approach C (remove `ref_*` free functions) for consistency with how
`ref_map`, `ref_fold_left`, etc. were handled. When the dispatch version
exists, the `ref_*` free function is redundant. Update doc examples on the
Ref traits to use the dispatch function.

However, the non-dispatch Val functions (e.g., `filter` in `filterable.rs`)
should be kept and aliased, following the Strategy B precedent set by
`filter_map` and `traverse`. They serve as documentation targets and as the
dispatch implementation target.

---

## Question 4: Dispatch module file organization

### Research findings

The current dispatch directory has 6 files, organized by trait group:

```
dispatch/
  functor.rs       -- FunctorDispatch + map
  semimonad.rs     -- BindDispatch + bind, bind_flipped,
                      ComposeKleisliDispatch + compose_kleisli,
                      compose_kleisli_flipped
  lift.rs          -- Lift2Dispatch-Lift5Dispatch + lift2-lift5
  foldable.rs      -- FoldRightDispatch, FoldLeftDispatch,
                      FoldMapDispatch + fold_right, fold_left, fold_map
  filterable.rs    -- FilterMapDispatch + filter_map
  traversable.rs   -- TraverseDispatch + traverse
```

The convention is: group closely related dispatch traits into one file, where
"closely related" means they belong to the same type class (e.g., all three
foldable dispatch traits share `foldable.rs`).

### Proposed organization for new dispatch traits

Following the existing convention:

- `dispatch/filterable.rs` (existing): Add `FilterDispatch`,
  `PartitionDispatch`, `PartitionMapDispatch` alongside the existing
  `FilterMapDispatch`. All four belong to the `Filterable` type class.

- `dispatch/filterable_with_index.rs` (new): `FilterWithIndexDispatch`,
  `FilterMapWithIndexDispatch`, `PartitionWithIndexDispatch`,
  `PartitionMapWithIndexDispatch`. All belong to `FilterableWithIndex`.

- `dispatch/functor_with_index.rs` (new): `MapWithIndexDispatch`. Only one
  trait, but it belongs to `FunctorWithIndex` which is a distinct type class
  from `Functor`.

- `dispatch/foldable_with_index.rs` (new): `FoldMapWithIndexDispatch`,
  `FoldRightWithIndexDispatch`, `FoldLeftWithIndexDispatch`. Mirrors the
  existing `foldable.rs` grouping.

- `dispatch/traversable_with_index.rs` (new): `TraverseWithIndexDispatch`.
  Mirrors `traversable.rs`.

- `dispatch/witherable.rs` (new): `WiltDispatch`, `WitherDispatch`. Both
  belong to the `Witherable` type class.

### Alternative: one file per dispatch trait

Each dispatch trait gets its own file. This would create 14 new files (one
per function being dispatched), bringing the total to 20 files in the
dispatch directory.

Trade-off: More files to navigate, but each file is simpler and self-
contained. The existing convention of grouping by type class is more
organized.

### Recommendation

Follow the existing convention: group dispatch traits by their parent type
class. This results in 6 new files (filterable_with_index,
functor_with_index, foldable_with_index, traversable_with_index,
witherable) plus modifications to the existing `filterable.rs`. The
`dispatch.rs` module file needs updating to declare and re-export these
new sub-modules.

---

## Question 5: Doc comment updates for dispatch functions

### Research findings

Examining the existing dispatch free functions, the documentation follows a
consistent pattern.

The dispatch `filter_map` in `dispatch/filterable.rs` (lines 205-261):

- Opens with a description of the operation.
- Explains the dispatch routing: "Dispatches to either
  `Filterable::filter_map` or `RefFilterable::ref_filter_map` based on the
  closure's argument type."
- Lists both dispatch paths with bullet points (owned vs. borrowed).
- Notes that `Marker` and `FA` are inferred automatically.
- States "The dispatch is resolved at compile time with no runtime cost."
- Provides examples showing both the owned and by-ref calling conventions.

The dispatch `map` in `dispatch/functor.rs` follows the same template.

The module-level doc comment also references both traits:
`//! Dispatch for [Functor::map] and [RefFunctor::ref_map].`

### What needs updating for new dispatch functions

For each new dispatch function, the doc comment should:

1. Reference both the Val and Ref trait methods by name with doc links.
2. Describe the dispatch routing (owned closure -> Val trait, reference
   closure -> Ref trait).
3. Show examples of both calling conventions.
4. Use `#[document_signature]`, `#[document_type_parameters(...)]`,
   `#[document_parameters(...)]`, `#[document_returns(...)]`, and
   `#[document_examples]` procedural macro attributes.

### Issue: doc examples on non-dispatch functions

When a non-dispatch free function is kept and aliased (e.g.,
`filterable_filter_map`), its doc examples still use the original name.
For example, `filterable.rs` line 405 has:

```rust
/// let y = filter_map::<OptionBrand, _, _, _, _>(|a| if a > 2 { ... }, x);
```

This is the aliased `filterable_filter_map` function, but its doc example
calls `filter_map` (the dispatch version). This works because the dispatch
version has the same calling convention for the Val path. However, it is
slightly misleading: the example does not exercise the function it documents.

For the new dispatch functions, the same pattern will apply. The non-dispatch
`filter` in `filterable.rs` will be aliased as `filterable_filter`, but its
doc examples should continue to call `filter::<OptionBrand, _, _, _>(...)`
(the dispatch version), since that is the function users will actually use.

### Recommendation

Follow the established template exactly. Each dispatch free function gets:

- Module-level doc linking both Val and Ref traits.
- Trait-level doc explaining the marker/FA inference.
- Free function doc explaining dispatch routing with two bullet points.
- Two examples: one owned, one by-ref.

For the aliased non-dispatch functions, keep their existing doc examples
pointing to the dispatch version. This is the existing convention and works
correctly.

---

## Question 6: Turbofish call site counts for functions being unified

### Research findings

Counted all turbofish call sites (`function_name::<...>`) in source code,
doc comments, tests, and benchmarks. Excluded `par_*`, `send_ref_*`, and
method-call-syntax uses (e.g., `Brand::filter`). Counts include both source
and documentation.

| Function                   | Val sites | Ref sites | Total | Notes                                           |
| -------------------------- | --------- | --------- | ----- | ----------------------------------------------- |
| `filter`                   | 13        | 2         | 15    | Includes benchmarks (3 sites).                  |
| `partition`                | 9         | 2         | 11    | Includes benchmarks (2 sites).                  |
| `partition_map`            | 15        | 2         | 17    | Includes benchmarks (2 sites).                  |
| `wilt`                     | 12        | 2         | 14    | Includes benchmarks (2 sites).                  |
| `wither`                   | 15        | 4         | 19    | Includes benchmarks (2 sites).                  |
| `map_with_index`           | 5         | 8         | 13    | Ref count includes send_ref variants excluded.  |
| `filter_with_index`        | 3         | 3         | 6     | Moderate count.                                 |
| `filter_map_with_index`    | 3         | 6         | 9     | Ref count slightly higher.                      |
| `partition_with_index`     | 3         | 2         | 5     | Small count.                                    |
| `partition_map_with_index` | 3         | 2         | 5     | Small count.                                    |
| `fold_map_with_index`      | 26        | 25        | 51    | Largest count. Heavily used in implementations. |
| `fold_right_with_index`    | 4         | 4         | 8     | Used mainly in trait defaults.                  |
| `fold_left_with_index`     | 3         | 2         | 5     | Used mainly in trait defaults.                  |
| `traverse_with_index`      | 10        | 8         | 18    | Used in impl blocks and doc examples.           |

Total call sites requiring turbofish updates: approximately 196.

### High-impact functions

`fold_map_with_index` has 51 call sites, by far the most. Many of these
are internal trait default implementations and `impl` blocks that use
method-call syntax (`Self::fold_map_with_index::<FnBrand, _, _>(...)`),
which are not free function calls and may not need updating if the dispatch
is only on the free function.

However, examining the actual call sites more carefully, a significant
portion use the UFCS method-call syntax (e.g.,
`Brand::fold_map_with_index::<FnBrand, _, _>(...)`), not the free function.
These method calls have different turbofish counts from the free function
and would NOT be affected by the dispatch changes (dispatch only affects
free functions, not trait method calls).

Free function turbofish call sites (those that will actually need updating):

| Function                   | Free fn sites |
| -------------------------- | ------------- |
| `filter`                   | 15            |
| `partition`                | 11            |
| `partition_map`            | 17            |
| `wilt`                     | 14            |
| `wither`                   | 19            |
| `map_with_index`           | ~5            |
| `filter_with_index`        | ~5            |
| `filter_map_with_index`    | ~6            |
| `partition_with_index`     | ~5            |
| `partition_map_with_index` | ~5            |
| `fold_map_with_index`      | ~6            |
| `fold_right_with_index`    | ~3            |
| `fold_left_with_index`     | ~3            |
| `traverse_with_index`      | ~5            |

Estimated total free function turbofish sites needing updates: ~125.

### Mitigation

This is comparable to prior dispatch additions. The `map` dispatch (which
changed turbofish from 2 to 5 parameters) required updating hundreds of
call sites across the codebase. The approach used was a systematic
search-and-replace, adjusting each turbofish to add the inferred `_, _`
parameters.

### Recommendation

Proceed with the turbofish changes. The counts are manageable. Implement
the dispatch functions in the order specified by the plan (simple group
first), updating turbofish counts for each function before moving to the
next. This limits the blast radius of each change and makes `just verify`
meaningful at each step.

---

## Summary of findings

1. **Re-export macro:** Works by scanning `src/classes/*.rs` for `pub fn`
   items. Dispatch modules in `dispatch/` are invisible to it. Each new
   dispatch function needs: (a) an alias for the non-dispatch Val function,
   (b) a manual `pub use` for the dispatch function, (c) a decision about
   the `ref_*` function.

2. **Name collisions:** None found. `filter`, `partition`, `partition_map`,
   `wilt`, `wither`, and all `*_with_index` names are unique across the
   codebase.

3. **`ref_*` function fate:** Inconsistent precedent. `ref_map` was removed
   entirely; `ref_filter_map` and `ref_traverse` were kept. Recommend
   removing `ref_*` free functions for consistency, but this is a breaking
   change that affects call sites using those functions directly.

4. **Dispatch file organization:** Follow the existing convention of
   grouping by parent type class. Six new files plus modifications to
   existing `dispatch/filterable.rs`.

5. **Doc comments:** Follow the established template exactly. Each dispatch
   function references both the Val and Ref traits, explains dispatch
   routing, and shows both calling conventions.

6. **Turbofish counts:** Approximately 125 free function turbofish sites
   need updating. `fold_map_with_index` has the most, but many of its uses
   are method-call syntax that is unaffected. The counts are manageable with
   systematic search-and-replace.
