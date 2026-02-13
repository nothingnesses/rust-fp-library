# Function Parameter Documentation Macro

## Motivation

### Problem Statement

Rust functions in the `fp-library` often have multiple arguments (parameters). Manually documenting these parameters in a consistent format is tedious and error-prone. The order of parameters in the documentation can easily drift from the actual code during refactoring, and ensuring every parameter has a description is a manual process.

Additionally, functions in functional programming libraries often have "curried" semantics, where a function returns another function (e.g., `impl Fn(A) -> B`). The arguments of these returned functions are conceptually parameters of the main function but don't appear in the function's argument list, making them difficult to document consistently.

### Solution

A procedural macro `doc_params` (or `doc_fn_params`) that generates the body of the "Arguments" (or "Parameters") documentation section based on the function's actual signature, including handling curried return types.

### Benefits

1.  **Consistency**: Ensures a uniform `* Name: Description` format across the codebase.
2.  **Correctness**: Validates at compile-time that every function parameter (excluding `self`) has a corresponding description.
3.  **Completeness**: Handles both explicit arguments and implicit "curried" arguments from returned function traits.
4.  **Maintenance**: Forces documentation updates when parameters are added or removed.

## Basic Usage

### Attribute Macro

The macro is applied as an attribute to function definitions. It takes a list of arguments, where each argument can be:
- A string literal: `"Description"` (uses the parameter name from code, or `_` for unnamed curried parameters).
- A tuple: `("Name Override", "Description")` (forces a specific name).

It generates only the list of parameters, allowing the user to provide the section header manually.

#### Example

```rust
/// Some docs about the function
///
/// ### Arguments
///
#[doc_params(
    "The function to apply to the elements.",
    ("ta", "The traversable structure."),
    ("context", "The applicative context (implicit in return type).")
)]
pub fn traverse<G, A, B, F>(f: F, ta: Kind<G, A>) -> impl Fn(Kind<F, Context>) -> Kind<G, Kind<F, B>>
where ...
{ ... }
```

### Expected Output

```rust
/// Some docs about the function
///
/// ### Arguments
///
/// * `f`: The function to apply to the elements.
/// * `ta`: The traversable structure.
/// * `context`: The applicative context (implicit in return type).
///
pub fn traverse<G, A, B, F>(f: F, ta: Kind<G, A>) -> ...
```

## Functional Requirements

### 1. One-to-One Mapping

**Requirement**: Every "logical" parameter must have exactly one corresponding entry in the macro arguments.

**Logical Parameters include**:
1.  **Explicit Arguments**: Parameters in the function definition (excluding `self`).
2.  **Curried Arguments**: Arguments of returned function types (`Fn`, `FnMut`, `FnOnce`) or HKTs acting as functions (e.g., `SendCloneableFn`, `CloneableFn`, `Function`).

**Behavior**:
- The macro iterates through the function's explicit inputs (skipping `self`).
- It then analyzes the return type. If the return type represents a function (via `impl Trait` or HKT bound), it traverses into that function's arguments recursively.
- It pairs each logical parameter with the next argument provided to the macro.
- If the count does not match, it emits a **compile-time error**.

### 2. Output Formatting

**Requirement**: Generate a Markdown list of parameters.

**Format**:
```markdown
* `Name`: Description
```

**Name Determination**:
1.  **Override**: If the macro argument is `("Name", "Desc")`, use `"Name"`.
2.  **Explicit Parameter**: If the macro argument is `"Desc"`, derive name from the argument pattern (stringify the pattern, e.g., `f`, `(a, b)`).
3.  **Curried/Unnamed Parameter**: If the macro argument is `"Desc"` and the parameter has no name (e.g., in `Fn(i32)`), use `_`.

### 3. In-Place Insertion

**Requirement**: Insert generated docs exactly where the attribute was placed.
- Reuse `insert_doc_comment` from `fp-macros/src/doc_utils.rs`.

### 4. Curried Semantics & HKT Support

**Requirement**: The macro must understand "function-like" return types, similar to `hm_signature`.

**Supported Traits**:
- Standard: `Fn`, `FnMut`, `FnOnce`.
- HKTs: `SendCloneableFn` (assoc type `SendOf`), `CloneableFn` (assoc type `Of`), `Function` (assoc type `Output` or similar).

**Detection Logic**:
- Should reuse/share logic with `hm_signature.rs` for detecting these traits and traversing their "arguments".
- Example: `fn foo() -> impl Fn(A) -> B` has 1 implicit parameter `A`.
- Example: `fn foo() -> <F as SendCloneableFn>::SendOf<'a, A, B>` has implicit parameters from the `SendOf` usage (likely `A`).

## Implementation Plan

### Phase 1: Refactoring & Shared Utilities
- Extract function trait detection and traversal logic from `fp-macros/src/hm_signature.rs` into a shared module (e.g., `fp-macros/src/function_utils.rs` or `doc_utils.rs`).
- Ensure `hm_signature` continues to work using the shared logic.

### Phase 2: Macro Implementation (`doc_params`)
- Create `fp-macros/src/doc_params.rs`.
- Implement parsing of macro arguments (supporting both `LitStr` and `(LitStr, LitStr)`).
- Implement logical parameter traversal (combining explicit args + curried return type args).
- Implement name resolution (pattern vs override vs `_`).
- Implement validation and doc generation.

### Phase 3: Updates to `doc_type_params`
- Update `doc_type_params` to also support tuple overrides for consistency.
- Ensure it handles the new argument format correctly.

### Phase 4: Testing
- Unit tests for:
    - Mixed argument types (strings and tuples).
    - Curried function traversal.
    - HKT function traversal.
    - Error conditions (count mismatch).
- Integration tests.

## Error Handling

- **Argument Mismatch**: Report expected vs found count.
- **Invalid Input**: Report if arguments are not strings or string tuples.

## Non-Goals

- **Deep Traversal**: The macro might limit recursion depth for curried functions to avoid infinite loops in pathological cases, though `hm_signature` logic usually handles this via type structure analysis.
