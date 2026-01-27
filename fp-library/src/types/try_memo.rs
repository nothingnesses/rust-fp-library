//! Implementations for [`TryMemo`], a lazily-computed, memoized fallible value.

use crate::types::{ArcMemoConfig, Memo, MemoConfig, RcMemoConfig, TryEval, TryTask};

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
pub struct TryMemo<'a, A, E, Config: MemoConfig = RcMemoConfig>
where
	A: 'a,
	E: 'a,
{
	pub(crate) inner: Config::TryLazy<'a, A, E>,
}

impl<'a, A, E, Config: MemoConfig> Clone for TryMemo<'a, A, E, Config>
where
	A: 'a,
	E: 'a,
{
	fn clone(&self) -> Self {
		Self { inner: self.inner.clone() }
	}
}

impl<'a, A, E, Config: MemoConfig> TryMemo<'a, A, E, Config>
where
	A: 'a,
	E: 'a,
{
	/// Gets the memoized result, computing on first access.
	///
	/// ### Type Signature
	///
	/// `forall e a. TryMemo a e -> Result a e`
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
	/// let memo = TryMemo::<_, _, RcMemoConfig>::new(|| Ok::<i32, ()>(42));
	/// assert_eq!(memo.get(), Ok(&42));
	/// ```
	pub fn get(&self) -> Result<&A, &E> {
		Config::force_try(&self.inner)
	}
}

impl<'a, A, E> TryMemo<'a, A, E, RcMemoConfig>
where
	A: 'a,
	E: 'a,
{
	/// Creates a new TryMemo that will run `f` on first access.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryMemo a e`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the initializer closure.
	///
	/// ### Parameters
	///
	/// * `f`: The closure that produces the result.
	///
	/// ### Returns
	///
	/// A new `TryMemo` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = TryMemo::<_, _, RcMemoConfig>::new(|| Ok::<i32, ()>(42));
	/// assert_eq!(memo.get(), Ok(&42));
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + 'a,
	{
		TryMemo { inner: RcMemoConfig::new_try_lazy(Box::new(f)) }
	}
}

impl<'a, A, E> From<TryEval<'a, A, E>> for TryMemo<'a, A, E, RcMemoConfig> {
	fn from(eval: TryEval<'a, A, E>) -> Self {
		Self::new(move || eval.run())
	}
}

impl<'a, A, E> From<TryTask<A, E>> for TryMemo<'a, A, E, RcMemoConfig>
where
	A: Send,
	E: Send,
{
	fn from(task: TryTask<A, E>) -> Self {
		Self::new(move || task.run())
	}
}

impl<'a, A, E> From<Memo<'a, A, ArcMemoConfig>> for TryMemo<'a, A, E, ArcMemoConfig>
where
	A: Clone + Send + Sync + 'a,
	E: Send + Sync + 'a,
{
	fn from(memo: Memo<'a, A, ArcMemoConfig>) -> Self {
		Self::new(move || Ok(memo.get().clone()))
	}
}

impl<'a, A, E> From<Memo<'a, A, RcMemoConfig>> for TryMemo<'a, A, E, RcMemoConfig>
where
	A: Clone + 'a,
	E: 'a,
{
	fn from(memo: Memo<'a, A, RcMemoConfig>) -> Self {
		Self::new(move || Ok(memo.get().clone()))
	}
}

impl<'a, A> TryMemo<'a, A, String, RcMemoConfig>
where
	A: 'a,
{
	/// Creates a TryMemo that catches unwinds (panics).
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> TryMemo a String`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the initializer closure.
	///
	/// ### Parameters
	///
	/// * `f`: The closure that might panic.
	///
	/// ### Returns
	///
	/// A new `TryMemo` instance where panics are converted to `Err(String)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = TryMemo::<_, String, RcMemoConfig>::catch_unwind(|| {
	///     if true { panic!("oops") }
	///     42
	/// });
	/// assert_eq!(memo.get(), Err(&"oops".to_string()));
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

impl<'a, A, E> TryMemo<'a, A, E, ArcMemoConfig>
where
	A: 'a,
	E: 'a,
{
	/// Creates a new TryMemo that will run `f` on first access.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryMemo a e`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the initializer closure.
	///
	/// ### Parameters
	///
	/// * `f`: The closure that produces the result.
	///
	/// ### Returns
	///
	/// A new `TryMemo` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = TryMemo::<_, _, ArcMemoConfig>::new(|| Ok::<i32, ()>(42));
	/// assert_eq!(memo.get(), Ok(&42));
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + Send + 'a,
	{
		TryMemo { inner: ArcMemoConfig::new_try_lazy(Box::new(f)) }
	}
}

/// Single-threaded fallible memoization alias.
pub type RcTryMemo<'a, A, E> = TryMemo<'a, A, E, RcMemoConfig>;

/// Thread-safe fallible memoization alias.
pub type ArcTryMemo<'a, A, E> = TryMemo<'a, A, E, ArcMemoConfig>;

#[cfg(test)]
mod tests {
	use crate::types::RcMemo;

	use super::*;
	use std::cell::RefCell;
	use std::rc::Rc;

	/// Tests that `TryMemo` caches successful results.
	///
	/// Verifies that the initializer is called only once for success.
	#[test]
	fn test_try_memo_caching_ok() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo: RcTryMemo<i32, ()> = RcTryMemo::new(move || {
			*counter_clone.borrow_mut() += 1;
			Ok(42)
		});

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(memo.get(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(memo.get(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `TryMemo` caches error results.
	///
	/// Verifies that the initializer is called only once for error.
	#[test]
	fn test_try_memo_caching_err() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo: RcTryMemo<i32, i32> = RcTryMemo::new(move || {
			*counter_clone.borrow_mut() += 1;
			Err(0)
		});

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(memo.get(), Err(&0));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(memo.get(), Err(&0));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `TryMemo` shares the cache across clones.
	///
	/// Verifies that clones see the same result.
	#[test]
	fn test_try_memo_sharing() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo: RcTryMemo<i32, ()> = RcTryMemo::new(move || {
			*counter_clone.borrow_mut() += 1;
			Ok(42)
		});
		let shared = memo.clone();

		assert_eq!(memo.get(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(shared.get(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests `catch_unwind`.
	///
	/// Verifies that panics are caught and converted to errors.
	#[test]
	fn test_catch_unwind() {
		let memo = RcTryMemo::catch_unwind(|| {
			if true {
				panic!("oops")
			}
			42
		});

		match memo.get() {
			Err(e) => assert_eq!(e, "oops"),
			Ok(_) => panic!("Should have failed"),
		}
	}

	/// Tests creation from `TryEval`.
	#[test]
	fn test_try_memo_from_try_eval() {
		let eval = TryEval::new(|| Ok::<i32, ()>(42));
		let memo = RcTryMemo::from(eval);
		assert_eq!(memo.get(), Ok(&42));
	}

	/// Tests creation from `TryTask`.
	#[test]
	fn test_try_memo_from_try_task() {
		let task = TryTask::<i32, ()>::ok(42);
		let memo = RcTryMemo::from(task);
		assert_eq!(memo.get(), Ok(&42));
	}

	/// Tests conversion to TryMemo.
	#[test]
	fn test_try_memo_from_rc_memo() {
		let memo = RcMemo::new(|| 42);
		let try_memo: crate::types::RcTryMemo<i32, ()> = crate::types::RcTryMemo::from(memo);
		assert_eq!(try_memo.get(), Ok(&42));
	}

	/// Tests conversion to ArcTryMemo.
	#[test]
	fn test_try_memo_from_arc_memo() {
		use crate::types::ArcMemo;
		let memo = ArcMemo::new(|| 42);
		let try_memo: crate::types::ArcTryMemo<i32, ()> = crate::types::ArcTryMemo::from(memo);
		assert_eq!(try_memo.get(), Ok(&42));
	}
}
