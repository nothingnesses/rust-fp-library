//! Memoized lazy evaluation for fallible computations.
//!
//! Computes a [`Result`] at most once and caches either the success value or error. All clones share the same cache. Available in both single-threaded [`RcTryLazy`] and thread-safe [`ArcTryLazy`] variants.

use crate::{
	Apply,
	brands::TryLazyBrand,
	classes::{CloneableFn, Deferrable, SendDeferrable},
	impl_kind,
	kinds::*,
	types::{ArcLazyConfig, Lazy, LazyConfig, RcLazyConfig, TryThunk, TryTrampoline},
};
use fp_macros::{doc_params, doc_type_params, hm_signature};

/// A lazily-computed, memoized value that may fail.
///
/// The computation runs at most once. If it succeeds, the value is cached.
/// If it fails, the error is cached. Subsequent accesses return the cached result.
///
/// ### Type Parameters
///
/// * `A`: The type of the computed value.
/// * `E`: The type of the error.
/// * `Config`: The memoization configuration.
///
/// ### Fields
///
/// * `0`: The internal lazy cell.
pub struct TryLazy<'a, A, E, Config: LazyConfig = RcLazyConfig>(
	pub(crate) Config::TryLazy<'a, A, E>,
)
where
	A: 'a,
	E: 'a;

impl<'a, A, E, Config: LazyConfig> Clone for TryLazy<'a, A, E, Config>
where
	A: 'a,
	E: 'a,
{
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<'a, A, E, Config: LazyConfig> TryLazy<'a, A, E, Config>
where
	A: 'a,
	E: 'a,
{
	/// Gets the memoized result, computing on first access.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Returns
	///
	/// A result containing a reference to the value or error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
	/// assert_eq!(memo.evaluate(), Ok(&42));
	/// ```
	pub fn evaluate(&self) -> Result<&A, &E> {
		Config::try_evaluate(&self.0)
	}
}

impl<'a, A, E> TryLazy<'a, A, E, RcLazyConfig>
where
	A: 'a,
	E: 'a,
{
	/// Creates a new `TryLazy` that will run `f` on first access.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the initializer closure.")]
	///
	/// ### Parameters
	///
	#[doc_params("The closure that produces the result.")]
	///
	/// ### Returns
	///
	/// A new `TryLazy` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
	/// assert_eq!(memo.evaluate(), Ok(&42));
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + 'a,
	{
		TryLazy(RcLazyConfig::try_lazy_new(Box::new(f)))
	}
}

impl<'a, A, E> From<TryThunk<'a, A, E>> for TryLazy<'a, A, E, RcLazyConfig> {
	fn from(eval: TryThunk<'a, A, E>) -> Self {
		Self::new(move || eval.evaluate())
	}
}

impl<'a, A, E> From<TryTrampoline<A, E>> for TryLazy<'a, A, E, RcLazyConfig>
where
	A: Send,
	E: Send,
{
	fn from(task: TryTrampoline<A, E>) -> Self {
		Self::new(move || task.evaluate())
	}
}

impl<'a, A, E> From<Lazy<'a, A, ArcLazyConfig>> for TryLazy<'a, A, E, ArcLazyConfig>
where
	A: Clone + Send + Sync + 'a,
	E: Send + Sync + 'a,
{
	fn from(memo: Lazy<'a, A, ArcLazyConfig>) -> Self {
		Self::new(move || Ok(memo.evaluate().clone()))
	}
}

impl<'a, A, E> From<Lazy<'a, A, RcLazyConfig>> for TryLazy<'a, A, E, RcLazyConfig>
where
	A: Clone + 'a,
	E: 'a,
{
	fn from(memo: Lazy<'a, A, RcLazyConfig>) -> Self {
		Self::new(move || Ok(memo.evaluate().clone()))
	}
}

impl<'a, A> TryLazy<'a, A, String, RcLazyConfig>
where
	A: 'a,
{
	/// Creates a `TryLazy` that catches unwinds (panics).
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the initializer closure.")]
	///
	/// ### Parameters
	///
	#[doc_params("The closure that might panic.")]
	///
	/// ### Returns
	///
	/// A new `TryLazy` instance where panics are converted to `Err(String)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = TryLazy::<_, String, RcLazyConfig>::catch_unwind(|| {
	///     if true { panic!("oops") }
	///     42
	/// });
	/// assert_eq!(memo.evaluate(), Err(&"oops".to_string()));
	/// ```
	pub fn catch_unwind<F>(f: F) -> Self
	where
		F: FnOnce() -> A + std::panic::UnwindSafe + 'a,
	{
		Self::new(move || {
			std::panic::catch_unwind(f).map_err(|e| {
				if let Some(s) = e.downcast_ref::<&str>() {
					s.to_string()
				} else if let Some(s) = e.downcast_ref::<String>() {
					s.clone()
				} else {
					"Unknown panic".to_string()
				}
			})
		})
	}
}

impl<'a, A, E> TryLazy<'a, A, E, ArcLazyConfig>
where
	A: 'a,
	E: 'a,
{
	/// Creates a new `TryLazy` that will run `f` on first access.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the initializer closure.")]
	///
	/// ### Parameters
	///
	#[doc_params("The closure that produces the result.")]
	///
	/// ### Returns
	///
	/// A new `TryLazy` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = TryLazy::<_, _, ArcLazyConfig>::new(|| Ok::<i32, ()>(42));
	/// assert_eq!(memo.evaluate(), Ok(&42));
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + Send + 'a,
	{
		TryLazy(ArcLazyConfig::try_lazy_new(Box::new(f)))
	}
}

impl_kind! {
	impl<E: 'static, Config: LazyConfig> for TryLazyBrand<E, Config> {
		type Of<'a, A: 'a>: 'a = TryLazy<'a, A, E, Config>;
	}
}

impl Deferrable for TryLazyBrand<(), RcLazyConfig> {
	/// Defers a computation that produces a `TryLazy` value.
	///
	/// This flattens the nested structure: instead of `TryLazy<TryLazy<A, E>, E>`, we get `TryLazy<A, E>`.
	/// The inner `TryLazy` is computed only when the outer `TryLazy` is evaluated.
	///
	/// ### Type Signature
	///
	#[hm_signature(Deferrable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the deferred value.",
		"The brand of the cloneable function wrapper."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The thunk that produces the lazy value.")]
	///
	/// ### Returns
	///
	/// A new `TryLazy` value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*, functions::*};
	///
	/// let lazy = defer::<TryLazyBrand<(), RcLazyConfig>, _, RcFnBrand>(
	///     cloneable_fn_new::<RcFnBrand, _, _>(|_| RcTryLazy::new(|| Ok(42)))
	/// );
	/// assert_eq!(lazy.evaluate(), Ok(&42));
	/// ```
	fn defer<'a, A: 'a, FnBrand: 'a + CloneableFn>(
		f: <FnBrand as CloneableFn>::Of<
			'a,
			(),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		A: Clone,
	{
		RcTryLazy::new(move || f(()).evaluate().map(|a| a.clone()).map_err(|e| e.clone()))
	}
}

impl<E> SendDeferrable for TryLazyBrand<E, ArcLazyConfig>
where
	E: Clone + Send + Sync + 'static,
{
	/// Defers a computation that produces a thread-safe `TryLazy` value.
	///
	/// This flattens the nested structure: instead of `ArcTryLazy<ArcTryLazy<A, E>, E>`, we get `ArcTryLazy<A, E>`.
	/// The inner `TryLazy` is computed only when the outer `TryLazy` is evaluated.
	///
	/// ### Type Signature
	///
	#[hm_signature(SendDeferrable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the value.", "The type of the value.")]
	///
	/// ### Parameters
	///
	#[doc_params("The thunk that produces the lazy value.")]
	///
	/// ### Returns
	///
	/// A new `ArcTryLazy` value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let lazy = TryLazyBrand::<(), ArcLazyConfig>::send_defer(|| ArcTryLazy::new(|| Ok(42)));
	/// assert_eq!(lazy.evaluate(), Ok(&42));
	/// ```
	fn send_defer<'a, A>(
		thunk: impl 'a
		+ Fn() -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		+ Send
		+ Sync
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		A: Clone + Send + Sync + 'a,
	{
		ArcTryLazy::new(move || thunk().evaluate().cloned().map_err(Clone::clone))
	}
}

/// Single-threaded fallible memoization alias.
pub type RcTryLazy<'a, A, E> = TryLazy<'a, A, E, RcLazyConfig>;

/// Thread-safe fallible memoization alias.
pub type ArcTryLazy<'a, A, E> = TryLazy<'a, A, E, ArcLazyConfig>;

#[cfg(test)]
mod tests {
	use crate::types::RcLazy;

	use super::*;
	use std::cell::RefCell;
	use std::rc::Rc;

	/// Tests that `TryLazy` caches successful results.
	///
	/// Verifies that the initializer is called only once for success.
	#[test]
	fn test_try_memo_caching_ok() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo: RcTryLazy<i32, ()> = RcTryLazy::new(move || {
			*counter_clone.borrow_mut() += 1;
			Ok(42)
		});

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(memo.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(memo.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `TryLazy` caches error results.
	///
	/// Verifies that the initializer is called only once for error.
	#[test]
	fn test_try_memo_caching_err() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo: RcTryLazy<i32, i32> = RcTryLazy::new(move || {
			*counter_clone.borrow_mut() += 1;
			Err(0)
		});

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(memo.evaluate(), Err(&0));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(memo.evaluate(), Err(&0));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `TryLazy` shares the cache across clones.
	///
	/// Verifies that clones see the same result.
	#[test]
	fn test_try_memo_sharing() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo: RcTryLazy<i32, ()> = RcTryLazy::new(move || {
			*counter_clone.borrow_mut() += 1;
			Ok(42)
		});
		let shared = memo.clone();

		assert_eq!(memo.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(shared.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests `catch_unwind`.
	///
	/// Verifies that panics are caught and converted to errors.
	#[test]
	fn test_catch_unwind() {
		let memo = RcTryLazy::catch_unwind(|| {
			if true {
				panic!("oops")
			}
			42
		});

		match memo.evaluate() {
			Err(e) => assert_eq!(e, "oops"),
			Ok(_) => panic!("Should have failed"),
		}
	}

	/// Tests creation from `TryThunk`.
	#[test]
	fn test_try_memo_from_try_eval() {
		let eval = TryThunk::new(|| Ok::<i32, ()>(42));
		let memo = RcTryLazy::from(eval);
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests creation from `TryTrampoline`.
	#[test]
	fn test_try_memo_from_try_task() {
		let task = TryTrampoline::<i32, ()>::ok(42);
		let memo = RcTryLazy::from(task);
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests conversion to TryLazy.
	#[test]
	fn test_try_memo_from_rc_memo() {
		let memo = RcLazy::new(|| 42);
		let try_memo: crate::types::RcTryLazy<i32, ()> = crate::types::RcTryLazy::from(memo);
		assert_eq!(try_memo.evaluate(), Ok(&42));
	}

	/// Tests conversion to ArcTryLazy.
	#[test]
	fn test_try_memo_from_arc_memo() {
		use crate::types::ArcLazy;
		let memo = ArcLazy::new(|| 42);
		let try_memo: crate::types::ArcTryLazy<i32, ()> = crate::types::ArcTryLazy::from(memo);
		assert_eq!(try_memo.evaluate(), Ok(&42));
	}

	/// Tests SendDefer implementation.
	#[test]
	fn test_send_defer() {
		use crate::classes::send_deferrable::send_defer;

		let memo: ArcTryLazy<i32, ()> =
			send_defer::<TryLazyBrand<(), ArcLazyConfig>, _, _>(|| ArcTryLazy::new(|| Ok(42)));
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests Defer implementation.
	#[test]
	fn test_defer() {
		use crate::brands::RcFnBrand;
		use crate::classes::deferrable::defer;
		use crate::functions::cloneable_fn_new;

		let memo: RcTryLazy<i32, ()> =
			defer::<TryLazyBrand<(), RcLazyConfig>, i32, RcFnBrand>(cloneable_fn_new::<
				RcFnBrand,
				_,
				_,
			>(|_| {
				RcTryLazy::new(|| Ok(42))
			}));
		assert_eq!(memo.evaluate(), Ok(&42));
	}
}
