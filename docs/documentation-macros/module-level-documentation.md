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
- Replaces the manual `#[document_impl]` macro, automatically applying the documentation logic to methods annotated with `#[hm_signature]` or `#[doc_type_params]`, using a hierarchical configuration system to resolve ambiguities.

## Scope and Responsibilities

### In Scope

The `document_module` macro is responsible for:

- **Parsing and extraction**: Reading `impl_kind!` invocations and trait `impl` blocks to extract type mappings
- **Documentation generation**: Generating HM signatures and type parameter documentation for methods with appropriate attributes
- **Syntactic transformation**: Resolving `Self` and `Self::AssocType` references based on extracted mappings
- **Error reporting**: Providing actionable compile-time errors for missing mappings or ambiguities

### Out of Scope

The `document_module` macro is **not responsible for**:

- **Semantic validation**: Verification of types, traits, or bounds correctness
- **Bound compatibility checking**: Validation that bounds in trait impls are compatible with bounds in `impl_kind!`
- **Type resolution**: Understanding or verifying type relationships
- **Lifetime validation**: Checking lifetime relationships or constraints

**Rationale**: These responsibilities belong to rustc's type checker. The macro operates purely on syntactic structures to extract and transform documentation. Any semantic errors will be caught during normal compilation.

**Implication**: The macro may generate documentation for code that rustc will later reject. This is acceptable - the build will still fail at the rustc stage, providing the user with standard compiler error messages. The macro focuses on documentation generation for valid code.

## Functional Requirements

### 1. Context Extraction (Full Projection & Impl-Scanning)

**Requirement**: The macro must extract a comprehensive mapping of `(Brand, Trait, AssociatedType, Generics) -> TargetType` from two sources. This extra detail is required to perform correct parameter substitution during resolution (e.g., mapping `Self::Of<X, Y>` to `Result<X, Y>` requires knowing that `Of<A, B> = Result<A, B>`).

#### 1.1 Extraction Sources

The macro scans for associated type definitions in the following locations:

**Included**:

1.  **`impl_kind!` Invocations** (top-level in the module):

    ```rust
    impl_kind! { for MyBrand { type Of<A> = Box<A>; } }
    ```

    Extracts: `(MyBrand, None, Of<A>)` -> `Box<A>`.

    The `Trait` is `None` because `impl_kind!` defines global defaults not tied to a specific trait.

2.  **Standard `impl` Blocks** (top-level in the module):
    ```rust
    impl Pointer for ArcBrand {
        type Of<T> = Arc<T>;
    }
    ```
    Extracts: `(ArcBrand, Some(Pointer), Of<T>)` -> `Arc<T>`.

**Excluded**:

- Function/method bodies
- Nested modules (without their own `#![document_module]`)
- **Trait definitions** (including default associated types and associated type declarations)
  - Note: While trait _declarations_ (`type Of<T>;`) are not extracted for mappings, they are recognized for documentation purposes. However, default associated type _definitions_ in traits (`type Of<T> = Vec<T>;`) are excluded. Users must explicitly override these in impl blocks.
- Associated constants
- Macro-generated code (not visible to the macro - see §7)

The macro aggregates these findings into a module-wide configuration.

#### 1.2 Multiple `impl_kind!` Blocks

**Behavior**: Multiple `impl_kind!` blocks for the same Brand in the same module are permitted and will be merged.

**Collision Handling**: If the same `(Brand, AssocName)` pair appears multiple times:

- If the `TargetType` (after positional generic substitution) is structurally identical, treat as the same definition (no error).
- If the `TargetType` differs, emit a compile-time error indicating conflicting definitions.

**Rationale**: Allows organizing large type definitions across multiple blocks for readability while catching genuine errors. No consolidation warning is issued as this pattern may be intentional.

**Example**:

```rust
// ✅ Valid: Non-conflicting definitions
impl_kind! {
    for MyBrand {
        type Of<A> = Box<A>;
    }
}

impl_kind! {
    for MyBrand {
        type SendOf<A> = Arc<A>;
    }
}

// ❌ Error: Conflicting definitions for same associated type
impl_kind! {
    for MyBrand {
        type Of<A> = Box<A>;
    }
}

impl_kind! {
    for MyBrand {
        type Of<A> = Rc<A>;  // Error: Of already defined with different target
    }
}
```

#### 1.3 Multiple Definitions of Same Associated Type

**Behavior**: When multiple `impl` blocks define the same associated type name for the same type, the macro **does not** perform collision detection or structural equivalence checking.

**Rationale**:

- **Validation is not the macro's responsibility**: The macro generates documentation, not validation. Rustc will catch any genuine type conflicts.
- **Simplicity**: No need for complex structural comparison logic.
- **Type aliases**: Avoiding equivalence checking allows type aliases to work seamlessly (§1.8).

**Each definition is tracked independently** and used for documentation generation in its respective context (the specific `impl Trait for Type` block where it appears).

**Example**:

```rust
trait Base {
    type Of<T>;
}

trait Derived: Base {
    // Inherits Of<T>
}

impl Base for MyBrand {
    type Of<T> = Box<T>;
}

impl Derived for MyBrand {
    type Of<T> = Box<T>;  // ✅ No conflict checking performed
    // Both definitions tracked independently
}

// This also works with type aliases:
impl SomeTrait for MyBrand {
    type Of<T> = MyAlias<T>;  // ✅ Tracked separately, no equivalence check
}
```

**Note**: If there are genuine conflicts (e.g., incompatible trait implementations), rustc will report them during compilation. The documentation macro focuses solely on extracting and documenting the definitions as written.

#### 1.4 Generics Handling

The extraction logic must use **Positional Mapping** to map generic parameters correctly.

- **Capture Full Context**: The mapping must be `(Brand, Trait, AssocName, AssocGenerics) -> TargetType`. The `TargetType` may refer to generics from the `impl` block.
- **Positional Matching**: Generic parameters are matched by position, not name, between `impl_kind!` and trait impls. This allows renamed generics while ensuring correct substitution.
- **Scope Awareness**: When substituting, ensure that `impl` generics are treated as "constants" in the scope, while associated type generics are substituted.
- **Lifetimes and Const Generics**: These are **erased** in HM signature documentation to reduce noise and maintain mathematical notation simplicity.
- **Bounds on Associated Type Parameters**: Bounds in associated type definitions (e.g., `type Of<T: Clone>`) are parsed but **erased** for projection purposes. The projection map stores only the type parameter names, not their bounds. This is consistent with HM signature erasure (§1.7) and simplifies the implementation.

**Example of Positional Matching**:

```rust
// In impl_kind!:
impl_kind! {
    impl<Config: LazyConfig> for LazyBrand<Config> {
        type Of<T> = Lazy<T, Config>;
    }
}

// In trait impl (renamed generic):
impl<C: LazyConfig + Send> Functor for LazyBrand<C> {
    fn map<A, B>(...) -> Apply!(<Self as Kind!(type Of<T>;)>::Of<B>) {
        // Resolution: Apply!(<Self as Kind!(type Of<T>;)>::Of<B>) -> Lazy<B, C>
        // Position 0: Config/C matched by position, uses C (the trait impl's name)
    }
}
```

**Parametric Brand Matching**: When matching parametric brands (e.g., `LazyBrand<Config>` against `LazyBrand<C>`), the macro uses structural matching:

1. Extract the base type path (e.g., `LazyBrand`)
2. Match generic parameters positionally (position 0: `Config` ↔ `C`)
3. Use the trait impl's generic names in the substitution (e.g., use `C`, not `Config`)

**Example**:

```rust
// impl_kind! defines: LazyBrand<Config: LazyConfig> with Of<T> = Lazy<T, Config>
// Trait impl uses: LazyBrand<C: LazyConfig + Send>
// Match: Position 0 maps Config ↔ C
// Substitution: Use C in the resolved type: Apply!(<Self as Kind!(type Of<T>;)>::Of<B>) → Lazy<B, C>
```

**Const Generics Handling**: Const generics in associated types are identified during parsing but **erased** from HM signature documentation. Array and const generic expressions are simplified to their base type structure.

**Erasure Rules**:

- `[T; N]` → `[T]` (array with const generic size becomes simple array of T)
- `[T; N * 2]` → `[T]` (complex const expressions erased)
- `[(T, U); N + M]` → `[(T, U)]` (tuple arrays simplified)
- `[T; 42]` → `[T]` (even concrete sizes erased for consistency)

**Rationale**:

- **HM signature simplicity**: Const generics are implementation details, not core to the mathematical type signature
- **Consistency**: All array sizes are treated uniformly
- **Rust signature remains**: The full Rust signature (visible above HM signature) shows all const generics

**Example**:

```rust
impl_kind! {
    for ArrayBrand {
        type Of<const N: usize, T> = [T; N];
    }
}

impl Functor for ArrayBrand {
    // Rust signature (preserved as-is):
    fn map<const N: usize, A, B>(arr: [A; N], f: impl Fn(A) -> B) -> [B; N]

    // HM signature generated (const generics erased):
    // forall A B. [A] -> (A -> B) -> [B]
}

// Complex expressions also simplified:
type Of<const N: usize, const M: usize, T> = [(T, T); N + M];
// HM signature shows: [(T, T)]
```

**Positional Matching**: During resolution, const generics are matched by position for substitution, but the final HM signature erases them:

```rust
fn foo<const X: usize, const Y: usize, A, B>() -> Apply!(<Self as Kind!(type Of<const N: usize, const M: usize, T>;)>::Of<X, Y, B>)
// Resolution: X→N (position 0 const), Y→M (position 1 const), B→T (position 0 type)
// Target: [(B, B); X + Y]
// HM signature: forall A B. [(B, B)]  (const generics erased)
```

#### 1.5 Where Clause Handling

The extraction logic must robustly parse `where` clauses on associated types (e.g., `type Of<A> = Foo<A> where A: Clone;`) but **erase** them for the purpose of building the projection map. The `where` clause is relevant for code validity but not for the structural mapping required for documentation.

#### 1.6 Cfg Handling

**Behavior**: The macro extracts from **all** `cfg` branches independently, treating each as a separate definition context. No conflict detection is performed across different `cfg` conditions.

**Rationale**:

- **Validation is not the macro's responsibility**: The macro is a documentation generator, not a validator. Rustc will catch any actual conflicts or invalid configurations.
- **Complete documentation**: Extracting all branches ensures documentation covers all possible feature configurations.
- **Simplicity**: No boolean logic analysis or mutual exclusivity detection is needed.

**Documentation Generation**: Where possible, the macro adds `#[doc(cfg(...))]` attributes to indicate feature requirements in the generated documentation.

**Example**:

```rust
#[cfg(feature = "sync")]
impl_kind! { for Brand { type Of<T> = Arc<T>; } }

#[cfg(not(feature = "sync"))]
impl_kind! { for Brand { type Of<T> = Rc<T>; } }

// ✅ Both extracted independently
// ✅ No conflict checking (even though both define Of<T>)
// Documentation shows both variants with cfg annotations where possible
```

**Implication**: If cfg branches contain genuinely conflicting definitions that could be active simultaneously, rustc will catch the error during compilation. The documentation macro focuses solely on generating complete documentation across all configurations.

#### 1.7 Unsized Types and Higher-Ranked Trait Bounds

**Decision**: Erase to simple representation in HM signatures.

**Rationale**: HM signatures are meant to be clean, mathematical abstractions. Full Rust type information (with all markers and bounds) is still available in the actual function signature documentation.

**Rules**:

- `T: ?Sized` → `T` (erase unsized marker)
- `for<'a> Fn(&'a T)` → Higher-ranked trait bounds erased (see function type handling below)
- `Self::Of<T: Clone>` → `Self::Of T` (erase bounds from type application)

**Function Types and Currying**:

The macro follows the rules defined in [`docs/documentation-macros/function-parameter-documentation.md`](function-parameter-documentation.md):

- **Uncurried functions** (taking tuples): Documented with tuple syntax
  - `fn foo(x: (A, B)) -> C` → `(A, B) -> C`
- **Curried functions** (returning `Fn`): Documented with arrow chains
  - `fn foo(x: A) -> impl Fn(B) -> C` → `A -> B -> C`

**Detection**: A function is "curried" if its return type is a function trait (`Fn`, `FnMut`, `FnOnce`) or HKT function trait (`CloneableFn`, `SendCloneableFn`, `Function`).

**HRTB Handling**: Higher-ranked trait bounds (`for<'a>`) are erased:

- `fn foo<T>() -> impl for<'a> Fn(&'a T) -> R` → `forall T. &T -> R`

**Lifetimes**: Completely erased from HM signatures (§1.11).

**Trait Objects**: Trait objects (`dyn Trait`) are preserved in HM signatures to indicate dynamic dispatch, but auto traits are erased:

- `fn foo() -> Box<dyn Iterator<Item = i32>>` → `forall. () -> Box (dyn Iterator (Item = i32))`
- `fn bar() -> Box<dyn Trait + Send + 'static>` → `forall. () -> Box (dyn Trait)` (Send and lifetime erased)

**Rationale**: Preserving `dyn` maintains semantic accuracy about dynamic dispatch. Auto traits (`Send`, `Sync`) and lifetimes are erased consistent with other erasure rules (§1.7, §1.11).

**Implementation Note**: Current `hm_signature.rs` already implements this correctly. The logic should be reused/adapted for `document_module`.

**Generic Constraints in Method Signatures**:

Method-level trait bounds are shown as constraints using Haskell-style syntax:

```rust
// Rust signature:
fn foo<T: Clone + Debug>(x: T) -> Apply!(<Self as Kind!(type Of<T>;)>::Of<T>)

// HM signature:
// forall T. [Clone T, Debug T] => T -> Self::Of T
```

**Format**: `[Constraint1, Constraint2, ...] =>`

**Note**: This differs from associated type bounds (§1.4), which are erased because they're part of the type definition. Method constraints are part of the function signature and are therefore documented.

**Implementation Note**: Current `hm_signature` macro already generates this format. Reuse that logic.

**Example**:

```rust
// Rust signature:
fn foo<T: ?Sized>(x: &T) -> impl for<'a> Fn(&'a T) -> String { ... }

// Generated HM signature:
// forall T. &T -> (&T -> String)
```

#### 1.8 Type Aliases

**Decision**: No alias resolution. No structural equivalence checking.

**Behavior**: Type aliases are preserved as-is in mappings and HM signatures. The macro does not attempt to resolve aliases or check if different type paths refer to the same underlying type.

**Rationale**:

- **Semantic intent**: Users write aliases for semantic reasons (e.g., `type UserId = String`). Preserving aliases maintains that intent.
- **Simplicity**: No need for complex type resolution logic.
- **Consistency**: Aligns with the "no semantic analysis" principle (§Scope and Responsibilities).

**Multiple definitions**: If the same associated type is defined with different paths (e.g., `Vec<T>` vs `MyAlias<T>` where `MyAlias<T> = Vec<T>`), they are treated as **distinct definitions** and tracked independently. No conflict checking is performed.

**Example**:

```rust
pub type MyAlias<T> = Vec<T>;

impl_kind! {
    for MyBrand {
        type Of<T> = MyAlias<T>;  // Preserved as MyAlias, not resolved to Vec
    }
}

impl SomeTrait for MyBrand {
    type Of<T> = Vec<T>;  // ✅ Tracked separately, no equivalence check
}

// Both definitions coexist; rustc validates consistency
```

#### 1.9 Self References and Forward References

**Decision**: Different rules apply based on context.

**In Associated Type Definitions** (RHS of `type Of<T> = ...`):

**`Self::` references are forbidden**:

- Emit compile-time error if `Self::` appears in the RHS
- **Rationale**: Prevents circular resolution during documentation generation

**Forward references to other associated types are allowed**:

- `type A<T> = B<T>` where `B<T>` is defined later is valid
- The macro preserves these references as-is without resolution
- **Rationale**: These are just type names; rustc validates their correctness

**Example**:

```rust
impl SomeTrait for MyBrand {
    type A<T> = B<T>;                    // ✅ Forward reference OK
    type B<T> = Vec<T>;
    type C<T> = (A<T>, B<T>);            // ✅ Multiple references OK
    type Bad<T> = Box<Self::Other<T>>;   // ❌ Error: Self:: forbidden
    type Other<T> = Vec<T>;
}
```

**In Method Signatures**:

- Nested `Self::` references are **allowed**
- Resolution is iterative (up to syntactic validity)
- No circular reference detection or depth limits
- **User Responsibility**: User must ensure types are valid; rustc will catch errors

**Example**:

```rust
// ✅ Valid: Nested Self:: in method signature
impl Monad for MyBrand {
    fn join(mma: Apply!(<Self as Kind!(type Of<T>;)>::Of<Apply!(<Self as Kind!(type Of<T>;)>::Of<T>)>)) -> Apply!(<Self as Kind!(type Of<T>;)>::Of<T>) {
        // If Of<T> = Box<T>, this resolves iteratively to:
        // Box<Box<T>> -> Box<T>
        // Perfectly valid
    }
}
```

**Rationale**:

- **Associated type definitions**: Forbidding `Self::` prevents circular resolution during documentation generation
- **Forward references**: No resolution needed; just preserve the type name
- **Method signatures**: Nested `Self::` is common and necessary (e.g., monad operations); rustc validates correctness

#### 1.10 HKT Macro Resolution

**Supported Macros**: The macro recognizes and traverses inside certain HKT-related macros to resolve `Self` references:

- `Apply!` - Always recognized
- `Kind!` - Always recognized (but treated as transparent for documentation)
- Additional aliases as implemented in existing macros

**Kind! Macro Transparency**: The `Kind!` macro is used for rustc's trait system resolution but is **transparent** to the documentation macro. Only the inner associated type reference and the final `Apply!` parameters are processed:

```rust
Apply!(<Self as Kind!(type Of<T>;)>::Of<B>)
//              ^^^^^^^^^^^^^^^^^^^^^^ For rustc trait resolution only
//                                  ^^ Actual parameters used for documentation
```

The generic parameters in `Kind!(type Of<...>)` are NOT validated by the documentation macro. Only the parameters in the final application (`::Of<B>`) are used for positional matching and resolution.

**Resolution Process**:

1. Identify macro invocation by name matching
2. Parse macro arguments (may contain `Self::` references)
3. Recursively resolve `Self::` in arguments
4. Reconstruct macro invocation with resolved types

**Example**:

```rust
impl Functor for MyBrand {
    fn map<A, B>(x: Apply!(<Self as Kind!(type Of<T>;)>::Of<A>)) -> Apply!(<Self as Kind!(type Of<T>;)>::Of<B>) {
        // Apply!(...) is traversed
        // Self resolved to MyBrand
        // Kind!(type Of<T>;) is transparent—only ::Of<B> matters
        // Result: Box<B> (if Of<T> = Box<T>)
    }
}
```

**Implementation Note**: Current `hm_signature` and `document_impl` macros already handle this correctly. The implementation should reuse/adapt that logic from `fp-macros/src/hm_signature.rs` and related files.

**Limitation**: Only recognized macros are traversed. Unknown macros are treated as opaque types.

#### 1.11 Lifetime Parameter Handling

**Decision**: Lifetimes are **completely erased** from HM signature documentation.

**Rationale**:

- HM signatures represent mathematical type relationships
- Lifetimes are Rust-specific implementation details
- The actual Rust signature (visible above the HM signature) shows all lifetimes
- Erasing lifetimes produces cleaner, more readable mathematical notation

**Extraction**:

- Lifetimes in associated type parameters are not included in projection map keys
- Lifetimes in target types are present but ignored for matching purposes
- During HM signature generation, lifetimes are completely erased

**Example**:

```rust
impl_kind! {
    for RefBrand {
        type Of<'a, T> = &'a T;
    }
}

impl Functor for RefBrand {
    // Rust signature (preserved as-is in docs):
    fn map<'a, A, B>(x: &'a A, f: impl Fn(A) -> B) -> &'a B

    // HM signature generated (lifetimes erased):
    // forall A B. &A -> (A -> B) -> &B
}
```

**Const Generics**: Similarly erased from HM signatures (though used for positional matching - see §1.4).

**Validation**: The macro does NOT validate lifetime relationships. Any lifetime errors will be caught by rustc during compilation.

### 2. Hierarchical Configuration

**Requirement**: To resolve the concrete type of `Self` (when used bare, e.g., `fn foo(self)`), the macro must support a precedence hierarchy:

1.  **Method Override**: `#[doc_use = "AssocName"]` on the method.
2.  **Impl Block Override**: `#[doc_use = "AssocName"]` on the `impl` block.
3.  **(Type, Trait)-Scoped Default**: `#[doc_default]` on the associated type definition in a trait `impl` block (applies only to that specific `impl Trait for Type` block).
4.  **Module Default**: `#[doc_default]` on the associated type definition in `impl_kind!` (applies globally within the module).
5.  **Fallback**: Error.

#### 2.1 (Type, Trait)-Scoped Defaults

**Decision**: `#[doc_default]` in a trait `impl` is **(Type, Trait)-scoped** and only applies to methods of that specific impl block.

**Clarification**: Each `impl Trait for Brand` block is an independent scope. A default marked in `impl Base for Brand` does **not** automatically apply to `impl Derived for Brand`, even if `Derived: Base`. Methods use the default from the impl block they are defined in.

**Rationale**:

- Types like `ArcBrand` implement multiple traits (`Pointer`, `RefCountedPointer`, `SendRefCountedPointer`) with different associated types (`Of`, `CloneableOf`, `SendOf`).
- Each trait's methods may naturally use a different associated type.
- (Type, Trait)-scoped defaults allow setting appropriate defaults per trait without requiring method-level annotations.
- Maintains explicitness while providing ergonomic defaults where semantically appropriate.
- The macro cannot reliably resolve trait hierarchies across module boundaries, so it treats each impl block independently.

**Resolution Algorithm**:
When resolving bare `Self` in a method within `impl Trait for Brand`:

1. Check for method-level `#[doc_use]`
2. Check for impl-block-level `#[doc_use]`
3. Check for (Type, Trait)-scoped default: `#[doc_default]` in this `impl Trait for Type` block
4. Check for module-level default: `#[doc_default]` in `impl_kind!`
5. Error if no default found

**Example**:

```rust
impl_kind! {
    for ArcBrand {
        #[doc_default]  // Global module default
        type Of<T> = Arc<T>;
    }
}

impl Pointer for ArcBrand {
    type Of<T> = Arc<T>;

    fn new<T>(value: T) -> Self {  // Uses global default: Arc<T> via Of
        Arc::new(value)
    }
}

impl RefCountedPointer for ArcBrand {
    #[doc_default]  // (Type, Trait)-scoped default for (ArcBrand, RefCountedPointer)
    type CloneableOf<T: ?Sized> = Arc<T>;

    fn new<T>(value: T) -> Self {  // Uses (Type,Trait) default: Arc<T> via CloneableOf
        Arc::new(value)
    }
}

impl SendRefCountedPointer for ArcBrand {
    type SendOf<T: ?Sized + Send + Sync> = Arc<T>;

    #[doc_use = "SendOf"]  // Explicit override needed (no default marked)
    fn send_new<T: Send + Sync>(value: T) -> Self {
        Arc::new(value)
    }
}
```

#### 2.2 Conflict Resolution

**Within Impl Block Scope**: If multiple associated types in the same `impl Trait for Type` block are marked `#[doc_default]`, emit a compile-time error.

**Across Impl Blocks**: Different impl blocks can have different defaults without conflict ((Type, Trait)-scoped resolution).

**Rationale**: Explicit is better than implicit. Ambiguities must be resolved by the user.

### 3. Automatic Documentation Application

**Requirement**: Documentation generation is **opt-in** via attributes on methods:

- **HM Signature Generation**:
  - The macro scans for `#[hm_signature]` invocations within `impl` blocks.
  - It replaces the `#[hm_signature]` attribute invocation in-place with the generated type signature documentation.
  - **Opt-in**: If `#[hm_signature]` is not present, no signature is generated.
- **Doc Type Params**:
  - Support `#[doc_type_params]` to document generic parameters.
  - Replaces the `#[doc_type_params]` attribute invocation in-place with generated documentation.
  - Only processes methods/functions that have this attribute.
  - Ensures feature parity with the previous implementation.

**Opt-In Nature**: Documentation generation **only occurs** for methods with `#[hm_signature]` or `#[doc_type_params]` attributes. This means:

- Visibility modifiers (`pub`, `pub(crate)`, private) are irrelevant for triggering generation
- `#[doc(hidden)]` items can still have documentation generated if attributes are present
- The macro processes all items uniformly; the attributes control what gets generated
- No documentation is generated for items without these attributes

**Rationale**: Since documentation is opt-in via explicit attributes, there's no need to skip items based on visibility or `doc(hidden)`. Users control exactly which methods receive generated documentation by applying the attributes. This provides maximum flexibility - users can document private helper methods for internal reference if desired.

**Documentation Placement**: Generated documentation is placed exactly where the documentation attribute (`#[hm_signature]`, `#[doc_type_params]`) appears in the source code. The attribute invocation itself is replaced with the generated doc comment. This matches the current behavior of the standalone `#[document_impl]` macro.

**Attribute Placement Requirements**:

Documentation attributes (`#[hm_signature]`, `#[doc_type_params]`) must be placed locations that are valid for documentation blocks.

**Valid**:

```rust
/// Description
///
/// ### Type Signature
///
#[hm_signature]
///
/// ### Parameters
fn foo() { ... }
```

```rust
/// Description
///
/// ### Type Signature
///
#[hm_signature]
fn foo() { ... }
```

```rust
#[hm_signature]
fn foo() { ... }
```

**Invalid**:

```rust
/// Description
fn
#[hm_signature]  // ❌ Wrong: Not a valid location for documentation comments.
foo() { ... }
```

The macro processes the item's attributes and doc comments as a sequence, replacing the `#[hm_signature]` or `#[doc_type_params]` attribute with generated documentation content.

**Rationale**: This placement allows precise control over where generated content appears in the final documentation structure. This is the current behavior of `#[document_impl]`, preserved for compatibility.

**Error Handling**: If attributes are placed outside doc comments (standard Rust position), emit a compile-time error with a suggestion to move them (see Error Message Requirements section).

**Multiple Attributes on One Method**: A method may have both `#[hm_signature]` and `#[doc_type_params]` if documentation for both aspects is desired.

- Each attribute is processed independently
- Each generates its respective documentation section
- Attributes are replaced in source order
- This is the current behavior of `document_impl`; preserved for compatibility

**Multiple `#[doc_use]` Attributes**: Only one `#[doc_use]` attribute is allowed per item (method or impl block). Multiple `#[doc_use]` attributes result in a compile-time error:

```rust
#[doc_use = "Of"]
#[doc_use = "SendOf"]  // ❌ Error: Multiple #[doc_use] attributes
fn foo() -> Self { ... }
```

**Exception**: Multiple attributes from `cfg_attr` that are mutually exclusive (different cfg conditions) are allowed:

```rust
#[cfg_attr(unix, doc_use = "UnixOf")]
#[cfg_attr(windows, doc_use = "WindowsOf")]
fn foo() -> Self { ... }  // ✅ OK: only one active per configuration
```

**Rationale**: Multiple overrides on the same item indicate configuration error. Explicit is better than implicit.

**Example**:

```rust
/// Description
///
/// ### Type Signature
///
#[hm_signature]
///
/// ### Type Parameters
///
#[doc_type_params]
///
/// ### Examples
fn foo<T>(...) { ... }
```

Both sections are generated, appearing where their attributes were placed.

**Example**:

```rust
impl Functor for MyBrand {
    /// Applies a function to the value inside.
    ///
    /// ### Type Signature
    ///
    #[hm_signature]  // ← This attribute is replaced in-place
    ///
    /// ### Parameters
    ///
    /// - `f`: The function to apply
    ///
    fn map<A, B>(self, f: impl Fn(A) -> B) -> Apply!(<Self as Kind!(type Of<T>;)>::Of<B>) { ... }
}
```

After macro expansion, the `#[hm_signature]` line is replaced with the generated HM signature documentation.

**Self Resolution**:
For methods with documentation attributes, resolve `Self` usage:

- **Path/Projected** (`Self::Assoc`): Map using the Context Extraction table (e.g., `Self::SendOf` -> `Arc`).
- **Bare** (`self`, `Self`): Map using the Hierarchical Default (following precedence rules in §2).
- **Nested Self References**: The macro supports nested `Self::` references (e.g., `Self::Of<Self::Of<T>>`) through iterative resolution. The resolution continues until no `Self::` remains (see §1.9 for circular reference handling).
- **Apply! Macro**: Explicitly traverse and resolve `Apply!` and `Kind!` invocations, substituting `Self` within them (§1.10).

**Associated Type Visibility**: The macro extracts and tracks **all** associated types regardless of visibility (`pub`, `pub(crate)`, private). This allows:

- Private helper methods to use `#[hm_signature]` for internal documentation
- Complete projection maps for all impl blocks
- Consistent behavior with opt-in principle

**Rationale**: Associated type visibility is a rustc concern, not a documentation concern. Since documentation generation is opt-in via explicit attributes (§3), visibility modifiers are irrelevant. Private types in impl blocks are accessible to methods in that block. Opt-in design ensures zero overhead for undocumented methods and makes the macro's impact explicit.

### 4. Regression Testing & Behavior Parity

**Requirement**: The implementation of `#[document_module]` must preserve all existing behaviors and edge cases currently handled by the standalone macros.

1.  **Test Preservation**: Existing tests from `fp-macros/src/hm_signature.rs`, `fp-macros/src/document_impl.rs`, and `fp-macros/src/doc_type_params.rs` must be adapted and maintained. They serve as the baseline for correct signature generation and documentation formatting.
2.  **Parity Verification**: The new macro must produce identical (or improved, where explicitly intended) documentation output for all scenarios covered by these tests, including:
    - Complex `Self` substitution and associated type resolution.
    - Integration with `Apply!` and `Kind!` macros.
    - HM signature formatting (forall, constraints, arrows).
    - Positional mapping of type parameters.

## Design Decisions

### 1. Update `impl_kind!` Parser

**Decision**: Modify the `impl_kind!` macro parser to accept and parse attributes (like `#[doc_default]`) on associated type definitions.

**Rationale**:

- **Enabling Configuration**: The current `impl_kind!` parser does not support attributes on associated types. To support "Module Default" configuration via `#[doc_default]` inside `impl_kind!`, the parser must be updated.
- **Standard Compliance**: This aligns `impl_kind!` syntax closer to standard Rust `impl` blocks, where attributes on associated types are valid.
- **Implementation**: The macro will parse these attributes to make them available for `document_module` extraction, but will strip them from the generated output to avoid "unused attribute" warnings.

**Architecture Clarification**: Due to macro expansion order (§7), `document_module` sees the `impl_kind! { ... }` invocation tokens directly, NOT the expanded code. Therefore, both macros share the same parser (`ImplKindInput` from `fp-macros/src/impl_kind.rs`):

- `impl_kind!` parses attributes, ignores documentation attributes during expansion, generates code
- `document_module` parses the same invocation tokens, uses documentation attributes for configuration, generates docs
- Neither depends on the other's expansion
- This shared parsing ensures consistency and reduces duplication

### 2. Syntax: `#[doc_default]` and `#[doc_use]`

**Decision**: Split configuration into two attributes:

- **Inside `impl_kind!` / `impl`**: `#[doc_default]` (marker) on a `type` definition sets the default for that Brand within the impl block's scope (or globally if in `impl_kind!`).
- **On `impl` / `fn`**: `#[doc_use = "AssocName"]` overrides the default to use the target of `AssocName`.

**Rationale**:

- **Clarity**: Separating the definition of a default (`doc_default`) from the usage/selection (`doc_use`) reduces confusion and makes the intent explicit.
- **Consistency**: Explicitly naming the associated type (`AssocName`) ensures that the override points to a valid, defined type mapping.
- **(Type, Trait) Scoping**: Placing `#[doc_default]` in trait impls enables (Type, Trait)-scoped defaults (also called "impl-block-scoped" - specific to one `impl Trait for Type` block) without polluting the global namespace.

### 3. Macro Type: Attribute Macro

**Decision**: `#[proc_macro_attribute]` applied to the module (`mod` item).

**Rationale**:

- **Full Visibility**: Attribute macros on modules receive the entire module content (including `impl_kind!`, `impl` blocks, and manual trait impls) as a single token stream. This allows comprehensive context extraction before processing items.
- **Zero Boilerplate**: Users only need to add one line (`#![document_module]`) per file, achieving the primary goal of reducing annotation noise.

### 4. Logic Reuse & Deprecation

**Decision**: Refactor `fp-macros/src/document_impl.rs` and `hm_signature.rs` to accept a rich `Config` object containing the projection map and defaults. The standalone `#[document_impl]` macro will be **deleted immediately**.

**Rationale**:

- **Maintainability**: The core logic for generating HM signatures (analyzing generics, formatting bounds) is complex. Duplicating it for the new macro would lead to bugs and drift.
- **Simplification**: The `#[document_module]` macro subsumes the functionality of `#[document_impl]`. Removing the standalone macro simplifies the codebase and removes the need to maintain `Cargo.toml` configuration.
- **Breaking Change Acceptable**: The library is pre-1.0 and under active development. Immediate deletion with clear changelog documentation is acceptable. The migration is straightforward: replace per-impl `#[document_impl]` with module-level `#![document_module]`.
- **Trade-off**: Previous `Cargo.toml` metadata (including any comments) will be lost, but the new system is self-documenting through source code, eliminating the synchronization problem.

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

### 7. Macro Expansion Order and Generated Code Limitations

**Decision**: Explicitly document that the macro **cannot** see `impl` blocks generated by other macros (except those visible before `document_module` expansion).

**Rationale**:

- **Technical Limitation**: Attribute macros run before the expansion of macros inside them. `document_module` will see the macro invocation (e.g., `my_macro! { ... }`), not the generated code.
- **User Action**: Users must manually configure documentation for macro-generated `impl` blocks using explicit overrides, or the generating macro must be updated to support documentation generation directly.

**Examples**:

```rust
#![document_module]

// ✅ Works: Direct impl_kind! invocation
impl_kind! {
    for MyBrand {
        type Of<A> = Box<A>;
    }
}

// ❌ Doesn't Work: impl_kind! via macro_rules!
macro_rules! generate_impl_kind {
    ($brand:ty, $target:ty) => {
        impl_kind! { for $brand { type Of<A> = $target<A>; } }
    };
}
generate_impl_kind!(MyBrand, Box);
// The macro sees the invocation, not the generated impl_kind!

// ⚠️ Undefined: Derive-generated impls
// Behavior depends on derive macro implementation and expansion order
#[derive(Kind)]
struct MyType;

// ❌ Doesn't Work: Included impls from build.rs
// The macro cannot see files included via include!()
include!(concat!(env!("OUT_DIR"), "/generated_impls.rs"));

// ✅ Workaround: Manual override for generated code
#[doc_use = "Of"]
impl SomeTrait for GeneratedBrand {
    // Methods can still use documentation attributes
    #[hm_signature]
    fn method(&self) -> Self { ... }
}
```

### 8. Coupling to `impl_kind!`

**Decision**: Share the parsing logic (the `ImplKindInput` struct in `fp-macros/src/impl_kind.rs`) between both macros.

**Rationale**:

- **Maintenance**: `document_module` must parse `impl_kind!` to extract context. Sharing the parser ensures that if `impl_kind!` syntax changes, `document_module` stays in sync automatically, preventing breakage.

### 9. Error Handling Strategy

**Decision**: Use **comprehensive error collection** in both passes.

**Pass 1 - Context Extraction**: The macro accumulates all errors encountered during context extraction. If Pass 1 has any errors, all errors are reported together and compilation fails immediately without proceeding to Pass 2.

**Pass 2 - Documentation Generation**: If Pass 1 succeeds, the macro proceeds to Pass 2 and continues processing all methods even if some fail:

1. Documentation generation continues for all remaining methods
2. All errors are accumulated
3. All errors are reported together at the end
4. Compilation fails (no partial documentation generated)

**Rationale**:

- **Pass 1**: Missing type mappings or ambiguous defaults would cause cascade failures in Pass 2. Stopping early provides clean error messages.
- **Pass 2**: Seeing all documentation generation errors at once allows fixing multiple issues in one iteration.
- **User Experience**: Complete feedback (all errors at once) is better than iterative fixing (one error at a time).
- **Quality assurance**: Since this is a documentation tool, broken docs are worse than no docs. Failing the build forces users to fix configuration issues.

**Example**: If 3 methods have missing defaults and 2 have invalid `#[doc_use]` references, all 5 errors are reported together, not one at a time.

### 10. Processing Model

**Decision**: Use a **two-pass processing model**:

1. **Pass 1 - Context Extraction**:

   - Parse all `impl_kind!` invocations
   - Scan all trait `impl` blocks for associated types
   - Build complete mapping: `(Brand, Trait, AssocName, Generics) -> TargetType`
   - Collect all `#[doc_default]` annotations
   - Validate for conflicts and collisions
   - **If any errors, report all and abort before Pass 2**

2. **Pass 2 - Documentation Generation** (only if Pass 1 succeeded):
   - Process each `impl` block
   - For methods with `#[hm_signature]` or `#[doc_type_params]`:
     - Resolve `Self` and `Self::Assoc` using the context from Pass 1
     - Validate all `#[doc_use]` references
     - Generate documentation
     - Replace attributes with generated docs

**Rationale**:

- **Order Independence**: Allows forward references; methods can use `#[doc_use]` for associated types defined later in the file.
- **Clear Separation**: Context building is separated from application, simplifying logic and error handling.
- **Complete Validation**: All references can be validated before any documentation is generated, ensuring atomic success/failure.
- **Clean Errors**: Fail-fast after Pass 1 prevents cascade errors.

### 11. Type Tracking and Terminology

**Decision**: Track all types appearing in trait implementations, without distinguishing "brands" from other types.

**Terminology**: In the specification and implementation, use "type" rather than "brand" where the distinction doesn't matter:

- Projection maps are indexed by type path: `(TypePath, Option<Trait>, AssocName, Generics) -> TargetType`
- Error messages: "Cannot resolve `Self::Of` for type `MyType`" (not "brand")
- The macro doesn't need to identify whether a type is a "brand" in the HKT sense

**Rationale**:

- **Generality**: The macro works for any type appearing in trait implementations, whether or not it's used as an HKT brand
- **Simplicity**: No need for brand identification heuristics (suffix matching, registration, etc.)
- **Future-proof**: Works with any type pattern

**Implementation Note**: Internally, the projection map keys on the type path from `impl Trait for Type`. Any type that appears in that position is tracked for documentation purposes.

## Error Message Requirements

**Requirement**: All compile-time errors must be high-quality, actionable, and contextual.

### Error Message Standards

1. **Contextual**: Show the exact location (span) where the error occurred
2. **Actionable**: Suggest concrete fixes
3. **Hierarchical**: For resolution failures, show the lookup chain attempted
4. **Clear**: Use plain language, avoid jargon where possible

### Context-Aware Available Types

When showing "Available associated types" in error messages, the list should be **context-appropriate**:

**For missing projection errors** (`Self::Foo<T>` cannot be resolved):

- Show all associated types for the Brand across all traits
- If the specific name exists, show which traits define it
- Otherwise, show all type names available

**For missing default errors** (bare `Self` cannot be resolved):

- Primary: Show types defined in the current impl block
- Secondary: Show types from other impl blocks for same Brand
- Mark scope clearly: "(in this impl)" vs "(in other traits)"

**Example**:

```rust
error: Cannot resolve bare `Self` for brand `MyBrand` - no default specified
   |
   = note: Available in this impl: Of, SendOf
   = note: Available in other traits: CloneableOf
   |
help: Mark one in this impl as default, or use explicit override
```

### Example Error Messages

#### Missing Associated Type Mapping

```rust
error: Cannot resolve `Self::Of` for brand `MyBrand`
  --> src/types/mytype.rs:42:5
   |
42 |     fn foo(self) -> Self::Of<i32> { ... }
   |                     ^^^^^^^^^^^^^ No associated type `Of` found for `MyBrand`
   |
help: Add an associated type definition in `impl_kind!`:
   |
   | impl_kind! {
   |     for MyBrand {
   |         type Of<T> = SomeType<T>;
   |     }
   | }
   |
   = note: or add it to a trait impl in this module
```

#### Missing Default for Bare Self

```rust
error: Cannot resolve bare `Self` for brand `MyBrand` - no default associated type specified
  --> src/types/mytype.rs:38:25
   |
38 |     fn foo(self) -> Self { ... }
   |                     ^^^^ Multiple associated types available, but no default marked
   |
   = note: Available in this impl: Of, SendOf
   = note: Available in other traits: CloneableOf
   |
help: Mark one as the default in `impl_kind!`:
   |
   | impl_kind! {
   |     for MyBrand {
   |         #[doc_default]
   |         type Of<T> = SomeType<T>;
   |         type SendOf<T> = SomeType<T>;
   |     }
   | }
   |
help: Or use an explicit override:
   |
   | #[doc_use = "Of"]
   | fn foo(self) -> Self { ... }
```

#### Conflicting Defaults

```rust
error: Multiple `#[doc_default]` annotations found for type `MyBrand` within impl block `impl Functor for MyBrand`
  --> src/types/mytype.rs:15:9
   |
15 |         #[doc_default]
   |         ^^^^^^^^^^^^^^ First default here
...
20 |         #[doc_default]
   |         ^^^^^^^^^^^^^^ Conflicting default here
   |
   = help: Remove one `#[doc_default]` annotation
   = note: Each impl block can have at most one default associated type
```

#### Invalid `#[doc_use]` Reference

```rust
error: `#[doc_use = "Typo"]` references unknown associated type
  --> src/types/mytype.rs:42:5
   |
42 |     #[doc_use = "Typo"]
   |     ^^^^^^^^^^^^^^^^^^^ No associated type named `Typo` found for brand `MyBrand`
   |
   = note: Available associated types: Of, CloneableOf, SendOf
   |
help: Did you mean `Of`?
```

#### Self Reference in Associated Type Definition

```rust
error: `Self::` reference forbidden in associated type definition
  --> src/types/mytype.rs:25:9
   |
25 |         type Recursive<T> = Box<Self::Other<T>>;
   |                                 ^^^^^^^^^^^^^^^ `Self::Other` used here
   |
   = note: Associated type definitions cannot reference `Self::` to prevent circular resolution
   = help: Use concrete type names instead
   |
   = note: Forward references to other associated types are allowed:
   |         type A<T> = B<T>;  // OK: B is another associated type
   |         type B<T> = Vec<T>;
```

#### Multiple `#[doc_use]` Attributes

```rust
error: Multiple `#[doc_use]` attributes found on same item
  --> src/types/mytype.rs:42:5
   |
42 |     #[doc_use = "Of"]
   |     ^^^^^^^^^^^^^^^^^ First override here
43 |     #[doc_use = "SendOf"]
   |     ^^^^^^^^^^^^^^^^^^^^^ Second override here
   |
   = help: Remove all but one `#[doc_use]` attribute
   = note: Use a single `#[doc_use]` to specify which associated type to use
```

#### Attribute Placement Error

```rust
error: Documentation attribute must be placed in a valid documentation location
  --> src/types/mytype.rs:10:1
   |
9  | fn
10 | #[hm_signature]
   | ^^^^^^^^^^^^^^^ Not in a valid documentation location
11 | foo() { ... }
   |
help: Place the attribute where documentation comments are valid
   |
   | /// Description
   | ///
   | /// ### Type Signature
   | ///
   | #[hm_signature]
   | fn foo() { ... }
```

## Testing Requirements

### Coverage Targets

**Minimum Requirements**:

- Unit test coverage: ≥80% of core logic (context extraction, resolution, doc generation)
- Integration test coverage: ≥90% of user-facing features
- All error paths must have explicit tests
- All examples in this specification must have corresponding tests

**Critical Paths** (require 100% coverage):

1. Projection map building (`impl_kind!` and trait impl scanning)
2. Hierarchical default resolution (method → impl → (Type,Trait) → module → error)
3. `Self` substitution in all contexts (bare, projected, nested, `Apply!`/`Kind!`)
4. Error message generation with proper spans and suggestions

**Regression Protection**:

- **ALL** existing tests from `hm_signature.rs`, `document_impl.rs`, `doc_type_params.rs` must be preserved (adapted as needed)
- Each test must verify identical output OR document intentional changes
- No behavioral regressions without explicit justification in commit message
- Baseline: Current macro behavior is the specification for compatibility

**Quality Gates**:

- All tests must pass before merge (no exceptions)
- No `#[ignore]`'d tests in main branch without issue tracking
- Compile-fail tests must verify exact error messages (not just "it fails")
- Property-based tests for complex logic (generic matching, type comparison)

### Unit Tests

1. **Context Extraction**

   - Parse `impl_kind!` with various attribute combinations
   - Parse trait `impl` blocks with associated types
   - Verify mapping generation: `(Brand, Trait, AssocName, Generics) -> TargetType`
   - Test multiple `impl_kind!` blocks with merge logic
   - Test structural equivalence collision detection

2. **Default Conflict Detection**

   - Test `#[doc_default]` conflict detection within single impl blocks (multiple defaults = error)
   - Verify different impl blocks can have different defaults (no cross-impl conflict checking)
   - Test (Type, Trait)-scoped vs. global defaults precedence

3. **Hierarchical Resolution**

   - Test method > impl > (Type,Trait)-default > module-default precedence
   - Verify (Type, Trait)-scoped default isolation
   - Test `#[doc_use]` override behavior

4. **Generic Mapping**

   - Test positional substitution with parametric brands
   - Test renamed generics (different names, same position)
   - Test complex nested generics
   - Test parametric brand matching with renamed generics
   - Test const generic erasure in HM signatures (`[T; N]` → `[T]`)
   - Test complex const expressions erasure (`[(T, U); N + M]` → `[(T, U)]`)

5. **Type Processing**
   - Test unsized type erasure (`?Sized` removal)
   - Test HRTB erasure and function type formatting
   - Test lifetime erasure (complete removal from HM signatures)
   - Test const generic erasure from HM signatures
   - Test type alias preservation (no resolution, no equivalence checking)
   - Test bound erasure in associated type definitions
   - Test nested `Self::` resolution (2-level, 3-level) in method signatures
   - Test circular reference detection in associated type definitions (Self:: forbidden)
   - Test forward references in associated types (non-Self:: references allowed)
   - Test trait object preservation with auto-trait erasure

### Integration Tests

1. **Full Module Processing**

   - End-to-end tests with complete realistic modules
   - Test modules with multiple brands and traits
   - Test interaction between all features (defaults, overrides, projections)

2. **Error Message Quality**

   - Snapshot tests for error output
   - Verify span accuracy
   - Verify help message quality
   - Test context-aware "available types" listing

3. **Edge Cases**
   - Empty modules (no-op behavior)
   - Modules with only `impl_kind!` (no trait impls)
   - Modules with only trait impls (no `impl_kind!`)
   - Single-method impls
   - Methods without documentation attributes (should be unchanged)
   - Nested modules with independent `#![document_module]` scopes
   - Missing defaults (should error with helpful message)
   - Multiple documentation attributes on the same method
   - CFG-conditional type definitions (all branches extracted)
   - cfg_attr expansion (all conditional attributes expanded)
   - Attribute placement errors
   - Multiple `#[doc_use]` attributes on same item (should error)

### Regression Tests

1. **Parity Tests**

   - Migrate all existing `#[document_impl]` tests
   - Compare output with current macro for same input
   - Verify identical HM signature generation
   - Verify identical type parameter documentation

2. **Backward Compatibility**
   - Ensure all existing valid code patterns still work
   - Test trait hierarchies
   - Test complex generic constraints

### Property-Based Tests

1. **Fuzz Testing**
   - Generate random valid module ASTs
   - Verify no panics (all errors are `Result::Err`, not panics)
   - Test with deeply nested generics
   - Test with many brands and traits

## Limitations

### 1. Cross-Module Visibility and Module Scope Isolation

The `#[document_module]` macro can only inspect the tokens within the module it is applied to. Each `#![document_module]` invocation creates an **isolated scope**.

**Behavior**: The `#![document_module]` attribute is **non-recursive**. It only processes items directly within the annotated module.

**Inline Submodules**: Nested inline modules (`mod inner { ... }`) are NOT processed unless they have their own `#![document_module]` attribute.

**Rationale**:

- Explicit scope control - each module opts in independently
- Predictable behavior - attribute affects only the annotated module
- Matches Rust's standard scoping intuitions

**Example**:

```rust
#![document_module]

impl_kind! { for OuterBrand { type Of<T> = Vec<T>; } }  // ✅ Processed
impl Trait for OuterBrand { ... }                        // ✅ Processed

mod inner {
    impl_kind! { for InnerBrand { type Of<T> = Box<T>; } }  // ❌ NOT processed
    // To enable processing, add #![document_module] at top of inner module
}

mod other {
    #![document_module]  // ✅ Independent scope
    impl_kind! { for InnerBrand { type Of<T> = Box<T>; } }  // ✅ Processed in this scope
}
```

**Nested Module Scope Isolation**: Each `#![document_module]` creates an independent scope. Child modules do NOT inherit parent module's type mappings:

```rust
#![document_module]
impl_kind! { for OuterBrand { type Of<T> = Vec<T>; } }

mod inner {
    #![document_module]

    impl SomeTrait for super::OuterBrand {
        #[hm_signature]
        fn method(&self) -> Self { ... }
        // ❌ Error: No mapping for OuterBrand in this scope
    }
}
```

**Workaround**: Repeat `impl_kind!` in nested modules or use `#[doc_use]`:

```rust
mod inner {
    #![document_module]

    #[doc_use = "Of"]  // Explicit override
    impl SomeTrait for super::OuterBrand {
        type Of<T> = Vec<T>;  // Redeclare if needed

        #[hm_signature]
        fn method(&self) -> Self { ... }  // ✅ Works
    }
}
```

**File Modules**: For file modules (`mod foo;` referring to `foo.rs`), the `#![document_module]` attribute must be placed at the top of the file (`foo.rs`), not at the module declaration site.

**Mitigation**: If the macro encounters a type that it cannot resolve (because the definition is in another module), it will:

1.  **Check for Manual Overrides**: Look for `#[doc_use = "..."]` on the `impl` block or method.
2.  **Fallback Error**: If no mapping and no override are found, emit a compile-time error instructing the user to co-locate the definition or provide explicit overrides.

**Rationale**: Proc macros receive module content as a single token stream without parent context. Strict lexical scoping is predictable and easy to reason about. This aligns with the general principle that the macro only operates on information explicitly present in its input.

### 2. Macro Expansion Order

Attribute macros run before the expansion of macros inside them. `#[document_module]` will not see `impl` blocks generated by other macros.

**Mitigation**: Explicitly documented with examples (see Design Decision #7). Code generated by other macros will not be automatically documented by this system. Users must use explicit overrides or modify the generating macro.

### 3. Circular References in Associated Types

If `Self::Assoc` appears in the right-hand side of an associated type definition, the macro will error rather than attempting resolution.

**Example**:

```rust
impl SomeTrait for MyBrand {
    type Assoc<T> = Box<Self::Other<T>>;  // ❌ Error: Circular reference
    type Other<T> = Vec<T>;
}
```

**Note**: This restriction applies only to associated type **definitions**. Methods may freely use nested `Self::` references in their signatures (e.g., `Self::Of<Self::Of<T>>`), which are resolved correctly.

**Rationale**: Resolving this correctly requires topological sorting and dependency analysis. The complexity is not justified given the rarity of this pattern. Users should define types concretely without `Self::` references.

### 4. Associated Type Defaults in Trait Definitions

The macro only extracts associated type mappings from `impl` blocks, not from trait definitions. If a trait provides a default associated type and the impl doesn't override it, the macro will have no projection for that type.

**Behavior**: Users must explicitly specify associated types in impl blocks, even when the trait provides a default.

**Example**:

```rust
trait MyTrait {
    type Of<T> = Vec<T>;  // Trait-provided default
}

// ❌ Error: If method uses #[hm_signature], no projection found
impl MyTrait for MyBrand {
    // No override; uses trait default
    #[hm_signature]
    fn method<T>(&self) -> Self::Of<T> { ... }  // Error: Cannot resolve Self::Of
}

// ✅ Solution: Explicit override
impl MyTrait for MyBrand {
    type Of<T> = Vec<T>;  // Explicit, even though trait provides default

    #[hm_signature]
    fn method<T>(&self) -> Self::Of<T> { ... }  // Works
}
```

**Rationale**: Parsing trait definitions across module/crate boundaries is complex and would result in inconsistent behavior (works for local traits, fails for external traits). Requiring explicit overrides keeps the implementation simple, predictable, and makes the code self-documenting.

### 5. Interaction with Other Procedural Macros

**Recommended Usage**: Apply `#![document_module]` as the first inner attribute in the module.

**Compatibility**: Most macros should work fine with `#[document_module]`:

- ✅ **Derive macros** and attribute macros on items (structs, enums, functions)
- ✅ **Method-level attributes** (e.g., `#[inline]`, `#[cfg(...)]`)
- ⚠️ **Module-level attribute macros** that transform the entire module may have undefined behavior

**Expansion Order**: `#[document_module]` sees the module content before other macros inside it expand. This means it can see `impl_kind!` invocations but not the code they generate.

**cfg_attr Handling**: The macro conservatively expands all `cfg_attr` conditions for documentation purposes:

```rust
#[cfg_attr(feature = "docs", doc_default)]
type Of<T> = Arc<T>;
```

Is treated as if `#[doc_default]` is always present, regardless of active features. This ensures complete documentation across all configurations.

**Limitation**: Only applies to documentation attributes (`doc_default`, `doc_use`). Other `cfg_attr` usage is passed through unchanged.

**Testing**: If combining with other macros, test your specific combination to ensure documentation is generated as expected.

### 6. Performance Considerations

**Priority**: Correctness over performance in initial implementation.

**Known Complexity**:

- Context extraction (Pass 1): O(n×k) where n = impl blocks, k = associated types
- Collision detection: O(k²) per Brand
- Documentation generation (Pass 2): O(m) where m = methods with doc attributes
- Structural comparison: O(t) per type, where t = type AST size

**Practical Limits** (expected ranges):

- Modules with <100 impl blocks: Fast (< 100ms additional compile time)
- Modules with 100-500 impl blocks: Acceptable (< 500ms)
- Modules with >500 impl blocks: Consider splitting into smaller modules
- Brands with <20 associated types: No issues
- Brands with >50 associated types: Collision detection may be noticeable

**Future Optimizations** (if needed):

- Caching for repeated structural comparisons
- Parallel processing of independent impl blocks
- Incremental compilation improvements

**Current Recommendation**: Implement correctly first, measure actual impact, optimize only if problems arise in real usage. The two-pass model is inherently efficient (single traversal of AST per pass).

## Documentation Best Practices

### Organizing Code for Documentation

1. **Co-locate Definitions**: Keep `impl_kind!` and trait implementations in the same module
2. **Use Section Markers**: Include empty section markers in doc comments where generated content should appear:
   ```rust
   /// Method description
   ///
   /// ### Type Signature
   ///
   #[hm_signature]
   ///
   /// ### Parameters
   /// ...
   ```
3. **Explicit Overrides**: For traits with defaults, explicitly override associated types
4. **Consistent Defaults**: Mark one primary associated type as `#[doc_default]` per brand

### Common Patterns

#### Single Associated Type

```rust
impl_kind! {
    for MyBrand {
        #[doc_default]
        type Of<T> = MyType<T>;
    }
}

impl Functor for MyBrand {
    #[hm_signature]
    fn map<A, B>(...) -> Apply!(<Self as Kind!(type Of<T>;)>::Of<B>) { ... }  // Uses default
}
```

#### Multiple Associated Types with (Type, Trait)-Scoped Defaults

```rust
impl Pointer for ArcBrand {
    #[doc_default]
    type Of<T> = Arc<T>;

    #[hm_signature]
    fn new<T>(...) -> Self { ... }  // Uses Of as default
}

impl SendRefCountedPointer for ArcBrand {
    #[doc_default]
    type SendOf<T: Send + Sync> = Arc<T>;

    #[hm_signature]
    fn send_new<T: Send + Sync>(...) -> Self { ... }  // Uses SendOf as default
}
```

#### Explicit Method-Level Override

```rust
impl ComplexTrait for MyBrand {
    type Of<T> = Box<T>;
    type SendOf<T: Send> = Arc<T>;

    #[doc_use = "Of"]
    #[hm_signature]
    fn local_method(&self) -> Self { ... }

    #[doc_use = "SendOf"]
    #[hm_signature]
    fn thread_safe_method(&self) -> Self { ... }
}
```

## Implementation Plan

1.  **Refactor `impl_kind!` Parser**:

    - Update `fp-macros/src/impl_kind.rs` to parse attributes on associated type definitions
    - Strip attributes from output to avoid warnings

2.  **Refactor Config**:

    - Update `Config` struct to support (Type, Trait)-scoped mappings: `(Brand, Option<Trait>, AssocName, Generics) -> TargetType`
    - Add (Type, Trait)-scoped and module-scoped defaults
    - Implement positional generic matching (no bound validation - out of scope)

3.  **Refactor Core Logic**:

    - Extract signature generation logic from `document_impl` into shared module (e.g., `signature_gen`)
    - Update to use new Config for resolution (Projection vs (Type,Trait)-Scoped vs Module Default)
    - Implement explicit `Self` substitution with circular reference detection (for associated type definitions only)
    - Add type erasure logic (unsized, HRTB, lifetimes, const generics)
    - Reuse existing `Apply!`/`Kind!` traversal logic from `hm_signature.rs`

4.  **Implement `document_module` - Pass 1 (Context Extraction)**:

    - Parse all `impl_kind!` invocations
    - Scan all trait `impl` blocks for associated types
    - Build comprehensive mapping
    - Detect and merge multiple `impl_kind!` blocks for same Brand
    - Validate for collisions using structural equivalence (AST-based, using/extending `hm_ast.rs`)
    - Collect and validate `#[doc_default]` annotations
    - Check for circular `Self::` references in associated type definitions

5.  **Implement `document_module` - Pass 2 (Documentation Generation)**:

    - Traverse module items
    - For each `impl`, collect attributes for overrides
    - For each method with `#[hm_signature]` or `#[doc_type_params]`:
      - Resolve `Self` using hierarchical rules
      - Validate all references against Pass 1 context
      - Generate documentation (reusing existing logic)
      - Replace attributes with generated docs

6.  **Error Handling**:

    - Implement **fail-fast after Pass 1**: If any errors in context extraction, report all Pass 1 errors and abort before Pass 2
    - **Rationale**: Pass 1 errors often cause cascade failures in Pass 2; clean error messages without spurious failures
    - Implement all error messages per requirements section
    - Add span tracking for precise error locations (use original source spans)
    - Implement help suggestions for common errors
    - Implement context-aware "available types" listing

7.  **Test Migration**:

    - Adapt existing tests from `hm_signature.rs`, `document_impl.rs`, `doc_type_params.rs`
    - Add new tests for (Type, Trait)-scoped defaults
    - Add new tests for two-pass processing
    - Add property-based tests for robustness
    - Ensure full behavioral parity with baseline

8.  **Update `lib.rs`**:

    - Export `document_module`
    - Delete `document_impl` macro
    - Update documentation

9.  **Documentation**:

    - Write comprehensive migration guide
    - Document all attributes (`#[doc_default]`, `#[doc_use]`, `#[hm_signature]`, `#[doc_type_params]`)
    - Provide examples for common patterns
    - Document limitations and workarounds

10. **Changelog**:
    - Document breaking change: `#[document_impl]` removed
    - Explain migration path
    - Note loss of `Cargo.toml` metadata (acceptable trade-off)
    - Highlight new features ((Type, Trait)-scoped defaults, opt-in documentation)
