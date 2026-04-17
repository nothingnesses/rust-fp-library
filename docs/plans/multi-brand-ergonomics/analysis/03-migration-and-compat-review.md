---
title: Migration and Compatibility Review
reviewer: Agent 3
date: 2026-04-17
scope: InferableBrand removal, attribute rename, explicit:: changes, doctest updates, migration ordering
---

# Migration and Compatibility Review

## 1. Scale of InferableBrand removal

### 1.1. Codebase surface area

`InferableBrand` appears in 170 occurrences across 21 files under `fp-library/src/`
alone, plus significant usage in `fp-macros/src/` (the macro crate), documentation,
tests, and POCs. The breakdown:

- **19 dispatch modules** (`fp-library/src/dispatch/*.rs`) contain `InferableBrand`
  in inference wrapper signatures, doc comments, and `Apply!` return types.
- **`fp-library/src/kinds.rs`** defines the trait family itself.
- **`fp-macros/src/hkt/impl_kind.rs`** generates `InferableBrand` impls.
- **`fp-macros/src/hkt/trait_kind.rs`** generates `InferableBrand` trait definitions.
- **`fp-macros/src/hkt/apply.rs`** contains `resolve_inferable_brand()` which
  preprocesses `InferableBrand!(SIG)` inside `Apply!` invocations.
- **`fp-macros/src/lib.rs`** exports the `InferableBrand!` proc macro.
- **`fp-macros/src/documentation/generation.rs`** uses `is_dispatch_container_param()`
  which checks for `InferableBrand_` prefixed bounds.
- **`fp-macros/src/core/constants.rs`** defines `INFERABLE_BRAND_MACRO`.
- **6 docs files** reference `InferableBrand` in explanatory text.
- **README.md** references `InferableBrand` twice.
- **3 integration test files** (`brand_inference_feasibility.rs`,
  `closureless_dispatch_poc.rs`, `slot_select_brand_poc.rs`) use it.
- **2 UI test files** (`result_no_inferable_brand.rs`,
  `tuple2_no_inferable_brand.rs`) plus their `.stderr` snapshots.
- **`fp-macros/tests/document_module_tests.rs`** uses `#[no_inferable_brand]`.
- **`fp-macros/src/documentation/signature_snapshot_tests.rs`** has hardcoded
  `InferableBrand_abc123` in expected output strings.

This is not a quick find-and-replace. The removal touches trait definitions,
macro generation, macro preprocessing, doc generation heuristics, snapshot
tests, integration tests, UI tests, and prose documentation.

## 2. Sites using `<FA as InferableBrand>::Brand` directly

### Finding

Every inference wrapper in `dispatch/` uses the pattern
`<FA as InferableBrand_HASH>::Brand` to project the brand from the container type.
These appear in three positions:

1. **Dispatch trait bounds**: e.g.,
   `f: impl FunctorDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, B, FA, Marker>`.
2. **Where clauses**: e.g., `FA: InferableBrand_cdc7cd43dac7585f`.
3. **Return types via `Apply!`**: e.g.,
   `Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!(...)>::Of<'a, B>)`.

All 19 dispatch modules exhibit this pattern. I count approximately 80+ distinct
occurrences of `<FA as InferableBrand_...>::Brand` across inference wrappers.

Additionally, the `explicit::` submodules inside several dispatch files also
use `<FA as InferableBrand_...>::Brand` in their signatures (e.g., `explicit::map`,
`explicit::bind`, `explicit::join`, `explicit::compact`, `explicit::separate`,
`explicit::alt`, `explicit::apply_first`, `explicit::apply_second`,
`explicit::contramap`).

### Issue

Under the plan, all of these must change from:

```
FA: InferableBrand_HASH
<FA as InferableBrand_HASH>::Brand
```

to:

```
FA: Slot_HASH<'a, Brand, A>
Brand  (as a free type parameter)
```

The return type pattern also changes. Today:

```
Apply!(<<FA as InferableBrand!(...)>::Brand as Kind!(...)>::Of<'a, B>)
```

After: the Brand is a direct type parameter, so the `Apply!` nesting simplifies to:

```
Apply!(<Brand as Kind!(...)>::Of<'a, B>)
```

**Sub-issue A: The `InferableBrand!` macro invocation form inside `Apply!`.**
The `Apply!` macro contains a preprocessing step (`resolve_inferable_brand()`)
that scans for `InferableBrand!(SIG)` tokens and replaces them with the hashed
identifier. After InferableBrand removal, this preprocessing code becomes dead
code. But the plan does not mention removing it or repurposing it.

Approaches:

1. **Remove `resolve_inferable_brand()` entirely.** Since `InferableBrand!` the
   macro is also removed, no token stream will ever contain `InferableBrand!(...)`
   patterns. This is the cleanest approach but requires verifying no external or
   test code calls `Apply!` with the old nesting pattern.
   - Trade-off: simple, but breaking for any code still using the old `Apply!` form.

2. **Keep `resolve_inferable_brand()` as a no-op compatibility shim.** If the
   preprocessing encounters `InferableBrand!` tokens, emit a compile error with a
   migration message.
   - Trade-off: more helpful error, slightly more code to maintain.

3. **Leave it in place but unreachable.** Since `InferableBrand!` the proc macro
   is removed, `resolve_inferable_brand()` will never encounter the token pattern.
   It becomes dead code that does nothing harmful.
   - Trade-off: no risk, but dead code accumulates.

**Recommendation:** Option 1. Remove `resolve_inferable_brand()` and the
`INFERABLE_BRAND_MACRO` constant. Also remove the `generate_inferable_brand_name()`
function from `fp-macros/src/hkt/canonicalizer.rs` and all its call sites. Since
this is a pre-1.0 library, clean removal is preferred over shims.

**Sub-issue B: `explicit::` functions that use `<FA as InferableBrand>::Brand`.**
The plan (Decision F) states that `explicit::map` will be rewritten to bound on
`Slot` with Brand pinned via turbofish. But the plan's integration surface table
only mentions `explicit::map` and the inference wrappers for each dispatch module.
It does not enumerate the full set of explicit functions that need rewriting.

From the codebase search, the following `explicit::` modules exist (19 total):

- functor, semimonad, bifunctor, foldable, foldable_with_index,
  filterable, filterable_with_index, traversable, traversable_with_index,
  bitraversable, bifoldable, witherable, lift, compactable, contravariant,
  alt, apply_first, apply_second.

Every one of these has an explicit function that takes Brand as a type parameter.
However, not all of them currently use InferableBrand. Some already take Brand as
a turbofish parameter (e.g., `compose_kleisli`, `compose_kleisli_flipped`). The
explicit functions that DO use `<FA as InferableBrand_...>::Brand` include:

- `explicit::map`, `explicit::bimap`, `explicit::bind`, `explicit::bind_flipped`,
  `explicit::join`, `explicit::traverse`, `explicit::traverse_with_index`,
  `explicit::bi_traverse`, `explicit::fold_left`, `explicit::fold_right`,
  `explicit::fold_map`, `explicit::fold_left_with_index`,
  `explicit::fold_right_with_index`, `explicit::fold_map_with_index`,
  `explicit::bi_fold_left`, `explicit::bi_fold_right`, `explicit::bi_fold_map`,
  `explicit::filter`, `explicit::filter_map`, `explicit::partition`,
  `explicit::partition_map`, `explicit::filter_with_index`,
  `explicit::filter_map_with_index`, `explicit::partition_with_index`,
  `explicit::partition_map_with_index`, `explicit::wilt`, `explicit::wither`,
  `explicit::lift2`, `explicit::lift3`, `explicit::lift4`, `explicit::lift5`,
  `explicit::compact`, `explicit::separate`, `explicit::alt`,
  `explicit::apply_first`, `explicit::apply_second`,
  `explicit::contramap`, `explicit::map_with_index`.

That is approximately 37 explicit functions that need rewriting.

Approaches:

1. **Rewrite all explicit functions to use `Slot` bounds.** This is the plan's
   stated direction. Every explicit function replaces `FA: InferableBrand_HASH`
   with `FA: Slot_HASH<'a, Brand, A>` and Brand becomes a turbofish parameter.
   - Trade-off: large mechanical change, but uniform and correct.

2. **Phase explicit rewrites separately.** Do inference wrappers first in phase 1,
   explicit functions in phase 2. This lets phase 1 verify Slot works before
   touching the explicit fallback path.
   - Trade-off: intermediate state where inference wrappers use Slot but explicit
     functions still use InferableBrand. This is impossible if InferableBrand is
     removed in phase 1 (step 5).

3. **Rewrite explicit functions to take Brand directly without Slot bounds.**
   Since explicit functions already take Brand as turbofish, they could bypass
   Slot entirely and bound directly on `Brand: Functor` (or whichever type class).
   - Trade-off: explicit functions would not benefit from Slot's Marker mechanism,
     but they do not need it since Brand is already specified.

**Recommendation:** Option 1 for consistency, but note the ordering constraint:
since phase 1 step 5 removes InferableBrand, all explicit functions must be
rewritten before or simultaneously with that step. The plan's phase 1 only
mentions `explicit::map` (step 7), but all 37 explicit functions must be updated
in phase 1 as well. This is an omission in the plan.

## 3. `#[no_inferable_brand]` -> `#[multi_brand]` rename

### Finding

The `#[no_inferable_brand]` attribute appears in:

- **10 `impl_kind!` invocations** in source code:
  - `fp-library/src/types/result.rs` (2 uses: lines 530, 1069).
  - `fp-library/src/types/tuple_2.rs` (2 uses: lines 445, 1211).
  - `fp-library/src/types/pair.rs` (2 uses: lines 814, 1600).
  - `fp-library/src/types/control_flow.rs` (2 uses: lines 1172, 1722).
  - `fp-library/src/types/try_thunk.rs` (2 uses: lines 965, 1700).

- **2 uses in test code**: `fp-macros/tests/document_module_tests.rs` (lines 91, 99).

- **Macro parsing code**: `fp-macros/src/hkt/impl_kind.rs` (lines 202-203, 363-367)
  checks for `"no_inferable_brand"` by string comparison.

- **Documentation** (6+ files reference the attribute by name).

### Issue

The rename itself is straightforward, but has a subtle semantic expansion. Today,
`#[no_inferable_brand]` means "suppress InferableBrand impl generation." The
renamed `#[multi_brand]` means "generate multiple Slot impls (one per brand)."
These have different cardinalities: the old attribute suppresses one thing,
the new attribute triggers generation of N things (one Slot impl per brand variant).

The plan does not specify how `impl_kind!` determines which brands to generate
Slot impls for when `#[multi_brand]` is present. Today, the macro simply skips
InferableBrand generation. Under the new semantics, it needs to emit Slot impls
for each brand, but the information about which brands exist for a given type is
not present in the `impl_kind!` invocation site. Each `impl_kind!` call handles
one brand at a time:

```rust
impl_kind! {
    #[no_inferable_brand]
    for ResultErrAppliedBrand<E> {
        type Of<'a, A: 'a>: 'a = Result<A, E>;
    }
}
```

This means `#[multi_brand]` does not need to know about other brands. It just
needs to generate a Slot impl for this specific brand. The "multi" in the name
indicates that the type has multiple brands, not that this invocation generates
multiple impls. This is potentially confusing but functionally correct.

Approaches:

1. **Rename directly to `#[multi_brand]`.** Simple mechanical change. Update 10
   source uses, 2 test uses, macro parsing code, and documentation.
   - Trade-off: name might confuse contributors into thinking the attribute
     generates multiple Slot impls per invocation.

2. **Rename to `#[multi_brand]` and add doc comments explaining semantics.**
   Same as option 1 but with explicit documentation in the macro that clarifies
   "this brand is one of several for its target type; generate a direct Slot impl
   rather than relying on the InferableBrand blanket."
   - Trade-off: slightly more work, significantly less confusion.

3. **Use a different name like `#[direct_slot]` that describes what it does rather
   than what the type is.** This avoids the semantic gap between "this type has
   multiple brands" and "generate a Slot impl here."
   - Trade-off: less intuitive for users who think in terms of "is this type
     single-brand or multi-brand?", but more accurate about the code generation
     effect.

**Recommendation:** Option 2. The name `#[multi_brand]` reads well at the use
site (`impl_kind!` for `ResultErrAppliedBrand` with `#[multi_brand]` clearly
communicates "Result has multiple brands"). But the macro documentation should
clarify that the attribute causes a direct Slot impl to be generated for this
brand, and that single-brand types also get Slot impls (just via a different
generation path).

## 4. Impact on existing doctests, examples, and UI tests

### 4.1. UI tests

Two UI test files must be deleted:

- `fp-library/tests/ui/result_no_inferable_brand.rs` (tests that `map` on
  `Result` fails because `Result` lacks `InferableBrand`).
- `fp-library/tests/ui/tuple2_no_inferable_brand.rs` (same for tuples).

Both of these test the exact behavior that the plan is designed to change.
After the migration, `map(|x: i32| x + 1, Ok::<i32, String>(5))` should
succeed rather than fail. The `.stderr` snapshot files must also be deleted.

New UI tests are needed (plan phase 1, step 8):

- Positive: closure-directed success for single and multi-brand.
- Negative: diagonal case (`Result<T, T>`) produces a compile error.
- Negative: unannotated closure on multi-brand produces a compile error.

### 4.2. Doctests in dispatch modules

Every inference wrapper function has doc comments containing `InferableBrand`
references. There are 37 doc-comment occurrences of `InferableBrand` across
dispatch modules (each inference wrapper says "inferred via `InferableBrand`").
These are prose references, not runnable doc tests, so they will not cause
compilation failures. However, they will be incorrect documentation after
the migration.

The plan mentions updating `map`'s doc comment (phase 3, step 3) but does not
mention updating the doc comments on the other 36+ inference wrapper functions.

Approaches:

1. **Update all 37 doc comments in a single pass.** This is mechanical: replace
   "via `InferableBrand`" with "via `Slot`" and update the explanation of how
   brand inference works.
   - Trade-off: tedious but necessary for correctness.

2. **Defer doc updates to phase 3.** Since these are prose, not code, the library
   compiles and tests pass with stale docs.
   - Trade-off: ships with incorrect documentation if phases are tested
     independently.

3. **Add a phase 1.5 doc-sweep step.** Between phase 1 and phase 2, update all
   dispatch module doc comments that reference InferableBrand.
   - Trade-off: clean intermediate state, but adds work to the critical path.

**Recommendation:** Option 1, done as part of the phase 1 rewrite of each
dispatch module. When rewriting the inference wrapper signature, update the doc
comment in the same change. This keeps each file internally consistent.

### 4.3. Existing doctests in explicit:: functions

The explicit function doctests use turbofish with Brand types directly (e.g.,
`explicit::map::<OptionBrand, _, _, _, _>(...)`). These do not reference
InferableBrand and should continue to work if the turbofish shape does not
change. However, Decision F states that explicit functions will be rewritten to
bound on Slot, which may change the number of turbofish parameters.

Today's `explicit::map` signature (from `functor.rs`):

```
pub fn map<'a, Brand: Kind_HASH, A: 'a, B: 'a, FA, Marker>(...)
```

This already takes Brand as the first turbofish parameter. If the new signature
keeps Brand in the same position, existing turbofish call sites survive. But if
Slot introduces additional type parameters, the turbofish count may change.

**Issue:** The plan claims the turbofish surface "contracts" (Decision F:
"only Brand is user-specified; the rest is inferred through Slot"). But the
current explicit functions already have Brand as the first parameter. If
anything, adding Slot bounds could increase the parameter count (e.g., adding
an `A` parameter that was previously inferred from InferableBrand). The plan
should clarify whether existing `explicit::map::<Brand, _, _, _, _>` call sites
survive unchanged.

### 4.4. Signature snapshot tests in fp-macros

`fp-macros/src/documentation/signature_snapshot_tests.rs` contains hardcoded
expected strings with `InferableBrand_abc123`. These are unit tests for the
HM signature rendering system and will fail immediately when InferableBrand
is removed.

### 4.5. HM signature rendering system

The `is_dispatch_container_param()` function in
`fp-macros/src/documentation/generation.rs` (line 956) checks whether a type
parameter's where-clause bound starts with `"InferableBrand_"`. This drives
the HM signature rendering that converts raw Rust signatures into readable
documentation. After InferableBrand removal, this heuristic must be updated
to check for `Slot_` bounds instead (or use a different detection strategy).

If this is not updated, the rendered HM signatures for all inference wrapper
functions will regress, showing raw type parameters instead of branded container
types.

Approaches:

1. **Update the heuristic to check for `Slot_` prefixed bounds.** Direct
   replacement: `name.starts_with("Slot_")`.
   - Trade-off: simple, mirrors the current approach.

2. **Use a more robust detection strategy** such as a custom attribute or
   marker on Slot-bounded parameters.
   - Trade-off: more resilient to future renames, but more invasive.

**Recommendation:** Option 1. The `Slot_` prefix is deterministic (generated
from the same hash as `Kind_`), so the string-prefix check remains reliable.

## 5. The `InferableBrand!` macro invocation form

### Finding

The `InferableBrand!` proc macro is declared in `fp-macros/src/lib.rs` and
serves one purpose: resolving a signature to its hashed `InferableBrand_HASH`
identifier name. It is used in two contexts:

1. **Standalone**: `InferableBrand!(type Of<'a, A: 'a>: 'a;)` produces the
   identifier `InferableBrand_cdc7cd43dac7585f`.
2. **Nested inside `Apply!`**: The `Apply!` macro preprocesses its input to
   resolve `InferableBrand!(SIG)` before parsing.

After removing the InferableBrand trait family, the `InferableBrand!` macro has
no valid output to produce. All downstream uses nested inside `Apply!` must be
rewritten to use Brand directly.

### Issue

The plan does not explicitly mention removing the `InferableBrand!` proc macro
from `fp-macros/src/lib.rs`. Phase 1 step 5 says "Remove `InferableBrand_*`
trait family and all impls" but does not mention the macro itself.

Approaches:

1. **Remove the `InferableBrand!` macro entirely.** This is the natural
   consequence of removing the trait family.
   - Trade-off: any external code using `InferableBrand!(...)` breaks with a
     "macro not found" error, which is clear enough.

2. **Replace the macro with a compile-error stub.** `InferableBrand!(...)` emits
   `compile_error!("InferableBrand! has been replaced by Slot; see migration guide")`.
   - Trade-off: more helpful error message for downstream users.

**Recommendation:** Option 1 for a pre-1.0 library. The removal is part of a
major API change that will already require reading the changelog. A stub adds
maintenance burden for marginal benefit.

## 6. Impact on downstream users

### Finding

The library is pre-1.0 and the plan explicitly states API-breaking changes are
acceptable. However, the concrete breakage for downstream code is:

1. **Inference wrappers (`map`, `bind`, etc.):** For single-brand types
   (`Option`, `Vec`, `Identity`, etc.), existing call sites like
   `map(|x| x + 1, Some(5))` should continue to work unchanged, since
   Slot covers the same inference for single-brand types. No breakage here.

2. **`explicit::` functions:** Existing turbofish call sites like
   `explicit::map::<OptionBrand, _, _, _, _>(f, fa)` may break if the turbofish
   parameter count or order changes. The plan should provide the exact new
   signature to verify compatibility.

3. **Multi-brand types that previously used `explicit::`:** Users who wrote
   `explicit::map::<ResultErrAppliedBrand<String>, _, _, _, _>(f, Ok(5))` will
   gain the ability to write `map(|x: i32| x + 1, Ok::<i32, String>(5))` instead.
   The explicit path should still work.

4. **Code that imports `InferableBrand_*` directly:** Any downstream code that
   bounded on `FA: InferableBrand_cdc7cd43dac7585f` or projected
   `<FA as InferableBrand_HASH>::Brand` will break. This is rare for external
   users but possible.

5. **Code using the `InferableBrand!` macro:** Breaks with "macro not found."

6. **Code referencing `#[no_inferable_brand]` in their own `impl_kind!` calls:**
   Any downstream crate that defines custom brands with `#[no_inferable_brand]`
   will get an "unknown attribute" error until they rename to `#[multi_brand]`.

## 7. Migration ordering constraints

### Finding

The plan specifies phase 1 steps 1-9, but the ordering has implicit
dependencies that are not fully explicit. Here is the dependency graph:

```
Step 1 (Add Slot trait family)
  |
  v
Step 2 (trait_kind! emits Slot)  <-- depends on Step 1
  |
  v
Step 3 (impl_kind! emits Slot impls)  <-- depends on Steps 1, 2
  |
  v
Step 4 (Rename attribute)  <-- depends on Step 3 (macro parsing code)
  |
  v
Step 5 (Remove InferableBrand)  <-- depends on Steps 6, 7, and ALL explicit rewrites
  |
Step 6 (Rewrite map inference wrapper)  <-- depends on Steps 1-3
Step 7 (Rewrite explicit::map)  <-- depends on Steps 1-3
```

### Issue A: Step 5 blocks on ALL dispatch rewrites, not just map

Step 5 removes InferableBrand entirely. But if only `map` and `explicit::map`
are rewritten (steps 6-7), the other 18 dispatch modules still reference
InferableBrand and will fail to compile. The plan says phase 2 handles the
remaining operations, but Decision B3 says all phases ship together.

If phases are implemented sequentially on a development branch (as stated),
there will be a period during phase 1 where InferableBrand is removed but
only `map` is migrated. The branch will not compile during this window.

Approaches:

1. **Move step 5 (InferableBrand removal) to after phase 2.** This way,
   InferableBrand and Slot coexist during phases 1-2, and removal happens
   only after all dispatch modules are migrated.
   - Trade-off: requires the coexistence to actually work. The plan rejects
     this under Decision D/A2 (the blanket from InferableBrand to Slot fails
     coherence). But coexistence does not require a blanket; both trait
     families can simply exist independently, with dispatch modules gradually
     migrating from one to the other.

2. **Rewrite all 19 dispatch modules (inference + explicit) in phase 1
   before removing InferableBrand.** Treat step 5 as the final step of
   phase 1, after all dispatch modules are migrated.
   - Trade-off: phase 1 becomes much larger (37+ explicit functions + 19+
     inference wrappers), but the branch always compiles after each step.

3. **Accept that the branch does not compile between steps.** Since it is
   a development branch, intermediate non-compiling states are acceptable
   as long as the final state compiles.
   - Trade-off: loses the ability to run tests at intermediate points, which
     the plan values ("Internal phasing gives a testbed for each operation").

**Recommendation:** Option 1. Keep InferableBrand present (but no longer used
by migrated modules) through phases 1-2. Remove it as the first step of phase 3.
This preserves the testability of intermediate states. The coexistence does not
require a blanket impl; it just means both traits exist, and dispatch modules
reference whichever one they have been migrated to. Newly migrated modules use
Slot; un-migrated ones still use InferableBrand. This is safe because the two
trait families are independent (no blanket between them under Decision A2).

### Issue B: The `Apply!` macro's InferableBrand preprocessing

If InferableBrand is removed before all `Apply!` invocations that nest
`InferableBrand!(...)` are updated, the `Apply!` macro will produce invalid
code (referencing a non-existent trait). Since `Apply!` invocations appear in
the return types of all inference wrappers and many explicit functions, these
must all be updated before or simultaneously with InferableBrand removal.

This reinforces the recommendation above: defer InferableBrand removal until
after all dispatch modules are migrated.

### Issue C: Documentation ordering

The plan places documentation updates in phase 3, but
`fp-library/docs/brand-dispatch-traits.md` currently describes a three-trait
design where `InferableBrand_*` and `Slot_*` coexist with a blanket bridging
them. The plan (Decision D) eliminates InferableBrand entirely, contradicting
this document. Under the recommended migration (keep InferableBrand through
phases 1-2), this doc would be accurate during implementation but stale by
phase 3. Updating it in phase 3 is fine.

However, the document's Slot design describes `type Out<B: 'a>: 'a` (a GAT
for slot replacement), while the plan's Slot design uses `type Marker` (Val/Ref
discrimination). These are structurally different. The existing
`brand-dispatch-traits.md` document is already outdated with respect to the
plan and should be updated early to avoid confusing implementers.

## 8. Phased release strategy testability

### Finding

Decision B3 says all phases ship together. The plan also says "phases are
implemented sequentially internally" to give "a testbed for each operation."

### Issue

As noted in section 7, if InferableBrand is removed in phase 1 step 5, the
branch does not compile until all dispatch modules are migrated. This
undermines the "testbed for each operation" rationale.

Furthermore, the plan does not specify what "testbed" means concretely. Is
it `just test`? `just verify`? Just compilation? Property-based tests?

Approaches:

1. **Define explicit checkpoints.** After each phase, the full `just verify`
   pipeline must pass. This requires the ordering changes recommended in
   section 7 (defer InferableBrand removal).
   - Trade-off: requires maintaining coexistence, but provides strong
     confidence at each stage.

2. **Accept compilation-only checkpoints.** After each step within a phase,
   `just check` passes. Full tests run only at phase boundaries.
   - Trade-off: faster iteration, but risks introducing subtle bugs.

3. **No formal checkpoints.** The branch may break during implementation;
   only the final state matters.
   - Trade-off: fastest implementation, but no safety net.

**Recommendation:** Option 1. After completing each phase, `just verify` should
pass. This is the strongest signal that the migration is correct and requires
the deferred-removal ordering from section 7.

## 9. Additional items not addressed by the plan

### 9.1. Missing dispatch modules: `apply` and `ref_apply`

There is no `dispatch/semiapplicative.rs` module. The `apply` and `ref_apply`
functions currently live in `fp-library/src/classes/semiapplicative.rs` and
`fp-library/src/classes/ref_semiapplicative.rs` as methods on the type class
traits, not as dispatch-pattern functions. The plan (phase 2) lists `apply`,
`ref_apply`, `apply_first`, `apply_second` for Slot migration, but
`apply_first` and `apply_second` already have dispatch modules while `apply`
and `ref_apply` do not.

Decision H (POC 8) validates that Slot-based inference works for apply, but
the plan does not mention creating a new dispatch module for semiapplicative.
This is either an intentional omission (apply/ref_apply stay as class methods
and get Slot bounds added there) or an oversight (a new dispatch module needs
to be created).

**Recommendation:** Clarify in the plan whether `apply`/`ref_apply` will get
dispatch modules analogous to `apply_first.rs`/`apply_second.rs`, or whether
Slot bounds will be added to the existing class-level methods.

### 9.2. The `compose_kleisli` and `compose_kleisli_flipped` functions

These already take Brand as a turbofish parameter and do not use InferableBrand.
The plan lists them under phase 2 for Slot migration, but they may not need
migration at all since they already bypass InferableBrand.

**Recommendation:** Verify whether compose_kleisli needs any changes. If not,
remove it from the phase 2 list to reduce scope.

### 9.3. Arity-2 InferableBrand (hash `266801a817966495`)

Bifunctor, bifoldable, and bitraversable dispatch modules use
`InferableBrand_266801a817966495` (the arity-2 variant). The plan discusses
Slot at arity 2 for `bimap` (validated by POC 6) but does not explicitly
mention the arity-2 InferableBrand removal. The same migration steps apply,
but implementers should be aware that two InferableBrand hash variants exist.

### 9.4. fp-macros CHANGELOG.md

The `fp-macros/CHANGELOG.md` references InferableBrand in several entries
describing past changes. These are historical records and should not be
modified. But new changelog entries should document the removal.
