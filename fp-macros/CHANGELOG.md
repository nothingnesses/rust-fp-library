# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
