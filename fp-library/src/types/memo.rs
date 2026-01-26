//! Implementations for [`Memo`], a lazily-computed, memoized value.
//!
//! This module provides the [`Memo`] type and its configuration trait [`MemoConfig`].

use std::cell::LazyCell;
use std::rc::Rc;
use std::sync::{Arc, LazyLock};

use crate::types::{Eval, Task, TryMemo};

/// Configuration for memoization strategy.
///
/// This trait bundles together the choices for:
/// - Pointer type (Rc vs Arc)
/// - Lazy cell type (LazyCell vs LazyLock)
///
/// # Note on Standard Library Usage
///
/// This design leverages Rust 1.80's `LazyCell` and `LazyLock` types,
/// which encapsulate the initialization-once logic.
pub trait MemoConfig: 'static {
	/// The lazy cell type for infallible memoization.
	type Lazy<A: 'static>: Clone;

	/// The lazy cell type for fallible memoization.
	type TryLazy<A: 'static, E: 'static>: Clone;

	/// The type of the initializer thunk.
	type Init<A: 'static>: ?Sized;

	/// The type of the fallible initializer thunk.
	type TryInit<A: 'static, E: 'static>: ?Sized;

	/// Creates a new lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Lazy a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `f`: The initializer thunk.
	///
	/// ### Returns
	///
	/// A new lazy cell.
	fn new_lazy<A: 'static>(f: Box<Self::Init<A>>) -> Self::Lazy<A>;

	/// Creates a new fallible lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryLazy a e`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	/// * `E`: The type of the error.
	///
	/// ### Parameters
	///
	/// * `f`: The initializer thunk.
	///
	/// ### Returns
	///
	/// A new fallible lazy cell.
	fn new_try_lazy<A: 'static, E: 'static>(f: Box<Self::TryInit<A, E>>) -> Self::TryLazy<A, E>;

	/// Forces evaluation and returns a reference.
	///
	/// ### Type Signature
	///
	/// `forall a. Lazy a -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `lazy`: The lazy cell to force.
	///
	/// ### Returns
	///
	/// A reference to the value.
	fn force<A: 'static>(lazy: &Self::Lazy<A>) -> &A;

	/// Forces evaluation and returns a reference to the result.
	///
	/// ### Type Signature
	///
	/// `forall e a. TryLazy a e -> Result a e`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	/// * `E`: The type of the error.
	///
	/// ### Parameters
	///
	/// * `lazy`: The fallible lazy cell to force.
	///
	/// ### Returns
	///
	/// A result containing a reference to the value or error.
	fn force_try<A: 'static, E: 'static>(lazy: &Self::TryLazy<A, E>) -> Result<&A, &E>;
}

/// Single-threaded memoization using `Rc<LazyCell>`.
///
/// Not thread-safe. Use [`ArcMemoConfig`] for multi-threaded contexts.
pub struct RcMemoConfig;

impl MemoConfig for RcMemoConfig {
	type Lazy<A: 'static> = Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>;
	type TryLazy<A: 'static, E: 'static> =
		Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E>>>>;
	type Init<A: 'static> = dyn FnOnce() -> A;
	type TryInit<A: 'static, E: 'static> = dyn FnOnce() -> Result<A, E>;

	/// Creates a new lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Lazy a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `f`: The initializer thunk.
	///
	/// ### Returns
	///
	/// A new lazy cell.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = RcMemoConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*RcMemoConfig::force(&lazy), 42);
	/// ```
	fn new_lazy<A: 'static>(f: Box<Self::Init<A>>) -> Self::Lazy<A> {
		Rc::new(LazyCell::new(f))
	}

	/// Creates a new fallible lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryLazy a e`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	/// * `E`: The type of the error.
	///
	/// ### Parameters
	///
	/// * `f`: The initializer thunk.
	///
	/// ### Returns
	///
	/// A new fallible lazy cell.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = RcMemoConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcMemoConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn new_try_lazy<A: 'static, E: 'static>(f: Box<Self::TryInit<A, E>>) -> Self::TryLazy<A, E> {
		Rc::new(LazyCell::new(f))
	}

	/// Forces evaluation and returns a reference.
	///
	/// ### Type Signature
	///
	/// `forall a. Lazy a -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `lazy`: The lazy cell to force.
	///
	/// ### Returns
	///
	/// A reference to the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = RcMemoConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*RcMemoConfig::force(&lazy), 42);
	/// ```
	fn force<A: 'static>(lazy: &Self::Lazy<A>) -> &A {
		LazyCell::force(lazy)
	}

	/// Forces evaluation and returns a reference to the result.
	///
	/// ### Type Signature
	///
	/// `forall e a. TryLazy a e -> Result a e`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	/// * `E`: The type of the error.
	///
	/// ### Parameters
	///
	/// * `lazy`: The fallible lazy cell to force.
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
	/// let lazy = RcMemoConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcMemoConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn force_try<A: 'static, E: 'static>(lazy: &Self::TryLazy<A, E>) -> Result<&A, &E> {
		LazyCell::force(lazy).as_ref()
	}
}

/// Thread-safe memoization using `Arc<LazyLock>`.
///
/// Requires `A: Send + Sync` for the value type.
pub struct ArcMemoConfig;

impl MemoConfig for ArcMemoConfig {
	type Lazy<A: 'static> = Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send>>>;
	type TryLazy<A: 'static, E: 'static> =
		Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send>>>;
	type Init<A: 'static> = dyn FnOnce() -> A + Send;
	type TryInit<A: 'static, E: 'static> = dyn FnOnce() -> Result<A, E> + Send;

	/// Creates a new lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Lazy a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `f`: The initializer thunk.
	///
	/// ### Returns
	///
	/// A new lazy cell.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = ArcMemoConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*ArcMemoConfig::force(&lazy), 42);
	/// ```
	fn new_lazy<A: 'static>(f: Box<Self::Init<A>>) -> Self::Lazy<A> {
		Arc::new(LazyLock::new(f))
	}

	/// Creates a new fallible lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryLazy a e`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	/// * `E`: The type of the error.
	///
	/// ### Parameters
	///
	/// * `f`: The initializer thunk.
	///
	/// ### Returns
	///
	/// A new fallible lazy cell.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = ArcMemoConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(ArcMemoConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn new_try_lazy<A: 'static, E: 'static>(f: Box<Self::TryInit<A, E>>) -> Self::TryLazy<A, E> {
		Arc::new(LazyLock::new(f))
	}

	/// Forces evaluation and returns a reference.
	///
	/// ### Type Signature
	///
	/// `forall a. Lazy a -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `lazy`: The lazy cell to force.
	///
	/// ### Returns
	///
	/// A reference to the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = ArcMemoConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*ArcMemoConfig::force(&lazy), 42);
	/// ```
	fn force<A: 'static>(lazy: &Self::Lazy<A>) -> &A {
		LazyLock::force(lazy)
	}

	/// Forces evaluation and returns a reference to the result.
	///
	/// ### Type Signature
	///
	/// `forall e a. TryLazy a e -> Result a e`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	/// * `E`: The type of the error.
	///
	/// ### Parameters
	///
	/// * `lazy`: The fallible lazy cell to force.
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
	/// let lazy = ArcMemoConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(ArcMemoConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn force_try<A: 'static, E: 'static>(lazy: &Self::TryLazy<A, E>) -> Result<&A, &E> {
		LazyLock::force(lazy).as_ref()
	}
}

/// A lazily-computed, memoized value with shared semantics.
///
/// The computation runs at most once; subsequent accesses return the cached value.
/// Cloning a `Memo` shares the underlying cache - all clones see the same value.
///
/// ### Type Parameters
///
/// * `A`: The type of the computed value.
/// * `Config`: The memoization configuration (determines Rc vs Arc).
///
/// ### Fields
///
/// * `inner`: The internal lazy cell.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let memo = Memo::<_, RcMemoConfig>::new(|| 5);
/// let shared = memo.clone();
///
/// // First force computes and caches:
/// let value = memo.get();
///
/// // Second force returns cached value (shared sees same result):
/// assert_eq!(shared.get(), value);
/// ```
pub struct Memo<A, Config: MemoConfig = RcMemoConfig>
where
	A: 'static,
{
	pub(crate) inner: Config::Lazy<A>,
}

impl<A, Config: MemoConfig> Clone for Memo<A, Config>
where
	A: 'static,
{
	fn clone(&self) -> Self {
		Self { inner: self.inner.clone() }
	}
}

impl<A, Config: MemoConfig> Memo<A, Config>
where
	A: 'static,
{
	/// Gets the memoized value, computing on first access.
	///
	/// ### Type Signature
	///
	/// `forall a. Memo a -> a`
	///
	/// ### Returns
	///
	/// A reference to the memoized value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = Memo::<_, RcMemoConfig>::new(|| 42);
	/// assert_eq!(*memo.get(), 42);
	/// ```
	pub fn get(&self) -> &A {
		Config::force(&self.inner)
	}
}

impl<A> Memo<A, RcMemoConfig>
where
	A: 'static,
{
	/// Creates a new Memo that will run `f` on first access.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Memo a`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the initializer closure.
	///
	/// ### Parameters
	///
	/// * `f`: The closure that produces the value.
	///
	/// ### Returns
	///
	/// A new `Memo` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = Memo::<_, RcMemoConfig>::new(|| 42);
	/// assert_eq!(*memo.get(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'static,
	{
		Memo { inner: RcMemoConfig::new_lazy(Box::new(f)) }
	}

	/// Creates a Memo from an Eval.
	///
	/// ### Type Signature
	///
	/// `forall a. Eval a -> Memo a`
	///
	/// ### Parameters
	///
	/// * `eval`: The `Eval` computation to memoize.
	///
	/// ### Returns
	///
	/// A new `Memo` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::new(|| 42);
	/// let memo = Memo::<_, RcMemoConfig>::from_eval(eval);
	/// assert_eq!(*memo.get(), 42);
	/// ```
	pub fn from_eval(eval: Eval<'static, A>) -> Self {
		Self::new(move || eval.run())
	}

	/// Creates a Memo from a Task.
	///
	/// ### Type Signature
	///
	/// `forall a. Task a -> Memo a`
	///
	/// ### Parameters
	///
	/// * `task`: The `Task` computation to memoize.
	///
	/// ### Returns
	///
	/// A new `Memo` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task = Task::now(42);
	/// let memo = Memo::<_, RcMemoConfig>::from_task(task);
	/// assert_eq!(*memo.get(), 42);
	/// ```
	pub fn from_task(task: Task<A>) -> Self
	where
		A: Send,
	{
		Self::new(move || task.run())
	}

	/// Converts to a TryMemo that always succeeds.
	///
	/// ### Type Signature
	///
	/// `forall e a. Memo a -> TryMemo a e`
	///
	/// ### Type Parameters
	///
	/// * `E`: The error type (which will never occur).
	///
	/// ### Returns
	///
	/// A `TryMemo` that always returns `Ok`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = Memo::<_, RcMemoConfig>::new(|| 42);
	/// let try_memo: TryMemo<i32, (), RcMemoConfig> = memo.into_try();
	/// assert_eq!(try_memo.get(), Ok(&42));
	/// ```
	pub fn into_try<E>(self) -> TryMemo<A, E, RcMemoConfig>
	where
		A: Clone,
		E: 'static,
	{
		TryMemo::<A, E, RcMemoConfig>::new(move || Ok(self.get().clone()))
	}
}

impl<A> Memo<A, ArcMemoConfig>
where
	A: 'static,
{
	/// Creates a new Memo that will run `f` on first access.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Memo a`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the initializer closure.
	///
	/// ### Parameters
	///
	/// * `f`: The closure that produces the value.
	///
	/// ### Returns
	///
	/// A new `Memo` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = Memo::<_, ArcMemoConfig>::new(|| 42);
	/// assert_eq!(*memo.get(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + Send + 'static,
	{
		Memo { inner: ArcMemoConfig::new_lazy(Box::new(f)) }
	}

	/// Converts to a TryMemo that always succeeds.
	///
	/// ### Type Signature
	///
	/// `forall e a. Memo a -> TryMemo a e`
	///
	/// ### Type Parameters
	///
	/// * `E`: The error type (which will never occur).
	///
	/// ### Returns
	///
	/// A `TryMemo` that always returns `Ok`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = Memo::<_, ArcMemoConfig>::new(|| 42);
	/// let try_memo: TryMemo<i32, (), ArcMemoConfig> = memo.into_try();
	/// assert_eq!(try_memo.get(), Ok(&42));
	/// ```
	pub fn into_try<E>(self) -> TryMemo<A, E, ArcMemoConfig>
	where
		A: Clone + Send + Sync,
		E: 'static + Send + Sync,
	{
		TryMemo::<A, E, ArcMemoConfig>::new(move || Ok(self.get().clone()))
	}
}

/// Single-threaded memoization alias.
pub type RcMemo<A> = Memo<A, RcMemoConfig>;

/// Thread-safe memoization alias.
pub type ArcMemo<A> = Memo<A, ArcMemoConfig>;

#[cfg(test)]
mod tests {
	use super::*;
	use std::cell::RefCell;
	use std::rc::Rc;

	/// Tests that `Memo` caches the result of the computation.
	///
	/// Verifies that the initializer is called only once.
	#[test]
	fn test_memo_caching() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo = RcMemo::new(move || {
			*counter_clone.borrow_mut() += 1;
			42
		});

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(*memo.get(), 42);
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(*memo.get(), 42);
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `Memo` shares the cache across clones.
	///
	/// Verifies that clones see the same value and don't recompute.
	#[test]
	fn test_memo_sharing() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo = RcMemo::new(move || {
			*counter_clone.borrow_mut() += 1;
			42
		});
		let shared = memo.clone();

		assert_eq!(*memo.get(), 42);
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(*shared.get(), 42);
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests thread safety of `ArcMemo`.
	///
	/// Verifies that `ArcMemo` can be shared across threads and computes only once.
	#[test]
	fn test_arc_memo_thread_safety() {
		use std::sync::atomic::{AtomicUsize, Ordering};
		use std::thread;

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();
		let memo = ArcMemo::new(move || {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			42
		});

		let mut handles = vec![];
		for _ in 0..10 {
			let memo_clone = memo.clone();
			handles.push(thread::spawn(move || {
				assert_eq!(*memo_clone.get(), 42);
			}));
		}

		for handle in handles {
			handle.join().unwrap();
		}

		assert_eq!(counter.load(Ordering::SeqCst), 1);
	}

	/// Tests creation from `Eval`.
	///
	/// Verifies `from_eval` works correctly.
	#[test]
	fn test_from_eval() {
		let eval = Eval::new(|| 42);
		let memo = RcMemo::from_eval(eval);
		assert_eq!(*memo.get(), 42);
	}

	/// Tests creation from `Task`.
	///
	/// Verifies `from_task` works correctly.
	#[test]
	fn test_from_task() {
		// Task requires Send, so we use a Send-compatible value
		let task = Task::now(42);
		let memo = RcMemo::from_task(task);
		assert_eq!(*memo.get(), 42);
	}

	/// Tests conversion to TryMemo.
	#[test]
	fn test_into_try() {
		let memo = RcMemo::new(|| 42);
		let try_memo: crate::types::RcTryMemo<i32, ()> = memo.into_try();
		assert_eq!(try_memo.get(), Ok(&42));
	}
}
