# Apply! Macro Unified Signature Syntax Plan

## 1. Executive Summary

This document outlines a plan to simplify the `Apply!` macro in `fp-macros` by modifying the `signature` parameter to contain both:

1. **Schema information**: Lifetime and type parameter counts, bounds (used to generate the Kind trait name)
2. **Concrete values**: Actual lifetime and type arguments (used for the `::Of<...>` projection)

This **replaces** the current separate `lifetimes` and `types` parameters when using `signature`, resulting in a more concise and intuitive API. This is a breaking change that simplifies the macro interface.

---

## 2. Current State Analysis

### 2.1 Current `Apply!` Syntax

The `Apply!` macro ([`fp-macros/src/lib.rs:167-172`](../../fp-macros/src/lib.rs)) currently supports two modes with named parameters:

**Using `signature`** (generates Kind trait name):

```rust
Apply!(
    brand: MyBrand,
    signature: ('a, T: Clone) -> Debug,
    lifetimes: ('static),
    types: (String)
)
// Expands to: <MyBrand as Kind_...>::Of<'static, String>
```

**Using explicit `kind`**:

```rust
Apply!(
    brand: MyBrand,
    kind: SomeKind,
    lifetimes: ('static),
    types: (String)
)
// Expands to: <MyBrand as SomeKind>::Of<'static, String>
```

### 2.2 Current Implementation

The [`ApplyInput`](../../fp-macros/src/apply.rs:36-46) struct separates concerns:

```rust
pub struct ApplyInput {
    pub brand: Type,
    pub kind_source: KindSource,      // signature OR explicit kind
    pub lifetimes: Punctuated<Lifetime, Token![,]>,  // concrete lifetime values
    pub types: Punctuated<Type, Token![,]>,          // concrete type values
}
```

The [`parse_signature()`](../../fp-macros/src/apply.rs:103-155) function parses the signature into a [`KindInput`](../../fp-macros/src/parse.rs:16-23) containing:

- `lifetimes`: Lifetime parameters (e.g., `'a`, `'b`)
- `types`: Type parameters with bounds (e.g., `T: Clone`)
- `output_bounds`: Bounds on the output type

### 2.3 Problem Statement

The current API has redundancy when the concrete arguments match the signature parameters:

```rust
// Common pattern: forward generic parameters from surrounding context
fn process<'a, T: Clone>(item: &'a T) -> Apply!(
    brand: MyBrand,
    signature: ('a, T: Clone),   // Parameters defined here...
    lifetimes: ('a),             // ...repeated here
    types: (T)                   // ...and here
)
```

Even when using different concrete values, the separation feels verbose:

```rust
Apply!(
    brand: MyBrand,
    signature: ('a, T: Clone),    // Schema defines the Kind
    lifetimes: ('static),         // Concrete values separate
    types: (String)
)
```

---

## 3. Proposed Solution

### 3.1 Unified Signature Syntax

**Replace** the current three-parameter syntax with a single unified parameter:

```rust
// NEW: Unified syntax (replaces old syntax)
Apply!(
    brand: MyBrand,
    signature: ('static, String: Clone) -> Debug
)
// Expands to: <MyBrand as Kind_...>::Of<'static, String>
```

The syntax encodes:
- **Concrete values**: The actual types/lifetimes to project (`'static`, `String`)
- **Bounds for Kind name**: The constraints after `:` (used to generate the trait name)

When using `signature`, the `lifetimes` and `types` parameters are **no longer accepted** - all information comes from the signature itself.

### 3.2 Syntax Specification

```
signature: (<params>) [-> <output_bounds>]

<params>  := <param> ["," <params>]
<param>   := <lifetime> | <type_param>

<lifetime>   := "'" IDENT                    // e.g., 'a, 'static
<type_param> := <type> [":" <bounds>]
<type>       := any valid Rust type          // e.g., T, String, Vec<u8>
<bounds>     := <bound> ["+" <bounds>]
<bound>      := <trait_bound> | <lifetime>

<output_bounds> := <bound> ["+" <output_bounds>]
```

### 3.3 Examples

| Input | Lifetimes | Types | Bounds | Kind Trait | Projection |
|-------|-----------|-------|--------|------------|------------|
| `('static, String)` | 1: `'static` | 1: `String` | none | `Kind_...` | `::Of<'static, String>` |
| `('a, T: Clone)` | 1: `'a` | 1: `T` | `Clone` | `Kind_...` | `::Of<'a, T>` |
| `(Vec<u8>: Send)` | 0 | 1: `Vec<u8>` | `Send` | `Kind_...` | `::Of<Vec<u8>>` |
| `('a, &'a str: Display) -> 'a` | 1: `'a` | 1: `&'a str` | `Display`, output `'a` | `Kind_...` | `::Of<'a, &'a str>` |

### 3.4 Two Modes of Operation

The `Apply!` macro will have exactly two modes:

**Mode 1: Using `signature`** (unified syntax)
```rust
Apply!(brand: MyBrand, signature: ('static, String: Clone) -> Debug)
```
- Schema and values combined in a single parameter
- `lifetimes` and `types` parameters are **not accepted**

**Mode 2: Using explicit `kind`**
```rust
Apply!(brand: MyBrand, kind: SomeKind, lifetimes: ('a), types: (T))
```
- Explicit Kind trait name provided
- `lifetimes` and `types` parameters are **required** (there's no schema to infer from)

This creates a clear separation: use `signature` for the common case, use `kind` for advanced/explicit cases.

---

## 4. Implementation Details

### 4.1 Modified Data Structures

#### New Signature Input Structure

```rust
/// A parameter in the unified signature syntax.
pub enum SignatureParam {
    /// A lifetime value (e.g., 'static, 'a)
    Lifetime(Lifetime),
    /// A type value with optional bounds (e.g., String: Clone)
    Type {
        ty: Type,
        bounds: Punctuated<TypeParamBound, Token![+]>,
    },
}

/// Parsed unified signature containing both schema and values.
pub struct UnifiedSignature {
    /// Parameters (lifetimes and types with bounds)
    pub params: Vec<SignatureParam>,
    /// Output bounds (e.g., -> Debug)
    pub output_bounds: Punctuated<TypeParamBound, Token![+]>,
}
```

#### Modified ApplyInput and KindSource

```rust
pub struct ApplyInput {
    pub brand: Type,
    pub kind_source: KindSource,
}

pub enum KindSource {
    /// Generated from unified signature (no separate lifetimes/types needed)
    Generated(UnifiedSignature),
    /// Explicit kind with required lifetimes and types
    Explicit {
        kind: Type,
        lifetimes: Punctuated<Lifetime, Token![,]>,
        types: Punctuated<Type, Token![,]>,
    },
}
```

### 4.2 Modified Parsing Logic

The [`parse_signature()`](../../fp-macros/src/apply.rs:103-155) function needs to be replaced:

```rust
fn parse_signature(input: ParseStream) -> syn::Result<UnifiedSignature> {
    let content;
    parenthesized!(content in input);

    let mut params = Vec::new();

    while !content.is_empty() {
        if content.peek(Lifetime) {
            // Lifetime parameter: 'a, 'static, etc.
            params.push(SignatureParam::Lifetime(content.parse()?));
        } else {
            // Type parameter: T, String, Vec<u8>, etc.
            let ty: Type = content.parse()?;

            // Optional bounds after ':'
            let bounds = if content.peek(Token![:]) {
                content.parse::<Token![:]>()?;
                parse_bounds(&content)?
            } else {
                Punctuated::new()
            };

            params.push(SignatureParam::Type { ty, bounds });
        }

        // Handle comma separator
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }
    }

    // Parse optional output bounds: -> Bound1 + Bound2
    let output_bounds = if input.peek(Token![->]) {
        input.parse::<Token![->]>()?;
        parse_output_bounds(input)?
    } else {
        Punctuated::new()
    };

    Ok(UnifiedSignature { params, output_bounds })
}

fn parse_bounds(input: ParseStream) -> syn::Result<Punctuated<TypeParamBound, Token![+]>> {
    let mut bounds = Punctuated::new();
    loop {
        if input.peek(Token![,]) || input.is_empty() {
            break;
        }
        bounds.push_value(input.parse()?);
        if input.peek(Token![+]) {
            bounds.push_punct(input.parse()?);
        } else {
            break;
        }
    }
    Ok(bounds)
}
```

The [`ApplyInput::parse()`](../../fp-macros/src/apply.rs:48-101) function needs to be modified to:
1. Reject `lifetimes` and `types` parameters when `signature` is used
2. Require `lifetimes` and `types` parameters when `kind` is used

### 4.3 Kind Name Generation

Extract schema information from the unified signature:

```rust
impl UnifiedSignature {
    /// Convert to KindInput for name generation.
    pub fn to_kind_input(&self) -> KindInput {
        let mut lifetimes = Punctuated::new();
        let mut types = Punctuated::new();

        // Create a mapping for lifetime canonicalization
        let mut lifetime_counter = 0;

        for param in &self.params {
            match param {
                SignatureParam::Lifetime(lt) => {
                    // Use canonical lifetime names for Kind generation
                    let canonical_lt = Lifetime::new(
                        &format!("'_{}", lifetime_counter),
                        lt.span()
                    );
                    lifetimes.push(canonical_lt);
                    lifetime_counter += 1;
                }
                SignatureParam::Type { bounds, .. } => {
                    // Use canonical type names for Kind generation
                    let canonical_ident = Ident::new(
                        &format!("T{}", types.len()),
                        Span::call_site()
                    );
                    types.push(TypeInput {
                        ident: canonical_ident,
                        bounds: bounds.clone(),
                    });
                }
            }
        }

        KindInput {
            lifetimes,
            types,
            output_bounds: self.output_bounds.clone(),
        }
    }

    /// Extract concrete lifetime values for projection.
    pub fn concrete_lifetimes(&self) -> Vec<&Lifetime> {
        self.params.iter()
            .filter_map(|p| match p {
                SignatureParam::Lifetime(lt) => Some(lt),
                _ => None,
            })
            .collect()
    }

    /// Extract concrete type values for projection.
    pub fn concrete_types(&self) -> Vec<&Type> {
        self.params.iter()
            .filter_map(|p| match p {
                SignatureParam::Type { ty, .. } => Some(ty),
                _ => None,
            })
            .collect()
    }
}
```

### 4.4 Code Generation

Update [`apply_impl()`](../../fp-macros/src/apply.rs:167-191):

```rust
pub fn apply_impl(input: ApplyInput) -> TokenStream {
    let brand = &input.brand;

    match &input.kind_source {
        KindSource::Generated(sig) => {
            let kind_input = sig.to_kind_input();
            let kind_name = generate_name(&kind_input);

            let lifetimes = sig.concrete_lifetimes();
            let types = sig.concrete_types();

            // Combine lifetimes and types
            let params = if lifetimes.is_empty() {
                quote! { #(#types),* }
            } else if types.is_empty() {
                quote! { #(#lifetimes),* }
            } else {
                quote! { #(#lifetimes),*, #(#types),* }
            };

            quote! {
                <#brand as #kind_name>::Of<#params>
            }
        }
        KindSource::Explicit { kind, lifetimes, types } => {
            // Combine lifetimes and types
            let params = if lifetimes.is_empty() {
                quote! { #types }
            } else if types.is_empty() {
                quote! { #lifetimes }
            } else {
                quote! { #lifetimes, #types }
            };

            quote! {
                <#brand as #kind>::Of<#params>
            }
        }
    }
}
```

---

## 5. Migration Strategy

### 5.1 Direct Replacement

Since this is a pre-1.0 library, we will directly replace the old syntax with no deprecation period:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Phase 1: Implement unified syntax                                   │
│  - Replace parsing for signature mode                               │
│  - Remove lifetimes/types acceptance in signature mode              │
│  - Keep explicit kind mode unchanged                                │
├─────────────────────────────────────────────────────────────────────┤
│ Phase 2: Migrate library code                                       │
│  - Update all fp-library usages to new syntax                       │
│  - Verify all tests pass                                            │
│  - Update documentation                                             │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 Code Migration Examples

**Before:**

```rust
fn map<'a, A: 'a, B: 'a, F>(
    f: F,
    fa: Apply!(
        brand: Self,
        signature: ('a, A: 'a) -> 'a,
        lifetimes: ('a),
        types: (A)
    ),
) -> Apply!(
    brand: Self,
    signature: ('a, A: 'a) -> 'a,
    lifetimes: ('a),
    types: (B)
)
```

**After:**

```rust
fn map<'a, A: 'a, B: 'a, F>(
    f: F,
    fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
```

---

## 6. Edge Cases and Considerations

### 6.1 Complex Types

The new syntax must handle complex type expressions:

| Type | Parsing Requirement |
|------|---------------------|
| `Vec<T>` | Generic types with angle brackets |
| `&'a T` | Reference types with lifetimes |
| `Box<dyn Fn(A) -> B>` | Trait objects and Fn traits |
| `(A, B)` | Tuple types |
| `[T; N]` | Array types |

All of these are valid `syn::Type` expressions and should parse correctly.

### 6.2 Ambiguity Resolution

**Question:** In `T: Clone`, is `T` a type parameter or a concrete type?

**Answer:** It's always treated as a **concrete value** in this context. If `T` is in scope as a generic parameter, it resolves to that. If `T` is a concrete type (unlikely name), it resolves to that. The semantics are identical to regular Rust type expressions.

### 6.3 Bound Canonicalization

The bounds are only used for **Kind trait name generation**, not for validation. The Rust compiler handles all type checking after macro expansion.

```rust
// The bound 'Clone' affects the Kind name, not String's validity
Apply!(brand: MyBrand, signature: (String: Clone))
// Generates: <MyBrand as Kind_...>::Of<String>
// Compiler verifies String: Clone at usage site
```

---

## 7. Testing Strategy

### 7.1 Unit Tests

1. **Parsing tests**: Verify `parse_signature()` correctly parses all syntax variations
2. **Extraction tests**: Verify `to_kind_input()`, `concrete_lifetimes()`, `concrete_types()`
3. **Generation tests**: Verify correct `TokenStream` output

### 7.2 Integration Tests

1. **End-to-end**: Complete `Apply!` usage with unified syntax
2. **Explicit kind mode**: Verify `kind:` with `lifetimes:`/`types:` still works

### 7.3 Compile-Fail Tests

1. **Invalid type syntax**: Malformed type expressions
2. **Missing brand**: Required parameter missing
3. **Conflicting sources**: Both `signature` and `kind` provided
4. **Invalid combination**: `signature` with `lifetimes`/`types` parameters (now an error)
5. **Missing required**: `kind` without `lifetimes`/`types` parameters

---

## 8. Documentation Updates

### 8.1 Macro Documentation

Update [`fp-macros/src/lib.rs`](../../fp-macros/src/lib.rs) documentation for `Apply!`:

```rust
/// Applies a brand to type arguments.
///
/// # Using `signature` (Recommended)
///
/// The `signature` parameter contains both the schema (for Kind trait name
/// generation) and the concrete values (for projection):
///
/// ```ignore
/// Apply!(brand: MyBrand, signature: ('static, String: Clone) -> Debug)
/// // Expands to: <MyBrand as Kind_...>::Of<'static, String>
/// ```
///
/// # Using explicit `kind`
///
/// For advanced cases where you need to specify an explicit Kind trait:
///
/// ```ignore
/// Apply!(brand: MyBrand, kind: SomeKind, lifetimes: ('a), types: (T))
/// // Expands to: <MyBrand as SomeKind>::Of<'a, T>
/// ```
```

### 8.2 Usage Guide

Document the two modes clearly:
1. `signature` mode: All-in-one syntax for typical usage
2. `kind` mode: Explicit Kind trait for advanced/custom cases

---

## 9. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Parsing ambiguity | Low | Medium | Extensive test coverage; syn handles Rust type syntax |
| Breaking existing code | Certain | Medium | Intentional breaking change; update all library code simultaneously |
| Performance regression | Very Low | Low | Parsing is compile-time only |
| User confusion | Low | Low | Clear documentation; simpler API with fewer options |

---

## 10. Success Criteria

1. **Unified syntax works**: `Apply!(brand: B, signature: ('a, T: Clone))` compiles and expands correctly
2. **Explicit kind mode works**: `Apply!(brand: B, kind: K, lifetimes: (...), types: (...))` continues to work
3. **Clean error on invalid usage**: `signature` with `lifetimes`/`types` produces clear error
4. **Test coverage**: All code paths have unit and integration tests
5. **Documentation**: Updated macro docs with clear examples for both modes
6. **Library migrated**: All `fp-library` code uses new syntax

---

## 11. References

- [Current Apply! Implementation](../../fp-macros/src/apply.rs)
- [KindInput Structure](../../fp-macros/src/parse.rs)
- [Kind Naming Refactor Plan](../kind-naming-refactor/plan.md)
- [syn Type Parsing](https://docs.rs/syn/latest/syn/enum.Type.html)
