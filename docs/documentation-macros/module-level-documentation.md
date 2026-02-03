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

**Generics Handling**:
The extraction logic must use **Positional Mapping** to map generic parameters correctly.

- **Parse Definition**: `type Of<A, B>` -> Parameters `[A, B]` (Indices: A=0, B=1).
- **Parse Target**: `Result<B, A>`.
- **Build Map**: Map usage based on indices. Target becomes `Result<$1, $0>`.
- **Apply**: When encountering `Self::Of<X, Y>`, substitute `X` (index 0) and `Y` (index 1) into the target structure -> `Result<Y, X>`.

This ensures that parameter swapping (e.g., `type Of<A, B> = Result<B, A>`), duplication, or omission is handled correctly.

**Where Clause Handling**:
The extraction logic must robustly parse `where` clauses on associated types (e.g., `type Of<A> = Foo<A> where A: Clone;`) but **ignore** them for the purpose of building the projection map. The `where` clause is relevant for code validity but not for the structural mapping required for documentation.

**Cfg Handling**:
The macro should **ignore** `cfg` attributes during extraction (i.e., extract everything). This ensures that documentation is complete regardless of the compiler's active feature flags, preventing broken links or missing docs for optional features.

### 2. Hierarchical Configuration

**Requirement**: To resolve the concrete type of `Self` (when used bare, e.g., `fn foo(self)`), the macro must support a precedence hierarchy:

1.  **Method Override**: `#[doc_primary = "AssocName"]` on the method.
2.  **Impl Block Override**: `#[doc_primary = "AssocName"]` on the `impl` block.
3.  **Module Default**: `#[primary]` (or `#[doc_primary]`) on the associated type definition in `impl_kind!` (or `impl` block).
4.  **Fallback**: Error.

    **Rationale**: Relying on implicit defaults (like "the associated type named `Of`") is fragile and "magic". If there are ambiguities (multiple associated types) and no explicit default is marked, the macro should error and force the user to disambiguate. Explicit is better than implicit.

    **Conflict Resolution**: If multiple associated types for the same Brand are marked as `#[doc_primary]` within the same module, the macro will emit a compile-time error. This ambiguity must be resolved by the user.

### 3. Automatic Documentation Application

**Requirement**: For every `impl` block found:

- Resolve `Self` usage:
  - **Path/Projected** (`Self::Assoc`): Map using the Context Extraction table (e.g., `Self::SendOf` -> `Arc`).
  - **Bare** (`self`, `Self`): Map using the Hierarchical Default (e.g., `self` -> `Box`).
    - **Brand Impls**: For `impl` blocks on Brands (e.g., `impl Kind for VecBrand`), the macro must **actively substitute** the `Self` type (which is `VecBrand`) with the concrete type constructor (e.g., `Vec`) found in the mapping. This ensures documentation shows `Vec<A>` instead of `VecBrand`.
    - **Concrete Impls**: For `impl` blocks on concrete types (e.g., `impl<A> Vec<A>`), `Self` is already concrete and is used as-is.
- Generate HM signatures and parameter docs respecting these resolutions.

## Design Decisions

### 1. Update `impl_kind!` Parser

**Decision**: Modify the `impl_kind!` macro parser to accept and parse attributes (like `#[doc_primary]`) on associated type definitions.

**Rationale**:

- **Enabling Configuration**: The current `impl_kind!` parser does not support attributes on associated types. To support "Module Default" configuration via `#[doc_primary]` inside `impl_kind!`, the parser must be updated.
- **Standard Compliance**: This aligns `impl_kind!` syntax closer to standard Rust `impl` blocks, where attributes on associated types are valid.
- **Implementation**: The macro will parse these attributes to make them available for `document_module` extraction, but will strip them from the generated output to avoid "unused attribute" warnings.

### 2. Syntax: `#[doc_primary]`

**Decision**: Use `#[doc_primary = "Name"]` (attribute with value) or `#[doc_primary]` (marker) to control defaults.

- **Inside `impl_kind!` / `impl`**: `#[doc_primary]` (marker) on a `type` definition sets the module default for that Brand.
- **On `impl` / `fn`**: `#[doc_primary = "AssocName"]` overrides the default to use the target of `AssocName`.

**Rationale**:

- **Consistency**: Using a unified attribute name for both marking the default definition and overriding the selection reduces cognitive load.
- **Clarity**: Explicitly naming the associated type (`AssocName`) ensures that the override points to a valid, defined type mapping, rather than an arbitrary string, preventing typo-induced errors.
- **Granularity**: Allowing overrides at both the `impl` block and method level handles edge cases where different methods on the same Brand conceptually operate on different concrete representations (e.g., `Box` vs `Rc` contexts).

### 3. Macro Type: Attribute Macro

**Decision**: `#[proc_macro_attribute]` applied to the module (`mod` item).

**Rationale**:

- **Full Visibility**: Attribute macros on modules receive the entire module content (including `impl_kind!`, `impl` blocks, and manual trait impls) as a single token stream. This allows a single-pass extraction of context before processing items, which is impossible with per-item macros.
- **Zero Boilerplate**: Users only need to add one line (`#![document_module]`) per file, achieving the primary goal of reducing annotation noise.

### 4. Logic Reuse & Deprecation

**Decision**: Refactor `fp-macros/src/document_impl.rs` and `hm_signature.rs` to accept a rich `Config` object containing the projection map and defaults. **Remove** the standalone `#[document_impl]` macro.

**Rationale**:

- **Maintainability**: The core logic for generating HM signatures (analyzing generics, formatting bounds) is complex. Duplicating it for the new macro would lead to bugs and drift.
- **Simplification**: The `#[document_module]` macro subsumes the functionality of `#[document_impl]`. Removing the standalone macro simplifies the codebase and removes the need to maintain `Cargo.toml` configuration.

### 5. Impl-Scanning Strategy

**Decision**: Extract associated type mappings from _any_ top-level `impl` block in the module, not just `impl_kind!`. The macro will **not** recursively scan inside function bodies or nested modules.

**Rationale**:

- **Robustness**: This supports types like `ArcBrand` in `fp-library/src/types/arc_ptr.rs` that manually implement traits (like `Pointer`) without using the `impl_kind!` macro.
- **Zero Config**: It allows the system to work "out of the box" for manual implementations without requiring a parallel "declaration" macro just for documentation metadata.
- **Completeness**: It captures all associated types (`CloneableOf`, `SendOf`) defined in standard Rust `impl` blocks, ensuring the "Full Projection" map is complete.

### 6. Attribute Placement

**Decision**: Enforce usage as an **inner attribute** (`#![document_module]`) at the top of the file.

**Rationale**:

- **Access to Content**: Inner attributes guarantee that the macro receives the file's content as its input. Outer attributes on `mod foo;` declarations might not have access to the content if it resides in a separate file (unless explicitly loaded, which is complex and non-standard for proc macros).

### 7. Macro Expansion Order

**Decision**: Explicitly document that the macro **cannot** see `impl` blocks generated by other macros (except `impl_kind!`).

**Rationale**:

- **Technical Limitation**: Attribute macros run before the expansion of macros inside them. `document_module` will see the macro invocation (e.g., `my_macro! { ... }`), not the generated code.
- **User Action**: Users must manually annotate macro-generated `impl` blocks if they exist, or the generating macro must be updated to support documentation generation directly.

### 8. Coupling to `impl_kind!`

**Decision**: Share the parsing logic (the `ImplKindInput` struct in `fp-macros/src/impl_kind.rs`) between both macros.

**Rationale**:

- **Maintenance**: `document_module` must parse `impl_kind!` to extract context. Sharing the parser ensures that if `impl_kind!` syntax changes, `document_module` stays in sync automatically, preventing breakage.

### 9. Error Handling Strategy

**Decision**: Use **Hard Errors** (compile errors) for any resolution failures or configuration ambiguities.

**Rationale**:

- **Quality Assurance**: Since this is a documentation tool, "silently broken" docs (e.g., missing signatures or incorrect types) are worse than no docs. Failing the build forces the user to fix the configuration, ensuring documentation accuracy.

## Limitations

### 1. Cross-Module Visibility

The `#[document_module]` macro can only inspect the tokens within the module it is applied to. If a Brand is defined (via `impl_kind!`) in one module but implemented (via `impl`) in another, applying `#[document_module]` to the implementation module will fail to resolve the Brand mappings because it cannot see the definition.

**Mitigation**: Enforce the co-location rule strictly. If the macro encounters a Brand that it cannot resolve (because the definition is in another module), it will:

1.  **Check for Manual Overrides**: Look for `#[doc_primary = "..."]` on the `impl` block.
2.  **Fallback Error**: If no mapping and no override are found, emit a compile-time error instructing the user to co-locate the definition or ensure a local `impl` block defines the mapping.

### 2. Macro Expansion Order

Attribute macros run before the expansion of macros inside them. `#[document_module]` will not see `impl` blocks generated by other macros.

**Mitigation**: Explicitly document this limitation. Code generated by other macros will not be automatically documented by this system.

## Implementation Plan

1.  **Refactor `impl_kind!` Parser**: Update `fp-macros/src/impl_kind.rs` to parse attributes on associated type definitions.
2.  **Refactor Config**: Update `Config` struct to support associated type mappings (using positional mapping) and defaults.
3.  **Refactor Core Logic**: Update `document_impl` and `hm_signature` to use the new Config for resolution (Projection vs Default) and explicit `Self` substitution.
4.  **Implement `document_module`**:
    - Parse `impl_kind!` and scan `impl` blocks to build the Config.
    - Traverse module items.
    - For each `impl`, check attributes for overrides.
    - For each method, check attributes for overrides.
    - Invoke generation logic.
5.  **Update `lib.rs`**: Export `document_module`.
