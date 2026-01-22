# Pointer Abstraction Implementation Checklist

This checklist tracks progress on implementing the `Pointer` → `RefCountedPointer` → `SendRefCountedPointer` type class hierarchy, refactoring `ClonableFn` to use it, and rewriting `Lazy` with shared memoization semantics. See [plan.md](./plan.md) for full context and design details.

**Note**: This is a breaking change. Backward compatibility is not maintained.

---

## Phase 1: Pointer Trait Foundation

### 1.1 Trait Definition

- [ ] Add missing Kind trait to `fp-library/src/kinds.rs`:
- [ ] Add `def_kind! { type Of<'a, A>; }` to support `SendDefer`
- [ ] Create `fp-library/src/classes/pointer.rs`
- [ ] Define `Pointer` base trait:

```rust
pub trait Pointer {
	/// The pointer type constructor.
	type Of<T: ?Sized>: Deref<Target = T>;

	/// Wraps a sized value in the pointer.
	///
	/// ### Type Signature
	///
	/// `forall a. Pointer p => a -> p a`
	///
	/// ### Type Parameters
	///
	/// * `T`: The type of the value to wrap.
	///
	/// ### Parameters
	///
	/// * `value`: The value to wrap in the pointer.
	///
	/// ### Returns
	///
	/// A new pointer of type `Of<T>` containing the value.
	fn new<T>(value: T) -> Self::Of<T> where Self::Of<T>: Sized;
}
```

- [ ] Define `RefCountedPointer` extension trait:

```rust
pub trait RefCountedPointer: Pointer {
	/// The clonable pointer type constructor.
	type CloneableOf<T: ?Sized>: Clone + Deref<Target = T>;

	/// Wraps a sized value in a clonable pointer.
	///
	/// ### Type Signature
	///
	/// `forall a. RefCountedPointer p => a -> p a`
	///
	/// ### Type Parameters
	///
	/// * `T`: The type of the value to wrap.
	///
	/// ### Parameters
	///
	/// * `value`: The value to wrap in the clonable pointer.
	///
	/// ### Returns
	///
	/// A new clonable pointer of type `CloneableOf<T>` containing the value.
	fn cloneable_new<T>(value: T) -> Self::CloneableOf<T> where Self::CloneableOf<T>: Sized;

	/// Attempts to unwrap the inner value if this is the sole reference.
	///
	/// ### Type Signature
	///
	/// `forall a. RefCountedPointer p => p a -> Result a (p a)`
	///
	/// ### Type Parameters
	///
	/// * `T`: The type of the value contained in the pointer.
	///
	/// ### Parameters
	///
	/// * `ptr`: The pointer to attempt to unwrap.
	///
	/// ### Returns
	///
	/// `Ok(value)` if the pointer was the sole reference, or `Err(ptr)` with the original pointer if shared.
	fn try_unwrap<T>(ptr: Self::CloneableOf<T>) -> Result<T, Self::CloneableOf<T>>;
}
```

- [ ] Define `SendRefCountedPointer` extension trait with `SendOf`:

```rust
pub trait SendRefCountedPointer: RefCountedPointer {
	/// The thread-safe pointer type constructor.
	type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;

	/// Wraps a sized value in a thread-safe pointer.
	///
	/// ### Type Signature
	///
	/// `forall a. (SendRefCountedPointer p, Send a, Sync a) => a -> p a`
	///
	/// ### Type Parameters
	///
	/// * `T`: The type of the value to wrap, must be `Send + Sync`.
	///
	/// ### Parameters
	///
	/// * `value`: The value to wrap in the thread-safe pointer.
	///
	/// ### Returns
	///
	/// A new thread-safe pointer of type `SendOf<T>` containing the value.
	fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>
	where
		Self::SendOf<T>: Sized;
}
```

- [ ] Define `ThunkWrapper` trait for pointer-brand-specific thunk storage:

```rust
/// Trait for pointer-brand-specific thunk storage.
///
/// ### Type Signature
///
/// `class ThunkWrapper w where`
///
pub trait ThunkWrapper {
	/// The cell type used for thunk storage.
	type Cell<T>;

	/// Creates a new cell containing an optional thunk.
	///
	/// ### Type Parameters
	///
	/// * `T`: The type of the thunk to store.
	///
	/// ### Parameters
	///
	/// * `value`: The optional thunk to store.
	///
	/// ### Returns
	///
	/// A new cell containing the thunk.
	fn new_cell<T>(value: Option<T>) -> Self::Cell<T>;

	/// Takes the thunk out of the cell, leaving `None` in its place.
	///
	/// ### Type Parameters
	///
	/// * `T`: The type of the thunk to take.
	///
	/// ### Parameters
	///
	/// * `cell`: The cell to take the thunk from.
	///
	/// ### Returns
	///
	/// The thunk if it was present, `None` otherwise.
	fn take<T>(cell: &Self::Cell<T>) -> Option<T>;
}
```

- [ ] Define `UnsizedCoercible` trait for basic function coercion:
```rust
/// Trait for pointer brands that can coerce to `dyn Fn`.
pub trait UnsizedCoercible: RefCountedPointer {
	/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
	///
	/// ### Type Signature
	///
	/// `forall a b. UnsizedCoercible p => (a -> b) -> p (a -> b)`
	///
	/// ### Type Parameters
	///
	/// * `A`: The input type of the function.
	/// * `B`: The output type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to coerce.
	///
	/// ### Returns
	///
	/// A clonable pointer containing the coerced function.
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::CloneableOf<dyn 'a + Fn(A) -> B>;
}
```
- [ ] Define `SendUnsizedCoercible` extension trait for thread-safe coercion:
```rust
/// Extension trait for thread-safe function coercion.
/// Follows ClonableFn → SendClonableFn pattern.
pub trait SendUnsizedCoercible: UnsizedCoercible + SendRefCountedPointer {
	/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
	///
	/// ### Type Signature
	///
	/// `forall a b. SendUnsizedCoercible p => (a -> b) -> p (a -> b)`
	///
	/// ### Type Parameters
	///
	/// * `A`: The input type of the function.
	/// * `B`: The output type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to coerce, must be `Send + Sync`.
	///
	/// ### Returns
	///
	/// A clonable pointer containing the coerced thread-safe function.
	fn coerce_fn_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::CloneableOf<dyn 'a + Fn(A) -> B + Send + Sync>;
}
```
- [ ] Implement `UnsizedCoercible` for `RcBrand` (basic coercion only)
- [ ] Implement `UnsizedCoercible` for `ArcBrand`
- [ ] Implement `SendUnsizedCoercible` for `ArcBrand` only (not RcBrand)
- [ ] Note: `Once` trait does NOT require `get_or_try_init` (nightly-only). We store `Result<A, LazyError>` in the cell and use stable `get_or_init` and `into_inner`.
- [ ] Add free functions `pointer_new`, `ref_counted_new`, `send_ref_counted_new`, and `try_unwrap`
- [ ] Add comprehensive documentation following `docs/architecture.md` standards
- [ ] Add module-level examples

### 1.2 Brand Definitions

- [ ] Add `RcBrand` struct to `fp-library/src/brands.rs`
- [ ] Add `ArcBrand` struct to `fp-library/src/brands.rs`
- [ ] Add `BoxBrand` struct placeholder (future extension)
- [ ] Add documentation for all brands explaining their use cases

### 1.3 Rc Implementation

- [ ] Create `fp-library/src/types/rc_ptr.rs`
- [ ] Implement `Pointer` for `RcBrand`:
- [ ] `type Of<T: ?Sized> = Rc<T>`
- [ ] `fn new<T>(value: T) -> Rc<T>` using `Rc::new`
- [ ] Implement `RefCountedPointer` for `RcBrand`:
- [ ] `type CloneableOf<T: ?Sized> = Rc<T>` (same as `Of<T>`)
- [ ] `fn cloneable_new<T>(value: T) -> Rc<T>`
- [ ] `fn try_unwrap<T>(ptr: Rc<T>) -> Result<T, Rc<T>>` using `Rc::try_unwrap`
- [ ] Implement `ThunkWrapper` for `RcBrand`:
- [ ] `type Cell<T> = RefCell<Option<T>>`
- [ ] `fn new_cell<T>` returns `RefCell::new(value)`
- [ ] `fn take<T>` uses `cell.borrow_mut().take()`
- [ ] Add documentation and examples
- [ ] Verify `RcBrand` does NOT implement `SendRefCountedPointer`

### 1.4 Arc Implementation

- [ ] Create `fp-library/src/types/arc_ptr.rs`
- [ ] Implement `Pointer` for `ArcBrand`:
- [ ] `type Of<T: ?Sized> = Arc<T>`
- [ ] `fn new<T>(value: T) -> Arc<T>` using `Arc::new`
- [ ] Implement `RefCountedPointer` for `ArcBrand`:
- [ ] `type CloneableOf<T: ?Sized> = Arc<T>` (same as `Of<T>`)
- [ ] `fn cloneable_new<T>(value: T) -> Arc<T>`
- [ ] `fn try_unwrap<T>(ptr: Arc<T>) -> Result<T, Arc<T>>` using `Arc::try_unwrap`
- [ ] Implement `SendRefCountedPointer` for `ArcBrand`:
- [ ] `type SendOf<T: ?Sized + Send + Sync> = Arc<T>`
- [ ] `fn send_new<T: Send + Sync>(value: T) -> Arc<T>`
- [ ] Implement `ThunkWrapper` for `ArcBrand` using `parking_lot::Mutex`:
- [ ] `type Cell<T> = parking_lot::Mutex<Option<T>>`
- [ ] `fn new_cell<T>` returns `Mutex::new(value)`
- [ ] `fn take<T>` uses `cell.lock().take()` (no poisoning with parking_lot)
- [ ] ⚠️ Note: Recursive forcing **will deadlock** at `OnceLock::get_or_init`, not at the Mutex
- [ ] Document deadlock behavior in module docs (see Known Limitations in plan.md)
- [ ] Add `parking_lot = "0.12"` to dependencies in `fp-library/Cargo.toml`
- [ ] Add documentation and examples

### 1.5 Free Functions & Re-exports

- [ ] Add free function `pointer_new<P: Pointer, T>` to `fp-library/src/classes/pointer.rs`
- [ ] Add free function `ref_counted_new<P: RefCountedPointer, T>` to same file
- [ ] Add free function `send_ref_counted_new<P: SendRefCountedPointer, T: Send + Sync>` to same file
- [ ] Add free function `try_unwrap<P: RefCountedPointer, T>` to same file
- [ ] Update `fp-library/src/classes.rs` to re-export `pointer` module
- [ ] Update `fp-library/src/functions.rs` to re-export free functions
- [ ] Update `fp-library/src/types.rs` to re-export `rc_ptr` and `arc_ptr` modules

### 1.6 Phase 1 Tests

- [ ] Unit test: `RcBrand::new` creates `Rc<T>`
- [ ] Unit test: `ArcBrand::new` creates `Arc<T>`
- [ ] Unit test: `RcBrand::cloneable_new` creates `Rc<T>`
- [ ] Unit test: `ArcBrand::cloneable_new` creates `Arc<T>`
- [ ] Unit test: `Clone` works for `RcBrand::CloneableOf<T>`
- [ ] Unit test: `Clone` works for `ArcBrand::CloneableOf<T>`
- [ ] Unit test: `Deref` works correctly for both
- [ ] Unit test: `RcBrand::try_unwrap` returns `Ok(value)` when sole reference
- [ ] Unit test: `RcBrand::try_unwrap` returns `Err(ptr)` when shared
- [ ] Unit test: `ArcBrand::try_unwrap` returns `Ok(value)` when sole reference
- [ ] Unit test: `ArcBrand::try_unwrap` returns `Err(ptr)` when shared
- [ ] Compile-fail test: `Rc<T>` is `!Send`
- [ ] Compile-success test: `Arc<T: Send + Sync>` is `Send + Sync`

---

## Phase 2: FnBrand Refactor

### 2.1 Brand Structure

- [ ] Add `FnBrand<PtrBrand: RefCountedPointer>` struct to `fp-library/src/brands.rs`
- [ ] Add type alias `pub type RcFnBrand = FnBrand<RcBrand>;`
- [ ] Add type alias `pub type ArcFnBrand = FnBrand<ArcBrand>;`
- [ ] Add documentation explaining the parameterization

### 2.2 Implementation via UnsizedCoercible Blanket Impl

- [ ] Create `fp-library/src/types/fn_brand.rs`
- [ ] Implement blanket `Function` for `FnBrand<P: UnsizedCoercible>` using `P::coerce_fn`
- [ ] Implement blanket `ClonableFn` for `FnBrand<P: UnsizedCoercible>` using `P::coerce_fn`
- [ ] Implement blanket `Semigroupoid` for `FnBrand<P: UnsizedCoercible>`
- [ ] Implement blanket `Category` for `FnBrand<P: UnsizedCoercible>`
- [ ] Implement blanket `SendClonableFn` for `FnBrand<P: SendUnsizedCoercible>` using `P::coerce_fn_send`
- [ ] Verify `FnBrand<RcBrand>` does NOT implement `SendClonableFn` (RcBrand doesn't impl SendUnsizedCoercible)
- [ ] Note: Third-party brands get FnBrand support by implementing `UnsizedCoercible`
- [ ] Note: Thread-safe third-party brands additionally implement `SendUnsizedCoercible`

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

### 3.1 Lazy Traits & Error

- [ ] Define `LazyConfig` configuration trait with associated types (Configuration Struct Pattern):
```rust
/// Configuration trait for valid Lazy brand combinations.
///
/// Uses Configuration Struct Pattern instead of `impl Trait<A,B,C> for ()`
/// to enable third-party extension without orphan rule violations.
/// Configuration trait for valid Lazy brand combinations.
///
/// ### Type Signature
///
/// `class LazyConfig c where`
///
pub trait LazyConfig: 'static {
	/// The pointer brand (RcBrand or ArcBrand).
	type PtrBrand: RefCountedPointer + ThunkWrapper;
	/// The once-cell brand (OnceCellBrand or OnceLockBrand)
	type OnceBrand: Once;
	/// The function brand (RcFnBrand or ArcFnBrand)
	type FnBrand: ClonableFn;
	/// The thunk type - ClonableFn::Of for Rc, SendClonableFn::SendOf for Arc
	type ThunkOf<'a, A>: Clone;
}
```
- [ ] Define `RcLazyConfig` configuration struct:

```rust
/// Configuration for single-threaded lazy evaluation.
pub struct RcLazyConfig;

impl LazyConfig for RcLazyConfig {
	type PtrBrand = RcBrand;
	type OnceBrand = OnceCellBrand;
	type FnBrand = RcFnBrand;
	type ThunkOf<'a, A> = <RcFnBrand as ClonableFn>::Of<'a, (), A>;
}
```

- [ ] Define `ArcLazyConfig` configuration struct:

```rust
/// Configuration for thread-safe lazy evaluation.
pub struct ArcLazyConfig;

impl LazyConfig for ArcLazyConfig {
	type PtrBrand = ArcBrand;
	type OnceBrand = OnceLockBrand;
	type FnBrand = ArcFnBrand;
	type ThunkOf<'a, A> = <ArcFnBrand as SendClonableFn>::SendOf<'a, (), A>;
}
```

- [ ] Define `SendLazyConfig` extension trait for thread-safe configurations:

```rust
/// Extension trait guaranteeing ThunkOf is Send + Sync.
/// Follows ClonableFn → SendClonableFn pattern.
///
/// ### Type Signature
///
/// `class LazyConfig c => SendLazyConfig c where`
///
pub trait SendLazyConfig: LazyConfig {
	/// The thread-safe thunk type.
	type SendThunkOf<'a, A: Send + Sync>: Clone + Send + Sync;
}

impl SendLazyConfig for ArcLazyConfig {
	type SendThunkOf<'a, A: Send + Sync> = <ArcFnBrand as SendClonableFn>::SendOf<'a, (), A>;
}
```

- [ ] Note: `RcLazyConfig` does NOT implement `SendLazyConfig`
- [ ] Document Configuration Struct Pattern benefits in module docs (see plan.md):
- [ ] Third-party extension enabled (no orphan rule violations)
- [ ] Cleaner type signatures (`Config` vs 3 separate parameters)
- [ ] Self-documenting configurations
- [ ] Add `RcLazyConfig` and `ArcLazyConfig` structs to `fp-library/src/brands.rs`
- [ ] Define `LazyError` struct with `Arc<str>` for thread-safe error messages:

````rust
use thiserror::Error;
use std::sync::Arc;

/// Error type for lazy evaluation failures.
/// Uses Arc<str> for thread-safe sharing - NOT raw panic payload.
///
/// ### Why Arc<str> instead of Box<dyn Any + Send>?
///
/// Raw panic payload is Send but NOT Sync, which would make LazyError !Sync,
/// breaking ArcLazy thread safety. Arc<str> is both Send and Sync.
#[derive(Debug, Clone, Error)]
#[error("thunk panicked during evaluation{}", .0.as_ref().map(|m| format!(": {}", m)).unwrap_or_default())]
pub struct LazyError(Option<Arc<str>>);

/// ### Fields
///
/// * `0`: The optional panic message stored as an `Arc<str>`.
///
/// ### Examples
///
/// ```rust
/// use fp_library::{brands::*, classes::*, functions::*};
///
/// let lazy = lazy_new::<RcLazyConfig, _>(clonable_fn_new::<RcFnBrand, _, _>(|_| {
///     panic!("computation failed");
/// }));
///
/// if let Err(e) = lazy_force::<RcLazyConfig, _>(&lazy) {
///     assert_eq!(e.panic_message(), Some("computation failed"));
/// }
/// ```
````

- [ ] Implement `LazyError::from_panic(payload)` constructor
- [ ] Extract message from payload via downcast to `&str` or `String`
- [ ] Store as `Arc<str>` for thread-safe sharing
- [ ] Use generic message for non-string panic payloads
- [ ] Implement `LazyError::poisoned()` for API completeness (not typically used with Arc storage)
- [ ] Implement `LazyError::panic_message() -> Option<&str>`
- [ ] Implement `LazyError::has_message() -> bool`
- [ ] Derive `Clone` for `LazyError` (required for `Arc<LazyError>` pattern)
- [ ] Add `thiserror = "1.0"` to dependencies in `fp-library/Cargo.toml`
- [ ] Create `fp-library/src/classes/send_defer.rs`
- [ ] Define `SendDefer` trait as **independent trait** (NOT extending `Defer`):
```rust
/// Trait for types that support thread-safe deferred evaluation.
///
/// ### HKT Requirement
///
/// This trait extends `Kind` and requires the implementor to support the `type Of<'a, A>` kind signature.
///
/// ### Design Note: Independent from Defer
///
/// Unlike other extension trait pairs (ClonableFn → SendClonableFn),
/// `SendDefer` does NOT extend `Defer`. This is intentional because:
///
/// 1. `Defer::defer` takes a non-Send+Sync thunk
/// 2. If ArcLazy implemented Defer, users could accidentally create
///    non-thread-safe ArcLazy values
/// 3. Making SendDefer independent prevents this misuse at compile time
///
/// This means:
/// - RcLazy implements Defer (not SendDefer)
/// - ArcLazy implements SendDefer (not Defer)
/// - There is no type that implements both
pub trait SendDefer: Kind {
	/// Creates a value from a thread-safe thunk-producing thunk.
	///
	/// ### Type Signature
	///
	/// `forall config a. (SendLazyConfig config, Send a, Sync a, Clone a) => (() -> Lazy config a) -> Lazy config a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to be computed lazily, must be `Clone + Send + Sync`.
	///
	/// ### Parameters
	///
	/// * `thunk`: The computation that produces a `Lazy` value.
	///
	/// ### Returns
	///
	/// A new `Lazy` value that defers the execution of the thunk.
	fn send_defer<'a, A>(thunk: impl 'a + Fn() -> Self::Of<'a, A> + Send + Sync) -> Self::Of<'a, A>
	where
		A: Clone + Send + Sync + 'a;
}
```
- [ ] Note: `SendDefer` is only implemented for `LazyBrand<ArcLazyConfig>`
- [ ] Note: `Defer` is only implemented for `LazyBrand<RcLazyConfig>` (NOT for ArcLazy)

### 3.2 Rewrite Lazy Type

- [ ] Rewrite `fp-library/src/types/lazy.rs`
- [ ] Define `LazyInner` struct for shared inner state using `Config: LazyConfig`:
```rust
struct LazyInner<'a, Config: LazyConfig, A> {
	/// Stores Result<A, Arc<LazyError>> to enable panic-safe evaluation with stable Rust.
	/// Uses Arc<LazyError> so all clones see the same error with the same message.
	once: <Config::OnceBrand as Once>::Of<Result<A, Arc<LazyError>>>,
	/// Thunk wrapped in ThunkWrapper::Cell for interior mutability
	/// Uses Config::ThunkOf for correct Send+Sync bounds
	thunk: <Config::PtrBrand as ThunkWrapper>::Cell<Config::ThunkOf<'a, A>>,
}
```
- [ ] New `Lazy` structure with 2 type parameters using `Config: LazyConfig`:

````rust
/// Lazily-computed value with shared memoization (Haskell-like semantics).
///
/// ### Type Parameters
///
/// * `Config`: A type implementing `LazyConfig` that bundles PtrBrand, OnceBrand, FnBrand.
/// * `A`: The type of the lazily-computed value.
pub struct Lazy<'a, Config: LazyConfig, A>(
	<Config::PtrBrand as RefCountedPointer>::CloneableOf<LazyInner<'a, Config, A>>,
);

/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::*, functions::*};
///
/// let lazy = lazy_new::<RcLazyConfig, _>(
///     clonable_fn_new::<RcFnBrand, _, _>(|_| 42)
/// );
///
/// assert_eq!(lazy_force_cloned::<RcLazyConfig, _>(&lazy), Ok(42));
/// ```
````

- [ ] Note: Only 2 type parameters (Config, A) - Config bundles PtrBrand/OnceBrand/FnBrand
- [ ] Note: Thunk stored in `Option<..>` wrapped in `ThunkWrapper::Cell` for cleanup
- [ ] Note: OnceCell stores `Result<A, Arc<LazyError>>` to avoid nightly-only `get_or_try_init`

### 3.2 Core Implementation

- [ ] Implement `Lazy::new(thunk)` method
- [ ] Takes `Config::ThunkOf<'a, A>` (ensures correct Send+Sync for ArcLazy)
- [ ] Creates new OnceCell via `Config::OnceBrand::new()`
- [ ] Wraps thunk in `Option::Some` via `Config::PtrBrand::new_cell`
- [ ] Wraps `LazyInner { once, thunk }` in `Config::PtrBrand::cloneable_new`
- [ ] Implement `Lazy::force(&self)` method returning `Result<&A, LazyError>`
- [ ] Takes `&self` (shared semantics)
- [ ] Dereferences through `CloneableOf` pointer
- [ ] Uses stable `OnceCell::get_or_init` (NOT nightly `get_or_try_init`)
- [ ] Cell stores `Result<A, Arc<LazyError>>`, so stable API suffices
- [ ] On first call: takes thunk via `Config::PtrBrand::take` with `expect()` (unreachable None)
- [ ] Wraps thunk call in `catch_unwind` with `AssertUnwindSafe` for panic safety
- [ ] Use `LazyError::from_panic(payload)` to preserve panic message for debugging
- [ ] Document `AssertUnwindSafe` invariant with detailed safety analysis:
	- Thunk ownership transferred (taken) before invocation
	- Result captures outcome (OnceCell stores Result\<A, Arc\<LazyError\>\>)
	- No shared mutable state during execution
	- Single-writer guarantee from OnceCell::get_or_init
	- Thunk is taken (moved out) atomically before execution begins
	- If panic occurs, thunk is already consumed (no partial state)
	- OnceCell stores the error, ensuring consistent state for all observers
- [ ] Returns `Ok(&A)` on success, `Err(LazyError)` if thunk panics
- [ ] Note: ThunkConsumed case unreachable due to `get_or_init`'s single-execution guarantee
- [ ] Implement `Lazy::force_cloned(&self)` method returning `Result<A, LazyError>`
- [ ] Calls `force` and clones result on success
- [ ] Requires `A: Clone`
- [ ] Document `A: Clone` limitation in method docs:
	- Shared memoization requires `Clone` for multiple callers
	- Alternatives: use `force`, wrap in `Rc`/`Arc`, or use `try_into_result`
- [ ] Implement `Lazy::force_or_panic(&self)` convenience method
- [ ] Calls `force_cloned` and unwraps with expect
- [ ] Requires `A: Clone`
- [ ] For use when panic on failure is acceptable
- [ ] Implement `Lazy::force_ref_or_panic(&self)` convenience method
- [ ] Calls `force` and unwraps with expect
- [ ] Does NOT require `A: Clone` (returns `&A`)
- [ ] For use when panic on failure is acceptable and cloning is not desired
- [ ] Implement `Lazy::try_get_ref(&self)` method
- [ ] Returns `Option<&A>` without forcing (None if not forced or if poisoned)
- [ ] Must handle `Result<A, Arc<LazyError>>` stored in cell
- [ ] Implement `Lazy::try_get(&self)` method
- [ ] Returns `Option<A>` (cloned) without forcing
- [ ] Implement `Lazy::is_forced(&self)` method
- [ ] Returns `bool` indicating if value computed successfully (not poisoned)
- [ ] Implement `Lazy::is_poisoned(&self)` method
- [ ] Returns `bool` indicating if thunk panicked during evaluation
- [ ] Implement `Lazy::get_error(&self)` method
- [ ] Returns `Option<LazyError>` - clones the stored `Arc<LazyError>` if poisoned
- [ ] Document: "Returns cloned error (via Arc::clone + LazyError::clone)"
- [ ] Document performance: "Lightweight clone - Arc increment + Arc<str> increment"
- [ ] Provides access to original panic message for debugging
- [ ] Implement `Lazy::try_into_result(self) -> Result<Result<A, LazyError>, Self>` method
- [ ] Checks if initialized before attempting `try_unwrap` to avoid re-allocation
- [ ] Uses `RefCountedPointer::try_unwrap` to check for sole ownership
- [ ] Attempts to extract owned value if sole reference (strong_count == 1)
- [ ] Returns `Ok(Ok(value))` if unique and forced successfully
- [ ] Returns `Ok(Err(LazyError))` if unique and poisoned
- [ ] Returns `Err(self)` if shared or not yet forced
- [ ] Does NOT require `A: Clone` (uses `Once::into_inner`)
- [ ] Add documentation with type signatures

### 3.3 Clone and Debug Implementation

- [ ] Implement `Clone` for `Lazy<'a, Config, A>` where `Config: LazyConfig`
- [ ] Clone must be cheap (just reference count increment)
- [ ] Verify clones share memoization state (via test)
- [ ] Implement `Debug` for `Lazy<'a, Config, A>` where `Config: LazyConfig, A: Debug`:
- [ ] Shows "Lazy::Unforced" before forcing
- [ ] Shows "Lazy::Forced(value)" after forcing with value's Debug output
- [ ] Shows "Lazy::Poisoned" if thunk panicked

### 3.4 TrySemigroup, TryMonoid, and Type Class Implementations

- [ ] Create `fp-library/src/classes/try_semigroup.rs` with `TrySemigroup` trait
- [ ] Document in trait-level docs that `try_combine` may always return `Ok`:
	```rust
	/// ### Note: Some Implementations Always Return Ok
	///
	/// For lazy types like `RcLazy` and `ArcLazy`, `try_combine` ALWAYS returns
	/// `Ok(lazy)` at the call site. Errors only surface when the resulting lazy
	/// is forced. This preserves lazy evaluation semantics but may surprise users
	/// expecting immediate validation.
	///
	/// If you need to fail early on already-poisoned operands, check
	/// `Lazy::is_poisoned()` before combining.
	```
- [ ] Create `fp-library/src/classes/try_monoid.rs` with `TryMonoid` trait
- [ ] Implement `TrySemigroup` for `RcLazy` (separate impl, not generic):
- [ ] Uses `ClonableFn::new` for thunk (no thread-safety requirement)
- [ ] `try_combine` returns `Ok(NEW lazy)` that defers forcing until demanded
- [ ] Document: "Always returns Ok - errors deferred until forcing"
- [ ] Requires `A: Semigroup + Clone` only (no `Send + Sync`)
- [ ] Type Error = LazyError
- [ ] Implement `TrySemigroup` for `ArcLazy` (separate impl, not generic):
- [ ] Uses `SendClonableFn::send_clonable_fn_new` for thread-safe thunk
- [ ] `try_combine` returns `Ok(NEW lazy)` that defers forcing until demanded
- [ ] Document: "Always returns Ok - errors deferred until forcing"
- [ ] Requires `A: Semigroup + Clone + Send + Sync`
- [ ] Type Error = LazyError
- [ ] Implement `TryMonoid` for `RcLazy`:
- [ ] Uses `ClonableFn::new` for thunk
- [ ] Requires `A: Monoid + Clone`
- [ ] Implement `TryMonoid` for `ArcLazy`:
- [ ] Uses `SendClonableFn::send_clonable_fn_new` for thread-safe thunk
- [ ] Requires `A: Monoid + Clone + Send + Sync`
- [ ] Note: Separate impls needed because generic impl would use wrong thunk type for ArcLazy
- [ ] **DO NOT** implement `Semigroup` for `Lazy`:
- [ ] Rationale: Semigroup laws require total functions
- [ ] A panicking `combine` violates algebraic laws users depend on
- [ ] Document: use `TrySemigroup::try_combine` for safe composition
- [ ] **DO NOT** implement `Monoid` for `Lazy`:
- [ ] Same rationale as Semigroup
- [ ] `try_empty` from TryMonoid is always safe
- [ ] Implement `Defer` for `LazyBrand<RcLazyConfig>` ONLY (not for ArcLazy):
- [ ] Uses `ClonableFn::new` for thunk (no thread-safety requirement)
- [ ] `defer(f)` creates `Lazy` that calls f, then forces result using `force_or_panic`
- [ ] Requires `A: Clone` only (no `Send + Sync`)
- [ ] **DO NOT** implement `Defer` for `LazyBrand<ArcLazyConfig>`:
- [ ] Rationale: Would allow non-thread-safe ArcLazy creation
- [ ] `Defer::defer` takes non-Send+Sync thunk, breaking ArcLazy's thread-safety guarantee
- [ ] Users must use `SendDefer::send_defer` instead for ArcLazy
- [ ] Implement `SendDefer` for `LazyBrand<ArcLazyConfig>` only:
- [ ] Uses `SendClonableFn::send_clonable_fn_new` for thread-safe thunk
- [ ] `send_defer(f)` takes `f: impl Fn() -> Lazy + Send + Sync`
- [ ] Requires `A: Clone + Send + Sync`
- [ ] Note: Separate impls - RcLazy has Defer, ArcLazy has SendDefer (mutually exclusive)
- [ ] Note: `SendDefer` does NOT extend `Defer` (independent traits - see Phase 1.1)

### 3.5 Kind Implementation

- [ ] Update `LazyBrand<Config: LazyConfig>` in `fp-library/src/brands.rs` (1 parameter)
- [ ] Note: LazyBrand now takes single Config parameter instead of 3 separate parameters
- [ ] Update `impl_kind!` for `LazyBrand`
- [ ] Verify kind signature matches expected HKT pattern

### 3.6 Free Functions

- [ ] Update `lazy_new` free function (or rename)
- [ ] Update `lazy_force` free function
- [ ] Add `lazy_force_cloned`, `lazy_force_or_panic`, and `lazy_force_ref_or_panic` free functions
- [ ] Add `lazy_is_forced`, `lazy_is_poisoned`, and `lazy_get_error` free functions
- [ ] Add `lazy_try_get`, `lazy_try_get_ref`, and `lazy_try_into_result` free functions
- [ ] Re-export in `fp-library/src/functions.rs`

### 3.7 Type Aliases

- [ ] Add `pub type RcLazy<'a, A> = Lazy<'a, RcLazyConfig, A>;`
- [ ] Add `pub type ArcLazy<'a, A> = Lazy<'a, ArcLazyConfig, A>;`
- [ ] Note: Type aliases now use Config structs (2 type params instead of 4)
- [ ] Document thread-safety characteristics of each alias

### 3.8 Phase 3 Tests

- [ ] Unit test: `Lazy::new` + `Lazy::force_cloned` returns `Ok(value)`
- [ ] Unit test: `Lazy::force` returns `Ok(&value)`
- [ ] Unit test: `Lazy::force_ref_or_panic` returns `&value` without Clone bound
- [ ] **Critical test**: Thunk is only called once across all clones
- [ ] **Critical test**: Clones share memoization (counter test)
- [ ] **Critical test**: Thunk is cleared after forcing (weak ref test)
- [ ] Unit test: `Lazy::is_forced` returns correct state
- [ ] Unit test: `Lazy::is_poisoned` returns `false` before forcing, `true` after panic
- [ ] Unit test: `Lazy::try_get_ref` returns `None` before forcing
- [ ] Unit test: `Lazy::try_get_ref` returns `Some(&value)` after successful forcing
- [ ] Unit test: `Lazy::try_get_ref` returns `None` after panic (poisoned state)
- [ ] **Panic safety test**: `force` returns `Err(LazyError)` when thunk panics
- [ ] **Panic safety test**: All clones see `Err(LazyError)` after panic (shared poisoned state)
- [ ] Unit test: `force_or_panic` panics with appropriate message on error
- [ ] Unit test: `force_ref_or_panic` panics with appropriate message on error
- [ ] Unit test: `TrySemigroup::try_combine` returns `Ok(lazy)` (always succeeds at call site)
- [ ] Unit test: `TrySemigroup::try_combine` result's force returns `Err` when inner thunk fails
- [ ] Unit test: `TrySemigroup::try_combine` defers forcing (truly lazy semantics)
- [ ] Unit test: `TryMonoid::try_empty` works correctly
- [ ] Compile-fail test: `Lazy` does NOT implement `Semigroup`
- [ ] Compile-fail test: `Lazy` does NOT implement `Monoid`
- [ ] Unit test: `Debug` shows "Lazy::Unforced" before forcing
- [ ] Unit test: `Debug` shows "Lazy::Forced(value)" after forcing
- [ ] Unit test: `Debug` shows "Lazy::Poisoned" after panic
- [ ] Unit test: `Defer::defer` works correctly for RcLazy
- [ ] Compile-fail test: `LazyBrand<RcLazyConfig>` does NOT implement `SendDefer`
- [ ] Unit test: `SendDefer::send_defer` works correctly for ArcLazy
- [ ] Compile-fail test: `LazyBrand<ArcLazyConfig>` does NOT implement `Defer`
- [ ] Property test: TrySemigroup associativity law (when all succeed)
- [ ] Property test: TryMonoid identity laws (when all succeed)
- [ ] Compile-fail test: `RcLazy` is `!Send`
- [ ] Compile-success test: `ArcLazy<A: Send + Sync>` is `Send + Sync`
- [ ] Compile-fail test: Custom `Lazy<MyInvalidConfig, _>` fails when `MyInvalidConfig` doesn't impl `LazyConfig`
- [ ] Compile-fail test: `ArcLazy::new` with non-`Send` closure fails
- [ ] Compile-fail test: `ArcLazy::new` with non-`Sync` closure fails

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
- [ ] Update `fp-library/src/brands.rs` with all new brands:
- [ ] RcBrand, ArcBrand, BoxBrand
- [ ] RcLazyConfig, ArcLazyConfig
- [ ] Update `fp-library/src/classes.rs` with `pointer` export
- [ ] Update `fp-library/src/classes.rs` with `send_defer` export
- [ ] Verify no circular dependencies

### 4.3 Documentation Files

- [ ] Update `docs/std-coverage-checklist.md` with:
- [ ] `Pointer` entry in Type Classes table
- [ ] `RefCountedPointer` entry in Type Classes table
- [ ] `SendRefCountedPointer` entry in Type Classes table
- [ ] `ThunkWrapper` entry in Type Classes table
- [ ] `LazyConfig` entry in Type Classes table
- [ ] `SendLazyConfig` entry in Type Classes table
- [ ] `RcBrand` entry in Data Types table
- [ ] `ArcBrand` entry in Data Types table
- [ ] `RcLazyConfig` entry in Data Types table
- [ ] `ArcLazyConfig` entry in Data Types table
- [ ] Update `LazyBrand` entry to reflect new semantics
- [ ] Update `docs/architecture.md`:
- [ ] Document the Pointer → RefCountedPointer → SendRefCountedPointer pattern
- [ ] Document the Configuration Struct Pattern for LazyConfig
- [ ] Document the blanket impl approach for FnBrand via UnsizedCoercible
- [ ] Document `ThunkWrapper` pattern
- [ ] Document independent Defer/SendDefer traits (not a hierarchy)
- [ ] Add "Extending Lazy for Custom Configurations" documentation:
- [ ] Add section to `fp-library/src/types/lazy.rs` module docs
- [ ] Include complete working example with custom `MyLazyConfig`
- [ ] Document `SendLazyConfig` implementation for thread-safe variants
- [ ] Explain Configuration Struct Pattern benefits over impl-for-()
- [ ] Update `docs/todo.md` to mark Lazy memoization item as addressed
- [ ] Update `docs/limitations.md`:
- [ ] Document FnBrand extensibility via UnsizedCoercible trait
- [ ] Document thunk cleanup differences between RcBrand (RefCell) and ArcBrand (Mutex)
- [ ] Document Known Limitations from plan.md:
	- [ ] LazyError loses original panic payload type (required for thread safety)
	- [ ] `Lazy::force_cloned` requires `A: Clone` (inherent to shared memoization)
	- [ ] `Lazy::force_or_panic` requires `A: Clone` (use `force_ref_or_panic` to avoid)
	- [ ] Recursive lazy evaluation deadlocks for ArcLazy, panics for RcLazy
	- [ ] AssertUnwindSafe usage and safety invariants

### 4.4 Final Verification

- [ ] `cargo test` passes (all tests)
- [ ] `cargo clippy` passes (no warnings)
- [ ] `cargo doc` builds without warnings
- [ ] Review generated documentation for clarity
- [ ] Run benchmarks to verify no performance regression

---

## Phase 5: Concurrency Testing with Loom

### 5.1 Setup

- [ ] Add `loom = "0.7"` to dev-dependencies in `fp-library/Cargo.toml`
- [ ] Create `fp-library/tests/loom_tests.rs`

### 5.2 Loom Tests for ArcLazy

- [ ] Test: Concurrent force from multiple threads - thunk called exactly once
- [ ] Test: All threads see same memoized value
- [ ] Test: Panic propagation across threads - all threads see `Err(LazyError)`
- [ ] Test: No deadlocks with Mutex in ThunkWrapper

### 5.3 Running Loom Tests

- [ ] Add CI command: `RUSTFLAGS="--cfg loom" cargo test --test loom_tests`
- [ ] Document loom testing in README or CONTRIBUTING

---

## Release Preparation

### Changelog

- [ ] Add entry to `fp-library/CHANGELOG.md` under `[Unreleased]`:

```markdown
### Changed (Breaking)

- `Lazy` now uses shared memoization semantics (Haskell-like)
	- Clones share memoization state
	- `force` and `force_cloned` take `&self` and return `Result<_, LazyError>`
	- `Lazy` has 2 type parameters: Config (implementing LazyConfig), A
	- Thunks cleared after forcing to free captured values
	- OnceCell stores `Result<A, Arc<LazyError>>` for panic-safe evaluation
- `RcFnBrand` is now a type alias for `FnBrand<RcBrand>`
- `ArcFnBrand` is now a type alias for `FnBrand<ArcBrand>`
- `LazyBrand` now takes 1 type parameter (Config) instead of 3
- `SendDefer` is now independent from `Defer` (not a subtrait)
	- RcLazy implements Defer only
	- ArcLazy implements SendDefer only

### Added

- `Pointer` base trait for heap-allocated pointers
- `RefCountedPointer` extension trait with `CloneableOf` for shared ownership
- `SendRefCountedPointer` extension trait with `SendOf` for thread-safe pointers
- `ThunkWrapper` trait for pointer-brand-specific thunk storage
- `LazyConfig` configuration trait for valid Lazy configurations
- `SendLazyConfig` extension trait for thread-safe configurations
- `RcLazyConfig` and `ArcLazyConfig` configuration structs
- `LazyError` struct with panic payload for debugging
- `UnsizedCoercible` trait for basic function coercion
- `SendUnsizedCoercible` trait for thread-safe function coercion
- `TrySemigroup` trait for fallible semigroup operations
- `TryMonoid` trait for fallible monoid operations
- `SendDefer` trait for thread-safe deferred evaluation (ArcLazy only)
- `RcBrand` and `ArcBrand` implementing the pointer hierarchy
- `BoxBrand` placeholder for future unique ownership support
- `FnBrand<PtrBrand>` generic function brand
- `RcLazy` and `ArcLazy` type aliases for common configurations
- `Lazy::force(&self) -> Result<&A, LazyError>` method (avoids cloning)
- `Lazy::force_cloned(&self) -> Result<A, LazyError>` method
- `Lazy::force_or_panic(&self) -> A` convenience method (requires Clone)
- `Lazy::force_ref_or_panic(&self) -> &A` convenience method (no Clone required)
- `Lazy::try_get_ref(&self) -> Option<&A>` method
- `Lazy::is_forced(&self) -> bool` method
- `Lazy::is_poisoned(&self) -> bool` method
- `Lazy::get_error(&self) -> Option<LazyError>` method

### Removed

- Old value-semantic `Lazy` implementation
- Separate `rc_fn.rs` and `arc_fn.rs` files (merged into `fn_brand.rs`)
- `ValidLazyCombination` trait (replaced by `LazyConfig` configuration structs)
```

---

## Implementation Notes

### Key Design Decisions

1. **Three-level trait hierarchy**: `Pointer` → `RefCountedPointer` → `SendRefCountedPointer` allows future `BoxBrand` support at the `Pointer` level without reference counting

2. **Additional Associated Type pattern**: `RefCountedPointer` adds `CloneableOf` with `Clone` bound; `SendRefCountedPointer` adds `SendOf` with `Send + Sync` bounds rather than using invalid `for<T: Trait>` syntax (which Rust doesn't support)

3. **Blanket impl via UnsizedCoercible**: `FnBrand<P>` implements `ClonableFn` for any `P: UnsizedCoercible`, enabling third-party extension

4. **Configuration Struct Pattern for Lazy**: `Lazy<Config, A>` uses a single `Config: LazyConfig` parameter that bundles PtrBrand/OnceBrand/FnBrand. This solves the orphan rule issue that blocked third-party extension with the `impl for ()` pattern.

5. **Type aliases for ergonomics**: `RcFnBrand`, `ArcFnBrand`, `RcLazy`, `ArcLazy` provide good defaults

6. **LazyConfig trait with ThunkOf**: Bundles all configuration into a single trait. The `ThunkOf` associated type ensures `ArcLazy` uses `SendClonableFn::SendOf` (with `Send + Sync` bounds) while `RcLazy` uses `ClonableFn::Of`.

7. **ThunkWrapper trait**: Abstracts over `RefCell<Option<Thunk>>` (for Rc) and `parking_lot::Mutex<Option<Thunk>>` (for Arc) to enable thunk cleanup after forcing. ⚠️ Note: Recursive forcing **will deadlock** for ArcLazy (at `OnceLock::get_or_init`) and **panic** for RcLazy (at `OnceCell::get_or_init`). This is documented as a Known Limitation.

8. **Panic-safe evaluation with stable Rust**: `force` returns `Result<&A, LazyError>` and uses stable `get_or_init` (NOT nightly-only `get_or_try_init`). The OnceCell stores `Result<A, Arc<LazyError>>` to capture both success and error states.

9. **LazyError with Arc<str>**: Stores the panic message as `Arc<str>` for thread-safe sharing. Using raw `Box<dyn Any + Send>` would make `LazyError` `!Sync`, breaking `ArcLazy` thread safety. The tradeoff is losing the ability to re-panic with the original payload, but thread safety is essential.

10. **Arc<LazyError> in OnceCell**: The cell stores `Result<A, Arc<LazyError>>` so all clones see the same error with the same message. Without `Arc`, secondary callers would get `LazyError::poisoned()` with no message.

11. **Quadruple force methods**: `force(&self) -> Result<&A, LazyError>` for explicit error handling; `force_cloned(&self) -> Result<A, LazyError>` clones; `force_or_panic(&self) -> A` for convenience (requires Clone); `force_ref_or_panic(&self) -> &A` for convenience without Clone requirement

12. **Clone bound limitation**: `force_cloned` and `force_or_panic` require `A: Clone` due to shared memoization semantics. Users needing to avoid cloning should use `force`, `force_ref_or_panic`, wrap values in `Rc`/`Arc`, or use `try_into_result` for unique ownership.

13. **AssertUnwindSafe invariant**: The `catch_unwind` in `force` uses `AssertUnwindSafe` safely because:

	- Thunk is taken (moved out) atomically before execution begins
	- If panic occurs, thunk is already consumed (no partial state)
	- Result stored in OnceCell captures panic state
	- No mutable references to shared state exist during thunk execution
	- Single-writer guarantee from OnceCell::get_or_init
	- The key insight: AssertUnwindSafe asserts that catching a panic won't violate memory safety, and here the thunk has no access to LazyInner during execution

14. **UnsizedCoercible/SendUnsizedCoercible traits**: Two-level hierarchy following ClonableFn → SendClonableFn pattern. `UnsizedCoercible` provides basic function coercion, `SendUnsizedCoercible` adds thread-safe coercion. RcBrand only implements `UnsizedCoercible` (no panicking methods).

15. **TrySemigroup/TryMonoid only (no Semigroup/Monoid)**: Lazy does NOT implement Semigroup or Monoid because those traits require total functions. A panicking `combine` violates algebraic laws. Users must use `TrySemigroup::try_combine` which makes fallibility explicit. Note: `try_combine` always returns `Ok` for Lazy - errors are deferred until forcing.

16. **Lazy TrySemigroup is truly lazy**: `try_combine` returns a NEW lazy that defers forcing until demanded. This preserves lazy semantics and allows building lazy computations without immediate failure. The tradeoff is that `try_combine` never fails at call site - use `is_poisoned()` if early validation is needed.

17. **Separate TrySemigroup impls for RcLazy and ArcLazy**: A generic impl would use `ClonableFn::Of` for the thunk, but `ArcLazy` requires `SendClonableFn::SendOf` with `Send + Sync` bounds. Separate impls ensure correct thunk type for each.

18. **Debug implementation**: Shows `Lazy::Unforced`, `Lazy::Forced(value)`, or `Lazy::Poisoned` depending on current state, requires `A: Debug`.

19. **try_into_result method**: Allows extracting owned value from unique Lazy reference without cloning. Returns `Err(self)` if shared or not yet forced.

20. **Configuration Struct Pattern**: Uses concrete structs (`RcLazyConfig`, `ArcLazyConfig`) implementing `LazyConfig` trait instead of `impl Trait<A,B,C> for ()`. This enables third-party extension without orphan rule violations - third parties can define their own config structs.

21. **Independent Defer/SendDefer traits**: Unlike other trait pairs (ClonableFn → SendClonableFn), `SendDefer` does NOT extend `Defer`. This prevents accidentally creating non-thread-safe `ArcLazy` values. RcLazy implements only `Defer`; ArcLazy implements only `SendDefer`.

22. **Loom concurrency testing**: Exhaustive testing of all thread interleavings for `ArcLazy` to verify correct synchronization.

### Files Summary

| Action  | File                                                                      |
| ------- | ------------------------------------------------------------------------- |
| Create  | `fp-library/src/classes/pointer.rs`                                       |
| Create  | `fp-library/src/classes/send_defer.rs`                                    |
| Create  | `fp-library/src/classes/try_semigroup.rs`                                 |
| Create  | `fp-library/src/classes/try_monoid.rs`                                    |
| Create  | `fp-library/src/types/rc_ptr.rs`                                          |
| Create  | `fp-library/src/types/arc_ptr.rs`                                         |
| Create  | `fp-library/src/types/fn_brand.rs`                                        |
| Create  | `fp-library/tests/loom_tests.rs`                                          |
| Delete  | `fp-library/src/types/rc_fn.rs`                                           |
| Delete  | `fp-library/src/types/arc_fn.rs`                                          |
| Rewrite | `fp-library/src/types/lazy.rs`                                            |
| Modify  | `fp-library/src/brands.rs`                                                |
| Modify  | `fp-library/src/classes.rs`                                               |
| Modify  | `fp-library/src/types.rs`                                                 |
| Modify  | `fp-library/src/functions.rs`                                             |
| Modify  | `fp-library/Cargo.toml` (add loom dev-dependency, parking_lot dependency) |

### Testing Strategy

1. **Unit tests**: In each module's `#[cfg(test)]` block
2. **Property tests**: In `fp-library/tests/property_tests.rs`
3. **Compile-fail tests**: In `fp-library/tests/ui/` directory
4. **Doc tests**: In documentation examples

### Critical Test Cases

The most important tests are:

1. **Shared memoization test** (must pass):

```rust
use fp_library::{brands::*, classes::*, functions::*};
let counter = Rc::new(Cell::new(0));
let counter_clone = counter.clone();

let lazy: RcLazy<i32> = lazy_new::<RcLazyConfig, _>(
	clonable_fn_new::<RcFnBrand, _, _>(move |_| {
		counter_clone.set(counter_clone.get() + 1);
		42
	})
);
let lazy2 = lazy.clone();

assert_eq!(lazy_force_cloned::<RcLazyConfig, _>(&lazy), Ok(42));
assert_eq!(counter.get(), 1);  // Called once
assert_eq!(lazy_force_cloned::<RcLazyConfig, _>(&lazy2), Ok(42));
assert_eq!(counter.get(), 1);  // Still 1 - shared!
```

2. **Panic safety test with shared error** (must pass):

```rust
use fp_library::{brands::*, classes::*, functions::*};
let lazy: RcLazy<i32> = lazy_new::<RcLazyConfig, _>(
	clonable_fn_new::<RcFnBrand, _, _>(|_| -> i32 { panic!("computation failed") })
);
let lazy2 = lazy.clone();

let err = lazy_force::<RcLazyConfig, _>(&lazy).unwrap_err();
// First caller gets the original panic message
assert_eq!(err.panic_message(), Some("computation failed"));

// Second call on any clone ALSO sees the same message (via Arc<LazyError>)
let err2 = lazy_force::<RcLazyConfig, _>(&lazy2).unwrap_err();
assert_eq!(err2.panic_message(), Some("computation failed"));

// Both errors are clones of the same Arc<LazyError>
assert_eq!(err.panic_message(), err2.panic_message());
```

3. **Thread safety test** (for ArcLazy):

```rust
use fp_library::{brands::*, classes::*, functions::*};
let lazy: ArcLazy<i32> = lazy_new::<ArcLazyConfig, _>(
	send_clonable_fn_new::<ArcFnBrand, _, _>(|_| 42)
);
std::thread::spawn(move || lazy_force_cloned::<ArcLazyConfig, _>(&lazy).unwrap()).join().unwrap();
```

4. **Compile-fail for RcLazy Send**:

```rust
let lazy: RcLazy<i32> = lazy_new::<RcLazyConfig, _>(/* ... */);
std::thread::spawn(move || lazy_force_cloned::<RcLazyConfig, _>(&lazy)); // Should fail!
```

5. **force_ref_or_panic without Clone** (must pass):

```rust
struct NonClone(i32);  // Does not implement Clone

use fp_library::{brands::*, classes::*, functions::*};
let lazy: RcLazy<NonClone> = lazy_new::<RcLazyConfig, _>(
	clonable_fn_new::<RcFnBrand, _, _>(|_| NonClone(42))
);

// This works - no Clone required
let value_ref: &NonClone = lazy_force_ref_or_panic::<RcLazyConfig, _>(&lazy);
assert_eq!(value_ref.0, 42);

// This would NOT compile - Clone required
// let value: NonClone = lazy_force_or_panic::<RcLazyConfig, _>(&lazy);
```

6. **ArcLazy does not implement Defer** (compile-fail):

```rust
// This should fail to compile - ArcLazy only implements SendDefer
use fp_library::classes::*;

fn use_defer<B: Defer>() {}
use_defer::<LazyBrand<ArcLazyConfig>>(); // Should fail!
```

6. **TrySemigroup::try_combine always returns Ok** (must pass):

```rust
// Even with poisoned operands, try_combine returns Ok
use fp_library::{brands::*, classes::*, functions::*};
let poisoned: RcLazy<i32> = lazy_new::<RcLazyConfig, _>(
	clonable_fn_new::<RcFnBrand, _, _>(|_| panic!("oops"))
);
let _ = lazy_force::<RcLazyConfig, _>(&poisoned); // Force to poison it

let normal: RcLazy<i32> = lazy_new::<RcLazyConfig, _>(
	clonable_fn_new::<RcFnBrand, _, _>(|_| 42)
);

// try_combine succeeds - error deferred until forcing
let combined = try_combine(poisoned, normal);
assert!(combined.is_ok()); // Always Ok at call site!

// Error surfaces when forcing
let result = lazy_force::<RcLazyConfig, _>(&combined.unwrap());
assert!(result.is_err()); // Error here, not at combine time
```
