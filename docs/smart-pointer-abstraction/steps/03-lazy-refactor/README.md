# Step 3: Lazy Refactor

This step implements the new `Lazy` type with Haskell-like shared memoization semantics, replacing the current value-semantic implementation.

## Goals

1.  Rewrite `fp-library/src/types/lazy.rs` to use shared semantics.
2.  Implement `LazyConfig` pattern for valid pointer/once-cell/function combinations.
3.  Implement `LazyError` for thread-safe panic propagation.
4.  Create `TrySemigroup` and `TryMonoid` traits for fallible combination.
5.  Implement `SendDefer` for thread-safe deferred evaluation.

## Technical Design

### Lazy Type with Shared Memoization

The new `Lazy` type replaces the current value-semantic implementation with Haskell-like shared memoization.

#### LazyConfig Trait

To prevent invalid `PtrBrand`/`OnceBrand`/`FnBrand` combinations at compile time and to specify the correct thunk type for thread safety, we use a **Configuration Struct Pattern**.

```rust
pub trait LazyConfig {
	/// The pointer brand for shared ownership (e.g., RcBrand, ArcBrand).
	type PtrBrand: RefCountedPointer + ThunkWrapper;
	/// The once-cell brand for memoization (e.g., OnceCellBrand, OnceLockBrand).
	type OnceBrand: Once;
	/// The function brand for thunk storage (e.g., RcFnBrand, ArcFnBrand).
	type FnBrand: ClonableFn;
	/// The thunk type to use for this configuration.
	type ThunkOf<'a, A>: Clone;
}

pub trait SendLazyConfig: LazyConfig {
	/// The thread-safe thunk type. Same as ThunkOf but guaranteed Send + Sync.
	type SendThunkOf<'a, A: Send + Sync>: Clone + Send + Sync;
}

pub struct RcLazyConfig;
impl LazyConfig for RcLazyConfig {
	type PtrBrand = RcBrand;
	type OnceBrand = OnceCellBrand;
	type FnBrand = RcFnBrand;
	type ThunkOf<'a, A> = <RcFnBrand as ClonableFn>::Of<'a, (), A>;
}

pub struct ArcLazyConfig;
impl LazyConfig for ArcLazyConfig {
	type PtrBrand = ArcBrand;
	type OnceBrand = OnceLockBrand;
	type FnBrand = ArcFnBrand;
	type ThunkOf<'a, A> = <ArcFnBrand as SendClonableFn>::SendOf<'a, (), A>;
}
impl SendLazyConfig for ArcLazyConfig {
	type SendThunkOf<'a, A: Send + Sync> = <ArcFnBrand as SendClonableFn>::SendOf<'a, (), A>;
}
```

#### Thunk Wrapper

To avoid retaining thunks (and their captured values) after forcing, the thunk is wrapped in `Option` and cleared after evaluation. The wrapper type varies by pointer brand.

```rust
pub type ThunkCell<PtrBrand, Thunk> = <PtrBrand as ThunkWrapper>::Cell<Thunk>;

pub trait ThunkWrapper {
	type Cell<T>;
	fn new_cell<T>(value: Option<T>) -> Self::Cell<T>;
	fn take<T>(cell: &Self::Cell<T>) -> Option<T>;
}

impl ThunkWrapper for RcBrand {
	type Cell<T> = std::cell::RefCell<Option<T>>;
	fn new_cell<T>(value: Option<T>) -> Self::Cell<T> { RefCell::new(value) }
	fn take<T>(cell: &Self::Cell<T>) -> Option<T> { cell.borrow_mut().take() }
}

impl ThunkWrapper for ArcBrand {
	// Requires parking_lot dependency
	type Cell<T> = parking_lot::Mutex<Option<T>>;
	fn new_cell<T>(value: Option<T>) -> Self::Cell<T> { parking_lot::Mutex::new(value) }
	fn take<T>(cell: &Self::Cell<T>) -> Option<T> { cell.lock().take() }
}
```

#### Core Structure

```rust
// fp-library/src/types/lazy.rs

struct LazyInner<'a, Config: LazyConfig, A> {
	/// The memoized result (computed at most once).
	/// Stores Result<A, Arc<LazyError>> to capture both successful values and errors.
	once: <Config::OnceBrand as Once>::Of<Result<A, Arc<LazyError>>>,
	/// The thunk, wrapped in ThunkWrapper::Cell for interior mutability.
	thunk: <Config::PtrBrand as ThunkWrapper>::Cell<Config::ThunkOf<'a, A>>,
}

pub struct Lazy<'a, Config: LazyConfig, A>(
	// CloneableOf wraps LazyInner for shared ownership
	<Config::PtrBrand as RefCountedPointer>::CloneableOf<LazyInner<'a, Config, A>>,
);
```

#### LazyError

```rust
#[derive(Debug, Clone, Error)]
#[error("thunk panicked during evaluation{}", .0.as_ref().map(|m| format!(": {}", m)).unwrap_or_default())]
pub struct LazyError(Option<Arc<str>>);

impl LazyError {
	pub fn from_panic(payload: Box<dyn std::any::Any + Send + 'static>) -> Self {
		// Implementation extracts string message into Arc<str>
	}
}
```

#### Implementation Details

The `force` method uses `get_or_init` (stable) instead of `get_or_try_init` (nightly) by storing `Result` in the cell.

```rust
impl<'a, Config: LazyConfig, A> Lazy<'a, Config, A> {
	pub fn force(this: &Self) -> Result<&A, LazyError> {
		let inner = &*this.0;
		let result: &Result<A, Arc<LazyError>> = <Config::OnceBrand as Once>::get_or_init(&inner.once, || {
			let thunk = Config::PtrBrand::take(&inner.thunk)
				.expect("unreachable: get_or_init guarantees single execution");
			std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| thunk(())))
				.map_err(|payload| Arc::new(LazyError::from_panic(payload)))
		});
		result.as_ref().map_err(|e| (**e).clone())
	}

	pub fn force_cloned(this: &Self) -> Result<A, LazyError> where A: Clone {
		Self::force(this).map(Clone::clone)
	}
}
```

### Type Class Implementations

#### TrySemigroup and TryMonoid

To enable safe composition of lazy values without hidden panics, we introduce `TrySemigroup` and `TryMonoid` traits that return `Result`.

```rust
// fp-library/src/classes/try_semigroup.rs
pub trait TrySemigroup: Sized {
	type Error;
	fn try_combine(x: Self, y: Self) -> Result<Self, Self::Error>;
}

// fp-library/src/classes/try_monoid.rs
pub trait TryMonoid: TrySemigroup {
	fn try_empty() -> Self;
}
```

#### SendDefer

Trait for deferred lazy evaluation with thread-safe thunks.

```rust
// fp-library/src/classes/send_defer.rs
pub trait SendDefer: Kind {
	fn send_defer<'a, A>(thunk: impl 'a + Fn() -> Self::Of<'a, A> + Send + Sync) -> Self::Of<'a, A>
	where
		A: Clone + Send + Sync + 'a;
}
```

## Checklist

- [ ] Rewrite `fp-library/src/types/lazy.rs`
   - Change to shared semantics using `RefCountedPointer::CloneableOf`
   - Use Configuration Struct Pattern with `LazyConfig` trait
   - Define `RcLazyConfig` and `ArcLazyConfig` configuration structs
   - Define `SendLazyConfig` extension trait
   - Store `Result<A, LazyError>` in OnceCell
   - Add `LazyError` struct with `Arc<str>`
   - Change `force` to return `Result<&A, LazyError>`
   - Add `force_or_panic` and `force_ref_or_panic` convenience methods
   - Add `is_poisoned` and `get_error` methods
   - Add `Debug` implementation
   - Use `LazyConfig::ThunkOf` for thunk type selection
- [ ] Add `RcLazy` and `ArcLazy` type aliases
- [ ] Create `fp-library/src/classes/try_semigroup.rs` with `TrySemigroup` trait
- [ ] Create `fp-library/src/classes/try_monoid.rs` with `TryMonoid` trait
- [ ] Create `fp-library/src/classes/send_defer.rs` with `SendDefer` trait
- [ ] Implement `TrySemigroup`, `TryMonoid`, `Defer` for `RcLazy`
- [ ] Implement `TrySemigroup`, `TryMonoid`, `SendDefer` for `ArcLazy`
- [ ] Update `LazyBrand` to take 1 config parameter: `LazyBrand<Config: LazyConfig>`
- [ ] Update `impl_kind!` for new `LazyBrand`
- [ ] Update all tests to handle `Result` return type from `force` and `force_cloned`
