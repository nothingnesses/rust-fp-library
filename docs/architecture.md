# Architecture & Design Decisions

This document records architectural decisions and design patterns used in `fp-library`.

## 1. Module Organization

### 1.1. Brand Structs (Centralized)

**Decision:**

Brand structs (e.g., `OptionBrand`) are **centralized** in `src/brands.rs`.

**Reasoning:**

- **Leaf Nodes:** In the dependency graph, Brand structs are **leaf nodes**; they have no outgoing edges (dependencies) to other modules in the crate.
- **Graph Stability:** Centralizing these leaf nodes in `brands.rs` creates a stable foundation. Higher-level modules (like `types/*.rs`) can import from this common sink without creating back-edges or cycles.

### 1.2. Free Functions (Distributed)

**Decision:**

Free function wrappers (e.g., `map`, `pure`) are **defined in their trait's module** (e.g., `src/classes/functor.rs`) and re-exported in `src/functions.rs`.

**Reasoning:**

- **Downstream Dependencies:** Free function wrappers are **downstream nodes**; they depend on (have outgoing edges to) the trait definitions.
- **Cycle Prevention:** `functions.rs` also contains generic helpers (like `compose`) which are **leaf nodes**. If we defined the downstream wrappers in `functions.rs`, the file would effectively become both upstream _and_ downstream of the trait modules. This would create **bidirectional dependencies** (cycles) if a trait module ever needed to import a helper.
- **Facade Pattern:** `functions.rs` acts as a **facade**, re-exporting symbols to provide a unified API surface without coupling the underlying definition graph.

## 2. Documentation & Examples

### 2.1. Signature Accuracy

The "Type Signature" section in documentation comments must be accurate to the code. They should correctly indicate if a function uses uncurried or curried semantics. Instances of `Brand` types in type parameters should be replaced with their corresponding concrete type in the signature, for consistency and to prevent confusion.

Example:

```haskell
forall fn_brand e b a. Semiapplicative (Result e) => (Result (fn_brand a b) e, Result a e) -> Result b e
```

Instead of:

```haskell
forall fn_brand e b a. Semiapplicative (ResultWithErrBrand e) => (Result (fn_brand a b) e) -> (Result a e) -> Result b e
```

### 2.2. Quantifier Accuracy & Ordering

The quantifiers in the "Type Signature" section must be:

- 2.2.1. Accurate and correctly ordered (matching the code).
- 2.2.2. But omit quantifiers that aren't used in the rest of the signature, for clarity.

### 2.3. Parameter List Ordering

The items in the "Type Parameters" section must be correctly ordered (matching the code).

### 2.4. Examples

Where possible, all examples should:

- 2.4.1. Import items using grouped wildcards instead of individually by name.

Example:

```rust
/// use fp_library::{brands::*, classes::*, functions::*};
```

Instead of:

```rust
/// use fp_library::functions::apply;
/// use fp_library::classes::cloneable_fn::CloneableFn;
/// use fp_library::brands::{ResultWithErrBrand, RcFnBrand};
```

- 2.4.2. Use the free-function versions of trait methods, imported with `fp_library::functions::*`, with as many holes as possible, instead of importing the trait methods directly and instead of showing the holes filled in.

Example:

```rust
map::<OptionBrand, _, _, _>(|x| x * 2, Some(5))
```

Instead of:

```rust
OptionBrand::map(|i| i * 2, Some(5))
```

**Reasoning:**

- **Standardization:** A consistent comment format ensures that all API documentation is uniform, easy to read, and provides all necessary information (signatures, parameters, examples) in a predictable structure.
- **Intended Usage:** The library is designed to be used via free functions with partial type inference (turbofish with holes). Examples must demonstrate this intended usage pattern to educate users on the ergonomic benefits.
