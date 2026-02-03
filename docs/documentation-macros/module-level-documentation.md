# Module-Level Documentation Macro Specification

## Motivation

### Problem Statement

The current `#[document_impl]` macro significantly improves documentation generation for Higher-Kinded Type (HKT) implementations. However, it introduces two new sources of friction:

1.  **Repetitive Annotation**: Every `impl` block in a file must be individually annotated with `#[document_impl]`. For types implementing many type classes (like `CatList`), this results in dozens of repeated annotations.
2.  **Configuration Duplication**: The mapping between "Brand" types (e.g., `CatListBrand`) and "Concrete" types (e.g., `CatList`) currently lives in `fp-library/Cargo.toml`. This duplicates information already present in the `impl_kind!` macro invocation within the source code, creating a risk of desynchronization.

### Solution

A new module-level procedural macro, `#[document_module]`, that:

- Is applied once to the module (e.g., as `#![document_module]`).
- Automatically extracts Brand-to-Concrete type mappings from both `impl_kind!` invocations and standard `impl` blocks (Impl-Scanning).
- Automatically applies the `document_impl` logic to all `impl` blocks, using a hierarchical configuration system to resolve ambiguities.

## Functional Requirements

### 1. Context Extraction (Full Projection & Impl-Scanning)

**Requirement**: The macro must extract a comprehensive mapping of `(Brand, AssociatedType, Generics) -> TargetType` from two sources. This extra detail is required to perform correct parameter substitution during resolution (e.g., mapping `Self::Of<X, Y>` to `Result<X, Y>` requires knowing that `Of<A, B> = Result<A, B>`).

1.  **`impl_kind!` Invocations**:

    ```rust
    impl_kind! { for MyBrand { type Of<A> = Box<A>; } }
    ```

    Extracts: `MyBrand::Of<A>` -> `Box<A>`.

2.  **Standard `impl` Blocks (Impl-Scanning)**:
    ```rust
    impl Pointer for ArcBrand {
        type Of<T> = Arc<T>;
    }
    ```
    Extracts: `ArcBrand::Of<T>` -> `Arc<T>`.

The macro aggregates these findings into a module-wide configuration.

### 2. Hierarchical Configuration

**Requirement**: To resolve the concrete type of `Self` (when used bare, e.g., `fn foo(self)`), the macro must support a precedence hierarchy:

1.  **Method Override**: `#[doc_primary = "AssocName"]` on the method.
2.  **Impl Block Override**: `#[doc_primary = "AssocName"]` on the `impl` block.
3.  **Global Default**: `#[primary]` (or `#[doc_primary]`) on the associated type definition in `impl_kind!` (or `impl` block).
4.  **Implicit Default**: The associated type named `Of`.
5.  **Fallback**: Error or Refuse Resolution (Do not resolve `Self`).

    **Rationale**: Relying on "the first associated type defined" is fragile, as source order can change during refactoring. If no explicit default is marked and no associated type is named `Of`, the macro should error or refuse to resolve `Self` (leaving it as is) rather than guessing. Silent defaults lead to confusing documentation.

### 3. Automatic Documentation Application

**Requirement**: For every `impl` block found:

- Resolve `Self` usage:
  - **Path/Projected** (`Self::Assoc`): Map using the Context Extraction table (e.g., `Self::SendOf` -> `Arc`).
  - **Bare** (`self`, `Self`): Map using the Hierarchical Default (e.g., `self` -> `Box`).
- Generate HM signatures and parameter docs respecting these resolutions.

## Design Decisions

### 1. Syntax: `#[doc_primary]`

**Decision**: Use `#[doc_primary = "Name"]` (attribute with value) or `#[doc_primary]` (marker) to control defaults.

- **Inside `impl_kind!` / `impl`**: `#[doc_primary]` (marker) on a `type` definition sets the global default for that Brand.
- **On `impl` / `fn`**: `#[doc_primary = "AssocName"]` overrides the default to use the target of `AssocName`.

**Rationale**:

- **Consistency**: Using a unified attribute name for both marking the default definition and overriding the selection reduces cognitive load.
- **Clarity**: Explicitly naming the associated type (`AssocName`) ensures that the override points to a valid, defined type mapping, rather than an arbitrary string, preventing typo-induced errors.
- **Granularity**: Allowing overrides at both the `impl` block and method level handles edge cases where different methods on the same Brand conceptually operate on different concrete representations (e.g., `Box` vs `Rc` contexts).

### 2. Macro Type: Attribute Macro

**Decision**: `#[proc_macro_attribute]` applied to the module (`mod` item).

**Rationale**:

- **Full Visibility**: Attribute macros on modules receive the entire module content (including `impl_kind!`, `impl` blocks, and manual trait impls) as a single token stream. This allows a single-pass extraction of context before processing items, which is impossible with per-item macros.
- **Zero Boilerplate**: Users only need to add one line (`#![document_module]`) per file, achieving the primary goal of reducing annotation noise.

### 3. Logic Reuse

**Decision**: Refactor `fp-macros/src/document_impl.rs` and `hm_signature.rs` to accept a rich `Config` object containing the projection map and defaults, rather than just string mappings.

**Rationale**:

- **Maintainability**: The core logic for generating HM signatures (analyzing generics, formatting bounds) is complex. Duplicating it for the new macro would lead to bugs and drift.
- **Consistency**: Refactoring the existing logic ensures that the legacy `#[document_impl]` macro (which uses `Cargo.toml`) and the new `#[document_module]` macro (which uses extracted context) behave exactly the same way regarding signature formatting.

### 4. Impl-Scanning Strategy

**Decision**: Extract associated type mappings from _any_ `impl` block in the module, not just `impl_kind!`.

**Rationale**:

- **Robustness**: This supports types like `ArcBrand` in `fp-library/src/types/arc_ptr.rs` that manually implement traits (like `Pointer`) without using the `impl_kind!` macro.
- **Zero Config**: It allows the system to work "out of the box" for manual implementations without requiring a parallel "declaration" macro just for documentation metadata.
- **Completeness**: It captures all associated types (`CloneableOf`, `SendOf`) defined in standard Rust `impl` blocks, ensuring the "Full Projection" map is complete.

## Implementation Plan

1.  **Refactor Config**: Update `Config` struct to support associated type mappings and defaults.
2.  **Refactor Core Logic**: Update `document_impl` and `hm_signature` to use the new Config for resolution (Projection vs Default).
3.  **Implement `document_module`**:
    - Parse `impl_kind!` and scan `impl` blocks to build the Config.
    - Traverse module items.
    - For each `impl`, check attributes for overrides.
    - For each method, check attributes for overrides.
    - Invoke generation logic.
4.  **Update `lib.rs`**: Export `document_module`.
