# Procedural Macro for Kind Trait Generation

## 1. Problem Statement
The `fp-library` uses a set of "Kind" traits to represent higher-kinded types (HKTs) of various arities and bounds. These traits follow a strict but complex naming convention based on De Bruijn indices, e.g., `Kind_L1_T1_B0l0_Ol0`.

**Issues:**
*   **Complexity:** Users must manually calculate and memorize these names.
*   **Brittleness:** A typo in the name (e.g., `L0` instead of `L1`) might compile but refer to the wrong trait, leading to confusing type errors.
*   **Maintenance:** Changing the naming convention requires updating every usage site.

## 2. Proposed Solution
Introduce a procedural macro `Kind!` (and an internal `def_kind!`) in a new crate `fp-macros`.

*   **`Kind!(...)`**: A macro used in type positions that expands to the canonical trait name.
    *   Input: `Kind!(('a), (T: 'a), (: 'a))`
    *   Output: `Kind_L1_T1_B0l0_Ol0`
*   **`def_kind!(...)`**: A macro used to define the traits in `fp-library`.
    *   Input: `def_kind!(('a), (T: 'a), (: 'a))`
    *   Output: `pub trait Kind_L1_T1_B0l0_Ol0 { ... }`

## 3. Justification
*   **Developer Experience:** Users simply describe the shape of the HKT they need.
*   **Correctness:** The macro guarantees that the name matches the signature.
*   **Future-Proofing:** The underlying naming scheme can change without breaking user code.

## 4. Implementation Details

### 4.1. Input Syntax
The macro will accept a tuple-like syntax:
```rust
Kind!(
    (lifetimes...), // e.g., ('a, 'b)
    (types...),     // e.g., (T: 'a, U)
    (output_bounds) // e.g., (: 'a)
)
```

### 4.2. Canonicalization Logic
To ensure deterministic names, the macro must normalize inputs:
1.  **Indexing:** Replace names with 0-based indices.
    *   Lifetimes: First seen lifetime becomes `0`, second `1`, etc.
    *   Types: First seen type becomes `0`, second `1`, etc.
2.  **Sorting:** Bounds must be sorted to ensure `T: A + B` generates the same name as `T: B + A`.

### 4.3. Name Generation
Construct the identifier string following the existing convention:
`Kind_L{num_lifetimes}_T{num_types}[_B{bounds}][_O{output_bounds}]`

### 4.4. Crate Structure
*   `fp-macros`: A `proc-macro` crate.
*   Dependencies: `syn` (parsing), `quote` (generation), `proc-macro2`.

## 5. Integration
*   `fp-library` will depend on `fp-macros`.
*   Existing `make_trait_kind!` macro_rules might be superseded or used by `def_kind!`.
