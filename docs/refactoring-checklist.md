# Zero-Cost Refactoring Checklist

This checklist tracks the progress of the Zero-Cost Abstractions Refactoring Plan.

## Phase 0: Setup Parallel Module Structure

- [x] **Step 0.1**: Create `v2` module structure.
  - [x] Create `fp-library/src/v2/mod.rs`.
  - [x] Create `fp-library/src/v2/classes/mod.rs`.
  - [x] Create `fp-library/src/v2/types/mod.rs`.
- [x] **Step 0.2**: Configure feature flags in `lib.rs`.
  - [x] Add `v2` module declaration guarded by feature flag (or just pub mod for now).

## Phase 1: Simplify and Restrict Function Wrapper Traits

_Note: These changes might need to be mirrored or adapted in `v2` if they are foundational, or the `v2` code can import them if they are compatible._

- [x] **Step 1.1**: Review `Function` trait in `fp-library/src/classes/function.rs`.
  - [x] Confirm it is kept as a base abstraction.
- [x] **Step 1.2**: Review `ClonableFn` trait in `fp-library/src/classes/clonable_fn.rs`.
  - [x] Remove it from signatures of `Functor`, `Semimonad`, `Foldable`, `Traversable` (will be done in Phase 2).
  - [x] Ensure it is retained for `Semiapplicative::apply`, `Lazy`, `Defer`, `Endofunction`, `Endomorphism`.

## Phase 2: Uncurry Type Class Traits

- [x] **Step 2.1**: Refactor `Functor` in `fp-library/src/classes/functor.rs`.
  - [x] Update `map` signature to be uncurried: `fn map<A, B, F>(f: F, fa: Apply0L1T<Self, A>) -> Apply0L1T<Self, B> where F: Fn(A) -> B`.
  - [x] Update free function `map`.
- [x] **Step 2.2**: Create `Lift` trait in `fp-library/src/classes/lift.rs`.
  - [x] Define `lift2` with signature: `fn lift2<A, B, C, F>(f: F, fa: ..., fb: ...) -> ... where F: Fn(A, B) -> C`.
- [x] **Step 2.3**: Refactor `Semiapplicative` in `fp-library/src/classes/semiapplicative.rs`.
  - [x] Make it extend `Lift + Functor`.
  - [x] Update `apply` signature (keep `ClonableFnBrand` for type erasure).
- [x] **Step 2.4**: Refactor `Semimonad` in `fp-library/src/classes/semimonad.rs`.
  - [x] Update `bind` signature to be uncurried: `fn bind<A, B, F>(ma: ..., f: F) -> ... where F: Fn(A) -> ...`.
- [x] **Step 2.5**: Refactor `Foldable` in `fp-library/src/classes/foldable.rs`.
  - [x] Update `fold_right`, `fold_left`, `fold_map` to be uncurried.
- [x] **Step 2.6**: Refactor `Traversable` in `fp-library/src/classes/traversable.rs`.
  - [x] Update `traverse` and `sequence` to be uncurried.
  - [x] Ensure `Clone` bounds are present where necessary.
- [x] **Step 2.7**: Refactor `ApplyFirst` and `ApplySecond`.
  - [x] Update `fp-library/src/classes/apply_first.rs` to extend `Lift` and use `lift2`.
  - [x] Update `fp-library/src/classes/apply_second.rs` to extend `Lift` and use `lift2`.
- [x] **Step 2.8**: Refactor `Semigroup` in `fp-library/src/classes/semigroup.rs`.
  - [x] Simplify `append` signature: `fn append(a: Self, b: Self) -> Self`.
- [x] **Step 2.9**: Refactor `Semigroupoid` and `Category`.
  - [x] Update `fp-library/src/classes/semigroupoid.rs`: `compose` uncurried.
  - [x] Update `fp-library/src/classes/category.rs`: `identity` (no changes needed usually, but check).
- [x] **Step 2.10**: Refactor `Pointed` in `fp-library/src/classes/pointed.rs`.
  - [x] Remove `ClonableFnBrand` from `pure`.
- [x] **Step 2.11**: Verify `Applicative` and `Monad`.
  - [x] Check `fp-library/src/classes/applicative.rs` and `fp-library/src/classes/monad.rs` for compatibility.

## Phase 3: Update Type Implementations

- [x] **Step 3.1**: Update `OptionBrand` in `fp-library/src/types/option.rs`.
  - [x] Implement uncurried `Functor`, `Semiapplicative`, `Semimonad`, etc.
- [x] **Step 3.2**: Update `VecBrand` in `fp-library/src/types/vec.rs`.
  - [x] Implement uncurried traits using iterator methods.
  - [x] Optimize `fold` methods.
- [x] **Step 3.3**: Update other brands.
  - [x] `IdentityBrand` (`fp-library/src/types/identity.rs`).
  - [x] `ResultWithErrBrand` (`fp-library/src/types/result/result_with_err.rs`).
  - [x] `ResultWithOkBrand` (`fp-library/src/types/result/result_with_ok.rs`).
  - [x] `Pair` brands (`fp-library/src/types/pair.rs` etc.).
- [x] **Step 3.4**: Update `LazyBrand` in `fp-library/src/types/lazy.rs`.
  - [x] **Bug Fix**: Remove erroneous `where A: 'static` clause (Appendix A.1).
  - [x] Implement `Functor`, `Semiapplicative`, `Semimonad` preserving laziness.
    - *Note: `Functor`, `Semiapplicative`, `Semimonad` were omitted for `Lazy` due to `Clone` requirement conflicts with trait definitions.*
    - *Note: HKT infrastructure was upgraded to `Kind1L1T` to support lifetimes in `Lazy`.*

## Phase 4: Update Helper Functions

- [ ] **Step 4.1**: Update `compose` in `fp-library/src/functions.rs`.
  - [ ] Make it uncurried: `fn compose<...>(f: F, g: G) -> impl Fn...`.
- [ ] **Step 4.2**: Update `constant`.
  - [ ] Make it uncurried.
- [ ] **Step 4.3**: Update `flip`.
  - [ ] Make it uncurried.

## Phase 5: Update Endofunction/Endomorphism Types

- [ ] **Step 5.1**: Reimplement `Endofunction` in `fp-library/src/types/endofunction.rs`.
  - [ ] Update `Semigroup` implementation to be uncurried.
- [ ] **Step 5.2**: Reimplement `Endomorphism` in `fp-library/src/types/endomorphism.rs`.
  - [ ] Update `Semigroup` implementation.

## Phase 6: Update Brand Infrastructure

- [ ] **Step 6.1**: Verify `RcFnBrand` and `ArcFnBrand`.
  - [ ] Ensure they still work for `apply` with heterogeneous functions.

## Phase 7: Update Documentation and Examples

- [ ] **Step 7.1**: Update Doc Comments.
  - [ ] Update signatures and examples in all modified files.
- [ ] **Step 7.2**: Update README.
  - [ ] Update usage patterns in `README.md`.
- [ ] **Step 7.3**: Update Doc Tests.
  - [ ] Rewrite doc tests to use uncurried API.
