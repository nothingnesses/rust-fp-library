# Dispatch Expansion: Open Questions Investigation 3

Focus: Interaction with existing code, m_do!/a_do! macros, and downstream plans.

## 1. m_do! and a_do! macro codegen

**Question:** Do the macros generate calls to any of the 9 functions being
unified (filter, partition, wilt, wither, etc.)? If so, turbofish counts in
the generated code would need updating.

**Finding:** Neither macro generates calls to any of the affected functions.

- `m_do!` (`fp-macros/src/m_do/codegen.rs`) generates only `bind` calls
  (with turbofish `bind::<#brand, _, _, _, _>`) and `pure`/`ref_pure` calls.
- `a_do!` (`fp-macros/src/a_do/codegen.rs`) generates `pure`/`ref_pure`,
  `map` (with turbofish `map::<#brand, _, _, _, _>`), and `liftN` (with
  computed underscore counts `2n + 2`).

None of these are in the dispatch-expansion scope. No macro changes are
required.

**Recommendation:** No action needed. The macros are safe from this plan.

## 2. Interaction with the brand-inference plan

**Question:** After dispatch is added, some functions classified as
"non-dispatch" in the brand-inference plan would become dispatch-based.
Does the brand-inference plan need updating?

**Finding:** Yes, the brand-inference plan (`docs/plans/brand-inference/plan.md`)
has a three-tier classification at lines 263-365:

- **Dispatch-based, full inference:** Currently lists only `map`, `bind`,
  `bind_flipped`, `filter_map`, `lift2`-`lift5`.
- **Non-dispatch, full inference:** Lists `filter`, `partition`,
  `partition_map`, `map_with_index`, `filter_with_index`,
  `filter_map_with_index`, `partition_with_index`, and
  `partition_map_with_index` as functions that get brand inference via
  `DefaultBrand` but have no Val/Ref dispatch. Their `ref_*` counterparts
  are listed separately in the "Ref-only non-dispatch functions" table.
- **Partial inference:** Lists `wilt`, `wither`, `ref_wilt`, `ref_wither`,
  `fold_map_with_index`, `fold_right_with_index`, `fold_left_with_index`,
  `ref_fold_map_with_index`, and `traverse_with_index` as partial-inference
  functions (Brand inferred, FnBrand/M remain explicit).

After dispatch expansion:

1. `filter`, `partition`, `partition_map`, `map_with_index`,
   `filter_with_index`, `filter_map_with_index`, `partition_with_index`,
   and `partition_map_with_index` move from "Non-dispatch, full inference"
   to "Dispatch-based, full inference." Their `ref_*` counterparts are
   absorbed into the dispatch function and should be removed from the
   "Ref-only" table.

2. `wilt`, `wither` become dispatch-based but remain partial inference
   (the `M` brand is still explicit). `ref_wilt` and `ref_wither` are
   absorbed. The brand-inference plan already lists them in partial
   inference, but currently shows `ref_wilt`/`ref_wither` as separate
   entries; those would merge.

3. `fold_map_with_index`, `fold_right_with_index`, `fold_left_with_index`,
   `traverse_with_index` become dispatch-based but remain partial inference
   (FnBrand is still explicit). Their `ref_*` counterparts are absorbed.

The brand-inference plan's naming convention section says non-dispatch
functions "each get an `_explicit` variant." After dispatch expansion,
these functions instead follow the dispatch naming convention (dispatch
version gets the canonical name, non-dispatch gets a module-prefixed
alias via `generate_function_re_exports!`). The `_explicit` suffix would
only apply after brand inference is added on top.

**Approaches:**

(a) Update the brand-inference plan now to reflect the post-dispatch-expansion
state. This makes the plan accurate for when it is implemented, since
dispatch expansion is a prerequisite. Risk: if the dispatch-expansion plan
changes, the brand-inference plan needs re-updating.

(b) Add a note to the brand-inference plan stating that the tier tables assume
dispatch expansion has been completed, and defer updating the tables until
dispatch expansion lands. Risk: the tables are currently inaccurate with
respect to their stated prerequisite.

(c) Keep both plans as-is, since the brand-inference plan already lists
dispatch expansion as a prerequisite and implicitly assumes it is done.

**Recommendation:** Option (b). Add a brief note at the top of the
brand-inference plan's tier table section saying "These tables assume
dispatch expansion is complete. Functions listed as non-dispatch will
become dispatch-based per `docs/plans/dispatch-expansion/plan.md`, and
their ref\_\* counterparts will be absorbed." This keeps both plans
self-consistent without requiring premature table rewrites.

## 3. Interaction with the ref-expansion plan (circular dependency risk)

**Question:** The dispatch-expansion plan's step 5 adds bifunctorial dispatch
after the ref-expansion plan adds RefBifunctor etc. Is there a circular
dependency risk?

**Finding:** No circular dependency. The dependency is strictly one-directional:

- Dispatch-expansion steps 1-4 (simple group, WithIndex, foldable/traversable
  WithIndex, wilt/wither) have no dependency on the ref-expansion plan. All
  Ref traits they dispatch to (`RefFilterable`, `RefFiltarable WithIndex`,
  `RefFunctorWithIndex`, `RefFoldableWithIndex`, `RefTraversableWithIndex`,
  `RefWitherable`) already exist.

- Dispatch-expansion step 5 (bifunctorial dispatch) depends on the
  ref-expansion plan providing `RefBifunctor`, `RefBifoldable`, and
  `RefBitraversable`. The ref-expansion plan does not depend on any
  dispatch traits.

The dispatch-expansion plan needs only the trait definitions from the
ref-expansion plan, not any dispatch infrastructure. The ref-expansion
plan creates traits with free functions (`ref_bimap`, `ref_bi_fold_right`,
etc.) that work independently; dispatch is layered on top afterward.

**One sequencing concern:** If the ref-expansion plan creates `ref_bimap`
as a standalone free function, and then the dispatch-expansion plan creates
a `bimap` dispatch function that absorbs it, the `ref_bimap` free function
would need to be renamed (e.g., `bifunctor_ref_bimap` via
`generate_function_re_exports!`). The ref-expansion plan should be aware
that its free function names are temporary canonical names that will be
demoted to aliases once dispatch is added.

**Recommendation:** Add a note to the ref-expansion plan's free function
naming section indicating that `ref_bimap`, `ref_bi_fold_right`, etc.
will be absorbed into dispatch functions by the dispatch-expansion plan,
and the standalone names will become module-prefixed aliases. This prevents
surprises when the names change.

## 4. Existing test and doc-test call sites needing turbofish updates

**Question:** How many call sites use the current turbofish counts for the
affected functions?

**Finding:** A comprehensive search of `fp-library/src/` found the following
call-site counts (free function calls with turbofish, in source, doc tests,
and inline tests):

| Function group                     | Val calls | Ref calls | Total |
| ---------------------------------- | --------- | --------- | ----- |
| `filter::<`                        | 10        | 6         | 16    |
| `partition::<`                     | 7         | (in ref)  | ~10   |
| `partition_map::<`                 | 13        | (in ref)  | ~17   |
| `wilt::<`                          | 9         | 7         | 16    |
| `wither::<`                        | 16        | 7         | 23    |
| `map_with_index::<`                | 5         | 1         | 6     |
| `filter/partition *_with_index::<` | 12        | 10        | 22    |
| `fold_*_with_index::<`             | 33        | ~8        | ~41   |
| `traverse_with_index::<`           | 10        | ~4        | ~14   |

Approximate total: ~165 call sites across source, doc examples, and tests.

No call sites were found in `fp-library/tests/` (the standalone test files).
All call sites are in `fp-library/src/` (inline tests, doc tests, and trait
default implementations).

The Val free functions become module-prefixed aliases (unchanged signatures),
so their turbofish counts remain the same. However, doc examples that
demonstrate the unified dispatch function will need to use the new turbofish
counts (+2 or +3 underscores). Trait implementations and default methods that
call the non-dispatch free function directly (e.g., `Self::filter(...)`) do
not use the dispatch function and are unaffected.

**Recommendation:** The bulk of the turbofish updates will be in doc examples
(on trait methods and free functions) that need to show the dispatch calling
convention. Source code that calls the internal trait methods
(`Brand::filter(...)`) or the aliased non-dispatch free functions
(`filterable_filter::<...>`) will not change. Plan step 7 ("Update call
sites") should explicitly scope which examples to update vs. leave as-is.
Consider a two-pass approach: first update only the free function doc examples
to use dispatch, then leave trait method doc examples calling the non-dispatch
variants (since those illustrate the trait, not the dispatch layer).

## 5. Benchmark files

**Question:** Do benchmark files use any of the affected functions?

**Finding:** Yes. Three benchmark files call affected functions:

- `fp-library/benches/benchmarks/vec.rs`: `filter::<VecBrand, _>` (1 call),
  `partition::<VecBrand, _>` (1 call), `wither::<VecBrand, OptionBrand, _, _>`
  (1 call), `wilt::<VecBrand, OptionBrand, _, _, _>` (1 call).
- `fp-library/benches/benchmarks/option.rs`: `filter::<OptionBrand, _>` (1),
  `partition::<OptionBrand, _>` (1), `wither::<OptionBrand, ...>` (1),
  `wilt::<OptionBrand, ...>` (1).
- `fp-library/benches/benchmarks/cat_list.rs`: `filter::<CatListBrand, _>` (1),
  `filter::<VecBrand, _>` (1, as a comparison baseline).

Total: ~10 benchmark call sites.

These all use the current non-dispatch turbofish counts. After dispatch
expansion, two options exist:

(a) Update the benchmark calls to use the dispatch function (new turbofish).
This benchmarks the dispatch overhead (should be zero, but confirms it).

(b) Change the benchmark calls to use the module-prefixed alias
(`filterable_filter`, etc.) to benchmark the direct path, keeping the
turbofish unchanged.

**Recommendation:** Do both. Keep existing benchmarks on the direct path
(renamed alias) to maintain regression baselines, and add new benchmarks
for the dispatch path to verify zero overhead. This follows the pattern
used when `map` dispatch was introduced.

## 6. The compose_kleisli tuple pattern precedent

**Question:** How exactly does compose_kleisli's tuple dispatch work, and
does the bimap dispatch proposal correctly follow it?

**Finding:** From `fp-library/src/classes/dispatch/semimonad.rs`:

- `ComposeKleisliDispatch` is a trait with type parameters
  `<'a, Brand, A, B, C, Marker>`. It is implemented for `(F, G)` (a tuple
  of two closures), not for individual closures.

- The free function `compose_kleisli` takes the tuple as a single argument:

  ```
  pub fn compose_kleisli<'a, Brand, A, B, C, Marker>(
      fg: impl ComposeKleisliDispatch<'a, Brand, A, B, C, Marker>,
      a: A,
  ) -> ...
  ```

- Turbofish: `compose_kleisli::<OptionBrand, _, _, _, _>((..., ...), 5)` with
  5 type params (Brand, A, B, C, Marker). The tuple `(f, g)` is passed as a
  single argument; `F` and `G` are not type parameters of the free function,
  they are inferred from the impl block.

- `compose_kleisli_flipped` is different: it has 8 type parameters
  (Brand, A, B, C, F, G, Marker) because it takes `(F, G)` and then
  internally reorders to `(G, F)`, so `F` and `G` must be named in the
  signature. Turbofish:
  `compose_kleisli_flipped::<OptionBrand, _, _, _, _, _, _>(...)`.

- The Val impl constrains both closures: `F: Fn(A) -> Of<B>`,
  `G: Fn(B) -> Of<C>`. The Ref impl constrains both: `F: Fn(&A) -> Of<B>`,
  `G: Fn(&B) -> Of<C>`. Mixed Val/Ref does not match either impl.

**Applicability to bimap dispatch:** The plan's `BimapDispatch` proposal
matches this pattern correctly. Key observations:

- `bimap((f, g), p)` mirrors `compose_kleisli((f, g), a)`.
- The trait is implemented for `(F, G)`, with `F` and `G` inferred (not
  free-function type parameters), so the turbofish does not include them.
- Both closures must agree on Val/Ref, just as both Kleisli arrows must agree.

One difference: `compose_kleisli` has no `FA` container parameter (the input
is a plain value `A`), so it does not have the `FA`/`Marker` dispatch on
the container side. `bimap` will have `FA` (the bifunctor container), adding
one more type parameter. The plan accounts for this.

**One subtle issue:** `compose_kleisli_flipped` names `F` and `G` in the
function signature because it needs to swap them. If `bimap_flipped` or
similar flipped variants are added later, they would face the same issue.
The dispatch-expansion plan does not mention flipped variants for bi-\*
functions, so this is not an immediate concern, but it is worth noting for
future extensibility.

**Recommendation:** The bimap tuple pattern is correctly modeled after
compose_kleisli. No changes needed. Document the compose_kleisli_flipped
asymmetry (extra type params for flipped variants) in case bi-\* flipped
variants are added later.
