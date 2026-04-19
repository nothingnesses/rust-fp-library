# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.0] - 2026-04-19

### Added

- **`InferableFnBrand` resolution in HM signatures**: HM signature generation now resolves `InferableFnBrand` wrapper types to their semantic equivalents, eliminating stringly-typed processing.

### Changed

- **`doc_include!` renamed to `include_documentation!`**: Follows Rust naming conventions (verb-noun order, paralleling `include_str!` and `include_bytes!` from std). All 21 call sites in fp-library updated.
- **`Slot` renamed to `InferableBrand`**: Generated trait names updated (`InferableBrand_*` instead of `Slot_*`). `FnBrandSlot` renamed to `InferableFnBrand`. `no_inferable_brand` attribute renamed to `multi_brand`.
- **`generate_function_re_exports!` removed**: Macro deleted; fp-library now uses manual `pub use` re-exports for dispatch functions.
- **Dispatch analysis uses `syn::Type` AST nodes**: Replaced stringly-typed processing with `syn` AST pattern matching in dispatch data structures, eliminating `parse_str` round-trips.
- **Absolute crate paths in macro output**: Generated code uses `::fp_library::` paths and raw string literals instead of relative paths.

### Fixed

- **`dyn` erased from trait objects in HM signatures**: Trait objects like `dyn Fn(A) -> B` now display without the `dyn` keyword in generated signatures.
- **Canonicalizer correctness**: Addressed audit findings for improved robustness.
- **Reversed tuple param order**: Fixed detection of reversed tuple parameter order in `compose_kleisli_flipped`.

## [0.7.1] - 2026-04-15

### Added

- **`doc_include!` proc macro**: Reads a markdown file at compile time and rewrites same-directory `.md` links to rustdoc intra-doc links (`[text][crate::docs::module_name]`). Enables cross-document links in rendered rustdoc output.

### Fixed

- **Explicit dispatch HM signatures**: `is_inferable_brand_param` renamed to `is_dispatch_container_param` and extended to recognize `*Dispatch` bounds (not just `InferableBrand_*`). This fixes explicit closureless dispatch functions (compact, separate, alt, join) whose HM signatures showed raw `FA` instead of branded container types. Also handles `impl *Dispatch` params for closureless explicit functions (e.g., `explicit::join`).

### Changed

- **`trait_kind!`/`impl_kind!` documentation**: Updated to mention `InferableBrand` trait generation.

## [0.7.0] - 2026-04-14

### Added

- **Dispatch-aware HM signature generation**: `#[document_module]` analyzes dispatch traits (Pass 1b) and generates Hindley-Milner type signatures for inference wrapper functions by building synthetic signatures that replace dispatch machinery with semantic equivalents. Produces signatures like `forall Brand A B. Functor Brand => (A -> B, Brand A) -> Brand B`.
- **`#[document_signature]` manual override**: `#[document_signature("forall A B. (A -> B) -> A -> B")]` emits the provided string directly, bypassing the generation pipeline.
- **`InferableBrand` trait generation**: `trait_kind!` generates `InferableBrand_*` traits alongside `Kind_*` traits. `impl_kind!` auto-generates `InferableBrand` impls for concrete types.
- **Inferred mode for `m_do!`/`a_do!`**: `m_do!({ ... })` infers the brand from the first monadic expression instead of requiring an explicit brand parameter.
- **`ref` qualifier for `m_do!`/`a_do!`**: `m_do!(ref Brand { ... })` generates code that dispatches to by-reference trait methods.
- **Exclusion support in `generate_function_re_exports!`**: `exclude { "module::function" }` syntax to suppress re-exports of functions superseded by dispatch versions.
- **Insta snapshot regression tests**: 19 per-file tests covering all 38 dispatch inference wrapper HM signatures, plus 14 edge case tests for unusual inputs and graceful fallback behavior.

### Changed

- **`#[document_signature]` attribute parsing**: Now accepts an optional string literal argument (previously rejected all arguments).
- **Dispatch analysis uses direct sources**: Container param mapping uses positional alignment from trait definition (not heuristic ident scanning). Brand param derived from trait definition's Kind\_\* bound (not Val impl where clause). Type param ordering follows trait definition order (not alphabetical sort). Inner Apply! macros in self-type elements resolved via `apply_worker`.
- **`m_do!`/`a_do!` codegen**: Updated to use `explicit::bind`/`explicit::map` paths instead of `bind_explicit`/`map_explicit`.

## [0.6.0] - 2026-03-14

### Added

- **`#[kind]` attribute macro**: Ergonomic Kind supertrait bounds for trait definitions. Replaces raw hash-based trait names (e.g., `Kind_cdc7cd43dac7585f`) with `#[kind(type Of<'a, A: 'a>: 'a;)]` annotations that compute the deterministic hash and append the correct Kind trait as a supertrait bound.
- **`Kind` variant in `TraitCategory`**: Proper classification of `Kind_*` traits in trait analysis, with `KIND_PREFIX` constant.
- **Duplicate attribute rejection**: All standalone `#[document_*]` attribute macros (`document_signature`, `document_type_parameters`, `document_parameters`, `document_returns`, `document_examples`) now reject duplicate usage on the same item with clear error messages. Inside `#[document_module]`, the same checks apply via `count_attributes` for traits, methods, and impl blocks. Compile-fail tests added for each macro.
- **Raw `Kind_*` supertrait warning**: `#[document_module]` now emits a compile-time warning when traits use raw `Kind_*` supertraits instead of the `#[kind(...)]` attribute.

### Changed

- **`#[document_signature]` restricted to functions and methods**: Using `#[document_signature]` on a trait definition now produces a clear error message instead of generating a class signature. Class signatures were redundant with `cargo doc`'s native supertrait display.
- **Validation label improvement**: `validate_container_documentation` now includes the trait name in its label (e.g., `Trait 'Functor'` instead of `Trait`).

### Removed

- **`#[document_fields]` macro**: Removed in favor of native `///` doc comments directly on struct fields and enum variants. The macro worker, support module, proc macro export, constant, and integration tests have all been deleted.

### Fixed

- **`WarningEmitter` name collision**: Fixed E0428 name conflicts when multiple `#[document_module]` invocations exist at the same scope by using a global `AtomicUsize` counter instead of a per-instance counter for warning constant names.

## [0.5.0] - 2026-03-13

### Added

- **`m_do!` proc macro**: Monadic do-notation that desugars flat syntax into nested `bind` calls, matching Haskell/PureScript `do` notation. Supports `x <- expr;` binds, `let` bindings, sequence (`expr;`), typed binds (`x: Type <- expr;`), and automatic `pure(...)` rewriting.
- **`a_do!` proc macro**: Applicative do-notation that desugars into `pure`/`map`/`lift2`–`lift5` calls, matching PureScript `ado` notation. Bindings are independent (not monadic), enabling applicative optimizations.
- `#[document_examples]` now handles `#[doc = concat!(...)]` attributes for trait-level doc examples.

### Changed

- Internal module organization: `do_notation` → `m_do`, `ado_notation` → `a_do`.

## [0.4.2] - 2026-03-10

### Changed

- Fixed documentation examples in `lib.rs` to include section headings generated by `#[document_*]` attribute macros.
- Documented `#[document_module]` validation rules and `impl Trait` lint comprehensively.
- Extracted `"no_validation"` string literal to `NO_VALIDATION` constant.

## [0.4.1] - 2026-03-10

### Added

- `#[document_returns]` attribute macro for generating "Returns" documentation sections.
- `#[document_examples]` attribute macro for inserting "Examples" headings and validating that doc comment code blocks contain assertion macros.
- Impl trait lint in `#[document_module]`: detects named generic type parameters that could use `impl Trait` syntax, emitting compile-time warnings. Suppressible with `#[allow_named_generics]`.
- Trait definition support in `#[document_module]` and `#[document_parameters]`, enabling validation and documentation generation for trait methods alongside impl blocks.

### Changed

- `#[document_module]` validation now emits compile-time warnings (via `proc-macro-warning`) instead of hard compile errors, allowing builds to succeed while flagging documentation issues.
- Internal refactoring: extracted constants to dedicated module, generalized method utilities for shared trait/impl processing, adopted `strip_prefix` over manual string slicing.

## [0.4.0] - 2026-02-13

### Added

- **New Macros**:
  - Added `document_fields!` macro for documenting struct/enum fields.
  - Added `document_module!` macro for processing module-level documentation and trait implementations.
- **Error Handling Infrastructure**: Added comprehensive error handling with `thiserror` integration, including proper error propagation and user-friendly compile errors.
- **Type Resolution System**: Added full type resolution infrastructure in `resolution` module (context, impl_key, projection_key, resolver).
- **Analysis Infrastructure**: Added `analysis` module for generic parameter analysis, pattern detection (FnBrand, Apply!), and trait classification.
- **Core Infrastructure**: Added `core` module with configuration management, error handling, and constants.
- **Support Utilities**: Added `support` module with AST utilities, attribute handling, parsing helpers, type visitors, and field documentation support.

### Changed

- **API Breaking - Macro Renames**:
  - Renamed `def_kind!` to `trait_kind!` for clarity.
  - Renamed `hm_signature!` to `document_signature!` for consistency.
  - Renamed `doc_type_params!` to `document_type_parameters!` for consistency.
  - Renamed `doc_params!` to `document_parameters!` for consistency.
- **Dependencies**: Added `visit` and `visit-mut` features to `syn` dependency. Added `thiserror` 2.0 for error handling.
- **README**: Removed detailed macro usage documentation from README (documentation now in rustdoc).
- **Module Structure (Internal Refactoring)**:
  - Reorganized codebase into clear module hierarchy: `analysis`, `codegen`, `core`, `documentation`, `hkt`, `hm`, `resolution`, `support`.
  - Split `hkt` functionality into submodules: `apply`, `associated_type`, `canonicalizer`, `impl_kind`, `input`, `trait_kind`.
  - Split `hm` (Hindley-Milner) into submodules: `ast`, `ast_builder`, `converter`.
  - Split `documentation` into submodules: `document_fields`, `document_module`, `document_parameters`, `document_signature`, `document_type_parameters`, `generation`, `templates`.
  - Split `codegen` into submodules for re-export generation.
  - Consolidated core types and error handling in `core` module.
- **Error Handling**: All macro workers now return `Result` types with proper error propagation instead of panicking.
- **Documentation**: Added `#![warn(missing_docs)]` and comprehensive module-level documentation throughout the crate.

### Removed

- **Old Module Structure**: Removed flat modules in favor of hierarchical organization: `canonicalize`, `def_kind`, `doc_params`, `doc_type_params`, `doc_utils`, `function_utils`, `generate`, `parse`, `re_export` (functionality moved to new module structure).

## [0.3.3] - 2026-02-02

### Added

- **Documentation Macros**:
  - Added `hm_signature` macro for generating Hindley-Milner type signatures.
  - Added `doc_type_params` macro for documenting generic type parameters.
  - Added `doc_params` macro for documenting function parameters.

### Changed

- **`def_kind!` Attributes**: Updated `def_kind!` macro to support attributes (e.g., doc comments) on associated type definitions.

## [0.3.2] - 2026-01-27

### Changed

- **`impl_kind!` Macro**: Updated to support `where` clauses in associated type definitions, enabling more complex type constraints in Kind implementations.

## [0.3.1] - 2026-01-23

### Changed

- **Macro Documentation**: Enhanced documentation for `Kind!`, `def_kind!`, `impl_kind!`, `Apply!`, `generate_function_re_exports!`, and `generate_trait_re_exports!` with detailed "Syntax", "Parameters", "Generates", and "Examples" sections.

## [0.3.0] - 2026-01-23

### Added

- **`generate_trait_re_exports!` Macro**: Added a new procedural macro to automatically generate `pub use` statements for public traits in a directory.

### Changed

- **`generate_reexports!` Rename (API Breaking)**:
  - Renamed `generate_reexports!` to `generate_function_re_exports!` to distinguish it from the new trait re-export macro.

## [0.2.2] - 2026-01-19

### Added

- **`generate_reexports!` Macro**: Added a new procedural macro to automatically generate `pub use` statements for public free functions in a directory, with support for aliasing (e.g., renaming `new` to `fn_new`).

### Fixed

- **`def_kind!` Documentation**: Fixed a bug where type parameter bounds were incorrectly formatted in the generated documentation (e.g., showing `A: A : 'a.bounds` instead of `A: 'a`).

## [0.2.1]

### Documentation

- **Terminology**: Standardized "Kind" to "`Kind`" across all crate documentation.
- **`impl_kind!`**: Updated documentation to clarify usage with bounds and multiple associated types.

## [0.2.0] - 2026-01-18

### Changed

- **`Apply!` Macro Refactor (API Breaking)**:
  - Removed "Explicit Kind Mode" and "Unified Signature Mode" in favor of a single, more explicit syntax.
  - New syntax: `Apply!(<Brand as Kind!(KindSignature)>::AssocType<Args>)`.
  - This syntax mimics fully qualified paths while allowing inline anonymous Kind trait definitions.
- **Multiple Associated Types (API Breaking)**:
  - Updated `Kind!` and `def_kind!` to support defining multiple associated types (e.g., `type Of<T>; type SendOf<T>;`).
  - Updated `impl_kind!` to support implementing multiple associated types.
  - Changed input syntax for `def_kind!` to use standard Rust associated type syntax (e.g., `def_kind!(type Of<T>;)` instead of `def_kind!((), (T), ())`).
- **Canonicalization (API Breaking)**:
  - Enhanced canonicalization to include associated type names and sort them for determinism.
  - Improved type parameter mapping (e.g., `T` -> `T0`) for robust hash generation.
- **Testing**:
  - Updated property-based tests to reflect the new `KindInput` syntax and `Canonicalizer` API.
  - Removed obsolete UI tests for deprecated `Apply!` modes.
- **Documentation**:
  - Corrected `def_kind!` macro documentation to reflect the correct input syntax.
  - Clarified `Apply!` macro documentation regarding the Kind trait reference syntax.
  - Added comprehensive examples for `Kind!`, `def_kind!`, and `impl_kind!` macros.
  - Updated module-level documentation to include all exported macros.

---

## [0.1.1] - 2026-01-16

### Added

- **`Apply!` Macro Enhancement**:
  - Added optional `output` parameter for accessing associated types other than `Of` (e.g., `SendOf`).
  - Example: `Apply!(brand: ArcFnBrand, kind: SendCloneableFn, output: SendOf, lifetimes: ('a), types: (i32, i32))`.
- **Testing**:
  - Added UI test `apply_invalid_output.rs` for invalid output parameter error messages.
  - Added unit tests for `output` parameter parsing and code generation.
- **Documentation**:
  - Updated README with `output` parameter documentation and examples.

---

## [0.1.0] - 2026-01-15

### Added

- **`def_kind!` Macro**: Procedural macro to define Kind traits with a specific signature (lifetimes, type parameters with bounds, and output bounds). Generates hash-based trait names for determinism.
- **`impl_kind!` Macro**: Procedural macro to implement a Kind trait for a brand type. Infers the correct Kind trait from the GAT signature.
- **`Apply!` Macro**: Procedural macro for type application - projects a brand to its concrete type. Supports unified signature syntax (`signature: ('a, T: Clone)`) and explicit kind mode (`kind: K, lifetimes: (...), types: (...)`).
- **Canonicalization Module**: Robust canonicalization of type bounds including:
  - Full path preservation (`std::fmt::Debug` → `tstd::fmt::Debug`)
  - Generic argument handling (`Iterator<Item = T>`)
  - Fn trait bounds (`Fn(A) -> B`)
  - Lifetime normalization (positional naming)
- **Hash-Based Naming**: Uses `rapidhash` for deterministic 64-bit Kind trait names (`Kind_{hash:016x}`).
- **Property Tests**: Comprehensive quickcheck-based tests for:
  - Hash determinism
  - Canonicalization equivalence
  - Bound order independence
  - Lifetime name independence
- **Compile-Fail Tests**: UI tests via `trybuild` for helpful error messages on invalid input.
- **Integration Tests**: End-to-end tests for all macro features.
