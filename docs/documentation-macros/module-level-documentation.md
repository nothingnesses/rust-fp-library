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

## Functional Requirements

### 1. Context Extraction (Full Projection & Impl-Scanning)

**Requirement**: The macro must extract a comprehensive mapping of `(Brand, Trait, AssociatedType, Generics) -> TargetType` from two sources. This extra detail is required to perform correct parameter substitution during resolution (e.g., mapping `Self::Of<X, Y>` to `Result<X, Y>` requires knowing that `Of<A, B> = Result<A, B>`).

#### 1.1 Extraction Sources

1.  **`impl_kind!` Invocations**:

    ```rust
    impl_kind! { for MyBrand { type Of<A> = Box<A>; } }
    ```

    Extracts: `(MyBrand, None, Of<A>)` -> `Box<A>`.

    The `Trait` is `None` because `impl_kind!` defines global defaults not tied to a specific trait.

2.  **Standard `impl` Blocks (Impl-Scanning)**:
    ```rust
    impl Pointer for ArcBrand {
        type Of<T> = Arc<T>;
    }
    ```
    Extracts: `(ArcBrand, Some(Pointer), Of<T>)` -> `Arc<T>`.

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

#### 1.3 Trait Hierarchy Collision Detection

**Challenge**: When multiple `impl` blocks define the same associated type name for the same Brand via trait inheritance, the macro must distinguish between inheritance (valid) and genuine collisions (error).

**Strategy**: Use **structural equivalence checking** rather than attempting to resolve trait hierarchies (which would require type system information unavailable to proc macros).

**Behavior**:

- If multiple trait impls for the same Brand define an associated type with the same name:
  - Check if the target types are syntactically/structurally identical
  - If yes: Treat as same definition (no collision)
  - If no: Error unless one is marked `#[doc_default]`

**Rationale**: This approach is implementable without type system access and handles most practical cases. It may require explicit `#[doc_default]` in edge cases where inherited types are re-stated differently.

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
    // ✅ Valid: No explicit redefinition, uses inherited Of<T>
}

// ⚠️ Edge case: Explicit re-statement
impl Base for AnotherBrand {
    type Of<T> = Box<T>;
}

impl Derived for AnotherBrand {
    type Of<T> = Box<T>;  // ✅ Valid: Structurally identical to Base's definition
}
```

#### 1.4 Generics Handling

The extraction logic must use **Positional Mapping** to map generic parameters correctly.

- **Capture Full Context**: The mapping must be `(Brand, Trait, AssocName, AssocGenerics) -> TargetType`. The `TargetType` may refer to generics from the `impl` block.
- **Positional Matching**: Generic parameters are matched by position, not name, between `impl_kind!` and trait impls. This allows renamed generics while ensuring correct substitution.
- **Scope Awareness**: When substituting, ensure that `impl` generics are treated as "constants" in the scope, while associated type generics are substituted.
- **Lifetimes and Const Generics**: These are **erased** in HM signature documentation to reduce noise and maintain mathematical notation simplicity. During extraction, const generics are identified but only type generics are used for positional matching.
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
    fn map<A, B>(...) -> Self::Of<B> {
        // Resolution: Self::Of<B> -> Lazy<B, C>
        // Position 0: Config/C matched by position, uses C (the trait impl's name)
    }
}
```

**Parametric Brand Matching**: When matching parametric brands (e.g., `LazyBrand<Config>` against `LazyBrand<C>`), the macro uses structural matching:

1. Extract the base type path (e.g., `LazyBrand`)
2. Match generic parameters positionally (position 0: `Config` ↔ `C`)
3. Validate that trait impl bounds are compatible with (i.e., a superset of or equal to) the bounds in `impl_kind!`
4. Use the trait impl's generic names in the substitution (e.g., use `C`, not `Config`)

**Example**:

```rust
// impl_kind! defines: LazyBrand<Config: LazyConfig> with Of<T> = Lazy<T, Config>
// Trait impl uses: LazyBrand<C: LazyConfig + Send>
// Match: Position 0 maps Config ↔ C
// Validation: C's bounds (LazyConfig + Send) are a superset of Config's bounds (LazyConfig) ✅
// Substitution: Use C in the resolved type: Self::Of<B> → Lazy<B, C>
```

**Const Generics Handling**: Const generics in associated types are identified during parsing but **erased** from the projection map keys. Only type parameters are used for positional matching.

**Example**:

```rust
impl_kind! {
    for ArrayBrand {
        type Of<const N: usize, T> = [T; N];  // N is const generic
    }
}

// Stored in projection map as: (ArrayBrand, None, Of, [T]) -> [T; N]
// Note: Only T is in the key; N is preserved in the target type but not used for matching

impl Functor for ArrayBrand {
    fn map<const N: usize, A, B>(...) -> Self::Of<N, B> { ... }
    // Resolution: Match only type params (B), const generics flow through
    // Resolves to: [B; N]
}
```

**Parametric Brand Matching**: When matching parametric brands (e.g., `LazyBrand<Config>` against `LazyBrand<C>`), the macro uses structural matching:

1. Extract the base type path (e.g., `LazyBrand`)
2. Match generic parameters positionally (position 0: `Config` ↔ `C`)
3. Validate that trait impl bounds are compatible with (i.e., a superset of or equal to) the bounds in `impl_kind!`
4. Use the trait impl's generic names in the substitution (e.g., use `C`, not `Config`)

**Example**:

```rust
// impl_kind! defines: LazyBrand<Config: LazyConfig> with Of<T> = Lazy<T, Config>
// Trait impl uses: LazyBrand<C: LazyConfig + Send>
// Match: Position 0 maps Config ↔ C
// Validation: C's bounds (LazyConfig + Send) are a superset of Config's bounds (LazyConfig) ✅
// Substitution: Use C in the resolved type: Self::Of<B> → Lazy<B, C>
```

**Const Generics Handling**: Const generics in associated types are identified during parsing but **erased** from the projection map keys. Only type parameters are used for positional matching.

**Example**:

```rust
impl_kind! {
    for ArrayBrand {
        type Of<const N: usize, T> = [T; N];  // N is const generic
    }
}

// Stored in projection map as: (ArrayBrand, None, Of, [T]) -> [T; N]
// Note: Only T is in the key; N is preserved in the target type but not used for matching

impl Functor for ArrayBrand {
    fn map<const N: usize, A, B>(...) -> Self::Of<N, B> { ... }
    // Resolution: Match only type params (B), const generics flow through
    // Resolves to: [B; N]
}
```

**Validation**: The macro validates that bounds in the trait impl are compatible with (i.e., a superset of or equal to) the bounds in `impl_kind!`.

#### 1.5 Where Clause Handling

The extraction logic must robustly parse `where` clauses on associated types (e.g., `type Of<A> = Foo<A> where A: Clone;`) but **ignore** them for the purpose of building the projection map. The `where` clause is relevant for code validity but not for the structural mapping required for documentation.

#### 1.6 Cfg Handling

The macro should **ignore** `cfg` attributes during extraction (i.e., extract everything). This ensures that documentation is complete regardless of the compiler's active feature flags, preventing broken links or missing docs for optional features.

#### 1.7 Unsized Types and Higher-Ranked Trait Bounds

**Decision**: Erase to simple representation in HM signatures.

**Rationale**: HM signatures are meant to be clean, mathematical abstractions. Full Rust type information (with all markers and bounds) is still available in the actual function signature documentation.

**Rules**:

- `T: ?Sized` → `T` (erase unsized marker)
- `for<'a> Fn(&'a T)` → `Fn &T` (erase HRTB, convert to curried form)
- `Self::Of<T: Clone>` → `Self::Of T` (erase bounds from type application)

**Example**:

```rust
// Rust signature:
fn foo<T: ?Sized>(x: &T) -> impl for<'a> Fn(&'a T) -> String { ... }

// Generated HM signature:
// forall T. &T -> (&T -> String)
```

#### 1.8 Type Aliases

**Decision**: No alias resolution.

**Behavior**: Type aliases are preserved as-is in mappings and HM signatures.

**Rationale**: Users write aliases for semantic reasons (e.g., `type UserId = String`). Resolving to the underlying type loses that semantic meaning. The alias itself communicates intent.

**Example**:

```rust
pub type MyAlias<T> = Vec<T>;

impl_kind! {
    for MyBrand {
        type Of<T> = MyAlias<T>;  // Preserved as MyAlias, not resolved to Vec
    }
}
```

### 2. Hierarchical Configuration

**Requirement**: To resolve the concrete type of `Self` (when used bare, e.g., `fn foo(self)`), the macro must support a precedence hierarchy:

1.  **Method Override**: `#[doc_use = "AssocName"]` on the method.
2.  **Impl Block Override**: `#[doc_use = "AssocName"]` on the `impl` block.
3.  **Trait-Specific Default**: `#[doc_default]` on the associated type definition in a trait `impl` block (applies only to that trait and its methods).
4.  **Module Default**: `#[doc_default]` on the associated type definition in `impl_kind!` (applies globally within the module).
5.  **Fallback**: Error.

#### 2.1 Trait-Scoped Defaults

**Decision**: `#[doc_default]` in a trait `impl` is trait-scoped and only applies to methods of that trait.

**Clarification**: "Trait-scoped" means **impl-block-scoped**. Each `impl Trait for Brand` block is an independent scope. A default marked in `impl Base for Brand` does **not** automatically apply to `impl Derived for Brand`, even if `Derived: Base`. Methods use the default from the impl block they are defined in.

**Rationale**:

- Types like `ArcBrand` implement multiple traits (`Pointer`, `RefCountedPointer`, `SendRefCountedPointer`) with different associated types (`Of`, `CloneableOf`, `SendOf`).
- Each trait's methods may naturally use a different associated type.
- Impl-block-scoped defaults allow setting appropriate defaults per trait without requiring method-level annotations.
- Maintains explicitness while providing ergonomic defaults where semantically appropriate.
- The macro cannot reliably resolve trait hierarchies across module boundaries, so it treats each impl block independently.

**Resolution Algorithm**:
When resolving bare `Self` in a method within `impl Trait for Brand`:

1. Check for method-level `#[doc_use]`
2. Check for impl-block-level `#[doc_use]`
3. Check for trait-specific default: `#[doc_default]` in this `impl Trait` block
4. Check for module-level default: `#[doc_default]` in `impl_kind!`
5. Error if no default found

**Example**:

```rust
impl_kind! {
    for ArcBrand {
        #[doc_default]  // Global default
        type Of<T> = Arc<T>;
    }
}

impl Pointer for ArcBrand {
    type Of<T> = Arc<T>;

    fn new<T>(value: T) -> Self {  // Uses global default: Arc<T>
        Arc::new(value)
    }
}

impl RefCountedPointer for ArcBrand {
    #[doc_default]  // Trait-specific default, overrides global for this trait
    type CloneableOf<T: ?Sized> = Arc<T>;

    fn new<T>(value: T) -> Self {  // Uses trait default: Arc<T> via CloneableOf
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

**Within Trait Scope**: If multiple associated types in the same trait `impl` are marked `#[doc_default]`, emit a compile-time error.

**Across Traits**: Different traits can have different defaults without conflict (trait-scoped resolution).

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

**Documentation Placement**: Generated documentation is placed exactly where the documentation attribute (`#[hm_signature]`, `#[doc_type_params]`) appears in the source code. The attribute invocation itself is replaced with the generated doc comment. This matches the current behavior of the standalone `#[document_impl]` macro.

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
    fn map<A, B>(self, f: impl Fn(A) -> B) -> Self::Of<B> { ... }
}
```

After macro expansion, the `#[hm_signature]` line is replaced with the generated HM signature documentation.

**Self Resolution**:
For methods with documentation attributes, resolve `Self` usage:

- **Path/Projected** (`Self::Assoc`): Map using the Context Extraction table (e.g., `Self::SendOf` -> `Arc`).
- **Bare** (`self`, `Self`): Map using the Hierarchical Default (following precedence rules).
- **Nested Self References**: The macro supports nested `Self::` references (e.g., `Self::Of<Self::Of<T>>`) through iterative resolution. The resolution continues until no `Self::` remains, without circular reference detection in nested contexts. The user is responsible for ensuring the types are valid; rustc will catch circular or invalid types.
- **Apply! Macro**: Explicitly traverse and resolve `Apply!` invocations, substituting `Self` within them.

**Visibility**: All items are processed regardless of visibility (`pub`, `pub(crate)`, private). Internal documentation is valuable for maintainers.

**Rationale**: Opt-in design ensures zero overhead for undocumented methods and makes the macro's impact explicit. Users add `#[hm_signature]` only where they want generated documentation.

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

### 2. Syntax: `#[doc_default]` and `#[doc_use]`

**Decision**: Split configuration into two attributes:

- **Inside `impl_kind!` / `impl`**: `#[doc_default]` (marker) on a `type` definition sets the default for that Brand within the trait's scope (or globally if in `impl_kind!`).
- **On `impl` / `fn`**: `#[doc_use = "AssocName"]` overrides the default to use the target of `AssocName`.

**Rationale**:

- **Clarity**: Separating the definition of a default (`doc_default`) from the usage/selection (`doc_use`) reduces confusion and makes the intent explicit.
- **Consistency**: Explicitly naming the associated type (`AssocName`) ensures that the override points to a valid, defined type mapping.
- **Trait Scoping**: Placing `#[doc_default]` in trait impls enables trait-specific defaults without polluting the global namespace.

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

**Decision**: Use **Hard Errors** (compile errors) for any resolution failures or configuration ambiguities. The macro collects all errors during processing and reports them together before failing.

**Error Collection**: The macro processes the entire module and accumulates errors in a collection. After completing both Pass 1 (context extraction) and Pass 2 (documentation generation), if any errors were encountered, they are all reported together and compilation fails. This all-or-nothing approach ensures:

- **Complete Feedback**: Developers see all errors at once, not one at a time
- **Consistent State**: Either all documentation is generated correctly, or none is generated
- **Quality Assurance**: "Silently broken" docs (e.g., missing signatures or incorrect types) are prevented

**Rationale**: Since this is a documentation tool, broken docs are worse than no docs. Failing the build forces users to fix configuration issues, ensuring documentation accuracy. Collecting all errors before failing provides better developer experience than fail-fast.

### 10. Processing Model

**Decision**: Use a **two-pass processing model**:

1. **Pass 1 - Context Extraction**:

   - Parse all `impl_kind!` invocations
   - Scan all trait `impl` blocks for associated types
   - Build complete mapping: `(Brand, Trait, AssocName, Generics) -> TargetType`
   - Collect all `#[doc_default]` annotations
   - Validate for conflicts and collisions

2. **Pass 2 - Documentation Generation**:
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

## Error Message Requirements

**Requirement**: All compile-time errors must be high-quality, actionable, and contextual.

### Error Message Standards

1. **Contextual**: Show the exact location (span) where the error occurred
2. **Actionable**: Suggest concrete fixes
3. **Hierarchical**: For resolution failures, show the lookup chain attempted
4. **Clear**: Use plain language, avoid jargon where possible

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
   = note: Available associated types: Of, CloneableOf, SendOf
   |
help: Mark one as the default in `impl_kind!`:
   |
   | impl_kind! {
   |     for MyBrand {
   |         #[doc_default]
   |         type Of<T> = SomeType<T>;
   |         type CloneableOf<T> = SomeType<T>;
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
error: Multiple `#[doc_default]` annotations found for brand `MyBrand` within trait `Functor`
  --> src/types/mytype.rs:15:9
   |
15 |         #[doc_default]
   |         ^^^^^^^^^^^^^^ First default here
...
20 |         #[doc_default]
   |         ^^^^^^^^^^^^^^ Conflicting default here
   |
   = help: Remove one `#[doc_default]` annotation or use trait-scoped defaults
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

#### Circular Reference in Associated Type

```rust
error: Circular reference detected in associated type definition
  --> src/types/mytype.rs:25:9
   |
25 |         type Recursive<T> = Box<Self::Other<T>>;
   |                                 ^^^^^^^^^^^^^^^ `Self::Other` used here
   |
   = note: Associated type definitions cannot reference other `Self::` types
   = help: Define concrete types directly without `Self::` references
```

## Testing Requirements

### Unit Tests

1. **Context Extraction**

   - Parse `impl_kind!` with various attribute combinations
   - Parse trait `impl` blocks with associated types
   - Verify mapping generation: `(Brand, Trait, AssocName, Generics) -> TargetType`
   - Test multiple `impl_kind!` blocks with merge logic
   - Test structural equivalence collision detection

2. **Collision Detection**

   - Verify errors for duplicate definitions with different targets
   - Verify acceptance of structurally identical duplicates
   - Test `#[doc_default]` conflict detection within traits
   - Test trait-scoped vs. global defaults

3. **Hierarchical Resolution**

   - Test method > impl > trait-default > module-default precedence
   - Verify trait-scoped default isolation
   - Test `#[doc_use]` override behavior

4. **Generic Mapping**

   - Test positional substitution with parametric brands
   - Test renamed generics (different names, same position)
   - Test bound validation (subset/superset checking)
   - Test complex nested generics
   - Test parametric brand matching with renamed generics
   - Test const generic erasure in projection maps

5. **Type Processing**
   - Test unsized type erasure (`?Sized` removal)
   - Test HRTB erasure and currying
   - Test lifetime and const generic erasure
   - Test type alias preservation (no resolution)
   - Test bound erasure in associated type definitions
   - Test nested `Self::` resolution (2-level, 3-level)

### Integration Tests

1. **Full Module Processing**

   - End-to-end tests with complete realistic modules
   - Test modules with multiple brands and traits
   - Test interaction between all features (defaults, overrides, projections)

2. **Error Message Quality**

   - Snapshot tests for error output
   - Verify span accuracy
   - Verify help message quality

3. **Edge Cases**
   - Empty modules (no-op behavior)
   - Modules with only `impl_kind!` (no trait impls)
   - Modules with only trait impls (no `impl_kind!`)
   - Single-method impls
   - Methods without documentation attributes (should be unchanged)
   - Nested modules with independent `#![document_module]` scopes
   - Trait defaults without explicit overrides (should error with helpful message)
   - Multiple documentation attributes on the same method

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
   - Verify no panics (all errors are Result::Err, not panics)
   - Test with deeply nested generics
   - Test with many brands and traits

## Limitations

### 1. Cross-Module Visibility and Module Scope Isolation

The `#[document_module]` macro can only inspect the tokens within the module it is applied to. Each `#![document_module]` invocation creates an **isolated scope**.

**Module Nesting**: Nested modules with `#![document_module]` are processed independently. A child module cannot see `impl_kind!` definitions from its parent module, and vice versa.

**Example**:

```rust
// src/types/my_type.rs
#![document_module]

impl_kind! {
    for OuterBrand { type Of<T> = Vec<T>; }
}

mod inner {
    #![document_module]  // Independent scope

    impl_kind! {
        for InnerBrand { type Of<T> = Box<T>; }
    }

    // ❌ Error: Cannot resolve OuterBrand::Of (not visible in this scope)
    impl SomeTrait for OuterBrand {
        #[hm_signature]
        fn method(&self) -> Self::Of<i32> { ... }
    }
}
```

**Mitigation**: If the macro encounters a Brand that it cannot resolve (because the definition is in another module), it will:

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
    type Assoc<T> = Box<Self::Other<T>>;  // ❌ Error
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

**Testing**: If combining with other macros, test your specific combination to ensure documentation is generated as expected.

### 6. Performance Considerations

**Current Priority**: Correctness over performance. The macro will be implemented for correct behavior first. Performance optimizations (if needed) will be addressed in future iterations based on actual measured impact.

**Future Work**:

- If compile-time impact becomes significant, consider caching strategies
- Measure incremental compilation impact on large modules
- Optimize if needed, but only after establishing correctness baseline

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
    fn map<A, B>(...) -> Self::Of<B> { ... }  // Uses default
}
```

#### Multiple Associated Types with Trait-Scoped Defaults

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

    - Update `Config` struct to support trait-scoped mappings: `(Brand, Option<Trait>, AssocName, Generics) -> TargetType`
    - Add trait-scoped and module-scoped defaults
    - Implement positional generic matching with bound validation

3.  **Refactor Core Logic**:

    - Extract signature generation logic from `document_impl` into shared module (e.g., `signature_gen`)
    - Update to use new Config for resolution (Projection vs Trait-Scoped vs Module Default)
    - Implement explicit `Self` substitution with circular reference detection
    - Add type erasure logic (unsized, HRTB, lifetimes, const generics)

4.  **Implement `document_module` - Pass 1 (Context Extraction)**:

    - Parse all `impl_kind!` invocations
    - Scan all trait `impl` blocks for associated types
    - Build comprehensive mapping
    - Detect and merge multiple `impl_kind!` blocks for same Brand
    - Validate for collisions using structural equivalence
    - Collect and validate `#[doc_default]` annotations

5.  **Implement `document_module` - Pass 2 (Documentation Generation)**:

    - Traverse module items
    - For each `impl`, collect attributes for overrides
    - For each method with `#[hm_signature]` or `#[doc_type_params]`:
      - Resolve `Self` using hierarchical rules
      - Validate all references against Pass 1 context
      - Generate documentation
      - Replace attributes with generated docs

6.  **Error Handling**:

    - Implement all error messages per requirements section
    - Add span tracking for precise error locations
    - Implement help suggestions for common errors

7.  **Test Migration**:

    - Adapt existing tests from `hm_signature.rs`, `document_impl.rs`, `doc_type_params.rs`
    - Add new tests for trait-scoped defaults
    - Add new tests for two-pass processing
    - Add property-based tests for robustness
    - Ensure full behavioral parity

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
    - Highlight new features (trait-scoped defaults, opt-in documentation)
