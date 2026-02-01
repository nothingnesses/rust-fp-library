use fp_macros::doc_params;
use fp_macros::doc_type_params;
use crate::types::{ArcLazyConfig, Lazy, LazyConfig, RcLazyConfig, TryThunk, TryTrampoline};

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
	/// `forall e a. TryLazy a e -> Result a e`
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
	/// `forall e a. (Unit -> Result a e) -> TryLazy a e`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the initializer closure."
	)]	///
	/// ### Parameters
	///
	#[doc_params(
		"The closure that produces the result."
	)]	///
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
	/// `forall a. (Unit -> a) -> TryLazy a String`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the initializer closure."
	)]	///
	/// ### Parameters
	///
	#[doc_params(
		"The closure that might panic."
	)]	///
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
	/// `forall e a. (Unit -> Result a e) -> TryLazy a e`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the initializer closure."
	)]	///
	/// ### Parameters
	///
	#[doc_params(
		"The closure that produces the result."
	)]	///
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
}
