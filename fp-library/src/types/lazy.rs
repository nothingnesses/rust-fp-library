//! Lazy value wrapper.
//!
//! This module defines the [`Lazy`] struct, which represents a lazily-computed, memoized value.
//! It implements [`Semigroup`], [`Monoid`], and [`Defer`].

use crate::{
	brands::{ArcBrand, ArcFnBrand, LazyBrand, OnceCellBrand, OnceLockBrand, RcBrand, RcFnBrand},
	classes::{
		clonable_fn::ClonableFn,
		defer::Defer,
		monoid::Monoid,
		once::Once,
		pointer::{RefCountedPointer, ThunkWrapper},
		semigroup::Semigroup,
		send_clonable_fn::SendClonableFn,
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
/// It ensures that compatible types are used together (e.g., `Rc` with `RefCell`, `Arc` with `Mutex`).
pub trait LazyConfig: 'static {
	/// The pointer brand for shared ownership (e.g., `RcBrand`, `ArcBrand`).
	type PtrBrand: RefCountedPointer + ThunkWrapper;
	/// The once-cell brand for memoization (e.g., `OnceCellBrand`, `OnceLockBrand`).
	type OnceBrand: Once;
	/// The function brand for thunk storage (e.g., `RcFnBrand`, `ArcFnBrand`).
	type FnBrand: ClonableFn;
	/// The thunk type to use for this configuration.
	/// Thunks deref to `Fn(()) -> A` to match `ClonableFn::Of<'a, (), A>`.
	type ThunkOf<'a, A>: Clone + Deref<Target: Fn(()) -> A>
	where
		A: 'a;

	/// Creates a new thunk from a closure.
	///
	/// ### Type Signature
	///
	/// `forall a f. (Fn(()) -> a) -> ThunkOf a`
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let thunk = RcLazyConfig::new_thunk(|_| 42);
	/// assert_eq!(thunk(()), 42);
	/// ```
	fn new_thunk<'a, A, F>(f: F) -> Self::ThunkOf<'a, A>
	where
		A: 'a,
		F: Fn(()) -> A + Clone + 'a;
}

/// Extension trait for thread-safe `Lazy` configurations.
pub trait SendLazyConfig: LazyConfig {
	/// The thread-safe thunk type. Same as `ThunkOf` but guaranteed `Send + Sync`.
	type SendThunkOf<'a, A: Send + Sync>: Clone
		+ Send
		+ Sync
		+ Deref<Target: Fn(()) -> A + Send + Sync>
	where
		A: 'a;

	/// Creates a new thread-safe thunk from a closure.
	///
	/// ### Type Signature
	///
	/// `forall a f. (Fn(()) -> a + Send + Sync) -> SendThunkOf a`
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
	/// A new thread-safe thunk.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let thunk = ArcLazyConfig::new_send_thunk(|_| 42);
	/// assert_eq!(thunk(()), 42);
	/// ```
	fn new_send_thunk<'a, A, F>(f: F) -> Self::SendThunkOf<'a, A>
	where
		A: Send + Sync + 'a,
		F: Fn(()) -> A + Send + Sync + 'a;

	/// Converts a thread-safe thunk into a regular thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. SendThunkOf a -> ThunkOf a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The return type of the thunk.
	///
	/// ### Parameters
	///
	/// * `t`: The thread-safe thunk.
	///
	/// ### Returns
	///
	/// The thunk as a regular `ThunkOf`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let send_thunk = ArcLazyConfig::new_send_thunk(|_| 42);
	/// let thunk = ArcLazyConfig::into_thunk(send_thunk);
	/// assert_eq!(thunk(()), 42);
	/// ```
	fn into_thunk<'a, A>(t: Self::SendThunkOf<'a, A>) -> Self::ThunkOf<'a, A>
	where
		A: 'a + Send + Sync;
}

/// Configuration for `Rc`-based `Lazy` values.
///
/// Uses `Rc` for shared ownership, `OnceCell` for memoization, and `RcFn` for thunks.
/// This configuration is not thread-safe.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcLazyConfig;

impl LazyConfig for RcLazyConfig {
	type PtrBrand = RcBrand;
	type OnceBrand = OnceCellBrand;
	type FnBrand = RcFnBrand;
	type ThunkOf<'a, A>
		= <RcFnBrand as ClonableFn>::Of<'a, (), A>
	where
		A: 'a;

	/// Creates a new thunk from a closure.
	///
	/// ### Type Signature
	///
	/// `forall a f. (Fn(()) -> a) -> ThunkOf a`
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let thunk = RcLazyConfig::new_thunk(|_| 42);
	/// assert_eq!(thunk(()), 42);
	/// ```
	fn new_thunk<'a, A, F>(f: F) -> Self::ThunkOf<'a, A>
	where
		A: 'a,
		F: Fn(()) -> A + Clone + 'a,
	{
		<RcFnBrand as ClonableFn>::new(f)
	}
}

/// Configuration for `Arc`-based `Lazy` values.
///
/// Uses `Arc` for shared ownership, `OnceLock` for memoization, and `ArcFn` for thunks.
/// This configuration is thread-safe when `A` is `Send + Sync`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcLazyConfig;

impl LazyConfig for ArcLazyConfig {
	type PtrBrand = ArcBrand;
	type OnceBrand = OnceLockBrand;
	type FnBrand = ArcFnBrand;
	// Use ClonableFn::Of (non-Send) for ThunkOf to satisfy LazyConfig bounds.
	// ArcLazy will not be Send by default; use SendLazyConfig for Send thunks.
	type ThunkOf<'a, A>
		= <ArcFnBrand as ClonableFn>::Of<'a, (), A>
	where
		A: 'a;

	/// Creates a new thunk from a closure.
	///
	/// ### Type Signature
	///
	/// `forall a f. (Fn(()) -> a) -> ThunkOf a`
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let thunk = ArcLazyConfig::new_thunk(|_| 42);
	/// assert_eq!(thunk(()), 42);
	/// ```
	fn new_thunk<'a, A, F>(f: F) -> Self::ThunkOf<'a, A>
	where
		A: 'a,
		F: Fn(()) -> A + Clone + 'a,
	{
		<ArcFnBrand as ClonableFn>::new(f)
	}
}

impl SendLazyConfig for ArcLazyConfig {
	type SendThunkOf<'a, A: Send + Sync>
		= <ArcFnBrand as SendClonableFn>::SendOf<'a, (), A>
	where
		A: 'a;

	/// Creates a new thread-safe thunk from a closure.
	///
	/// ### Type Signature
	///
	/// `forall a f. (Fn(()) -> a + Send + Sync) -> SendThunkOf a`
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
	/// A new thread-safe thunk.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let thunk = ArcLazyConfig::new_send_thunk(|_| 42);
	/// assert_eq!(thunk(()), 42);
	/// ```
	fn new_send_thunk<'a, A, F>(f: F) -> Self::SendThunkOf<'a, A>
	where
		A: Send + Sync + 'a,
		F: Fn(()) -> A + Send + Sync + 'a,
	{
		<ArcFnBrand as SendClonableFn>::send_clonable_fn_new(f)
	}

	/// Converts a thread-safe thunk into a regular thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. SendThunkOf a -> ThunkOf a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The return type of the thunk.
	///
	/// ### Parameters
	///
	/// * `t`: The thread-safe thunk.
	///
	/// ### Returns
	///
	/// The thunk as a regular `ThunkOf`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let send_thunk = ArcLazyConfig::new_send_thunk(|_| 42);
	/// let thunk = ArcLazyConfig::into_thunk(send_thunk);
	/// assert_eq!(thunk(()), 42);
	/// ```
	fn into_thunk<'a, A>(t: Self::SendThunkOf<'a, A>) -> Self::ThunkOf<'a, A>
	where
		A: 'a + Send + Sync,
	{
		t
	}
}

/// Type alias for `Rc`-based `Lazy` values.
pub type RcLazy<'a, A> = Lazy<'a, RcLazyConfig, A>;

/// Type alias for `Arc`-based `Lazy` values.
pub type ArcLazy<'a, A> = Lazy<'a, ArcLazyConfig, A>;

/// Error type for `Lazy` evaluation failures.
///
/// This error is returned when a thunk panics during evaluation.
#[derive(Debug, Clone, Error)]
#[error("thunk panicked during evaluation{}", .0.as_ref().map(|m| format!(": {}", m)).unwrap_or_default())]
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
	/// use fp_library::types::lazy::LazyError;
	///
	/// let payload = Box::new("oops");
	/// let error = LazyError::from_panic(payload);
	/// assert_eq!(format!("{}", error), "thunk panicked during evaluation: oops");
	/// ```
	pub fn from_panic(payload: Box<dyn std::any::Any + Send + 'static>) -> Self {
		let msg = if let Some(s) = payload.downcast_ref::<&str>() {
			Some(Arc::from(*s))
		} else {
			payload.downcast_ref::<String>().map(|s| Arc::from(s.as_str()))
		};
		Self(msg)
	}
}

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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
	/// assert_eq!(Lazy::force(&lazy).unwrap(), &42);
	/// ```
	pub fn new(thunk: Config::ThunkOf<'a, A>) -> Self {
		let inner = LazyInner {
			once: Config::OnceBrand::new(),
			thunk: Config::PtrBrand::new_cell(Some(thunk)),
		};
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
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
			Err(e) => std::panic::resume_unwind(Box::new(format!("{}", e))),
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
	/// assert_eq!(Lazy::force_ref_or_panic(&lazy), &42);
	/// ```
	pub fn force_ref_or_panic(this: &Self) -> &A {
		match Self::force(this) {
			Ok(v) => v,
			Err(e) => std::panic::resume_unwind(Box::new(format!("{}", e))),
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
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

// Note: We do NOT implement TrySemigroup/TryMonoid explicitly for Lazy.
// Since Lazy implements Semigroup/Monoid, it inherits the blanket impls from
// TrySemigroup/TryMonoid which use Error = Infallible. Users should handle
// errors via force() returning Result<&A, LazyError>.

impl<'a, Config: LazyConfig, A: Semigroup + Clone + 'a> Semigroup for Lazy<'a, Config, A> {
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	/// use fp_library::types::string; // Import Semigroup impl for String
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
		let thunk = Config::new_thunk(move |_| {
			let x_val = match Lazy::force(&x) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(format!("{}", e))),
			};
			let y_val = match Lazy::force(&y) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(format!("{}", e))),
			};
			Semigroup::append(x_val, y_val)
		});
		Lazy::new(thunk)
	}
}

impl<'a, Config: LazyConfig, A: Monoid + Clone + 'a> Monoid for Lazy<'a, Config, A> {
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
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	/// use fp_library::types::string; // Import Monoid impl for String
	///
	/// let x = empty::<RcLazy<String>>();
	/// assert_eq!(Lazy::force_or_panic(&x), "".to_string());
	/// ```
	fn empty() -> Self {
		let thunk = Config::new_thunk(move |_| Monoid::empty());
		Lazy::new(thunk)
	}
}

impl<'a, Config: LazyConfig, A: Clone + 'a> Defer<'a> for Lazy<'a, Config, A> {
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
	/// * `FnBrand_`: The brand of the clonable function wrapper.
	///
	/// ### Parameters
	///
	/// * `f`: A thunk (wrapped in a clonable function) that produces the value.
	///
	/// ### Returns
	///
	/// A new lazy value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::lazy::*};
	///
	/// let lazy = defer::<RcLazy<i32>, RcFnBrand>(
	///     clonable_fn_new::<RcFnBrand, _, _>(|_| RcLazy::new(RcLazyConfig::new_thunk(|_| 42)))
	/// );
	/// assert_eq!(Lazy::force_or_panic(&lazy), 42);
	/// ```
	fn defer<FnBrand_>(f: <FnBrand_ as ClonableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized,
		FnBrand_: ClonableFn + 'a,
	{
		let thunk = Config::new_thunk(move |_| {
			let inner_lazy = f(());
			match Lazy::force(&inner_lazy) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(format!("{}", e))),
			}
		});
		Lazy::new(thunk)
	}
}

use crate::classes::send_defer::SendDefer;

impl<Config: SendLazyConfig> SendDefer for LazyBrand<Config> {
	/// Creates a deferred value from a thread-safe thunk.
	///
	/// ### Type Signature
	///
	/// `forall config a. (Send a, Sync a) => (() -> Lazy config a) -> Lazy config a`
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
	/// use fp_library::{brands::*, classes::send_defer::*, functions::*, types::lazy::*};
	///
	/// let lazy = send_defer::<LazyBrand<ArcLazyConfig>, _, _>(|| ArcLazy::new(ArcLazyConfig::new_thunk(|_| 42)));
	/// assert_eq!(Lazy::force_or_panic(&lazy), 42);
	/// ```
	fn send_defer<'a, A>(thunk: impl 'a + Fn() -> Self::Of<'a, A> + Send + Sync) -> Self::Of<'a, A>
	where
		A: Clone + Send + Sync + 'a,
	{
		let thunk = Config::new_send_thunk(move |_| {
			let inner_lazy = thunk();
			match Lazy::force(&inner_lazy) {
				Ok(v) => v.clone(),
				Err(e) => std::panic::resume_unwind(Box::new(format!("{}", e))),
			}
		});
		Lazy::new(Config::into_thunk(thunk))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::RcFnBrand,
		classes::{clonable_fn::ClonableFn, defer::Defer},
	};
	use std::{cell::RefCell, rc::Rc};

	/// Tests that `Lazy::force` memoizes the result.
	#[test]
	fn force_memoization() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();

		let lazy = RcLazy::new(<RcFnBrand as ClonableFn>::new(move |_| {
			*counter_clone.borrow_mut() += 1;
			42
		}));

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(Lazy::force(&lazy).unwrap(), &42);
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(Lazy::force(&lazy).unwrap(), &42);
		// Since we clone before forcing, and OnceCell is not shared across clones (it's deep cloned),
		// the counter increments again.
		// WAIT: The new implementation uses shared semantics!
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

		let lazy = RcLazy::defer::<RcFnBrand>(<RcFnBrand as ClonableFn>::new(move |_| {
			*counter_clone.borrow_mut() += 1;
			RcLazy::new(<RcFnBrand as ClonableFn>::new(|_| 42))
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

		let lazy = RcLazy::new(<RcFnBrand as ClonableFn>::new(move |_| {
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
		assert_eq!(format!("{}", err), "thunk panicked during evaluation: oops");
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
}
