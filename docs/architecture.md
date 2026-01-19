# Architecture & Design Decisions

This document records architectural decisions and design patterns used in `fp-library`.

## Module Organization

### 1. Brand Structs (Centralized)

**Decision:**
Brand structs (e.g., `OptionBrand`) are **centralized** in `src/brands.rs`.

**Reasoning:**

- **Leaf Nodes:** In the dependency graph, Brand structs are **leaf nodes**; they have no outgoing edges (dependencies) to other modules in the crate.
- **Graph Stability:** Centralizing these leaf nodes in `brands.rs` creates a stable foundation. Higher-level modules (like `types/*.rs`) can import from this common sink without creating back-edges or cycles.

### 2. Free Functions (Distributed)

**Decision:**
Free function wrappers (e.g., `map`, `pure`) are **defined in their trait's module** (e.g., `src/classes/functor.rs`) and re-exported in `src/functions.rs`.

**Reasoning:**

- **Downstream Dependencies:** Free function wrappers are **downstream nodes**; they depend on (have outgoing edges to) the trait definitions.
- **Cycle Prevention:** `functions.rs` also contains generic helpers (like `compose`) which are **leaf nodes**. If we defined the downstream wrappers in `functions.rs`, the file would effectively become both upstream _and_ downstream of the trait modules. This would create **bidirectional dependencies** (cycles) if a trait module ever needed to import a helper.
- **Facade Pattern:** `functions.rs` acts as a **facade**, re-exporting symbols to provide a unified API surface without coupling the underlying definition graph.

### 3. Type Parameter Ordering

**Decision:**
Type parameters for traits and functions are ordered according to two principles, applied in sequence:

1.  **Inference Priority:** Concrete types that cannot be easily inferred by the compiler (e.g., "Brand" types or return types) are placed **before** types that can be inferred.
2.  **Dependency Order:** Type parameters that are dependencies of other type parameters are placed **before** the dependent parameters.

**Reasoning:**

- **Ergonomics:** Placing uninferable types first allows users to specify them using turbofish syntax (e.g., `map::<OptionBrand, _, _, _>(...)`) without needing to specify inferable types that would otherwise precede them.
  - _Note:_ Lifetime parameters do not affect this ordering as they are generally omitted in turbofish syntax (e.g., `func::<Type>` is valid even if `func` has lifetime parameters).
- **Readability & Convention:** Ordering dependencies before dependents (e.g., `A, B` before `F: Fn(A) -> B`) mirrors the logical structure of the types and aligns with Rust standard library conventions (e.g., `std::iter::Map<I, F>` where `I` is the iterator and `F` is the function).

#### Determining Inferability

To determine which type parameters can be inferred (and thus should be placed later in the list):

- **Inferable:** Parameters that appear in the function's input arguments (e.g., `A` in `fn map<A, B>(a: A, ...)`). The compiler can determine these from the values passed at the call site.
- **Uninferable:** Parameters that only appear in the return type (e.g., `M` in `fn empty<M>() -> M`) or act as "Brand" markers for trait selection. These require explicit specification via turbofish (`::<>`) and should be placed first for better ergonomics.

### 4. Documentation & Examples

**Decision:**
Documentation must adhere to the following standards regarding formatting, type signatures, and examples:

1. **Documentation Templates:** Documentation comments should use the following formats.

   - **Functions & Methods:** (Note: functions use uncurried semantics, so the documentation should reflect this)

     ````rust
     /// Short description.
     ///
     /// Comprehensive explanation detailing the purpose of the function.
     ///
     /// ### Type Signature
     ///
     /// `Accurate "Hindley-Milner"/Haskell-like type signature with quantifiers if appropriate`
     ///
     /// ### Type Parameters
     ///
     /// * `TypeParameter1`: Explanation of `TypeParameter1`'s purpose.
     /// ...other type parameters
     ///
     /// ### Parameters
     ///
     /// * `parameter1`: Explanation of `parameter1`'s purpose.
     /// ...other parameters
     ///
     /// ### Returns
     ///
     /// Explanation of the value returned.
     ///
     /// ### Examples
     ///
     /// ```
     /// Comprehensive examples specific to the type implementing the trait, showing the full extent of the function's capabilities
     /// ```
     ````

   - **Modules:**

     ````rust
     //! Short description of the module.
     //!
     //! Comprehensive explanation of the module's purpose, scope, and key components.
     //!
     //! ### Examples
     //!
     //! ```
     //! // Module-level examples
     //! ```
     ````

   - **Structs & Enums:**

     ````rust
     /// Short description.
     ///
     /// Comprehensive explanation.
     ///
     /// ### Type Parameters
     ///
     /// * `T`: Explanation of `T`'s purpose.
     ///
     /// ### Fields
     ///
     /// * `field_name`: Explanation of the field (if not documented individually).
     ///
     /// ### Examples
     ///
     /// ```
     /// // Examples
     /// ```
     ````

   - **Struct Fields:**

     ```rust
     /// Short description of the field's purpose and constraints.
     ```

Sections in the documentation that would be empty should be omitted.

2. **Signature Accuracy:** The "Type Signature" section in documentation comments must be accurate to the code. Instances of `Brand` types in type parameters should be replaced with their corresponding concrete type in the signature, for consistency and to prevent confusion. E.g. prefer `forall fn_brand e b a. Semiapplicative (Result e) => (Result (fn_brand a b) e, Result a e) -> Result b e` instead of `forall fn_brand e b a. Semiapplicative (ResultWithErrBrand e) => (Result (fn_brand a b) e, Result a e) -> Result b e`.
3. **Quantifier Accuracy & Ordering:** The quantifiers in the "Type Signature" section must be:

   3.1. Accurate and correctly ordered (matching the code).

   3.2. But omit quantifiers that aren't used in the rest of the signature, for clarity.

4. **Parameter List Ordering:** The items in the "Type Parameters" section must be correctly ordered (matching the code).
5. **Examples:** Where possible, all examples should:

   5.1. Import items using grouped wildcards instead of individually by name. Example:

   ```rust
   /// use fp_library::{brands::*, classes::*, functions::*};
   ```

   Instead of:

   ```rust
   /// use fp_library::functions::apply;
   /// use fp_library::classes::clonable_fn::ClonableFn;
   /// use fp_library::brands::{ResultWithErrBrand, RcFnBrand};
   ```

   5.2. Use the free-function versions of trait methods, imported with `fp_library::functions::*`, with as many holes as possible, instead of importing the trait methods directly and instead of showing the holes filled in.

   - Example: `map::<OptionBrand, _, _, _>(|x| x * 2, Some(5))` instead of `OptionBrand::map(|i| i * 2, Some(5))`.

**Reasoning:**

- **Standardization:** A consistent comment format ensures that all API documentation is uniform, easy to read, and provides all necessary information (signatures, parameters, examples) in a predictable structure.
- **Consistency:** Documentation must accurately reflect the architectural decisions regarding type parameter ordering (Section 3) to avoid confusion.
- **Intended Usage:** The library is designed to be used via free functions with partial type inference (turbofish with holes). Examples must demonstrate this intended usage pattern to educate users on the ergonomic benefits of the chosen type parameter ordering.
