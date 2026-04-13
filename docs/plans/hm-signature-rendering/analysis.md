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

### Not affected

- `_explicit` variants in `dispatch/` (re-exports, same underlying functions).
- Trait methods in `classes/` (these have clean signatures without dispatch or
  brand inference machinery).
- Proc macro docs in `fp-macros/` (already `ignore`, different issue).

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

**Note:** This is arguably the most impactful issue for readability, but also the
hardest to fix correctly. The `FA` -> `F A` substitution requires understanding
the semantic relationship between `FA`, the inferred `Brand`, and `A`. Simply
renaming `FA` to `F A` in the output would be incorrect for borrowed containers
(`&Option<i32>` is `&F A`, not `F A`).

### Issue 7: Rustdoc raw signature (uncontrollable)

**Symptom:** The `pre.rust.item-decl` block generated by rustdoc shows the full
Rust signature with all the implementation details, hashes, and macro
invocations.

**Root cause:** This is generated by rustdoc, not by `#[document_signature]`.
The proc macro can only add doc comments; it cannot modify the signature that
rustdoc renders.

**Scope:** All 37 functions.

**Mitigation:** This cannot be fixed directly. However, if the HM "Type
Signature" section is clear and correct, it compensates for the raw signature.
Users familiar with FP will look at the HM section first.

## Approaches

### Approach A: Targeted filtering (Issues 1, 4, 5)

Add `InferableBrand_*` to the set of filtered traits in `classify_trait()`, and
add `Marker` and `FnBrand` to the set of hidden type parameters.

**Changes:**

1. In `classify_trait()`: add a check for `INFERABLE_BRAND_PREFIX` that returns
   `TraitCategory::Kind` (or a new `TraitCategory::InferableBrand` variant).

2. In `format_generics()`: filter out type parameters named `Marker` and
   `FnBrand` from the `forall` list. This could be done via:
   - A hardcoded list of hidden parameter names.
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
`impl DispatchTrait<Brand, A, B, FA, Marker>` to an arrow type.

**Sub-approaches:**

#### B1: Dispatch traits as a new `TraitCategory`

Add `TraitCategory::DispatchTrait` to `classify_trait()`. When a dispatch trait
is detected (by suffix `Dispatch` or by a configured list), extract the
"meaningful" type parameters (the ones representing input/output types) and
construct an arrow type.

**Challenge:** Each dispatch trait has a different parameter layout. The
"meaningful" parameters are at different positions depending on the trait. The
macro would need to know the semantic meaning of each parameter, which is not
derivable from the trait name alone.

**Sub-challenge:** Bifunctor dispatch traits like `BimapDispatch` take a
tuple of closures `(Fn(A) -> B, Fn(C) -> D)`, not a single closure. The arrow
representation would need to be a tuple of arrows.

#### B2: Annotation-driven dispatch recognition

Add an attribute (e.g., `#[document_dispatch(A -> B)]`) to the `impl Trait`
parameter that tells the HM pipeline how to render the dispatch trait. The
attribute provides the arrow type directly.

```rust
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
    #[document_dispatch(A -> B)]
    f: impl FunctorDispatch<...>,
    fa: FA,
) -> ...
```

**Challenge:** Attributes on function parameters are unstable in Rust
(`#![feature(param_attrs)]`). However, proc macros can parse and strip them
before the compiler sees them.

#### B3: Configuration-driven mapping

Add a mapping in `Cargo.toml` (or a separate config file) that maps dispatch
trait names to their arrow type templates:

```toml
[package.metadata.document_signature.dispatch_traits]
FunctorDispatch = { inputs = ["A"], output = "B" }
BindDispatch = { inputs = ["A"], output = "F B" }
FoldRightDispatch = { inputs = ["A", "B"], output = "B" }
```

**Trade-offs (all B sub-approaches):**

- Pro: Fixes the most user-visible issue (closure types).
- Con: Significant complexity. Each approach requires understanding the
  relationship between dispatch trait parameters and the user-visible types.
- Con: B1 is fragile (hardcodes parameter positions). B2 requires parameter
  attributes (proc-macro workaround needed). B3 is the most maintainable but
  requires the most upfront config work.
- Con: Getting this wrong produces misleading signatures, which is worse than
  showing the raw dispatch trait name.

**Effort estimate:** Medium to large.

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

#### C2: Simplify the return type pattern

Instead of parsing the full `Apply!` invocation, recognize the common pattern
`<<FA as InferableBrand!(...)>::Brand as Kind!(...)>::Of<'a, B>` and simplify
it to `F B` (where `F` is the inferred brand from `FA`).

This requires:

1. Detecting the `InferableBrand!(...)` macro in qualified-self position.
2. Resolving `FA` to a brand variable (e.g., `F`).
3. Extracting the type arguments from `::Of<'a, B>`.
4. Constructing `Constructor("F", [Variable("B")])`.

#### C3: Replace Apply! in return types with a doc alias

Replace the `Apply!` macro in function return types with a type alias
(e.g., `type Mapped<FA, B> = ...`) and use that in the signature. The HM
pipeline would then see a simple type alias instead of a macro.

**Trade-offs:**

- C1 is closest to the existing architecture and reuses the InferableBrand
  resolution logic. But it adds a pre-processing step to the HM pipeline.
- C2 produces the cleanest output but requires semantic understanding of the
  InferableBrand/Kind pattern. It is also the most brittle if the pattern
  changes.
- C3 avoids macro parsing entirely but changes the actual Rust signature,
  which may have downstream effects on type inference and error messages.

**Effort estimate:** Medium.

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

**Trade-offs:**

- D1 is simple but relies on naming conventions that may not hold.
- D2 produces the most accurate output but is very complex and requires
  cross-referencing multiple bounds.
- D3 is zero-effort in the macro but adds documentation burden and doesn't
  produce the clean Haskell-like signatures that FP users expect.

**Effort estimate:** D1 small, D2 large, D3 zero (doc-only).

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

1. **Approach A** (filtering): Fix issues 1, 4, 5. Quick wins.
2. **Approach C1 or C2** (Apply! resolution): Fix issue 3. Medium effort.
3. **Approach B3** (config-driven dispatch): Fix issue 2. Medium effort.
4. **Approach D3** (legend): Mitigate issue 6. Zero macro effort.
5. **Approach E** (manual override): Fallback for any remaining edge cases.

This produces:

```text
forall FA A B. Functor FA => (A -> B, FA) -> FA B
```

Which is close to ideal. The remaining difference from the fully Haskell-like
signature (`F A` instead of `FA`) is explained by the legend.

If D1 (naming-convention substitution) is also applied, the output becomes:

```text
forall F A B. Functor F => (A -> B, F A) -> F B
```

Which matches the Haskell ideal exactly for simple cases.

## Recommendation

**Tier 1 (do before release, low risk):**

- Approach A: Filter `InferableBrand_*` and hide `Marker`/`FnBrand` from
  `forall`. This is a few lines of code and immediately improves every
  function's HM signature.

**Tier 2 (do before release if time permits, medium risk):**

- Approach C (Apply! resolution): Fix the `-> macro` return type. This is the
  second most impactful issue after issue 2.
- Approach B3 (config-driven dispatch mapping): Fix the dispatch trait
  rendering. This requires the most design work but has the biggest payoff.

**Tier 3 (nice to have, can defer):**

- Approach D1 (FA -> F A naming convention): Polish.
- Approach E (manual override): Safety net for edge cases.

**Tier 4 (do not do):**

- Approach D2 (InferableBrand-driven substitution): Too complex for the
  benefit. The naming convention (D1) gets 90% of the way there.

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
