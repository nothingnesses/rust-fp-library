//! Lazy value wrapper.
//!
//! This module defines the [`Lazy`] struct, which represents a lazily-computed, memoized value.
//! It implements [`Semigroup`], [`Monoid`], and [`Defer`].
//!
//! ## Configurations
//!
//! - [`RcLazyConfig`] / [`RcLazy`]: Single-threaded lazy values using [`Rc`](std::rc::Rc). Not thread-safe.
//! - [`ArcLazyConfig`] / [`ArcLazy`]: Thread-safe lazy values using [`Arc`]. Requires `A: Send + Sync`.

use crate::{
	brands::{ArcBrand, ArcFnBrand, LazyBrand, OnceCellBrand, OnceLockBrand, RcBrand, RcFnBrand},
	classes::{
		cloneable_fn::CloneableFn, defer::Defer, monoid::Monoid, once::Once,
		ref_counted_pointer::RefCountedPointer, semigroup::Semigroup,
		send_cloneable_fn::SendCloneableFn, thunk_wrapper::ThunkWrapper,
	},
	impl_kind,
	kinds::*,
};
use std::{
	fmt::{self, Debug, Formatter},
	ops::Deref,
	sync::Arc,
};
use thiserror::Error;

/// Configuration trait for `Lazy` types.
///
/// This trait defines the types used for pointer storage, memoization, and thunk execution.
/// It ensures that compatible types are used together (e.g., `Rc` with `OnceCell`, `Arc` with `OnceLock`).
pub trait LazyConfig: Sized + 'static {
	/// The pointer brand for shared ownership (e.g., `RcBrand`, `ArcBrand`).
	type PtrBrand: RefCountedPointer + ThunkWrapper;
	/// The once-cell brand for memoization (e.g., `OnceCellBrand`, `OnceLockBrand`).
	type OnceBrand: Once;
	/// The function brand for thunk storage (e.g., `RcFnBrand`, `ArcFnBrand`).
	type FnBrand: CloneableFn;
	/// The thunk type to use for this configuration.
	/// Thunks deref to `Fn(()) -> A` to match the cloneable function wrapper.
	type ThunkOf<'a, A>: Clone + Deref<Target: Fn(()) -> A>
	where
		A: 'a;
}

/// Trait for `Lazy` configurations that support semigroup operations.
pub trait LazySemigroup<A>: LazyConfig {
	/// Combines two lazy values.
	///
	/// This method combines two lazy values into a new lazy value.
	///
	/// ### Type Signature
	///
	/// `forall config a. Semigroup a => (Lazy config a, Lazy config a) -> Lazy config a`
	///
	/// ### Parameters
	///
	/// * `x`: The first lazy value.
	/// * `y`: The second lazy value.
	///
	/// ### Returns
	///
	/// A new lazy value that combines the results.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let x = RcLazy::new(RcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
	/// let y = RcLazy::new(RcLazyConfig::new_thunk(|_| "World!".to_string()));
	/// // Note: LazySemigroup::append is usually called via Semigroup::append on the Lazy type
	/// let z = <RcLazyConfig as LazySemigroup<String>>::append(x, y);
	/// assert_eq!(Lazy::force_or_panic(&z), "Hello, World!".to_string());
	/// ```
	fn append<'a>(
		x: Lazy<'a, Self, A>,
		y: Lazy<'a, Self, A>,
	) -> Lazy<'a, Self, A>
	where
		A: Semigroup + Clone + 'a;
}

/// Trait for `Lazy` configurations that support monoid operations.
pub trait LazyMonoid<A>: LazySemigroup<A> {
	/// Returns the identity element.
	///
	/// This method returns a lazy value that evaluates to the identity element.
	///
	/// ### Type Signature
	///
	/// `forall config a. Monoid a => () -> Lazy config a`
	///
	/// ### Returns
	///
	/// A lazy value containing the identity element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let x = <RcLazyConfig as LazyMonoid<String>>::empty();
	/// assert_eq!(Lazy::force_or_panic(&x), "".to_string());
	/// ```
	fn empty<'a>() -> Lazy<'a, Self, A>
	where
		A: Monoid + Clone + 'a;
}

/// Trait for `Lazy` configurations that support defer operations.
pub trait LazyDefer<'a, A>: LazyConfig {
	/// Creates a value from a computation that produces the value.
	///
	/// This method defers the construction of a `Lazy` value.
	///
	/// ### Type Signature
	///
	/// `forall config a. (() -> Lazy config a) -> Lazy config a`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the cloneable function wrapper.
	///
	/// ### Parameters
	///
	/// * `f`: A thunk (wrapped in a cloneable function) that produces the value.
	///
	/// ### Returns
	///
	/// A new lazy value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::lazy::*};
	///
	/// let lazy = <RcLazyConfig as LazyDefer<i32>>::defer::<RcFnBrand>(
	///     cloneable_fn_new::<RcFnBrand, _, _>(|_| RcLazy::new(RcLazyConfig::new_thunk(|_| 42)))
	/// );
	/// assert_eq!(Lazy::force_or_panic(&lazy), 42);
	/// ```
	fn defer<FnBrand>(
		f: <FnBrand as CloneableFn>::Of<'a, (), Lazy<'a, Self, A>>
	) -> Lazy<'a, Self, A>
	where
		FnBrand: CloneableFn + 'a,
		A: Clone + 'a;
}

// =============================================================================
// RcLazyConfig - Single-threaded lazy values
// =============================================================================

/// Configuration for `Rc`-based `Lazy` values.
///
/// Uses `Rc` for shared ownership, `OnceCell` for memoization, and `RcFn` for thunks.
/// This configuration is **not thread-safe**.
///
/// ### Examples
///
/// ```
/// use fp_library::types::lazy::*;
///
/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
/// assert_eq!(Lazy::force_or_panic(&lazy), 42);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcLazyConfig;

impl RcLazyConfig {
	/// Creates a new thunk from a closure.
	///
	/// ### Type Signature
	///
	/// `forall a. (Fn(()) -> a) -> ThunkOf a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The return type of the closure.
	/// * `F`: The closure type.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to wrap.
	///
	/// ### Returns
	///
	/// A new thunk.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let thunk = RcLazyConfig::new_thunk(|_| 42);
	/// assert_eq!(thunk(()), 42);
	/// ```
	pub fn new_thunk<'a, A, F>(f: F) -> <Self as LazyConfig>::ThunkOf<'a, A>
	where
		A: 'a,
		F: Fn(()) -> A + Clone + 'a,
	{
		<RcFnBrand as CloneableFn>::new(f)
	}
}

impl LazyConfig for RcLazyConfig {
	type PtrBrand = RcBrand;
	type OnceBrand = OnceCellBrand;
	type FnBrand = RcFnBrand;
	type ThunkOf<'a, A>
		= <RcFnBrand as CloneableFn>::Of<'a, (), A>
	where
		A: 'a;
}

impl<A> LazySemigroup<A> for RcLazyConfig {
	/// Combines two lazy values.
	///
	/// The combination is itself lazy: the result is a new thunk that, when forced,
	/// forces both input values and combines them.
	///
	/// ### Type Signature
	///
	/// `forall a. Semigroup a => (RcLazy a, RcLazy a) -> RcLazy a`
	///
	/// ### Parameters
	///
	/// * `x`: The first lazy value.
	/// * `y`: The second lazy value.
	///
	/// ### Returns
	///
	/// A new lazy value that combines the results.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let x = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
	/// let y = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "World!".to_string()));
	/// let z = ArcLazyConfig::append(x, y);
	/// assert_eq!(Lazy::force_or_panic(&z), "Hello, World!".to_string());
	/// ```
	fn append<'a>(
		x: Lazy<'a, Self, A>,
		y: Lazy<'a, Self, A>,
	) -> Lazy<'a, Self, A>
	where
		A: Semigroup + Clone + 'a,
	{
		let thunk = Self::new_thunk(move |_| {
			let x_val = match Lazy::force(&x) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
			};
			let y_val = match Lazy::force(&y) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
			};
			Semigroup::append(x_val, y_val)
		});
		Lazy::new(thunk)
	}
}

impl<A> LazyMonoid<A> for RcLazyConfig {
	/// Returns the identity element.
	///
	/// This method returns a lazy value that evaluates to the underlying type's identity element.
	///
	/// ### Type Signature
	///
	/// `forall a. Monoid a => () -> RcLazy a`
	///
	/// ### Returns
	///
	/// A lazy value containing the identity element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let x: ArcLazy<String> = ArcLazyConfig::empty();
	/// assert_eq!(Lazy::force_or_panic(&x), "".to_string());
	/// ```
	fn empty<'a>() -> Lazy<'a, Self, A>
	where
		A: Monoid + Clone + 'a,
	{
		let thunk = Self::new_thunk(move |_| Monoid::empty());
		Lazy::new(thunk)
	}
}

impl<'a, A> LazyDefer<'a, A> for RcLazyConfig {
	/// Creates a value from a computation that produces the value.
	///
	/// This method defers the construction of a `Lazy` value.
	///
	/// ### Type Signature
	///
	/// `forall a. (() -> RcLazy a) -> RcLazy a`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the cloneable function wrapper.
	///
	/// ### Parameters
	///
	/// * `f`: A thunk (wrapped in a cloneable function) that produces the value.
	///
	/// ### Returns
	///
	/// A new lazy value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::lazy::*};
	///
	/// let lazy = RcLazyConfig::defer::<RcFnBrand>(
	///     cloneable_fn_new::<RcFnBrand, _, _>(|_| RcLazy::new(RcLazyConfig::new_thunk(|_| 42)))
	/// );
	/// assert_eq!(Lazy::force_or_panic(&lazy), 42);
	/// ```
	fn defer<FnBrand>(
		f: <FnBrand as CloneableFn>::Of<'a, (), Lazy<'a, Self, A>>
	) -> Lazy<'a, Self, A>
	where
		FnBrand: CloneableFn + 'a,
		A: Clone + 'a,
	{
		let thunk = Self::new_thunk(move |_| {
			let inner_lazy = f(());
			match Lazy::force(&inner_lazy) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
			}
		});
		Lazy::new(thunk)
	}
}

// =============================================================================
// ArcLazyConfig - Thread-safe lazy values
// =============================================================================

/// Configuration for `Arc`-based `Lazy` values.
///
/// Uses `Arc` for shared ownership, `OnceLock` for memoization, and thread-safe `ArcFn` for thunks.
/// This configuration is **thread-safe** and requires `A: Send + Sync` for full functionality.
///
/// ### Thread Safety
///
/// `ArcLazy<A>` is `Send + Sync` when `A: Send + Sync`. This allows lazy values
/// to be shared across threads safely.
///
/// ### Examples
///
/// ```
/// use fp_library::types::lazy::*;
/// use std::thread;
///
/// let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(|_| 42));
/// let lazy_clone = lazy.clone();
///
/// let handle = thread::spawn(move || {
///     Lazy::force_or_panic(&lazy_clone)
/// });
///
/// assert_eq!(handle.join().unwrap(), 42);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcLazyConfig;

impl ArcLazyConfig {
	/// Creates a new thread-safe thunk from a closure.
	///
	/// The closure must be `Send + Sync` to ensure thread safety.
	///
	/// ### Type Signature
	///
	/// `forall a. (Fn(()) -> a + Send + Sync) -> ThunkOf a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The return type of the closure.
	/// * `F`: The closure type (must be `Send + Sync`).
	///
	/// ### Parameters
	///
	/// * `f`: The closure to wrap.
	///
	/// ### Returns
	///
	/// A new thread-safe thunk.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let thunk = ArcLazyConfig::new_thunk(|_| 42);
	/// assert_eq!(thunk(()), 42);
	/// ```
	pub fn new_thunk<'a, A, F>(f: F) -> <Self as LazyConfig>::ThunkOf<'a, A>
	where
		A: 'a,
		F: Fn(()) -> A + Send + Sync + 'a,
	{
		<ArcFnBrand as SendCloneableFn>::send_cloneable_fn_new(f)
	}
}

impl LazyConfig for ArcLazyConfig {
	type PtrBrand = ArcBrand;
	type OnceBrand = OnceLockBrand;
	type FnBrand = ArcFnBrand;
	// Use SendOf for thread-safe thunks
	type ThunkOf<'a, A>
		= <ArcFnBrand as SendCloneableFn>::SendOf<'a, (), A>
	where
		A: 'a;
}

// LazySemigroup for ArcLazyConfig requires A: Send + Sync because:
// 1. The closure captures Lazy values which must be Send + Sync to be in a Send + Sync closure
// 2. The result A is stored in OnceLock which requires Send + Sync for thread-safe access
impl<A: Send + Sync> LazySemigroup<A> for ArcLazyConfig {
	/// Combines two lazy values.
	///
	/// The combination is itself lazy: the result is a new thunk that, when forced,
	/// forces both input values and combines them.
	///
	/// ### Type Signature
	///
	/// `forall a. (Semigroup a, Send a, Sync a) => (ArcLazy a, ArcLazy a) -> ArcLazy a`
	///
	/// ### Parameters
	///
	/// * `x`: The first lazy value.
	/// * `y`: The second lazy value.
	///
	/// ### Returns
	///
	/// A new lazy value that combines the results.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let x = RcLazy::new(RcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
	/// let y = RcLazy::new(RcLazyConfig::new_thunk(|_| "World!".to_string()));
	/// let z = RcLazyConfig::append(x, y);
	/// assert_eq!(Lazy::force_or_panic(&z), "Hello, World!".to_string());
	/// ```
	fn append<'a>(
		x: Lazy<'a, Self, A>,
		y: Lazy<'a, Self, A>,
	) -> Lazy<'a, Self, A>
	where
		A: Semigroup + Clone + 'a,
	{
		let thunk = Self::new_thunk(move |_| {
			let x_val = match Lazy::force(&x) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
			};
			let y_val = match Lazy::force(&y) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
			};
			Semigroup::append(x_val, y_val)
		});
		Lazy::new(thunk)
	}
}

impl<A: Send + Sync> LazyMonoid<A> for ArcLazyConfig {
	/// Returns the identity element.
	///
	/// This method returns a lazy value that evaluates to the underlying type's identity element.
	///
	/// ### Type Signature
	///
	/// `forall a. (Monoid a, Send a, Sync a) => () -> ArcLazy a`
	///
	/// ### Returns
	///
	/// A lazy value containing the identity element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let x: RcLazy<String> = RcLazyConfig::empty();
	/// assert_eq!(Lazy::force_or_panic(&x), "".to_string());
	/// ```
	fn empty<'a>() -> Lazy<'a, Self, A>
	where
		A: Monoid + Clone + 'a,
	{
		let thunk = Self::new_thunk(move |_| Monoid::empty());
		Lazy::new(thunk)
	}
}

// Note: LazyDefer is NOT implemented for ArcLazyConfig because the Defer trait
// allows any FnBrand, but ArcLazy requires Send + Sync closures. Users should
// use SendDefer instead for thread-safe deferred lazy evaluation.

// =============================================================================
// Type Aliases
// =============================================================================

/// Type alias for `Rc`-based `Lazy` values.
///
/// Use this for single-threaded lazy evaluation. Not thread-safe.
pub type RcLazy<'a, A> = Lazy<'a, RcLazyConfig, A>;

/// Type alias for `Arc`-based `Lazy` values.
///
/// Use this for thread-safe lazy evaluation. Requires `A: Send + Sync`.
pub type ArcLazy<'a, A> = Lazy<'a, ArcLazyConfig, A>;

// =============================================================================
// LazyError
// =============================================================================

/// Error type for `Lazy` evaluation failures.
///
/// This error is returned when a thunk panics during evaluation.
#[derive(Clone, Debug, Default, Error, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("thunk panicked during evaluation{}", .0.as_ref().map(|m| format!(": {m}")).unwrap_or_default())]
pub struct LazyError(Option<Arc<str>>);

impl LazyError {
	/// Creates a `LazyError` from a panic payload.
	///
	/// ### Type Signature
	///
	/// `Box (dyn Any + Send) -> LazyError`
	///
	/// ### Parameters
	///
	/// * `payload`: The panic payload.
	///
	/// ### Returns
	///
	/// A new `LazyError`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let payload = Box::new("oops");
	/// let error = LazyError::from_panic(payload);
	/// assert_eq!(error.to_string(), "thunk panicked during evaluation: oops");
	/// ```
	pub fn from_panic(payload: Box<dyn std::any::Any + Send + 'static>) -> Self {
		let msg = if let Some(s) = payload.downcast_ref::<&str>() {
			Some(Arc::from(*s))
		} else {
			payload.downcast_ref::<String>().map(|s| Arc::from(s.as_str()))
		};
		Self(msg)
	}

	/// Returns the panic message, if available.
	///
	/// ### Type Signature
	///
	/// `LazyError -> Option &str`
	///
	/// ### Returns
	///
	/// The panic message as a string slice, or `None` if no message was captured.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let payload = Box::new("oops");
	/// let error = LazyError::from_panic(payload);
	/// assert_eq!(error.panic_message(), Some("oops"));
	/// ```
	pub fn panic_message(&self) -> Option<&str> {
		self.0.as_deref()
	}
}

// =============================================================================
// LazyInner and Lazy
// =============================================================================

struct LazyInner<'a, Config: LazyConfig, A: 'a> {
	/// The memoized result (computed at most once).
	/// Stores Result<A, Arc<LazyError>> to capture both successful values and errors.
	once: <Config::OnceBrand as Once>::Of<Result<A, Arc<LazyError>>>,
	/// The thunk, wrapped in ThunkWrapper::Cell for interior mutability.
	thunk: <Config::PtrBrand as ThunkWrapper>::Cell<Config::ThunkOf<'a, A>>,
}

/// Represents a lazily-computed, memoized value with shared semantics.
///
/// `Lazy` stores a computation (a thunk) that is executed only when the value is needed.
/// The result is then cached (memoized) so that subsequent accesses return the same value
/// without re-executing the computation.
///
/// This `Lazy` type uses shared semantics: cloning the `Lazy` value shares the
/// underlying memoization state. If one clone forces the value, all other
/// clones see the result.
///
/// ### Type Parameters
///
/// * `Config`: The configuration for the `Lazy` value (e.g., `RcLazyConfig`, `ArcLazyConfig`).
/// * `A`: The type of the value.
///
/// ### Configuration Choice
///
/// - Use [`RcLazy`] for single-threaded contexts
/// - Use [`ArcLazy`] for thread-safe contexts (requires `A: Send + Sync`)
pub struct Lazy<'a, Config: LazyConfig, A>(
	// CloneableOf wraps LazyInner for shared ownership
	<Config::PtrBrand as RefCountedPointer>::CloneableOf<LazyInner<'a, Config, A>>,
);

impl<'a, Config: LazyConfig, A> Lazy<'a, Config, A> {
	/// Creates a new `Lazy` value from a thunk.
	///
	/// This method creates a new `Lazy` value that will evaluate the given thunk when forced.
	///
	/// ### Type Signature
	///
	/// `forall config a. config::ThunkOf a -> Lazy config a`
	///
	/// ### Parameters
	///
	/// * `thunk`: The thunk that produces the value.
	///
	/// ### Returns
	///
	/// A new `Lazy` value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
	/// assert_eq!(Lazy::force(&lazy).unwrap(), &42);
	/// ```
	pub fn new(thunk: Config::ThunkOf<'a, A>) -> Self {
		let inner =
			LazyInner { once: Config::OnceBrand::new(), thunk: Config::PtrBrand::new(Some(thunk)) };
		Self(Config::PtrBrand::cloneable_new(inner))
	}

	/// Forces the evaluation of the thunk and returns the value.
	///
	/// If the value has already been computed, the cached value is returned.
	/// If the computation panics, the panic is caught and returned as a `LazyError`.
	/// Subsequent calls will return the same error.
	///
	/// ### Type Signature
	///
	/// `forall config a. Lazy config a -> Result (&a) LazyError`
	///
	/// ### Parameters
	///
	/// * `this`: The lazy value to force.
	///
	/// ### Returns
	///
	/// The computed value or an error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
	/// assert_eq!(Lazy::force(&lazy).unwrap(), &42);
	/// ```
	pub fn force(this: &Self) -> Result<&A, LazyError> {
		let inner = &*this.0;
		let result: &Result<A, Arc<LazyError>> =
			<Config::OnceBrand as Once>::get_or_init(&inner.once, || {
				let thunk = Config::PtrBrand::take(&inner.thunk)
					.expect("unreachable: get_or_init guarantees single execution");
				// Call thunk with () since it's Fn(()) -> A
				std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| thunk(())))
					.map_err(|payload| Arc::new(LazyError::from_panic(payload)))
			});
		result.as_ref().map_err(|e| (**e).clone())
	}

	/// Forces the evaluation of the thunk and returns the value, cloning it.
	///
	/// This is a convenience method that clones the value after forcing it.
	///
	/// ### Type Signature
	///
	/// `forall config a. Clone a => Lazy config a -> Result a LazyError`
	///
	/// ### Parameters
	///
	/// * `this`: The lazy value to force.
	///
	/// ### Returns
	///
	/// The computed value (cloned) or an error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
	/// assert_eq!(Lazy::force_cloned(&lazy).unwrap(), 42);
	/// ```
	pub fn force_cloned(this: &Self) -> Result<A, LazyError>
	where
		A: Clone,
	{
		Self::force(this).cloned()
	}

	/// Forces the evaluation of the thunk and returns the value, panicking if the thunk panics.
	///
	/// This method unwraps the result of `force`, propagating any panic that occurred during evaluation.
	///
	/// ### Type Signature
	///
	/// `forall config a. Clone a => Lazy config a -> a`
	///
	/// ### Parameters
	///
	/// * `this`: The lazy value to force.
	///
	/// ### Returns
	///
	/// The computed value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
	/// assert_eq!(Lazy::force_or_panic(&lazy), 42);
	/// ```
	pub fn force_or_panic(this: &Self) -> A
	where
		A: Clone,
	{
		match Self::force(this) {
			Ok(v) => v.clone(),
			Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
		}
	}

	/// Forces the evaluation of the thunk and returns a reference to the value, panicking if the thunk panics.
	///
	/// This method unwraps the result of `force`, propagating any panic that occurred during evaluation.
	///
	/// ### Type Signature
	///
	/// `forall config a. Lazy config a -> &a`
	///
	/// ### Parameters
	///
	/// * `this`: The lazy value to force.
	///
	/// ### Returns
	///
	/// A reference to the computed value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
	/// assert_eq!(Lazy::force_ref_or_panic(&lazy), &42);
	/// ```
	pub fn force_ref_or_panic(this: &Self) -> &A {
		match Self::force(this) {
			Ok(v) => v,
			Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
		}
	}

	/// Returns true if the lazy value has been forced and panicked.
	///
	/// ### Type Signature
	///
	/// `forall config a. Lazy config a -> bool`
	///
	/// ### Parameters
	///
	/// * `this`: The lazy value to check.
	///
	/// ### Returns
	///
	/// `true` if the value has been forced and panicked, `false` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| panic!("oops")));
	/// let _ = Lazy::force(&lazy);
	/// assert!(Lazy::is_poisoned(&lazy));
	/// ```
	pub fn is_poisoned(this: &Self) -> bool {
		let inner = &*this.0;
		if let Some(result) = <Config::OnceBrand as Once>::get(&inner.once) {
			result.is_err()
		} else {
			false
		}
	}

	/// Returns the error if the lazy value has been forced and panicked.
	///
	/// ### Type Signature
	///
	/// `forall config a. Lazy config a -> Option LazyError`
	///
	/// ### Parameters
	///
	/// * `this`: The lazy value to check.
	///
	/// ### Returns
	///
	/// The error if the value has been forced and panicked, `None` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::lazy::*;
	///
	/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| panic!("oops")));
	/// let _ = Lazy::force(&lazy);
	/// assert!(Lazy::get_error(&lazy).is_some());
	/// ```
	pub fn get_error(this: &Self) -> Option<LazyError> {
		let inner = &*this.0;
		if let Some(Err(e)) = <Config::OnceBrand as Once>::get(&inner.once) {
			Some(LazyError(e.0.clone()))
		} else {
			None
		}
	}
}

impl<'a, Config: LazyConfig, A: Debug> Debug for Lazy<'a, Config, A> {
	fn fmt(
		&self,
		f: &mut Formatter<'_>,
	) -> fmt::Result {
		let inner = &*self.0;
		f.debug_struct("Lazy")
			.field("value", &<Config::OnceBrand as Once>::get(&inner.once))
			.finish()
	}
}

impl<'a, Config: LazyConfig, A: Clone> Clone for Lazy<'a, Config, A> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl_kind! {
	impl<Config: LazyConfig> for LazyBrand<Config> {
		type Of<'a, A: 'a>: 'a = Lazy<'a, Config, A>;
	}
}

// =============================================================================
// Type Class Implementations
// =============================================================================

// Note: We do NOT implement TrySemigroup/TryMonoid explicitly for Lazy.
// Since Lazy implements Semigroup/Monoid, it inherits the blanket impls from
// TrySemigroup/TryMonoid which use Error = Infallible. Users should handle
// errors via force() returning Result<&A, LazyError>.

impl<'a, Config: LazySemigroup<A>, A: Semigroup + Clone + 'a> Semigroup for Lazy<'a, Config, A> {
	/// Combines two lazy values.
	///
	/// The combination is itself lazy: the result is a new thunk that, when forced,
	/// forces both input values and combines them.
	///
	/// ### Type Signature
	///
	/// `forall config a. Semigroup a => (Lazy config a, Lazy config a) -> Lazy config a`
	///
	/// ### Parameters
	///
	/// * `x`: The first lazy value.
	/// * `y`: The second lazy value.
	///
	/// ### Returns
	///
	/// A new lazy value that combines the results.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{functions::*, types::*};
	///
	/// let x = RcLazy::new(RcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
	/// let y = RcLazy::new(RcLazyConfig::new_thunk(|_| "World!".to_string()));
	/// let z = append::<RcLazy<_>>(x, y);
	/// assert_eq!(Lazy::force_or_panic(&z), "Hello, World!".to_string());
	/// ```
	fn append(
		x: Self,
		y: Self,
	) -> Self {
		Config::append(x, y)
	}
}

impl<'a, Config: LazyMonoid<A>, A: Monoid + Clone + 'a> Monoid for Lazy<'a, Config, A> {
	/// Returns the identity element.
	///
	/// This method returns a lazy value that evaluates to the underlying type's identity element.
	///
	/// ### Type Signature
	///
	/// `forall config a. Monoid a => () -> Lazy config a`
	///
	/// ### Returns
	///
	/// A lazy value containing the identity element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{functions::*, types::*};
	///
	/// let x = empty::<RcLazy<String>>();
	/// assert_eq!(Lazy::force_or_panic(&x), "".to_string());
	/// ```
	fn empty() -> Self {
		Config::empty()
	}
}

impl<'a, Config: LazyDefer<'a, A>, A: Clone + 'a> Defer<'a> for Lazy<'a, Config, A> {
	/// Creates a value from a computation that produces the value.
	///
	/// This method defers the construction of a `Lazy` value.
	///
	/// ### Type Signature
	///
	/// `forall config a. (() -> Lazy config a) -> Lazy config a`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the cloneable function wrapper.
	///
	/// ### Parameters
	///
	/// * `f`: A thunk (wrapped in a cloneable function) that produces the value.
	///
	/// ### Returns
	///
	/// A new lazy value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::lazy::*};
	///
	/// let lazy = defer::<RcLazy<i32>, RcFnBrand>(
	///     cloneable_fn_new::<RcFnBrand, _, _>(|_| RcLazy::new(RcLazyConfig::new_thunk(|_| 42)))
	/// );
	/// assert_eq!(Lazy::force_or_panic(&lazy), 42);
	/// ```
	fn defer<FnBrand>(f: <FnBrand as CloneableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized,
		FnBrand: CloneableFn + 'a,
	{
		Config::defer::<FnBrand>(f)
	}
}

use crate::classes::send_defer::SendDefer;

impl SendDefer for LazyBrand<ArcLazyConfig> {
	/// Creates a deferred value from a thread-safe thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. (Send a, Sync a) => (() -> ArcLazy a) -> ArcLazy a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `thunk`: The function that produces the value.
	///
	/// ### Returns
	///
	/// A deferred value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::lazy::*};
	///
	/// let lazy = send_defer::<LazyBrand<ArcLazyConfig>, _, _>(|| ArcLazy::new(ArcLazyConfig::new_thunk(|_| 42)));
	/// assert_eq!(Lazy::force_or_panic(&lazy), 42);
	/// ```
	fn send_defer<'a, A>(thunk: impl 'a + Fn() -> Self::Of<'a, A> + Send + Sync) -> Self::Of<'a, A>
	where
		A: Clone + Send + Sync + 'a,
	{
		let thunk = ArcLazyConfig::new_thunk(move |_| {
			let inner_lazy = thunk();
			match Lazy::force(&inner_lazy) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(e.to_string())),
			}
		});
		Lazy::new(thunk)
	}
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::RcFnBrand,
		classes::{cloneable_fn::CloneableFn, defer::Defer},
	};
	use std::{cell::RefCell, rc::Rc};

	/// Tests that `Lazy::force` memoizes the result.
	#[test]
	fn force_memoization() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();

		let lazy = RcLazy::new(<RcFnBrand as CloneableFn>::new(move |_| {
			*counter_clone.borrow_mut() += 1;
			42
		}));

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(Lazy::force(&lazy).unwrap(), &42);
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(Lazy::force(&lazy).unwrap(), &42);
		// The new implementation uses shared semantics!
		// So cloning the Lazy should SHARE the OnceCell.
		let lazy_clone = lazy.clone();
		assert_eq!(Lazy::force(&lazy_clone).unwrap(), &42);
		assert_eq!(*counter.borrow(), 1); // Should still be 1
	}

	/// Tests that `Lazy::defer` delays execution until forced.
	#[test]
	fn defer_execution_order() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();

		let lazy = RcLazy::defer::<RcFnBrand>(<RcFnBrand as CloneableFn>::new(move |_| {
			*counter_clone.borrow_mut() += 1;
			RcLazy::new(<RcFnBrand as CloneableFn>::new(|_| 42))
		}));

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(Lazy::force(&lazy).unwrap(), &42);
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that panics are caught and cached.
	#[test]
	fn panic_caching() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();

		let lazy = RcLazy::new(<RcFnBrand as CloneableFn>::new(move |_| {
			*counter_clone.borrow_mut() += 1;
			if *counter_clone.borrow() == 1 {
				panic!("oops");
			}
			42
		}));

		assert!(Lazy::force(&lazy).is_err());
		assert_eq!(*counter.borrow(), 1);
		assert!(Lazy::force(&lazy).is_err());
		assert_eq!(*counter.borrow(), 1); // Should not re-execute
	}

	/// Tests that `force_or_panic` returns the value on success.
	#[test]
	fn force_or_panic_success() {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
		assert_eq!(Lazy::force_or_panic(&lazy), 42);
	}

	/// Tests that `force_or_panic` propagates the panic on failure.
	#[test]
	#[should_panic(expected = "thunk panicked during evaluation: oops")]
	fn force_or_panic_failure() {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| panic!("oops")));
		Lazy::force_or_panic(&lazy);
	}

	/// Tests that `force_ref_or_panic` returns a reference to the value on success.
	#[test]
	fn force_ref_or_panic_success() {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
		assert_eq!(Lazy::force_ref_or_panic(&lazy), &42);
	}

	/// Tests that `force_ref_or_panic` propagates the panic on failure.
	#[test]
	#[should_panic(expected = "thunk panicked during evaluation: oops")]
	fn force_ref_or_panic_failure() {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| panic!("oops")));
		Lazy::force_ref_or_panic(&lazy);
	}

	/// Tests `is_poisoned` and `get_error` state transitions.
	#[test]
	fn is_poisoned_and_get_error() {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| panic!("oops")));
		assert!(!Lazy::is_poisoned(&lazy));
		assert!(Lazy::get_error(&lazy).is_none());

		let _ = Lazy::force(&lazy);

		assert!(Lazy::is_poisoned(&lazy));
		let err = Lazy::get_error(&lazy).unwrap();
		assert_eq!(err.to_string(), "thunk panicked during evaluation: oops");
	}

	/// Tests that `force_cloned` returns a cloned value.
	#[test]
	fn force_cloned() {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
		assert_eq!(Lazy::force_cloned(&lazy).unwrap(), 42);
	}

	/// Tests `Semigroup::append` for `Lazy`.
	#[test]
	fn semigroup_append() {
		let x = RcLazy::new(RcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
		let y = RcLazy::new(RcLazyConfig::new_thunk(|_| "World!".to_string()));
		let z = Semigroup::append(x, y);
		assert_eq!(Lazy::force_or_panic(&z), "Hello, World!".to_string());
	}

	/// Tests `Monoid::empty` for `Lazy`.
	#[test]
	fn monoid_empty() {
		let x = <RcLazy<String> as Monoid>::empty();
		assert_eq!(Lazy::force_or_panic(&x), "".to_string());
	}

	/// Tests that `ArcLazy` is thread-safe.
	#[test]
	fn arc_lazy_thread_safety() {
		use std::sync::{Arc, Mutex};
		use std::thread;

		let counter = Arc::new(Mutex::new(0));
		let counter_clone = counter.clone();

		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(move |_| {
			let mut guard = counter_clone.lock().unwrap();
			*guard += 1;
			42
		}));

		let lazy_clone = lazy.clone();

		let handle = thread::spawn(move || Lazy::force_or_panic(&lazy_clone));

		assert_eq!(handle.join().unwrap(), 42);
		assert_eq!(Lazy::force_or_panic(&lazy), 42);
		// Should only be computed once due to shared memoization
		assert_eq!(*counter.lock().unwrap(), 1);
	}

	/// Tests `Semigroup::append` for `ArcLazy`.
	#[test]
	fn arc_lazy_semigroup_append() {
		let x = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
		let y = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "World!".to_string()));
		let z = Semigroup::append(x, y);
		assert_eq!(Lazy::force_or_panic(&z), "Hello, World!".to_string());
	}

	/// Tests `Monoid::empty` for `ArcLazy`.
	#[test]
	fn arc_lazy_monoid_empty() {
		let x = <ArcLazy<String> as Monoid>::empty();
		assert_eq!(Lazy::force_or_panic(&x), "".to_string());
	}
}
