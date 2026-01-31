# Hindley-Milner Type Signature Generation Macro

## Motivation

### Problem Statement

The fp-library codebase uses Hindley-Milner style type signatures in documentation comments to communicate the essential semantics of functions and type class methods. These signatures:

- Provide a clean, mathematical view of type transformations
- Relate to functional programming literature (Haskell, PureScript, etc.)
- Omit Rust-specific implementation details (lifetimes, Clone bounds, HKT encoding artifacts)
- Make the conceptual operation clear at a glance

However, **manually maintaining these signatures is error-prone and labor-intensive**:

1. **Consistency issues**: Different files may use different conventions for what to include/omit
2. **Maintenance burden**: Changes to Rust signatures require manual updates to doc comments
3. **Human error**: Easy to forget to update documentation when refactoring
4. **Learning curve**: Contributors need to understand the conventions for writing these signatures

### Solution

A procedural macro that **automatically generates Hindley-Milner style type signatures** from Rust function/method definitions. The macro should:

- Parse Rust type signatures
- Apply consistent transformation rules
- Generate a single-line doc comment with the type signature
- Accurately reflect the actual function structure (curried vs uncurried)
- Reduce maintenance burden and ensure consistency

### Benefits

1. **Consistency**: All generated signatures follow the same rules
2. **Accuracy**: Signatures always match the actual Rust implementation
3. **Maintainability**: Refactoring automatically updates documentation
4. **Developer experience**: Contributors don't need to learn HM signature conventions
5. **Documentation quality**: Reduces errors and omissions in docs

## Basic Usage

### Attribute Macro

The macro should be applied as an attribute to function/method definitions and expands to a doc comment in place:

```rust
/// Some docs about the function
#[hm_signature]
pub fn wither<'a, F: Witherable, M: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
    func: Func,
    ta: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>>>)
where
    Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
    Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
{
    // implementation
}
```

### Expected Output

The macro should expand to:

```rust
/// Some docs about the function
/// `forall f m a b. (Witherable f, Applicative m) => (a -> m (Option b), f a) -> m (f b)`
pub fn wither<'a, F: Witherable, M: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
    func: Func,
    ta: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>>>)
where
    Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
    Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
{
    // implementation
}
```

### Design Principles

1. **Simple output**: Just generates a single-line doc comment with the signature in backticks
2. **In-place expansion**: Expands exactly where the attribute is placed
3. **Accurate reflection**: The signature should accurately reflect the function structure (curried vs uncurried)
4. **No configuration bloat**: Minimal to no configuration options - the macro does one thing well
5. **Opt-in only**: If you don't want a signature, don't use the macro

## Functional Requirements

### 1. Lifetime Elimination

**Requirement**: Strip all lifetime parameters from the generated signature.

**Rationale**: Lifetimes are Rust-specific and don't represent semantic type relationships.

**Examples**:

```rust
// Input
fn foo<'a, A: 'a>(x: A) -> A

// Output
// forall a. a -> a
```

```rust
// Input
fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(...)

// Output
// forall a b f. (Applicative f) => ...
```

**Edge cases**:
- `'static` bounds should be omitted
- Multiple lifetime parameters should all be removed
- Lifetime bounds in where clauses should be stripped

### 2. Function Trait Transformation

**Requirement**: Convert Fn/FnMut/FnOnce trait bounds and HKT function types to arrow syntax.

**Standard Fn bounds**:

```rust
// Simple function
Func: Fn(A) -> B
// → a -> b

// Tuple input
Func: Fn((A, B)) -> C
// → (a, b) -> c

// Multiple parameters (should be converted to tuple)
Func: Fn(A, B, C) -> D
// → (a, b, c) -> d

// Nested functions (higher-order)
Func: Fn(A) -> Box<dyn Fn(B) -> C>
// → a -> (b -> c)
```

**HKT function types**:

The macro must robustly handle any type parameter bound to a trait with associated types that dereference to Fn-family traits. These include:

- `SendCloneableFn` (associated type: `SendOf`)
- `CloneableFn` (associated type: `Of`)
- `Function` (and any other custom function traits)

The key insight is that the generic parameter name is not necessarily `FnBrand` - it could be any identifier. The macro should:

1. Detect trait bounds like `T: SendCloneableFn`, `T: CloneableFn`, `T: Function`
2. Recognize associated type applications like `<T as SendCloneableFn>::SendOf<'a, A, B>`
3. Convert these to arrow syntax based on the type parameters

```rust
// Generic detection (T could be any name)
where T: SendCloneableFn
<T as SendCloneableFn>::SendOf<'a, A, B>
// → a -> b

where Brand: CloneableFn
<Brand as CloneableFn>::Of<'a, (A, B), C>
// → (a, b) -> c

where F: Function
<F as Function>::Output<'a, Input, Result>
// → input -> result
```

**Pattern matching strategy**:

1. For any where clause `T: TraitName`, check if `TraitName` is in the list of known function traits
2. Look for associated type applications: `<T as TraitName>::AssocType<'a, ...>`
3. The last type parameter is the return type, all others form the input
4. If there's only one input type, use it directly; if multiple, wrap in tuple

**Edge cases**:
- FnMut and FnOnce should use same arrow syntax (HM doesn't distinguish)
- Closures with move semantics should still be arrows
- Generic return types that are functions should nest properly
- The type parameter could appear in trait bounds without being used as a function type

### 3. Trait Bound Filtering

**Requirement**: Distinguish between type class constraints (keep) and implementation details (omit).

#### Omit (Implementation Details)

These bounds are Rust-specific and should be filtered out:

- **Derivable traits**: `Clone`, `Copy`, `Debug`, `Display`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default`
- **Thread safety**: `Send`, `Sync`
- **Special traits**: `Sized`, `Unpin`, `'static`
- **Function HKT traits**: Any type bound to `CloneableFn`, `SendCloneableFn`, `Function`, etc.

#### Keep (Type Class Constraints)

These represent semantic FP constraints:

- **Core functors**: `Functor`, `Apply`, `Applicative`, `Monad`
- **Folding**: `Foldable`, `ParFoldable`, `Traversable`, `Witherable`, `Filterable`, `Compactable`
- **Algebraic**: `Semigroup`, `Monoid`
- **Structural**: `Bifunctor`, `Pointed`, `Category`, `Semigroupoid`

**Examples**:

```rust
// Input
where
    A: Clone + Debug + Send,
    F: Functor,
    M: Applicative + Clone,
    B: Monoid + Eq,

// Output constraint section
// (Functor f, Applicative m, Monoid b) =>
```

**Edge cases**:
- Combined bounds like `Functor + Applicative` should be split
- Negative bounds (if any) should be omitted
- Generic bounds on associated types need special handling

### 4. HKT Type Application Resolution

**Requirement**: Convert the fp-library's HKT encoding to readable type application syntax.

**Pattern recognition**:

```rust
// Single application
Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
// → f a

// Nested application
Apply!(<M as Kind!(...)>::Of<'a, Apply!(<F as Kind!(...)>::Of<'a, B>>)
// → m (f b)

// Bifunctor application (2 type params)
Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>)
// → p a c
```

**Complex examples**:

```rust
// From witherable.rs
Apply!(<M as Kind!(...)>::Of<'a, Pair<
    Apply!(<Self as Kind!(...)>::Of<'a, O>),
    Apply!(<Self as Kind!(...)>::Of<'a, E>),
>>)
// → m (Pair (self o) (self e))
```

**Edge cases**:
- Deeply nested HKT applications (3+ levels)
- Mixed concrete and HKT types: `Vec<Apply!(<F as Kind!(...)>::Of<'a, A>>)` → `Vec (f a)`

### 5. Type Constructor and Brand Identification

**Requirement**: Distinguish between type variables, type constructors (brands), and concrete types, with special handling for brand types.

**Rules**:

1. **Type variables** (generic without trait bounds or with only impl detail bounds):
   - Appear as uppercase in Rust: `A`, `B`, `M`
   - Convert to lowercase: `a`, `b`, `m`

2. **Type constructors** (generic with type class bounds):
   - Have bounds like `F: Functor`, `M: Applicative`
   - Convert to lowercase: `f`, `m`
   - Include in constraint section

3. **Brand types**:
   - Implement traits from [`fp-library/src/classes.rs`](fp-library/src/classes.rs:1)
   - Represent partially applied or unapplied type constructors
   - Examples: `OptionBrand`, `ResultBrand`, `VecBrand`, `IdentityBrand`
   - **Important**: Output the actual type name, not the brand name
     - `OptionBrand` → `Option`
     - `IdentityBrand` → `Identity`
     - `VecBrand` → `Vec`
   - This requires the macro to understand brand naming conventions (typically `TypeBrand` → `Type`)

4. **Concrete types**:
   - Standard types: `Option`, `Result`, `Vec`, `String`, `i32`, etc.
   - Keep as-is (PascalCase or snake_case)
   - Generic application:
     - `Option<A>` → `Option a`
     - `Result<O, E>` → `Result o e`
     - `Vec<A>` → `Vec a`

**Examples**:

```rust
// Input with brands
fn map<F: Functor, A, B>(f: impl Fn(A) -> B, fa: Apply!(<F as Kind!(...)>::Of<'a, A>)) -> Apply!(<F as Kind!(...)>::Of<'a, B>)

// If F is bound to OptionBrand:
// Output: forall a b. Functor Option => (a -> b, Option a) -> Option b

// Input with explicit brand
fn wither<F: Witherable, M: Applicative, A, B>(...)
where F = OptionBrand, M = VecBrand

// Output uses actual type names
// forall a b. (Witherable Option, Applicative Vec) => ...
```

**Brand detection strategy**:

1. Check if type parameter name ends with `Brand`
2. Strip the `Brand` suffix to get the actual type name
3. Use the actual type name in the generated signature
4. This handles cases like:
   - `OptionBrand` → `Option`
   - `ResultBrand` → `Result`
   - `VecBrand` → `Vec`
   - `IdentityBrand` → `Identity`
   - `PairBrand` → `Pair`

**Edge cases**:
- Custom brands that don't follow `TypeBrand` naming convention
- Brands for types with different names (may need configuration/mapping)
- Type parameters that happen to end in "Brand" but aren't actually brands

### 6. Self/This Handling for Methods

**Requirement**: Convert `Self` in trait methods to lowercase `self`.

**Rules**:

1. In trait definitions, `Self` represents the type implementing the trait
2. Convert to lowercase: `Self` → `self`
3. Keep it simple - always use `self`, regardless of the trait

**Examples**:

```rust
// In Functor trait
trait Functor {
    fn map<A, B>(f: impl Fn(A) -> B, fa: Self::Of<A>) -> Self::Of<B>;
}
// → forall self a b. Functor self => (a -> b, self a) -> self b

// In Witherable trait  
trait Witherable {
    fn wither<M, A, B>(..., ta: Self::Of<A>) -> M::Of<Self::Of<B>>
}
// → forall self m a b. (Witherable self, Applicative m) => ...
```

### 7. Standard Type Application

**Requirement**: Handle standard Rust generic types.

**Common patterns**:

```rust
Option<A>           // → Option a
Result<O, E>        // → Result o e  
Vec<A>              // → Vec a
Pair<A, B>          // → Pair a b
(A, B)              // → (a, b)
(A, B, C)           // → (a, b, c)
Box<A>              // → a (Box is implementation detail)
Arc<A>              // → a (Arc is implementation detail)
Rc<A>               // → a (Rc is implementation detail)
```

**Edge cases**:
- `PhantomData<T>` should probably be omitted
- Smart pointers (Box, Rc, Arc) are implementation details
- Iterator types might need special handling

### 8. Accurate Representation of Function Structure

**Requirement**: The signature should accurately reflect whether the function is curried or uncurried in Rust.

**Rule**: Analyze the actual function definition to determine the structure.

**Uncurried functions** (multiple parameters):
```rust
// Input
fn fold_right(func: impl Fn(A, B) -> B, init: B, fa: F::Of<A>) -> B

// Output (parameters grouped as tuple)
// ((a, b) -> b, b, f a) -> b
```

**Curried functions** (return type is a function):
```rust
// Input  
fn curry<A, B, C>(f: impl Fn(A, B) -> C) -> impl Fn(A) -> impl Fn(B) -> C

// Output (nested arrows)
// ((a, b) -> c) -> a -> b -> c
```

**Implementation notes**:
- Most fp-library functions are uncurried (take multiple parameters at once)
- Should accurately represent the Rust signature structure
- Don't try to convert between styles - represent what's actually there

### 9. Constraint Formatting

**Requirement**: Format type class constraints in Haskell style.

**Rules**:

1. Collect all type class bounds from generics and where clauses
2. Group by type variable
3. Format as `(Constraint1 v1, Constraint2 v2, ...) =>`
4. Order constraints conventionally (structural → behavioral)

**Examples**:

```rust
// Single constraint
where F: Functor
// → Functor f =>

// Multiple constraints on different types
where F: Functor, M: Applicative
// → (Functor f, Applicative m) =>

// Multiple constraints on same type
where F: Functor + Foldable + Traversable
// → (Functor f, Foldable f, Traversable f) =>
// Or: (Traversable f) =>  (if Traversable implies the others)
```

**Advanced**:

```rust
// From par_foldable
where
    FnBrand: SendCloneableFn,
    F: ParFoldable,
    M: Monoid + Send + Sync,

// Output (omitting Send + Sync and FnBrand entirely)
// (ParFoldable f, Monoid m) =>
```

**Edge cases**:
- Super-trait relationships (Applicative implies Functor)
- Whether to show redundant constraints
- Ordering of constraints

### 10. Forall Quantification

**Requirement**: Generate proper universal quantification preserving the order from Rust code.

**Rules**:

1. List all type variables (lowercase)
2. **Preserve the order from the Rust type parameter list**
3. Format as `forall var1 var2 var3.`
4. Keep it simple - don't try to reorder conventionally

**Examples**:

```rust
// Simple - preserve order
fn map<F, A, B>(...)
// → forall f a b. ...

// Preserve declaration order
fn traverse<'a, A, B, F, Func>(...)
where F: Applicative
// → forall a b f. (Applicative f) => ...

// Different order - respect it
fn wilt<'a, M, O, E, A, Self>(...)
where Self: Witherable, M: Applicative
// → forall m o e a self. (Witherable self, Applicative m) => ...
```

**Edge cases**:
- No type parameters: omit forall entirely
- Only concrete types: omit forall
- Single type variable: `forall a.`

### 11. Special Cases and Edge Handling

#### Associated Types

```rust
// Input
where M::Pure<A>: SomeTrait

// If Pure is the HKT application, resolve it
// Otherwise, keep as-is
```

#### Const Generics

```rust
// Input
fn foo<const N: usize, A>(arr: [A; N]) -> A

// Output (omit const generics as impl detail)
// forall a. [a] -> a
```

#### Default Type Parameters

```rust
// Input
fn foo<A, B = i32>(a: A) -> B

// Output (defaults don't appear in HM)
// forall a b. a -> b
```

#### Trait Objects

```rust
// Input
fn foo(f: &dyn Fn(i32) -> i32) -> i32

// Output
// (i32 -> i32) -> i32
```

#### Impl Trait

```rust
// Input
fn foo(f: impl Fn(A) -> B) -> C

// Output
// (a -> b) -> c
```

### 12. Return Type Handling

**Requirement**: Transform return types following the same rules as parameters.

**Examples**:

```rust
// Simple
-> B
// → b

// HKT application
-> Apply!(<F as Kind!(...)>::Of<'a, B>)
// → f b

// Nested
-> Apply!(<M as Kind!(...)>::Of<'a, Apply!(<F as Kind!(...)>::Of<'a, B>>)
// → m (f b)

// Tuple
-> (A, B)
// → (a, b)

// Result as return
-> Result<O, E>
// → Result o e
```

### 13. Parameter Handling

**Requirement**: Transform parameter types and combine into signature, accurately reflecting the function's structure.

**Rules**:

1. Extract type from each parameter
2. Transform according to type rules
3. If the function takes multiple parameters at once: format as tuple `(type1, type2, ...) -> return`
4. If the function is curried (returns a function): nest arrows `type1 -> type2 -> ... -> return`
5. The macro should detect the actual structure from the function definition

**Examples**:

```rust
// Uncurried function (multiple parameters)
fn foo(f: Func, x: A, y: B) -> C
where Func: Fn(A) -> B
// → ((a -> b), a, b) -> c

// Curried function (returns a function)
fn curry<A, B, C>(f: impl Fn(A, B) -> C) -> impl Fn(A) -> impl Fn(B) -> C
// → ((a, b) -> c) -> a -> b -> c

// Function parameter (no extra parens around single arrow)
fn map(f: impl Fn(A) -> B, fa: F::Of<A>) -> F::Of<B>
// → (a -> b, f a) -> f b
```

### 14. Documentation Output

**Requirement**: Generate a simple doc comment line with the signature.

**Rules**:

1. Output a single doc comment line
2. Format: `/// \`generated signature here\``
3. The attribute expands directly to this line in place
4. No section titles, no searching for existing content
5. User controls placement by where they put the attribute

**Examples**:

```rust
// User writes:
/// Maps a function over a structure
#[hm_signature]
pub fn map<F, A, B>(...)

// Expands to:
/// Maps a function over a structure
/// `forall f a b. Functor f => (a -> b, f a) -> f b`
pub fn map<F, A, B>(...)
```

```rust
// User controls placement:
#[hm_signature]
/// More detailed docs here
pub fn traverse<T, F, A, B>(...)

// Expands to:
/// `forall t f a b. (Traversable t, Applicative f) => ...`
/// More detailed docs here
pub fn traverse<T, F, A, B>(...)
```

## Implementation Strategy

### Phase 1: Core Parser
- Parse function/method signatures with `syn`
- Extract type parameters, bounds, parameter types, return type
- Build internal representation

### Phase 2: Type Transformation
- Implement lifetime elimination
- Implement Fn trait → arrow transformation
- Implement HKT macro pattern recognition
- Implement type variable detection
- Implement brand → type name conversion

### Phase 3: Constraint Processing
- Filter bounds (keep vs omit)
- Group and format constraints
- Identify and skip function HKT trait bounds

### Phase 4: Formatting
- Generate forall quantification (preserving order)
- Format constraints
- Format parameters and return type
- Generate final string

### Phase 5: Output Generation
- Generate single doc comment line
- Format with backticks
- Return as TokenStream

## Testing Requirements

### Unit Tests
- Test each transformation rule individually
- Test edge cases for each rule
- Test brand detection and transformation

### Integration Tests
- Test on actual fp-library functions
- Compare generated vs hand-written signatures
- Test on various trait methods

### Snapshot Tests
- Capture output for typical cases
- Detect unintended changes
- Document expected behavior

## Future Enhancements

1. **Type alias expansion**: Expand type aliases to underlying types for clearer signatures (requires type resolution during macro expansion)
2. **Super-trait constraint minimization**: Minimize redundant constraints when super-trait relationships are known
3. **Smart brand detection**: Automatically detect brands by checking for `Kind` trait implementation or specific associated types
4. **IDE Support**: Provide code actions to manually trigger generation
5. **Validation**: Check hand-written signatures against generated ones
6. **Linting**: Warn about inconsistencies between hand-written and generated signatures
7. **LaTeX Output**: Generate signatures for papers/documentation in LaTeX format
8. **Interactive Mode**: CLI tool to experiment with transformations and test edge cases

## Non-Goals

1. **Type inference**: This is not a type checker
2. **Semantic validation**: Doesn't verify correctness of implementations
3. **Cross-language**: Only focuses on Rust → HM transformation
4. **Runtime behavior**: Purely compile-time documentation generation

## Design Decisions

### 1. Type Alias Handling

**Decision**: Keep type aliases as-is in generated signatures

Type aliases will be preserved in the generated signatures without expansion. This is the simplest approach since procedural macros don't have access to type resolution information during macro expansion (which happens before type checking).

**Example**:
```rust
type MyVec<T> = Vec<T>;
fn foo(x: MyVec<i32>) -> MyVec<String>

// Signature preserves alias:
// MyVec i32 -> MyVec String
```

**Future enhancement**: Expand type aliases to their underlying types. This would require either:
- Using compiler internals (complex, unstable)
- External tool integration (e.g., rust-analyzer)
- Build-time type information extraction

### 2. Super-Trait Constraint Handling

**Decision**: Show all constraints initially, with minimization as a future enhancement

The initial implementation will show all trait bounds explicitly, even if they are redundant due to super-trait relationships. This is simpler to implement and safer.

**Current behavior**:
```rust
where F: Functor + Foldable + Traversable
// → (Functor f, Foldable f, Traversable f) =>
```

**Future enhancement**: Minimize to most specific trait only
```rust
where F: Functor + Foldable + Traversable
// → (Traversable f) =>  (since Traversable implies the others)
```

This requires the macro to understand super-trait relationships, which can be:
- Hardcoded for known fp-library traits
- Discovered via trait resolution (complex)

### 3. Custom Derive Macro Handling

**Decision**: Ignore derive macros entirely

Derive macros are orthogonal to type signatures. They generate trait implementations but don't affect the function signature itself. The macro will simply ignore all derive macros.

### 4. Conditional Compilation Handling

**Decision**: Ignore `#[cfg(...)]` attributes and generate signatures as-is

The macro will generate type signatures regardless of conditional compilation attributes. If a function is conditionally compiled away, the signature won't matter. If it exists, the signature should be present.

### 5. Brand Name Resolution

**Decision**: Three-tier fallback strategy with TOML configuration support

Brand types (like `OptionBrand`, `VecBrand`) will be resolved to their actual type names using a three-tier approach:

**Resolution strategy**:
1. **First**: Check configuration file for custom mapping
2. **Second**: Apply convention-based transformation (strip `Brand` suffix)
3. **Third**: Use full name as-is if neither works

**Configuration support**:

Configuration will be read from the `Cargo.toml` metadata section. This is the standard location for tool-specific configuration in Rust projects.

**Configuration location**: `Cargo.toml` in the project root (accessible via `CARGO_MANIFEST_DIR` environment variable)

**Format**:
```toml
# In Cargo.toml
[package.metadata.hm_signature]
brand_mappings = { CustomBrand = "Custom", SpecialTypeBrand = "SpecialType", LegacyBrand = "Legacy" }
```

**Examples**:
```rust
// With config mapping:
// brand_mappings: { SpecialBrand = "Special" }
SpecialBrand → Special  (from config)

// Convention-based (no config needed):
OptionBrand → Option    (strip "Brand")
VecBrand → Vec          (strip "Brand")

// No match (use as-is):
MyCustomType → MyCustomType  (full name preserved)
```

**Future enhancement**: Smart detection by checking if type implements `Kind` trait or has specific associated types, eliminating the need for manual configuration in most cases.
