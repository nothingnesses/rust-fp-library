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

```rust
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

```rust
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
in the token stream, or explicit configuration.

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

| Pattern                                | What needs analyzing                       | Feasibility |
| -------------------------------------- | ------------------------------------------ | ----------- |
| Dispatch trait -> semantic type class  | Strip `Dispatch` suffix                    | High        |
| Dispatch trait -> arrow type           | Extract Fn bounds from the dispatch `impl` | Medium      |
| `Apply!` with nested `InferableBrand!` | Token stream pattern matching              | Medium      |
| `FA` -> `F A` substitution             | Cross-reference InferableBrand + dispatch  | Medium      |

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
However, deriving the **arrow type** (the closure signature) requires knowing
the parameter layout of each dispatch trait, which varies.

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
The `InferableBrand!` resolution always resolves to the concrete container's
brand; the `Kind!(...)` is always the matching arity. This consistency makes
pattern-based extraction feasible.

## Issues

### Issue 1: `InferableBrand_*` not filtered from constraints

**Symptom:** The HM signature shows `InferableBrand_cdc7cd43dac7585f FA =>` as a
constraint.

**Root cause:** `classify_trait()` in `fp-macros/src/analysis/traits.rs` checks
for the `Kind_` prefix to filter generated Kind traits, but does not check for
the `InferableBrand_` prefix. The constant `INFERABLE_BRAND_PREFIX` exists in
`constants.rs` but is not used in `classify_trait()`. The trait falls through to
`TraitCategory::Other(name)` and is rendered as a visible constraint.

**Scope:** All 37 functions.

**Automatic fix feasibility:** 100%. Add one line to `classify_trait()`:
`n if n.starts_with(markers::INFERABLE_BRAND_PREFIX) => TraitCategory::Kind`.

### Issue 2: Dispatch traits rendered as type constructors

**Symptom:** `FunctorDispatch FA A B FA Marker` appears where `A -> B` should.

**Root cause:** The `impl FunctorDispatch<...>` parameter is handled by
`visit_impl_trait()` in `ast_builder.rs`, which extracts the first trait bound
and converts it via `trait_bound_to_hm_type()`. Since `FunctorDispatch` is not
recognized as an `Fn`-like trait (it is classified as `Other`), it is rendered as
a constructor application with all its type arguments, rather than being
converted to an arrow type.

The dispatch traits serve the same semantic role as `Fn(A) -> B` (they represent
a callable that maps from input types to an output type), but their argument
structure is different: the type arguments include the brand, the container type
`FA`, and the dispatch marker, none of which are part of the user-visible
function signature.

**Scope:** 36 of 37 functions (all except `contramap`).

**Automatic fix feasibility:** The dispatch trait _name_ is automatically
detectable (suffix `Dispatch`). The semantic type class name is derivable by
stripping the suffix. However, the closure's arrow type (which parameters are
inputs, which is the output) varies per dispatch trait and cannot be derived
from the name alone. Two sub-issues:

1. **Constraint emission** (which type class to show): Fully automatic via
   suffix stripping plus a mapping from operation name to type class.
2. **Arrow type construction** (the closure signature): Requires knowing which
   of the dispatch trait's type parameters are the meaningful input/output types.
   This requires either (a) structural analysis of the dispatch trait's `impl`
   bounds, (b) configuration, or (c) annotation.

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

**Root cause:** `visit_macro()` in `ast_builder.rs` calls
`get_apply_macro_parameters()` from `patterns.rs` to extract the brand and type
arguments from an `Apply!` invocation. When the `Apply!` input contains a nested
`InferableBrand!` macro inside the qualified path (e.g.,
`<<FA as InferableBrand!(...)>::Brand as Kind!(...)>::Of<'a, B>`), the parser
fails to extract the brand because the token stream contains an unexpanded
macro invocation where it expects a type path. The fallback is to render the
entire thing as the literal string `macro`.

**Scope:** 20 of 37 functions.

**Automatic fix feasibility:** High. The `InferableBrand!(...)` and `Kind!(...)`
macros in the token stream follow a fixed syntactic pattern. The HM pipeline
can detect `InferableBrand!` by name, extract the `FA` type variable from the
`<<FA as InferableBrand!(...)>` position, and use it to construct the return
type without fully expanding the macro. This is similar to how
`resolve_inferable_brand()` in `apply.rs` works.

**Nesting patterns:**

| Pattern        | Example                                                      | Functions                                       |
| -------------- | ------------------------------------------------------------ | ----------------------------------------------- |
| Simple         | `Apply!(<<FA as IB!(...)>::Brand as Kind!(...)>::Of<B>)`     | `map`, `filter_map`, `lift2`-`lift5`, etc. (14) |
| Nested Apply   | `Apply!(<F ...>::Of<Apply!(<<FA as IB!(...)> ...>::Of<B>)>)` | `traverse`, `traverse_with_index`, `wither` (3) |
| Tuple of Apply | `(Apply!(...E), Apply!(...O))`                               | `partition`, `partition_map`, etc. (3+)         |
| Both           | `Apply!(<F>::Of<(Apply!(...E), Apply!(...O))>)`              | `wilt` (1)                                      |

### Issue 4: `Marker` type parameter in `forall`

**Symptom:** `Marker` appears as a user-visible type variable in `forall FA A B Marker`.

**Root cause:** `Marker` is a regular type parameter in the Rust signature with
no `Fn*` bound. The `format_generics()` function includes all type parameters
in `forall` unless they have an `Fn*` bound that was expanded into `fn_bounds`.
Since `Marker` has no such bound (it is constrained only implicitly through the
dispatch trait), it appears in `forall`.

**Scope:** 35 of 37 functions.

**Automatic fix feasibility:** 100%. The parameter is always named `Marker`.
It can be filtered by exact name match or by recognizing that it only appears
as a type argument of a dispatch trait (never in parameters or return type
independently).

### Issue 5: `FnBrand` type parameter in `forall`

**Symptom:** `FnBrand` appears as a user-visible type variable in `forall`.

**Root cause:** Same as Issue 4. `FnBrand` is a type parameter that controls
whether the dispatch uses `CloneFn`-wrapped closures or bare closures. It is
an implementation detail the user never specifies.

**Scope:** Functions with foldable, traversable, or witherable dispatch (those
that need closure-wrapping): `fold_right`, `fold_left`, `fold_map`,
`fold_map_with_index`, `fold_right_with_index`, `fold_left_with_index`,
`traverse`, `traverse_with_index`, `wilt`, `wither`, `bi_fold_left`,
`bi_fold_right`, `bi_fold_map`, `bi_traverse` (14 functions).

**Automatic fix feasibility:** 100%. Same approach as Issue 4. The parameter
is always named `FnBrand`.

### Issue 6: `FA`/`FB`/`FC` container variables not simplified

**Symptom:** The HM signature shows `forall FA FB A B C` and parameter types
like `(... FA, FB)` instead of the more natural `forall F A B C` with
`(... F A, F B)`.

**Root cause:** `FA`, `FB`, etc. are Rust type parameters that represent the
concrete container types (e.g., `Option<i32>`, `&Vec<String>`). They are not
the same as the higher-kinded `F A` in Haskell. In the Rust encoding, the
relationship `FA = Brand::Of<A>` is expressed through trait bounds, not through
the type parameter name. The HM pipeline has no mechanism to recognize this
relationship and substitute `FA` with `F A`.

**Scope:** All 37 functions.

**Automatic fix feasibility:** Medium. The substitution requires knowing:

1. Which type parameters have `InferableBrand` bounds (detectable).
2. What the element type is for each container (requires cross-referencing with
   the dispatch trait's type parameters).
3. Whether the container is borrowed (`&FA` -> `F A` or `&(F A)`).

A naming-convention approach (`FA` -> `F A` when `FA` has an `InferableBrand`
bound) gets 90% of the way there without cross-referencing.

**Note:** This is arguably the most impactful issue for readability, but also the
hardest to fix correctly.

## Approaches

### Approach A: Targeted filtering (Issues 1, 4, 5)

Add `InferableBrand_*` to the set of filtered traits in `classify_trait()`, and
add `Marker` and `FnBrand` to the set of hidden type parameters.

**Changes:**

1. In `classify_trait()`: add a check for `INFERABLE_BRAND_PREFIX` that returns
   `TraitCategory::Kind` (or a new `TraitCategory::InferableBrand` variant).

2. In `format_generics()`: filter out type parameters named `Marker` and
   `FnBrand` from the `forall` list. This could be done via:
   - A hardcoded list of hidden parameter names (simplest; only two names).
   - A configuration option (`hidden_type_parameters`).
   - A naming convention (parameters starting with a specific prefix).

**Trade-offs:**

- Pro: Simple, low-risk changes. Each is independent and can be tested in
  isolation.
- Pro: No changes to the HM AST or type conversion logic.
- Con: Does not address issues 2, 3, or 6. The HM signature would go from
  broken to partially cleaned up, but still not ideal.
- Con: Hardcoding parameter names is fragile; a naming convention or config
  option is more robust but requires more design work.

**Effort estimate:** Small. A few lines in `classify_trait()` and
`format_generics()`, plus config if desired.

### Approach B: Dispatch trait recognition (Issue 2)

Teach the HM pipeline to recognize dispatch traits as function-like, converting
`impl DispatchTrait<Brand, A, B, FA, Marker>` to an arrow type and emitting
the semantic type class as a constraint.

**Sub-approaches:**

#### B1: Automatic suffix-based detection with structural analysis

Add `TraitCategory::DispatchTrait` to `classify_trait()`, detected by the
`Dispatch` suffix. When a dispatch trait is encountered in `visit_impl_trait()`:

1. Extract the semantic type class name by stripping the `Dispatch` suffix.
2. Emit the type class name as a constraint on the inferred brand variable
   (e.g., `Functor FA`).
3. For the arrow type, analyze which of the dispatch trait's type arguments
   also appear as standalone function type parameters (these are the "semantic"
   parameters) vs those that only appear as dispatch infrastructure.

**Heuristic for arrow type extraction:** The dispatch trait's type parameters
fall into predictable categories:

- Parameters that appear as the function's own generic parameters and are NOT
  `Brand`, `FA`/`FB`/`FC`, `Marker`, or `FnBrand` -> these are semantic
  (input/output types like `A`, `B`).
- The output type is determinable from the function's return type.
- Parameters matching `InferableBrand`-bounded variables -> container/brand.
- `Marker` -> always last, always infrastructure.

**Challenge:** While the semantic parameters can be identified by elimination,
determining which are inputs and which is the output requires either positional
conventions or return type analysis. For closureless dispatch traits, there is
no arrow type at all.

**Trade-offs:**

- Pro: Fully automatic; no per-function configuration.
- Pro: Self-maintaining; new dispatch traits are recognized automatically.
- Con: Heuristic may produce incorrect arrows for unusual dispatch layouts.
- Con: Medium complexity; needs careful handling of all sub-patterns.

#### B2: Annotation-driven dispatch recognition

Add an attribute (e.g., `#[document_dispatch(A -> B)]`) that tells the HM
pipeline how to render the dispatch trait:

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
    #[document_dispatch(A -> B)]
    f: impl FunctorDispatch<...>,
    fa: FA,
) -> ...
```

The attribute provides the arrow type directly. The semantic type class name
can still be derived automatically from the trait suffix.

**Note:** Attributes on function parameters are unstable in Rust
(`#![feature(param_attrs)]`). However, proc macros can parse and strip them
before the compiler sees them, so this works in practice.

**Trade-offs:**

- Pro: Precise control; no heuristic errors.
- Pro: Self-documenting; the annotation shows the intended semantics.
- Con: 36 functions need manual annotation.
- Con: Maintenance burden; annotation must stay in sync with the dispatch trait.

#### B3: Configuration-driven mapping

Add a mapping in `Cargo.toml` metadata (where config already lives) that maps
dispatch trait names to their arrow type templates:

```toml
[package.metadata.document_signature.dispatch_traits]
FunctorDispatch = { arrow = "(A) -> B", constraint = "Functor" }
BindDispatch = { arrow = "(A) -> F B", constraint = "Semimonad" }
FoldRightDispatch = { arrow = "(A, B) -> B", constraint = "Foldable" }
```

**Trade-offs:**

- Pro: Central configuration; easy to audit and maintain.
- Pro: Precise control without touching function code.
- Con: Requires upfront work to write all 25+ mappings.
- Con: Must be kept in sync when dispatch traits are added/modified.

#### B4: Hybrid automatic + override

Use B1 (automatic suffix-based detection) as the default, with B2 or B3 as an
override mechanism for cases where the heuristic fails. This gives the best of
both worlds: most dispatch traits work automatically, and edge cases get manual
correction.

**Trade-offs:**

- Pro: Minimal manual work (only annotate the exceptions).
- Pro: Self-maintaining for the common cases.
- Con: Two systems to understand (automatic + override).
- Con: Users must know when to apply the override.

**Effort estimate:** B1 medium, B2 small-medium (plus annotation), B3 medium,
B4 medium.

### Approach C: Apply!/InferableBrand! resolution (Issue 3)

Fix the `visit_macro()` and `get_apply_macro_parameters()` functions to handle
nested `InferableBrand!` macros inside `Apply!` invocations.

**Sub-approaches:**

#### C1: Pre-expand InferableBrand! before Apply! parsing

Before passing the `Apply!` token stream to `get_apply_macro_parameters()`,
scan for `InferableBrand!` invocations and replace them with a placeholder
type (e.g., the brand type variable from the enclosing function's generics).

This mirrors what `resolve_inferable_brand()` in `apply.rs` does for the
`Apply!` macro itself, but at the documentation level.

**Implementation sketch:**

1. In `visit_macro()`, before calling `get_apply_macro_parameters()`, scan the
   token stream for `InferableBrand` followed by `!`.
2. Replace the entire `InferableBrand!(...)` invocation with a synthetic
   identifier (e.g., `__Brand`).
3. The resulting token stream is now parseable by the existing Apply! logic.
4. In the HM output, map `__Brand` back to the brand variable.

#### C2: Pattern-based return type simplification

Instead of parsing the full `Apply!` invocation, recognize the overall pattern
and simplify directly:

- Detect `<<FA as InferableBrand!(...)>::Brand as Kind!(...)>::Of<'a, B>` in the
  token stream.
- Extract `FA` as the container variable and `B` (or `B, D`, etc.) as the type
  arguments.
- Construct `Constructor("FA", [Variable("B")])` directly.

This is a pattern-matching approach that skips the intermediate macro expansion
step entirely.

**Implementation sketch:**

1. In `visit_macro()`, after detecting an `Apply!` invocation, scan for the
   double-qualified-self pattern: `<< {ident} as InferableBrand! ... >`.
2. Extract the `ident` (e.g., `FA`) and the final `::Of<...>` arguments.
3. Build the HM constructor directly.

#### C3: Replace Apply! in return types with a type alias

Replace the `Apply!` macro in function return types with a type alias
(e.g., `type Mapped<FA, B> = ...`) and use that in the signature. The HM
pipeline would then see a simple type alias instead of a macro.

**Trade-offs:**

- C1 is closest to the existing architecture and reuses patterns from
  `resolve_inferable_brand()`. Moderate complexity; adds a pre-processing step.
- C2 produces the cleanest output and is the most direct. It is pattern-specific
  and would need updating if the Apply!/InferableBrand! pattern changes, but
  the pattern has been stable.
- C3 avoids macro parsing entirely but changes the actual Rust signature,
  which may have downstream effects on type inference and error messages.
  It also doesn't solve the problem for new functions that use the Apply! pattern
  directly.

**Effort estimate:** C1 medium, C2 medium, C3 medium (different trade-offs).

### Approach D: FA -> F A substitution (Issue 6)

Replace concrete container variables (`FA`, `FB`, etc.) with higher-kinded
applications (`F A`, `F B`, etc.) in the HM output.

**Sub-approaches:**

#### D1: Naming convention

If a type parameter name matches the pattern `F[A-Z]` (a single uppercase
letter following `F`), and the function has an `InferableBrand` bound on that
parameter, substitute it with `F <second-letter>` in the HM output.

Example: `FA` with `InferableBrand` bound -> `F A`.

**Challenge:** This breaks for multi-letter names (`FB` -> `F B` is fine, but
what about `FC`, `FD` in lift functions?). It also doesn't handle the reference
case (`&FA` should be `&(F A)` or just `F A` with a note about borrowing).

#### D2: InferableBrand-driven substitution

When a type parameter has an `InferableBrand` bound, the pipeline knows that
`FA::Brand` is the brand and `FA` is `Brand::Of<A>`. Use this to construct
`F A` where `F` is the brand variable and `A` is the element type.

**Challenge:** The element type `A` is not directly derivable from the
`InferableBrand` bound alone. The pipeline would need to also inspect the
dispatch trait bound to find which type parameter is the element type.

#### D3: Leave FA as-is, add a legend

Instead of substituting, add a note to the documentation explaining that `FA`
represents `F A` (a container of type `F` holding elements of type `A`). This
could be a one-time explanation in the module docs or a per-function note.

#### D4: Rename Rust type parameters

Instead of post-hoc substitution in the HM pipeline, rename the actual Rust
type parameters from `FA` to `F` (or `T`, `M`, etc.) in the source code, and
use explicit where-clause bounds to express the container relationship. The HM
pipeline would then naturally produce `F` instead of `FA`.

**Challenge:** This changes the Rust API surface and error messages. It may also
conflict with how brand inference works (the `FA` name is conventional in the
dispatch trait definitions).

**Trade-offs:**

- D1 is simple but relies on naming conventions that may not hold for all cases.
  Handles the common case (`FA`, `FB`) well.
- D2 produces the most accurate output but is very complex and requires
  cross-referencing multiple bounds.
- D3 is zero-effort in the macro but adds documentation burden and doesn't
  produce the clean Haskell-like signatures that FP users expect.
- D4 is the most "correct" approach but has the widest blast radius.

**Effort estimate:** D1 small, D2 large, D3 zero (doc-only), D4 medium-large.

### Approach E: Attribute-driven signature override

Add an attribute that specifies the complete HM signature directly, bypassing
the generation pipeline entirely.

```rust
#[document_signature("forall F A B. Functor F => (A -> B, F A) -> F B")]
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(...) -> ... { ... }
```

**Trade-offs:**

- Pro: Guarantees correct output. No inference errors.
- Pro: Simple implementation (just emit the provided string).
- Pro: Each function author controls exactly what appears.
- Con: Manual maintenance. If the function signature changes, the doc string
  must be updated separately. This is a source of staleness.
- Con: 37 functions need manual annotation.
- Con: Doesn't compose with `#[document_type_parameters]` (the type parameter
  names in the override may not match the Rust names).

**Effort estimate:** Small for implementation, medium for annotating all 37
functions.

### Approach F: Hybrid (recommended)

Combine approaches to get the best coverage with manageable effort:

1. **Approach A** (filtering): Fix issues 1, 4, 5. Quick wins, fully automatic.
2. **Approach C1 or C2** (Apply! resolution): Fix issue 3. Medium effort,
   fully automatic.
3. **Approach B1 or B4** (dispatch recognition): Fix issue 2. Medium effort,
   mostly automatic with optional overrides.
4. **Approach D1** (naming convention): Fix issue 6. Small effort, mostly
   automatic.
5. **Approach E** (manual override): Fallback for any remaining edge cases.

This produces the progression:

```text
Current:    forall FA A B Marker. InferableBrand_cdc7cd43dac7585f FA => (FunctorDispatch FA A B FA Marker, FA) -> macro
After A:    forall FA A B. (FunctorDispatch FA A B FA, FA) -> macro
After A+C:  forall FA A B. (FunctorDispatch FA A B FA, FA) -> FA B
After A+B+C: forall FA A B. Functor FA => (A -> B, FA) -> FA B
After A+B+C+D: forall F A B. Functor F => (A -> B, F A) -> F B
```

Each step is independently valuable and can be shipped incrementally.

## Recommendation

**Tier 1 (do first, low risk):**

- Approach A: Filter `InferableBrand_*` and hide `Marker`/`FnBrand` from
  `forall`. This is a few lines of code and immediately improves every
  function's HM signature.

**Tier 2 (high impact, medium risk):**

- Approach C (Apply! resolution): Fix the `-> macro` return type. This is the
  second most impactful issue after issue 2.
- Approach B (dispatch recognition): Fix the dispatch trait rendering. B4
  (hybrid automatic + override) is recommended: automatic suffix detection
  handles common cases, with B2-style annotations for edge cases.

**Tier 3 (polish, low risk):**

- Approach D1 (FA -> F A naming convention): Small change, big readability win.
- Approach E (manual override): Safety net for edge cases that resist automatic
  handling.

**Not recommended:**

- Approach D2 (InferableBrand-driven substitution): Too complex for the
  benefit. D1 gets 90% of the way there.
- Approach D4 (rename Rust type parameters): Too wide a blast radius for a
  documentation improvement.

## Appendix: Current vs ideal HM signatures

### Simple map (functor.rs)

|              | Signature                                                                                                     |
| ------------ | ------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FA A B Marker. InferableBrand_cdc7cd43dac7585f FA => (FunctorDispatch FA A B FA Marker, FA) -> macro` |
| After Tier 1 | `forall FA A B. (FunctorDispatch FA A B FA, FA) -> macro`                                                     |
| After Tier 2 | `forall FA A B. Functor FA => (A -> B, FA) -> FA B`                                                           |
| After Tier 3 | `forall F A B. Functor F => (A -> B, F A) -> F B`                                                             |

### Traverse (traversable.rs)

|              | Signature                                                                                                                          |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FnBrand FA A B F Marker. InferableBrand_cdc7cd43dac7585f FA => (TraverseDispatch FnBrand FA A B F FA Marker, FA) -> macro` |
| After Tier 1 | `forall FA A B F. (TraverseDispatch FA A B F FA, FA) -> macro`                                                                     |
| After Tier 2 | `forall FA A B F. (Applicative F, Traversable FA) => (A -> F B, FA) -> F (FA B)`                                                   |
| After Tier 3 | `forall T A B F. (Applicative F, Traversable T) => (A -> F B, T A) -> F (T B)`                                                     |

### Closureless alt (alt.rs)

|              | Signature                                                                                                |
| ------------ | -------------------------------------------------------------------------------------------------------- |
| Current      | `forall FA A Marker. (InferableBrand_cdc7cd43dac7585f FA, AltDispatch FA A Marker) => (FA, FA) -> macro` |
| After Tier 1 | `forall FA A. AltDispatch FA A => (FA, FA) -> macro`                                                     |
| After Tier 2 | `forall FA A. Alt FA => (FA, FA) -> FA A`                                                                |
| After Tier 3 | `forall F A. Alt F => (F A, F A) -> F A`                                                                 |

### Bifunctor bimap (bifunctor.rs)

|              | Signature                                                                                                           |
| ------------ | ------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FA A B C D Marker. InferableBrand_266801a817966495 FA => (BimapDispatch FA A B C D FA Marker, FA) -> macro` |
| After Tier 1 | `forall FA A B C D. (BimapDispatch FA A B C D FA, FA) -> macro`                                                     |
| After Tier 2 | `forall FA A B C D. Bifunctor FA => ((A -> B, C -> D), FA) -> FA B D`                                               |
| After Tier 3 | `forall P A B C D. Bifunctor P => ((A -> B, C -> D), P A C) -> P B D`                                               |

### Fold right (foldable.rs)

|              | Signature                                                                                                                          |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FnBrand FA A B Marker. InferableBrand_cdc7cd43dac7585f FA => (FoldRightDispatch FnBrand FA A B FA Marker, B, FA) -> macro` |
| After Tier 1 | `forall FA A B. (FoldRightDispatch FA A B FA, B, FA) -> macro`                                                                     |
| After Tier 2 | `forall FA A B. Foldable FA => ((A, B) -> B, B, FA) -> B`                                                                          |
| After Tier 3 | `forall F A B. Foldable F => ((A, B) -> B, B, F A) -> B`                                                                           |

### Lift2 (lift.rs)

|              | Signature                                                                                                                 |
| ------------ | ------------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FA FB A B C Marker. InferableBrand_cdc7cd43dac7585f FA => (Lift2Dispatch FA A B C FA FB Marker, FA, FB) -> macro` |
| After Tier 1 | `forall FA FB A B C. (Lift2Dispatch FA A B C FA FB, FA, FB) -> macro`                                                     |
| After Tier 2 | `forall FA FB A B C. Semiapplicative FA => ((A, B) -> C, FA, FB) -> FA C`                                                 |
| After Tier 3 | `forall F A B C. Semiapplicative F => ((A, B) -> C, F A, F B) -> F C`                                                     |

### Wilt (witherable.rs)

|              | Signature                                                                                                                          |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------- |
| Current      | `forall FnBrand FA M A E O Marker. InferableBrand_cdc7cd43dac7585f FA => (WiltDispatch FnBrand FA M A E O FA Marker, FA) -> macro` |
| After Tier 1 | `forall FA M A E O. (WiltDispatch FA M A E O FA, FA) -> macro`                                                                     |
| After Tier 2 | `forall FA M A E O. (Applicative M, Witherable FA) => (A -> M (Either E O), FA) -> M (FA E, FA O)`                                 |
| After Tier 3 | `forall T M A E O. (Applicative M, Witherable T) => (A -> M (Either E O), T A) -> M (T E, T O)`                                    |
