# Type Parameter Documentation Macro

## Motivation

### Problem Statement

Rust functions in the `fp-library` often have complex generic type parameters, including lifetimes, brands, and higher-kinded type constraints. Manually documenting these parameters in a consistent format is tedious and error-prone. The order of parameters in the documentation can easily drift from the actual code during refactoring.

### Solution

A procedural macro `doc_type_params` that generates the body of the "Type Parameters" documentation section based on the function's actual generic signature.

### Benefits

1.  **Consistency**: Ensures a uniform `* Name: Description` format across the codebase.
2.  **Correctness**: Validates at compile-time that every generic parameter has a corresponding description.
3.  **Maintenance**: Forces documentation updates when generic parameters are added or removed (due to the count check).

## Basic Usage

### Attribute Macro

The macro is applied as an attribute to function definitions. It takes a list of string literals, one for each generic parameter (lifetime, type, or const) in the function signature. It generates only the list of parameters, allowing the user to provide the section header manually.

#### Example

```rust
/// Some docs about the function
///
/// ### Type Parameters
///
#[doc_type_params(
    "The lifetime of the structure.",
    "The brand of the witherable structure.",
    "The applicative context.",
    "The type of the elements.",
)]
pub fn wilt<'a, F: Witherable, M: Applicative, A: 'a + Clone>(...)
```

### Expected Output

The macro expands to include the list of type parameters in the documentation:

```rust
/// Some docs about the function
///
/// ### Type Parameters
///
/// * `'a`: The lifetime of the structure.
/// * `F`: The brand of the witherable structure.
/// * `M`: The applicative context.
/// * `A`: The type of the elements.
///
pub fn wilt<'a, F: Witherable, M: Applicative, A: 'a + Clone>(...)
```

## Functional Requirements

### 1. One-to-One Mapping

**Requirement**: Every generic parameter in the function definition (lifetimes, types, consts) must have exactly one corresponding description string in the macro arguments.

**Behavior**:
- The macro iterates through the function's generic parameters in definition order.
- It pairs each parameter name with the next string argument.
- If the number of arguments does not exactly match the number of generic parameters, the macro emits a **compile-time error**.

### 2. Output Formatting

**Requirement**: Generate a Markdown list of parameters. The macro does **not** generate the section header (e.g., "### Type Parameters").

**Format**:
```markdown
* `Name`: Description
```

- `Name` is the identifier of the generic parameter (e.g., `'a`, `T`, `N`).
- `Description` is the string provided in the macro argument.

### 3. In-Place Insertion

**Requirement**: The generated documentation must be inserted exactly where the macro attribute was placed, preserving the relative order with other documentation comments and attributes (like `hm_signature`).

**Implementation Strategy**:
- Reuse the documentation insertion logic from `hm_signature`.
- Refactor the insertion logic into a shared utility to ensure consistency between both macros.

### 4. Parameter Types

**Requirement**: Handle all types of generic parameters.

- **Lifetimes**: `fn foo<'a>(...)` -> `* 'a: ...`
- **Types**: `fn foo<T>(...)` -> `* T: ...`
- **Consts**: `fn foo<const N: usize>(...)` -> `* N: ...`

## Implementation Plan

### Phase 1: Shared Utilities
- Refactor `fp-macros/src/hm_signature.rs` to extract `insert_doc_comment` (and potentially `generate_doc_comment`) into a shared module (e.g., `fp-macros/src/doc_utils.rs`).
- Update `hm_signature` to use this shared utility.

### Phase 2: Macro Implementation
- Create `fp-macros/src/doc_type_params.rs`.
- Implement parsing of `ItemFn` and the macro arguments (`Punctuated<LitStr, Token![,]>`).
- Implement the validation logic (count check).
- Implement the generation of the documentation string.
- Use the shared utility to insert the comment.

### Phase 3: Testing
- Unit tests for:
    - Correct count matching.
    - Error on mismatch (too few/too many args).
    - Correct formatting of output (no header).
    - Handling of mixed generics (lifetimes, types, consts).
- Integration tests in `fp-macros/tests` to verify it works alongside `hm_signature`.

## Error Handling

The macro will use `syn::Error` to report issues:

- **Argument Mismatch**: "Expected N description arguments, found M." (Points to the macro invocation).
- **Invalid Input**: "Expected string literal." (Points to the invalid argument).

## Non-Goals

- **Automatic Description Generation**: The macro will not attempt to infer descriptions.
- **Filtering**: The macro will not automatically skip "internal" parameters. If a parameter exists in the signature, it must be documented (even if just as "Internal parameter").
