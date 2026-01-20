# Smart Pointer Abstraction Implementation Checklist

This checklist tracks progress on implementing the `SmartPointer` type class, refactoring `ClonableFn` to use it, and rewriting `Lazy` with shared memoization semantics. See [plan.md](./plan.md) for full context and design details.

**Note**: This is a breaking change. Backward compatibility is not maintained.

---

## Phase 1: SmartPointer Foundation

### 1.1 Trait Definition

- [ ] Create `fp-library/src/classes/smart_pointer.rs`
- [ ] Define `SmartPointer` trait:
  ```rust
  pub trait SmartPointer {
      type Of<T: ?Sized>: Clone + Deref<Target = T>;
      fn new<T>(value: T) -> Self::Of<T> where Self::Of<T>: Sized;
  }
  ```
- [ ] Define `SendSmartPointer` extension trait:
  ```rust
  pub trait SendSmartPointer: SmartPointer
  where
      for<T: Send + Sync> Self::Of<T>: Send + Sync,
  {}
  ```
- [ ] Add comprehensive documentation following `docs/architecture.md` standards
- [ ] Add module-level examples

### 1.2 Brand Definitions

- [ ] Add `RcBrand` struct to `fp-library/src/brands.rs`
- [ ] Add `ArcBrand` struct to `fp-library/src/brands.rs`
- [ ] Add documentation for both brands explaining their use cases

### 1.3 Rc Implementation

- [ ] Create `fp-library/src/types/rc_ptr.rs`
- [ ] Implement `SmartPointer` for `RcBrand`:
  - [ ] `type Of<T: ?Sized> = Rc<T>`
  - [ ] `fn new<T>(value: T) -> Rc<T>` using `Rc::new`
- [ ] Add `impl_kind!` for `RcBrand` if needed
- [ ] Add documentation and examples
- [ ] Verify `RcBrand` does NOT implement `SendSmartPointer`

### 1.4 Arc Implementation

- [ ] Create `fp-library/src/types/arc_ptr.rs`
- [ ] Implement `SmartPointer` for `ArcBrand`:
  - [ ] `type Of<T: ?Sized> = Arc<T>`
  - [ ] `fn new<T>(value: T) -> Arc<T>` using `Arc::new`
- [ ] Implement `SendSmartPointer` for `ArcBrand`
- [ ] Add `impl_kind!` for `ArcBrand` if needed
- [ ] Add documentation and examples

### 1.5 Free Functions & Re-exports

- [ ] Add free function `smart_pointer_new` to `fp-library/src/classes/smart_pointer.rs`
- [ ] Update `fp-library/src/classes.rs` to re-export `smart_pointer` module
- [ ] Update `fp-library/src/functions.rs` to re-export `smart_pointer_new` (prefixed)
- [ ] Update `fp-library/src/types.rs` to re-export `rc_ptr` and `arc_ptr` modules

### 1.6 Phase 1 Tests

- [ ] Unit test: `RcBrand::new` creates `Rc<T>`
- [ ] Unit test: `ArcBrand::new` creates `Arc<T>`
- [ ] Unit test: `Clone` works for `RcBrand::Of<T>`
- [ ] Unit test: `Clone` works for `ArcBrand::Of<T>`
- [ ] Unit test: `Deref` works correctly for both
- [ ] Compile-fail test: `RcBrand::Of<T>` is `!Send`
- [ ] Compile-success test: `ArcBrand::Of<T: Send + Sync>` is `Send + Sync`

---

## Phase 2: FnBrand Refactor

### 2.1 Brand Structure

- [ ] Add `FnBrand<PtrBrand: SmartPointer>` struct to `fp-library/src/brands.rs`
- [ ] Add type alias `pub type RcFnBrand = FnBrand<RcBrand>;`
- [ ] Add type alias `pub type ArcFnBrand = FnBrand<ArcBrand>;`
- [ ] Add documentation explaining the parameterization

### 2.2 Implementation via Macro

- [ ] Create `fp-library/src/types/fn_brand.rs`
- [ ] Define `impl_fn_brand!` macro for reducing duplication
- [ ] Implement for `FnBrand<RcBrand>`:
  - [ ] `Function` trait
  - [ ] `ClonableFn` trait
  - [ ] `Semigroupoid` trait
  - [ ] `Category` trait
- [ ] Implement for `FnBrand<ArcBrand>`:
  - [ ] `Function` trait
  - [ ] `ClonableFn` trait
  - [ ] `Semigroupoid` trait  
  - [ ] `Category` trait
- [ ] Implement `SendClonableFn` for `FnBrand<ArcBrand>` only
- [ ] Verify `FnBrand<RcBrand>` does NOT implement `SendClonableFn`

### 2.3 Remove Old Files

- [ ] Delete `fp-library/src/types/rc_fn.rs`
- [ ] Delete `fp-library/src/types/arc_fn.rs`
- [ ] Update `fp-library/src/types.rs` to remove old re-exports
- [ ] Update `fp-library/src/types.rs` to add `fn_brand` re-export

### 2.4 Update Dependent Code

- [ ] Search for all uses of `RcFnBrand` - should work via type alias
- [ ] Search for all uses of `ArcFnBrand` - should work via type alias
- [ ] Update any direct imports from old modules
- [ ] Fix any compilation errors

### 2.5 Phase 2 Tests

- [ ] All existing `RcFnBrand` tests still pass
- [ ] All existing `ArcFnBrand` tests still pass
- [ ] `SendClonableFn` tests pass for `FnBrand<ArcBrand>`
- [ ] Compile-fail: `FnBrand<RcBrand>` cannot be used with `SendClonableFn`
- [ ] Semigroupoid associativity law
- [ ] Category identity laws

---

## Phase 3: Lazy Refactor

### 3.1 Rewrite Lazy Type

- [ ] Rewrite `fp-library/src/types/lazy.rs`
- [ ] New structure with shared semantics:
  ```rust
  pub struct Lazy<'a, PtrBrand, OnceBrand, A>(
      <PtrBrand as SmartPointer>::Of<(
          <OnceBrand as Once>::Of<A>,
          <FnBrand<PtrBrand> as ClonableFn>::Of<'a, (), A>,
      )>,
  )
  ```
- [ ] Note: Only 3 type parameters (not 4) - FnBrand is derived from PtrBrand

### 3.2 Core Implementation

- [ ] Implement `Lazy::new(thunk)` method
  - [ ] Creates new OnceCell
  - [ ] Wraps `(OnceCell, thunk)` in SmartPointer
- [ ] Implement `Lazy::force(&self)` method
  - [ ] Takes `&self` (not `Self` - shared semantics!)
  - [ ] Dereferences smart pointer
  - [ ] Uses `OnceCell::get_or_init`
  - [ ] Returns cloned value
- [ ] Add documentation with type signatures

### 3.3 Clone Implementation

- [ ] Implement `Clone` for `Lazy`
- [ ] Clone must be cheap (just smart pointer clone)
- [ ] Verify clones share memoization state (via test)

### 3.4 Type Class Implementations

- [ ] Implement `Semigroup` for `Lazy` where `A: Semigroup + Clone`
  - [ ] `append` creates new `Lazy` that forces both and appends
- [ ] Implement `Monoid` for `Lazy` where `A: Monoid + Clone`
  - [ ] `empty` creates `Lazy` returning `Monoid::empty()`
- [ ] Implement `Defer` for `Lazy`
  - [ ] `defer(f)` creates `Lazy` that calls f, then forces result

### 3.5 Kind Implementation

- [ ] Update `LazyBrand<PtrBrand, OnceBrand>` in `fp-library/src/brands.rs`
- [ ] Update `impl_kind!` for `LazyBrand`
- [ ] Verify kind signature matches expected HKT pattern

### 3.6 Free Functions

- [ ] Update `lazy_new` free function (or rename)
- [ ] Update `lazy_force` free function
- [ ] Re-export in `fp-library/src/functions.rs`

### 3.7 Type Aliases

- [ ] Add `pub type RcLazy<'a, A> = Lazy<'a, RcBrand, OnceCellBrand, A>;`
- [ ] Add `pub type ArcLazy<'a, A> = Lazy<'a, ArcBrand, OnceLockBrand, A>;`
- [ ] Document thread-safety characteristics of each alias

### 3.8 Phase 3 Tests

- [ ] Unit test: `Lazy::new` + `Lazy::force` returns correct value
- [ ] **Critical test**: Thunk is only called once across all clones
- [ ] **Critical test**: Clones share memoization (counter test)
- [ ] Unit test: `Semigroup::append` works correctly
- [ ] Unit test: `Monoid::empty` works correctly
- [ ] Unit test: `Defer::defer` works correctly
- [ ] Property test: Semigroup associativity law
- [ ] Property test: Monoid identity laws
- [ ] Compile-fail test: `RcLazy` is `!Send`
- [ ] Compile-success test: `ArcLazy<A: Send + Sync>` is `Send + Sync`

---

## Phase 4: Integration & Polish

### 4.1 Documentation

- [ ] Update module-level docs in `lazy.rs` explaining shared semantics
- [ ] Add "Migration from old Lazy" section (for reference, not compat)
- [ ] Ensure all public items have examples
- [ ] Doc tests pass for all examples

### 4.2 Module Structure Updates

- [ ] Update `fp-library/src/types.rs`:
  - [ ] Add `rc_ptr` export
  - [ ] Add `arc_ptr` export  
  - [ ] Add `fn_brand` export
  - [ ] Remove `rc_fn` export
  - [ ] Remove `arc_fn` export
- [ ] Update `fp-library/src/brands.rs` with all new brands
- [ ] Update `fp-library/src/classes.rs` with `smart_pointer` export
- [ ] Verify no circular dependencies

### 4.3 Documentation Files

- [ ] Update `docs/std-coverage-checklist.md` with:
  - [ ] `SmartPointer` entry in Type Classes table
  - [ ] `SendSmartPointer` entry in Type Classes table
  - [ ] `RcBrand` entry in Data Types table
  - [ ] `ArcBrand` entry in Data Types table
  - [ ] Update `LazyBrand` entry to reflect new semantics
- [ ] Update `docs/architecture.md`:
  - [ ] Document the SmartPointer + SendSmartPointer pattern
  - [ ] Document the macro-based impl approach for FnBrand
- [ ] Update `docs/todo.md` to mark Lazy memoization item as addressed
- [ ] Update `docs/limitations.md` if any new limitations discovered

### 4.4 Final Verification

- [ ] `cargo test` passes (all tests)
- [ ] `cargo clippy` passes (no warnings)
- [ ] `cargo doc` builds without warnings
- [ ] Review generated documentation for clarity
- [ ] Run benchmarks to verify no performance regression

---

## Release Preparation

### Changelog

- [ ] Add entry to `fp-library/CHANGELOG.md` under `[Unreleased]`:
  ```markdown
  ### Changed (Breaking)
  - `Lazy` now uses shared memoization semantics (Haskell-like)
    - Clones share memoization state
    - `force` takes `&self` instead of `self`
    - Type parameters reduced from 4 to 3
  - `RcFnBrand` is now a type alias for `FnBrand<RcBrand>`
  - `ArcFnBrand` is now a type alias for `FnBrand<ArcBrand>`
  
  ### Added
  - `SmartPointer` type class abstracting over `Rc` and `Arc`
  - `SendSmartPointer` extension trait for thread-safe smart pointers
  - `RcBrand` and `ArcBrand` implementing `SmartPointer`
  - `FnBrand<PtrBrand>` generic function brand
  - `RcLazy` and `ArcLazy` type aliases for common configurations
  
  ### Removed
  - Old value-semantic `Lazy` implementation
  - Separate `rc_fn.rs` and `arc_fn.rs` files (merged into `fn_brand.rs`)
  ```

---

## Implementation Notes

### Key Design Decisions

1. **Extension trait pattern**: `SendSmartPointer` extends `SmartPointer` mirroring how `SendClonableFn` extends `ClonableFn`

2. **Macro for unsized coercion**: `impl_fn_brand!` macro handles the `Rc::new`/`Arc::new` calls that perform unsized coercion to `dyn Fn`

3. **Reduced type parameters**: `Lazy` now has 3 parameters instead of 4 because `FnBrand` is derived from `PtrBrand`

4. **Type aliases for ergonomics**: `RcFnBrand`, `ArcFnBrand`, `RcLazy`, `ArcLazy` provide good defaults

### Files Summary

| Action | File |
|--------|------|
| Create | `fp-library/src/classes/smart_pointer.rs` |
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
   let lazy = Lazy::new(|| { counter.set(counter.get() + 1); 42 });
   let lazy2 = lazy.clone();
   
   assert_eq!(Lazy::force(&lazy), 42);
   assert_eq!(counter.get(), 1);  // Called once
   assert_eq!(Lazy::force(&lazy2), 42);
   assert_eq!(counter.get(), 1);  // Still 1 - shared!
   ```

2. **Thread safety test** (for ArcLazy):
   ```rust
   let lazy: ArcLazy<i32> = Lazy::new(|| 42);
   std::thread::spawn(move || Lazy::force(&lazy)).join().unwrap();
   ```

3. **Compile-fail for RcLazy Send**:
   ```rust
   let lazy: RcLazy<i32> = Lazy::new(|| 42);
   std::thread::spawn(move || Lazy::force(&lazy)); // Should fail!
   ```
