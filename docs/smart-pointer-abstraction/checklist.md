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
* \[ ] Define `SendRefCountedPointer` extension trait with `SendOf`:
  ```rust
  pub trait SendRefCountedPointer: RefCountedPointer {
      type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;
      fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>
      where
          Self::SendOf<T>: Sized;
  }
  ```
* \[ ] Define `ThunkWrapper` trait for pointer-brand-specific thunk storage:
  ```rust
  pub trait ThunkWrapper {
      type Cell<T>;
      fn new_cell<T>(value: Option<T>) -> Self::Cell<T>;
      fn take<T>(cell: &Self::Cell<T>) -> Option<T>;
  }
  ```
* \[ ] Define `ValidLazyCombination` marker trait with `ThunkOf` associated type for valid PtrBrand/OnceBrand/FnBrand combinations:
  ```rust
  pub trait ValidLazyCombination<PtrBrand, OnceBrand, FnBrand> {
      /// Thunk type: ClonableFn::Of for Rc, SendClonableFn::SendOf for Arc
      type ThunkOf<'a, A>: Clone;
  }
  impl ValidLazyCombination<RcBrand, OnceCellBrand, RcFnBrand> for () {
      type ThunkOf<'a, A> = <RcFnBrand as ClonableFn>::Of<'a, (), A>;
  }
  impl ValidLazyCombination<ArcBrand, OnceLockBrand, ArcFnBrand> for () {
      type ThunkOf<'a, A> = <ArcFnBrand as SendClonableFn>::SendOf<'a, (), A>;
  }
  ```
* \[ ] Define `LazyError` struct with panic payload for debugging:
  ```rust
  use thiserror::Error;
  use std::any::Any;

  /// Error type for lazy evaluation failures.
  /// Stores the panic payload for debugging purposes.
  #[derive(Debug, Error)]
  #[error("thunk panicked during evaluation{}", .0.as_ref().and_then(|p| p.panic_message()).map(|m| format!(": {}", m)).unwrap_or_default())]
  pub struct LazyError(Option<PanicPayload>);

  /// Wrapper for panic payload with helper methods.
  #[derive(Debug)]
  pub struct PanicPayload(Box<dyn Any + Send + 'static>);
  ```
* \[ ] Implement `LazyError::from_panic(payload)` constructor
* \[ ] Implement `LazyError::poisoned()` for secondary access errors
* \[ ] Implement `LazyError::panic_message() -> Option<&str>`
* \[ ] Implement `PanicPayload::panic_message() -> Option<&str>` (downcast to \&str or String)
* \[ ] Implement `PanicPayload::into_inner()` for re-panicking
* \[ ] Add `thiserror = "1.0"` to dependencies in `fp-library/Cargo.toml`
* \[ ] Define `UnsizedCoercible` trait for basic function coercion:
  ```rust
  /// Trait for pointer brands that can coerce to `dyn Fn`.
  pub trait UnsizedCoercible: RefCountedPointer {
      fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::CloneableOf<dyn 'a + Fn(A) -> B>;
  }
  ```
* \[ ] Define `SendUnsizedCoercible` extension trait for thread-safe coercion:
  ```rust
  /// Extension trait for thread-safe function coercion.
  /// Follows ClonableFn → SendClonableFn pattern.
  pub trait SendUnsizedCoercible: UnsizedCoercible + SendRefCountedPointer {
      fn coerce_fn_send<'a, A, B>(
          f: impl 'a + Fn(A) -> B + Send + Sync
      ) -> Self::CloneableOf<dyn 'a + Fn(A) -> B + Send + Sync>;
  }
  ```
* \[ ] Implement `UnsizedCoercible` for `RcBrand` (basic coercion only)
* \[ ] Implement `UnsizedCoercible` for `ArcBrand`
* \[ ] Implement `SendUnsizedCoercible` for `ArcBrand` only (not RcBrand)
* \[ ] Note: `Once` trait does NOT require `get_or_try_init` (nightly-only). We store `Result<A, LazyError>` in the cell and use stable `get_or_init`.
* \[ ] Add free functions `pointer_new`, `ref_counted_new`, and `send_ref_counted_new`
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
* \[ ] Implement `ThunkWrapper` for `RcBrand`:
  * \[ ] `type Cell<T> = RefCell<Option<T>>`
  * \[ ] `fn new_cell<T>` returns `RefCell::new(value)`
  * \[ ] `fn take<T>` uses `cell.borrow_mut().take()`
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
* \[ ] Implement `SendRefCountedPointer` for `ArcBrand`:
  * \[ ] `type SendOf<T: ?Sized + Send + Sync> = Arc<T>`
  * \[ ] `fn send_new<T: Send + Sync>(value: T) -> Arc<T>`
* \[ ] Implement `ThunkWrapper` for `ArcBrand`:
  * \[ ] `type Cell<T> = Mutex<Option<T>>`
  * \[ ] `fn new_cell<T>` returns `Mutex::new(value)`
  * \[ ] `fn take<T>` uses graceful poisoning: `cell.lock().unwrap_or_else(|p| p.into_inner()).take()`
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

### 2.2 Implementation via UnsizedCoercible Blanket Impl

* \[ ] Create `fp-library/src/types/fn_brand.rs`
* \[ ] Implement blanket `Function` for `FnBrand<P: UnsizedCoercible>` using `P::coerce_fn`
* \[ ] Implement blanket `ClonableFn` for `FnBrand<P: UnsizedCoercible>` using `P::coerce_fn`
* \[ ] Implement blanket `Semigroupoid` for `FnBrand<P: UnsizedCoercible>`
* \[ ] Implement blanket `Category` for `FnBrand<P: UnsizedCoercible>`
* \[ ] Implement blanket `SendClonableFn` for `FnBrand<P: SendUnsizedCoercible>` using `P::coerce_fn_send`
* \[ ] Verify `FnBrand<RcBrand>` does NOT implement `SendClonableFn` (RcBrand doesn't impl SendUnsizedCoercible)
* \[ ] Note: Third-party brands get FnBrand support by implementing `UnsizedCoercible`
* \[ ] Note: Thread-safe third-party brands additionally implement `SendUnsizedCoercible`

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
* \[ ] Define `LazyInner` struct for shared inner state:
  ```rust
  struct LazyInner<'a, PtrBrand, OnceBrand, FnBrand, A>
  where
      PtrBrand: RefCountedPointer + ThunkWrapper,
      OnceBrand: Once,
      FnBrand: ClonableFn,
      (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>,
  {
      /// Stores Result<A, LazyError> to enable panic-safe evaluation with stable Rust
      once: <OnceBrand as Once>::Of<Result<A, LazyError>>,
      /// Thunk wrapped in ThunkWrapper::Cell for interior mutability
      /// Uses ValidLazyCombination::ThunkOf for correct Send+Sync bounds
      thunk: <PtrBrand as ThunkWrapper>::Cell<
          <() as ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>>::ThunkOf<'a, A>
      >,
  }
  ```
* \[ ] New `Lazy` structure with 4 type parameters and `ValidLazyCombination` enforcement:
  ```rust
  pub struct Lazy<'a, PtrBrand, OnceBrand, FnBrand, A>(
      <PtrBrand as RefCountedPointer>::CloneableOf<LazyInner<'a, PtrBrand, OnceBrand, FnBrand, A>>,
  )
  where
      PtrBrand: RefCountedPointer + ThunkWrapper,
      OnceBrand: Once,
      FnBrand: ClonableFn,
      (): ValidLazyCombination<PtrBrand, OnceBrand, FnBrand>;
  ```
* \[ ] Note: 4 type parameters - FnBrand is separate to enable generic code over both thread-local and thread-safe variants
* \[ ] Note: Thunk stored in `Option<..>` wrapped in `ThunkWrapper::Cell` for cleanup
* \[ ] Note: OnceCell stores `Result<A, LazyError>` to avoid nightly-only `get_or_try_init`

### 3.2 Core Implementation

* \[ ] Implement `Lazy::new(thunk)` method
  * \[ ] Takes `ValidLazyCombination::ThunkOf` (ensures correct Send+Sync for ArcLazy)
  * \[ ] Creates new OnceCell via `OnceBrand::new()`
  * \[ ] Wraps thunk in `Option::Some` via `PtrBrand::new_cell`
  * \[ ] Wraps `LazyInner { once, thunk }` in `PtrBrand::cloneable_new`
* \[ ] Implement `Lazy::force_ref(&self)` method returning `Result<&A, LazyError>`
  * \[ ] Takes `&self` (shared semantics)
  * \[ ] Dereferences through `CloneableOf` pointer
  * \[ ] Uses stable `OnceCell::get_or_init` (NOT nightly `get_or_try_init`)
  * \[ ] Cell stores `Result<A, LazyError>`, so stable API suffices
  * \[ ] On first call: takes thunk via `PtrBrand::take` with `expect()` (unreachable None)
  * \[ ] Wraps thunk call in `catch_unwind` with `AssertUnwindSafe` for panic safety
  * \[ ] Use `LazyError::from_panic(payload)` to preserve panic message for debugging
  * \[ ] Use `LazyError::poisoned()` for secondary access (result already in cell)
  * \[ ] Document `AssertUnwindSafe` invariant with detailed safety analysis:
    * Thunk ownership transferred (taken) before invocation
    * Result captures outcome (OnceCell stores Result\<A, LazyError>)
    * No shared mutable state during execution
    * Single-writer guarantee from OnceCell::get\_or\_init
  * \[ ] Returns `Ok(&A)` on success, `Err(LazyError)` if thunk panics
  * \[ ] Note: ThunkConsumed case unreachable due to `get_or_init`'s single-execution guarantee
* \[ ] Implement `Lazy::force(&self)` method returning `Result<A, LazyError>`
  * \[ ] Calls `force_ref` and clones result on success
  * \[ ] Requires `A: Clone`
* \[ ] Implement `Lazy::force_or_panic(&self)` convenience method
  * \[ ] Calls `force` and unwraps with expect
  * \[ ] For use when panic on failure is acceptable
* \[ ] Implement `Lazy::try_get_ref(&self)` method
  * \[ ] Returns `Option<&A>` without forcing (None if not forced or if poisoned)
  * \[ ] Must handle `Result<A, LazyError>` stored in cell
* \[ ] Implement `Lazy::try_get(&self)` method
  * \[ ] Returns `Option<A>` (cloned) without forcing
* \[ ] Implement `Lazy::is_forced(&self)` method
  * \[ ] Returns `bool` indicating if value computed successfully (not poisoned)
* \[ ] Implement `Lazy::is_poisoned(&self)` method
  * \[ ] Returns `bool` indicating if thunk panicked during evaluation
* \[ ] Implement `Lazy::try_into_result(self) -> Result<Result<A, LazyError>, Self>` method
  * \[ ] Attempts to extract owned value if sole reference (strong\_count == 1)
  * \[ ] Returns `Ok(Ok(value))` if unique and forced successfully
  * \[ ] Returns `Ok(Err(LazyError))` if unique and poisoned
  * \[ ] Returns `Err(self)` if shared or not yet forced
  * \[ ] Note: Requires `RefCountedPointer::try_unwrap` extension (deferred)
* \[ ] Add documentation with type signatures

### 3.3 Clone and Debug Implementation

* \[ ] Implement `Clone` for `Lazy`
* \[ ] Clone must be cheap (just reference count increment)
* \[ ] Verify clones share memoization state (via test)
* \[ ] Implement `Debug` for `Lazy` where `A: Debug`:
  * \[ ] Shows "Lazy::Unforced" before forcing
  * \[ ] Shows "Lazy::Forced(value)" after forcing with value's Debug output
  * \[ ] Shows "Lazy::Poisoned" if thunk panicked

### 3.4 TrySemigroup, TryMonoid, and Type Class Implementations

* \[ ] Create `fp-library/src/classes/try_semigroup.rs` with `TrySemigroup` trait
* \[ ] Create `fp-library/src/classes/try_monoid.rs` with `TryMonoid` trait
* \[ ] Implement `TrySemigroup` for `Lazy` where `A: Semigroup + Clone`
  * \[ ] `try_combine` returns NEW lazy that defers forcing until demanded (truly lazy)
  * \[ ] Inner closure captures x and y, forces both at evaluation time
  * \[ ] Panics from x or y are caught by force\_ref's catch\_unwind
  * \[ ] Type Error = LazyError
* \[ ] Implement `TryMonoid` for `Lazy` where `A: Monoid + Clone`
  * \[ ] `try_empty` creates `Lazy` returning `Monoid::empty()` (never fails)
* \[ ] **DO NOT** implement `Semigroup` for `Lazy`:
  * \[ ] Rationale: Semigroup laws require total functions
  * \[ ] A panicking `combine` violates algebraic laws users depend on
  * \[ ] Document: use `TrySemigroup::try_combine` for safe composition
* \[ ] **DO NOT** implement `Monoid` for `Lazy`:
  * \[ ] Same rationale as Semigroup
  * \[ ] `try_empty` from TryMonoid is always safe
* \[ ] Implement `Defer` for `LazyBrand<PtrBrand, OnceBrand, FnBrand>`
  * \[ ] `defer(f)` creates `Lazy` that calls f, then forces result using `force_or_panic`
  * \[ ] Inner Lazy created on-demand when outer is forced (optimisation)

### 3.5 Kind Implementation

* \[ ] Update `LazyBrand<PtrBrand, OnceBrand, FnBrand>` in `fp-library/src/brands.rs` (3 parameters)
* \[ ] Update `impl_kind!` for `LazyBrand`
* \[ ] Verify kind signature matches expected HKT pattern

### 3.6 Free Functions

* \[ ] Update `lazy_new` free function (or rename)
* \[ ] Update `lazy_force` free function
* \[ ] Re-export in `fp-library/src/functions.rs`

### 3.7 Type Aliases

* \[ ] Add `pub type RcLazy<'a, A> = Lazy<'a, RcBrand, OnceCellBrand, RcFnBrand, A>;`
* \[ ] Add `pub type ArcLazy<'a, A> = Lazy<'a, ArcBrand, OnceLockBrand, ArcFnBrand, A>;`
* \[ ] Document thread-safety characteristics of each alias

### 3.8 Phase 3 Tests

* \[ ] Unit test: `Lazy::new` + `Lazy::force` returns `Ok(value)`
* \[ ] Unit test: `Lazy::force_ref` returns `Ok(&value)`
* \[ ] **Critical test**: Thunk is only called once across all clones
* \[ ] **Critical test**: Clones share memoization (counter test)
* \[ ] **Critical test**: Thunk is cleared after forcing (weak ref test)
* \[ ] Unit test: `Lazy::is_forced` returns correct state
* \[ ] Unit test: `Lazy::is_poisoned` returns `false` before forcing, `true` after panic
* \[ ] Unit test: `Lazy::try_get_ref` returns `None` before forcing
* \[ ] Unit test: `Lazy::try_get_ref` returns `Some(&value)` after successful forcing
* \[ ] Unit test: `Lazy::try_get_ref` returns `None` after panic (poisoned state)
* \[ ] **Panic safety test**: `force_ref` returns `Err(LazyError)` when thunk panics
* \[ ] **Panic safety test**: All clones see `Err(LazyError)` after panic (shared poisoned state)
* \[ ] Unit test: `force_or_panic` panics with appropriate message on error
* \[ ] Unit test: `TrySemigroup::try_combine` returns lazy that computes combined value
* \[ ] Unit test: `TrySemigroup::try_combine` defers forcing (truly lazy semantics)
* \[ ] Unit test: `TrySemigroup::try_combine` result's force returns `Err(LazyError)` when inner thunk fails
* \[ ] Unit test: `TryMonoid::try_empty` works correctly
* \[ ] Compile-fail test: `Lazy` does NOT implement `Semigroup`
* \[ ] Compile-fail test: `Lazy` does NOT implement `Monoid`
* \[ ] Unit test: `Debug` shows "Lazy::Unforced" before forcing
* \[ ] Unit test: `Debug` shows "Lazy::Forced(value)" after forcing
* \[ ] Unit test: `Debug` shows "Lazy::Poisoned" after panic
* \[ ] Unit test: `Defer::defer` works correctly
* \[ ] Property test: TrySemigroup associativity law (when all succeed)
* \[ ] Property test: TryMonoid identity laws (when all succeed)
* \[ ] Compile-fail test: `RcLazy` is `!Send`
* \[ ] Compile-success test: `ArcLazy<A: Send + Sync>` is `Send + Sync`
* \[ ] Compile-fail test: `Lazy<ArcBrand, OnceCellBrand, ArcFnBrand, _>` fails (ValidLazyCombination)
* \[ ] Compile-fail test: `Lazy<RcBrand, OnceLockBrand, RcFnBrand, _>` fails (ValidLazyCombination)
* \[ ] Compile-fail test: `Lazy<ArcBrand, OnceLockBrand, RcFnBrand, _>` fails (FnBrand mismatch)

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
  * \[ ] `ThunkWrapper` entry in Type Classes table
  * \[ ] `ValidLazyCombination` entry in Type Classes table
  * \[ ] `RcBrand` entry in Data Types table
  * \[ ] `ArcBrand` entry in Data Types table
  * \[ ] Update `LazyBrand` entry to reflect new semantics
* \[ ] Update `docs/architecture.md`:
  * \[ ] Document the Pointer → RefCountedPointer → SendRefCountedPointer pattern
  * \[ ] Document the macro-based impl approach for FnBrand
  * \[ ] Document `ThunkWrapper` and `ValidLazyCombination` patterns
* \[ ] Add "Extending FnBrand for Custom Pointers" documentation:
  * \[ ] Add section to `fp-library/src/types/fn_brand.rs` module docs
  * \[ ] Include complete working example with `Function`, `ClonableFn`, `Semigroupoid`, `Category`
  * \[ ] Document `SendClonableFn` implementation for thread-safe variants
  * \[ ] Explain unsized coercion limitation and why macro/manual impl is needed
* \[ ] Update `docs/todo.md` to mark Lazy memoization item as addressed
* \[ ] Update `docs/limitations.md`:
  * \[ ] Document FnBrand extensibility limitation (macro required)
  * \[ ] Document thunk cleanup differences between RcBrand (RefCell) and ArcBrand (Mutex)

### 4.4 Final Verification

* \[ ] `cargo test` passes (all tests)
* \[ ] `cargo clippy` passes (no warnings)
* \[ ] `cargo doc` builds without warnings
* \[ ] Review generated documentation for clarity
* \[ ] Run benchmarks to verify no performance regression

***

## Phase 5: Concurrency Testing with Loom

### 5.1 Setup

* \[ ] Add `loom = "0.7"` to dev-dependencies in `fp-library/Cargo.toml`
* \[ ] Create `fp-library/tests/loom_tests.rs`

### 5.2 Loom Tests for ArcLazy

* \[ ] Test: Concurrent force from multiple threads - thunk called exactly once
* \[ ] Test: All threads see same memoized value
* \[ ] Test: Panic propagation across threads - all threads see `Err(LazyError)`
* \[ ] Test: No deadlocks with Mutex in ThunkWrapper

### 5.3 Running Loom Tests

* \[ ] Add CI command: `RUSTFLAGS="--cfg loom" cargo test --test loom_tests`
* \[ ] Document loom testing in README or CONTRIBUTING

***

## Release Preparation

### Changelog

* \[ ] Add entry to `fp-library/CHANGELOG.md` under `[Unreleased]`:
  ```markdown
  ### Changed (Breaking)
  - `Lazy` now uses shared memoization semantics (Haskell-like)
    - Clones share memoization state
    - `force` and `force_ref` take `&self` and return `Result<_, LazyError>`
    - `Lazy` has 4 type parameters: PtrBrand, OnceBrand, FnBrand, A
    - Thunks cleared after forcing to free captured values
    - OnceCell stores `Result<A, LazyError>` for panic-safe evaluation
  - `RcFnBrand` is now a type alias for `FnBrand<RcBrand>`
  - `ArcFnBrand` is now a type alias for `FnBrand<ArcBrand>`

  ### Added
  - `Pointer` base trait for heap-allocated pointers
  - `RefCountedPointer` extension trait with `CloneableOf` for shared ownership
  - `SendRefCountedPointer` extension trait with `SendOf` for thread-safe pointers
  - `ThunkWrapper` trait for pointer-brand-specific thunk storage
  - `ValidLazyCombination` marker trait with `ThunkOf` associated type enforcing valid PtrBrand/OnceBrand/FnBrand triples and selecting correct thunk type for thread safety
  - `LazyError` struct with panic payload for debugging
  - `PanicPayload` wrapper with `panic_message()` and `into_inner()` methods
  - `UnsizedCoercible` trait for basic function coercion
  - `SendUnsizedCoercible` trait for thread-safe function coercion
  - `TrySemigroup` trait for fallible semigroup operations
  - `TryMonoid` trait for fallible monoid operations
  - `RcBrand` and `ArcBrand` implementing the pointer hierarchy
  - `BoxBrand` placeholder for future unique ownership support
  - `FnBrand<PtrBrand>` generic function brand
  - `RcLazy` and `ArcLazy` type aliases for common configurations
  - `Lazy::force_ref(&self) -> Result<&A, LazyError>` method (avoids cloning)
  - `Lazy::force(&self) -> Result<A, LazyError>` method
  - `Lazy::force_or_panic(&self) -> A` convenience method
  - `Lazy::try_get_ref(&self) -> Option<&A>` method
  - `Lazy::is_forced(&self) -> bool` method
  - `Lazy::is_poisoned(&self) -> bool` method

  ### Removed
  - Old value-semantic `Lazy` implementation
  - Separate `rc_fn.rs` and `arc_fn.rs` files (merged into `fn_brand.rs`)
  ```

***

## Implementation Notes

### Key Design Decisions

1. **Three-level trait hierarchy**: `Pointer` → `RefCountedPointer` → `SendRefCountedPointer` allows future `BoxBrand` support at the `Pointer` level without reference counting

2. **Additional Associated Type pattern**: `RefCountedPointer` adds `CloneableOf` with `Clone` bound; `SendRefCountedPointer` adds `SendOf` with `Send + Sync` bounds rather than using invalid `for<T: Trait>` syntax (which Rust doesn't support)

3. **Macro for unsized coercion**: `impl_fn_brand!` macro handles the `Rc::new`/`Arc::new` calls that perform unsized coercion to `dyn Fn`

4. **Four type parameters for Lazy**: `Lazy<PtrBrand, OnceBrand, FnBrand, A>` - FnBrand is separate to enable generic code over both thread-local and thread-safe variants

5. **Type aliases for ergonomics**: `RcFnBrand`, `ArcFnBrand`, `RcLazy`, `ArcLazy` provide good defaults

6. **ValidLazyCombination marker trait with ThunkOf**: Enforces valid `PtrBrand`/`OnceBrand`/`FnBrand` triples at compile time, preventing misconfigurations. The `ThunkOf` associated type ensures `ArcLazy` uses `SendClonableFn::SendOf` (with `Send + Sync` bounds) while `RcLazy` uses `ClonableFn::Of`.

7. **ThunkWrapper trait**: Abstracts over `RefCell<Option<Thunk>>` (for Rc) and `Mutex<Option<Thunk>>` (for Arc) to enable thunk cleanup after forcing. Arc uses graceful mutex poisoning recovery.

8. **Panic-safe evaluation with stable Rust**: `force_ref` returns `Result<&A, LazyError>` and uses stable `get_or_init` (NOT nightly-only `get_or_try_init`). The OnceCell stores `Result<A, LazyError>` to capture both success and error states.

9. **LazyError with panic payload**: Stores the original panic payload for debugging. `LazyError::from_panic(payload)` captures the payload, `LazyError::poisoned()` creates secondary errors without payload. `PanicPayload::panic_message()` attempts to extract string messages via downcasting.

10. **Triple force methods**: `force_ref(&self) -> Result<&A, LazyError>` for explicit error handling; `force(&self) -> Result<A, LazyError>` clones; `force_or_panic(&self) -> A` for convenience

11. **AssertUnwindSafe invariant**: The `catch_unwind` in `force_ref` uses `AssertUnwindSafe` safely because: (1) thunk is taken before invocation, (2) Result stored in OnceCell captures panic state, (3) no mutable references to shared state exist during thunk execution

12. **UnsizedCoercible/SendUnsizedCoercible traits**: Two-level hierarchy following ClonableFn → SendClonableFn pattern. `UnsizedCoercible` provides basic function coercion, `SendUnsizedCoercible` adds thread-safe coercion. RcBrand only implements `UnsizedCoercible` (no panicking methods).

13. **TrySemigroup/TryMonoid only (no Semigroup/Monoid)**: Lazy does NOT implement Semigroup or Monoid because those traits require total functions. A panicking `combine` violates algebraic laws. Users must use `TrySemigroup::try_combine` which makes fallibility explicit.

14. **Lazy TrySemigroup is truly lazy**: `try_combine` returns a NEW lazy that defers forcing until demanded. This preserves lazy semantics and allows building lazy computations without immediate failure.

15. **Debug implementation**: Shows `Lazy::Unforced`, `Lazy::Forced(value)`, or `Lazy::Poisoned` depending on current state, requires `A: Debug`.

16. **try\_into\_result method**: Allows extracting owned value from unique Lazy reference without cloning. Returns `Err(self)` if shared or not yet forced.

17. **ValidLazyCombination impl-for-() pattern**: Uses "type witness" pattern where `()` carries the impl. The constraint `(): ValidLazyCombination<P, O, F>` means "unit type has impl for this combination". This is idiomatic Rust for compile-time validation.

18. **Loom concurrency testing**: Exhaustive testing of all thread interleavings for `ArcLazy` to verify correct synchronization.

### Files Summary

| Action | File |
|--------|------|
| Create | `fp-library/src/classes/pointer.rs` |
| Create | `fp-library/src/classes/try_semigroup.rs` |
| Create | `fp-library/src/classes/try_monoid.rs` |
| Create | `fp-library/src/types/rc_ptr.rs` |
| Create | `fp-library/src/types/arc_ptr.rs` |
| Create | `fp-library/src/types/fn_brand.rs` |
| Create | `fp-library/tests/loom_tests.rs` |
| Delete | `fp-library/src/types/rc_fn.rs` |
| Delete | `fp-library/src/types/arc_fn.rs` |
| Rewrite | `fp-library/src/types/lazy.rs` |
| Modify | `fp-library/src/brands.rs` |
| Modify | `fp-library/src/classes.rs` |
| Modify | `fp-library/src/types.rs` |
| Modify | `fp-library/src/functions.rs` |
| Modify | `fp-library/Cargo.toml` (add loom dev-dependency) |

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

   assert_eq!(Lazy::force(&lazy), Ok(42));
   assert_eq!(counter.get(), 1);  // Called once
   assert_eq!(Lazy::force(&lazy2), Ok(42));
   assert_eq!(counter.get(), 1);  // Still 1 - shared!
   ```

2. **Panic safety test with message** (must pass):
   ```rust
   let lazy = RcLazy::new(
       clonable_fn_new::<RcFnBrand, _, _>(|_| -> i32 { panic!("computation failed") })
   );
   let lazy2 = lazy.clone();

   let err = Lazy::force_ref(&lazy).unwrap_err();
   // First caller gets the original panic message
   assert_eq!(err.panic_message(), Some("computation failed"));
   assert!(err.has_payload());

   // Second call on any clone sees poisoned error (no payload)
   let err2 = Lazy::force_ref(&lazy2).unwrap_err();
   assert_eq!(err2.panic_message(), None);
   assert!(!err2.has_payload());
   ```

3. **Thread safety test** (for ArcLazy):
   ```rust
   let lazy: ArcLazy<i32> = ArcLazy::new(
       send_clonable_fn_new::<ArcFnBrand, _, _>(|_| 42)
   );
   std::thread::spawn(move || Lazy::force(&lazy).unwrap()).join().unwrap();
   ```

4. **Compile-fail for RcLazy Send**:
   ```rust
   let lazy: RcLazy<i32> = RcLazy::new(/* ... */);
   std::thread::spawn(move || Lazy::force(&lazy)); // Should fail!
   ```
