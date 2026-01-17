# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Multiple Associated Types**:
  - Updated `Kind!` and `def_kind!` to support defining multiple associated types (e.g., `type Of<T>; type SendOf<T>;`).
  - Updated `impl_kind!` to support implementing multiple associated types.
  - Changed input syntax for `def_kind!` to use standard Rust associated type syntax (e.g., `def_kind!(type Of<T>;)` instead of `def_kind!((), (T), ())`).
- **Canonicalization**:
  - Enhanced canonicalization to include associated type names and sort them for determinism.
  - Improved type parameter mapping (e.g., `T` -> `T0`) for robust hash generation.
- **Testing**:
  - Updated property-based tests to reflect the new `KindInput` syntax and `Canonicalizer` API.

---

## [0.1.1] - 2026-01-16

### Added

- **`Apply!` Macro Enhancement**:
  - Added optional `output` parameter for accessing associated types other than `Of` (e.g., `SendOf`).
  - Example: `Apply!(brand: ArcFnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (i32, i32))`.
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
