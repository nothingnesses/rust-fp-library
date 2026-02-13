# fp-macros Clean-Room Redesign Specification

**Version:** 2.0  
**Date:** 2026-02-09  
**Status:** Specification  

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Current State Analysis](#2-current-state-analysis)
3. [Identified Issues](#3-identified-issues)
4. [Core Principles](#4-core-principles)
5. [High-Level Architecture](#5-high-level-architecture)
6. [Key Design Decisions](#6-key-design-decisions)
7. [Module-by-Module Specification](#7-module-by-module-specification)
8. [Migration Strategy](#8-migration-strategy)
9. [Quality Assurance](#9-quality-assurance)
10. [Appendices](#10-appendices)

---

## 1. Executive Summary

### 1.1 Purpose

This specification defines a clean-room redesign of the `fp-macros` crate to address architectural inconsistencies, reduce code duplication, and establish clear patterns for maintainability while preserving all existing functionality and documentation quality.

### 1.2 Scope

**In Scope:**
- Complete architectural restructuring
- Unified error handling system
- Consolidated configuration management
- Standardized patterns across all macros
- Enhanced modularity and testability
- API breakages for improved clarity

**Out of Scope:**
- Changes to macro semantics (behavior must match exactly)
- Removal of existing features
- Changes to generated documentation format
- Performance optimizations (maintain current performance baseline)

### 1.3 Goals

1. **Consistency:** Establish uniform patterns for error handling, configuration, and code organization
2. **Maintainability:** Reduce duplication and improve code clarity
3. **Robustness:** Strengthen error handling and input validation
4. **Extensibility:** Design for easy addition of new macros and features
5. **Documentation:** Maintain excellent documentation quality while improving internal documentation

---

## 2. Current State Analysis

### 2.1 Overall Assessment

**Grade:** B+ / Very Good

The current implementation demonstrates strong engineering practices with excellent documentation, modular architecture, comprehensive error handling, and good test coverage. However, several architectural inconsistencies and code duplication patterns warrant a redesign.

### 2.2 Strengths

#### 2.2.1 Documentation Quality ✅

All public macros include:
- Clear syntax descriptions with code blocks
- Multiple usage examples
- Documented limitations
- Generated code samples
- Cross-references between related macros

**Example:** [`Kind!` macro documentation](fp-macros/src/lib.rs:30-89) provides comprehensive examples and clearly documents syntax limitations.

#### 2.2.2 Error System Foundation ✅

[`error.rs`](fp-macros/src/error.rs:1-289) provides:
- Structured error types using `thiserror`
- Span tracking for precise error locations
- Contextual error messages with `.context()` method
- Conversion traits for `syn::Error` interop
- Multiple error categories (Parse, Validation, Resolution, Unsupported, Internal, Io)

**Example:**
```rust
pub enum Error {
    Parse(#[from] syn::Error),
    Validation { message: String, span: Span },
    Resolution { message: String, span: Span, available_types: Vec<String> },
    Unsupported(#[from] UnsupportedFeature),
    Internal(String),
    Io(#[from] std::io::Error),
}
```

#### 2.2.3 Modular Architecture ✅

Clear separation of concerns:
- `hkt/` - Higher-Kinded Type macros
- `documentation/` - Documentation generation
- `resolution/` - Type resolution
- `analysis/` - Generic and trait analysis
- `hm_conversion/` - Hindley-Milner type conversion

#### 2.2.4 Comprehensive Testing ✅

- Unit tests in each module
- Property tests in [`property_tests.rs`](fp-macros/src/property_tests.rs:1)
- Compile-fail tests
- Documentation examples as tests

#### 2.2.5 Input Validation ✅

Strong validation patterns:
```rust
// patterns.rs:54-83
impl Parse for KindInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // ... parsing logic ...
        
        // Validation: non-empty
        if assoc_types.is_empty() {
            return Err(Error::validation(
                Span::call_site(),
                "Kind definition must have at least one associated type"
            ).into());
        }
        
        // Validation: no const generics
        for assoc in &assoc_types {
            for param in &assoc.generics.params {
                if let GenericParam::Const(const_param) = param {
                    return Err(Error::Unsupported(
                        UnsupportedFeature::ConstGenerics { 
                            span: const_param.ident.span() 
                        }
                    ).into());
                }
            }
        }
        
        Ok(KindInput { assoc_types })
    }
}
```

### 2.3 Weaknesses Summary

| Category | Issue | Severity | Impact |
|----------|-------|----------|--------|
| Architecture | Dual configuration systems | 🔴 High | Confusion, potential duplication |
| Architecture | Module import confusion | ⚠️ Medium | Reduced clarity |
| Consistency | Mixed error handling patterns | 🔴 High | Harder to reason about |
| Duplication | Error conversion boilerplate | 🔴 High | Maintenance burden |
| Duplication | Re-export implementation | ⚠️ Medium | Cognitive overhead |
| Standards | File I/O in proc macros | ⚠️ Medium | Build reproducibility concerns |
| Consistency | Error message formatting | ⚠️ Medium | User experience |
| Reusability | Hardcoded paths in re-exports | ⚠️ Medium | Limited reusability |

---

## 3. Identified Issues

### 3.1 CRITICAL ISSUES 🔴

#### 3.1.1 Dual Configuration Systems

**Location:** 
- [`fp-macros/src/config/mod.rs`](fp-macros/src/config/mod.rs:1-13)
- [`fp-macros/src/core/config.rs`](fp-macros/src/core/mod.rs:8-9)

**Problem:**
Two separate configuration modules exist with overlapping responsibilities:

1. `crate::config` - Contains `load_config()` and `Config` type
2. `crate::core::config` - Contains `get_config()`

**Impact:**
- Developers unsure which to use
- Potential for configuration state inconsistencies
- Duplicate logic for configuration loading
- Harder to maintain configuration schema

**Evidence:**
```rust
// In hm_signature.rs
use crate::config::{Config, load_config};
let config = load_config();

// In core/config.rs (presumably)
pub use config::get_config;
```

**Root Cause:**
Incremental refactoring without consolidation of configuration concerns.

#### 3.1.2 Inconsistent Error Handling Patterns

**Locations:** Throughout codebase

**Problem:**
Three different error handling patterns coexist:

**Pattern A - Custom Error Type (Preferred):**
```rust
// hkt/apply.rs:73-76
let kind_name = match generate_name(&input.kind_input) {
    Ok(name) => name,
    Err(e) => return syn::Error::from(e).to_compile_error(),
};
```

**Pattern B - Direct syn::Error:**
```rust
// Various locations
return syn::Error::new(span, "message").to_compile_error();
```

**Pattern C - compile_error! macro:**
```rust
// re_export.rs:186-194
return vec![quote! {
    compile_error!(concat!("Failed to read directory: '", #path_str, "'"));
}];
```

**Impact:**
- Inconsistent error messages
- Harder to test error conditions
- Difficult to add contextual information
- Mixed error presentation to users

**Statistics:**
- Pattern A: ~40% of error sites
- Pattern B: ~50% of error sites
- Pattern C: ~10% of error sites

#### 3.1.3 Error Conversion Boilerplate

**Locations:**
- [`hkt/apply.rs:75`](fp-macros/src/hkt/apply.rs:75)
- [`hkt/kind.rs:17`](fp-macros/src/hkt/kind.rs:17)
- [`hkt/impl_kind.rs:175`](fp-macros/src/hkt/impl_kind.rs:175)

**Problem:**
The pattern `syn::Error::from(e).to_compile_error()` is repeated verbatim across multiple files.

**Code Repetition:**
```rust
// In apply.rs
let kind_name = match generate_name(&input.kind_input) {
    Ok(name) => name,
    Err(e) => return syn::Error::from(e).to_compile_error(),
};

// In kind.rs
let name = match generate_name(&input) {
    Ok(name) => name,
    Err(e) => return syn::Error::from(e).to_compile_error(),
};

// In impl_kind.rs
let kind_trait_name = match generate_name(&kind_input) {
    Ok(name) => name,
    Err(e) => return syn::Error::from(e).to_compile_error(),
};
```

**Impact:**
- Violates DRY principle
- Harder to change error reporting strategy
- Increases maintenance burden
- Copy-paste errors possible

### 3.2 MODERATE ISSUES ⚠️

#### 3.2.1 Module Import Confusion

**Location:** [`fp-macros/src/lib.rs:11`](fp-macros/src/lib.rs:11)

**Problem:**
The `common` module is declared in `lib.rs` but never used via that path:

```rust
// Declared in lib.rs
pub(crate) mod common;

// But all usage is:
use crate::common::syntax::{GenericItem, insert_doc_comment};
use crate::common::attributes::has_attr;
use crate::common::errors::known_attrs;
// etc.
```

**Impact:**
- Misleading code structure
- Unclear module organization
- New contributors may be confused

#### 3.2.2 Re-export Implementation Duplication

**Location:** [`fp-macros/src/re_export.rs:203-273`](fp-macros/src/re_export.rs:203-273)

**Problem:**
`generate_function_re_exports_impl()` and `generate_trait_re_exports_impl()` share 95% of their logic through `generate_re_exports_impl()` using an enum discriminator.

**Current Design:**
```rust
enum ItemKind {
    Function,
    Trait,
}

fn generate_re_exports_impl(input: &ReexportInput, kind: ItemKind) -> TokenStream {
    // ... 70 lines of shared logic ...
    
    match kind {
        ItemKind::Function => { /* 5 lines */ },
        ItemKind::Trait => { /* 5 lines */ },
    }
}
```

**Impact:**
- Adds cognitive overhead with enum discriminator
- Not easily extensible for new item kinds
- The abstraction doesn't eliminate duplication, just hides it

#### 3.2.3 File I/O in Procedural Macros

**Location:** [`re_export.rs:156-199`](fp-macros/src/re_export.rs:156-199)

**Problem:**
Procedural macros perform filesystem operations at compile time:

```rust
let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
let base_path = Path::new(&manifest_dir).join(input.path.value());

if let Ok(entries) = fs::read_dir(&base_path) {
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = fs::read_to_string(&path)?;
            // ...
        }
    }
}
```

**Impact:**
- Build reproducibility issues (file system state affects compilation)
- IDE integration problems (macros may not expand correctly in IDEs)
- Unclear dependency tracking (Cargo doesn't know about these file dependencies)
- Cross-compilation complications

**Mitigation Present:**
Error handling converts I/O failures to compile errors rather than panics, but doesn't address the fundamental issue.

**Industry Best Practice:**
File scanning should be done in build scripts (`build.rs`) that generate code, not in procedural macros.

#### 3.2.4 Hardcoded Paths in Re-exports

**Location:** [`re_export.rs:262`](fp-macros/src/re_export.rs:262)

**Problem:**
Base module path is hardcoded:

```rust
match kind {
    ItemKind::Function => {
        quote! {
            pub use crate::classes::{  // <- Hardcoded
                #(#re_exports),*
            };
        }
    }
    // ...
}
```

**Impact:**
- Not reusable for other module structures
- Limits macro applicability to specific project layout
- Requires code changes for different module hierarchies

#### 3.2.5 Nested Module Pattern Detection

**Location:** [`re_export.rs:52-68`](fp-macros/src/re_export.rs:52-68)

**Problem:**
Special handling for project-specific `pub use inner::*;` pattern:

```rust
fn detect_reexport_pattern(file: &syn::File) -> Option<String> {
    for item in &file.items {
        if let Item::Use(use_item) = item
            && matches!(use_item.vis, Visibility::Public(_))
        {
            if let syn::UseTree::Path(path) = &use_item.tree
                && let syn::UseTree::Glob(_) = &*path.tree
            {
                return Some(path.ident.to_string());  // Returns "inner"
            }
        }
    }
    None
}
```

**Impact:**
- Couples macro to specific code organization pattern
- Reduces generalizability
- May break if project structure changes

#### 3.2.6 Error Message Formatting Inconsistency

**Problem:**
Inconsistent error message styles across the codebase:

**Style A - Capitalized, full sentence:**
```rust
"Validation error: Kind definition must have at least one associated type"
```

**Style B - Lowercase, fragment:**
```rust
"expected `Kind`"
```

**Style C - Format with values:**
```rust
format!("cannot resolve: {}", type_name)
```

**Impact:**
- Inconsistent user experience
- Harder to write error message guidelines
- Difficult to maintain consistent tone

#### 3.2.7 Attribute Parsing Inconsistency

**Location:** [`hm_signature.rs:33-39`](fp-macros/src/documentation/hm_signature.rs:33-39)

**Problem:**
Different macros validate attributes differently:

```rust
// hm_signature.rs
if !attr.is_empty() {
    return syn::Error::new(..., "hm_signature does not accept arguments").to_compile_error();
}

// doc_params.rs - Different validation approach
// doc_type_params.rs - Yet another validation approach
```

**Impact:**
- Inconsistent user experience
- Duplicated validation logic
- Harder to ensure all macros handle attributes correctly

### 3.3 MINOR ISSUES ℹ️

#### 3.3.1 Dead Code Annotations

**Location:** [`hkt/impl_kind.rs:28-70`](fp-macros/src/hkt/impl_kind.rs:28-70)

**Issue:**
Multiple structs use `#[allow(dead_code)]`:

```rust
#[allow(dead_code)]
pub struct ImplKindInput {
    pub impl_generics: Generics,
    pub for_token: Token![for],
    pub brand: Type,
    // ...
}
```

**Reason:**
Fields are used in `Parse` implementation and token generation, but the compiler doesn't recognize this as "usage".

**Impact:**
- Minor - acceptable pattern for proc-macro input structures
- Could confuse new contributors
- May hide actual dead code if added in the future

**Recommendation:**
Add explanatory comments.

#### 3.3.2 Non-Standard Naming Convention

**Locations:** [`Kind!`](fp-macros/src/lib.rs:92) and [`Apply!`](fp-macros/src/lib.rs:335) macros

```rust
#[allow(non_snake_case)]
pub fn Kind(input: TokenStream) -> TokenStream
```

**Assessment:**
This is **intentional and correct** for HKT-style APIs, mimicking type-level syntax. No action needed.

#### 3.3.3 Documentation Complexity

**Location:** [`document_module`](fp-macros/src/lib.rs:729) macro

**Issue:**
The two-pass analysis design is complex but lacks architectural documentation:

```rust
// Pass 1: Context Extraction (handles both top-level and nested)
if let Err(e) = extract_context(&items, &mut config) {
    return e.to_compile_error();
}

// Also recursively extract from nested modules
let mut extractor = ContextExtractorVisitor { ... };
for item in &mut items {
    extractor.visit_item_mut(item);
}

// Pass 2: Documentation Generation (handles both top-level and nested)
if let Err(e) = generate_docs(&mut items, &config) {
    return e.to_compile_error();
}
```

**Impact:**
- Harder for new contributors to understand
- Maintenance challenges
- Risk of introducing bugs during modifications

**Recommendation:**
Add comprehensive architectural documentation explaining the two-pass design rationale.

---

## 4. Core Principles

The redesign is guided by the following principles, in priority order:

### 4.1 Principle 1: Single Responsibility

**Statement:**
Each module should have one clear, well-defined responsibility.

**Rationale:**
- Improves maintainability
- Simplifies testing
- Reduces coupling between components

**Application:**
- Configuration: Single module handling all configuration concerns
- Error handling: Unified error system with consistent patterns
- Each macro type (HKT, documentation, re-export) in isolated modules

**Examples:**
- ❌ **Before:** `config/` and `core/config` both handling configuration
- ✅ **After:** Single `config` module with clear API

### 4.2 Principle 2: Fail Fast with Context

**Statement:**
Errors should be detected as early as possible and include maximum context for debugging.

**Rationale:**
- Better developer experience
- Easier debugging
- Prevents cascading errors

**Application:**
- Input validation in `Parse` implementations
- Rich error types with spans and suggestions
- Contextual error messages

**Examples:**
```rust
// ❌ Before: Generic error
Err(syn::Error::new(span, "invalid input"))

// ✅ After: Contextual error
Err(Error::validation(span, "Kind definition must have at least one associated type")
    .with_suggestion("Add at least one associated type, e.g., `type Of<T>;`"))
```

### 4.3 Principle 3: Convention over Configuration

**Statement:**
Prefer sensible defaults and conventional patterns over extensive configuration options.

**Rationale:**
- Reduces cognitive load
- Improves consistency across usage
- Simplifies API

**Application:**
- Default behavior that works for 90% of cases
- Configuration only for genuinely variable concerns
- Clear documentation of conventions

**Examples:**
- Default brand mappings for common types
- Standard attribute handling patterns
- Consistent error message formatting

### 4.4 Principle 4: Explicit over Implicit

**Statement:**
Make behavior and dependencies explicit rather than relying on hidden assumptions.

**Rationale:**
- Reduces surprises
- Improves code readability
- Easier to reason about behavior

**Application:**
- Explicit error handling (no silent failures)
- Clear module boundaries
- Documented assumptions

**Examples:**
```rust
// ❌ Before: Implicit file system dependency
generate_function_re_exports!("src/classes", {})

// ✅ After: Explicit with build script or inline data
generate_function_re_exports! {
    modules: [functor, monad, applicative],
    aliases: { identity: fn_identity }
}
```

### 4.5 Principle 5: Design for Testability

**Statement:**
Architecture should facilitate comprehensive testing at all levels.

**Rationale:**
- Catches bugs early
- Enables confident refactoring
- Documents expected behavior

**Application:**
- Pure functions where possible
- Dependency injection for external resources
- Test helpers and fixtures

**Examples:**
- Mock file system for re-export testing
- Test macros for error conditions
- Property-based testing for transformations

### 4.6 Principle 6: Progressive Disclosure

**Statement:**
Present simple interfaces for common cases while allowing access to advanced features when needed.

**Rationale:**
- Lowers learning curve
- Supports both novice and expert users
- Reduces API surface area

**Application:**
- Simple macro invocations for standard cases
- Advanced options available via attributes
- Clear upgrade path from simple to complex usage

**Examples:**
```rust
// Simple case
#[hm_signature]
fn map<F, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B>

// Advanced case with configuration
#[hm_signature(ignore_traits = ["Clone"])]
fn map<F, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B>
```

### 4.7 Principle 7: Preserve Backwards Compatibility in Behavior

**Statement:**
While API breakages are acceptable, all existing behaviors, tests, and documentation quality must be maintained or improved.

**Rationale:**
- Protects user investment in current API
- Ensures redesign is purely structural, not functional
- Validates that redesign is complete

**Application:**
- All existing tests must pass with minimal changes
- Generated documentation must be identical or better
- Migration guide for API changes

---

## 5. High-Level Architecture

### 5.1 Architectural Overview

```
fp-macros/
├── src/
│   ├── lib.rs                 # Public API surface (macro entry points)
│   ├── core/                  # Core infrastructure (CONSOLIDATED)
│   │   ├── mod.rs
│   │   ├── config.rs          # Unified configuration
│   │   ├── error.rs           # Error types and handling
│   │   └── result.rs          # Result type aliases and helpers
│   ├── support/               # Support utilities (RENAMED from common)
│   │   ├── mod.rs
│   │   ├── attributes.rs      # Attribute parsing and filtering
│   │   ├── syntax.rs          # Syntax tree helpers
│   │   ├── parsing.rs         # Common parsing patterns
│   │   └── validation.rs      # Input validation helpers
│   ├── hkt/                   # Higher-Kinded Type macros
│   │   ├── mod.rs
│   │   ├── naming.rs          # Kind trait name generation
│   │   ├── definition.rs      # def_kind! implementation
│   │   ├── implementation.rs  # impl_kind! implementation
│   │   ├── application.rs     # Apply! implementation
│   │   └── types.rs           # Shared input types
│   ├── documentation/         # Documentation generation macros
│   │   ├── mod.rs
│   │   ├── signature.rs       # hm_signature implementation
│   │   ├── params.rs          # doc_params implementation
│   │   ├── type_params.rs     # doc_type_params implementation
│   │   ├── module.rs          # document_module implementation
│   │   ├── generator.rs       # Documentation generation logic
│   │   └── templates.rs       # Documentation templates
│   ├── analysis/              # Type and trait analysis
│   │   ├── mod.rs
│   │   ├── generics.rs        # Generic parameter analysis
│   │   ├── traits.rs          # Trait classification
│   │   ├── bounds.rs          # Bound analysis
│   │   └── types.rs           # Type analysis
│   ├── conversion/            # Type conversion (RENAMED from hm_conversion)
│   │   ├── mod.rs
│   │   ├── ast.rs             # HM type AST
│   │   ├── converter.rs       # Rust → HM conversion
│   │   ├── patterns.rs        # Pattern detection
│   │   ├── transform.rs       # Type transformations
│   │   └── visitors/          # AST visitors
│   ├── resolution/            # Type resolution
│   │   ├── mod.rs
│   │   ├── context.rs         # Context extraction
│   │   ├── resolver.rs        # Type resolver
│   │   └── projection.rs      # Projection key handling
│   └── codegen/               # Code generation (NEW)
│       ├── mod.rs
│       ├── reexport.rs        # Re-export generation (REFACTORED)
│       └── traits.rs          # Common codegen traits
├── tests/
│   ├── compile_fail.rs
│   ├── integration/           # Integration tests
│   ├── unit/                  # Unit tests
│   └── fixtures/              # Test fixtures
└── Cargo.toml
```

### 5.2 Module Responsibilities

| Module | Responsibility | Key Types/Functions |
|--------|---------------|---------------------|
| `core` | Infrastructure, configuration, errors | `Config`, `Error`, `Result<T>` |
| `support` | Reusable utilities | Attribute helpers, syntax helpers, parsing patterns |
| `hkt` | Higher-Kinded Type macros | `Kind!`, `def_kind!`, `impl_kind!`, `Apply!` |
| `documentation` | Documentation generation | `#[hm_signature]`, `#[doc_params]`, `#[doc_type_params]`, `#[document_module]` |
| `analysis` | Type and trait analysis | Generic analysis, trait classification |
| `conversion` | Type conversion | Rust types → Hindley-Milner |
| `resolution` | Type resolution | Self and associated type resolution |
| `codegen` | Code generation | Re-export generation, common codegen patterns |

### 5.3 Dependency Flow

```
lib.rs
  ↓
[hkt, documentation, codegen]  ← Public macro implementations
  ↓
[analysis, conversion, resolution]  ← Domain logic
  ↓
[support]  ← Utilities
  ↓
[core]  ← Infrastructure
```

**Key Constraint:** No circular dependencies. Dependencies flow downward only.

### 5.4 Error Handling Architecture

```
┌─────────────────────────────────────────────────┐
│  Macro Entry Point (lib.rs)                     │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│  Implementation Layer                            │
│  - Returns Result<T, core::Error>               │
│  - Rich error types with context                │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│  Error Conversion (via ToCompileError trait)    │
│  - core::Error → syn::Error → TokenStream       │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│  Compile Error Output                           │
│  - User-facing error message                    │
└─────────────────────────────────────────────────┘
```

**Pattern:**
```rust
// Entry point
pub fn my_macro(input: TokenStream) -> TokenStream {
    match my_macro_impl(input.into()) {
        Ok(output) => output,
        Err(e) => e.to_compile_error(),
    }
}

// Implementation
fn my_macro_impl(input: TokenStream) -> Result<TokenStream> {
    // Use ? operator throughout
    let parsed = parse_input(input)?;
    let validated = validate_input(parsed)?;
    let output = generate_output(validated)?;
    Ok(output)
}
```

### 5.5 Configuration Architecture

```
┌─────────────────────────────────────────────────┐
│  Cargo.toml                                      │
│  [package.metadata.fp_macros]                   │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│  core::config::ConfigLoader                     │
│  - Reads CARGO_MANIFEST_DIR/Cargo.toml          │
│  - Parses [package.metadata.fp_macros]          │
│  - Validates configuration                      │
│  - Caches result                                │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│  core::config::Config (read-only)               │
│  - brand_mappings: HashMap<String, String>      │
│  - apply_macro_aliases: Vec<String>             │
│  - ignored_traits: HashSet<String>              │
│  - codegen_options: CodegenOptions              │
└─────────────────────────────────────────────────┘
```

**Usage:**
```rust
use crate::core::config;

let cfg = config::get();  // Single source of truth
let brand_name = cfg.brand_mappings.get("OptionBrand");
```

---

## 6. Key Design Decisions

### 6.1 Decision 1: Unified Error System

**Decision:**
All errors must flow through `core::Error` type with standardized conversion to `TokenStream`.

**Rationale:**
- Eliminates inconsistent error handling patterns
- Enables structured error reporting
- Facilitates error context enrichment
- Simplifies testing of error conditions

**Implementation:**

```rust
// core/error.rs
pub enum Error {
    Parse(syn::Error),
    Validation { message: String, span: Span, suggestion: Option<String> },
    Resolution { message: String, span: Span, available: Vec<String> },
    Unsupported { feature: String, span: Span },
    Internal { message: String, backtrace: Option<String> },
    Io { source: io::Error, context: String },
}

impl Error {
    pub fn validation(span: Span, message: impl Into<String>) -> Self { ... }
    pub fn with_suggestion(self, suggestion: impl Into<String>) -> Self { ... }
    pub fn with_context(self, context: impl Into<String>) -> Self { ... }
}

// core/result.rs
pub trait ToCompileError {
    fn to_compile_error(self) -> TokenStream;
}

impl ToCompileError for Error {
    fn to_compile_error(self) -> TokenStream {
        let syn_error: syn::Error = self.into();
        syn_error.to_compile_error()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Migration Impact:**
- All `return syn::Error::new(...).to_compile_error()` → `return Err(Error::validation(...))?`
- All `match ... Err(e) => return syn::Error::from(e).to_compile_error()` → Use `?` operator
- Estimated changes: ~50 locations

### 6.2 Decision 2: Consolidated Configuration

**Decision:**
Single `core::config` module providing thread-safe, cached access to configuration.

**Rationale:**
- Eliminates confusion between config modules
- Ensures consistent configuration across macro invocations
- Simplifies configuration loading and caching
- Provides single source of truth

**Implementation:**

```rust
// core/config.rs
use once_cell::sync::Lazy;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Config {
    pub brand_mappings: HashMap<String, String>,
    pub apply_macro_aliases: Vec<String>,
    pub ignored_traits: HashSet<String>,
    pub codegen: CodegenConfig,
}

#[derive(Debug, Clone)]
pub struct CodegenConfig {
    pub base_module_path: String,  // e.g., "crate::classes"
    pub re_export_style: ReExportStyle,
}

static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(load_config()));

pub fn get() -> Arc<Config> {
    CONFIG.clone()
}

fn load_config() -> Config {
    // Read from CARGO_MANIFEST_DIR/Cargo.toml
    // Parse [package.metadata.fp_macros]
    // Apply defaults
    // Validate
    Config { ... }
}
```

**Migration Impact:**
- Replace `crate::config::load_config()` → `crate::core::config::get()`
- Replace `crate::core::config::get_config()` → `crate::core::config::get()`
- Remove duplicate configuration modules
- Estimated changes: ~20 locations

### 6.3 Decision 3: Trait-Based Codegen

**Decision:**
Introduce trait-based code generation abstraction to eliminate duplication in re-export macros.

**Rationale:**
- Reduces duplication between function and trait re-exports
- Enables easy addition of new item types
- Improves testability
- Makes code generation patterns explicit

**Implementation:**

```rust
// codegen/traits.rs
pub trait ItemCollector {
    type Item;
    
    fn collect_from_file(&self, file: &syn::File, module_name: &str) -> Vec<Self::Item>;
    fn is_public(&self, item: &Self::Item) -> bool;
    fn get_name(&self, item: &Self::Item) -> String;
}

pub struct FunctionCollector;
pub struct TraitCollector;

impl ItemCollector for FunctionCollector {
    type Item = syn::ItemFn;
    
    fn collect_from_file(&self, file: &syn::File, module_name: &str) -> Vec<Self::Item> {
        // Implementation
    }
}

// codegen/reexport.rs
pub fn generate_re_exports<C: ItemCollector>(
    directory: &str,
    collector: C,
    aliases: HashMap<String, String>,
) -> Result<TokenStream> {
    // Unified implementation using the collector trait
}
```

**Migration Impact:**
- Refactor `generate_function_re_exports_impl` to use trait
- Refactor `generate_trait_re_exports_impl` to use trait
- Estimated changes: ~100 lines refactored, net reduction of ~50 lines

### 6.4 Decision 4: Build Script for Re-exports

**Decision:**
Move file system scanning from procedural macros to build scripts.

**Rationale:**
- Follows Rust best practices
- Improves build reproducibility
- Better IDE integration
- Clearer dependency tracking
- Enables proper caching

**Implementation:**

**Option A: Build Script Generates Rust Code**
```rust
// build.rs
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_exports.rs");
    
    // Scan src/classes/
    let exports = scan_and_generate_exports("src/classes");
    
    // Write to OUT_DIR
    fs::write(&dest_path, exports).unwrap();
    
    // Tell Cargo to rerun if files change
    println!("cargo:rerun-if-changed=src/classes/");
}

// src/lib.rs
include!(concat!(env!("OUT_DIR"), "/generated_exports.rs"));
```

**Option B: Macro with Embedded Metadata**
```rust
// Generated by build script at build time
generate_function_re_exports! {
    modules: [
        (functor, [map, fmap]),
        (monad, [bind, chain]),
        // ...
    ],
    aliases: {
        identity: fn_identity,
    }
}
```

**Recommendation:** Option B - keeps macro-based API while eliminating file I/O.

**Migration Impact:**
- Add `build.rs` script
- Modify re-export macros to accept metadata instead of scanning
- Update documentation
- Estimated effort: Medium (new build script, refactor re-export logic)

### 6.5 Decision 5: Build-Time Metadata Generation with Simple API

**Decision:**
Move file system scanning from procedural macros to build scripts, but keep the macro invocation simple and automated.

**Rationale:**
- **Automation preserved:** Users still don't maintain item lists manually
- **Solves file I/O issue:** Build scripts can scan files, proc macros cannot
- **Better build system integration:** Proper dependency tracking
- **IDE friendly:** Metadata available for analysis tools
- **Maintains simple API:** Macro invocation stays nearly the same

**Implementation:**

**Build Script (`build.rs`):**
```rust
use std::fs;
use std::path::Path;

fn main() {
    // Scan src/classes/ directory at build time
    let metadata = scan_directory_for_reexports("src/classes");
    
    // Generate Rust code containing the metadata
    let metadata_code = generate_metadata_module(&metadata);
    
    // Write to OUT_DIR
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("reexport_metadata.rs");
    fs::write(&dest_path, metadata_code).unwrap();
    
    // Tell Cargo to rerun if directory changes
    println!("cargo:rerun-if-changed=src/classes/");
}

fn scan_directory_for_reexports(path: &str) -> Vec<ModuleInfo> {
    // Parse each .rs file in directory
    // Extract public functions, traits, types
    // Handle nested module pattern (pub use inner::*)
    // Return structured metadata
}

fn generate_metadata_module(metadata: &[ModuleInfo]) -> String {
    // Generate Rust code like:
    // pub const FUNCTION_ITEMS: &[(&str, &[&str])] = &[
    //     ("functor", &["map", "fmap", "void"]),
    //     ("monad", &["bind", "chain"]),
    // ];
}
```

**Generated Metadata (in OUT_DIR):**
```rust
// reexport_metadata.rs (auto-generated)
pub const FUNCTION_ITEMS: &[(&str, &[&str])] = &[
    ("functor", &["map", "fmap", "void"]),
    ("monad", &["bind", "chain", "join"]),
    ("applicative", &["pure", "lift_a2"]),
];

pub const TRAIT_ITEMS: &[(&str, &[&str])] = &[
    ("functor", &["Functor"]),
    ("monad", &["Monad"]),
];
```

**Macro Implementation:**
```rust
pub fn generate_function_re_exports_impl(input: ReexportInput) -> TokenStream {
    // Include the pre-generated metadata
    include!(concat!(env!("OUT_DIR"), "/reexport_metadata.rs"));
    
    // Use FUNCTION_ITEMS instead of scanning files
    let mut re_exports = Vec::new();
    
    for (module_name, items) in FUNCTION_ITEMS {
        for item_name in *items {
            // Generate re-export using metadata
            // Apply aliases if configured
        }
    }
    
    // Generate output
    quote! {
        pub use crate::classes::{
            #(#re_exports),*
        };
    }
}
```

**User API (UNCHANGED or minimal changes):**
```rust
// Option A: Completely unchanged
generate_function_re_exports!("src/classes", {
    identity: fn_identity,
});

// Option B: Simplified (path inferred from build script)
generate_function_re_exports! {
    aliases: {
        identity: fn_identity,
    }
}
```

**Benefits:**
- ✅ **Full automation:** Build script scans and extracts items automatically
- ✅ **No manual lists:** Users don't maintain item lists
- ✅ **Build reproducibility:** File scanning happens at build time
- ✅ **Proper caching:** Cargo knows when to rebuild
- ✅ **IDE support:** Generated metadata can be analyzed
- ✅ **Simple API:** Macro usage stays simple
- ✅ **Handles nested modules:** Build script can detect `pub use inner::*` pattern

**Migration Impact:**
- Add `build.rs` script with scanning logic
- Update macro to use included metadata instead of fs::read_dir
- Macro invocations **stay the same** or become simpler
- Estimated effort: Medium (build script development)
- User code changes: Minimal to none

### 6.6 Decision 6: Standardized Attribute Parsing

**Decision:**
Create unified attribute parsing infrastructure in `support::attributes`.

**Rationale:**
- Eliminates duplication
- Ensures consistent handling
- Easier to add new attributes
- Better error messages

**Implementation:**

```rust
// support/attributes.rs
pub struct AttributeParser {
    allowed: HashSet<&'static str>,
}

impl AttributeParser {
    pub fn new(allowed: &[&'static str]) -> Self { ... }
    
    pub fn parse(&self, attrs: TokenStream) -> Result<ParsedAttributes> { ... }
    
    pub fn validate_empty(&self, attrs: TokenStream) -> Result<()> {
        if !attrs.is_empty() {
            return Err(Error::validation(
                Span::call_site(),
                "This macro does not accept attributes"
            ));
        }
        Ok(())
    }
}

// Usage
let parser = AttributeParser::new(&["ignore_traits", "brand_mappings"]);
let attrs = parser.parse(attr)?;
```

**Migration Impact:**
- Refactor all attribute parsing to use unified infrastructure
- Estimated changes: ~10 locations

### 6.7 Decision 7: Error Message Style Guide

**Decision:**
Establish and enforce consistent error message formatting.

**Style Guide:**

1. **Format:** `"<context>: <specific issue>"`
2. **Capitalization:** Lowercase for specific issue, capitalize context
3. **Punctuation:** No trailing period
4. **Suggestions:** Use `with_suggestion()` for actionable fixes
5. **Available values:** Use structured error types (e.g., `Resolution` variant)

**Examples:**

```rust
// ❌ Bad
Error::validation(span, "Invalid Input!")
Error::validation(span, "kind definition must have at least one associated type.")

// ✅ Good
Error::validation(span, "Kind definition must have at least one associated type")
    .with_suggestion("Add at least one associated type, e.g., `type Of<T>;`")

Error::resolution(span, "Cannot resolve associated type `Of`", vec!["SendOf", "Of2"])
```

**Migration Impact:**
- Update all error messages to follow style guide
- Estimated changes: ~60 error messages

### 6.8 Decision 8: Module Renaming for Clarity

**Decision:**
Rename modules for improved clarity:

| Old Name | New Name | Rationale |
|----------|----------|-----------|
| `common` | `support` | More descriptive, avoids overuse of "common" |
| `hm_conversion` | `conversion` | Shorter, still clear in context |
| `re_export.rs` | `codegen/reexport.rs` | Better organization |

**Migration Impact:**
- Update all imports
- Estimated changes: ~100 import statements

### 6.9 Decision 9: Enhanced Documentation

**Decision:**
Add comprehensive internal documentation for complex subsystems.

**Requirements:**

1. **Module Documentation:** Each module must have a header comment explaining:
   - Purpose
   - Key types/functions
   - Example usage
   - Related modules

2. **Architectural Documentation:** Complex subsystems must have dedicated docs:
   - `docs/architecture/error-handling.md`
   - `docs/architecture/two-pass-analysis.md`
   - `docs/architecture/type-resolution.md`

3. **Decision Records:** Significant design decisions must be documented:
   - `docs/decisions/001-unified-error-system.md`
   - `docs/decisions/002-build-script-reexports.md`

**Migration Impact:**
- Add documentation
- No code changes
- Estimated effort: Medium (documentation writing)

---

## 7. Module-by-Module Specification

### 7.1 `core` Module

**Responsibility:** Core infrastructure including configuration, error handling, and result types.

**Public API:**

```rust
// core/mod.rs
pub mod config;
pub mod error;
pub mod result;

pub use config::{Config, get as get_config};
pub use error::Error;
pub use result::{Result, ToCompileError};
```

**Files:**

#### 7.1.1 `core/config.rs`

```rust
//! Configuration management for fp-macros.
//!
//! This module provides a unified, thread-safe configuration system that loads
//! settings from `Cargo.toml` and caches them for the duration of compilation.

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Main configuration structure
#[derive(Debug, Clone)]
pub struct Config {
    /// Brand name mappings for documentation
    /// Maps brand struct names to their display names
    pub brand_mappings: HashMap<String, String>,
    
    /// Macro names that should be treated as Apply!
    pub apply_macro_aliases: Vec<String>,
    
    /// Traits to ignore in signature constraints
    pub ignored_traits: HashSet<String>,
    
    /// Code generation options
    pub codegen: CodegenConfig,
}

/// Code generation configuration
#[derive(Debug, Clone)]
pub struct CodegenConfig {
    /// Base module path for re-exports (e.g., "crate::classes")
    pub base_module_path: String,
    
    /// Re-export style preference
    pub re_export_style: ReExportStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReExportStyle {
    /// Group all items in single use statement
    Grouped,
    /// Individual use statement per item
    Individual,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            brand_mappings: default_brand_mappings(),
            apply_macro_aliases: vec!["Apply".to_string()],
            ignored_traits: default_ignored_traits(),
            codegen: CodegenConfig {
                base_module_path: "crate::classes".to_string(),
                re_export_style: ReExportStyle::Grouped,
            },
        }
    }
}

static CONFIG: Lazy<Arc<Config>> = Lazy::new(|| Arc::new(load_config()));

/// Get the global configuration
pub fn get() -> Arc<Config> {
    CONFIG.clone()
}

fn load_config() -> Config {
    // Implementation: read from Cargo.toml, merge with defaults
}

fn default_brand_mappings() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("OptionBrand".to_string(), "Option".to_string());
    map.insert("VecBrand".to_string(), "Vec".to_string());
    // ... more defaults
    map
}

fn default_ignored_traits() -> HashSet<String> {
    let mut set = HashSet::new();
    set.insert("Sized".to_string());
    set.insert("Send".to_string());
    set.insert("Sync".to_string());
    set
}
```

#### 7.1.2 `core/error.rs`

```rust
//! Unified error handling for fp-macros.
//!
//! This module provides structured error types with rich context for
//! generating helpful compile-time error messages.

use proc_macro2::Span;
use std::fmt;
use thiserror::Error;

/// Result type alias using unified error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for fp-macros
#[derive(Debug, Error)]
pub enum Error {
    /// Parsing error from syn
    #[error("{0}")]
    Parse(#[from] syn::Error),
    
    /// Validation error with optional suggestion
    #[error("{message}")]
    Validation {
        message: String,
        span: Span,
        suggestion: Option<String>,
    },
    
    /// Resolution error with available alternatives
    #[error("{message}")]
    Resolution {
        message: String,
        span: Span,
        available: Vec<String>,
    },
    
    /// Unsupported feature
    #[error("Unsupported feature: {feature}")]
    Unsupported {
        feature: String,
        span: Span,
    },
    
    /// Internal error (should never happen)
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        backtrace: Option<String>,
    },
    
    /// I/O error with context
    #[error("I/O error: {context}")]
    Io {
        #[source]
        source: std::io::Error,
        context: String,
    },
}

impl Error {
    /// Create a validation error
    pub fn validation(span: Span, message: impl Into<String>) -> Self {
        Error::Validation {
            message: message.into(),
            span,
            suggestion: None,
        }
    }
    
    /// Add a suggestion to this error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        if let Error::Validation { suggestion: s, .. } = &mut self {
            *s = Some(suggestion.into());
        }
        self
    }
    
    /// Create a resolution error
    pub fn resolution(
        span: Span,
        message: impl Into<String>,
        available: Vec<String>,
    ) -> Self {
        Error::Resolution {
            message: message.into(),
            span,
            available,
        }
    }
    
    /// Create an unsupported feature error
    pub fn unsupported(span: Span, feature: impl Into<String>) -> Self {
        Error::Unsupported {
            feature: feature.into(),
            span,
        }
    }
    
    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Error::Internal {
            message: message.into(),
            backtrace: std::backtrace::Backtrace::capture().map(|b| b.to_string()),
        }
    }
    
    /// Add context to an error
    pub fn with_context(self, context: impl fmt::Display) -> Self {
        match self {
            Error::Validation { message, span, suggestion } => Error::Validation {
                message: format!("{}: {}", context, message),
                span,
                suggestion,
            },
            Error::Resolution { message, span, available } => Error::Resolution {
                message: format!("{}: {}", context, message),
                span,
                available,
            },
            other => other,
        }
    }
    
    /// Get the span for this error
    pub fn span(&self) -> Span {
        match self {
            Error::Parse(e) => e.span(),
            Error::Validation { span, .. } => *span,
            Error::Resolution { span, .. } => *span,
            Error::Unsupported { span, .. } => *span,
            Error::Internal { .. } => Span::call_site(),
            Error::Io { .. } => Span::call_site(),
        }
    }
}

/// Convert to syn::Error for proc macro output
impl From<Error> for syn::Error {
    fn from(err: Error) -> Self {
        let span = err.span();
        let message = err.to_string();
        
        let mut syn_err = syn::Error::new(span, message);
        
        // Add suggestion as a note
        if let Error::Validation { suggestion: Some(s), .. } = &err {
            syn_err.combine(syn::Error::new(span, format!("help: {}", s)));
        }
        
        // Add available alternatives
        if let Error::Resolution { available, .. } = &err {
            if !available.is_empty() {
                syn_err.combine(syn::Error::new(
                    span,
                    format!("note: available alternatives: {}", available.join(", "))
                ));
            }
        }
        
        syn_err
    }
}
```

#### 7.1.3 `core/result.rs`

```rust
//! Result type helpers and conversion traits.

use proc_macro2::TokenStream;

pub use crate::core::error::{Error, Result};

/// Trait for converting errors to compile-time errors
pub trait ToCompileError {
    fn to_compile_error(self) -> TokenStream;
}

impl ToCompileError for Error {
    fn to_compile_error(self) -> TokenStream {
        let syn_error: syn::Error = self.into();
        syn_error.to_compile_error()
    }
}

impl<T> ToCompileError for Result<T> {
    fn to_compile_error(self) -> TokenStream {
        match self {
            Ok(_) => panic!("Called to_compile_error on Ok value"),
            Err(e) => e.to_compile_error(),
        }
    }
}

/// Extension trait for Result with macro-specific operations
pub trait ResultExt<T> {
    /// Convert Result to TokenStream, returning output on success or error on failure
    fn into_token_stream(self) -> TokenStream;
}

impl<T: quote::ToTokens> ResultExt<T> for Result<T> {
    fn into_token_stream(self) -> TokenStream {
        match self {
            Ok(value) => quote::quote!(#value),
            Err(e) => e.to_compile_error(),
        }
    }
}
```

### 7.2 `support` Module

**Responsibility:** Reusable utilities for parsing, validation, and syntax tree manipulation.

**Public API:**

```rust
// support/mod.rs
pub mod attributes;
pub mod parsing;
pub mod syntax;
pub mod validation;

pub use attributes::{AttributeParser, DocAttributeFilter};
pub use parsing::{parse_comma_separated, parse_optional};
pub use syntax::{GenericItem, insert_doc_comment, get_doc};
pub use validation::{validate_generics, validate_bounds};
```

#### 7.2.1 `support/attributes.rs`

```rust
//! Attribute parsing and filtering utilities.

use crate::core::{Error, Result};
use proc_macro2::{Span, TokenStream};
use syn::Attribute;

/// Parser for macro attributes with validation
pub struct AttributeParser {
    allowed: HashSet<&'static str>,
}

impl AttributeParser {
    pub fn new(allowed: &[&'static str]) -> Self {
        AttributeParser {
            allowed: allowed.iter().copied().collect(),
        }
    }
    
    pub fn validate_empty(&self, attrs: TokenStream) -> Result<()> {
        if !attrs.is_empty() {
            return Err(Error::validation(
                Span::call_site(),
                "This macro does not accept attributes"
            ));
        }
        Ok(())
    }
    
    pub fn parse(&self, attrs: TokenStream) -> Result<ParsedAttributes> {
        // Implementation
    }
}

/// Filter for documentation-specific attributes
pub struct DocAttributeFilter;

impl DocAttributeFilter {
    /// Filter out doc-specific attributes that should not appear in generated code
    pub fn filter_doc_attrs(attrs: &[Attribute]) -> Vec<Attribute> {
        attrs.iter()
            .filter(|attr| {
                !attr.path().is_ident("doc_default")
                    && !attr.path().is_ident("doc_use")
            })
            .cloned()
            .collect()
    }
}
```

#### 7.2.2 `support/validation.rs`

```rust
//! Input validation helpers.

use crate::core::{Error, Result};
use proc_macro2::Span;
use syn::{GenericParam, Generics};

/// Validate that generics don't contain unsupported features
pub fn validate_generics(generics: &Generics) -> Result<()> {
    for param in &generics.params {
        if let GenericParam::Const(const_param) = param {
            return Err(Error::unsupported(
                const_param.ident.span(),
                "Const generic parameters are not supported in Kind definitions"
            ).with_suggestion(
                "Remove const parameters or use a different approach"
            ));
        }
    }
    Ok(())
}

/// Validate that a list is non-empty
pub fn validate_non_empty<T>(items: &[T], span: Span, what: &str) -> Result<()> {
    if items.is_empty() {
        return Err(Error::validation(
            span,
            format!("{} must not be empty", what)
        ));
    }
    Ok(())
}
```

### 7.3 `hkt` Module

**Responsibility:** Higher-Kinded Type macro implementations.

**Public API:**

```rust
// hkt/mod.rs
mod application;
mod definition;
mod implementation;
mod naming;
mod types;

pub use application::apply_impl;
pub use definition::def_kind_impl;
pub use implementation::impl_kind_impl;
pub use naming::generate_kind_name;
pub use types::{ApplyInput, KindInput, ImplKindInput};
```

#### 7.3.1 `hkt/naming.rs`

```rust
//! Kind trait name generation.

use crate::core::Result;
use crate::hkt::types::KindInput;
use proc_macro2::Ident;

/// Generate a deterministic name for a Kind trait based on its signature
pub fn generate_kind_name(input: &KindInput) -> Result<Ident> {
    // Implementation using hash-based naming
}
```

#### 7.3.2 `hkt/definition.rs`

```rust
//! def_kind! macro implementation.

use crate::core::Result;
use crate::hkt::{generate_kind_name, KindInput};
use proc_macro2::TokenStream;
use quote::quote;

pub fn def_kind_impl(input: KindInput) -> Result<TokenStream> {
    use crate::support::validation::validate_generics;
    
    // Validate input
    for assoc in &input.assoc_types {
        validate_generics(&assoc.generics)?;
    }
    
    let name = generate_kind_name(&input)?;
    let assoc_types = generate_assoc_types(&input);
    let doc = generate_documentation(&name, &input);
    
    Ok(quote! {
        #[doc = #doc]
        #[allow(non_camel_case_types)]
        pub trait #name {
            #(#assoc_types)*
        }
    })
}

fn generate_assoc_types(input: &KindInput) -> Vec<TokenStream> {
    // Implementation
}

fn generate_documentation(name: &Ident, input: &KindInput) -> String {
    // Implementation
}
```

### 7.4 `documentation` Module

**Responsibility:** Documentation generation macro implementations.

**Structure:**

```rust
// documentation/mod.rs
mod signature;      // hm_signature implementation
mod params;         // doc_params implementation
mod type_params;    // doc_type_params implementation
mod module;         // document_module implementation
mod generator;      // Shared generation logic
mod templates;      // Documentation templates

pub use signature::hm_signature_impl;
pub use params::doc_params_impl;
pub use type_params::doc_type_params_impl;
pub use module::document_module_impl;
```

### 7.5 `codegen` Module (NEW)

**Responsibility:** Code generation utilities including re-export generation.

```rust
// codegen/mod.rs
mod reexport;
mod traits;

pub use reexport::{generate_function_re_exports, generate_trait_re_exports};
pub use traits::ItemCollector;
```

#### 7.5.1 `codegen/traits.rs`

```rust
//! Code generation traits.

use syn::File;

/// Trait for collecting items from parsed Rust files
pub trait ItemCollector {
    type Item;
    
    /// Collect items from a parsed file
    fn collect_from_file(&self, file: &File) -> Vec<Self::Item>;
    
    /// Check if an item is public
    fn is_public(&self, item: &Self::Item) -> bool;
    
    /// Get the item's name
    fn get_name(&self, item: &Self::Item) -> String;
}
```

#### 7.5.2 `codegen/reexport.rs`

```rust
//! Re-export generation using trait-based abstraction.

use crate::codegen::traits::ItemCollector;
use crate::core::Result;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;

pub struct ReExportConfig {
    pub base_path: String,
    pub modules: HashMap<String, Vec<String>>,
    pub aliases: HashMap<String, String>,
}

pub fn generate_re_exports<C: ItemCollector>(
    config: ReExportConfig,
    collector: C,
) -> Result<TokenStream> {
    // Unified implementation using the collector trait
    // No file I/O - works from provided metadata
}

pub struct FunctionCollector;
pub struct TraitCollector;

impl ItemCollector for FunctionCollector {
    type Item = syn::ItemFn;
    // Implementation
}

impl ItemCollector for TraitCollector {
    type Item = syn::ItemTrait;
    // Implementation
}
```

---

## 8. Migration Strategy

### 8.1 Migration Phases

The migration will proceed in phases to minimize risk and enable incremental validation.

#### Phase 1: Foundation (Infrastructure)

**Objective:** Establish core infrastructure without breaking existing functionality.

**Steps:**

1. **Create new module structure**
   - Create `core/` directory with `config.rs`, `error.rs`, `result.rs`
   - Create `support/` directory (initially empty)
   - Keep existing modules functional

2. **Implement unified configuration**
   - Implement `core::config::Config` with all fields
   - Implement `core::config::get()` with caching
   - Add tests for configuration loading
   - **DO NOT** remove old config modules yet

3. **Implement unified error system**
   - Implement `core::Error` enum with all variants
   - Implement `ToCompileError` trait
   - Add comprehensive error tests
   - **DO NOT** change existing error handling yet

4. **Validation:**
   - All tests pass
   - New infrastructure has >90% test coverage
   - Documentation complete for new modules

#### Phase 2: Support Utilities

**Objective:** Extract and consolidate support utilities.

**Steps:**

1. **Move common utilities to support/**
   - Copy (don't move) `common/attributes.rs` → `support/attributes.rs`
   - Copy `common/syntax.rs` → `support/syntax.rs`
   - Create `support/parsing.rs` with common parsing patterns
   - Create `support/validation.rs` with validation helpers

2. **Implement AttributeParser**
   - Create unified attribute parsing infrastructure
   - Add tests

3. **Validation:**
   - All tests pass
   - Support module has >85% test coverage

#### Phase 3: HKT Macros Migration

**Objective:** Migrate HKT macros to use new infrastructure.

**Steps:**

1. **Refactor Kind! macro**
   - Update to use `core::Error`
   - Update to use `core::config::get()`
   - Update error messages per style guide
   - Keep same behavior

2. **Refactor def_kind! macro**
   - Same updates as Kind!
   - Ensure documentation generation unchanged

3. **Refactor impl_kind! macro**
   - Same updates
   - Use `support::validation` helpers

4. **Refactor Apply! macro**
   - Same updates

5. **Validation:**
   - All existing tests pass
   - Generated code identical to before (diff check)
   - Error messages improved

#### Phase 4: Documentation Macros Migration

**Objective:** Migrate documentation macros to use new infrastructure.

**Steps:**

1. **Refactor hm_signature**
   - Use unified error handling
   - Use unified configuration
   - Use AttributeParser

2. **Refactor doc_params**
   - Same updates

3. **Refactor doc_type_params**
   - Same updates

4. **Refactor document_module**
   - Same updates
   - Add architectural documentation for two-pass analysis

5. **Validation:**
   - All tests pass
   - Generated documentation identical
   - Error messages improved

#### Phase 5: Codegen Migration
**Objective:** Refactor re-export generation to use build-time metadata while preserving simple API.
**Steps:**
1. **Create build script infrastructure**
   - Create `build.rs` with directory scanning logic
   - Implement file parsing to extract public items
   - Handle nested module pattern (`pub use inner::*`)
   - Generate metadata file in `OUT_DIR`
   - Add proper `cargo:rerun-if-changed` directives
2. **Implement trait-based codegen**
   - Create `codegen/` module
   - Implement `ItemCollector` trait for abstraction
   - Implement `FunctionCollector` and `TraitCollector`
   - Create metadata inclusion utilities
3. **Update re-export macro implementation**
   - Modify macros to `include!` metadata from `OUT_DIR`
   - Remove all file I/O code (fs::read_dir, fs::read_to_string, etc.)
   - Use pre-generated metadata instead of runtime scanning
   - Keep same or simpler user-facing API
   - Maintain alias functionality
4. **Test and validate**
   - Verify build script generates correct metadata
   - Verify macros expand using metadata
   - **User invocations stay the same** (or become simpler)
   - Verify generated re-exports are identical to before

5. **Validation:**
   - Generated re-exports identical to previous version
   - Build script triggers rebuilds correctly when files change
   - All tests pass
   - No file I/O in procedural macros
   - User code requires minimal to no changes

#### Phase 6: Cleanup

**Objective:** Remove old code and finalize migration.

**Steps:**

1. **Remove duplicate modules**
   - Delete old `config/` module (keep only `core/config`)
   - Delete old `core/config.rs` (if duplicate)
   - Delete `common/` module (replaced by `support/`)

2. **Rename modules**
   - Rename `hm_conversion/` → `conversion/`
   - Move `re_export.rs` → `codegen/reexport.rs` (if not done)

3. **Update all imports**
   - Replace `crate::config` → `crate::core::config`
   - Replace `crate::common` → `crate::support`
   - Replace `crate::hm_conversion` → `crate::conversion`

4. **Update documentation**
   - Update README
   - Update module documentation
   - Update examples

5. **Validation:**
   - All tests pass
   - No compiler warnings
   - Documentation builds successfully
   - No dead code

#### Phase 7: Enhancement

**Objective:** Add improvements and polish.

**Steps:**

1. **Add comprehensive documentation**
   - Write architectural documentation for complex subsystems
   - Add decision records
   - Update inline documentation

2. **Enhance error messages**
   - Review all error messages against style guide
   - Add suggestions where helpful
   - Ensure consistent formatting

3. **Add performance tests**
   - Benchmark macro expansion time
   - Compare with previous version
   - Ensure no performance regression

4. **Final code review**
   - Review all changed code
   - Ensure style consistency
   - Remove any TODO comments

5. **Validation:**
   - Documentation complete and accurate
   - All tests pass
   - No performance regression
   - Code quality checks pass

### 8.2 Migration Checklist

#### Infrastructure
- [ ] Create `core/` module with `config.rs`, `error.rs`, `result.rs`
- [ ] Create `support/` module structure
- [ ] Implement unified `Config` type
- [ ] Implement `core::config::get()` with caching
- [ ] Implement `core::Error` enum
- [ ] Implement `ToCompileError` trait
- [ ] Add comprehensive tests for core infrastructure
- [ ] Verify all tests still pass

#### Support Utilities
- [ ] Create `support/attributes.rs` with `AttributeParser`
- [ ] Create `support/validation.rs` with validation helpers
- [ ] Create `support/parsing.rs` with parsing patterns
- [ ] Migrate `common/syntax.rs` → `support/syntax.rs`
- [ ] Add tests for support utilities
- [ ] Verify all tests still pass

#### HKT Macros
- [ ] Refactor `Kind!` to use unified error handling
- [ ] Refactor `def_kind!` to use unified error handling
- [ ] Refactor `impl_kind!` to use unified error handling
- [ ] Refactor `Apply!` to use unified error handling
- [ ] Update all error messages per style guide
- [ ] Verify generated code identical to before
- [ ] Verify all tests pass

#### Documentation Macros
- [ ] Refactor `hm_signature` to use unified infrastructure
- [ ] Refactor `doc_params` to use unified infrastructure
- [ ] Refactor `doc_type_params` to use unified infrastructure
- [ ] Refactor `document_module` to use unified infrastructure
- [ ] Add architectural documentation for two-pass analysis
- [ ] Verify generated documentation identical
- [ ] Verify all tests pass

#### Codegen
- [ ] Create `build.rs` script with directory scanning
- [ ] Implement file parsing to extract public items
- [ ] Handle nested module pattern detection (`pub use inner::*`)
- [ ] Implement metadata generation in `OUT_DIR`
- [ ] Add `cargo:rerun-if-changed` directives
- [ ] Create `codegen/` module
- [ ] Implement `ItemCollector` trait
- [ ] Implement `FunctionCollector` and `TraitCollector`
- [ ] Update re-export macros to use `include!` for metadata
- [ ] Remove all file I/O from procedural macros
- [ ] **User invocations stay the same** (or become simpler)
- [ ] Verify build script generates correct metadata
- [ ] Verify macros expand using metadata
- [ ] Verify generated re-exports identical
- [ ] Verify all tests pass

#### Cleanup
- [ ] Remove old `config/` module
- [ ] Remove old `common/` module
- [ ] Rename `hm_conversion/` → `conversion/`
- [ ] Update all imports
- [ ] Remove dead code
- [ ] Fix all compiler warnings
- [ ] Verify all tests pass
- [ ] Verify documentation builds

#### Enhancement
- [ ] Write architectural documentation
- [ ] Add decision records
- [ ] Review all error messages
- [ ] Add performance benchmarks
- [ ] Final code review
- [ ] Update README and examples
- [ ] Verify all quality checks pass

### 8.3 Breaking Changes

The following breaking changes are acceptable:

#### Import Path Changes
```rust
// Before
use fp_macros::config::Config;
use fp_macros::common::syntax::GenericItem;

// After
use fp_macros::core::config::Config;
use fp_macros::support::syntax::GenericItem;
```

#### Configuration API Changes
```rust
// Before
let config = load_config();

// After
let config = core::config::get();
```

#### Re-export Macro Invocation Changes
```rust
// Before (current)
generate_function_re_exports!("src/classes", {
    identity: fn_identity,
});

// After (UNCHANGED or simpler)
// Option A: Stays exactly the same
generate_function_re_exports!("src/classes", {
    identity: fn_identity,
});

// Option B: Even simpler (path configured in build script)
generate_function_re_exports! {
    aliases: {
        identity: fn_identity,
    }
}

// Implementation difference: Uses build-time metadata instead of runtime file I/O
// User API preserved - no manual item lists required
```

#### Error Type Changes (Internal Only)
Internal error types change but user-facing error messages remain the same or improve.

### 8.4 Compatibility Guarantees

**MUST Remain Identical:**
- Generated trait definitions from `def_kind!`
- Generated impl blocks from `impl_kind!`
- Generated type projections from `Apply!`
- Generated documentation from all doc macros
- Macro behavior and semantics
- Error positions and spans

**MAY Change:**
- Error message wording (must be equal or better)
- Internal module structure
- Internal function signatures
- Import paths

**MUST NOT Break:**
- Any existing valid macro invocation
- Generated code compilation
- Test suite

### 8.5 Validation Criteria

Each phase must meet these criteria before proceeding:

1. **All tests pass** - No test regressions
2. **No new warnings** - Clean compilation
3. **Documentation builds** - `cargo doc` succeeds
4. **Generated code identical** - Use diff tools to verify
5. **Error messages equal or better** - Review error output
6. **Performance maintained** - Benchmark macro expansion

### 8.6 Rollback Strategy

If issues arise during migration:

1. **Git branches** - Each phase in separate branch
2. **Commit granularity** - Small, logical commits
3. **Testing checkpoints** - Test after each commit
4. **Revert capability** - Can revert individual commits
5. **Feature flags** - Use conditional compilation if needed

---

## 9. Quality Assurance

### 9.1 Testing Strategy

#### 9.1.1 Unit Tests

**Coverage Target:** >85% for all new code

**Required Tests:**
- Configuration loading and caching
- Error creation and conversion
- Attribute parsing
- Validation helpers
- Name generation
- Each macro implementation

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_validation_creation() {
        let err = Error::validation(Span::call_site(), "test message");
        assert!(err.to_string().contains("test message"));
    }
    
    #[test]
    fn test_error_with_suggestion() {
        let err = Error::validation(Span::call_site(), "invalid")
            .with_suggestion("try this instead");
        let syn_err: syn::Error = err.into();
        assert!(syn_err.to_string().contains("help:"));
    }
}
```

#### 9.1.2 Integration Tests

**Required Tests:**
- End-to-end macro expansion
- Multiple macro combinations
- Edge cases and corner cases
- Error conditions

**Example:**
```rust
#[test]
fn test_kind_and_apply_integration() {
    def_kind!(type Of<T>;);
    
    impl_kind! {
        for OptionBrand {
            type Of<T> = Option<T>;
        }
    }
    
    type Result = Apply!(<OptionBrand as Kind!(type Of<T>;)>::Of<i32>);
    // Should expand to: Option<i32>
}
```

#### 9.1.3 Compile-Fail Tests

**Required Tests:**
- Invalid syntax
- Unsupported features
- Type errors
- Missing required elements

**Example:**
```rust
// tests/ui/const_generics_not_supported.rs
use fp_macros::def_kind;

def_kind!(type Of<const N: usize>;);
//~^ ERROR: Const generic parameters are not supported
```

#### 9.1.4 Property-Based Tests

**Required Tests:**
- Name generation determinism
- Canonicalization correctness
- Type transformation properties

**Example:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_name_generation_deterministic(input in any::<KindInput>()) {
        let name1 = generate_kind_name(&input)?;
        let name2 = generate_kind_name(&input)?;
        assert_eq!(name1, name2);
    }
}
```

#### 9.1.5 Regression Tests

**Process:**
1. For each bug fix, add test reproducing the bug
2. Verify test fails before fix
3. Verify test passes after fix
4. Keep test in suite permanently

### 9.2 Code Quality Standards

#### 9.2.1 Rustfmt

All code must be formatted with `rustfmt`:
```bash
cargo fmt --all -- --check
```

#### 9.2.2 Clippy

All code must pass Clippy without warnings:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**Allowed Exceptions:**
- `#[allow(non_snake_case)]` for `Kind!` and `Apply!` macros (intentional)
- `#[allow(dead_code)]` for proc macro input structs (explained with comment)

#### 9.2.3 Documentation

**Requirements:**
- All public items must have doc comments
- All modules must have module-level documentation
- Examples in doc comments must be tested
- Complex algorithms must be explained

**Format:**
```rust
/// Brief one-line summary.
///
/// More detailed explanation of what this does, why it exists,
/// and how to use it.
///
/// # Examples
///
/// ```
/// use fp_macros::core::Error;
/// let err = Error::validation(span, "message");
/// ```
///
/// # Errors
///
/// Returns error if validation fails.
```

#### 9.2.4 Error Handling

**Requirements:**
- No panics in production code (except for internal invariants)
- No unwrap() or expect() (use ? operator)
- All errors must include spans
- Error messages follow style guide

**Example:**
```rust
// ❌ Bad
let value = some_option.unwrap();
let result = some_result.expect("should work");

// ✅ Good
let value = some_option.ok_or_else(|| Error::internal("expected value"))?;
let result = some_result?;
```

### 9.3 Performance Requirements

#### 9.3.1 Compilation Time

**Target:** Macro expansion should not add more than 5% to compilation time.

**Measurement:**
```bash
cargo clean
cargo build --timings
# Compare macro expansion time before/after
```

#### 9.3.2 Memory Usage

**Target:** No significant memory regression during macro expansion.

**Measurement:**
```bash
RUSTFLAGS="-Z macro-backtrace" cargo +nightly build
# Monitor peak memory usage
```

### 9.4 Security Considerations

#### 9.4.1 Path Traversal

**Risk:** User-provided paths in macros could access arbitrary files.

**Mitigation:**
- Validate all paths are within project directory
- Use canonicalization to prevent .. escapes
- Limit to specific allowed directories

**Example:**
```rust
fn validate_path(path: &Path) -> Result<()> {
    let canonical = path.canonicalize()
        .map_err(|e| Error::io(e, format!("Invalid path: {:?}", path)))?;
    
    let project_root = env::var("CARGO_MANIFEST_DIR")?;
    let root_canonical = Path::new(&project_root).canonicalize()?;
    
    if !canonical.starts_with(&root_canonical) {
        return Err(Error::validation(
            Span::call_site(),
            "Path must be within project directory"
        ));
    }
    
    Ok(())
}
```

#### 9.4.2 Resource Exhaustion

**Risk:** Malicious input could cause excessive memory or CPU usage.

**Mitigation:**
- Limit recursion depth
- Limit input size
- Timeout for expensive operations

**Example:**
```rust
const MAX_ASSOC_TYPES: usize = 100;
const MAX_RECURSION_DEPTH: usize = 64;

fn validate_input_size(input: &KindInput) -> Result<()> {
    if input.assoc_types.len() > MAX_ASSOC_TYPES {
        return Err(Error::validation(
            Span::call_site(),
            format!("Too many associated types (max: {})", MAX_ASSOC_TYPES)
        ));
    }
    Ok(())
}
```

### 9.5 Documentation Requirements

#### 9.5.1 User Documentation

**Required:**
- README with overview and quick start
- Migration guide for breaking changes
- Examples for all macros
- Configuration documentation
- Troubleshooting guide

#### 9.5.2 Developer Documentation

**Required:**
- Architecture overview
- Module responsibility matrix
- Design decision records
- Testing guide
- Contribution guidelines

#### 9.5.3 API Documentation

**Required:**
- All public items documented
- Examples that compile and pass tests
- Links between related items
- Clear error documentation

---

## 10. Appendices

### 10.1 Appendix A: Complete Module Structure

```
fp-macros/
├── Cargo.toml
├── README.md
├── build.rs                           # NEW: Module metadata generation
├── src/
│   ├── lib.rs                         # Public API surface
│   │
│   ├── core/                          # Core infrastructure
│   │   ├── mod.rs
│   │   ├── config.rs                  # Unified configuration
│   │   ├── error.rs                   # Error types
│   │   └── result.rs                  # Result helpers
│   │
│   ├── support/                       # Support utilities (renamed from common)
│   │   ├── mod.rs
│   │   ├── attributes.rs              # Attribute parsing
│   │   ├── syntax.rs                  # Syntax helpers
│   │   ├── parsing.rs                 # Parsing patterns
│   │   └── validation.rs              # Validation helpers
│   │
│   ├── hkt/                           # Higher-Kinded Type macros
│   │   ├── mod.rs
│   │   ├── naming.rs                  # Kind name generation
│   │   ├── definition.rs              # def_kind! implementation
│   │   ├── implementation.rs          # impl_kind! implementation
│   │   ├── application.rs             # Apply! implementation
│   │   └── types.rs                   # Shared types
│   │
│   ├── documentation/                 # Documentation generation
│   │   ├── mod.rs
│   │   ├── signature.rs               # hm_signature
│   │   ├── params.rs                  # doc_params
│   │   ├── type_params.rs             # doc_type_params
│   │   ├── module.rs                  # document_module
│   │   ├── generator.rs               # Generation logic
│   │   └── templates.rs               # Templates
│   │
│   ├── analysis/                      # Type and trait analysis
│   │   ├── mod.rs
│   │   ├── generics.rs                # Generic analysis
│   │   ├── traits.rs                  # Trait classification
│   │   ├── bounds.rs                  # Bound analysis
│   │   └── types.rs                   # Type analysis
│   │
│   ├── conversion/                    # Type conversion (renamed from hm_conversion)
│   │   ├── mod.rs
│   │   ├── ast.rs                     # HM type AST
│   │   ├── converter.rs               # Rust → HM conversion
│   │   ├── patterns.rs                # Pattern detection
│   │   ├── transform.rs               # Transformations
│   │   └── visitors/
│   │       ├── mod.rs
│   │       ├── extraction.rs
│   │       └── transformation.rs
│   │
│   ├── resolution/                    # Type resolution
│   │   ├── mod.rs
│   │   ├── context.rs                 # Context extraction
│   │   ├── resolver.rs                # Type resolver
│   │   └── projection.rs              # Projection keys
│   │
│   └── codegen/                       # Code generation (NEW)
│       ├── mod.rs
│       ├── reexport.rs                # Re-export generation
│       └── traits.rs                  # Codegen traits
│
├── tests/
│   ├── integration/                   # Integration tests
│   │   ├── hkt.rs
│   │   ├── documentation.rs
│   │   └── codegen.rs
│   ├── ui/                            # Compile-fail tests
│   │   ├── invalid_input.rs
│   │   └── invalid_input.stderr
│   └── fixtures/                      # Test fixtures
│
├── benches/                           # Performance benchmarks
│   └── macro_expansion.rs
│
└── docs/
    ├── architecture/
    │   ├── overview.md
    │   ├── error-handling.md
    │   ├── two-pass-analysis.md
    │   └── type-resolution.md
    ├── decisions/
    │   ├── 001-unified-error-system.md
    │   ├── 002-build-script-reexports.md
    │   └── 003-trait-based-codegen.md
    └── migration-guide.md
```

### 10.2 Appendix B: Error Message Style Guide

#### Format

```
<Context>: <specific issue>
```

#### Rules

1. **Capitalization:** 
   - Context: Title case (e.g., "Kind definition")
   - Issue: Lowercase (e.g., "must have at least one associated type")

2. **Punctuation:**
   - No trailing period
   - Use colons to separate context and issue

3. **Clarity:**
   - Be specific about what went wrong
   - Use precise terminology
   - Avoid jargon where possible

4. **Actionability:**
   - Include suggestions when possible
   - Show available alternatives
   - Point to documentation

#### Examples

**✅ Good:**
```rust
Error::validation(span, "Kind definition must have at least one associated type")
    .with_suggestion("Add at least one associated type, e.g., `type Of<T>;`")

Error::resolution(
    span,
    "Cannot resolve associated type `Of` for Self",
    vec!["SendOf", "SyncOf"]
)

Error::unsupported(span, "Const generic parameters are not supported in Kind definitions")
    .with_suggestion("Remove const parameters or use a different approach")
```

**❌ Bad:**
```rust
Error::validation(span, "Invalid Input!")  // Too vague, exclamation mark
Error::validation(span, "kind definition must have at least one associated type.")  // Not capitalized, trailing period
Error::validation(span, "error")  // Too vague
```

### 10.3 Appendix C: Configuration Schema

```toml
[package.metadata.fp_macros]

# Brand name mappings for documentation
# Maps Rust type names to display names in HM signatures
[package.metadata.fp_macros.brand_mappings]
OptionBrand = "Option"
VecBrand = "Vec"
ResultBrand = "Result"

# Macro names treated as Apply!
apply_macro_aliases = ["Apply", "MyCustomApply"]

# Traits to ignore in HM signature constraints
ignored_traits = ["Sized", "Send", "Sync", "Clone", "Debug"]

# Code generation options
[package.metadata.fp_macros.codegen]
base_module_path = "crate::classes"
re_export_style = "grouped"  # or "individual"

# Module re-export configuration (generated by build script)
[[package.metadata.fp_macros.codegen.modules]]
name = "functor"
items = ["map", "fmap", "void"]

[[package.metadata.fp_macros.codegen.modules]]
name = "monad"
items = ["bind", "chain", "join"]

# Re-export aliases
[package.metadata.fp_macros.codegen.aliases]
"functor::identity" = "fn_identity"
"category::identity" = "category_identity"
```

### 10.4 Appendix D: Decision Record Template

```markdown
# Decision Record: [Title]

**Status:** [Proposed | Accepted | Deprecated | Superseded]  
**Date:** YYYY-MM-DD  
**Deciders:** [Names/roles]  

## Context

[Describe the context and problem statement]

## Decision

[Describe the decision]

## Rationale

[Explain why this decision was made]

## Consequences

### Positive
- [List positive consequences]

### Negative
- [List negative consequences and how they're mitigated]

## Alternatives Considered

### Alternative 1: [Name]
- **Description:** [Brief description]
- **Pros:** [List pros]
- **Cons:** [List cons]
- **Reason for rejection:** [Why not chosen]

## Implementation Notes

[Any notes for implementation]

## Related Decisions

- [Links to related decisions]
```

### 10.5 Appendix E: Testing Matrix

| Feature | Unit Tests | Integration Tests | Compile-Fail Tests | Property Tests |
|---------|------------|-------------------|-------------------|----------------|
| Configuration Loading | ✓ | ✓ | - | - |
| Error Creation | ✓ | - | - | - |
| Error Conversion | ✓ | ✓ | ✓ | - |
| Attribute Parsing | ✓ | - | ✓ | - |
| Kind! Macro | ✓ | ✓ | ✓ | ✓ (determinism) |
| def_kind! Macro | ✓ | ✓ | ✓ | ✓ (determinism) |
| impl_kind! Macro | ✓ | ✓ | ✓ | - |
| Apply! Macro | ✓ | ✓ | ✓ | - |
| hm_signature | ✓ | ✓ | ✓ | - |
| doc_params | ✓ | ✓ | ✓ | - |
| doc_type_params | ✓ | ✓ | ✓ | - |
| document_module | ✓ | ✓ | ✓ | - |
| Re-export Generation | ✓ | ✓ | ✓ | - |
| Type Resolution | ✓ | ✓ | - | - |
| HM Conversion | ✓ | ✓ | - | ✓ (round-trip) |

### 10.6 Appendix F: Common Patterns

#### Pattern 1: Macro Entry Point

```rust
#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    match my_macro_impl(input.into()) {
        Ok(output) => output,
        Err(e) => e.to_compile_error(),
    }
}

fn my_macro_impl(input: TokenStream) -> Result<TokenStream> {
    let parsed = syn::parse2::<MyInput>(input)?;
    let validated = validate(parsed)?;
    let output = generate(validated)?;
    Ok(output)
}
```

#### Pattern 2: Validation

```rust
fn validate(input: MyInput) -> Result<MyInput> {
    use crate::support::validation::*;
    
    validate_non_empty(&input.items, input.span, "Item list")?;
    validate_generics(&input.generics)?;
    
    // Custom validation
    if input.name.to_string().starts_with("_") {
        return Err(Error::validation(
            input.name.span(),
            "Name cannot start with underscore"
        ).with_suggestion("Use a different name"));
    }
    
    Ok(input)
}
```

#### Pattern 3: Error with Context

```rust
fn process_items(items: &[Item]) -> Result<Vec<TokenStream>> {
    items.iter()
        .map(|item| {
            process_item(item)
                .map_err(|e| e.with_context(format!("While processing item {}", item.name)))
        })
        .collect()
}
```

#### Pattern 4: Configuration Access

```rust
fn my_function() -> Result<TokenStream> {
    let config = core::config::get();
    
    let brand_name = config.brand_mappings
        .get("OptionBrand")
        .unwrap_or(&"Option".to_string());
    
    // Use brand_name...
}
```

### 10.7 Appendix G: Glossary

| Term | Definition |
|------|------------|
| **Brand** | A zero-sized type used to tag Higher-Kinded Types |
| **HKT** | Higher-Kinded Type - a type constructor that abstracts over type constructors |
| **Kind trait** | A trait representing a higher-kinded type signature |
| **Associated type** | A type member of a trait that can be specified by implementors |
| **Projection** | The process of applying a brand to type arguments to get a concrete type |
| **HM type** | Hindley-Milner type - a type in a mathematical type system used for type inference |
| **Span** | A source code location used for error reporting |
| **TokenStream** | A sequence of tokens representing Rust code |
| **Proc macro** | Procedural macro - a function that operates on TokenStreams |
| **Compile-fail test** | A test that verifies code fails to compile with expected errors |

### 10.8 Appendix H: References

#### Internal Documentation
- Architecture overview: `docs/architecture/overview.md`
- Error handling guide: `docs/architecture/error-handling.md`
- Two-pass analysis: `docs/architecture/two-pass-analysis.md`
- Type resolution: `docs/architecture/type-resolution.md`

#### External Resources
- Rust Procedural Macros Book: https://doc.rust-lang.org/reference/procedural-macros.html
- syn Documentation: https://docs.rs/syn/latest/syn/
- quote Documentation: https://docs.rs/quote/latest/quote/
- Higher-Kinded Types in Rust: Various blog posts and discussions

#### Related Tools
- `cargo-expand`: View macro expansions
- `cargo-fmt`: Code formatting
- `cargo-clippy`: Linting
- `cargo-doc`: Documentation generation

---

## Summary

This specification defines a comprehensive redesign of the `fp-macros` crate that:

1. **Addresses all identified issues** through systematic refactoring
2. **Establishes clear architectural principles** for long-term maintainability
3. **Provides detailed implementation guidance** with concrete code examples
4. **Defines rigorous quality standards** for testing and documentation
5. **Enables safe migration** through phased approach with validation at each step

The redesign maintains all existing functionality while significantly improving:
- **Consistency:** Unified error handling and configuration
- **Clarity:** Better module organization and naming
- **Robustness:** Enhanced validation and error messages
- **Maintainability:** Reduced duplication and improved patterns
- **Extensibility:** Trait-based abstractions for future growth

**Key Success Metrics:**
- All existing tests pass
- Generated code remains identical
- Error messages equal or better
- No performance regression
- >85% test coverage
- Zero compiler warnings
- Complete documentation

**Migration Approach:**
- Phased implementation with validation checkpoints
- API breakages acceptable where they improve clarity
- Behavior must remain identical
- Clear migration guide for users
- Rollback capability at each phase

This specification serves as the blueprint for a production-ready, maintainable, and extensible macro system that will serve the fp-library project for years to come.
