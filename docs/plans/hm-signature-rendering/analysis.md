# HM Signature Rendering Analysis

## Problem Statement

The `#[document_signature]` proc macro generates Hindley-Milner type signatures
for functions in the rendered documentation. After the brand-inference and
dispatch-expansion work, the inference wrapper functions in `functions/` produce
broken or misleading HM signatures. This affects all 37 public functions in the
inference API, which is the primary user-facing surface of the library.

Example: the `map` function currently renders as:

```text
forall FA A B Marker. InferableBrand_cdc7cd43dac7585f FA => (FunctorDispatch FA A B FA Marker, FA) -> macro
```

The ideal rendering would be:

```text
forall F A B. Functor F => (A -> B, F A) -> F B
```

## Scope

### Affected functions

All 37 public functions in `fp-library/src/functions/`:

- **19 files**, every file affected.
- **37 functions total**: 32 use arity-1 `InferableBrand_cdc7cd43dac7585f`, 5 use
  arity-2 `InferableBrand_266801a817966495`.
- **36 of 37** use a `*Dispatch` trait for their closure/container parameter.
  The exception is `contramap` which calls the trait method directly.
- **35 of 37** have a `Marker` type parameter.
- **20 of 37** use `Apply!` with nested `InferableBrand!` in the return type.
- **10 of 37** have an additional `WithIndex` bound on the inferred brand.

### Function categories

| Category               | Count | Examples                                                  | Distinguishing feature                        |
| ---------------------- | ----- | --------------------------------------------------------- | --------------------------------------------- |
| Closure-based dispatch | 23    | `map`, `bind`, `fold_right`, `filter`, `traverse`, `wilt` | Single closure + container(s)                 |
| Closureless dispatch   | 7     | `alt`, `compact`, `separate`, `join`, `apply_first`       | No closure; container type drives dispatch    |
| Multi-closure          | 8     | `lift2`-`lift5`, `bimap`, `bi_fold_left`, `bi_traverse`   | Multiple closures (tuple or separate params)  |
| Direct trait call      | 1     | `contramap`                                               | No dispatch trait; calls Contravariant method |

### Not affected

- `_explicit` variants in `dispatch/` (re-exports, same underlying functions).
- Trait methods in `classes/` (these have clean signatures without dispatch or
  brand inference machinery).
- Proc macro docs in `fp-macros/` (already `ignore`, different issue).

## HM Pipeline Architecture

Understanding the existing pipeline is essential for evaluating approaches. The
`#[document_signature]` macro transforms Rust function signatures through these
stages:

### Stage 1: Entry and parsing (`document_signature_worker`)

Parses the item into `RustAst`, validates no duplicate attributes, gets cached
configuration via `get_config()`, calls `generate_signature()`.

### Stage 2: Generic analysis (`analyze_generics`)

Extracts from the function signature:

- `generic_names: HashSet<String>` - type parameter names
- `fn_bounds: HashMap<String, HmAst>` - maps function-typed variables to their
  HM arrow types (e.g., `F` -> `Arrow(Variable("A"), Variable("B"))`)

Function-typed bounds are detected by `get_fn_type_from_bound()`, which calls
`classify_trait()` to check if a bound is `Fn*` or `FnBrand`:

```rust,ignore
match classify_trait(&name, config) {
    TraitCategory::FnTrait => Some(trait_bound_to_hm_arrow(...)),
    TraitCategory::FnBrand => Some(HmAst::Variable(FN_BRAND_MARKER)),
    _ => None,
}
```

Variables with `Fn*` bounds are excluded from the `forall` clause and their
arrow types are inlined into the parameter list.

### Stage 3: Constraint formatting (`format_generics`)

Processes all type parameters and where-clause predicates:

- Collects type variables for the `forall` clause (filtering out function-typed
  variables)
- Extracts trait constraints using `classify_trait()` and `format_trait_bound()`
- Filters constraints by category:

```rust,ignore
match classify_trait(&trait_name, config) {
    TraitCategory::FnTrait | TraitCategory::FnBrand | TraitCategory::Kind => None,
    TraitCategory::Other(name) => {
        if config.ignored_traits().contains(&name) { None }
        else { Some(format!("{name} {type_var}")) }
    }
}
```

### Stage 4: Type conversion (`type_to_hm` via `HmAstBuilder`)

The core transformation engine uses the visitor pattern to convert Rust types to
HM AST nodes. Key handlers:

- **`visit_path`**: Processes type paths. Strips smart pointers (`Box`, `Arc`,
  `Rc`), applies brand name formatting (strip `Brand` suffix, apply config
  mappings), checks concrete types set.
- **`visit_macro`**: Detects `Apply!` invocations, calls
  `get_apply_macro_parameters()` to extract brand and type arguments. Falls back
  to `HmAst::Variable("macro")` on parse failure.
- **`visit_impl_trait`**: Extracts the first trait bound from `impl Trait`
  parameters. Converts via `trait_bound_to_hm_type()`.
- **Qualified paths**: Detects `<F as Kind!(...)>::Of<A>` patterns and builds
  constructor chains.

### Stage 5: Assembly (`SignatureData::fmt`)

Builds the final string: `forall {vars}. {constraints} => {params} -> {return}`.

### Key extension points

The pipeline has clear injection points for new intelligence:

1. **`classify_trait()`** (`analysis/traits.rs:32-44`): Hardcoded match on trait
   name. Adding new categories here is trivial.
2. **`format_generics()`** (`document_signature.rs:173-220`): Decides which type
   parameters appear in `forall` and which constraints are visible.
3. **`visit_impl_trait()`** (`hm/ast_builder.rs`): Handles `impl Trait`
   parameters. Currently treats non-Fn traits as type constructors.
4. **`visit_macro()`** (`hm/ast_builder.rs`): Handles macro invocations in types.
   Currently only handles `Apply!` with simple qualified paths.
5. **`get_fn_type_from_bound()`** (`analysis/traits.rs:50-68`): Decides if a
   bound represents a function type. Currently only recognizes `Fn*` and
   `FnBrand`.

### What the macro cannot access

Proc macros operate on token streams only. They cannot:

- Inspect actual trait implementations or definitions
- Resolve type aliases to their underlying types
- Validate that a bound actually exists in scope
- Access module structure or crate graph

All intelligence must be derived from naming conventions, structural patterns
in the token stream, or explicit configuration. However, `#[document_module]`
processes entire modules and can perform cross-item analysis within a single
module (see Approach G).

## Automatic Detectability Assessment

A key question is whether the issues can be addressed automatically (based on
structural patterns) or require manual per-function configuration. Research
across all 37 functions shows:

### Fully automatic (regex/pattern-based)

| Pattern                       | Detection method                   | Confidence |
| ----------------------------- | ---------------------------------- | ---------- |
| `InferableBrand_[0-9a-f]{16}` | Prefix match on `InferableBrand_`  | 100%       |
| `Kind_[0-9a-f]{16}`           | Prefix match on `Kind_` (existing) | 100%       |
| Dispatch trait suffix         | Name ends with `Dispatch`          | 100%       |
| `Marker` type parameter       | Name is exactly `Marker`           | 100%       |
| `FnBrand` type parameter      | Name is exactly `FnBrand`          | 100%       |

### Requires structural analysis

| Pattern                                | What needs analyzing                      | Feasibility |
| -------------------------------------- | ----------------------------------------- | ----------- |
| Dispatch trait -> semantic type class  | Strip `Dispatch` suffix                   | High        |
| Dispatch trait -> arrow type           | Extract Fn bounds from the dispatch impl  | Medium      |
| `Apply!` with nested `InferableBrand!` | Token stream pattern matching             | Medium      |
| `FA` -> `F A` substitution             | Cross-reference InferableBrand + dispatch | Medium      |

### Dispatch trait naming is deterministic

Every dispatch trait follows the pattern `{Operation}Dispatch`, and the operation
maps directly to a type class:

| Dispatch trait                  | Semantic type class | Operation        |
| ------------------------------- | ------------------- | ---------------- |
| `FunctorDispatch`               | `Functor`           | `map`            |
| `BindDispatch`                  | `Semimonad`         | `bind`           |
| `JoinDispatch`                  | `Semimonad`         | `join`           |
| `AltDispatch`                   | `Alt`               | `alt`            |
| `ApplyFirstDispatch`            | `Semiapplicative`   | `apply_first`    |
| `ApplySecondDispatch`           | `Semiapplicative`   | `apply_second`   |
| `CompactDispatch`               | `Compactable`       | `compact`        |
| `SeparateDispatch`              | `Compactable`       | `separate`       |
| `FilterDispatch`                | `Filterable`        | `filter`         |
| `FilterMapDispatch`             | `Filterable`        | `filter_map`     |
| `PartitionDispatch`             | `Filterable`        | `partition`      |
| `PartitionMapDispatch`          | `Filterable`        | `partition_map`  |
| `FoldRightDispatch`             | `Foldable`          | `fold_right`     |
| `FoldLeftDispatch`              | `Foldable`          | `fold_left`      |
| `FoldMapDispatch`               | `Foldable`          | `fold_map`       |
| `Lift2Dispatch`-`Lift5Dispatch` | `Semiapplicative`   | `liftN`          |
| `TraverseDispatch`              | `Traversable`       | `traverse`       |
| `WiltDispatch`                  | `Witherable`        | `wilt`           |
| `WitherDispatch`                | `Witherable`        | `wither`         |
| `BimapDispatch`                 | `Bifunctor`         | `bimap`          |
| `BiFoldLeftDispatch`            | `Bifoldable`        | `bi_fold_left`   |
| `BiFoldRightDispatch`           | `Bifoldable`        | `bi_fold_right`  |
| `BiFoldMapDispatch`             | `Bifoldable`        | `bi_fold_map`    |
| `BiTraverseDispatch`            | `Bitraversable`     | `bi_traverse`    |
| `MapWithIndexDispatch`          | `FunctorWithIndex`  | `map_with_index` |
| etc.                            |                     |                  |

The dispatch trait name alone is sufficient to derive the semantic type class.
However, deriving the **arrow type** (the closure signature) varies per dispatch
trait and cannot be derived from the name alone. It can be derived from the
dispatch trait's `impl` block (see Approach G).

### Apply! patterns are highly consistent

All `Apply!` invocations containing `InferableBrand!` follow one of four
structural patterns:

| Pattern        | Example                                                      | Count |
| -------------- | ------------------------------------------------------------ | ----- |
| Simple         | `Apply!(<<FA as IB!(...)>::Brand as Kind!(...)>::Of<B>)`     | 14    |
| Nested         | `Apply!(<F ...>::Of<Apply!(<<FA as IB!(...)> ...>::Of<B>)>)` | 3     |
| Tuple of Apply | `(Apply!(...E), Apply!(...O))`                               | 3+    |
| Both           | `Apply!(<F>::Of<(Apply!(...E), Apply!(...O))>)`              | 1     |

The inner structure is always
`<<FA as InferableBrand!(...)>::Brand as Kind!(...)>::Of<'a, B>`.

### Dispatch trait impl block patterns

All 18 dispatch files follow one of 6 consistent patterns. Extraction
reliability is 98%+. No higher-ranked trait bounds, no GATs, no ambiguous
syntax. Key findings:

- **Fn bounds** are always directly on closure type parameters in the where
  clause, in the form `F: Fn(A) -> B + 'a`.
- **Semantic type class** is always `Brand: TypeClass` in the where clause.
- **Val/Ref discrimination** is always via the last generic argument to the
  trait (`Val` or `Ref`).
- **Closureless dispatch** has no `Fn` bound; the container itself is `self`.
- **Multiple semantic constraints** occur in traverse/witherable (e.g.,
  `Brand: Traversable` + `F: Applicative`).
- **Tuple closures** (bimap, bi*fold*\*) have separate `Fn` bounds on each
  generic parameter (`F` and `G`).
- **With-index variants** include `Brand::Index` in the `Fn` signature.

## Issues

### Issue 1: `InferableBrand_*` not filtered from constraints

**Symptom:** The HM signature shows `InferableBrand_cdc7cd43dac7585f FA =>` as a
constraint.

**Root cause:** `classify_trait()` checks for the `Kind_` prefix to filter
generated Kind traits, but does not check for the `InferableBrand_` prefix. The
constant `INFERABLE_BRAND_PREFIX` exists in `constants.rs` but is not used in
`classify_trait()`.

**Scope:** All 37 functions.

**Automatic fix feasibility:** 100%. Add one line to `classify_trait()`.

### Issue 2: Dispatch traits rendered as type constructors

**Symptom:** `FunctorDispatch FA A B FA Marker` appears where `A -> B` should.

**Root cause:** `impl FunctorDispatch<...>` is handled by `visit_impl_trait()`
which treats it as a regular type constructor. The dispatch trait's semantic
role (representing a callable) is not recognized.

**Scope:** 36 of 37 functions (all except `contramap`).

**Two sub-problems:**

1. **Constraint emission** (which type class to show): Fully automatic via
   suffix stripping. `FunctorDispatch` -> `Functor`.
2. **Arrow type construction** (the closure signature): Varies per dispatch
   trait. Requires either heuristic analysis of the function signature, or
   access to the dispatch trait's `impl` block (see Approach G).

**Sub-patterns:**

| Pattern             | Dispatch trait                                                  | Ideal HM                     | Functions                                                           |
| ------------------- | --------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------- |
| Simple map          | `FunctorDispatch<Brand, A, B, FA, Marker>`                      | `A -> B`                     | `map`                                                               |
| Bind                | `BindDispatch<Brand, A, B, FA, Marker>`                         | `A -> F B`                   | `bind`, `bind_flipped`                                              |
| Fold                | `FoldRightDispatch<FnBrand, Brand, A, B, FA, Marker>`           | `(A, B) -> B`                | `fold_right`, `fold_left`                                           |
| Fold map            | `FoldMapDispatch<FnBrand, Brand, A, M, FA, Marker>`             | `A -> M`                     | `fold_map`                                                          |
| Traverse            | `TraverseDispatch<FnBrand, Brand, A, B, F, FA, Marker>`         | `A -> F B`                   | `traverse`                                                          |
| Lift N              | `LiftNDispatch<Brand, A, B, ..., FA, FB, ..., Marker>`          | `(A, B, ...) -> Z`           | `lift2` through `lift5`                                             |
| Filter              | `FilterDispatch<Brand, A, FA, Marker>`                          | `A -> Bool`                  | `filter`                                                            |
| Filter map          | `FilterMapDispatch<Brand, A, B, FA, Marker>`                    | `A -> Option B`              | `filter_map`                                                        |
| Partition           | `PartitionDispatch<Brand, A, FA, Marker>`                       | `A -> Bool`                  | `partition`                                                         |
| Partition map       | `PartitionMapDispatch<Brand, A, E, O, FA, Marker>`              | `A -> Either E O`            | `partition_map`                                                     |
| Bimap               | `BimapDispatch<Brand, A, B, C, D, FA, Marker>`                  | `(A -> B, C -> D)`           | `bimap`                                                             |
| Bi-fold             | `BiFoldLeftDispatch<FnBrand, Brand, A, B, C, FA, Marker>`       | `((C, A) -> C, (C, B) -> C)` | `bi_fold_left`, etc.                                                |
| Bi-traverse         | `BiTraverseDispatch<FnBrand, Brand, A, B, C, D, F, FA, Marker>` | `(A -> F C, B -> F D)`       | `bi_traverse`                                                       |
| Closureless         | `AltDispatch<Brand, A, Marker>`                                 | (no closure)                 | `alt`, `compact`, `separate`, `join`, `apply_first`, `apply_second` |
| Wilt                | `WiltDispatch<FnBrand, Brand, M, A, E, O, FA, Marker>`          | `A -> M (F E, F O)`          | `wilt`                                                              |
| Wither              | `WitherDispatch<FnBrand, Brand, M, A, B, FA, Marker>`           | `A -> M (Option B)`          | `wither`                                                            |
| With-index variants | Same as above but with `WithIndex` bound                        | Same but with index param    | 10 functions                                                        |

### Issue 3: `Apply!` with nested `InferableBrand!` renders as `macro`

**Symptom:** Return types containing `Apply!` with nested `InferableBrand!`
render as `macro` instead of the actual type.

**Root cause:** `visit_macro()` calls `get_apply_macro_parameters()` which fails
to parse the token stream when it contains a nested `InferableBrand!` macro
inside the qualified path.

**Scope:** 20 of 37 functions.

**Automatic fix feasibility:** High. The `InferableBrand!(...)` and `Kind!(...)`
macros follow a fixed syntactic pattern that can be detected and simplified.

**Nesting patterns:**

| Pattern        | Example                                                      | Functions                                       |
| -------------- | ------------------------------------------------------------ | ----------------------------------------------- |
| Simple         | `Apply!(<<FA as IB!(...)>::Brand as Kind!(...)>::Of<B>)`     | `map`, `filter_map`, `lift2`-`lift5`, etc. (14) |
| Nested Apply   | `Apply!(<F ...>::Of<Apply!(<<FA as IB!(...)> ...>::Of<B>)>)` | `traverse`, `traverse_with_index`, `wither` (3) |
| Tuple of Apply | `(Apply!(...E), Apply!(...O))`                               | `partition`, `partition_map`, etc. (3+)         |
| Both           | `Apply!(<F>::Of<(Apply!(...E), Apply!(...O))>)`              | `wilt` (1)                                      |

### Issue 4: `Marker` type parameter in `forall`

**Symptom:** `Marker` appears as a user-visible type variable.

**Root cause:** `Marker` has no `Fn*` bound, so it is not filtered by
`format_generics()`.

**Scope:** 35 of 37 functions.

**Automatic fix feasibility:** 100%. Filter by exact name match.

### Issue 5: `FnBrand` type parameter in `forall`

**Symptom:** `FnBrand` appears as a user-visible type variable.

**Root cause:** Same as Issue 4.

**Scope:** 14 functions (foldable, traversable, witherable dispatch).

**Automatic fix feasibility:** 100%. Filter by exact name match.

### Issue 6: `FA`/`FB`/`FC` container variables not simplified

**Symptom:** The HM signature shows `forall FA FB A B C` instead of the more
natural `forall F A B C` with `F A`, `F B` notation.

**Root cause:** `FA` is a Rust type parameter representing the concrete container
(e.g., `Option<i32>`). The HM pipeline has no mechanism to recognize the
`FA = Brand::Of<A>` relationship and substitute `FA` with `F A`.

**Scope:** All 37 functions.

**Automatic fix feasibility:** Medium. A naming-convention approach (`FA` with
`InferableBrand` bound -> `F A`) gets 90% of the way there. Full accuracy
requires cross-referencing with the dispatch trait's type parameters.

## Approaches

### Approach A: Targeted filtering (Issues 1, 4, 5)

Add `InferableBrand_*` to the set of filtered traits in `classify_trait()`, and
filter `Marker` and `FnBrand` type parameters from the `forall` clause.

**Changes:**

1. In `classify_trait()`: add
   `n if n.starts_with(markers::INFERABLE_BRAND_PREFIX) => TraitCategory::Kind`.
2. In `format_generics()`: filter type parameters named `Marker` and `FnBrand`.

**Trade-offs:**

- Pro: Trivial changes. Each is independent and testable in isolation.
- Pro: No changes to the HM AST or type conversion logic.
- Con: Does not address issues 2, 3, or 6.

**Effort:** Small. A few lines of code.

**Manual configuration required:** None.

### Approach B: Automatic dispatch trait recognition (Issue 2, partial)

Detect dispatch traits by the `Dispatch` suffix in `classify_trait()` and emit
the semantic type class name as a constraint. This addresses the **constraint
emission** sub-problem of Issue 2 but not the **arrow type construction**.

**Changes:**

1. Add `TraitCategory::DispatchTrait(String)` variant, detected by
   `name.ends_with("Dispatch")`.
2. In `format_generics()`, when a dispatch trait is detected on a type variable,
   emit the semantic type class name (stripped suffix) as a constraint on the
   `InferableBrand`-bounded variable.
3. Suppress the dispatch-trait-bounded variable from `forall` (similar to how
   `Fn*`-bounded variables are suppressed).

**Arrow type challenge:** The arrow type (e.g., `A -> B` for map, `(A, B) -> B`
for fold_right) cannot be derived from the dispatch trait name alone. Two paths:

- **Heuristic elimination:** Identify semantic type parameters by excluding known
  infrastructure (`Brand`, `FA`/`FB`, `Marker`, `FnBrand`). The remaining
  parameters are inputs/output. The output is derivable from the function's
  return type. This works for simple cases but may produce incorrect results for
  complex dispatch layouts.
- **Cross-item analysis via Approach G:** Extract the arrow type from the
  dispatch trait's `impl` block, which contains the ground truth `Fn(A) -> B`
  bound.

**Trade-offs:**

- Pro: Fully automatic. No per-function configuration.
- Pro: Self-maintaining; new dispatch traits are recognized automatically.
- Con: Without Approach G, the arrow type relies on heuristics that may fail for
  complex patterns (wilt, bi_traverse, lift3+).

**Effort:** Medium. Suffix detection is trivial; heuristic arrow extraction is
moderate; integration with Approach G is the full solution.

**Manual configuration required:** None.

### Approach C: Apply!/InferableBrand! resolution (Issue 3)

Fix `visit_macro()` and `get_apply_macro_parameters()` to handle nested
`InferableBrand!` macros inside `Apply!` invocations.

**Two sub-approaches (both fully automatic):**

#### C1: Pre-expand InferableBrand! before Apply! parsing

Scan the `Apply!` token stream for `InferableBrand!` invocations and replace
them with a synthetic identifier before passing to `get_apply_macro_parameters()`.
Mirrors `resolve_inferable_brand()` in `apply.rs`.

#### C2: Pattern-based return type simplification

Recognize the double-qualified-self pattern
`<<FA as InferableBrand!(...)>::Brand as Kind!(...)>::Of<'a, B>` directly and
build `Constructor("FA", [Variable("B")])` without intermediate expansion.

**Trade-offs:**

- C1 reuses existing patterns and is closer to the architecture.
- C2 is more direct but pattern-specific.
- Both are fully automatic with no configuration.

**Effort:** Medium.

**Manual configuration required:** None.

### Approach D: FA -> F A naming convention (Issue 6)

If a type parameter matches `F[A-Z]` (single uppercase letter after `F`) and has
an `InferableBrand` bound, substitute it with `F <second-letter>` in the HM
output. For arity-2 brands, the variable name is used directly as the
constructor (e.g., `FA` with arity-2 brand -> `FA B D`).

**Trade-offs:**

- Pro: Simple pattern matching on the existing `InferableBrand` bound detection.
- Pro: Handles the common cases (`FA`, `FB`) well.
- Con: Does not handle borrowed containers (`&FA`).
- Con: Relies on naming conventions.

**Effort:** Small.

**Manual configuration required:** None.

### Approach E: Manual signature override (safety net)

Allow `#[document_signature]` to accept an optional string argument that
overrides the generated signature entirely:

```rust,ignore
#[document_signature("forall F A B. Functor F => (A -> B, F A) -> F B")]
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(...) -> ... { ... }
```

**Trade-offs:**

- Pro: Guarantees correct output for any edge case.
- Pro: Simple implementation.
- Con: **Requires manual input per function.** 37 functions need annotation.
- Con: Manual maintenance; annotation can go stale if the signature changes.

**Effort:** Small for implementation, medium for annotating 37 functions.

**Manual configuration required:** Yes, per function. This is a fallback for
cases that resist automatic handling, not a primary strategy.

### Approach G: Co-location via `#[document_module]` (Issues 2, 3, 6)

Place dispatch trait definitions and inference wrapper functions in the same
module processed by `#[document_module]`, so the macro can extract arrow type
and semantic constraint information from dispatch trait `impl` blocks.

**Core insight:** `#[document_module]` processes entire module token streams in
multiple passes. If the dispatch trait `impl` blocks (which contain the ground
truth: `Brand: Functor`, `F: Fn(A) -> B`) are in the same module as the wrapper
function, the macro can extract this during Pass 1 and use it during Pass 2.

**`#[document_module]` already supports cross-item analysis:** It uses a `Config`
struct to pass state between passes (self-type resolution, projection maps from
`impl_kind!`, scoped defaults). Adding dispatch trait analysis requires new
fields in `Config` and a new analysis pass, following the same pattern.

#### G1: Rename dispatch functions to `_explicit`, merge wrapper functions in

Rename dispatch free functions at their definition site (e.g., `map` becomes
`map_explicit` in `dispatch/functor.rs`), then move the inference wrapper `map`
from `functions/functor.rs` into `dispatch/functor.rs`. Both functions coexist
in the same `#[document_module]` module.

**Why the `_explicit` rename is needed:** Currently, both the dispatch module and
the function module export a function named `map`. The `_explicit` suffix is
applied during re-export in `functions.rs` via aliasing. Moving the suffix to
the definition site eliminates the naming collision, allowing both `map_explicit`
(dispatch) and `map` (inference) to coexist in one module.

**This resolves the original 5 blocking issues:**

1. **Name collision** -> Solved. `map_explicit` and `map` are different names.
2. **Nested `#[document_module]`** -> Solved. One module, one macro invocation.
3. **Signature incompatibility** -> No longer a problem; different names.
4. **Self-referential imports** -> Solved. Same scope, no import needed.
5. **Contravariant exception** -> Stays in `functions/contravariant.rs`.

**Remaining issues (none blocking):**

| Severity    | Issue                                        | Resolution                                                             |
| ----------- | -------------------------------------------- | ---------------------------------------------------------------------- |
| Medium-High | Module doc comments need merging             | Write unified narrative covering dispatch + inference                  |
| Medium      | `compose_kleisli` lacks inference wrapper    | Suffix it for consistency, or document why it has no inference variant |
| Medium      | `crate::dispatch::map` path changes (semver) | Pre-1.0; treat as intentional. Update 2 call sites in `vec.rs`         |
| Medium      | `cargo doc` sidebar changes                  | Verify after merge                                                     |
| Low         | Remove redundant self-imports after merge    | Mechanical cleanup                                                     |

**Blast radius of the rename:**

- 23 files to modify
- 37 function definitions to rename
- ~80 re-export lines to update
- ~52 test calls to update
- ~170-200 total line changes (mechanical find-and-replace)

**`#[document_module]` changes needed:**

1. New `dispatch_trait_info` field in `Config` to store extracted trait metadata.
2. New Pass 1 analysis: iterate module items, find dispatch traits and their
   `impl` blocks, extract `Fn` bounds and semantic type class bounds.
3. In Pass 2: when processing a function with `impl *Dispatch<...>` parameter,
   look up the pre-analyzed trait info to generate the correct arrow type and
   semantic constraint.

**Trade-offs:**

- Pro: Uses ground truth from `impl` blocks. No heuristics.
- Pro: Self-maintaining; new dispatch traits are automatically analyzed.
- Pro: Solves issues 2, 3, and 6 simultaneously.
- Pro: The `_explicit` rename aligns implementation with documented API (docs
  already use `_explicit` names everywhere).
- Con: Requires a one-time codebase reorganization (~200 line changes).
- Con: Merged modules are larger (dispatch/foldable.rs grows to ~900 lines).
- Con: `#[document_module]` needs new analysis logic (medium complexity).

**Effort:** Medium-large (rename + merge + new analysis pass).

**Manual configuration required:** None. The `_explicit` rename and file merge
are one-time mechanical changes.

#### G3: Generate wrapper functions from dispatch trait definitions

Apply a macro (e.g., `#[derive_inference_wrapper]`) to the dispatch trait that
generates the wrapper function in the same expansion context.

**Trade-offs:**

- Pro: Eliminates 37 hand-written wrapper functions entirely.
- Pro: Same ground-truth extraction as G1.
- Pro: No file reorganization needed.
- Con: Requires a new proc macro with sophisticated code generation.
- Con: Generated code is implicit; harder to read and debug.
- Con: Must handle all dispatch patterns (closure-based, closureless,
  multi-closure, with-index, bifunctor).

**Effort:** Large (new proc macro + integration).

**Manual configuration required:** None.

### Rejected approaches

The following approaches were investigated and rejected:

- **`include!` for cross-file token streams:** Proc macros expand before
  `include!` is resolved by the compiler. The included file remains invisible
  to the macro.
- **Build script metadata extraction:** Non-idiomatic. Synchronization between
  metadata and source code is fragile. Build scripts cannot reliably set env
  vars that proc macros read.
- **Shared metadata file:** Same synchronization problems as build script.
- **Move dispatch traits into functions/ (reverse merge):** Feasible but
  organizationally messy and strictly inferior to G1.
- **Configuration-driven dispatch mapping (TOML):** Requires manual mapping of
  ~25 dispatch traits to arrow types. Must stay in sync. Superseded by G1/G3
  which extract the same information automatically.
- **Annotation-driven dispatch (`#[document_dispatch(A -> B)]`):** Requires
  manual annotation on 36 functions. Must stay in sync. Superseded by G1/G3.
- **InferableBrand-driven FA substitution:** Too complex for the benefit.
  D1 (naming convention) gets 90% of the way there.
- **Rename Rust type parameters (FA -> F):** Changes the Rust API surface and
  error messages. Too wide a blast radius for a documentation improvement.

## Recommendation

### Tier 1: Quick wins (do first)

**Approach A:** Filter `InferableBrand_*` and hide `Marker`/`FnBrand`. A few
lines of code, immediately improves every function's HM signature. No risk.

### Tier 2: Core fix

Two paths are viable for the arrow type and return type resolution:

**Path 1: Incremental (A + B + C + D)**

Apply approaches independently:

1. A: Filtering (issues 1, 4, 5)
2. B: Suffix-based dispatch recognition with heuristic arrows (issue 2)
3. C1 or C2: Apply!/InferableBrand! resolution (issue 3)
4. D1: FA -> F A naming convention (issue 6)

Each step is independently shippable. Total effort: medium. Uses heuristics
for arrow types, which may produce incorrect results for complex patterns.

**Path 2: Architectural (A + G1)**

Apply Approach A first, then do the `_explicit` rename + merge (G1). The
`#[document_module]` cross-item analysis extracts ground-truth arrow types from
dispatch trait `impl` blocks, solving issues 2, 3, and 6 simultaneously.

Higher upfront effort (rename + merge + analysis pass), but produces correct
results for all patterns and is self-maintaining. D1 may still be useful for
the `contramap` edge case that stays in `functions/`.

**Path 3: Generative (A + G3)**

Apply Approach A first, then implement `#[derive_inference_wrapper]` (G3) to
generate wrapper functions from dispatch traits. Highest upfront effort, but
eliminates hand-written wrappers entirely and provides the same ground-truth
extraction as G1.

### Tier 3: Safety net

**Approach E:** Manual signature override for any remaining edge cases.

### Progression examples

```text
Current:       forall FA A B Marker. InferableBrand_cdc7cd43dac7585f FA => (FunctorDispatch FA A B FA Marker, FA) -> macro
After Tier 1:  forall FA A B. (FunctorDispatch FA A B FA, FA) -> macro
After Path 1:  forall F A B. Functor F => (A -> B, F A) -> F B
After Path 2:  forall F A B. Functor F => (A -> B, F A) -> F B
After Path 3:  forall F A B. Functor F => (A -> B, F A) -> F B
```

All three paths reach the same end state. The difference is in maintenance
burden, correctness guarantees, and upfront investment.

## Appendix: Current vs ideal HM signatures

### Simple map (functor.rs)

|              | Signature                                                                                                     |
| ------------ | ------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FA A B Marker. InferableBrand_cdc7cd43dac7585f FA => (FunctorDispatch FA A B FA Marker, FA) -> macro` |
| After Tier 1 | `forall FA A B. (FunctorDispatch FA A B FA, FA) -> macro`                                                     |
| After Tier 2 | `forall F A B. Functor F => (A -> B, F A) -> F B`                                                             |

### Traverse (traversable.rs)

|              | Signature                                                                                                                          |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FnBrand FA A B F Marker. InferableBrand_cdc7cd43dac7585f FA => (TraverseDispatch FnBrand FA A B F FA Marker, FA) -> macro` |
| After Tier 1 | `forall FA A B F. (TraverseDispatch FA A B F FA, FA) -> macro`                                                                     |
| After Tier 2 | `forall T A B F. (Applicative F, Traversable T) => (A -> F B, T A) -> F (T B)`                                                     |

### Closureless alt (alt.rs)

|              | Signature                                                                                                |
| ------------ | -------------------------------------------------------------------------------------------------------- |
| Current      | `forall FA A Marker. (InferableBrand_cdc7cd43dac7585f FA, AltDispatch FA A Marker) => (FA, FA) -> macro` |
| After Tier 1 | `forall FA A. AltDispatch FA A => (FA, FA) -> macro`                                                     |
| After Tier 2 | `forall F A. Alt F => (F A, F A) -> F A`                                                                 |

### Bifunctor bimap (bifunctor.rs)

|              | Signature                                                                                                           |
| ------------ | ------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FA A B C D Marker. InferableBrand_266801a817966495 FA => (BimapDispatch FA A B C D FA Marker, FA) -> macro` |
| After Tier 1 | `forall FA A B C D. (BimapDispatch FA A B C D FA, FA) -> macro`                                                     |
| After Tier 2 | `forall P A B C D. Bifunctor P => ((A -> B, C -> D), P A C) -> P B D`                                               |

### Fold right (foldable.rs)

|              | Signature                                                                                                                          |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FnBrand FA A B Marker. InferableBrand_cdc7cd43dac7585f FA => (FoldRightDispatch FnBrand FA A B FA Marker, B, FA) -> macro` |
| After Tier 1 | `forall FA A B. (FoldRightDispatch FA A B FA, B, FA) -> macro`                                                                     |
| After Tier 2 | `forall F A B. Foldable F => ((A, B) -> B, B, F A) -> B`                                                                           |

### Lift2 (lift.rs)

|              | Signature                                                                                                                 |
| ------------ | ------------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FA FB A B C Marker. InferableBrand_cdc7cd43dac7585f FA => (Lift2Dispatch FA A B C FA FB Marker, FA, FB) -> macro` |
| After Tier 1 | `forall FA FB A B C. (Lift2Dispatch FA A B C FA FB, FA, FB) -> macro`                                                     |
| After Tier 2 | `forall F A B C. Semiapplicative F => ((A, B) -> C, F A, F B) -> F C`                                                     |

### Wilt (witherable.rs)

|              | Signature                                                                                                                          |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FnBrand FA M A E O Marker. InferableBrand_cdc7cd43dac7585f FA => (WiltDispatch FnBrand FA M A E O FA Marker, FA) -> macro` |
| After Tier 1 | `forall FA M A E O. (WiltDispatch FA M A E O FA, FA) -> macro`                                                                     |
| After Tier 2 | `forall T M A E O. (Applicative M, Witherable T) => (A -> M (Either E O), T A) -> M (T E, T O)`                                    |
