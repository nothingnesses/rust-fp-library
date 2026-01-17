# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Data Shrinking Typeclasses**:
  - Added `Compactable`, `Filterable` and `Witherable` typeclasses for discarding values in contexts.

### Changed
- **`Apply!` Macro Migration**:
  - Migrated all usages of `Apply!` to the new syntax: `Apply!(<Brand as Kind!(KindSignature)>::AssocType<Args>)`.
  - Converted usages of the deprecated "Explicit Kind Mode" to standard Rust syntax (e.g., `<Brand as Kind>::Of<Args>`).
- **Kind Trait Refactor**:
  - Updated `Kind` traits to support multiple associated types (e.g., `Of`, `SendOf`).
  - Updated `def_kind!` and `impl_kind!` macros to use standard Rust syntax for associated type definitions.
  - Updated internal Kind trait hashes to reflect the new canonicalization logic.

---

## [0.3.0] - 2026-01-16

### Added

- **Thread Safety and Parallelism**:
  - Added `SendClonableFn` extension trait for thread-safe function wrappers with `Send + Sync` bounds.
  - Added `ParFoldable` trait providing `par_fold_map` and `par_fold_right` for parallel folding operations.
  - Added `SendEndofunction` type for thread-safe endofunctions using `ArcFnBrand`.
  - Implemented `SendClonableFn` for `ArcFnBrand` with `new_send` constructor.
  - Implemented `ParFoldable` for `VecBrand` (with optional Rayon parallelism) and `OptionBrand`.
- **Feature Flags**:
  - Added optional `rayon` feature (`rayon = ["dep:rayon"]`) enabling parallel execution in `VecBrand::par_fold_map`.
- **Testing Infrastructure**:
  - Added compile-fail tests using `trybuild` to verify thread safety error messages.
  - Added UI tests for `SendClonableFn`: `new_send_not_send.rs`, `new_send_not_sync.rs`, `rc_fn_not_send.rs`.
  - Added property-based tests for `ParFoldable` in `tests/property_tests.rs`.
  - Added thread safety integration tests in `tests/thread_safety.rs`.

### Changed

- **API Breaking Changes**:
  - `Foldable` trait methods (`fold_right`, `fold_left`, `fold_map`) now require a `FnBrand` type parameter.
  - `Traversable::traverse` reorders function parameter `Func` to come before `A` and `B`.
  - `Semiapplicative::apply` and `Defer::defer` reorder type parameters to put `FnBrand` first.
  - `Semimonad::bind` and `Lift::lift2` reorder type parameters to put function type `F` first.
- **Parameter Naming**:
  - Renamed internal parameters `f` to `func` and `init` to `initial` in folding traits for clarity.
  - Renamed `ClonableFnBrand` type parameter to `FnBrand` across the library.
- **Documentation**:
  - Updated function and method documentation in `fp-library/src/classes/` to follow a consistent format with detailed sections for type signatures, type parameters, parameters, returns, and examples.
  - Rewrote module-level documentation in `fp-library/src/classes.rs` for clarity and accuracy regarding Brand types and HKT simulation.
  - Added missing module-level documentation to all type class modules.
  - Standardized law section headers from `# Laws` to `### Laws`.
  - Updated README with new "Thread Safety and Parallelism" section and usage examples.
  - Updated README dependency version from `0.2` to `0.3`.
- **Dependencies**:
  - Added `rayon = "1.11"` as optional dependency.
  - Added `trybuild = "1.0"` as dev-dependency for compile-fail tests.
  - Changed `fp-macros` dependency version from `"0.1.0"` to `"0.1"` for semver compatibility.

---

## [0.2.0] - 2026-01-15

### Changed

- **`Apply!` Syntax**: Simplified `Apply!` macro syntax. The `signature` parameter now accepts a unified syntax that includes both schema and concrete values (e.g., `signature: ('a, T: Clone)`). The `lifetimes` and `types` parameters are no longer accepted when using `signature`.
- **HKT Documentation**: Updated README with `impl_kind!` macro usage example for defining Kind implementations.
- **Project Structure**: Fixed documentation to reflect correct module paths (`fp-library/src/kinds` instead of `fp-library/src/hkt`).

---

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
