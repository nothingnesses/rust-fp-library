# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased fp-library 0.3.0]

### Added

- **Thread Safety and Parallelism**:
  - Added `SendClonableFn` extension trait for thread-safe function wrappers.
  - Added `ParFoldable` trait for parallel folding operations.
  - Added `SendEndofunction` type for thread-safe endofunctions.
  - Implemented `SendClonableFn` for `ArcFnBrand`.
  - Implemented `ParFoldable` for `VecBrand` and `OptionBrand`.
  - Added optional `rayon` feature for parallel execution in `VecBrand`.

### Changed

- Update function and method documentation in `fp-library/src/classes/` to follow a consistent format with detailed sections for type signatures, parameters, and examples.
- Rewrite module-level documentation in `fp-library/src/classes.rs` for clarity and accuracy regarding Brand types and HKT simulation.
- Add missing module-level documentation to all type class modules in `fp-library/src/classes/`.

## [Unreleased fp-macros 0.1.1]

### Added

- Updated `Apply!` to support optional `output` parameter for accessing associated types other than `Of` (e.g., `SendOf`).

## [fp-library 0.2.0] - 2026-01-15

### Changed

- **`Apply!` Syntax**: Simplified `Apply!` macro syntax. The `signature` parameter now accepts a unified syntax that includes both schema and concrete values (e.g., `signature: ('a, T: Clone)`). The `lifetimes` and `types` parameters are no longer accepted when using `signature`.
- **HKT Documentation**: Updated README with `impl_kind!` macro usage example for defining Kind implementations.
- **Project Structure**: Fixed documentation to reflect correct module paths (`fp-library/src/kinds` instead of `fp-library/src/hkt`).

---

## [fp-macros 0.1.0] - 2026-01-15

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

---

## [fp-library 0.1.0] - 2026-01-12

### Added

- **Zero-Cost Abstractions**: The library has been completely refactored to use uncurried, monomorphized type classes. This eliminates the overhead of intermediate closures and dynamic dispatch for most operations.
- **`Lift` Trait**: A new trait for lifting binary functions into a context (`lift2`). This enables zero-cost combination of contexts without creating intermediate closures.
- **`Kind1L1T`**: Upgraded HKT infrastructure to support types with lifetimes (like `Lazy`).
- **`VecBrand::construct` / `deconstruct`**: Re-introduced uncurried versions of these helper methods.
- **Tests**: Added property-based tests for `Pair`, `Endomorphism`, `Endofunction` and unit tests for `OnceCell`, `OnceLock`.

### Changed

- **Uncurried API**: All type class methods (`map`, `bind`, `apply`, `fold_right`, etc.) are now uncurried.
  - `map(f)(fa)` -> `map(f, fa)`
  - `bind(ma)(f)` -> `bind(ma, f)`
- **Generic Bounds**: Trait methods now use generic `F: Fn(A) -> B` bounds instead of `ClonableFn` where possible, enabling inlining and monomorphization.
- **`Lazy`**: Now implements `Semigroup`, `Monoid`, and `Defer`. It does _not_ implement `Functor` or `Monad` due to `Clone` requirements for memoization.
- **`Endofunction` / `Endomorphism`**: Updated to work with the new uncurried `Semigroup` trait while preserving type erasure for composition.

### Removed

- **Legacy v1 API**: The entire curried API (formerly under `classes`, `types`, `functions`) has been removed.
- **Feature Flags**: `v1` and `v2` feature flags have been removed. The library now provides a single, unified API.
- **`construct` / `deconstruct` (Curried)**: The curried versions were removed in favor of the uncurried ones.

### Fixed

- **`clippy::multiple_bound_locations`**: Resolved warnings in core traits.
- **Internal Imports**: Fixed all internal imports to reflect the new module structure.
- **Brand Types**: Brand types (e.g., `OptionBrand`, `VecBrand`) have been moved to `crate::brands` and are no longer re-exported from `crate::types`. Users should import them from `fp_library::brands`.
