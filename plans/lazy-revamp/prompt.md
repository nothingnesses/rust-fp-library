Implement Step 1 of @/docs/lazy-revamp/plan.md . Ensure all implemented code is FULLY compliant with :

`````md
### Type Parameter Ordering

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

### Documentation & Examples

**Decision:**
Documentation must adhere to the following standards regarding formatting, type signatures, and examples:

1. **Documentation Templates:** Documentation comments should use the following formats.

   - **Functions & Methods:** (Note: functions use uncurried semantics, so the documentation should reflect this)

````rust
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
/// use fp_library::classes::cloneable_fn::CloneableFn;
/// use fp_library::brands::{ResultWithErrBrand, RcFnBrand};
```

5.2. Use the free-function versions of trait methods, imported with `fp_library::functions::*`, with as many holes as possible, instead of importing the trait methods directly and instead of showing the holes filled in.

- Example: `map::<OptionBrand, _, _, _>(|x| x * 2, Some(5))` instead of `OptionBrand::map(|i| i * 2, Some(5))`.

**Reasoning:**

- **Standardization:** A consistent comment format ensures that all API documentation is uniform, easy to read, and provides all necessary information (signatures, parameters, examples) in a predictable structure.
- **Consistency:** Documentation must accurately reflect the architectural decisions regarding type parameter ordering (Section 3) to avoid confusion.
- **Intended Usage:** The library is designed to be used via free functions with partial type inference (turbofish with holes). Examples must demonstrate this intended usage pattern to educate users on the ergonomic benefits of the chosen type parameter ordering.
`````

Look at @/fp-library/src/classes/functor.rs and @/fp-library/src/types/identity.rs for examples of required documentation.

In case of ANY ambiguities or decisions that need to be made, STOP immediately and ask the user for clarification and approval:

1. Detail all viable approaches
2. State trade-offs for each
3. Propose your recommended solution
4. Await explicit approval before proceeding