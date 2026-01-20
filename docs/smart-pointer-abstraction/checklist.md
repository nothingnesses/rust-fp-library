# Pointer Abstraction Implementation Checklist

This checklist tracks progress on implementing the `Pointer` → `RefCountedPointer` → `SendRefCountedPointer` type class hierarchy, refactoring `ClonableFn` to use it, and rewriting `Lazy` with shared memoization semantics. See [plan.md](./plan.md) for full context and design details.

**Note**: This is a breaking change. Backward compatibility is not maintained.

***

## Phase 1: Pointer Trait Foundation

### 1.1 Trait Definition

* \[ ] Create `fp-library/src/classes/pointer.rs`
* \[ ] Define `Pointer` base trait:
  ```rust
  pub trait Pointer {
      type Of<T: ?Sized>: Deref<Target = T>;
      fn new<T>(value: T) -> Self::Of<T> where Self::Of<T>: Sized;
  }
  ```
* \[ ] Define `RefCountedPointer` extension trait:
  ```rust
  pub trait RefCountedPointer: Pointer {
      type CloneableOf<T: ?Sized>: Clone + Deref<Target = T>;
      fn cloneable_new<T>(value: T) -> Self::CloneableOf<T> where Self::CloneableOf<T>: Sized;
  }
  ```
* \[ ] Define `SendRefCountedPointer` marker trait:
  ```rust
  pub trait SendRefCountedPointer: RefCountedPointer
  where
      for<T: Send + Sync> Self::CloneableOf<T>: Send + Sync,
  {}
  ```
* \[ ] Add free functions `pointer_new` and `ref_counted_new`
* \[ ] Add comprehensive documentation following `docs/architecture.md` standards
* \[ ] Add module-level examples

### 1.2 Brand Definitions

* \[ ] Add `RcBrand` struct to `fp-library/src/brands.rs`
* \[ ] Add `ArcBrand` struct to `fp-library/src/brands.rs`
* \[ ] Add `BoxBrand` struct placeholder (future extension)
* \[ ] Add documentation for all brands explaining their use cases

### 1.3 Rc Implementation

* \[ ] Create `fp-library/src/types/rc_ptr.rs`
* \[ ] Implement `Pointer` for `RcBrand`:
  * \[ ] `type Of<T: ?Sized> = Rc<T>`
  * \[ ] `fn new<T>(value: T) -> Rc<T>` using `Rc::new`
* \[ ] Implement `RefCountedPointer` for `RcBrand`:
  * \[ ] `type CloneableOf<T: ?Sized> = Rc<T>` (same as `Of<T>`)
  * \[ ] `fn cloneable_new<T>(value: T) -> Rc<T>`
* \[ ] Add documentation and examples
* \[ ] Verify `RcBrand` does NOT implement `SendRefCountedPointer`

### 1.4 Arc Implementation

* \[ ] Create `fp-library/src/types/arc_ptr.rs`
* \[ ] Implement `Pointer` for `ArcBrand`:
  * \[ ] `type Of<T: ?Sized> = Arc<T>`
  * \[ ] `fn new<T>(value: T) -> Arc<T>` using `Arc::new`
* \[ ] Implement `RefCountedPointer` for `ArcBrand`:
  * \[ ] `type CloneableOf<T: ?Sized> = Arc<T>` (same as `Of<T>`)
  * \[ ] `fn cloneable_new<T>(value: T) -> Arc<T>`
* \[ ] Implement `SendRefCountedPointer` for `ArcBrand`
* \[ ] Add documentation and examples

### 1.5 Free Functions & Re-exports

* \[ ] Add free function `pointer_new<P: Pointer, T>` to `fp-library/src/classes/pointer.rs`
* \[ ] Add free function `ref_counted_new<P: RefCountedPointer, T>` to same file
* \[ ] Update `fp-library/src/classes.rs` to re-export `pointer` module
* \[ ] Update `fp-library/src/functions.rs` to re-export free functions
* \[ ] Update `fp-library/src/types.rs` to re-export `rc_ptr` and `arc_ptr` modules

### 1.6 Phase 1 Tests

* \[ ] Unit test: `RcBrand::new` creates `Rc<T>`
* \[ ] Unit test: `ArcBrand::new` creates `Arc<T>`
* \[ ] Unit test: `RcBrand::cloneable_new` creates `Rc<T>`
* \[ ] Unit test: `ArcBrand::cloneable_new` creates `Arc<T>`
* \[ ] Unit test: `Clone` works for `RcBrand::CloneableOf<T>`
* \[ ] Unit test: `Clone` works for `ArcBrand::CloneableOf<T>`
* \[ ] Unit test: `Deref` works correctly for both
* \[ ] Compile-fail test: `Rc<T>` is `!Send`
* \[ ] Compile-success test: `Arc<T: Send + Sync>` is `Send + Sync`

***

## Phase 2: FnBrand Refactor

### 2.1 Brand Structure

* \[ ] Add `FnBrand<PtrBrand: RefCountedPointer>` struct to `fp-library/src/brands.rs`
* \[ ] Add type alias `pub type RcFnBrand = FnBrand<RcBrand>;`
* \[ ] Add type alias `pub type ArcFnBrand = FnBrand<ArcBrand>;`
* \[ ] Add documentation explaining the parameterization

### 2.2 Implementation via Macro

* \[ ] Create `fp-library/src/types/fn_brand.rs`
* \[ ] Define `impl_fn_brand!` macro for reducing duplication
* \[ ] Implement for `FnBrand<RcBrand>`:
  * \[ ] `Function` trait
  * \[ ] `ClonableFn` trait
  * \[ ] `Semigroupoid` trait
  * \[ ] `Category` trait
* \[ ] Implement for `FnBrand<ArcBrand>`:
  * \[ ] `Function` trait
  * \[ ] `ClonableFn` trait
  * \[ ] `Semigroupoid` trait
  * \[ ] `Category` trait
* \[ ] Implement `SendClonableFn` for `FnBrand<ArcBrand>` only
* \[ ] Verify `FnBrand<RcBrand>` does NOT implement `SendClonableFn`

### 2.3 Remove Old Files

* \[ ] Delete `fp-library/src/types/rc_fn.rs`
* \[ ] Delete `fp-library/src/types/arc_fn.rs`
* \[ ] Update `fp-library/src/types.rs` to remove old re-exports
* \[ ] Update `fp-library/src/types.rs` to add `fn_brand` re-export

### 2.4 Update Dependent Code

* \[ ] Search for all uses of `RcFnBrand` - should work via type alias
* \[ ] Search for all uses of `ArcFnBrand` - should work via type alias
* \[ ] Update any direct imports from old modules
* \[ ] Fix any compilation errors

### 2.5 Phase 2 Tests

* \[ ] All existing `RcFnBrand` tests still pass
* \[ ] All existing `ArcFnBrand` tests still pass
* \[ ] `SendClonableFn` tests pass for `FnBrand<ArcBrand>`
* \[ ] Compile-fail: `FnBrand<RcBrand>` cannot be used with `SendClonableFn`
* \[ ] Semigroupoid associativity law
* \[ ] Category identity laws

***

## Phase 3: Lazy Refactor

### 3.1 Rewrite Lazy Type

* \[ ] Rewrite `fp-library/src/types/lazy.rs`
* \[ ] New structure with shared semantics:
  ```rust
  pub struct Lazy<'a, PtrBrand, OnceBrand, A>(
      <PtrBrand as RefCountedPointer>::CloneableOf<(
          <OnceBrand as Once>::Of<A>,
          <FnBrand<PtrBrand> as ClonableFn>::Of<'a, (), A>,
      )>,
  )
  where
      PtrBrand: RefCountedPointer,
      OnceBrand: Once;
  ```
* \[ ] Note: Only 3 type parameters (not 4) - FnBrand is derived from PtrBrand

### 3.2 Core Implementation

* \[ ] Implement `Lazy::new(thunk)` method
  * \[ ] Creates new OnceCell via `OnceBrand::new()`
  * \[ ] Wraps `(OnceCell, thunk)` in `PtrBrand::cloneable_new`
* \[ ] Implement `Lazy::force(&self)` method
  * \[ ] Takes `&self` (not `Self` - shared semantics!)
  * \[ ] Dereferences through `CloneableOf` pointer
  * \[ ] Uses `OnceCell::get_or_init`
  * \[ ] Returns cloned value
* \[ ] Implement `Lazy::try_get(&self)` method
  * \[ ] Returns `Option<A>` without forcing
* \[ ] Add documentation with type signatures

### 3.3 Clone Implementation

* \[ ] Implement `Clone` for `Lazy`
* \[ ] Clone must be cheap (just reference count increment)
* \[ ] Verify clones share memoization state (via test)

### 3.4 Type Class Implementations

* \[ ] Implement `Semigroup` for `Lazy` where `A: Semigroup + Clone`
  * \[ ] `combine` creates new `Lazy` that forces both and combines
* \[ ] Implement `Monoid` for `Lazy` where `A: Monoid + Clone`
  * \[ ] `empty` creates `Lazy` returning `Monoid::empty()`
* \[ ] Implement `Defer` for `LazyBrand`
  * \[ ] `defer(f)` creates `Lazy` that calls f, then forces result

### 3.5 Kind Implementation

* \[ ] Update `LazyBrand<PtrBrand, OnceBrand>` in `fp-library/src/brands.rs`
* \[ ] Update `impl_kind!` for `LazyBrand`
* \[ ] Verify kind signature matches expected HKT pattern

### 3.6 Free Functions

* \[ ] Update `lazy_new` free function (or rename)
* \[ ] Update `lazy_force` free function
* \[ ] Re-export in `fp-library/src/functions.rs`

### 3.7 Type Aliases

* \[ ] Add `pub type RcLazy<'a, A> = Lazy<'a, RcBrand, OnceCellBrand, A>;`
* \[ ] Add `pub type ArcLazy<'a, A> = Lazy<'a, ArcBrand, OnceLockBrand, A>;`
* \[ ] Document thread-safety characteristics of each alias

### 3.8 Phase 3 Tests

* \[ ] Unit test: `Lazy::new` + `Lazy::force` returns correct value
* \[ ] **Critical test**: Thunk is only called once across all clones
* \[ ] **Critical test**: Clones share memoization (counter test)
* \[ ] Unit test: `Semigroup::combine` works correctly
* \[ ] Unit test: `Monoid::empty` works correctly
* \[ ] Unit test: `Defer::defer` works correctly
* \[ ] Property test: Semigroup associativity law
* \[ ] Property test: Monoid identity laws
* \[ ] Compile-fail test: `RcLazy` is `!Send`
* \[ ] Compile-success test: `ArcLazy<A: Send + Sync>` is `Send + Sync`

***

## Phase 4: Integration & Polish

### 4.1 Documentation

* \[ ] Update module-level docs in `lazy.rs` explaining shared semantics
* \[ ] Add "Migration from old Lazy" section (for reference, not compat)
* \[ ] Ensure all public items have examples
* \[ ] Doc tests pass for all examples

### 4.2 Module Structure Updates

* \[ ] Update `fp-library/src/types.rs`:
  * \[ ] Add `rc_ptr` export
  * \[ ] Add `arc_ptr` export
  * \[ ] Add `fn_brand` export
  * \[ ] Remove `rc_fn` export
  * \[ ] Remove `arc_fn` export
* \[ ] Update `fp-library/src/brands.rs` with all new brands
* \[ ] Update `fp-library/src/classes.rs` with `pointer` export
* \[ ] Verify no circular dependencies

### 4.3 Documentation Files

* \[ ] Update `docs/std-coverage-checklist.md` with:
  * \[ ] `Pointer` entry in Type Classes table
  * \[ ] `RefCountedPointer` entry in Type Classes table
  * \[ ] `SendRefCountedPointer` entry in Type Classes table
  * \[ ] `RcBrand` entry in Data Types table
  * \[ ] `ArcBrand` entry in Data Types table
  * \[ ] Update `LazyBrand` entry to reflect new semantics
* \[ ] Update `docs/architecture.md`:
  * \[ ] Document the Pointer → RefCountedPointer → SendRefCountedPointer pattern
  * \[ ] Document the macro-based impl approach for FnBrand
* \[ ] Update `docs/todo.md` to mark Lazy memoization item as addressed
* \[ ] Update `docs/limitations.md` if any new limitations discovered

### 4.4 Final Verification

* \[ ] `cargo test` passes (all tests)
* \[ ] `cargo clippy` passes (no warnings)
* \[ ] `cargo doc` builds without warnings
* \[ ] Review generated documentation for clarity
* \[ ] Run benchmarks to verify no performance regression

***

## Release Preparation

### Changelog

* \[ ] Add entry to `fp-library/CHANGELOG.md` under `[Unreleased]`:
  ```markdown
  ### Changed (Breaking)
  - `Lazy` now uses shared memoization semantics (Haskell-like)
    - Clones share memoization state
    - `force` takes `&self` instead of `self`
    - Type parameters reduced from 4 to 3
  - `RcFnBrand` is now a type alias for `FnBrand<RcBrand>`
  - `ArcFnBrand` is now a type alias for `FnBrand<ArcBrand>`

  ### Added
  - `Pointer` base trait for heap-allocated pointers
  - `RefCountedPointer` extension trait with `CloneableOf` for shared ownership
  - `SendRefCountedPointer` marker trait for thread-safe pointers
  - `RcBrand` and `ArcBrand` implementing the pointer hierarchy
  - `BoxBrand` placeholder for future unique ownership support
  - `FnBrand<PtrBrand>` generic function brand
  - `RcLazy` and `ArcLazy` type aliases for common configurations

  ### Removed
  - Old value-semantic `Lazy` implementation
  - Separate `rc_fn.rs` and `arc_fn.rs` files (merged into `fn_brand.rs`)
  ```

***

## Implementation Notes

### Key Design Decisions

1. **Three-level trait hierarchy**: `Pointer` → `RefCountedPointer` → `SendRefCountedPointer` allows future `BoxBrand` support at the `Pointer` level without reference counting

2. **Additional Associated Type pattern**: `RefCountedPointer` adds `CloneableOf` with `Clone` bound rather than trying to add bounds to inherited `Of` (which Rust doesn't allow)

3. **Macro for unsized coercion**: `impl_fn_brand!` macro handles the `Rc::new`/`Arc::new` calls that perform unsized coercion to `dyn Fn`

4. **Reduced type parameters**: `Lazy` now has 3 parameters instead of 4 because `FnBrand` is derived from `PtrBrand`

5. **Type aliases for ergonomics**: `RcFnBrand`, `ArcFnBrand`, `RcLazy`, `ArcLazy` provide good defaults

### Files Summary

| Action | File |
|--------|------|
| Create | `fp-library/src/classes/pointer.rs` |
| Create | `fp-library/src/types/rc_ptr.rs` |
| Create | `fp-library/src/types/arc_ptr.rs` |
| Create | `fp-library/src/types/fn_brand.rs` |
| Delete | `fp-library/src/types/rc_fn.rs` |
| Delete | `fp-library/src/types/arc_fn.rs` |
| Rewrite | `fp-library/src/types/lazy.rs` |
| Modify | `fp-library/src/brands.rs` |
| Modify | `fp-library/src/classes.rs` |
| Modify | `fp-library/src/types.rs` |
| Modify | `fp-library/src/functions.rs` |

### Testing Strategy

1. **Unit tests**: In each module's `#[cfg(test)]` block
2. **Property tests**: In `fp-library/tests/property_tests.rs`
3. **Compile-fail tests**: In `fp-library/tests/ui/` directory
4. **Doc tests**: In documentation examples

### Critical Test Cases

The most important tests are:

1. **Shared memoization test** (must pass):
   ```rust
   let counter = Rc::new(Cell::new(0));
   let counter_clone = counter.clone();

   let lazy = RcLazy::new(
       clonable_fn_new::<RcFnBrand, _, _>(move |_| {
           counter_clone.set(counter_clone.get() + 1);
           42
       })
   );
   let lazy2 = lazy.clone();

   assert_eq!(Lazy::force(&lazy), 42);
   assert_eq!(counter.get(), 1);  // Called once
   assert_eq!(Lazy::force(&lazy2), 42);
   assert_eq!(counter.get(), 1);  // Still 1 - shared!
   ```

2. **Thread safety test** (for ArcLazy):
   ```rust
   let lazy: ArcLazy<i32> = ArcLazy::new(
       send_clonable_fn_new::<ArcFnBrand, _, _>(|_| 42)
   );
   std::thread::spawn(move || Lazy::force(&lazy)).join().unwrap();
   ```

3. **Compile-fail for RcLazy Send**:
   ```rust
   let lazy: RcLazy<i32> = RcLazy::new(/* ... */);
   std::thread::spawn(move || Lazy::force(&lazy)); // Should fail!
   ```
