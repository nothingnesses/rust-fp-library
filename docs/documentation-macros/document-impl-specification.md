# `document_impl` Macro Specification

## Motivation

### Problem Statement

Writing `impl` blocks for Higher-Kinded Types (HKT) involves significant boilerplate for documentation:

1.  **Redundant Trait Names**: Methods inside `impl Trait for Type` must currently be annotated with `#[hm_signature(Trait)]` to correctly generate the `Trait self` constraint. The user must manually repeat the trait name for every method.
2.  **Missing Context**: The current [`hm_signature`](../../fp-macros/src/hm_signature.rs) macro only analyzes the function's AST. It ignores bounds defined on the `impl` block (e.g., `impl<T: Clone> ...`), leading to inaccurate signatures like `forall t. t -> t` instead of `forall t. Clone t => t -> t`.
3.  **Inconsistent Documentation**: Documenting generic parameters requires manually adding [`#[doc_type_params]`](../../fp-macros/src/doc_type_params.rs) to every method, often duplicating documentation for trait-level parameters (like `T` in `impl<T>`).

### Solution

A new procedural macro, `#[document_impl]`, applied to the `impl` block itself. This macro acts as an orchestrator that:

- Parses the full `impl` context (trait name, generics, bounds).
- Automatically generates accurate HM signatures for all methods, including `impl` bounds.
- Distributes trait-level parameter documentation to all methods.

## Functional Requirements

### 1. Trait Name Inference

**Requirement**: Automatically infer the trait name from the `impl` signature for use in HM signatures.
**Input**: `impl Functor for CatListBrand`
**Effect**: Methods behave as if `#[hm_signature(Functor)]` was applied, but with better context.

### 2. Accurate Bound Capture

**Requirement**: Capture generic bounds defined on the `impl` block and include them in the generated HM signature.
**Input**:

```rust
impl<T: Clone> MyTrait for MyType<T> {
    fn foo(x: T) -> T
}
```

**Output Signature**: `forall t. Clone t => t -> t`
(Current behavior without this macro would represent `T` as unbounded).

### 3. Trait Parameter Documentation

**Requirement**: Allow documenting trait-level generic parameters once at the `impl` level, and automatically include them in the documentation for every method. This includes **all** parameters (lifetimes, types, consts).
**Syntax**: Use **Positional Syntax** (Consistent with existing macros).

```rust
#[document_impl(
    // Positional mapping to impl generics
    doc_type_params("The element type")
)]
impl<T> ...
```

### 4. Method Parameter Documentation

**Requirement**: Support standard `#[doc_type_params]` on methods for method-level generics, composing correctly with the trait-level documentation.

## Design Decisions

### 1. Direct Expansion with Marker Placement

**Decision**: The `document_impl` macro will **not** delegate to `#[hm_signature]` by adding attributes. Instead, it will look for the `#[hm_signature]` attribute on methods and **replace it** with the generated documentation.

**Rationale**:

- `hm_signature` attribute syntax does not support passing arbitrary `where` clauses or complex bounds easily.
- By performing the expansion directly, `document_impl` can construct a "synthetic" function signature in memory that:
  1.  Merges the `impl` generics with the method generics.
  2.  Substitutes `Self` types with the concrete `impl` type (e.g., `CatList<A>`).
  3.  Injects the trait implementation itself as a bound (e.g., `Semigroup (CatList A)`).
- This synthetic signature is then passed to the shared core logic of `hm_signature`, guaranteeing that the generated string includes all constraints.

> **Note on Shadowing**: Rust forbids shadowing generic parameters within the same item (error `E0403`). Therefore, the implementation does not need to handle name collisions between `impl` and method generics during the merge process.

### 2. Shared Core Logic and Simplification

**Decision**:

- Refactor [`hm_signature`](../../fp-macros/src/hm_signature.rs) and [`doc_type_params`](../../fp-macros/src/doc_type_params.rs) to expose their core logic as reusable functions.
- Modify `hm_signature` to **no longer accept** a trait name argument.

**Rationale**:

- **Reuse**: Avoids code duplication and ensures consistency.
- **Simplification**: Since all HKT trait impls must use `document_impl` (which infers the trait name), the manual argument is obsolete. Standalone usage remains supported (without arguments).
- **Fallback**: If used on a trait method without `document_impl`, it gracefully falls back to a signature without the trait constraint.

### 3. Positional Syntax for Documentation

**Decision**: Maintain the existing **Positional Syntax** for `doc_type_params`. **Mandatory documentation** is enforced.

**Syntax Rules**:

- Arguments are a list of strings or tuples.
- Arguments match generic parameters by position (index).
- `"Description"`: Use the generic's actual name for display.
- `("OverrideName", "Description")`: Use the override name for display.

**Constraints**:

- **No Skipping**: Documentation is mandatory for all generic parameters (types, lifetimes, consts). Empty descriptions or skipping parameters is not supported.

**Rationale**:

- **Consistency**: Matches existing `doc_type_params` usage throughout the codebase.
- **Simplicity**: Standard Rust literals/tuples are easy to parse and read.
- **Explicit**: Unambiguous separation between data (description) and metadata (display name).
- **Completeness**: Enforces full documentation coverage for the library.

### 4. Documentation Ordering and Opt-in

**Decision**: Trait-level parameter documentation will be inserted **only if** the `#[doc_type_params]` attribute is present on the method, and it will be inserted **before** it.

**Rationale**:

- **Opt-in**: Prevents documentation clutter on helper methods that don't explicitly request type parameter docs.
- **Ordering**: Trait parameters (context) should appear before method parameters (specifics).
- **Expansion**: The `document_impl` macro will inject `#[doc = "..."]` lines for the trait parameters immediately before the `#[doc_type_params]` attribute. The existing `doc_type_params` attribute remains (or is expanded by the standalone macro) to generate method docs.

### 5. Out of Scope: Associated Type Resolution

**Decision**: Associated types (e.g., `Iterator::Item`) will NOT be resolved to their concrete types in the HM signature. They will appear as `Type::Assoc` (e.g., `CatList::Item`).

**Rationale**:

- **Complexity**: Resolving associated types requires full type inference or deeper analysis of the `impl` block's associated type definitions, which is significant additional complexity.
- **Acceptability**: Displaying the associated type path is sufficient for documentation purposes in the initial version.

## Implementation Strategy

### Phase 1: Refactoring

1.  Extract `generate_signature` logic from [`hm_signature.rs`](../../fp-macros/src/hm_signature.rs) into a public/internal API that accepts a `syn::Signature` (or equivalent) and context.
2.  Refactor [`doc_utils.rs`](../../fp-macros/src/doc_utils.rs) to allow generic parameter matching logic to be called on `ItemImpl` generics (or arbitrary generic lists).

### Phase 2: `document_impl` Parser

1.  Implement `document_impl` to parse `syn::ItemImpl`.
2.  Extract `trait_` path (for the name).
3.  Extract `generics` (params and where clause) from the `impl`.
4.  Parse `doc_type_params` argument from `document_impl` attribute.

### Phase 3: Expansion Logic

1.  Iterate over `impl.items`.
2.  For each `ImplItem::Fn`:
    - **HM Signature**:
      - Look for `#[hm_signature]` attribute.
      - If found:
        - Clone the method signature.
        - **Substitute `Self`**: Recursively replace all occurrences of `Self` (return type, arguments, bounds) with the concrete `impl` type (e.g., `CatList<A>`).
        - **Synthesize Trait Bound**: Create a bound `ConcreteType: Trait` and add it to the signature's `where` clause.
        - **Merge Generics**: Merge `impl` generics into the cloned signature's generics, ensuring lifetime parameters precede type parameters.
        - Call the shared `generate_signature` function (passing `None` for trait name).
        - **Replace** the `#[hm_signature]` attribute with the generated `#[doc = "..."]`.
    - **Doc Params**:
      - Look for `#[doc_type_params]` attribute.
      - If found:
        - Match parsed `doc_type_params` arguments (from `document_impl`) against `impl` generics by position.
        - Generate `#[doc = "..."]` lines.
        - Insert generated docs **before** the `#[doc_type_params]` attribute.

### Phase 4: Integration

1.  Register `document_impl` in [`lib.rs`](../../fp-macros/src/lib.rs).
2.  Add integration tests ensuring bounds are correctly captured.

## Usage Example

```rust
#[document_impl(
    // Document 'A' (impl generic)
    doc_type_params("The type of the elements.")
)]
impl<A: Clone> Semigroup for CatList<A> {
    /// Appends two lists.
    #[hm_signature] // Marker: Put signature HERE
    /// Some other documentation
    #[doc_type_params] // Marker: Put parameter docs HERE (even if empty for method)
    /// Some more documentation
    fn append(self, other: Self) -> Self { ... }
}
```

**Expands to (conceptually):**

```rust
impl<A: Clone> Semigroup for CatList<A> {
    /// Appends two lists.
    /// `forall a. (Semigroup (CatList a), Clone a) => (CatList a, CatList a) -> CatList a`
    /// Some other documentation
    /// * `A`: The type of the elements.
    /// Some more documentation
    fn append(self, other: Self) -> Self { ... }
}
```
