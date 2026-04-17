# fp-macros crate refactoring audit

Audit of the current state of `fp-macros/src/` for refactoring
opportunities. No bugs were found; all items are structural improvements.

## 1. Where-clause scanning duplication

**Severity:** High
**Files:** `analysis/dispatch.rs`, `documentation/generation.rs`,
`analysis/generics.rs`

The pattern of iterating where-clause predicates, matching
`WherePredicate::Type`, then scanning trait bounds appears 8+ times in
`dispatch.rs` alone:

- `extract_kind_trait_name()` (lines ~328-349)
- `find_brand_param_from_trait_def()` (lines ~548-565)
- `find_brand_param()` (lines ~575-605)
- `extract_semantic_constraint()` (lines ~663-703)
- `extract_secondary_constraints()` (lines ~716-746)
- `extract_single_arrow()` (lines ~760-784)
- `extract_tuple_arrow()` (lines ~795-818)

Plus `generation.rs` has `has_where_bound_matching()`,
`extract_fn_brand_resolutions()`, and `is_tuple_order_reversed()` with
the same structure.

**Recommendation:** Extract a shared iterator helper:

```rust
fn for_each_type_predicate(
    generics: &Generics,
    f: impl FnMut(&PredicateType) -> ControlFlow<T>,
) -> Option<T>
```

Or a simpler filtering iterator:

```rust
fn type_predicates(generics: &Generics) -> impl Iterator<Item = &PredicateType>
```

## 2. Inline bounds + where clause duplication

**Severity:** High
**File:** `analysis/dispatch.rs`

Several functions scan both inline generic param bounds and where-clause
bounds with near-identical code in each branch:

- `extract_kind_trait_name()`: lines 308-325 (inline) vs 328-349 (where)
- `find_brand_param()`: lines 609-627 (inline) vs 577-605 (where)
- `extract_semantic_constraint()`: lines 687-705 (inline) vs 663-685 (where)
- `extract_single_arrow()`: lines 774-784 (inline) vs 760-773 (where)
- `extract_tuple_arrow()`: lines 809-818 (inline) vs 795-808 (where)

**Recommendation:** Create a helper that collects all trait bounds for a
named type parameter from both sources:

```rust
fn collect_bounds_for_param<'a>(
    param_name: &str,
    generics: &'a Generics,
) -> Vec<&'a TypeParamBound>
```

## 3. Trait bound name extraction pattern

**Severity:** Medium
**File:** `analysis/dispatch.rs`

The expression `.path.segments.last().map(|s| s.ident.to_string())
.unwrap_or_default()` appears 8+ times for extracting a trait name from
a `TraitBound`.

**Recommendation:** Extract helper:

```rust
fn trait_bound_name(bound: &TraitBound) -> Option<&syn::Ident>
```

Note: `support/get_parameters.rs` already has `last_path_segment()` which
returns `Option<&PathSegment>`. A similar helper for TraitBound would
eliminate this pattern.

## 4. Trait classification inconsistency

**Severity:** Medium
**Files:** `analysis/traits.rs`, `analysis/dispatch.rs`,
`documentation/document_signature.rs`

`analysis/traits.rs` defines `classify_trait()` which categorizes traits
into `FnTrait`, `FnBrand`, `Kind`, `ApplyMacro`, `Other`. But
`dispatch.rs` does not use it, instead inlining similar logic:

- `is_semantic_type_class()` (lines 634-656) duplicates the exclusion
  logic from `classify_trait()`
- `is_fn_like()` (lines 1006-1009) is a subset of `classify_trait()`'s
  `FnTrait` category

`document_signature.rs` does use `classify_trait()` (line 269), showing
the intended pattern.

**Recommendation:** Use `classify_trait()` in `dispatch.rs` and remove
the inlined checks, or extend `classify_trait()` with a
`Dispatch`/`InferableBrand` category.

## 5. Long functions needing decomposition

**Severity:** Medium

| Function                       | File                          | Lines | Issue                                                                        |
| ------------------------------ | ----------------------------- | ----- | ---------------------------------------------------------------------------- |
| `HmAstBuilder::visit_path()`   | `hm/ast_builder.rs`           | ~190  | Handles qualified paths, multi-segment, single-segment, smart pointers, Self |
| `a_do_worker()`                | `a_do/codegen.rs`             | ~135  | Combines parsing, transformation, code generation                            |
| `Canonicalizer::visit_path()`  | `hkt/canonicalizer.rs`        | ~120  | Handles type params, path segments, angle/parenthesized args                 |
| `VisitMut for SelfSubstitutor` | `resolution/resolver.rs`      | ~130  | visit_type_path_mut + visit_type_macro_mut + visit_signature_mut             |
| `classify_return_type()`       | `analysis/dispatch.rs`        | ~80   | Nested if-let chains for tuple, macro, nested cases                          |
| `build_synthetic_signature()`  | `documentation/generation.rs` | ~260  | Builds generic params, function params, return type                          |

**Recommendation:** Decompose `build_synthetic_signature()` first (highest
impact), then `HmAstBuilder::visit_path()`.

## 6. Duplicated ref-mode handling in do-notation

**Severity:** Medium
**Files:** `a_do/codegen.rs`, `m_do/codegen.rs`

Both files have identical code for:

- Bind parameter formatting (match on `(ref_mode, ty)` tuple, 4 arms):
  `a_do/codegen.rs` lines 47-56, `m_do/codegen.rs` lines 35-44

- Container reference wrapping (`if ref_mode { &(expr) } else { expr }`):
  `a_do/codegen.rs` lines 59, 91; `m_do/codegen.rs` lines 46, 77

**Recommendation:** Extract shared helpers into a `do_notation` utility
module:

```rust
fn format_bind_param(pat: &Pat, ty: Option<&Type>, ref_mode: bool) -> TokenStream
fn wrap_container_ref(expr: TokenStream, ref_mode: bool) -> TokenStream
```

## 7. Generic type argument extraction duplication

**Severity:** Medium
**File:** `hm/ast_builder.rs`

The pattern of extracting type arguments from
`PathArguments::AngleBracketed` appears 4 times (lines ~94-100,
~163-172, ~204-215, ~236-250) with near-identical code:

```rust
let mut type_args = Vec::new();
for arg in &args.args {
    if let GenericArgument::Type(inner_ty) = arg {
        type_args.push(self.visit(inner_ty));
    }
}
```

**Recommendation:** Extract `fn extract_type_args(&mut self, args:
&AngleBracketedGenericArguments) -> Vec<HmAst>`.

## 8. Lifetime/type param extraction duplication

**Severity:** Medium
**Files:** `hkt/trait_kind.rs` (lines 53-66), `hkt/impl_kind.rs`
(lines 293-306)

Both files extract lifetimes and type parameters from Generics with
near-identical code.

**Recommendation:** Add to `analysis/generics.rs`:

```rust
pub fn extract_lifetime_names(generics: &Generics) -> Vec<&syn::Lifetime>
pub fn extract_type_idents(generics: &Generics) -> Vec<&syn::Ident>
```

## 9. Parameter sprawl in process_document_signature

**Severity:** Medium
**File:** `documentation/generation.rs` (lines 88-99)

`process_document_signature()` takes 10 parameters. Several are related
context that could be grouped:

```rust
method, attr_pos, self_ty, self_ty_path, _trait_name,
trait_path_str, document_use, item_impl_generics, config, errors
```

**Recommendation:** Group into a context struct:

```rust
struct ImplContext<'a> {
    self_ty: &'a syn::Type,
    self_ty_path: &'a str,
    trait_path_str: Option<&'a str>,
    document_use: Option<&'a str>,
    item_impl_generics: &'a syn::Generics,
}
```

## 10. Config struct has too many responsibilities

**Severity:** Low
**File:** `core/config.rs`

`Config` combines user configuration (brand mappings from Cargo.toml),
runtime projections, impl-level docs, signature hashes, and dispatch
trait info in a single struct.

**Recommendation:** Consider splitting into focused types:
`UserConfig` (already exists as a field), `ProjectionMap`,
`ResolutionContext`. Low priority since the current approach works.

## 11. Error handling inconsistency

**Severity:** Low
**Files:** Throughout crate

Three error approaches coexist:

- `ErrorCollector` with `.collect()` / `.collect_our_result()` for
  multi-error accumulation (documentation/generation.rs)
- Direct `syn::Error` returns via `?` (m_do/input.rs, hkt/ files)
- `crate::core::Error` with validation/parse variants
  (documentation/document_signature.rs)

**Recommendation:** Document when each approach should be used:

- `ErrorCollector` for processing lists of items (e.g., methods in impl)
- `?` with `syn::Result` for single-item processing
- `crate::core::Error` for validation with rich context

## 12. Dead code / overly broad API

**Severity:** Low
**Files:** `support/attributes.rs`, `support/get_parameters.rs`

- `AttributeExt` trait has methods marked `#[allow(dead_code)]`:
  `find_and_remove()`, `find_and_remove_value()` (lines ~227-331)
- `Parameter::Implicit` variant has `#[expect(dead_code)]` on its
  `syn::Type` field (line 39)

These are acknowledged with explicit annotations. Consider removing if
they remain unused after the current development phase.

## 13. Stringly-typed processing (remaining)

**Severity:** Low
**Files:** `hkt/canonicalizer.rs`, `hm/ast_builder.rs`,
`resolution/resolver.rs`

Covered in detail in the separate tracking document at
`~/Documents/projects/notes/stringly-typed/fp-macros-instances.md`.

Key remaining items:

- `canonicalizer.rs` lines 188, 294: `quote!().to_string()` for const
  expression canonicalization
- `ast_builder.rs`: `generic_names: HashSet<String>` could be
  `HashSet<syn::Ident>`
- `resolver.rs`: String-keyed HashMaps for substitution mappings

## Priority summary

| Priority | Items                                    | Est. lines saved |
| -------- | ---------------------------------------- | ---------------- |
| High     | 1, 2 (where-clause + bounds duplication) | ~120             |
| Medium   | 3, 4, 5, 6, 7, 8, 9                      | ~150             |
| Low      | 10, 11, 12, 13                           | ~30              |

Items 1 and 2 would have the highest impact: extracting shared
where-clause iteration and bounds collection helpers would eliminate
the most duplicated code and make `dispatch.rs` significantly shorter.
