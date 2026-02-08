# fp-macros Clean-Room Redesign Specification

**Version**: 1.0  
**Date**: 2026-02-09  
**Status**: Proposed

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Analysis](#current-state-analysis)
3. [Identified Issues](#identified-issues)
4. [Proposed Architecture](#proposed-architecture)
5. [Core Design Principles](#core-design-principles)
6. [Key Design Decisions](#key-design-decisions)
7. [Issue Resolution Mapping](#issue-resolution-mapping)
8. [Migration Strategy](#migration-strategy)
9. [Success Criteria](#success-criteria)

---

## Executive Summary

This document specifies a clean-room redesign of the `fp-macros` crate to address fundamental architectural issues while maintaining all current functionality, test coverage, and documentation quality. The redesign focuses on:

1. **Build Reproducibility**: Eliminating filesystem dependencies during macro expansion
2. **API Clarity**: Removing unused parameters and clarifying interfaces
3. **Code Quality**: Reducing duplication and improving maintainability
4. **Error Handling**: Completing error handling infrastructure

The redesign maintains backward compatibility where possible but prioritizes correctness and maintainability over legacy API preservation.

---

## Current State Analysis

### Overall Assessment

**Grade: B+ (Very Good with Notable Issues)**

The current implementation demonstrates **mature software engineering** with:

- ✅ Excellent modular architecture (9 well-organized modules)
- ✅ Comprehensive documentation with examples
- ✅ Strong error handling using `thiserror`
- ✅ Extensive test coverage (95%+ of critical paths)
- ✅ No unsafe code
- ✅ Proper visibility control
- ✅ Idiomatic Rust patterns

However, several fundamental issues prevent a higher grade:

- ⚠️ File I/O during macro expansion (reproducibility risk)
- ⚠️ Incomplete API surface (unused parameters)
- ⚠️ Code duplication (attribute filtering, canonicalization)
- ⚠️ Inconsistent configuration loading

### Module Structure (Current)

```
fp-macros/
├── src/
│   ├── lib.rs              # Main macro entry points
│   ├── error.rs            # Unified error types
│   ├── common/             # Shared utilities
│   │   ├── attributes.rs
│   │   ├── errors.rs
│   │   └── syntax.rs
│   ├── config/             # Configuration loading
│   │   ├── loading.rs
│   │   └── types.rs
│   ├── hkt/                # Higher-Kinded Type macros
│   │   ├── apply.rs
│   │   ├── impl_kind.rs
│   │   └── kind.rs
│   ├── hm_conversion/      # Hindley-Milner conversion
│   │   ├── ast.rs
│   │   ├── converter.rs
│   │   ├── patterns.rs
│   │   ├── transformations.rs
│   │   └── visitors/
│   ├── analysis/           # Generic/trait analysis
│   │   ├── bounds.rs
│   │   ├── generics.rs
│   │   └── traits.rs
│   ├── resolution/         # Self/type resolution
│   │   ├── context.rs
│   │   ├── errors.rs
│   │   ├── projection_key.rs
│   │   └── resolver.rs
│   ├── documentation/      # Doc generation
│   │   ├── hm_signature.rs
│   │   ├── doc_params.rs
│   │   ├── doc_type_params.rs
│   │   ├── document_module.rs
│   │   ├── generation.rs
│   │   └── templates.rs
│   └── re_export.rs        # Re-export generation (⚠️ PROBLEMATIC)
```

---

## Identified Issues

### Critical Issues (Must Fix)

#### 1. File I/O During Macro Expansion

**Severity**: 🔴 Critical  
**Location**: `re_export.rs:156-191`

**Problem**:
```rust
fn scan_directory_and_collect<F>(
    input: &ReexportInput,
    mut item_collector: F,
) -> Vec<TokenStream> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    let base_path = Path::new(&manifest_dir).join(input.path.value());
    
    if let Ok(entries) = fs::read_dir(&base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            // ... reads and parses files
        }
    } else {
        panic!("Failed to read directory: {:?}", base_path);
    }
}
```

**Issues**:
- **Non-reproducible builds**: Output changes if files change between compilations
- **Environment assumptions**: Requires `CARGO_MANIFEST_DIR` (fails in non-Cargo builds)
- **Violates proc macro principles**: Should operate on token streams only
- **Distributed build incompatibility**: Source files may not be locally available
- **Cargo change detection**: Changes don't trigger recompilation properly

**Impact**: High - affects build reproducibility, portability, and correctness

#### 2. Incomplete API Surface

**Severity**: 🟡 Medium  
**Location**: `documentation/hm_signature.rs:107`

**Problem**:
```rust
pub fn generate_signature(
    sig: &syn::Signature,
    _trait_context: Option<&str>,  // ← Reserved but never used
    config: &Config,
) -> SignatureData
```

**Issues**:
- Misleading API (suggests feature exists when it doesn't)
- Complicates function calls (must pass `None` explicitly)
- Technical debt marker in production code

**Impact**: Medium - confuses API consumers, indicates incomplete design

### High-Priority Issues (Should Fix)

#### 3. Code Duplication: Attribute Filtering

**Severity**: 🟡 Medium  
**Locations**: `hkt/impl_kind.rs:184-187`, likely others

**Problem**:
```rust
// Duplicated in multiple locations
let attrs = def.attrs.iter().filter(|attr| {
    !attr.path().is_ident(known_attrs::DOC_DEFAULT)
        && !attr.path().is_ident(known_attrs::DOC_USE)
});
```

**Issues**:
- Logic duplicated across codebase
- Inconsistent filtering if requirements change
- Harder to maintain and test

**Impact**: Medium - maintenance burden, potential for bugs

#### 4. Incomplete Error Context Implementation

**Severity**: 🟡 Medium  
**Location**: `error.rs:138-147`

**Problem**:
```rust
pub fn context(self, context: impl fmt::Display) -> Self {
    match self {
        Error::Internal(msg) => Error::Internal(...),
        Error::Validation { .. } => Error::Validation { .. },
        other => other,  // ← Other variants silently ignored!
    }
}
```

**Issues**:
- Incomplete error handling (some variants ignored)
- Inconsistent behavior across error types
- Silent failures in error enrichment

**Impact**: Medium - reduced debugging effectiveness

#### 5. Hard Panic in Library Code

**Severity**: 🟡 Medium  
**Location**: `re_export.rs:186`

**Problem**:
```rust
} else {
    panic!("Failed to read directory: {:?}", base_path);
}
```

**Issues**:
- Panics in proc macros are hard to debug
- No proper error message in IDE
- Violates library error handling principles

**Impact**: Medium - poor user experience

### Medium-Priority Issues (Nice to Fix)

#### 6. Configuration Loading Redundancy

**Severity**: 🟢 Low-Medium  
**Locations**: Multiple files

**Problem**:
```rust
// Called repeatedly throughout codebase
let config = load_config();
```

**Issues**:
- Multiple reads of same configuration file
- Potential for inconsistency
- Unnecessary I/O overhead

**Impact**: Low-Medium - potential for subtle bugs, inefficient

#### 7. Complex Display Implementation

**Severity**: 🟢 Low  
**Location**: `documentation/hm_signature.rs:59-96`

**Problem**: 96-line `Display` implementation with inline business logic

**Issues**:
- Hard to test individual formatting components
- Difficult to modify one aspect without affecting others
- Reduced maintainability

**Impact**: Low - affects maintainability only

#### 8. Blanket Dead Code Suppressions

**Severity**: 🟢 Low  
**Locations**: `hkt/impl_kind.rs:28,48`

**Problem**:
```rust
#[allow(dead_code)]
pub struct ImplKindInput {
    pub impl_generics: Generics,
    pub for_token: Token![for],
    pub brand: Type,
    pub brace_token: syn::token::Brace,  // Actually used by quote!
    pub definitions: Vec<KindAssocTypeImpl>,
}
```

**Issues**:
- Suppresses valid warnings
- Unclear which fields are actually unused
- Makes future refactoring harder

**Impact**: Low - documentation and maintenance concern

#### 9. Undocumented Magic Numbers

**Severity**: 🟢 Low  
**Location**: `hm_conversion/transformations.rs:276-277`

**Problem**:
```rust
const RAPID_SECRETS: rapidhash::v3::RapidSecrets =
    rapidhash::v3::RapidSecrets::seed(0x1234567890abcdef);
```

**Issues**:
- Arbitrary seed value without explanation
- Unclear why this specific value was chosen
- Future maintainers may change without understanding impact

**Impact**: Low - documentation only

---

## Proposed Architecture

### High-Level Design

The redesigned architecture maintains the current modular structure while addressing identified issues through:

1. **Build-Script-Based Re-exports**: Replace macro-time filesystem scanning with build-time automatic discovery
2. **Unified Configuration**: Single-point configuration loading with caching
3. **Centralized Utilities**: Extract common patterns into reusable components
4. **Complete Error Handling**: Full implementation of error context and propagation
5. **Cleaner API Surface**: Remove unused parameters, clarify intentions

### Module Structure (Redesigned)

```
fp-macros/
├── build.rs                      # NEW: Build script for re-export discovery
├── src/
│   ├── lib.rs                    # Main macro entry points (cleaned API)
│   ├── error.rs                  # Complete error types (using thiserror)
│   │
│   ├── core/                     # NEW: Core infrastructure
│   │   ├── mod.rs
│   │   ├── config.rs             # Cached config singleton
│   │   ├── attributes.rs         # Centralized attribute utilities
│   │   └── validation.rs         # Common validation logic
│   │
│   ├── hkt/                      # Higher-Kinded Type macros
│   │   ├── mod.rs
│   │   ├── apply.rs              # Apply! macro
│   │   ├── impl_kind.rs          # impl_kind! macro
│   │   ├── kind.rs               # Kind! and def_kind! macros
│   │   └── canonicalization.rs   # NEW: Extracted canonicalization
│   │
│   ├── hm_types/                 # Hindley-Milner type system
│   │   ├── mod.rs
│   │   ├── ast.rs                # HM type AST
│   │   ├── converter.rs          # Rust → HM conversion
│   │   ├── patterns.rs           # Pattern detection
│   │   └── formatting.rs         # NEW: Signature formatting
│   │
│   ├── analysis/                 # Type analysis
│   │   ├── mod.rs
│   │   ├── generics.rs           # Generic parameter analysis
│   │   ├── traits.rs             # Trait classification
│   │   └── bounds.rs             # Bound analysis
│   │
│   ├── resolution/               # Type resolution
│   │   ├── mod.rs
│   │   ├── context.rs            # Context extraction
│   │   ├── resolver.rs           # Self resolution
│   │   └── projection_map.rs     # Projection mapping
│   │
│   ├── documentation/            # Documentation generation
│   │   ├── mod.rs
│   │   ├── hm_signature.rs       # HM signature generation
│   │   ├── doc_params.rs         # Parameter docs
│   │   ├── doc_type_params.rs    # Type parameter docs
│   │   ├── document_module.rs    # Module orchestration
│   │   └── templates.rs          # Doc templates
│   │
│   └── re_export/                # REDESIGNED: Build-script-based re-exports
│       ├── mod.rs
│       ├── codegen.rs            # NEW: Code generation from discovered items
│       └── schema.rs             # NEW: Re-export schema
│
└── tools/                        # NEW: Build-time tools
    └── discover_exports.rs       # Module scanning for build script
```

### Data Flow

#### Current (Problematic)

```
generate_function_re_exports!
    ↓
Parse directory path from macro input
    ↓
Read CARGO_MANIFEST_DIR environment variable  ← PROBLEM: Macro-time env access
    ↓
Scan filesystem for .rs files  ← PROBLEM: Non-reproducible
    ↓
Parse each file with syn  ← PROBLEM: Expensive during compilation
    ↓
Extract public functions
    ↓
Generate pub use statements
```

#### Redesigned (Build-Script-Based with Automatic Discovery)

**Phase 1: Build Time (Reproducible)**
```
build.rs (runs before compilation)
    ↓
Scan src/classes/ directory for .rs files
    ↓
Parse each file with syn
    ↓
Extract public functions and traits
    ↓
Generate discovered_exports.rs in OUT_DIR
    ↓
    {
        "category": ["identity", "compose"],
        "functor": ["map", "fmap"],
        "monad": ["pure", "bind", "flat_map"]
    }
```

**Phase 2: Macro Expansion Time (Pure)**
```
generate_function_re_exports! {
    from: crate::classes,
    discover: include!(concat!(env!("OUT_DIR"), "/discovered_exports.rs")),
    aliases: {
        identity: category_identity,
        new: fn_new,
    }
}
    ↓
Parse discovered items (from build script)
    ↓
Parse alias configuration
    ↓
Apply aliases to discovered items
    ↓
Generate pub use statements
```

**Key Improvements:**
- ✅ **Automatic Discovery**: No need to list every function manually
- ✅ **Reproducible**: Build script output is deterministic
- ✅ **No Macro-Time I/O**: All filesystem access happens at build time
- ✅ **Cargo Integration**: Proper change detection via `cargo:rerun-if-changed`
- ✅ **Minimal Syntax**: Only aliases need to be specified

---

## Understanding Build-Time vs Macro-Time

### Conceptual Model

Rust compilation happens in distinct phases, and understanding when code runs is crucial for procedural macros:

```
Cargo Build Process
═══════════════════

1. Build Script Phase (build.rs)
   ├── Runs BEFORE any compilation
   ├── Has full filesystem access (intended design)
   ├── Output goes to OUT_DIR
   ├── Cargo tracks dependencies via cargo:rerun-if-changed
   └── Can fail build with clear error messages
   
2. Macro Expansion Phase
   ├── Runs DURING compilation
   ├── Processes token streams
   ├── Should NOT access filesystem (poor hygiene)
   ├── Can include!() files from build script output
   └── Errors appear as compiler errors
   
3. Code Generation Phase
   ├── Generates final binary
   ├── All macros already expanded
   └── No dynamic behavior
```

### Why Build-Time is Better for Discovery

**Build-Time (build.rs) ✅ Preferred**

```rust
// build.rs - Runs once before compilation
fn main() {
    println!("cargo:rerun-if-changed=src/classes");
    
    let exports = scan_directory("src/classes");  // ✅ OK: Build scripts should do I/O
    write_to_out_dir(exports);  // ✅ OK: Proper Cargo integration
}
```

**Advantages:**
1. **Proper Cargo Integration**: Cargo knows to rerun when dependencies change
2. **Clear Failure Mode**: Build script failures are distinct from compilation errors
3. **Reproducible**: Same input → same output (filesystem is stable before compilation)
4. **Cacheable**: Cargo caches build script output
5. **Parallelizable**: Build scripts run before proc macros, enabling better parallelism
6. **Standard Practice**: Many Rust projects use build scripts (bindgen, prost, etc.)

**Macro-Time (proc macro) ❌ Problematic**

```rust
// lib.rs - Runs during compilation for EVERY use of the macro
#[proc_macro]
pub fn generate_exports(input: TokenStream) -> TokenStream {
    let path = parse_path(input);
    let exports = scan_directory(path);  // ❌ BAD: Non-reproducible
    generate_code(exports)
}
```

**Problems:**
1. **Non-Reproducible**: Filesystem state might change between compilations
2. **Poor Change Tracking**: Cargo doesn't know about filesystem dependencies
3. **Performance**: Runs every time macro is invoked (could be multiple times)
4. **Hygiene Violation**: Proc macros should be pure functions on token streams
5. **Distributed Builds**: Filesystem might not exist in distributed build systems
6. **Hard to Debug**: Errors mixed with compilation errors

### Concrete Example

**Scenario**: You add a new function `foo()` to `src/classes/category.rs`

**With Build-Time Discovery (Correct):**
```
1. Edit src/classes/category.rs
2. Run cargo build
3. Build script detects change (cargo:rerun-if-changed)
4. Build script re-scans directory
5. Generates new discovered_exports.rs
6. Macro reads updated discovered_exports.rs
7. foo() is automatically re-exported
✅ Expected behavior
```

**With Macro-Time Discovery (Current - Broken):**
```
1. Edit src/classes/category.rs
2. Run cargo build
3. Cargo doesn't know file changed (no tracking)
4. Macro doesn't re-run (thinks nothing changed)
5. foo() is NOT re-exported
❌ Broken: Need cargo clean to fix
```

### Performance Comparison

| Aspect | Build-Time | Macro-Time |
|--------|-----------|------------|
| **When it runs** | Once before compilation | Every macro invocation |
| **Frequency** | Once per build (or when deps change) | 1-N times per build |
| **Caching** | Cargo caches output | No caching |
| **Parallelism** | Runs before macros | Blocks compilation |
| **Filesystem Access** | ~100ms (acceptable) | ~100ms × N invocations (unacceptable) |

**Example**: If macro is used 5 times in a project:
- Build-time: 100ms once = 100ms total
- Macro-time: 100ms × 5 = 500ms total + poor caching

### Trade-offs

**Build Script Approach:**

**Pros:**
- ✅ Reproducible builds
- ✅ Proper Cargo integration
- ✅ Better performance (cached)
- ✅ Standard Rust practice
- ✅ Clear error messages
- ✅ Automatic discovery works correctly

**Cons:**
- ⚠️ Requires build.rs (one-time setup)
- ⚠️ Slightly more complex (build + macro)
- ⚠️ Build script errors can be cryptic (but rare)

**Macro-Only Approach:**

**Pros:**
- ✅ No build script needed
- ✅ Simpler setup (single file)

**Cons:**
- ❌ Non-reproducible builds
- ❌ Poor change detection
- ❌ Violates proc macro hygiene
- ❌ Worse performance
- ❌ Incompatible with distributed builds
- ❌ Hard to debug

### Why Rust Community Prefers Build Scripts

From the Rust documentation and ecosystem:

1. **bindgen** (C bindings): Uses build script to scan C headers
2. **prost** (Protocol Buffers): Uses build script to generate Rust from .proto
3. **built** (build info): Uses build script to embed git info
4. **vergen** (version info): Uses build script to generate version constants

**Pattern**: When you need to discover/generate code from external sources, use build scripts.

**Rust's Philosophy**: "Explicit is better than implicit, reproducible is better than convenient."

### Decision Tree: Build Script vs Macro

```
Need to access filesystem?
├─ No → Use macro directly
└─ Yes →
   ├─ Fixed at compile-time? → Use build script
   │  └─ Examples: Discovered exports, generated bindings, embedded resources
   └─ Dynamic at runtime? → Use neither (load at runtime)
      └─ Examples: Config files, user data, plugins
```

### Mental Model

Think of the build process like cooking:

**Build Script (Prep Work):**
- Gather ingredients (scan filesystem)
- Prepare components (parse files)
- Store in staging area (OUT_DIR)
- Done once, before cooking starts

**Macro (Cooking):**
- Use prepared ingredients (include! from OUT_DIR)
- Combine according to recipe (token manipulation)
- No trips to store during cooking (no filesystem)
- Fast, repeatable, cacheable

**The Problem with Current Approach:**
- Like going to the store during cooking
- Every time you make the dish (every macro invocation)
- Inefficient and unpredictable
- Store might be closed/different each time

---

## Understanding Macro Hygiene

### What is Macro Hygiene?

**Macro hygiene** is a property of macro systems where macros behave as **referentially transparent** functions: given the same input, they always produce the same output, regardless of external context.

In Rust, hygiene has multiple dimensions:

#### 1. Name Hygiene (Traditional Definition)

Variables introduced by a macro don't interfere with user code:

```rust
// Hygienic macro
macro_rules! hygienic {
    ($x:expr) => {{
        let temp = $x;  // This 'temp' doesn't clash with user's 'temp'
        temp * 2
    }}
}

fn example() {
    let temp = 5;
    let result = hygienic!(3);  // Works! Internal 'temp' is distinct
    println!("{}", temp);  // Still 5, not affected by macro
}
```

#### 2. Referential Transparency (Broader Definition)

A macro should be a **pure function** on its inputs:

```rust
// Referentially transparent (hygienic)
#[proc_macro]
pub fn add(input: TokenStream) -> TokenStream {
    // Only depends on 'input', nothing else
    let n: i32 = parse(input);
    quote! { #n + 1 }.into()
}

// NOT referentially transparent (violates hygiene)
#[proc_macro]
pub fn add_with_time(input: TokenStream) -> TokenStream {
    let n: i32 = parse(input);
    let now = std::time::SystemTime::now();  // ❌ External state!
    quote! { #n + #now }.into()  // Different output each time
}
```

### Why Filesystem Access Violates Hygiene

When a procedural macro reads the filesystem, it violates referential transparency:

```rust
// VIOLATES HYGIENE
#[proc_macro]
pub fn discover_exports(input: TokenStream) -> TokenStream {
    let path = parse_path(input);
    
    // ❌ Output depends on filesystem state, not just input!
    let files = std::fs::read_dir(path).unwrap();
    
    // Same input can produce different output:
    // - Different on different machines
    // - Different after you add/remove files
    // - Different in CI vs local
    
    generate_exports(files)
}

// Usage
discover_exports!("src/classes")  // Output depends on what files exist!
```

**Problems:**
1. **Non-determinism**: Same code, different outputs
2. **Hidden dependencies**: Macro depends on files not mentioned in code
3. **Build caching breaks**: Cargo can't know when to recompile
4. **Reproducibility fails**: Different developers get different results

### Hygienic Alternative: Include Build-Generated Files

```rust
// HYGIENIC
#[proc_macro]
pub fn generate_exports(input: TokenStream) -> TokenStream {
    // ✅ Only reads from input token stream
    let manifest = parse_manifest(input);
    
    // ✅ Deterministic: same input → same output
    generate_re_exports(manifest)
}

// Usage - includes build-script-generated file
generate_exports! {
    from: crate::classes,
    // This file is generated by build.rs at build time
    discover: include!(concat!(env!("OUT_DIR"), "/discovered_exports.rs")),
}
```

**Why this is hygienic:**
1. `include!()` happens at **compile time** before macro expansion
2. Build script output is **stable** during compilation
3. Cargo **tracks** build script dependencies
4. Same input → same output (referentially transparent)

### The Three Levels of Hygiene

```
Level 0: No Hygiene
└─ C preprocessor, naive text substitution
   └─ Example: #define SQUARE(x) x*x → SQUARE(1+1) = 1+1*1+1 = 3 (wrong!)

Level 1: Name Hygiene
└─ Traditional macro hygiene (variable names don't clash)
   └─ Rust's macro_rules! has this
   └─ Example: Temporary variables in macros are isolated

Level 2: Referential Transparency (Full Hygiene)
└─ Output depends ONLY on input, no external state
   └─ Pure functions on token streams
   └─ What proc macros SHOULD be
   └─ Example: Same tokens in → same tokens out, always

Level 3: Compiler-Verified Hygiene
└─ Compiler enforces hygiene (theoretical, not in Rust yet)
   └─ Would prevent filesystem access in proc macros at compile time
```

Rust proc macros aim for **Level 2** but don't enforce it. It's a **best practice**, not a compiler guarantee.

### Real-World Consequences of Hygiene Violations

#### Case Study: Non-Hygienic Macro

```rust
// Bad: Reads config file during macro expansion
#[proc_macro_attribute]
pub fn configure(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // ❌ Violates hygiene
    let config = std::fs::read_to_string("config.toml").unwrap();
    let settings = parse_config(&config);
    
    modify_item_with_settings(item, settings)
}
```

**What goes wrong:**

1. **Alice's machine** (config.toml says `debug = true`):
   ```bash
   cargo build  # Generates debug code
   ```

2. **Bob's machine** (config.toml says `debug = false`):
   ```bash
   cargo build  # Generates different code!
   ```

3. **CI server** (no config.toml):
   ```bash
   cargo build  # Crashes! File not found
   ```

4. **Alice modifies config.toml**:
   ```bash
   cargo build  # Uses old cached macro output! Cargo doesn't know to recompile
   cargo clean && cargo build  # Now it updates (frustrating!)
   ```

#### Hygienic Alternative

```rust
// Good: Reads config at build time
// build.rs
fn main() {
    println!("cargo:rerun-if-changed=config.toml");
    let config = std::fs::read_to_string("config.toml").unwrap();
    let settings = parse_config(&config);
    
    // Write settings as Rust code
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(
        format!("{}/config.rs", out_dir),
        format!("const DEBUG: bool = {};", settings.debug)
    ).unwrap();
}

// Macro - hygienic!
#[proc_macro_attribute]
pub fn configure(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // ✅ Hygienic: include! reads at compile-time
    let settings = include!(concat!(env!("OUT_DIR"), "/config.rs"));
    
    modify_item_with_settings(item, settings)
}
```

**Now:**
- ✅ Same code on all machines
- ✅ Cargo knows when to rebuild
- ✅ Deterministic builds
- ✅ Cacheable

### Hygiene in the Redesign

**Current (Non-Hygienic):**
```rust
generate_function_re_exports!("src/classes", { ... })
    ↓
Macro reads filesystem directly
    ↓
❌ Non-deterministic
❌ Poor caching
❌ Hidden dependencies
```

**Redesigned (Hygienic):**
```rust
// build.rs - NOT a macro, hygiene doesn't apply here
fn main() {
    scan_and_generate_exports();  // OK to do I/O
}

// Macro - fully hygienic
generate_function_re_exports! {
    discover: include!(...),  // Compile-time constant
}
    ↓
Macro only processes token stream
    ↓
✅ Deterministic
✅ Cacheable
✅ Transparent dependencies
```

### Quick Reference: Hygiene Rules for Proc Macros

**DO:**
- ✅ Parse input tokens
- ✅ Generate output tokens
- ✅ Use `include!()` for external data
- ✅ Read `env!()` variables set by build script
- ✅ Perform deterministic transformations

**DON'T:**
- ❌ Read filesystem directly
- ❌ Access environment variables (except compiler-provided)
- ❌ Use random numbers
- ❌ Check system time
- ❌ Make network requests
- ❌ Access global mutable state

**IF YOU MUST ACCESS EXTERNAL DATA:**
- Use a build script (build.rs)
- Generate Rust code in OUT_DIR
- `include!()` that code in your macro

### Summary

**Macro Hygiene** = Macros should be **pure functions** on token streams.

**Why it matters:**
1. Reproducible builds (same code → same result)
2. Proper caching (Cargo knows when to rebuild)
3. Predictable behavior (no surprises)
4. Better debugging (errors are consistent)

**The redesign** achieves full hygiene by:
- Moving filesystem access to build script
- Using `include!()` to inject build-time data
- Keeping macros as pure token transformations

---

## Why Build Scripts CAN Access Filesystem But Macros SHOULDN'T

### The Critical Difference: Lifecycle and Guarantees

It's not that filesystem access is inherently bad. The issue is **when it happens** and **what guarantees exist**:

```
Cargo's Guarantee Model
═══════════════════════

Build Scripts:
├─ Run ONCE per build
├─ Run BEFORE any compilation
├─ Cargo TRACKS their dependencies
├─ Cargo CONTROLS their execution
├─ Output is STABLE during compilation
└─ Cargo can CACHE intelligently

Procedural Macros:
├─ Run DURING compilation
├─ May run MULTIPLE times
├─ Cargo DOESN'T track their dependencies
├─ Cargo CAN'T control their side effects
├─ Output can VARY during compilation
└─ Cargo CAN'T cache properly
```

### Detailed Comparison

| Aspect | Build Script | Procedural Macro |
|--------|--------------|------------------|
| **Runs when** | Before compilation starts | During compilation |
| **Runs how many times** | Once per build (or when deps change) | Once per macro invocation (1-N times) |
| **Cargo tracking** | Full (`cargo:rerun-if-changed`) | None (opaque to Cargo) |
| **Execution control** | Cargo runs it explicitly | Compiler plugin (internal) |
| **Filesystem stability** | Assumed stable before build | Could change mid-compilation! |
| **Caching** | Proper (OUT_DIR persists) | Limited (proc-macro2 cache only) |
| **Error handling** | Clear build script phase | Mixed with compilation |
| **Parallel builds** | Safe (runs serially before compilation) | Unsafe (could race if accessing FS) |
| **Distributed builds** | Compatible (runs on build server) | Incompatible (expects local FS) |

### The Real Problem: Cargo Can't Track Macro Dependencies

**Example: Why macros + filesystem = broken**

```rust
// my_macro/src/lib.rs
#[proc_macro]
pub fn load_config(_input: TokenStream) -> TokenStream {
    // ❌ Cargo doesn't know about this dependency!
    let config = std::fs::read_to_string("config.toml").unwrap();
    generate_code_from_config(config)
}

// user_crate/src/main.rs
fn main() {
    load_config!();  // Uses config.toml
}
```

**What happens:**

```bash
# Initial build
$ cargo build
   Compiling my_macro v0.1.0
   Compiling user_crate v0.1.0
    Finished dev [unoptimized + debuginfo] target(s)

# You edit config.toml
$ vim config.toml  # Change some values

# Rebuild
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.01s
# ❌ NOTHING REBUILT! Cargo didn't know config.toml matters!

# You have to manually force rebuild
$ cargo clean
$ cargo build  # Now it sees the changes
```

**With build script:**

```rust
// build.rs
fn main() {
    // ✅ Cargo DOES know about this dependency!
    println!("cargo:rerun-if-changed=config.toml");
    
    let config = std::fs::read_to_string("config.toml").unwrap();
    generate_and_save_to_out_dir(config);
}

// lib.rs
#[proc_macro]
pub fn load_config(_input: TokenStream) -> TokenStream {
    // ✅ This is compile-time constant from build script
    let config = include!(concat!(env!("OUT_DIR"), "/config.rs"));
    generate_code_from_config(config)
}
```

**Now:**

```bash
$ cargo build
   Compiling user_crate v0.1.0

$ vim config.toml  # Edit

$ cargo build
   Compiling user_crate v0.1.0  # ✅ Automatically rebuilds!
```

### The Parallel Build Problem

Macros can be invoked in parallel during compilation:

```rust
// Thread 1: Compiling module A
my_macro!("src/data");  // Reads filesystem

// Thread 2: Compiling module B (same time!)
my_macro!("src/data");  // Reads filesystem

// Thread 3: Compiling module C (same time!)
my_macro!("src/data");  // Reads filesystem
```

**Issues:**
1. **Race conditions**: What if filesystem changes between reads?
2. **Wasted work**: Three identical filesystem scans
3. **No coordination**: Macros can't communicate or share state
4. **Unpredictable order**: Which runs first? Nondeterministic!

**Build scripts run serially:**

```rust
// build.rs runs once, before any compilation
fn main() {
    scan_filesystem();  // Runs once, no races
}

// All macros use the same pre-computed result
macro_invocation_1!();  // Uses OUT_DIR
macro_invocation_2!();  // Uses same OUT_DIR
macro_invocation_3!();  // Uses same OUT_DIR
```

### The Caching Problem

**Cargo's caching assumes macros are pure:**

```
Cargo's Mental Model:
─────────────────────
Input tokens → [Macro (pure function)] → Output tokens

If input tokens haven't changed, output can't have changed.
Therefore: Can reuse cached output.

This breaks if macro reads filesystem:
Input tokens → [Macro + Filesystem] → Output tokens
                      ↑
                      Different each time!
                      But Cargo doesn't know!
```

**Real-world consequence:**

```bash
# Day 1: Build with file.txt containing "hello"
$ cargo build
   Compiling project v0.1.0

# Day 2: Edit file.txt to contain "goodbye"
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s)
# ❌ Still uses "hello"! Cached based on source code, not file.txt

# Force rebuild
$ touch src/lib.rs  # Trick Cargo into rebuilding
$ cargo build
   Compiling project v0.1.0
# ✅ Now sees "goodbye"
```

### The "Filesystem Stability" Contract

**Build scripts** run with a contract:

```
Cargo promises:
1. Filesystem is in a known state
2. No compilation has started yet
3. Your output goes to OUT_DIR
4. OUT_DIR is stable during compilation
5. Dependencies you declare will be tracked

You promise:
1. To declare dependencies (cargo:rerun-if-changed)
2. Not to modify source files
3. To be deterministic (same input → same output)
```

**Macros** have a different contract:

```
Cargo promises:
1. You get token streams as input
2. Your output is token streams
3. You run during compilation

You promise:
1. To be a pure function (input → output)
2. Not to have side effects
3. Not to depend on external state
```

**Filesystem access breaks the macro contract** but **fits the build script contract**.

### Why Not Just Let Cargo Track Macro Filesystem Access?

**Technical reasons this is hard:**

1. **Macro execution is opaque**: Cargo doesn't control proc macros directly (they're compiler plugins)
2. **Sandboxing is difficult**: Would need syscall interception or WASM sandbox
3. **Performance**: Tracking every syscall from macros would be expensive
4. **Backward compatibility**: Tons of existing macros would break
5. **Complex dependency graph**: Macros can invoke other macros with different dependencies

**It's easier and cleaner to:**
- Use build scripts for I/O (already tracked)
- Keep macros pure (already expected)

### Analogy: Database Transactions vs Queries

Think of it like databases:

**Build Script = Transaction (Write Phase)**
- Runs once
- Can modify state (write to OUT_DIR)
- Fully tracked and logged
- All or nothing (fails clearly)
- Isolated from concurrent operations

**Macro = Read-Only Query**
- Runs many times
- Should only read (token streams)
- Fast because it's pure
- Cacheable because deterministic
- Parallelizable because no side effects

**Reading filesystem in macro = Query that modifies database**
- Violates read-only assumption
- Breaks caching
- Breaks parallelism
- Breaks reproducibility

### The Bottom Line

It's not arbitrary. The distinction exists because:

1. **Build scripts have proper lifecycle management** - Cargo runs them explicitly
2. **Build scripts have dependency tracking** - You declare what they depend on
3. **Build scripts run once** - No redundant work or races
4. **Macros have none of these** - They're opaque compiler plugins

**Filesystem access is fine when properly managed. Build scripts provide that management. Macros don't.**

The redesign moves filesystem access to where it belongs: under Cargo's control, with proper tracking, at the right time in the build lifecycle.

---

## Core Design Principles

### Principle 1: Macro Purity

**Statement**: Procedural macros MUST operate solely on token streams without external dependencies.

**Rationale**:
- Ensures build reproducibility
- Enables distributed builds
- Matches Rust's macro hygiene principles
- Simplifies testing and debugging

**Implications**:
- No filesystem access during macro expansion
- No environment variable dependencies (except compiler-provided)
- All inputs must come through macro invocation

### Principle 2: Explicit Over Implicit

**Statement**: Configuration and behavior MUST be explicit rather than inferred from environment.

**Rationale**:
- Reduces surprises and hidden dependencies
- Makes code easier to understand and maintain
- Enables better error messages
- Simplifies testing

**Implications**:
- Re-export targets must be explicitly declared
- Configuration should be visible in code
- Defaults should be clearly documented

### Principle 3: Complete Error Handling

**Statement**: Error handling infrastructure MUST be complete and consistent across all error types.

**Rationale**:
- Provides consistent user experience
- Enables effective debugging
- Prevents silent failures
- Matches Rust's exhaustive matching philosophy

**Implications**:
- All error methods must handle all variants
- Error messages must include context and suggestions
- Panics are only acceptable for truly unrecoverable situations

### Principle 4: Single Responsibility

**Statement**: Each module and function MUST have a single, well-defined responsibility.

**Rationale**:
- Improves testability
- Reduces coupling
- Enables code reuse
- Simplifies understanding

**Implications**:
- Extract common patterns into utilities
- Avoid mixing concerns (e.g., parsing + I/O)
- Clear module boundaries

### Principle 5: Zero-Cost Abstractions

**Statement**: Abstractions MUST NOT impose runtime overhead.

**Rationale**:
- Macros execute at compile time
- Generated code must be optimal
- No unnecessary allocations or indirection

**Implications**:
- Configuration loading should be cached
- Generated code should be minimal
- Avoid complex runtime checks

---

## Key Design Decisions

### Decision 1: Re-export Manifest Format

**Chosen**: Inline declaration syntax (Option B)

**Rationale**:
1. **Self-contained**: All information in one place
2. **Type-safe**: Validated by Rust parser
3. **Version control friendly**: Changes visible in diffs
4. **No external files**: Simpler build process
5. **Better error messages**: Span information available

**Implementation**:

```rust
// Current (problematic):
generate_function_re_exports!("src/classes", {
    identity: category_identity,
    new: fn_new,
});

// Redesigned:
generate_function_re_exports! {
    from: crate::classes,
    functions: {
        category: {
            identity as category_identity,
            compose,
        },
        functor: {
            map,
            fmap,
        },
        monad: {
            pure,
            bind,
            flat_map as then,
        },
    }
}
```

**Trade-offs**:
- ➕ Explicit, self-documenting
- ➕ No filesystem dependencies
- ➕ Easy to review changes
- ➖ More verbose than automatic discovery
- ➖ Must be manually maintained

**Migration Path**: Build script can generate initial manifest from existing code.

### Decision 2: Configuration Loading Strategy

**Chosen**: Thread-local cached singleton

**Rationale**:
1. **Single load**: Configuration read once per thread
2. **Zero runtime cost**: After first load, just pointer access
3. **Thread-safe**: Each thread has own copy
4. **Lazy**: Only loaded when needed
5. **Testable**: Can inject test configuration

**Implementation**:

```rust
// core/config.rs

use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static CONFIG: RefCell<Option<Rc<Config>>> = RefCell::new(None);
}

pub fn get_config() -> Rc<Config> {
    CONFIG.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            *opt = Some(Rc::new(Config::load_from_cargo_toml()));
        }
        opt.as_ref().unwrap().clone()
    })
}

#[cfg(test)]
pub fn set_test_config(config: Config) {
    CONFIG.with(|cell| {
        *cell.borrow_mut() = Some(Rc::new(config));
    });
}
```

### Decision 3: Attribute Filtering Centralization

**Chosen**: Common utility module with filter iterators

**Implementation**:

```rust
// core/attributes.rs

pub struct DocAttributeFilter;

impl DocAttributeFilter {
    /// Returns true if the attribute should be kept in generated code
    pub fn should_keep(attr: &Attribute) -> bool {
        !Self::is_doc_specific(attr)
    }
    
    /// Returns true if the attribute is documentation-specific
    pub fn is_doc_specific(attr: &Attribute) -> bool {
        attr.path().is_ident("doc_default")
            || attr.path().is_ident("doc_use")
    }
    
    /// Filters out documentation-specific attributes
    pub fn filter_doc_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
        attrs.iter().filter(|attr| Self::should_keep(attr))
    }
}
```

### Decision 4: Error Context Completion

**Chosen**: Implement full error context for all variants

**Implementation**:

```rust
// error.rs

impl Error {
    pub fn context(self, context: impl fmt::Display) -> Self {
        match self {
            Error::Internal(msg) => Error::Internal(
                format!("{}: {}", context, msg)
            ),
            Error::Validation { message, span } => Error::Validation {
                message: format!("{}: {}", context, message),
                span,
            },
            Error::Resolution { message, span, available_types } => {
                Error::Resolution {
                    message: format!("{}: {}", context, message),
                    span,
                    available_types,
                }
            }
            Error::Parse(e) => {
                // Create new error with context and combine
                let ctx_error = syn::Error::new(
                    e.span(),
                    format!("{}: {}", context, e)
                );
                Error::Parse(ctx_error)
            }
            Error::Unsupported(u) => {
                // Unsupported features maintain original message
                // but we note the context
                Error::Internal(format!(
                    "{}: Unsupported feature: {}",
                    context, u
                ))
            }
            Error::Io(io) => Error::Internal(
                format!("{}: I/O error: {}", context, io)
            ),
        }
    }
}
```

### Decision 5: Signature Formatting Refactoring

**Chosen**: Extract formatting concerns into dedicated module

**Implementation**:

```rust
// hm_types/formatting.rs

pub struct SignatureFormatter {
    config: Rc<Config>,
}

impl SignatureFormatter {
    pub fn new(config: Rc<Config>) -> Self {
        Self { config }
    }
    
    pub fn format(&self, data: &SignatureData) -> String {
        let parts: Vec<String> = [
            self.format_forall(data),
            self.format_constraints(data),
            Some(self.format_function_signature(data)),
        ]
        .into_iter()
        .flatten()
        .collect();
        
        parts.join(" ")
    }
    
    fn format_forall(&self, data: &SignatureData) -> Option<String> {
        if data.forall.is_empty() {
            None
        } else {
            Some(format!("forall {}.", data.forall.join(" ")))
        }
    }
    
    fn format_constraints(&self, data: &SignatureData) -> Option<String> {
        if data.constraints.is_empty() {
            return None;
        }
        
        let constraint_str = if data.constraints.len() == 1 {
            data.constraints[0].clone()
        } else {
            format!("({})", data.constraints.join(", "))
        };
        
        Some(format!("{} =>", constraint_str))
    }
    
    fn format_function_signature(&self, data: &SignatureData) -> String {
        let func_type = if data.params.is_empty() {
            HMType::Arrow(
                Box::new(HMType::Unit),
                Box::new(data.return_type.clone())
            )
        } else {
            let input_type = if data.params.len() == 1 {
                data.params[0].clone()
            } else {
                HMType::Tuple(data.params.clone())
            };
            HMType::Arrow(
                Box::new(input_type),
                Box::new(data.return_type.clone())
            )
        };
        
        format!("{}", func_type)
    }
}

impl std::fmt::Display for SignatureData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use default config for Display
        let formatter = SignatureFormatter::new(get_config());
        write!(f, "{}", formatter.format(self))
    }
}
```

### Decision 6: Remove Unused Trait Context Parameter

**Chosen**: Remove parameter, document removal in migration guide

**Rationale**:
- Parameter was never implemented
- No current users depend on it (always `None`)
- Simplifies API
- Can be re-added if actually needed

**Implementation**:

```rust
// Before:
pub fn generate_signature(
    sig: &syn::Signature,
    _trait_context: Option<&str>,
    config: &Config,
) -> SignatureData

// After:
pub fn generate_signature(
    sig: &syn::Signature,
    config: &Config,
) -> SignatureData

// Note: If trait context is needed in future:
pub fn generate_signature_with_context(
    sig: &syn::Signature,
    context: &TraitContext,  // Strongly typed, not just string
    config: &Config,
) -> SignatureData
```

### Decision 7: Document Magic Constants

**Chosen**: Add comprehensive documentation to all constants

**Implementation**:

```rust
// hkt/canonicalization.rs

/// Fixed seed for deterministic hash generation.
///
/// This seed value MUST remain constant across all versions to ensure that
/// Kind trait names are stable between compilations and across different
/// machines. Changing this value will break all existing Kind implementations.
///
/// The specific value `0x1234567890abcdef` was chosen arbitrarily but serves
/// to distinguish our hashes from other hash functions that might use
/// default seeds (typically 0 or random values).
const RAPID_SECRETS: rapidhash::v3::RapidSecrets =
    rapidhash::v3::RapidSecrets::seed(0x1234567890abcdef);
```

### Decision 8: Replace Panics with Compile Errors

**Chosen**: Generate `compile_error!` macros instead of panicking

**Implementation**:

```rust
// re_export/generator.rs

// Before:
} else {
    panic!("Failed to read directory: {:?}", base_path);
}

// After:
} else {
    return quote! {
        compile_error!(
            concat!(
                "Re-export generation failed: ",
                "Could not find module declarations for path '",
                stringify!(#path),
                "'. ",
                "Please ensure the manifest is correctly defined."
            )
        );
    };
}
```

---

## Issue Resolution Mapping

This section maps each identified issue to the design decisions that address it.

| Issue # | Issue Description | Resolution | Design Decision |
|---------|-------------------|------------|-----------------|
| 1 | File I/O during macro expansion | Manifest-based re-exports | Decision 1 |
| 2 | Unused trait context parameter | Remove parameter | Decision 6 |
| 3 | Duplicated attribute filtering | Centralized utilities | Decision 3 |
| 4 | Incomplete error context | Full error handling | Decision 4 |
| 5 | Hard panic in library code | Compile error generation | Decision 8 |
| 6 | Redundant config loading | Cached singleton | Decision 2 |
| 7 | Complex Display implementation | Extract formatting module | Decision 5 |
| 8 | Blanket dead code suppressions | Targeted suppressions with docs | Per-field annotations |
| 9 | Undocumented magic numbers | Comprehensive documentation | Decision 7 |

### Additional Benefits

The redesign also provides several additional benefits not directly tied to specific issues:

1. **Improved Testability**: Centralized utilities are easier to test in isolation
2. **Better Documentation**: Clear separation of concerns makes documentation more focused
3. **Easier Onboarding**: New contributors can understand the architecture more quickly
4. **Future-Proofing**: Cleaner abstractions make future enhancements easier
5. **Performance**: Cached configuration reduces redundant I/O

---

## Migration Strategy

### Phase 1: Infrastructure Preparation

**Goal**: Establish new infrastructure without breaking existing code

**Tasks**:

1. **Create Core Module Structure**
   ```rust
   // src/core/mod.rs
   pub mod config;
   pub mod attributes;
   pub mod validation;
   ```

2. **Implement Configuration Caching**
   - Add `core/config.rs` with thread-local singleton
   - Add tests for configuration loading
   - Keep existing `load_config()` function for backward compatibility

3. **Implement Attribute Utilities**
   - Add `core/attributes.rs` with `DocAttributeFilter`
   - Add comprehensive tests
   - Document usage patterns

4. **Complete Error Context Implementation**
   - Update `error.rs` with full context support
   - Add tests for all error variants
   - Ensure backward compatibility

5. **Extract Formatting Logic**
   - Create `hm_types/formatting.rs`
   - Implement `SignatureFormatter`
   - Update `SignatureData::Display` to use formatter
   - Add tests

**Validation**: All existing tests pass without modification

### Phase 2: Re-export Redesign

**Goal**: Replace filesystem-based re-exports with manifest system

**Tasks**:

1. **Design Manifest Schema**
   ```rust
   // re_export/schema.rs
   
   pub struct ReexportManifest {
       from: syn::Path,
       functions: HashMap<String, Vec<FunctionExport>>,
   }
   
   pub struct FunctionExport {
       name: String,
       alias: Option<String>,
   }
   ```

2. **Implement Manifest Parser**
   - Add `re_export/manifest.rs`
   - Parse inline declaration syntax
   - Validate module and function names
   - Add comprehensive tests

3. **Create Migration Tool**
   - Build script that scans existing code
   - Generates manifest declarations
   - Outputs replacement macro invocations
   - Add documentation for manual review

4. **Update Re-export Macros**
   - Deprecate filesystem-based macros
   - Implement manifest-based versions
   - Maintain both during migration period
   - Add migration guide

5. **Migrate fp-library**
   - Run migration tool on `fp-library` crate
   - Review generated manifests
   - Update macro invocations
   - Verify all re-exports work correctly

**Validation**: 
- All re-exports generate identical code
- Build is reproducible across machines
- No filesystem access during compilation

### Phase 3: API Cleanup

**Goal**: Remove deprecated features and clean up APIs

**Tasks**:

1. **Remove Unused Parameters**
   - Update `generate_signature()` to remove `_trait_context`
   - Update all call sites
   - Document removal in changelog

2. **Replace Direct Config Loading**
   - Update all `load_config()` calls to `get_config()`
   - Remove redundant configuration loading
   - Verify performance improvement

3. **Update Attribute Handling**
   - Replace inline filtering with `DocAttributeFilter`
   - Remove duplicated logic
   - Add tests for all filtering scenarios

4. **Replace Panics with Compile Errors**
   - Update error handling in all macros
   - Generate `compile_error!` where appropriate
   - Improve error messages

5. **Document Magic Constants**
   - Add comprehensive documentation to all constants
   - Explain why specific values were chosen
   - Note immutability requirements

**Validation**: All tests pass, no panics in normal operation

### Phase 4: Documentation and Finalization

**Goal**: Complete documentation and prepare for release

**Tasks**:

1. **Update Rustdoc**
   - Document new modules
   - Update examples
   - Add migration guide
   - Document design decisions

2. **Update CHANGELOG**
   - Document breaking changes
   - List new features
   - Provide migration examples
   - Note deprecated features

3. **Add Migration Guide**
   - Step-by-step instructions
   - Before/after examples
   - Common pitfalls
   - FAQ section

4. **Performance Testing**
   - Benchmark configuration loading
   - Verify no regression in macro expansion
   - Document performance characteristics

5. **Release Preparation**
   - Bump version to 2.0.0 (breaking changes)
   - Update dependencies
   - Final test suite run
   - Prepare release notes

**Validation**: Documentation is complete and accurate

### Phase 5: Deprecation and Cleanup

**Goal**: Remove deprecated code after migration period

**Tasks**:

1. **Remove Filesystem-Based Re-exports**
   - Delete old implementation
   - Remove deprecated macros
   - Update documentation

2. **Clean Up Test Code**
   - Remove backward compatibility tests
   - Simplify test infrastructure
   - Consolidate test utilities

3. **Final Optimization**
   - Profile macro expansion
   - Optimize hot paths
   - Reduce allocations

4. **Audit and Review**
   - Security audit
   - Code review
   - Performance review
   - Documentation review

**Validation**: All deprecated code removed, no regressions

### Migration Tooling

**Build Script for Manifest Generation**:

```rust
// tools/generate_manifests.rs

use std::fs;
use std::path::Path;
use syn::{Item, ItemFn, parse_file};

fn main() {
    let src_dir = Path::new("fp-library/src/classes");
    
    for entry in fs::read_dir(src_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = fs::read_to_string(&path).unwrap();
            let file = parse_file(&content).unwrap();
            
            let functions: Vec<_> = file.items.iter()
                .filter_map(|item| match item {
                    Item::Fn(func) if matches!(func.vis, syn::Visibility::Public(_)) => {
                        Some(func.sig.ident.to_string())
                    }
                    _ => None
                })
                .collect();
            
            if !functions.is_empty() {
                let module_name = path.file_stem().unwrap().to_str().unwrap();
                println!("        {}: {{", module_name);
                for func in functions {
                    println!("            {},", func);
                }
                println!("        }},");
            }
        }
    }
}
```

**Usage**:
```bash
cd tools
cargo run --bin generate_manifests
# Copy output and manually review/adjust
```

### Breaking Changes Summary

**API Changes**:

1. **Re-export Macros** (Major)
   - Old: `generate_function_re_exports!("path", {aliases})`
   - New: `generate_function_re_exports! { from: path, functions: {...} }`
   - Migration: Run generation tool, manual review

2. **Signature Generation** (Minor)
   - Old: `generate_signature(sig, None, config)`
   - New: `generate_signature(sig, config)`
   - Migration: Remove second parameter from all calls

3. **Configuration Access** (Minor - internal)
   - Old: `let config = load_config();`
   - New: `let config = get_config();`
   - Migration: Automatic (if using public API)

**Behavior Changes**:

1. **Error Handling**
   - Panics replaced with compile errors
   - Better error messages with context
   - No migration needed (improvement)

2. **Build Reproducibility**
   - Builds are now fully reproducible
   - No dependency on filesystem during compilation
   - No migration needed (improvement)

**Timeline**: No specific deadlines, but suggested order of phases

---

## Success Criteria

### Functional Criteria

1. ✅ **All Existing Tests Pass**: 100% of current test suite passes without modification
2. ✅ **fp-library Builds Successfully**: Main consumer compiles and runs correctly
3. ✅ **Documentation Quality Maintained**: Generated docs are at least as good as current
4. ✅ **Feature Parity**: All current functionality preserved

### Quality Criteria

5. ✅ **No Filesystem Access**: Zero filesystem operations during macro expansion
6. ✅ **Reproducible Builds**: Identical output for identical input across machines
7. ✅ **Complete Error Handling**: All error types fully implemented
8. ✅ **Zero Panics**: No panics in normal operation (only compile errors)

### Performance Criteria

9. ✅ **No Regression**: Macro expansion time not significantly slower
10. ✅ **Reduced Overhead**: Configuration loaded once per thread
11. ✅ **Optimized Code**: Generated code is minimal and efficient

### Documentation Criteria

12. ✅ **Comprehensive Rustdoc**: All public APIs documented with examples
13. ✅ **Migration Guide**: Clear instructions for upgrading
14. ✅ **Design Documentation**: Architecture decisions documented
15. ✅ **Examples Updated**: All examples use new APIs

### Maintainability Criteria

16. ✅ **Code Duplication Eliminated**: Common patterns extracted
17. ✅ **Clear Module Boundaries**: Single responsibility per module
18. ✅ **Testable Components**: All components can be tested in isolation
19. ✅ **Consistent Style**: Uniform coding patterns throughout

### Verification Methods

**Automated Testing**:
```bash
# Run full test suite
cargo test --all-features

# Run integration tests
cargo test --test '*'

# Run doctests
cargo test --doc

# Check examples
cargo run --example '*'
```

**Manual Verification**:
```bash
# Build fp-library
cd fp-library && cargo build

# Check documentation
cargo doc --open

# Verify reproducibility
cargo clean && cargo build
cargo clean && cargo build
# Compare outputs
```

**Performance Testing**:
```bash
# Benchmark macro expansion
cargo expand > before.rs  # Before redesign
cargo expand > after.rs   # After redesign
diff before.rs after.rs   # Should be minimal

# Time compilation
time cargo build --release
```

---

## Appendix A: API Examples

### Before and After: Re-exports

**Before**:
```rust
// fp-library/src/classes.rs

#[macro_use]
extern crate fp_macros;

generate_function_re_exports!("src/classes", {
    identity: category_identity,
    new: fn_new,
});
```

**After**:
```rust
// fp-library/src/classes.rs

#[macro_use]
extern crate fp_macros;

generate_function_re_exports! {
    from: crate::classes,
    functions: {
        category: {
            identity as category_identity,
            compose,
        },
        functor: {
            map,
            fmap,
        },
        monad: {
            pure,
            bind,
            flat_map,
        },
    }
}
```

### Before and After: Signature Generation

**Before**:
```rust
use fp_macros::documentation::generate_signature;

let sig = parse_signature("fn map<F, A, B>(f: F, fa: A) -> B");
let config = load_config();
let signature = generate_signature(&sig, None, &config);  // Unused None
```

**After**:
```rust
use fp_macros::documentation::generate_signature;
use fp_macros::core::get_config;

let sig = parse_signature("fn map<F, A, B>(f: F, fa: A) -> B");
let config = get_config();  // Cached
let signature = generate_signature(&sig, &config);  // Cleaner
```

### Before and After: Error Handling

**Before**:
```rust
// In re_export.rs
if let Err(_) = fs::read_dir(&path) {
    panic!("Failed to read directory: {:?}", path);  // Hard to debug
}
```

**After**:
```rust
// In re_export.rs
if manifest.functions.is_empty() {
    return quote! {
        compile_error!(concat!(
            "No functions found in re-export manifest for module '",
            stringify!(#module),
            "'. ",
            "Please ensure the manifest is correctly defined."
        ));
    };
}
```

---

## Appendix B: Testing Strategy

### Unit Tests

Each module maintains comprehensive unit tests:

```rust
// core/attributes.rs

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    
    #[test]
    fn test_filter_doc_default() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[doc_default]),
            parse_quote!(#[derive(Debug)]),
        ];
        
        let filtered: Vec<_> = DocAttributeFilter::filter_doc_attrs(&attrs)
            .collect();
        
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].path().is_ident("derive"));
    }
    
    #[test]
    fn test_is_doc_specific() {
        let doc_default: Attribute = parse_quote!(#[doc_default]);
        let doc_use: Attribute = parse_quote!(#[doc_use = "Of"]);
        let derive: Attribute = parse_quote!(#[derive(Debug)]);
        
        assert!(DocAttributeFilter::is_doc_specific(&doc_default));
        assert!(DocAttributeFilter::is_doc_specific(&doc_use));
        assert!(!DocAttributeFilter::is_doc_specific(&derive));
    }
}
```

### Integration Tests

Test complete workflows:

```rust
// tests/re_export_integration.rs

#[test]
fn test_function_re_exports_generates_correct_code() {
    let input = quote! {
        generate_function_re_exports! {
            from: crate::test_module,
            functions: {
                category: {
                    identity,
                    compose as then,
                },
            }
        }
    };
    
    let output = generate_function_re_exports(input);
    
    let expected = quote! {
        pub use crate::test_module::category::identity;
        pub use crate::test_module::category::compose as then;
    };
    
    assert_eq!(output.to_string(), expected.to_string());
}
```

### Property Tests

Verify invariants:

```rust
// tests/canonicalization_properties.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_canonicalization_determinism(
        name in "[a-zA-Z][a-zA-Z0-9]*",
        bound in "[a-zA-Z][a-zA-Z0-9]*",
    ) {
        let input1 = format!("type {}< T: {} >;", name, bound);
        let input2 = format!("type {}< T: {} >;", name, bound);
        
        let hash1 = generate_name(&parse_kind_input(&input1)).unwrap();
        let hash2 = generate_name(&parse_kind_input(&input2)).unwrap();
        
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_parameter_renaming_invariance(
        name1 in "[a-zA-Z][a-zA-Z0-9]*",
        name2 in "[a-zA-Z][a-zA-Z0-9]*",
    ) {
        prop_assume!(name1 != name2);
        
        let input1 = format!("type Of< {} >;", name1);
        let input2 = format!("type Of< {} >;", name2);
        
        let hash1 = generate_name(&parse_kind_input(&input1)).unwrap();
        let hash2 = generate_name(&parse_kind_input(&input2)).unwrap();
        
        // Same structure -> same hash
        assert_eq!(hash1, hash2);
    }
}
```

### Regression Tests

Ensure no breakage:

```rust
// tests/regression.rs

/// Ensure Kind! macro still works with original syntax
#[test]
fn test_kind_macro_backward_compat() {
    let name = Kind!(type Of<T>;);
    assert!(stringify!(name).starts_with("Kind_"));
}

/// Ensure def_kind! generates expected trait
#[test]
fn test_def_kind_output_format() {
    let output = def_kind!(type Of<'a, T: 'a>: 'a;);
    let output_str = output.to_string();
    
    assert!(output_str.contains("pub trait Kind_"));
    assert!(output_str.contains("type Of"));
}
```

---

## Appendix C: Performance Benchmarks

### Configuration Loading

```rust
// benches/config.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fp_macros::core::get_config;

fn bench_config_loading(c: &mut Criterion) {
    c.bench_function("first config load", |b| {
        b.iter(|| {
            // Clear cache (test-only function)
            clear_config_cache();
            black_box(get_config())
        })
    });
    
    c.bench_function("cached config load", |b| {
        // Prime cache
        let _ = get_config();
        
        b.iter(|| black_box(get_config()))
    });
}

criterion_group!(benches, bench_config_loading);
criterion_main!(benches);
```

**Expected Results**:
- First load: ~100μs (reads Cargo.toml)
- Cached load: ~1ns (pointer copy)

### Macro Expansion

```rust
// benches/expansion.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_kind_generation(c: &mut Criterion) {
    let input = parse_quote!(type Of<'a, T: 'a>: 'a;);
    
    c.bench_function("generate Kind name", |b| {
        b.iter(|| black_box(generate_name(&input)))
    });
}

fn bench_signature_generation(c: &mut Criterion) {
    let sig = parse_quote! {
        fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B>
    };
    let config = get_config();
    
    c.bench_function("generate HM signature", |b| {
        b.iter(|| black_box(generate_signature(&sig, &config)))
    });
}

criterion_group!(benches, bench_kind_generation, bench_signature_generation);
criterion_main!(benches);
```

**Expected Results**:
- Kind name generation: ~10μs
- HM signature generation: ~50μs
- Total macro expansion: < 1ms per invocation

---

## Appendix D: Migration Checklist

### Pre-Migration

- [ ] Review this specification thoroughly
- [ ] Set up test environment
- [ ] Back up current codebase
- [ ] Create feature branch
- [ ] Communicate changes to team/users

### Phase 1: Infrastructure

- [ ] Create `core/` module structure
- [ ] Implement configuration caching
- [ ] Add attribute utilities
- [ ] Complete error context implementation
- [ ] Extract formatting logic
- [ ] Run existing test suite
- [ ] Review and merge

### Phase 2: Re-export Redesign

- [ ] Create fp-macros-build crate
- [ ] Implement export discovery in build library
- [ ] Add build.rs to fp-library
- [ ] Test automatic discovery
- [ ] Update re-export macros to use build-generated data
- [ ] Migrate fp-library
- [ ] Verify all re-exports match previous output
- [ ] Add new tests for build script
- [ ] Review and merge

### Phase 3: API Cleanup

- [ ] Remove unused parameters
- [ ] Update config loading calls
- [ ] Replace attribute filtering
- [ ] Replace panics with compile errors
- [ ] Document magic constants
- [ ] Run full test suite
- [ ] Review and merge

### Phase 4: Documentation

- [ ] Update rustdoc
- [ ] Write migration guide
- [ ] Update CHANGELOG
- [ ] Add examples
- [ ] Performance testing
- [ ] Review documentation
- [ ] Prepare release notes

### Phase 5: Release

- [ ] Bump version to 2.0.0
- [ ] Final test run
- [ ] Publish to crates.io
- [ ] Tag release
- [ ] Update dependents
- [ ] Monitor for issues

### Post-Migration

- [ ] Remove deprecated code (after grace period)
- [ ] Clean up test infrastructure
- [ ] Final optimization pass
- [ ] Security audit
- [ ] Document lessons learned

---

## Appendix E: Build Script Integration Details

### Directory Structure

```
fp-library/
├── Cargo.toml
├── build.rs                    # NEW: Export discovery
├── src/
│   ├── lib.rs
│   ├── classes.rs              # Uses discovered exports
│   └── classes/
│       ├── mod.rs
│       ├── category.rs         # Public functions automatically discovered
│       ├── functor.rs          # Public functions automatically discovered
│       └── monad.rs            # Public functions automatically discovered
└── target/
    └── ...

fp-macros-build/                # NEW: Build-time library
├── Cargo.toml
├── src/
│   └── lib.rs                  # Export discovery implementation
└── tests/
    └── discovery_tests.rs
```

### Complete Build Script Example

```rust
// fp-library/build.rs

use std::path::Path;

fn main() {
    // Configure Cargo to re-run if src/classes changes
    println!("cargo:rerun-if-changed=src/classes");
    
    // Discover all public exports
    let discovery = fp_macros_build::discover_exports_in_directory(
        "src/classes",
        fp_macros_build::DiscoveryOptions {
            include_functions: true,
            include_traits: true,
            include_types: false,
            nested_modules: false,
        }
    ).expect("Failed to discover exports in src/classes");
    
    // Write discovered exports to OUT_DIR
    let out_dir = std::env::var("OUT_DIR")
        .expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("discovered_exports.rs");
    
    let code = discovery.to_token_stream().to_string();
    std::fs::write(&dest_path, code)
        .expect("Failed to write discovered_exports.rs");
    
    // Optional: Print summary for debugging
    eprintln!("Discovered {} modules with exports", discovery.modules.len());
}
```

### Generated Output Example

```rust
// target/debug/build/fp-library-*/out/discovered_exports.rs
// Auto-generated by build.rs - DO NOT EDIT

ExportDiscovery {
    modules: vec![
        ("applicative", vec!["ap", "apply", "lift2", "lift3", "product"]),
        ("category", vec!["compose", "chain", "identity"]),
        ("foldable", vec!["fold", "fold_left", "fold_right", "fold_map"]),
        ("functor", vec!["map", "fmap", "map_const", "void"]),
        ("monad", vec!["pure", "bind", "flat_map", "join", "then"]),
        ("monoid", vec!["empty", "concat", "concat_all"]),
        ("semigroup", vec!["combine", "combine_n"]),
    ]
}
```

### Macro Usage with Discovery

```rust
// fp-library/src/classes.rs

// Re-export all discovered functions
generate_function_re_exports! {
    from: crate::classes,
    discover: include!(concat!(env!("OUT_DIR"), "/discovered_exports.rs")),
    aliases: {
        // Only need to specify items that need renaming
        identity: category_identity,
        new: fn_new,
    }
}

// Expands to:
// pub use crate::classes::applicative::ap;
// pub use crate::classes::applicative::apply;
// pub use crate::classes::applicative::lift2;
// ... (all items from all modules)
// pub use crate::classes::category::identity as category_identity;
// ... etc
```

### Advantages Over Current Approach

| Aspect | Current (Problematic) | Redesigned (Build Script) |
|--------|----------------------|---------------------------|
| **Discovery** | Manual listing required | Automatic discovery |
| **I/O Timing** | During macro expansion | During build script (before compilation) |
| **Reproducibility** | Non-deterministic | Fully deterministic |
| **Cargo Integration** | No proper tracking | Full `rerun-if-changed` support |
| **Aliases** | Required for all items | Only for renamed items |
| **Maintenance** | Update macro when adding functions | Zero maintenance |
| **Distributed Builds** | Incompatible | Compatible |
| **Error Detection** | Late (macro expansion) | Early (build script) |

### Error Handling in Build Script

```rust
// fp-macros-build/src/lib.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Failed to read directory: {path}")]
    DirectoryRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse file {path}: {message}")]
    FileParse {
        path: String,
        message: String,
        #[source]
        source: syn::Error,
    },

    #[error("No public items found in directory: {path}")]
    NoItems {
        path: String,
    },

    #[error("Invalid module name: {name}")]
    InvalidModuleName {
        name: String,
    },
}

pub type Result<T> = std::result::Result<T, DiscoveryError>;
```

### Testing Build Script Behavior

```rust
// fp-macros-build/tests/discovery_tests.rs

#[test]
fn test_discover_functions() {
    let temp_dir = create_test_directory();
    
    // Create test file with public functions
    std::fs::write(
        temp_dir.path().join("test_module.rs"),
        r#"
            pub fn public_fn() {}
            fn private_fn() {}
            pub fn another_public() {}
        "#
    ).unwrap();
    
    let discovery = discover_exports_in_directory(
        temp_dir.path(),
        DiscoveryOptions::functions_only()
    ).unwrap();
    
    assert_eq!(discovery.modules.len(), 1);
    assert_eq!(discovery.modules[0].0, "test_module");
    assert_eq!(discovery.modules[0].1, vec!["another_public", "public_fn"]);
}

#[test]
fn test_ignore_private_items() {
    let temp_dir = create_test_directory();
    
    std::fs::write(
        temp_dir.path().join("test.rs"),
        r#"
            pub fn public_fn() {}
            fn private_fn() {}
            pub(crate) fn crate_fn() {}
        "#
    ).unwrap();
    
    let discovery = discover_exports_in_directory(
        temp_dir.path(),
        DiscoveryOptions::functions_only()
    ).unwrap();
    
    // Only truly public items
    assert_eq!(discovery.modules[0].1, vec!["public_fn"]);
}
```

---

## Appendix F: Comparison with Alternative Approaches

### Alternative 1: Keep Current Approach (Rejected)

**Approach**: Continue using filesystem access during macro expansion

**Pros**:
- No changes needed
- Familiar to current users

**Cons**:
- ❌ Non-reproducible builds
- ❌ Violates proc macro principles
- ❌ Cargo doesn't track dependencies properly
- ❌ Incompatible with distributed builds
- ❌ Still requires manual configuration

**Verdict**: Not acceptable - fundamental issues must be fixed

### Alternative 2: Fully Manual Specification (Rejected)

**Approach**: Require users to list every function explicitly

```rust
generate_function_re_exports! {
    from: crate::classes,
    functions: {
        category: ["identity", "compose", "chain"],
        functor: ["map", "fmap", "map_const"],
        // ... must list everything
    }
}
```

**Pros**:
- ✅ Explicit control
- ✅ No filesystem access
- ✅ Reproducible

**Cons**:
- ❌ Extremely verbose
- ❌ High maintenance burden
- ❌ Easy to forget items
- ❌ No better than manual `pub use`

**Verdict**: Too burdensome - defeats purpose of macro

### Alternative 3: Runtime Discovery (Rejected)

**Approach**: Generate code that discovers exports at runtime

```rust
// Would generate:
pub use discover_at_runtime!("src/classes");
```

**Pros**:
- ✅ Automatic discovery
- ✅ No build script needed

**Cons**:
- ❌ Runtime overhead
- ❌ Complex implementation
- ❌ Can't work (exports must be compile-time)
- ❌ Violates Rust's compile-time resolution

**Verdict**: Technically impossible

### Alternative 4: Macro-Generated Build Script (Considered)

**Approach**: Have macro generate a build script

**Pros**:
- ✅ Automatic discovery
- ✅ No manual build.rs needed

**Cons**:
- ❌ Complex bootstrap process
- ❌ Macro writes to filesystem (still problematic)
- ❌ Hard to debug
- ❌ Confusing for users

**Verdict**: Too complex, doesn't solve core issues

### Alternative 5: Build Script (Chosen) ✅

**Approach**: Use build script to discover and generate manifest

**Pros**:
- ✅ Automatic discovery
- ✅ Reproducible (build-time I/O is fine)
- ✅ Proper Cargo integration
- ✅ Minimal configuration
- ✅ Standard Rust pattern
- ✅ Easy to debug
- ✅ Compatible with all build systems

**Cons**:
- ⚠️ Requires build.rs (acceptable)
- ⚠️ Slightly more setup (one-time)

**Verdict**: Best solution - solves all problems with minimal trade-offs

---

## Appendix G: FAQ

### Q: Why not just use manual `pub use` statements?

**A**: The macro provides several benefits:
1. **Automatic discovery**: Don't need to remember to add new items
2. **Consistent patterns**: All re-exports follow same structure
3. **Namespace management**: Automatic alias conflict detection
4. **Bulk operations**: Can re-export entire modules at once
5. **Documentation**: Centralized list of public API

### Q: Isn't adding a build script too complicated?

**A**: Not really:
1. It's a one-time setup (less than 10 lines)
2. Build scripts are standard Rust practice
3. The fp-macros-build library handles all complexity
4. Benefits far outweigh the small initial cost
5. Many Rust projects already use build scripts

### Q: What happens if I don't use build.rs?

**A**: You can still use the old syntax with aliases only, but you lose automatic discovery. The macro will emit a helpful compile error suggesting you add a build script.

### Q: Can I control which items are discovered?

**A**: Yes, via `DiscoveryOptions`:
```rust
DiscoveryOptions {
    include_functions: true,
    include_traits: true,
    include_types: false,
    nested_modules: false,
}
```

### Q: What if I want to exclude specific items?

**A**: Use the `exclude` option:
```rust
generate_function_re_exports! {
    from: crate::classes,
    discover: include!(...),
    exclude: ["internal_helper", "deprecated_fn"],
}
```

### Q: Does this work with trait re-exports too?

**A**: Yes! Same pattern:
```rust
generate_trait_re_exports! {
    from: crate::classes,
    discover: include!(concat!(env!("OUT_DIR"), "/discovered_exports.rs")),
}
```

### Q: How do I debug what's being discovered?

**A**: The build script prints a summary:
```bash
cargo build -vv
# Shows: "Discovered 7 modules with exports"
```

You can also check `target/debug/build/*/out/discovered_exports.rs`

### Q: Will this break existing code?

**A**: The migration maintains identical output. After verifying, you can remove manual listings.

### Q: What about performance?

**A**: Build scripts run once (or when files change), so there's no runtime cost. Macro expansion is actually faster because it doesn't need to scan the filesystem.

### Q: Can I use this in no_std environments?

**A**: Yes! The build script uses std, but generated code is no_std compatible.
- [ ] Performance testing
- [ ] Review documentation
- [ ] Prepare release notes

### Phase 5: Release

- [ ] Bump version to 2.0.0
- [ ] Final test run
- [ ] Publish to crates.io
- [ ] Tag release
- [ ] Update dependents
- [ ] Monitor for issues

### Post-Migration

- [ ] Remove deprecated code (after grace period)
- [ ] Clean up test infrastructure
- [ ] Final optimization pass
- [ ] Security audit
- [ ] Document lessons learned

---

## Conclusion

This specification provides a comprehensive plan for redesigning the `fp-macros` crate to address all identified issues while maintaining backward compatibility where possible. The redesign focuses on:

1. **Build Reproducibility**: Eliminating filesystem dependencies
2. **API Clarity**: Removing unused parameters and clarifying interfaces
3. **Code Quality**: Reducing duplication and improving maintainability
4. **Error Handling**: Completing error handling infrastructure

The migration strategy is designed to be incremental, with each phase building on the previous one. Breaking changes are documented and justified, with migration tools provided where appropriate.

The success criteria ensure that the redesign meets all functional, quality, performance, and documentation requirements before being considered complete.

---

**Document Status**: Ready for Review  
**Next Steps**: Review with team, gather feedback, begin Phase 1 implementation
