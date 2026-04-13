# Plan: HM Signature Rendering Fix

## Current Progress

- Steps 1-3: Done. Phase 1 complete.
- Step 4: Done. All 18 dispatch modules have `explicit` submodules. Flat
  re-exports removed from `dispatch.rs`. `functions.rs` has `pub mod explicit`
  with nested path re-exports. Old `_explicit` alias block removed. All ~85
  internal call sites, doc examples, benchmark files, test files, macro codegen,
  and diagnostic messages updated. `compose_kleisli` and `compose_kleisli_flipped`
  remain at dispatch module top level (no inference wrappers). `contramap_explicit`
  left as-is (special case, no dispatch trait).
- Steps 5-12: Not started.

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
