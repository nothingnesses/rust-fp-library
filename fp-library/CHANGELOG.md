# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.13.0] - 2026-03-13

### Added
- **`WithIndex` supertrait**: New trait encoding the functional dependency `f -> i` from PureScript, providing an associated `Index` type that uniquely determines the index type for a brand. Prevents inconsistent index types across `FunctorWithIndex`, `FoldableWithIndex`, and `TraversableWithIndex` for the same brand.
- **Parallel trait hierarchy**: A full set of parallel type classes mirroring the sequential ones, all accepting plain `impl Fn + Send + Sync` closures with no wrapper types required:
  - `ParFunctor` with `par_map`.
  - `ParCompactable` with `par_compact` and `par_separate`.
  - `ParFilterable` (extends `ParFunctor + ParCompactable`) with `par_filter_map` and `par_filter`, including default implementations derived from `par_map` + `par_compact`.
  - `ParFunctorWithIndex` (extends `ParFunctor + FunctorWithIndex`) with `par_map_with_index`.
  - `ParFoldableWithIndex` (extends `ParFoldable + FoldableWithIndex`) with `par_fold_map_with_index`.
- **Parallel trait implementations** for `VecBrand` and `CatListBrand`: `ParFunctor`, `ParCompactable`, `ParFilterable`, `ParFoldable`, `ParFunctorWithIndex`, `ParFoldableWithIndex`.
- **`WithIndex` implementations**: `VecBrand` (`Index = usize`), `CatListBrand` (`Index = usize`), `OptionBrand` (`Index = ()`).
- **Inherent parallel methods** on `Vec` and `CatList`: `par_map`, `par_compact`, `par_separate`, `par_filter_map`, `par_filter`, `par_fold_map`, `par_map_with_index`, `par_fold_map_with_index`.

### Changed
- **`ParFoldable` redesigned (API Breaking)**: The trait now accepts plain `impl Fn(A) -> M + Send + Sync` closures instead of requiring `SendCloneableFn` wrapper types and an `FnBrand` type parameter. The `A: Clone` bound has been removed, and `M: Sync` relaxed to `M: Send`. The `Foldable` supertrait requirement has been replaced with `Kind`.
- **`*WithIndex` traits refactored (API Breaking)**: `FunctorWithIndex`, `FoldableWithIndex`, and `TraversableWithIndex` no longer take a generic index type parameter `<I>`. Instead, the index type is obtained from the `WithIndex` supertrait's associated `Index` type. Trait bounds like `Brand: FunctorWithIndex<usize>` become `Brand: FunctorWithIndex<Index = usize>`.
- **Indexed optics trait bounds updated**: All indexed optic constructors updated from `FunctorWithIndex<I>` / `FoldableWithIndex<I>` / `TraversableWithIndex<I>` bounds to use associated type equality (e.g., `FoldableWithIndex<Index = I>`).

### Removed
- **`par_fold_right` (API Breaking)**: Removed `ParFoldable::par_fold_right` method and its free function. The endofunction encoding required for a general right fold has a sequential application step, making it not genuinely parallel. Use `par_fold_map` with a commutative `Monoid` instead.
- **`SendEndofunction` (API Breaking)**: Removed the thread-safe endofunction wrapper type and its module. This type was used internally by the removed `par_fold_right`.
- **`ParFoldable` implementations for single-element types (API Breaking)**: Removed from `IdentityBrand`, `OptionBrand`, `PairFirstAppliedBrand`, `PairSecondAppliedBrand`, `ResultErrAppliedBrand`, `ResultOkAppliedBrand`, `Tuple1Brand`, `Tuple2FirstAppliedBrand`, `Tuple2SecondAppliedBrand`, `StepLoopAppliedBrand`, and `StepDoneAppliedBrand`, where parallelism provided no benefit.

## [0.12.0] - 2026-03-13

### Added
- **`Alt` / `Plus` / `Alternative` type classes**: Associative choice (`alt`), identity for choice (`plus_empty`), and `Alternative` (blanket impl for `Applicative + Plus`). Implementations for `Option`, `Vec`, `CatList`.
- **Numeric algebra hierarchy**: `Semiring`, `Ring`, `CommutativeRing`, `EuclideanRing` (with `gcd`/`lcm`), `DivisionRing`, `Field`, `HeytingAlgebra`. Instances for all Rust numeric primitives and `bool`. Integer instances use wrapping arithmetic.
- **Monoid newtype wrappers**: `Additive`, `Multiplicative`, `Conjunctive`, `Disjunctive`, `First`, `Last`, `Dual` with `Semigroup`/`Monoid` instances.
- **`Semimonad` derived combinators**: `bind_flipped`, `join`, `compose_kleisli`, `compose_kleisli_flipped`.
- **`Monad` derived combinators**: `if_m`, `when_m`, `unless_m`.
- **`Applicative` combinators**: `when`, `unless`.
- **`Monoid` combinator**: `power` (binary exponentiation via repeated `append`).
- **`Lift` functions**: `lift3`, `lift4`, `lift5` for higher-arity lifting via nested `lift2`.
- **`on` function**: Apply a binary function after projecting both arguments (`on(f, g, x, y) = f(g(x), g(y))`).
- **Law-checking doc examples** across 22 type class traits: `Applicative`, `Bifunctor`, `Bifoldable`, `Bitraversable`, `Category`, `Choice`, `Compactable`, `Contravariant`, `Filterable`, `Foldable`, `FoldableWithIndex`, `Functor`, `FunctorWithIndex`, `Monad`, `Monoid`, `Semiapplicative`, `Semigroup`, `Semigroupoid`, `Strong`, `Traversable`, `TraversableWithIndex`, `Witherable`.

### Changed
- Updated doc examples and imports to use renamed `m_do!`/`a_do!` macros (from `m!`/`ado!`).

### Fixed
- Profunctor composition law: corrected `g` argument order.
- 15 broken rustdoc links in newtype wrapper module docs.

## [0.11.1] - 2026-03-10

### Added
- **`Pipe` trait**: Left-to-right function application via method syntax (`value.pipe(f)`), similar to PureScript's `#` or Haskell's `&` operator.
- **`Bifoldable` implementations**: Added `Bifoldable` for `PairBrand` and `StepBrand`.
- **`Bitraversable` implementations**: Added `Bitraversable` for `PairBrand` and `StepBrand`.
- **Indexed type class implementations for `CatList`**: Added `FunctorWithIndex<usize>`, `FoldableWithIndex<usize>`, and `TraversableWithIndex<usize>` for `CatListBrand`.
- **Inherent methods**: Added inherent methods to data types, with type class trait implementations delegating to them:
  - `Identity`: `map`, `lift2`, `apply`, `bind`, `fold_right`, `fold_left`, `fold_map`, `traverse`, `sequence`.
  - `Pair`: `bimap`, `map_first`, `map_second`, `fold`, `bi_fold_right`, `bi_fold_left`, `bi_fold_map`, `bi_traverse`, `bind`, `bind_first`.
  - `Step`: `bimap`, `map_loop`, `map_done`, `bi_fold_right`, `bi_fold_left`, `bi_fold_map`, `bi_traverse`, `fold_right`, `fold_left`, `fold_map`, `bind`, `bind_loop`.
  - `ConstVal`: `map`, `lift2`, `apply_first`, `apply_second`, `pure`.
  - `CatList`: `pure`, `map`, `bind`, `fold_right`, `fold_left`, `fold_map`, `map_with_index`, `fold_map_with_index`, `traverse_with_index`.
  - `Lazy`: `ref_map`.

## [0.11.0] - 2026-03-10

### Changed
- **`ConstBrand` Location (API Breaking)**: Moved `ConstBrand` from `types::const_val` to the `brands` module, consistent with all other brand types. Import path changed from `fp_library::types::const_val::ConstBrand` to `fp_library::brands::ConstBrand`.
- Updated feature listings in `lib.rs` and `README.md` to accurately reflect all type classes, optics, and data types in the codebase.
- Expanded optics module documentation: complete comparison table with all optic types, corrected struct type parameter names (`P` → `PointerBrand`), and comprehensive module organization listing.

### Fixed
- Incorrect turbofish arity for `map` in README (4 type parameters → 3).
- Outdated dependency version in README (`0.9` → `0.10`).
- Doc examples for `Const` type class implementations updated to use correct `ConstBrand` import path from `brands`.

## [0.10.0] - 2026-03-10

### Added
- **Profunctor Optics System**: Full profunctor-encoded optics library with type-safe composition:
  - **Optic types**: `Lens`/`LensPrime`, `Prism`/`PrismPrime`, `Iso`/`IsoPrime`, `AffineTraversal`/`AffineTraversalPrime`, `Traversal`/`TraversalPrime`, `Fold`/`FoldPrime`, `Getter`/`GetterPrime`, `Setter`/`SetterPrime`, `Grate`/`GratePrime`, `Review`/`ReviewPrime`.
  - **Indexed optics**: `IndexedLens`/`IndexedLensPrime`, `IndexedTraversal`/`IndexedTraversalPrime`, `IndexedFold`/`IndexedFoldPrime`, `IndexedGetter`/`IndexedGetterPrime`, `IndexedSetter`/`IndexedSetterPrime`.
  - **Composed optic**: Type-safe composition of any two compatible optics via `Composed`.
  - **Rank-2 optic traits**: `IsoOptic`, `LensOptic`, `PrismOptic`, `AffineTraversalOptic`, `TraversalOptic`, `GetterOptic`, `FoldOptic`, `SetterOptic`, `GrateOptic`, `ReviewOptic`, plus indexed variants.
  - **Internal profunctors**: `Exchange`, `Market`, `Forget`, `Shop`, `Stall`, `Tagged`, `Grating`, `Zipping`, `Reverse`, `Bazaar`, `Indexed`.
  - **Optics helper functions**: `optics_view`, `optics_set`, `optics_over`, `optics_preview`, `optics_review`, `optics_compose`, and indexed variants (`optics_indexed_view`, `optics_indexed_fold_map`, etc.).
  - **`FoldFunc` trait**: Non-allocating fold evaluation, replacing the `Vec`-based intermediate collection in `Fold`.
  - **`TraversalFunc`** and **`IndexedTraversalFunc`** traits for traversal internals.
- **Profunctor Type Classes**:
  - `Profunctor` with `dimap`, `lmap`, `rmap`.
  - `Strong` (product lifting), `Choice` (sum lifting).
  - `Costrong`, `Cochoice` (dual profunctor operations).
  - `Closed` (exponentiation), parameterized over `CloneableFn` brand.
  - `Wander` (traversal lifting).
- **New Type Classes**:
  - `Arrow` for lifting pure functions into profunctors.
  - `Bifoldable` and `Bitraversable` for two-argument type constructors.
  - `FunctorWithIndex`, `FoldableWithIndex`, `TraversableWithIndex` for index-carrying operations.
  - `Contravariant` for contravariant functors.
- **`Bazaar`** type for characterizing traversals internally.
- **`TakeCell`** abstraction added to `RefCountedPointer` for single-use extraction.
- **`Zipping`** profunctor for `Grate` zip operations.
- **Clippy configuration**: Workspace-level warnings for panicky code (`unwrap_used`, `expect_used`, `indexing_slicing`, `panic`, `todo`, `unimplemented`, `unreachable`).

### Changed
- **Brand Renames (API Breaking)**: Partially-applied brands renamed to use "Applied" convention:
  - `PairWithFirstBrand` -> `PairFirstAppliedBrand`, `PairWithSecondBrand` -> `PairSecondAppliedBrand`.
  - `ResultWithErrBrand` -> `ResultErrAppliedBrand`, `ResultWithOkBrand` -> `ResultOkAppliedBrand`.
  - `StepWithLoopBrand` -> `StepLoopAppliedBrand`, `StepWithDoneBrand` -> `StepDoneAppliedBrand`.
  - `TryThunkWithErrBrand` -> `TryThunkErrAppliedBrand`, `TryThunkWithOkBrand` -> `TryThunkOkAppliedBrand`.
  - `Tuple2WithFirstBrand` -> `Tuple2FirstAppliedBrand`, `Tuple2WithSecondBrand` -> `Tuple2SecondAppliedBrand`.
- **`impl Trait` Migration (API Breaking)**: Function/closure type parameters that appear only once in a signature now use `impl Trait` instead of named generics. This reduces generic arity on free functions across all type class traits (e.g., `map`, `bind`, `fold_right`, `compose`, `flip`). Call sites using explicit turbofish syntax need fewer type arguments.
- **`Re` renamed to `Reverse`**, `re` to `reverse`.
- **`helpers` module renamed to `functions`**.
- **Optics module restructured**: Optics traits moved to `classes/optics/`, optics brands moved to `brands/optics/`.
- **Optics constructors cleaned up**: Legacy `new` constructors removed from `Lens`, `LensPrime`, `AffineTraversal`, `AffineTraversalPrime`, `PrismPrime`. Convenience constructors (`from_view_set`, `from_preview_set`, `from_option`) added for `Clone` types.
- **Unnecessary `Clone` bounds removed** from `Lens`, `Prism`, `AffineTraversal`, and `Grate` operations.
- **`Forget` profunctor parameterized** over pointer brand for cloneability, fixing runtime panics when using Traversals as Folds.
- **`Closed` trait generalized** to accept a `CloneableFn` brand parameter instead of hardcoded `Box`.
- **Documentation**: All `classes/` modules wrapped with `#[document_module]` for automated validation and generation.

### Fixed
- Runtime panic when using Traversals as Folds (Forget profunctor now cloneable via `FnBrand`).
- Indexed optics composition and evaluation issues.
- Unnecessary `S: Clone` bound on `PrismPrime` trait implementations.

## [0.9.0] - 2026-02-13

### Added
- **Tuple Types**:
  - Added `Tuple1Brand` for 1-tuples `(A,)` with full type class implementations (`Functor`, `Applicative`, `Monad`, `Foldable`, `Traversable`, `ParFoldable`).
  - Added `Tuple2Brand` for 2-tuples `(A, B)` with Bifunctor over both positions.
  - Added `Tuple2WithFirstBrand<First>` for 2-tuples with first element fixed, providing Functor over the second element.
  - Added `Tuple2WithSecondBrand<Second>` for 2-tuples with second element fixed, providing Functor over the first element.
- **Feature Flags**:
  - Added optional `serde` feature for serialization/deserialization support on pure data types.

### Changed
- **API Breaking - Macro Exports**:
  - Updated re-export from `def_kind!` to `trait_kind!` to match the renamed macro in `fp-macros`.
- **Documentation (Non-Breaking)**:
  - Comprehensive documentation improvements across all type classes and types using the new documentation macros (`document_signature!`, `document_type_parameters!`, `document_parameters!`, `document_fields!`, `document_module!`).
  - Enhanced inline documentation examples and type signatures throughout the library.
  - Improved module-level documentation for better API discoverability.

### Removed
- **Brand Types (API Breaking)**:
  - Removed `BoxBrand` (unused).
  - Removed `EndofunctionBrand` (internal implementation detail).
  - Removed `EndomorphismBrand` (internal implementation detail).
  - Removed `FreeBrand` (internal implementation detail).

## [0.8.0] - 2026-02-02

### Added
- **Macros**: Exported `doc_type_params`, `doc_params`, and `hm_signature` from `fp_macros`.
- **Evaluable**: Added `Evaluable` trait for types that can be evaluated to a value (replacing `Runnable`).
- **Deferrable**: Implemented `Deferrable` for `Free`.

### Changed
- **Free Monad (API Breaking)**:
  - Renamed `Free::run` to `Free::evaluate`.
  - Renamed `Free::roll` to `Free::wrap`.
  - Refactored `Free` to use `Evaluable` instead of `Runnable`.
  - Renamed internal types `Val` to `TypeErasedValue` and `Cont` to `Continuation`.

### Removed
- **Runnable (API Breaking)**: Removed `Runnable` trait in favor of `Evaluable`.

## [0.7.0] - 2026-01-27

### Added
- **Lazy Evaluation Revamp**:
  - **`Lazy` / `TryLazy`**: Added `Lazy` and `TryLazy` types for shared memoization (renamed from `Memo`/`TryMemo`). Supports `Rc` and `Arc` backing via `LazyConfig`.
  - **`Trampoline` / `TryTrampoline`**: Added `Trampoline` and `TryTrampoline` for stack-safe, non-memoized computations using `Free` monad (renamed from `Task`/`TryTask`).
  - **`Thunk` / `TryThunk`**: Added `Thunk` and `TryThunk` for HKT-compatible deferred computations (renamed from `Eval`/`TryEval`).
  - **`Free` Monad**: Added `Free` monad implementation with `CatList`-based O(1) bind for stack safety.
  - **Data Structure**: Added `CatList` (concatenation list) with O(1) operations.
  - **Traits**:
    - Added `MonadRec` trait for stack-safe tail recursion.
    - Added `RefFunctor` trait for mapping over types that yield references.
    - Added `Bifunctor` trait for mapping over two type arguments.
    - Added `Runnable` trait for types that can be executed to produce a value.
- **Benchmarks**: Added benchmarks for `CatList` and missing trait methods.

### Changed
- **Lazy Evaluation Revamp (API Breaking)**:
  - **Renaming**:
    - Renamed `Memo`/`TryMemo` to `Lazy`/`TryLazy` to align with industry-standard terminology for memoized lazy values.
    - Renamed `Eval`/`TryEval` to `Thunk`/`TryThunk` for more precise terminology (non-memoized deferred computations).
    - Renamed `Task`/`TryTask` to `Trampoline`/`TryTrampoline` to avoid confusion with async tasks and highlight stack-safety mechanism.
    - Renamed `MemoConfig` to `LazyConfig`, `RcMemoConfig` to `RcLazyConfig`, `ArcMemoConfig` to `ArcLazyConfig`.
    - Renamed `RcMemo`/`ArcMemo` to `RcLazy`/`ArcLazy`, `RcTryMemo`/`ArcTryMemo` to `RcTryLazy`/`ArcTryLazy`.
    - Renamed `EvalBrand` to `ThunkBrand`, `MemoBrand` to `LazyBrand`.
    - Renamed `Trampoline::now` to `Trampoline::pure`, `Trampoline::later` to `Trampoline::new`.
    - Renamed `TryTrampoline::try_later` to `TryTrampoline::new`.
    - Renamed `Thunk::force` to `Thunk::run`.
    - Renamed `flat_map` to `bind` in `Trampoline`, `Thunk`, `Free` and their "Try" variants.
  - **Conversions**: Replaced ad-hoc conversion methods (`from_memo`, `into_try`, etc.) with standard `From` trait implementations.
  - **Step**: Added comprehensive typeclass implementations (`Functor`, `Bifunctor`, `Foldable`, etc.) for `Step`.
  - **`Lazy` Lifetimes**: Refactored `Lazy` to support lifetimes, removing the strict `'static` requirement.
- **Documentation**:
  - Updated `docs/architecture.md` and `README.md` to reflect the new distinction between `Lazy` (shared caching), `Trampoline` (stack-safe computation), and `Thunk` (HKT-compatible).

### Removed
- **Lazy Evaluation Revamp (API Breaking)**:
  - Removed old `Lazy`, `OnceCell`, `OnceLock` types and their associated brands (replaced by new `Lazy` type).
  - Removed `TrySemigroup` and `TryMonoid` traits.

## [0.6.1] - 2026-01-23

### Added
- **Exports**: Exported `LazyConfig` from `fp_library::types` to allow users to implement custom lazy configurations.

### Changed
- **Refactor**: Renamed internal type parameter `FnBrand_` to `FnBrand` in `LazyDefer` and `Defer` implementations for consistency.

## [0.6.0] - 2026-01-23

### Added
- **Pointer Abstraction**:
  - Added `Pointer`, `RefCountedPointer`, and `SendRefCountedPointer` traits for abstracting over smart pointers (Rc/Arc).
  - Added `UnsizedCoercible` and `SendUnsizedCoercible` traits for function coercion.
  - Added `RcBrand` and `ArcBrand` implementations in `src/types/rc_ptr.rs` and `src/types/arc_ptr.rs`.
  - Added `FnBrand<P>` generic implementation to replace `RcFnBrand` and `ArcFnBrand`.
- **Lazy**:
  - Added `RcLazy` and `ArcLazy` type aliases.
  - Added `LazyError` for thread-safe panic propagation with `panic_message` method.
  - Added `force_or_panic` and other convenience methods to `Lazy`.
  - Implemented `TrySemigroup`, `TryMonoid`, and `SendDefer` for `Lazy`.
  - Added `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default` derives to `LazyError`.
- **Free Functions**:
  - Added free function wrappers for `ThunkWrapper` and `UnsizedCoercible`.

### Changed
- **Renames (API Breaking)**:
  - Renamed `clonable` to `cloneable` in all filenames and identifiers (e.g., `CloneableFn`, `SendCloneableFn`).
  - Renamed `coerce_fn_send` to `coerce_send_fn`.
  - Renamed creation functions in `src/functions.rs`:
    - `pointer_new` -> `new`
    - `ref_counted_new` -> `cloneable_new`
    - `send_ref_counted_new` -> `send_new`
  - Renamed `ThunkWrapper::new_cell` to `ThunkWrapper::new`.
- **Lazy Refactor (API Breaking)**:
  - Refactored `Lazy` to use shared memoization semantics (Haskell-like) using `RefCountedPointer`.
  - Refactored `Lazy` to use `LazySemigroup`, `LazyMonoid`, and `LazyDefer` helper traits.
- **Module Structure**:
  - Split pointer traits into separate modules in `src/classes/`.
  - Moved `RcBrand` and `ArcBrand` to `src/types/rc_ptr.rs` and `src/types/arc_ptr.rs`.
- **Documentation**:
  - Standardized inline documentation examples to use free functions and turbofish syntax.
  - Updated architecture documentation.

### Removed
- **Legacy Types**:
  - Removed `RcFnBrand` and `ArcFnBrand` in favor of generic `FnBrand<P>`.

## [0.5.0] - 2026-01-19

### Added
- **Architecture Documentation**: Added `docs/architecture.md` detailing module organization, type parameter ordering, and documentation standards.
- **README**: Added `Function`, `CloneableFn`, `SendCloneableFn`, and `ParFoldable` to the features list.

### Changed
- **Type Parameter Ordering (API Breaking)**:
  - Reordered type parameters across the entire library to prioritize uninferable types (e.g., return types) over inferable types (e.g., input types, function types). This improves ergonomics when using turbofish syntax.
  - **Functor**: `map<B, A, F>` (was `map<F, A, B>`).
  - **Lift**: `lift2<C, A, B, F>` (was `lift2<F, A, B, C>`).
  - **Semiapplicative**: `apply<FnBrand, B, A>` (was `apply<FnBrand, A, B>`).
  - **Semimonad**: `bind<B, A, F>` (was `bind<F, A, B>`).
  - **Foldable**:
    - `fold_right<FnBrand, B, A, F>` (was `fold_right<FnBrand, F, A, B>`).
    - `fold_left<FnBrand, B, A, F>` (was `fold_left<FnBrand, F, A, B>`).
    - `fold_map<FnBrand, M, A, Func>` (was `fold_map<FnBrand, Func, A, M>`).
  - **Traversable**: `traverse<F, B, A, Func>` (was `traverse<F, Func, A, B>`).
  - **ParFoldable**:
    - `par_fold_map<M, A>` (was `par_fold_map<A, M>`).
    - `par_fold_right<B, A>` (was `par_fold_right<A, B>`).
  - **Compactable**: `separate<O, E>` (was `separate<E, O>`).
  - **Filterable**:
    - `partition_map<O, E, A, Func>` (was `partition_map<Func, A, E, O>`).
    - `filter_map<B, A, Func>` (was `filter_map<Func, A, B>`).
  - **Witherable**:
    - `wilt<M, O, E, A, Func>` (was `wilt<Func, M, A, E, O>`).
    - `wither<M, B, A, Func>` (was `wither<Func, M, A, B>`).
- **Renames (API Breaking)**:
  - Renamed `SendCloneableFn::new_send` to `SendCloneableFn::send_cloneable_fn_new` to facilitate unique re-exports.
- **Parameter Ordering (API Breaking)**:
  - Reordered arguments for `ParFoldable::par_fold_map` and `ParFoldable::par_fold_right` to place the function argument first (e.g., `par_fold_map(func, fa)`), aligning with `Foldable` conventions.
- **Documentation**:
  - Updated all code examples in README and crate documentation to use free functions (e.g., `map(f, x)`) instead of trait methods, reflecting the intended usage pattern.
  - Updated type signatures in documentation to accurately reflect uncurried semantics and type parameter ordering.
  - Added "Documentation" section to README linking to architecture and limitations docs.

## [0.4.1]

### Documentation
- **Brand Types**: Updated documentation for all Brand types in `src/brands.rs` to fix broken links and improve clarity.

## [0.4.0] - 2026-01-18

### Added
- **Data Shrinking Typeclasses**:
  - Added `Compactable`, `Filterable` and `Witherable` typeclasses for discarding values in contexts.
  - Implemented `Compactable`, `Filterable`, and `Witherable` for `OptionBrand` and `VecBrand`.
  - Added property-based tests and edge case tests for `Compactable`, `Filterable`, and `Witherable` implementations for `Option` and `Vec`.

### Changed
- **Data Shrinking API (API Breaking)**:
  - Updated `Compactable::separate`, `Filterable::partition_map`, and `Witherable::wilt` to return `Pair<Success, Failure>` (e.g., `Pair<Ok, Err>`), aligning with Rust's `Result` and `Iterator::partition` conventions.
  - Added default implementations for `Filterable` and `Witherable` methods.
  - Added comprehensive documentation for `Compactable`, `Filterable`, and `Witherable`.
- **`Apply!` Macro Migration**:
  - Migrated all usages of `Apply!` to the new syntax: `Apply!(<Brand as Kind!(KindSignature)>::AssocType<Args>)`.
  - Converted usages of the deprecated "Explicit Kind Mode" to standard Rust syntax (e.g., `<Brand as Kind>::Of<Args>`).
- **Kind Trait Refactor (API Breaking)**:
  - Updated `Kind` traits to support multiple associated types (e.g., `Of`, `SendOf`).
  - Updated `def_kind!` and `impl_kind!` macros to use standard Rust syntax for associated type definitions.
  - Updated internal Kind trait hashes to reflect the new canonicalization logic.

---

## [0.3.0] - 2026-01-16

### Added

- **Thread Safety and Parallelism**:
  - Added `SendCloneableFn` extension trait for thread-safe function wrappers with `Send + Sync` bounds.
  - Added `ParFoldable` trait providing `par_fold_map` and `par_fold_right` for parallel folding operations.
  - Added `SendEndofunction` type for thread-safe endofunctions using `ArcFnBrand`.
  - Implemented `SendCloneableFn` for `ArcFnBrand` with `new_send` constructor.
  - Implemented `ParFoldable` for `VecBrand` (with optional Rayon parallelism) and `OptionBrand`.
- **Feature Flags**:
  - Added optional `rayon` feature (`rayon = ["dep:rayon"]`) enabling parallel execution in `VecBrand::par_fold_map`.
- **Testing Infrastructure**:
  - Added compile-fail tests using `trybuild` to verify thread safety error messages.
  - Added UI tests for `SendCloneableFn`: `new_send_not_send.rs`, `new_send_not_sync.rs`, `rc_fn_not_send.rs`.
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
  - Renamed `CloneableFnBrand` type parameter to `FnBrand` across the library.
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
- **Generic Bounds**: Trait methods now use generic `F: Fn(A) -> B` bounds instead of `CloneableFn` where possible, enabling inlining and monomorphization.
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
