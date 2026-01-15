# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-15

### Added

- **`fp-macros` Crate**: New procedural macro crate for HKT boilerplate reduction.
  - **`def_kind!`**: Macro for defining Kind traits with specified lifetimes, type parameters, and bounds.
  - **`impl_kind!`**: Macro for implementing Kind traits for brand types. Automatically infers the correct Kind trait from the GAT signature.
  - **`Apply!`**: Macro for projecting brand types to concrete types using named parameters (`brand`, `signature`, `lifetimes`, `types`).
- **Hash-Based Kind Naming**: Kind traits now use deterministic hash-based names (`Kind_{hash:016x}`) generated from canonicalized signatures, ensuring consistent naming across compilations.
- **Canonicalization System**: Full path preservation and generic argument handling in trait bounds, including support for `Fn`, `FnMut`, `FnOnce` bounds.
- **Property-Based Tests**: Comprehensive quickcheck tests for hash determinism, canonicalization equivalence, bound order independence, and lifetime name independence.
- **Compile-Fail Tests**: `trybuild` tests verifying helpful error messages for invalid macro inputs.

### Changed

- **Library Migration**: All type implementations (`Option`, `Result`, `Vec`, `Identity`, etc.) now use `impl_kind!` macro instead of manual trait implementations.
- **Documentation**: README updated with macro usage examples and corrected directory references (`fp-library/src/kinds` instead of `fp-library/src/hkt`).

## [0.1.0] - 2026-01-12

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
