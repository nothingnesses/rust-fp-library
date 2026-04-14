# Plan: HM Signature Rendering Fix

## Current Progress

- Steps 1-3: Done. Phase 1 complete.
- Step 4: Done. All 18 dispatch modules have `explicit` submodules. Flat
  re-exports removed from `dispatch.rs`. `functions.rs` has `pub mod explicit`
  with nested path re-exports. Old `_explicit` alias block removed. All ~85
  internal call sites, doc examples, benchmark files, test files, macro codegen,
  and diagnostic messages updated. `compose_kleisli` and `compose_kleisli_flipped`
  remain at dispatch module top level (no inference wrappers).
  `contramap_explicit` moved to `explicit::contramap` for consistency (follow-up
  commit).
- Step 5: Done. All 37 inference wrapper functions moved from
  `functions/*.rs` into their corresponding `dispatch/*.rs` modules.
  18 function files deleted. `functions.rs` re-exports from
  `crate::dispatch::*` instead of `self::*`. Compile-fail `.stderr`
  golden files updated (map path changed to `src/dispatch/functor.rs`).
  `functions/contravariant.rs` kept as-is.
- Step 6: Done. Created `fp-macros/src/analysis/dispatch.rs` with
  `DispatchTraitInfo`, `DispatchArrow`, `DispatchArrowParam` data
  structures and `analyze_dispatch_traits()` entry point. Integrated
  into `document_module_worker()` as Pass 1b after `get_context()`.
  Added `dispatch_traits` field to `Config`. 4 unit tests passing.
  The dispatch info is populated but not yet consumed by signature
  generation (steps 7-8).
- Steps 7-8: In progress.
  - 7-8a: Done. Fixed `format_brand_name()` to not strip "Brand" suffix
    when result would be empty.
  - 7-8b: Done. Added `brand_param`, `kind_trait_name` (extracted from
    Brand's `Kind_*` bound on the trait definition), and `container_params`
    (position-based mapping of container type params to element types) to
    `DispatchTraitInfo`.
  - 7-8c: Done. Replaced raw `String` arrow output with structured
    `ArrowOutput` enum (Plain, BrandApplied, OtherApplied). Added
    `classify_arrow_output()` function.
  - 7-8d: Done. Added `extract_container_params()` with position-based
    analysis of dispatch trait generic params. Detects element types
    (single-letter params) vs container types (multi-letter params
    between Brand and Marker).
  - 7-8e: Done. `build_synthetic_signature()` implemented. Constructs
    a `syn::Signature` with Brand param, Kind hash bounds, qualified
    path container types, Fn closure params, and structured return types.
    Delegates to existing `generate_signature()` pipeline. Old helper
    functions removed (`generate_dispatch_signature`,
    `find_inferable_brand_params`, `dispatch_arrow_to_hm`,
    `arrow_output_to_hm`, `type_to_param_hm`, `inferable_param_to_hm`,
    `build_dispatch_return_type`). Stale `#[expect(dead_code)]`
    annotations removed from fields now consumed.
  - 7-8f: In progress. Three major fixes applied:
    - ArrowOutput::Plain now stores raw Rust tokens (not HM-simplified).
      Fixes filter_map, partition_map, and other functions whose closures
      return compound types like Option<B>.
    - Container param mapping rewritten to use Val impl type arguments
      instead of naive ratio split. Fixes map showing Brand A B.
    - Return structure Of counting uses >::Of pattern instead of all Of
      occurrences. Fixes simple returns misclassified as Nested.
    - Closureless dispatch: InferableBrand-bounded params detected from
      original sig's where clause and substituted. Fixes alt/compact/join.
    - Original function params preserved (non-container, non-dispatch
      params like fold_right's initial: B kept as-is).
  - Constants: Moved hardcoded "Brand", "Marker", "FnBrand", "Dispatch"
    strings to `core/constants.rs`.
  - Nested Apply! handling verified: the existing pipeline recursively
    visits qualified paths and Apply! macros via the TypeVisitor pattern.
    No depth limit. Single-pass recursion resolves all nesting correctly.
    The synthetic signature constructs proper syn::Type trees that the
    visitor handles automatically.
- Steps 9-12: Not started.

### Steps 7-8f current output status

30 of 37 functions produce correct signatures. All string-based Apply!
macro parsing has been replaced with the proper token-stream parser
(`get_apply_macro_parameters`).

**Functions producing correct signatures (30):**

| Function                   | Signature                                                                                                                |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `map`                      | `forall Brand A B. Functor Brand => (A -> B, Brand A) -> Brand B`                                                        |
| `fold_right`               | `forall Brand A B. Foldable Brand => ((A, B) -> B, B, Brand A) -> B`                                                     |
| `fold_left`                | `forall Brand A B. Foldable Brand => ((B, A) -> B, B, Brand A) -> B`                                                     |
| `fold_map`                 | `forall Brand A M. (Foldable Brand, Monoid M) => (A -> M, Brand A) -> M`                                                 |
| `alt`                      | `forall Brand A. Alt Brand => (Brand A, Brand A) -> Brand A`                                                             |
| `compact`                  | `forall Brand A. Compactable Brand => Brand A -> Brand A`                                                                |
| `join`                     | `forall Brand A. Semimonad Brand => Brand A -> Brand A`                                                                  |
| `bind`                     | `forall Brand A B. Semimonad Brand => (Brand A, A -> Brand B) -> Brand B`                                                |
| `bind_flipped`             | `forall Brand A B. Semimonad Brand => (A -> Brand B, Brand A) -> Brand B`                                                |
| `filter`                   | `forall Brand A. Filterable Brand => (A -> bool, Brand A) -> Brand A`                                                    |
| `filter_map`               | `forall Brand A B. Filterable Brand => (A -> Option B, Brand A) -> Brand B`                                              |
| `partition`                | `forall Brand A. Filterable Brand => (A -> bool, Brand A) -> (Brand A, Brand A)`                                         |
| `partition_map`            | `forall Brand A E O. Filterable Brand => (A -> Result O E, Brand A) -> (Brand E, Brand O)`                               |
| `lift2`                    | `forall Brand A B C. Lift Brand => ((A, B) -> C, Brand A, Brand B) -> Brand C`                                           |
| `lift3`                    | `forall Brand A B C D. Lift Brand => ((A, B, C) -> D, Brand A, Brand B, Brand C) -> Brand D`                             |
| `lift4`                    | `forall Brand A B C D E. Lift Brand => ((A, B, C, D) -> E, Brand A, Brand B, Brand C, Brand D) -> Brand E`               |
| `lift5`                    | `forall Brand A B C D E G. Lift Brand => ((A, B, C, D, E) -> G, Brand A, Brand B, Brand C, Brand D, Brand E) -> Brand G` |
| `bimap`                    | `forall Brand A C B D. Bifunctor Brand => ((A -> B, C -> D), Brand A C) -> Brand B D`                                    |
| `bi_fold_left`             | `forall Brand A B C. Bifoldable Brand => (((C, A) -> C, (C, B) -> C), C, Brand A B) -> C`                                |
| `bi_fold_right`            | `forall Brand A B C. Bifoldable Brand => (((A, C) -> C, (B, C) -> C), C, Brand A B) -> C`                                |
| `bi_fold_map`              | `forall Brand A B M. (Bifoldable Brand, Monoid M) => ((A -> M, B -> M), Brand A B) -> M`                                 |
| `wither`                   | `forall Brand A M B. (Witherable Brand, Applicative M) => Brand A -> M (Brand B)`                                        |
| `map_with_index`           | `forall Brand A B. FunctorWithIndex Brand => ((Index, A) -> B, Brand A) -> Brand B`                                      |
| `fold_left_with_index`     | `forall Brand A B. FoldableWithIndex Brand => ((Index, B, A) -> B, B, Brand A) -> B`                                     |
| `fold_right_with_index`    | `forall Brand A B. FoldableWithIndex Brand => ((Index, A, B) -> B, B, Brand A) -> B`                                     |
| `fold_map_with_index`      | `forall Brand A M. (FoldableWithIndex Brand, Monoid M) => ((Index, A) -> M, Brand A) -> M`                               |
| `filter_with_index`        | `forall Brand A. FilterableWithIndex Brand => ((Index, A) -> bool, Brand A) -> Brand A`                                  |
| `filter_map_with_index`    | `forall Brand A B. FilterableWithIndex Brand => ((Index, A) -> Option B, Brand A) -> Brand B`                            |
| `partition_with_index`     | `forall Brand A. FilterableWithIndex Brand => ((Index, A) -> bool, Brand A) -> (Brand A, Brand A)`                       |
| `partition_map_with_index` | `forall Brand A E O. FilterableWithIndex Brand => ((Index, A) -> Result O E, Brand A) -> (Brand E, Brand O)`             |

**Remaining issues (7 functions):**

| Issue                                      | Functions                                             | Root cause                                                                                                                                                                                                                                                                             | Approach                                                                                                                        |
| ------------------------------------------ | ----------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Container shows `Brand B` not `Brand A`    | traverse, traverse_with_index                         | `extract_container_params` extracts element type `B` from Apply! args when it should extract `A`. The Apply! in the Val impl's type args for the container param IS `Apply!(<Brand>::Of<A>)` which should give `A`, but the parser may be picking up a different arg. Needs debugging. | Debug `extract_apply_type_args` output for traverse's container param.                                                          |
| Arrow output `()` for Apply! returns       | compose_kleisli, compose_kleisli_flipped, bi_traverse | Sub-arrow output types are `Apply!(<Brand>::Of<B>)` which `classify_arrow_output` should classify as `BrandApplied`. The parser may fail on the inner Apply! inside the Fn bound's return type (Apply! inside a parenthesized Fn arg context).                                         | Debug `classify_arrow_output` for these cases.                                                                                  |
| Second param `FA` not substituted          | apply_first, apply_second                             | Second param type is `<FA as ApplyFirstDispatch<...>>::FB` (associated type projection), not in container_map.                                                                                                                                                                         | Extract `type FB = Apply!(...)` from Val impl's associated type items. Resolve projections during synthetic signature building. |
| Missing closure param                      | wilt                                                  | Falls through to standalone macro entirely. Complex: FnBrand + Applicative secondary + tuple-inside-applicative return.                                                                                                                                                                | Debug why `find_dispatch_trait_in_sig` fails or `build_synthetic_signature` returns None.                                       |
| Container shows `Brand E` not correct type | separate                                              | InferableBrand fallback uses Tuple return's first element (`[E]`) as container element types, but the actual input container holds `Result<O, E>` not just `E`.                                                                                                                        | The container_map should have the correct mapping from `extract_container_params`. Check why it's not matching.                 |

### Step 7-8g: Regression tests for all 37 signatures

After all functions produce correct signatures, add regression tests
that assert on the exact HM output for every inference wrapper function.
These tests ensure future changes to the dispatch analysis, synthetic
signature builder, or HM pipeline do not silently break any signatures.

Approach: add test cases in `fp-macros/src/documentation/document_signature.rs`
(or a new test module) that:

1. Construct a minimal module containing a dispatch trait, its Val/Ref
   impls, and the inference wrapper function.
2. Run `document_module_worker` on it.
3. Extract the generated doc comment containing the HM signature.
4. Assert it matches the expected string exactly.

Alternatively, use `trybuild` or snapshot tests on the actual dispatch
module files, comparing the generated doc output against golden files.

The test should cover all 37 functions to catch regressions in:

- Brand variable naming
- Container param substitution
- Arrow type extraction (single, tuple, closureless)
- Return type structure (Plain, Applied, Nested, Tuple)
- Secondary constraints
- Associated type rendering (Index)
- Nested Apply! resolution

### Steps 7-8 revised approach: synthetic signature rewriting

The initial approach (building a custom `SignatureData` from scratch in
`generate_dispatch_signature()`) proved fragile:

- String-based `Of` counting in return types was unreliable (macro
  argument text inflates the count).
- Hardcoded brand variable name `"F"` caused collisions (e.g., traverse
  has an explicit `F: Applicative` param).
- Multi-container params (`FB`, `FC` in lift) weren't handled.
- Arrow output types containing `Apply!(...)` weren't simplified.

**Revised approach:** Instead of building `SignatureData` from scratch,
construct a **synthetic `syn::Signature`** that replaces dispatch
machinery with semantic equivalents, then call the existing
`generate_signature()` pipeline. The existing pipeline already handles
Apply! simplification, qualified path resolution, brand name formatting,
and constraint rendering correctly (as proven by the trait method
signatures in `classes/`).

**How it works for each function pattern:**

Simple (map):

- Original: `map<FA, A, B, Marker>(f: impl FunctorDispatch<...>, fa: FA) -> Apply!(<<FA as IB!>::Brand ...>::Of<B>)`
- Synthetic: `map<Brand: Functor, A, B>(f: impl Fn(A) -> B, fa: <Brand as Kind_hash>::Of<A>) -> <Brand as Kind_hash>::Of<B>`
- Pipeline output: `forall Brand A B. Functor Brand => (A -> B, Brand A) -> Brand B`

Traverse:

- Original: `traverse<FnBrand, FA, A, B, F: Kind, Marker>(func: impl TraverseDispatch<...>, ta: FA) -> Apply!(...)`
- Synthetic: `traverse<Brand: Traversable, A, B, F: Applicative>(func: impl Fn(A) -> <F as Kind_hash>::Of<B>, ta: <Brand as Kind_hash>::Of<A>) -> <F as Kind_hash>::Of<<Brand as Kind_hash>::Of<B>>`
- Pipeline output: `forall Brand A B F. (Traversable Brand, Applicative F) => (A -> F B, Brand A) -> F (Brand B)`

Lift2:

- Original: `lift2<FA, FB, A, B, C, Marker>(f: impl Lift2Dispatch<...>, fa: FA, fb: FB) -> Apply!(...)`
- Synthetic: `lift2<Brand: Lift, A, B, C>(f: impl Fn(A, B) -> C, fa: <Brand as Kind_hash>::Of<A>, fb: <Brand as Kind_hash>::Of<B>) -> <Brand as Kind_hash>::Of<C>`
- Pipeline output: `forall Brand A B C. Lift Brand => ((A, B) -> C, Brand A, Brand B) -> Brand C`

### Decisions

**Decision 1: Brand variable name (DECIDED)**

Use `Brand` as the type parameter name in the synthetic signature.
This matches the existing convention in non-dispatch free functions
(e.g., `join<'a, Brand: Semimonad, A>` renders as
`forall Brand A. Semimonad Brand => Brand (Brand A) -> Brand A`).

Requires a fix to `format_brand_name()`: currently `strip_suffix("Brand")`
on `"Brand"` produces `Some("")` (empty string). Add a guard so that
stripping is skipped when the result would be empty:

```rust
if let Some(stripped) = name.strip_suffix(BRAND_SUFFIX) {
    if stripped.is_empty() { name.to_string() } else { stripped.to_string() }
} else {
    name.to_string()
}
```

Secondary brand params (e.g., `F: Applicative` in traverse,
`M: Applicative` in witherable) keep their original names from the
dispatch trait definition.

**Decision 2: Kind hash fallback (DECIDED)**

The `Kind_*` hash is extracted from the Brand parameter's bounds in
the dispatch trait definition. Currently always directly visible
(convention, not enforced). If the Kind hash is not found (e.g., if a
future trait uses a supertrait instead of a direct Kind bound), skip
synthetic signature generation and leave `#[document_signature]` for
the standalone macro to handle. This graceful fallback means the system
never produces incorrect output; it just produces the less-clean
Phase 1 output (InferableBrand and Marker already filtered).

**Decision 3: Replace initial approach entirely (DECIDED)**

Replace `generate_dispatch_signature()` and all its helper functions
with `build_synthetic_signature()`. The initial approach's code is
removed entirely. Manual override (step 9) provides a safety net for
any remaining edge cases.

### Open Questions

**Open Question 1: Multi-container param handling (lift2-5)**

`lift2` has `fa: FA` and `fb: FB`. Both are containers of the same
brand. In the synthetic signature, both must become
`<Brand as Kind_hash>::Of<A>` and `<Brand as Kind_hash>::Of<B>`.

The problem: how does the builder know which element type each container
param maps to?

| Option                                          | Approach                                                                                                                                                                                                                                                                                                 | Pros                                                     | Cons                                                                            |
| ----------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------- |
| A: Name convention                              | `FA` -> applied to `A` (strip `F` prefix, rest is element type). `FB` -> applied to `B`.                                                                                                                                                                                                                 | Simple string manipulation                               | Fragile if naming convention changes; assumes F-prefix                          |
| B: Position-based from dispatch trait type args | The dispatch trait definition `Lift2Dispatch<'a, Brand, A, B, C, FA, FB, Marker>` establishes a positional mapping. Extract which trait type param is `Brand`, which are element types, and which are container types. Map by position: container params appear after element types and before `Marker`. | Robust; doesn't depend on names; correct by construction | More complex extraction logic; must understand the trait's generic param layout |
| C: Detect by absence of own bounds              | Params that have no bounds of their own (only appear as dispatch trait type args) are containers. Derive their element type from the dispatch trait's type arg ordering.                                                                                                                                 | Doesn't depend on names                                  | Could misidentify params; still needs position info for element type mapping    |

Recommendation: Option B is the most robust. The dispatch trait's type
param list is the ground truth for the mapping.

**Open Question 2: Complex arrow output types (bind, traverse closures)**

The closure in `bind` returns `Apply!(<Brand as Kind!(...)>::Of<'a, B>)`.
In the synthetic signature, this should become
`<Brand as Kind_hash>::Of<'a, B>` (which the pipeline simplifies to
`Brand B`).

The arrow output is currently stored as a raw string in `DispatchArrow`.
It needs to become a structured representation.

| Option                                                          | Approach                                                                                                                                                                                                                                                                                                                                                                                | Pros                                                                                           | Cons                                                                                                                                                                    |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A: Detect Apply!/Brand in raw string, substitute                | String-match or token-match the arrow output. Replace `Brand` with the brand variable name and construct a qualified path.                                                                                                                                                                                                                                                              | Straightforward for common patterns                                                            | String parsing; could be fragile with nested patterns                                                                                                                   |
| B: Store arrow output as structured data                        | During extraction, classify each arrow output as: plain type variable (e.g., `B`), brand-applied (e.g., `Brand B`), other-brand-applied (e.g., `F B` for traverse). Store as an enum. Construct the synthetic `syn::Type` from the structured data.                                                                                                                                     | Clean; no string parsing at generation time; parallels `ReturnStructure`                       | More complex `DispatchArrow` data structure; extraction logic must handle Apply! macro tokens                                                                           |
| C: Extract from trait method signature instead of impl Fn bound | The dispatch trait's `dispatch()` method already has parameter types expressed in terms of `Brand`, `A`, `B` etc. Instead of extracting the arrow from the impl block's `Fn(A) -> R` bound, extract it from the trait method's non-self parameter types and return type. The method params (minus `self` and `fa`) define the closure's inputs, and the return type defines the output. | Uses well-formed syn::Type values; no string parsing; directly reusable in synthetic signature | Different extraction source than current; the method params include `fa` which must be identified and excluded; closureless dispatch has no closure param in the method |

Recommendation: Option B provides the best balance. It keeps the current
extraction source (impl block Fn bounds, which are reliable) while
avoiding string manipulation at generation time. The structured
representation directly maps to syn::Type construction.

**Data required from `DispatchTraitInfo`:**

- `kind_trait_name: Option<String>` - the `Kind_*` hash from the Brand
  param's bound (e.g., `"Kind_cdc7cd43dac7585f"`). `None` if not found
  (triggers fallback to standalone macro).

- `return_structure: ReturnStructure` - already implemented. Classification
  of the dispatch method's return type (Plain, Applied, Nested).
  Needs a `Tuple` variant added (for separate, partition, wilt).

- Arrow type info - already stored as `DispatchArrow`. The `output`
  field (currently a raw `String`) should become structured data
  (pending open question 2 decision).

- Secondary constraints and type param names - already stored.

- Brand param name - already stored implicitly (always `"Brand"`
  currently). Used as the synthetic signature's brand type param name.

**Implementation changes (after open questions are resolved):**

Replace `generate_dispatch_signature()` and all its helper functions
(`type_to_param_hm`, `inferable_param_to_hm`, `build_dispatch_return_type`,
`find_inferable_brand_params`, `dispatch_arrow_to_hm`) with a single
`build_synthetic_signature()` function that:

1. Creates a new `syn::Signature` with `Brand` as a type param bounded
   by the semantic constraint and Kind hash.
2. Replaces the `impl *Dispatch<...>` parameter with `impl Fn(...) -> R`
   using the extracted arrow type (pending structured arrow output).
3. Replaces container parameters with `<Brand as Kind_hash>::Of<X>`
   qualified paths (pending multi-container mapping decision).
4. Constructs the return type from `ReturnStructure`.
5. Calls `generate_signature()` on this synthetic signature.

### Steps 7-8 output quality issues (initial approach, superseded)

The generated signatures are structurally correct but have refinement
issues. Current output vs ideal for representative functions:

| Function     | Current output                                                                         | Ideal                                                                          |
| ------------ | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `map`        | `forall F A B. Functor F => (A -> B, F A) -> G (F B)`                                  | `forall F A B. Functor F => (A -> B, F A) -> F B`                              |
| `fold_right` | `forall F A B. Foldable F => ((A, B) -> B, B, F A) -> B`                               | Correct                                                                        |
| `alt`        | `forall F A. Alt F => (F A, F A) -> G (F A)`                                           | `forall F A. Alt F => (F A, F A) -> F A`                                       |
| `lift2`      | `forall F FB A B C. Lift F => ((A, B) -> C, F A, FB) -> G (F C)`                       | `forall F A B C. Lift F => ((A, B) -> C, F A, F B) -> F C`                     |
| `bind`       | `forall F A B. Semimonad F => (A -> Apply!(...), F A) -> G (F B)`                      | `forall F A B. Semimonad F => (A -> F B, F A) -> F B`                          |
| `traverse`   | `forall F A B F. (Traversable F, Applicative F) => (A -> Apply!(...), F A) -> F (F B)` | `forall T A B F. (Traversable T, Applicative F) => (A -> F B, T A) -> F (T B)` |

**Issue 1: Return type shows `G (F B)` instead of `F B` for simple cases.**
Root cause: `extract_return_structure()` counts `Of` occurrences in the
trait's `dispatch()` method return type, but `Apply!(<Brand as Kind!(...)>::Of<'a, B>)`
contains `Of` inside the `Kind!(...)` macro argument string as well as
the actual application. This makes `of_count >= 2` for simple return
types, triggering the `Nested` branch. The `extract_non_brand_param()`
then finds a spurious outer brand from the macro text.

Possible fix: parse the `dispatch()` return type as a `syn::Type::Macro`
and inspect the token stream structurally rather than using string
matching on the rendered text.

**Issue 2: `FB`/`FC` not substituted to `F B`/`F C` in lift functions.**
Root cause: `find_inferable_brand_params()` correctly finds `FA` (it has
an `InferableBrand_*` where-clause bound), but `FB`/`FC`/`FD`/`FE` do
not have their own `InferableBrand_*` bounds. They are constrained
through the dispatch trait's type arguments, not independently. The
current approach only detects params with explicit `InferableBrand_*`
bounds.

Possible fix: detect container params by name pattern (starts with `F`
and is followed by a single uppercase letter) in addition to by
`InferableBrand_*` bounds. Or, extract the mapping from the dispatch
trait's type parameter list.

**Issue 3: Arrow output for bind/traverse contains raw `Apply!(...)` text.**
Root cause: the arrow type is extracted from the Val impl's `Fn(A) -> R`
bound. For bind, `R` is `Apply!(<Brand as Kind!(...)>::Of<'a, B>)` which
is stored as a raw string. The HM pipeline needs to simplify this to
`F B`.

Possible fix: when extracting the arrow output type, detect `Apply!`
patterns and simplify to `Brand arg1 arg2...` notation. Or, use the
same `ReturnStructure` approach for arrow output types.

**Issue 4: traverse has `F` name collision (container brand and applicative brand).**
Root cause: the brand variable is hardcoded to `"F"`. When traverse has
an explicit `F: Kind_*` type parameter (the applicative brand), both
the inferred container brand and the explicit applicative brand render
as `F`.

Possible fix: use different variable names for different roles. The
container brand could use `T` (for the traversable) and the applicative
could keep `F`. Detect the collision from the function's type parameter
list.

## Deviations

### Step 4

1. **`contramap` added to `explicit` module.** The plan originally said
   `contramap_explicit` would be handled in Phase 3. Instead, it was moved to
   `functions::explicit::contramap` immediately for API consistency, re-exporting
   from `classes::contravariant::contramap`. The `generate_function_re_exports!`
   macro's alias entry was replaced with an exclude entry.

2. **Additional affected areas not mentioned in plan:**
   - Macro codegen in `m_do!/a_do!` (`fp-macros/src/m_do/codegen.rs`,
     `fp-macros/src/a_do/codegen.rs`) emitted `bind_explicit::` and
     `map_explicit::` in generated code. Updated to `explicit::bind::` and
     `explicit::map::`.
   - `#[diagnostic::on_unimplemented]` note text in
     `fp-macros/src/hkt/trait_kind.rs` referenced `_explicit` suffix. Updated.
   - `.stderr` golden files for compile-fail tests
     (`tests/ui/result_no_inferable_brand.stderr`,
     `tests/ui/tuple2_no_inferable_brand.stderr`) contained the diagnostic
     note text and needed updating.
   - `generate_function_re_exports!` macro in `functions.rs` had a
     `contravariant::contramap` alias entry that needed migration.

### Step 5

1. **Compile-fail `.stderr` golden files needed updating.** The error
   messages reference the source location of the `map` function, which
   moved from `src/functions/functor.rs` to `src/dispatch/functor.rs`.
   Updated both `.stderr` files.

2. **No other deviations.** The merge was mechanical. All dispatch
   modules already imported `kinds::*` (which includes `InferableBrand_*`),
   so no additional imports were needed for the inference wrappers.

### Step 6

1. **`#[document_signature]` on free functions is not processed by
   `#[document_module]`.** The plan assumed `document_module` handles
   `#[document_signature]` on all items. In reality, it only processes
   the attribute on impl methods and trait methods (via `process_document_signature`
   in `generation.rs`). For `Item::Fn`, the attribute is left in the output
   and expanded later by the standalone `document_signature_worker` proc macro.
   This means the dispatch-aware signature generation (steps 7-8) must either:
   (a) have `document_module` intercept `#[document_signature]` on `Item::Fn`
   items during Pass 2 (where it has Config with dispatch info), or
   (b) find another way to pass dispatch info to the standalone macro.
   Option (a) is recommended; it mirrors how impl methods are already handled.

2. **`DispatchTraitInfo` fields marked `#[expect(dead_code)]`.** The struct
   fields are populated in step 6 but not consumed until steps 7-8. Using
   `#[expect]` instead of `#[allow]` so the compiler flags the suppression
   as unnecessary once the fields are consumed.

3. **`is_tuple_closure` detected false positives for unit types.** The
   initial implementation matched `Type::Tuple(_)` which includes `()`
   (unit type used as the self type in closureless dispatch test cases).
   Fixed by checking `tuple.elems.len() >= 2`.

4. **`LiftFn` added to infrastructure traits.** The `FnBrand: LiftFn`
   bound appears in foldable/traversable dispatch impls. Without filtering
   it, `find_brand_param` would incorrectly identify `FnBrand` as the
   Brand parameter (since `LiftFn` passes the `is_semantic_type_class`
   check). Added `LiftFn` and `SendLiftFn` to `INFRASTRUCTURE_TRAITS`.

### Steps 7-8

1. **`#[document_module]` must intercept `#[document_signature]` on
   `Item::Fn` items.** The plan assumed `document_module` already handles
   `#[document_signature]` on all items. It only processes the attribute
   on impl methods and trait methods. For `Item::Fn`, the attribute was
   left for the standalone `document_signature_worker` proc macro. The
   solution: `process_fn_dispatch_signature()` in `generation.rs` now
   checks for dispatch trait references and intercepts the attribute
   during Pass 2, removing it so the standalone macro does not also
   process it. Non-dispatch functions still fall through to the standalone
   macro.

2. **`ReturnStructure` added to `DispatchTraitInfo`.** The plan originally
   proposed parsing the wrapper function's return type to determine the
   HM return type. This proved fragile because the wrapper's return type
   contains nested `Apply!` and `InferableBrand!` macros whose token
   text confuses string-based `Of` counting. The solution: extract the
   return structure from the dispatch _trait's_ `dispatch()` method,
   which uses `Brand` directly (no InferableBrand noise). The
   `ReturnStructure` enum (Plain, Applied, Nested) encodes the structure
   for direct HM generation without string parsing.

3. **String-based `Of` counting proved unreliable.** The initial approach
   counted `Of` substrings in the return type text to distinguish simple
   (`F B`) from nested (`G (F B)`) returns. This fails because `Apply!`
   macro arguments (e.g., `Kind!(type Of<'a, T: 'a>: 'a;)`) contain
   `Of` as part of the signature definition text, inflating the count.
   Even counting `Of <` (with angle bracket) picks up `Of` inside macro
   arguments. The trait-definition-based approach (`ReturnStructure`)
   avoids this entirely.

4. **`#[expect(dead_code)]` on `trait_name` field.** After steps 7-8
   consume most `DispatchTraitInfo` fields, `trait_name` remains unused
   (stored for diagnostics). Kept with `#[expect(dead_code)]`.

5. **Open questions resolved: position-based container mapping and
   structured arrow output.** Both recommended approaches adopted.
   `extract_container_params()` uses position-based analysis of the
   dispatch trait's generic param list to map container params to element
   types. `ArrowOutput` enum replaces raw string storage. Both
   integrated into `DispatchTraitInfo` with `#[expect(dead_code)]`
   until consumed by `build_synthetic_signature()`.

6. **Hardcoded strings moved to constants.** `"Brand"`, `"Marker"`,
   `"FnBrand"`, `"Dispatch"` moved to `core/constants.rs` as
   `DEFAULT_BRAND_PARAM`, `MARKER_PARAM`, `FN_BRAND_PARAM`,
   `DISPATCH_SUFFIX`. All usages in `dispatch.rs` and `generation.rs`
   updated.

7. **Synthetic signature builder implemented (step 7-8e).**
   `build_synthetic_signature()` constructs a `syn::Signature` with
   Brand param, Kind hash bounds, qualified path container types, Fn
   closure params, and structured return types. The existing
   `generate_signature()` pipeline handles the rest. Old helper
   functions removed entirely (code in git history).
   `#[expect(dead_code)]` annotations removed from fields now consumed.
   Tuple closure handling is a placeholder (returns `()` param) pending
   further work.

8. **`extract_return_structure` still uses string-based Of counting.**
   The `ReturnStructure` approach was intended to avoid string parsing,
   but the extraction function itself still counts `Of` substrings in
   the trait method's return type text. This is unreliable for the same
   reason it was unreliable in the wrapper function (Apply! macro
   arguments contain `Of`). The trait method return type is a
   `syn::Type::Macro` that needs structural parsing, not string matching.
   This is the primary remaining extraction issue.

9. **`extract_container_params` rewritten to use Val impl type args.**
   The position-based heuristic was replaced with extraction from the
   Val impl's trait type arguments. For each Apply! macro arg in the
   impl, the element types are extracted from the Of<'a, ElementType>
   pattern. This is correct and robust.

10. **Nested Apply! handling verified correct.** The existing HM pipeline
    recursively visits qualified paths and Apply! macros via the
    TypeVisitor pattern. `visit_path` recurses on QSelf types and each
    generic argument. `visit_macro` recurses on both brand and args.
    No depth limit; single-pass recursion resolves all nesting. This
    means the synthetic signature approach handles traverse's
    `<F as Kind>::Of<'a, <Brand as Kind>::Of<'a, B>>` correctly as
    `F (Brand B)` without any special multi-pass logic.

11. **Three extraction fixes applied in one commit.** Container param
    mapping (Val impl args), return structure Of counting (>::Of pattern),
    and closureless dispatch (InferableBrand fallback) were all fixed
    together. The param builder now transforms the original function's
    params instead of building from scratch, preserving non-container
    non-dispatch params like fold_right's `initial: B`.

12. **ArrowOutput::Plain stores raw Rust tokens.** Changed from
    HM-simplified text (e.g., "Option B") to raw token text (e.g.,
    "Option < B >"). This enables `syn::parse_str` to reconstruct
    the type for the synthetic signature. Without this fix, functions
    with compound closure return types (filter_map, partition_map, etc.)
    fell through to the standalone macro.

## Prerequisites

- Analysis document: `docs/plans/hm-signature-rendering/analysis.md`
- Brand inference plan (implemented): `docs/plans/brand-inference/plan.md`
- Dispatch expansion plan (implemented): `docs/plans/dispatch-expansion/plan.md`

## Motivation

The `#[document_signature]` proc macro produces broken HM type signatures for
all 37 inference wrapper functions in `functions/`. The rendered signatures
expose internal machinery (`InferableBrand_cdc7cd43dac7585f`, dispatch trait
names, `Marker`, `FnBrand`, `macro` return types) instead of clean
Haskell-style signatures. This is the primary user-facing API surface and the
documentation is currently unusable for understanding function types.

## Goals

1. All 37 inference wrapper functions render correct, clean HM signatures.
2. No manual per-function configuration or annotation required for common cases.
3. The fix is self-maintaining: adding a new dispatch trait and wrapper
   automatically produces correct signatures.
4. A manual override mechanism exists as a safety net for edge cases.

## Design

### Three-phase approach

**Phase 1 (Approach A):** Quick-win filtering. Add `InferableBrand_*` to
filtered traits, hide `Marker` and `FnBrand` from `forall`. Independently
shippable, immediately improves all signatures.

**Phase 2 (Approach G1):** Architectural fix. Move inference wrapper functions
into dispatch modules, placing dispatch functions in an `explicit` submodule
to avoid naming conflicts. Extend `#[document_module]` to perform cross-item
analysis: extract arrow types and semantic type class constraints from dispatch
trait `impl` blocks, then use that information when generating HM signatures
for wrapper functions in the same module.

**Phase 3 (Approach E):** Add manual override support to `#[document_signature]`
as a safety net for any functions that resist automatic handling.

### Module structure after Phase 2

Each dispatch module will contain the dispatch trait, its impl blocks, the
inference wrapper function, and an `explicit` submodule containing the dispatch
free function:

```rust
// fp-library/src/dispatch/functor.rs
#[fp_macros::document_module]
pub(crate) mod inner {
    use { ... };

    // -- Dispatch trait and impl blocks (unchanged) --

    pub trait FunctorDispatch<'a, Brand, A, B, FA, Marker> {
        fn dispatch(self, fa: FA) -> Apply!(...);
    }

    impl<'a, Brand, A, B, F>
        FunctorDispatch<'a, Brand, A, B, Apply!(...), Val> for F
    where
        Brand: Functor,
        F: Fn(A) -> B + 'a,
    { ... }

    impl<'a, 'b, Brand, A, B, F>
        FunctorDispatch<'a, Brand, A, B, &'b Apply!(...), Ref> for F
    where
        Brand: RefFunctor,
        F: Fn(&A) -> B + 'a,
    { ... }

    // -- Inference wrapper (moved from functions/functor.rs) --

    pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
        f: impl FunctorDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, B, FA, Marker>,
        fa: FA,
    ) -> Apply!(...)
    where
        FA: InferableBrand_cdc7cd43dac7585f,
    {
        f.dispatch(fa)
    }

    // -- Dispatch free function in explicit submodule --

    pub mod explicit {
        use super::*;

        pub fn map<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker>(
            f: impl FunctorDispatch<'a, Brand, A, B, FA, Marker>,
            fa: FA,
        ) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
        { f.dispatch(fa) }
    }
}

pub use inner::*;
```

### Re-export structure after Phase 2

The `functions` module is the sole user-facing API surface. The `dispatch`
module is an implementation detail that houses the traits, impl blocks, and
function definitions but does not present a flat API.

```rust
// fp-library/src/dispatch.rs
//
// NO flat re-exports. Only module declarations.
// (The existing flat re-exports like `pub use functor::map` are removed.
//  They were the primary API before brand inference was implemented;
//  now they are redundant with `functions::*`.)

pub mod alt;
pub mod apply_first;
// ... etc (module declarations only)
```

```rust
// fp-library/src/functions.rs
//
// The sole user-facing API surface.

// Inference wrappers (from dispatch modules, top-level of each)
pub use crate::dispatch::{
    functor::map,
    semimonad::{bind, bind_flipped, join, compose_kleisli, compose_kleisli_flipped},
    // ... etc
};

// Explicit dispatch functions (from explicit submodules, flat aggregation)
pub mod explicit {
    pub use crate::dispatch::functor::explicit::map;
    pub use crate::dispatch::semimonad::explicit::{bind, bind_flipped, join};
    pub use crate::dispatch::foldable::explicit::{fold_left, fold_right, fold_map};
    pub use crate::dispatch::lift::explicit::{lift2, lift3, lift4, lift5};
    // ... ~37 entries total
}

// Functions without dispatch (unchanged)
pub use self::contravariant::contramap;
```

User-facing API paths:

- `fp_library::functions::map` - inference wrapper (primary API)
- `fp_library::functions::explicit::map` - dispatch with turbofish
- `fp_library::functions::map_explicit` - removed (breaking change)
- `fp_library::dispatch::map` - removed (no flat re-exports in dispatch)
- `fp_library::dispatch::functor::map` - inference wrapper (internal path)
- `fp_library::dispatch::functor::explicit::map` - dispatch function (internal path)

### `#[document_module]` cross-item analysis

During Pass 1 (context extraction), `#[document_module]` will:

1. Scan module items for traits whose names end with `Dispatch`.
2. For each dispatch trait, find its `impl` blocks (in the same module).
3. From the Val impl block (identified by scanning all trait type arguments
   for `Val` by name, not by position):
   - Extract `Brand: TypeClass` from both inline generic parameter bounds
     and the where clause (merge both sources) -> semantic constraint.
   - Extract `F: Fn(A, ...) -> R` from both inline and where clause bounds
     -> arrow type.
   - Extract secondary constraints (e.g., `F: Applicative` for traverse).
4. Store this in `Config` as a `HashMap<String, DispatchTraitInfo>`.

Note: the analysis does not assume marker position or bound location style.
Currently all dispatch traits use `Val`/`Ref` as the last type argument and
place all bounds in the where clause, but the analysis is robust to either
convention.

During Pass 2 (documentation generation), when processing a function with
`impl SomeDispatch<...>` in its parameters:

1. Look up `SomeDispatch` in the dispatch trait info map.
2. Replace the dispatch trait rendering with the extracted arrow type.
3. Emit the semantic type class as a constraint on the container variable.
4. Apply FA -> F A substitution using the extracted type parameter info.
5. Resolve Apply!/InferableBrand! return types using the trait's return type.

### Manual override (Phase 3)

`#[document_signature]` gains an optional string argument:

```rust
#[document_signature("forall F A B. Functor F => (A -> B, F A) -> F B")]
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(...) -> ... { ... }
```

When the string is provided, the macro emits it directly as the signature doc
comment, bypassing the generation pipeline. This is for `contramap` (no
dispatch trait) and any future edge cases.

## Implementation Steps

### Phase 1: Targeted filtering

#### Step 1: Filter `InferableBrand_*` from HM constraints

Add `InferableBrand_` prefix check to `classify_trait()` in
`fp-macros/src/analysis/traits.rs`:

```rust
n if n.starts_with(markers::INFERABLE_BRAND_PREFIX) => TraitCategory::Kind,
```

**Files changed:** `fp-macros/src/analysis/traits.rs`

**Verification:** `just verify`. Check that `InferableBrand_*` no longer appears
in rendered signatures.

#### Step 2: Filter `Marker` and `FnBrand` from `forall`

In `format_generics()` in `fp-macros/src/documentation/document_signature.rs`,
filter type parameters named `Marker` or `FnBrand` from the `forall` variable
list.

Add constants to `fp-macros/src/core/constants.rs`:

```rust
pub mod markers {
    // ... existing ...
    pub const HIDDEN_TYPE_PARAMS: &[&str] = &["Marker", "FnBrand"];
}
```

In `format_generics()`, when building the `type_vars` list, skip parameters
whose name is in `HIDDEN_TYPE_PARAMS`.

**Files changed:** `fp-macros/src/core/constants.rs`,
`fp-macros/src/documentation/document_signature.rs`

**Verification:** `just verify`. Check that `Marker` and `FnBrand` no longer
appear in `forall` clauses.

#### Step 3: Add unit tests for Phase 1

Add test cases to the existing signature test module in
`fp-macros/src/documentation/document_signature.rs` that verify:

1. `InferableBrand_*` bounds are filtered from constraints.
2. `Marker` type parameter is excluded from `forall`.
3. `FnBrand` type parameter is excluded from `forall`.
4. Non-infrastructure type parameters with similar names (e.g., a user type
   called `MarkerTrait`) are NOT filtered.

**Files changed:** `fp-macros/src/documentation/document_signature.rs`

**Verification:** `just test -p fp-macros`

### Phase 2: Module restructure and cross-item analysis

#### Step 4: Move dispatch functions into `explicit` submodules

For each dispatch module file in `fp-library/src/dispatch/`:

1. Wrap the existing dispatch free function(s) in a `pub mod explicit { ... }`
   block inside the existing `mod inner`.
2. Add `use super::*;` at the top of the `explicit` module. This imports all
   public items from the parent `mod inner` (dispatch traits, types, etc.).
   Macros like `Apply!` and `Kind!` are expanded by the compiler and do not
   need importing. Note: `use super::*` does NOT re-export private `use`
   imports from the parent, but the explicit functions only need public items.
3. Keep the dispatch trait and impl blocks at the top level of `mod inner`.
4. Ensure each dispatch module file retains `pub use inner::*;` at the bottom
   (all currently have this). This is required for the `explicit` submodule
   to be accessible from outside the file (e.g.,
   `crate::dispatch::functor::explicit::map`). Without it, the `explicit`
   submodule would be trapped inside the `pub(crate) mod inner` scope.

Remove flat re-exports from `fp-library/src/dispatch.rs`:

- Delete the `pub use { functor::map, semimonad::bind, ... }` block
  (lines 115-173). These were the primary API before brand inference was
  implemented; they are now redundant. The `dispatch` module retains only
  its `pub mod` declarations.

Update re-exports in `fp-library/src/functions.rs`:

- Remove the `_explicit` alias re-exports (lines 193-236).
- Add a `pub mod explicit` that re-exports from
  `crate::dispatch::*::explicit::*` (flat aggregation, ~37 entries).

Update all internal call sites (~85 occurrences) that reference dispatch
functions by `_explicit` paths or `crate::dispatch::map`-style flat paths:

- `fp-library/src/types/vec.rs`: 2 calls to `crate::dispatch::map` ->
  `crate::dispatch::functor::explicit::map`.
- `fp-library/src/dispatch.rs` tests (~52 calls): `super::functor::map` ->
  `super::functor::explicit::map` (etc.).
- Test files (`do_notation.rs`, `ado_notation.rs`): `bind_explicit` ->
  `explicit::bind` (etc.).
- Doc examples in dispatch modules and `functions.rs`: update to use
  `functions::explicit::map` paths.
- Internal code in `types/tuple_1.rs` (~12 uses), `types/tuple_2.rs` (~16
  uses), `types/fn_brand.rs` (~2 uses): update `_explicit` paths.

**Files changed:** All 18 files in `fp-library/src/dispatch/`,
`fp-library/src/dispatch.rs`, `fp-library/src/functions.rs`,
`fp-library/src/types/vec.rs`, `fp-library/src/types/tuple_1.rs`,
`fp-library/src/types/tuple_2.rs`, `fp-library/src/types/fn_brand.rs`,
test files, doc examples.

**Verification:** `just verify`. Grep for `_explicit` and `crate::dispatch::`
flat paths to confirm no stale references remain.

#### Step 5: Move inference wrappers into dispatch modules

For each function module in `fp-library/src/functions/` (except
`contravariant.rs`):

1. Move the inference wrapper function(s) from the function module into the
   corresponding dispatch module's `mod inner` block (at the top level,
   alongside the dispatch trait).
2. Move any doc comments, attributes (`#[document_signature]`,
   `#[document_type_parameters]`, etc.) along with the function.
3. Add any additional imports the wrapper needs (e.g., `InferableBrand_*`,
   `InferableBrand!`, `Kind!`) to the dispatch module's import block.
4. Remove the now-empty function module files.

Update `fp-library/src/functions.rs`:

- Remove `mod` declarations for the deleted function modules.
- Update re-exports to reference inference wrappers from their new locations
  in `crate::dispatch::*`.

Keep `fp-library/src/functions/contravariant.rs` as-is (no dispatch trait).

**Files changed:** All 18 files in `fp-library/src/dispatch/`, all 18 files
in `fp-library/src/functions/` (deleted), `fp-library/src/functions.rs`,
`fp-library/src/classes.rs` (if needed for imports).

**Verification:** `just verify`.

#### Step 6: Add dispatch trait analysis to `#[document_module]`

Create `fp-macros/src/analysis/dispatch.rs`:

1. Define `DispatchTraitInfo` struct:

   ```rust
   pub struct DispatchTraitInfo {
       /// e.g., "FunctorDispatch"
       pub trait_name: String,
       /// e.g., "Functor" (the primary semantic type class)
       pub semantic_constraint: String,
       /// Secondary constraints on other type params (e.g., "Applicative" on F
       /// for traverse, "Applicative" on M for witherable)
       pub secondary_constraints: Vec<(String, String)>,
       /// Arrow type extracted from the Fn bound, or None for closureless
       pub arrow_type: Option<HmAst>,
       /// Whether this is a closureless dispatch (alt, compact, join, etc.)
       pub closureless: bool,
       /// Brand arity (1 for most types, 2 for bifunctor)
       pub brand_arity: u8,
       /// Return type structure
       pub return_structure: ReturnStructure,
   }

   pub enum ReturnStructure {
       /// F B (most functions)
       Simple { element_types: Vec<String> },
       /// (F E, F O) (partition, separate, wilt)
       Tuple(Vec<ReturnStructure>),
       /// G (F B) (traverse, wither, bi_traverse)
       Nested { outer: String, inner: Box<ReturnStructure> },
       /// B, M (fold operations returning a plain type, not wrapped)
       Plain(String),
   }
   ```

2. Implement `analyze_dispatch_trait()`: given a trait item whose name ends
   with `Dispatch`, extract its generic parameters.

3. Implement `analyze_dispatch_impl()`: given an impl block for a dispatch
   trait, extract:
   - The marker type (Val/Ref) by scanning all trait type arguments by name,
     not by position.
   - The Brand type parameter, identified by having a type-class bound (a
     bound matching a known type class name or a non-marker, non-Fn trait),
     not by assuming the parameter is named `Brand`.
   - `Fn` bounds identified by their `Fn*` trait, not by parameter name
     (closure parameters may be named `F`, `G`, `Func`, etc.). Use the
     existing `get_fn_type_from_bound()` which already identifies by trait.
   - Merge bounds from both inline generic parameter bounds and the where
     clause.
   - Secondary constraints on non-Brand type parameters (e.g.,
     `M: Applicative` for witherable, `F: Applicative` for traverse).
   - The return type structure from the dispatch method's return type.

4. Implement `link_wrapper_to_dispatch()`: given a function with
   `impl *Dispatch<...>` in its signature, look up the trait info and return
   the association.

Add to `fp-macros/src/core/config.rs`:

```rust
pub dispatch_traits: HashMap<String, DispatchTraitInfo>,
```

Integrate into `fp-macros/src/documentation/document_module.rs`:

- In Pass 1 (after `get_context`): call `analyze_dispatch_traits()` on all
  module items, populate `config.dispatch_traits`.
- In Pass 2: when processing a function item, check if it has an
  `impl *Dispatch<...>` parameter. If so, call the signature generator with
  the dispatch trait info to produce the correct HM signature.

**Files changed:** New file `fp-macros/src/analysis/dispatch.rs`,
`fp-macros/src/analysis.rs` (module declaration),
`fp-macros/src/core/config.rs`,
`fp-macros/src/documentation/document_module.rs`,
`fp-macros/src/documentation/document_signature.rs` (or new generation logic).

**Verification:** `just verify`. Inspect rendered docs for all 37 functions.

#### Step 7: Handle FA -> F A substitution and associated types

In the dispatch-aware signature generator (from Step 6):

1. When a type parameter has an `InferableBrand` bound (detected in Step 1),
   and the dispatch trait analysis provides the element type mapping
   (e.g., `FA` maps to `Brand::Of<A>`), substitute `FA` with `F A` in the
   HM output.
2. For arity-2 brands, substitute `FA` with `P A C` (using the dispatch
   trait's type parameters to determine argument order).
3. For `&FA` (borrowed containers), render as `F A` (the reference is a
   Rust detail, not part of the HM type).
4. For associated types in Fn bounds (e.g., `Fn(Brand::Index, A) -> B` in
   FunctorWithIndex), render the associated type as just the type name
   without the path prefix: `Brand::Index` -> `Index`. The HM signature
   becomes `(Index, A) -> B`.

**Files changed:** `fp-macros/src/documentation/document_signature.rs` or the
new dispatch-aware generator.

**Verification:** `just verify`. Check that `forall F A B` appears instead of
`forall FA A B`. Check that `Brand::Index` renders as `Index`.

#### Step 8: Handle Apply!/InferableBrand! return types

In the dispatch-aware signature generator:

1. When the function's return type contains `Apply!` with nested
   `InferableBrand!`, use the dispatch trait's return type info to construct
   the correct HM return type directly, bypassing the current macro parsing.
2. For simple cases: `Apply!(<<FA as IB!(...)>::Brand as Kind!(...)>::Of<B>)`
   becomes `F B`.
3. For nested cases: `Apply!(<G>::Of<Apply!(<<FA as IB!(...)>::...>::Of<B>)>)`
   becomes `G (F B)`.
4. For tuple cases: `(Apply!(...E), Apply!(...O))` becomes `(F E, F O)`.

**Files changed:** Same as Step 7.

**Verification:** `just verify`. Check that return types render correctly
instead of `macro`.

### Phase 3: Manual override

#### Step 9: Add string argument to `#[document_signature]`

Modify `document_signature_worker()` in
`fp-macros/src/documentation/document_signature.rs`:

1. Parse the attribute's token stream. If non-empty, treat it as a string
   literal containing the complete HM signature.
2. When a string is provided, skip the signature generation pipeline and emit
   the string directly as the doc comment.
3. When no string is provided, use the existing (now enhanced) pipeline.

Apply the override to `contramap` in
`fp-library/src/functions/contravariant.rs` (the one function without a
dispatch trait).

**Files changed:** `fp-macros/src/documentation/document_signature.rs`,
`fp-library/src/functions/contravariant.rs`.

**Verification:** `just verify`.

#### Step 10: Add unit tests for Phase 3

Add test cases verifying:

1. `#[document_signature("forall A B. (B -> A, F A) -> F B")]` emits the
   provided string.
2. `#[document_signature]` (no argument) still generates automatically.
3. Edge case: empty string argument is rejected with a compile error.

**Files changed:** `fp-macros/src/documentation/document_signature.rs`

**Verification:** `just test -p fp-macros`

### Finalization

#### Step 11: Update documentation

1. Update `fp-library/docs/dispatch.md` to reflect the new module structure
   (inference wrappers in dispatch modules, `explicit` submodule).
2. Update `fp-library/docs/architecture.md` section on free function
   distribution to describe the three-layer structure:
   `classes/ -> dispatch/ (traits + inference wrappers + explicit/) -> functions.rs (re-exports)`.
3. Update `fp-library/docs/project-structure.md` module dependency diagram.
4. Update `CLAUDE.md` if any conventions change.

**Files changed:** `fp-library/docs/dispatch.md`,
`fp-library/docs/architecture.md`, `fp-library/docs/project-structure.md`,
`CLAUDE.md`.

**Verification:** `just doc --workspace --all-features --open`. Visually
inspect rendered docs.

#### Step 12: Update plan

Mark all steps as done. Document any deviations from the plan.

## Verification Strategy

After each step: `just verify` (fmt, check, clippy, doc, test).

After Phase 2: visually inspect rendered documentation for all 37 functions.
Verify each signature matches the "After Tier 2" column in the analysis
document's appendix.

## Breaking Changes

- `fp_library::functions::map_explicit` path removed. Replaced by
  `fp_library::functions::explicit::map`. Same for all 37 `_explicit` variants.
- `crate::dispatch::map` flat path removed. The dispatch module no longer
  re-exports functions at the module level. Access dispatch functions via
  `crate::dispatch::functor::explicit::map` (nested) or
  `fp_library::functions::explicit::map` (flat, user-facing).
- The `functions/` submodule directory is removed (except `contravariant.rs`).
  Functions are re-exported at `fp_library::functions::*` unchanged.

These are pre-1.0 API refinements. The flat re-exports in `dispatch.rs` were
the primary API before brand inference was implemented; they are now superseded
by `functions::*` (inference) and `functions::explicit::*` (dispatch).

## Agent Review Findings

### Resolved questions

1. **`#[document_module]` nested module support:** Confirmed. It recursively
   processes nested modules in all 3 passes. `apply_to_nested_modules` passes
   mutable Config in Pass 1 and immutable Config in Pass 2. The `explicit`
   submodule's functions will have access to dispatch trait info.

2. **`use super::*` sufficiency:** Confirmed. Dispatch modules already import
   `kinds::*` (which includes `InferableBrand_*` traits) and `fp_macros::*`.
   The `explicit` submodule gets everything it needs via `use super::*;`.

3. **Import compatibility:** Both dispatch and function modules import
   `kinds::*` and `fp_macros::*`. The inference wrapper's imports are a
   subset of what the dispatch module already has. No additional imports
   needed beyond removing the now-unnecessary `crate::dispatch::functor::FunctorDispatch`.

4. **Validation of explicit functions:** `#[document_module]` validates
   `#[document_examples]` on all `pub fn` items, including in nested modules.
   Explicit functions must keep their documentation attributes when moved
   into the `explicit` submodule.

5. **Dispatch analysis ordering:** Should happen after `get_context()` in
   Pass 1. No dependency on projections or defaults. No ordering issues
   possible (traits must be defined before impls in Rust).

6. **Reusable code:** `get_fn_type_from_bound()` (95% reusable for Fn bound
   extraction), `classify_trait()` (reusable for trait categorization),
   `ErrorCollector` (standard error handling pattern).

### Issues found

1. **~85 `_explicit` path usages need migration.** Tests, internal code,
   doc examples, and benchmarks reference `map_explicit`, `bind_explicit`,
   etc. All must be updated to `explicit::map`, `explicit::bind`, etc. in
   Step 4. This is mechanical but must be comprehensive.

2. **`compose_kleisli` and `compose_kleisli_flipped` have no inference
   wrappers.** They take explicit Brand parameters and have no
   `InferableBrand` bounds. Decision: keep them at the dispatch module
   top level (alongside the inference wrappers) and re-export from
   `functions.rs` directly. They don't belong in the `explicit` submodule
   because they ARE the only variant (no inference wrapper exists).

3. **`crate::dispatch::map` path removed.** The flat re-exports in
   `dispatch.rs` are deleted entirely (see Resolved Design Decisions).
   No ambiguity remains.

4. **Return type variant detection (Step 8).** The wrapper function's
   `Marker` parameter is inferred at call site (Val or Ref). The macro
   cannot determine which variant statically. Resolution: store both Val
   and Ref return types in `DispatchTraitInfo`. Since both variants have
   the same HM return type structure (only the Rust types differ, not the
   HM representation), emit the Val variant's return type. The HM signature
   `F B` is the same whether the Rust type is `Brand::Of<'a, B>` or
   `Brand::Of<'a, B>` (they're identical at the HM level).

5. **`contramap` stays in `functions/contravariant.rs`.** Document this
   exception in `functions.rs` module docs. Phase 3's manual override
   (Approach E) handles its HM signature.

6. **New code estimate:** ~650-900 LOC across `analysis/dispatch.rs`,
   Config additions, `document_module.rs` integration, and signature
   generation logic.

## Resolved Design Decisions

1. **`dispatch.rs` flat re-exports: removed.** The flat `pub use` block in
   `dispatch.rs` (lines 115-173) was the primary API before brand inference.
   It is now redundant. `dispatch.rs` retains only `pub mod` declarations.
   The `functions` module is the sole user-facing API surface.

2. **`explicit` aggregation: only in `functions.rs`.** The flat
   `functions::explicit::*` path is the ergonomic user-facing API. Power
   users can access `dispatch::functor::explicit::map` via nested paths.
   Only one re-export list to maintain.

3. **`compose_kleisli` / `compose_kleisli_flipped`: top-level in dispatch
   module.** They have no inference wrappers, so they stay at the dispatch
   module top level (not in `explicit`) and are re-exported directly from
   `functions.rs`.
