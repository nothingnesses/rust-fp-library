# Architecture & Design Decisions

This document records architectural decisions and design patterns used in `fp-library`.

## 1. Zero-Cost Abstractions & Uncurrying

**Decision:**

The library adopts an **uncurried** API design using **monomorphized** functions (`impl Fn`) for the majority of operations, while reserving **dynamic dispatch** (`dyn Fn`) only for specific "functions-as-data" use cases.

**Reasoning:**

- **Performance (Zero-Cost):**
  - **Monomorphization:** Uncurried functions (e.g., `map(f, fa)`) allow the compiler to monomorphize the function `f`, inlining it and removing vtable lookups.
  - **Allocation Removal:** Curried functions (e.g., `map(f)(fa)`) required creating intermediate closures and often wrapping them in `Rc`/`Arc` to satisfy type signatures. Uncurried versions avoid this allocation entirely.
- **Ergonomics:**
  - Standard Rust idioms prefer uncurried functions.
  - Type inference is significantly improved, reducing the need for complex type annotations.
- **Granularity:**
  - The `Lift` trait (providing `lift2`) allows for zero-cost combination of contexts without the intermediate closure allocations required by `apply`.

**Hybrid Approach:**

While the core API is uncurried, the library retains the ability to handle functions as data where necessary:

1. **Zero-Cost Operations (Uncurried, `impl Fn`):**

   - Used for: `map`, `bind`, `fold`, `traverse`, `lift2`.
   - Characteristics: No heap allocation, static dispatch, full inlining.

2. **Functions-as-Data (Dynamic, `dyn Fn`):**

   - Used for: `Semiapplicative::apply` (heterogeneous collections), `Lazy` (thunks), `Endofunction` (composition).
   - Characteristics: Requires `Rc`/`Arc` wrapping (via `FnBrand`), dynamic dispatch.
   - Justification: These operations fundamentally require storing functions of potentially different concrete types in the same structure, or cloning functions where the concrete type is anonymous.

## 2. Pointer Abstraction & Shared Semantics

**Decision:**

The library uses a unified pointer hierarchy to abstract over reference counting strategies (`Rc` vs `Arc`) and to enable shared memoization semantics for lazy evaluation.

**Hierarchy:**

- `Pointer`: Base trait for heap-allocated pointers (requires `Deref`).
- `RefCountedPointer`: Extends `Pointer` with `CloneableOf` (requires `Clone + Deref`).
- `SendRefCountedPointer`: Extends `RefCountedPointer` with `SendOf` (requires `Send + Sync`).

**Patterns:**

1. **Generic Function Brands:** `FnBrand<P>` is parameterized over a `RefCountedPointer` brand `P`.

   - `RcFnBrand` is a type alias for `FnBrand<RcBrand>`.
   - `ArcFnBrand` is a type alias for `FnBrand<ArcBrand>`.
   - This allows unified implementation of `CloneableFn` while `SendCloneableFn` is only implemented when `P: SendRefCountedPointer`.

2. **Shared Memoization:** `Lazy` uses a configuration trait (`LazyConfig`) to abstract over the underlying storage and synchronization primitives, ensuring shared memoization semantics across clones.

   - `Lazy<'a, A, Config>` is parameterized by a `LazyConfig` which defines the storage type.
   - `RcLazy` uses `Rc<LazyCell>` for single-threaded, shared memoization.
   - `ArcLazy` uses `Arc<LazyLock>` for thread-safe, shared memoization.
   - This ensures Haskell-like semantics where forcing one reference updates the value for all clones.

**Reasoning:**

- **Correctness:** Ensures `Lazy` behaves correctly as a shared thunk rather than a value that is re-evaluated per clone.
- **Performance:** Leverages standard library types (`LazyCell`, `LazyLock`) for efficient, correct-by-construction memoization.
- **Flexibility:** Separates the concern of *memoization* (`Lazy`) from *computation* (`Trampoline`/`Thunk`), allowing users to choose the right tool for the job (e.g., `Trampoline` for stack-safe recursion, `Lazy` for caching).

## 3. Granular Lazy Evaluation Types

**Decision:**

The library provides three distinct types for lazy evaluation: `Thunk`, `Trampoline`, and `Lazy`. This granular approach is a deliberate architectural choice to address the specific challenges of functional programming in an eager, systems language like Rust.

**Reasoning:**

In lazy languages like Haskell, the runtime manages evaluation strategies automatically. In Rust, we must be explicit about these concerns to maintain performance and safety guarantees. This design allows users to pay only for the features they need.

### 3.1. The `Thunk<'a, A>` Type: Lightweight & HKT-Compatible

**Purpose:** `Thunk` is a minimal deferred computation designed for "glue code" and scenarios requiring borrowing.

**Design Rationale:**
- **HKT Compatibility:** `Thunk` is designed to be a first-class citizen in the library's Higher-Kinded Type (HKT) system. It implements `Functor`, `Semimonad`, and other traits directly, allowing it to be used generically where other `Functor`s or `Monad`s are expected.
- **Borrowing Support:** By using lifetime parameters (`'a`), `Thunk` can capture and return references to data on the stack. This is impossible with `'static`-only types.
- **Zero-Overhead Computation:** `Thunk` is essentially a wrapper around a `Box<dyn FnOnce() -> A + 'a>`. It adds minimal overhead, making it ideal for short chains of operations.

**Trade-offs:**
- **Not Stack-Safe:** The primary limitation of `Thunk` is that it is not stack-safe. Each call to `.bind()` adds a frame to the call stack. Deep recursion (>1000 calls) will cause a stack overflow. This is an acceptable trade-off for its speed and flexibility.

### 3.2. The `Trampoline<A>` Type: Stack-Safe Recursion

**Purpose:** `Trampoline` is a "heavy-duty" monadic type for deferred computations that require guaranteed stack safety, such as deep recursion or long pipelines.

**Design Rationale:**
- **Stack Safety via Trampolining:** `Trampoline` is built on the `Free<Thunk, A>` monad. This construction implements a trampoline, which iteratively processes a list of continuations instead of making recursive calls on the call stack. This allows for unlimited recursion depth without stack overflow.
- **`'static` Requirement:** The trampoline mechanism and the `Free` monad's internal structure require that the contained value `A` be `'static`. This is because the trampoline may need to store the value or intermediate states across multiple iterations, and the Rust borrow checker cannot easily prove lifetime safety for these complex, recursive patterns.
- **Performance:** While `Trampoline` guarantees safety, it comes with a performance cost compared to `Thunk` due to the indirection and allocation within the `Free` monad structure.

**Trade-offs:**
- **Loss of Borrowing:** The `'static` constraint means `Trampoline` cannot work with borrowed data. This is a significant limitation when integrating with code that uses references extensively.
- **Higher Overhead:** The trampoline mechanism is more complex and slower than a direct function call.

### 3.3. The `Lazy<'a, A, Config>` Type: Caching & Shared Semantics

**Purpose:** `Lazy` is not a computation type itself, but a wrapper around a computation that ensures it runs at most once, with the result shared across all clones.

**Design Rationale:**
- **Separation of Concerns:** `Lazy` decouples the *act of computation* from the *act of caching*. This allows any `Thunk` or `Trampoline` to be memoized by simply wrapping it in a `Lazy`.
- **Shared, Lazy Initialization:** `Lazy` uses Rust's `std::cell::LazyCell` (for `RcLazy`) or `std::sync::LazyLock` (for `ArcLazy`). These types provide "initialization-once" guarantees, ensuring the computation runs exactly one time, no matter how many times `.get()` is called on any clone.
- **Configuration via `LazyConfig`:** The `LazyConfig` trait abstracts over the choice of pointer (`Rc` vs `Arc`) and lazy cell type, allowing users to select single-threaded (`RcLazy`) or thread-safe (`ArcLazy`) memoization.

**Trade-offs:**
- **Allocation and Synchronization:** Caching requires memory allocation for the thunk and, in the case of `ArcLazy`, synchronization primitives. This overhead is justified only for expensive computations.
- **Not a Control Flow Structure:** `Lazy` is primarily a data container. While it has some monadic properties, it's not the right tool for building complex computational pipelines.

### 3.4. The Granular Approach in Rust

This three-type system is a direct response to Rust's characteristics as an eagerly-evaluated, systems language:

1.  **Explicit Trade-offs:** In Rust, you cannot have it all. You must choose between performance (`Thunk`), safety (`Trampoline`), and caching (`Lazy`). This design makes those choices explicit and manageable.
2.  **Zero-Cost Abstractions:** The system allows users to compose these types. For example, you can use a `Trampoline` for a recursive algorithm, wrap it in a `Lazy` to cache the result, and then use an `Thunk` to borrow that result for a final transformation. The user only pays for the `Lazy`'s allocation and the `Trampoline`'s trampoline, not for features they aren't using.
3.  **Integration with Ownership:** The lifetime system of Rust is respected. `Thunk` can play nicely with the borrow checker, while `Trampoline` and `Lazy` provide clear pathways (`'static`, `Arc`) for sharing data across contexts where borrowing is not possible.

## 4. Module Organization

### 4.1. Brand Structs (Centralized)

**Decision:**

Brand structs (e.g., `OptionBrand`) are **centralized** in `src/brands.rs`.

**Reasoning:**

- **Leaf Nodes:** In the dependency graph, Brand structs are **leaf nodes**; they have no outgoing edges (dependencies) to other modules in the crate.
- **Graph Stability:** Centralizing these leaf nodes in `brands.rs` creates a stable foundation. Higher-level modules (like `types/*.rs`) can import from this common sink without creating back-edges or cycles.

### 4.2. Free Functions (Distributed)

**Decision:**

Free function wrappers (e.g., `map`, `pure`) are **defined in their trait's module** (e.g., `src/classes/functor.rs`) and re-exported in `src/functions.rs`.

**Reasoning:**

- **Downstream Dependencies:** Free function wrappers are **downstream nodes**; they depend on (have outgoing edges to) the trait definitions.
- **Cycle Prevention:** `functions.rs` also contains generic helpers (like `compose`) which are **leaf nodes**. If we defined the downstream wrappers in `functions.rs`, the file would effectively become both upstream _and_ downstream of the trait modules. This would create **bidirectional dependencies** (cycles) if a trait module ever needed to import a helper.
- **Facade Pattern:** `functions.rs` acts as a **facade**, re-exporting symbols to provide a unified API surface without coupling the underlying definition graph.

## 5. Type Parameter Ordering

**Decision:**

Type parameters for traits and functions are ordered primarily by **Inference Priority**, and secondarily by **Dependency Order**:

1. **Inference Priority:** Type parameters are ordered by inferability: **rarely inferable** types (e.g., "Brand" markers) are placed first, followed by **context-dependent** types (e.g., return types), and finally **usually inferable** types (e.g., input arguments).
2. **Dependency Order:** Type parameters that are dependencies of other type parameters are placed **before** the dependent parameters.

**Reasoning:**

- **Ergonomics:** Placing uninferable types first allows users to specify them using turbofish syntax (e.g., `map::<OptionBrand, _, _, _>(...)`) without needing to specify inferable types that would otherwise precede them.
- _Note:_ Lifetime parameters do not affect this ordering as they are generally omitted in turbofish syntax (e.g., `func::<Type>` is valid even if `func` has lifetime parameters).
- **Readability & Convention:** Ordering dependencies before dependents (e.g., `A, B` before `F: Fn(A) -> B`) mirrors the logical structure of the types and aligns with Rust standard library conventions (e.g., `std::iter::Map<I, F>` where `I` is the iterator and `F` is the function).

#### Determining Inferability

> **Note:** Inferability is fundamentally a property of the _call site_, not the function signature alone. The same type parameter may be inferable in one context and not another. The categories below are _heuristics_ for predicting typical inference behavior.

Type parameters fall into three categories based on how reliably the compiler can infer them:

1. **Rarely Inferable:** Parameters appearing only in `where` bounds, or "Brand" markers used for trait selection. These typically require explicit turbofish specification.

```rust
// Brand only appears in where bounds for trait selection
fn pure<Brand, A>(value: A) -> Kind<Brand, A>
where
   Brand: Pointed<A>
{ ... }

// Brand cannot be inferred; turbofish required
pure::<OptionBrand, _>(42)  // ✓
pure(42)                    // ✗ cannot infer Brand
```

2. **Context-Dependent:** Parameters appearing only in the return type. These _may_ be inferred if the call-site provides type context, but often require turbofish when context is absent.

```rust
fn default<T: Default>() -> T { T::default() }

let x: i32 = default();      // ✓ T inferred from annotation
vec.push(default());         // ✓ T inferred from vec's element type
let y = default();           // ✗ cannot infer T
let z = default::<String>(); // ✓ explicit turbofish
```

3. **Usually Inferable:** Parameters appearing in function input arguments. The compiler infers these from call-site values.

```rust
fn map<Brand, FnBrand, A, B>(f: Kind<FnBrand, A, B>, fa: Kind<Brand, A>) -> Kind<Brand, B>
where
   Brand: Functor<FnBrand, A, B>
{ ... }

// A and B inferred from closure and input; Brand still needs turbofish
map::<OptionBrand, _, _, _>(|x| x * 2, Some(5))
```

**Ordering:** Place type parameters in the order listed above (rarely inferable → context-dependent → usually inferable). This enables ergonomic turbofish patterns like `func::<Brand, _, _>(...)`, where users specify only the uninferable parameters and let the compiler fill in the rest.

## 6. Documentation & Examples

**Decision:**

Documentation must adhere to the following standards regarding formatting, type signatures, and examples:

### 6.1. **Documentation Templates**

Documentation comments should use the following formats.

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

- **Struct Fields and Enum Variants:**

```rust
/// Short description of the field/variants's purpose and constraints.
```

- **Unit Tests:**

```rust
/// Explanation of what the test is testing for.
///
/// Explanation of how the test is testing for what it's testing.
```

Sections in the documentation that would be empty should be omitted.

### 6.2. **Signature Accuracy**

The "Type Signature" section in documentation comments must be accurate to the code. They should correctly indicate if a function uses uncurried or curried semantics. Instances of `Brand` types in type parameters should be replaced with their corresponding concrete type in the signature, for consistency and to prevent confusion.

Example:

```haskell
forall fn_brand e b a. Semiapplicative (Result e) => (Result (fn_brand a b) e, Result a e) -> Result b e
```

Instead of:

```haskell
forall fn_brand e b a. Semiapplicative (ResultWithErrBrand e) => (Result (fn_brand a b) e) -> (Result a e) -> Result b e
```

### 6.3. **Quantifier Accuracy & Ordering**

The quantifiers in the "Type Signature" section must be:

- 6.3.1. Accurate and correctly ordered (matching the code).
- 6.3.2. But omit quantifiers that aren't used in the rest of the signature, for clarity.

### 6.4. **Parameter List Ordering**

The items in the "Type Parameters" section must be correctly ordered (matching the code).

### 6.5. **Examples**

Where possible, all examples should:

- 6.5.1. Import items using grouped wildcards instead of individually by name.

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

- 6.5.2. Use the free-function versions of trait methods, imported with `fp_library::functions::*`, with as many holes as possible, instead of importing the trait methods directly and instead of showing the holes filled in.

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
- **Consistency:** Documentation must accurately reflect the architectural decisions regarding type parameter ordering (Section 3) to avoid confusion.
- **Intended Usage:** The library is designed to be used via free functions with partial type inference (turbofish with holes). Examples must demonstrate this intended usage pattern to educate users on the ergonomic benefits of the chosen type parameter ordering.
