use crate::{
	Apply,
	brands::LazyBrand,
	classes::{CloneableFn, Defer, RefFunctor, SendDefer},
	impl_kind,
	kinds::*,
	types::{Thunk, Trampoline},
};
use std::{
	cell::LazyCell,
	rc::Rc,
	sync::{Arc, LazyLock},
};

/// Configuration for memoization strategy.
///
/// This trait bundles together the choices for:
/// - Pointer type ([`Rc`] vs [`Arc`]).
/// - Lazy cell type ([`LazyCell`] vs [`LazyLock`]).
///
/// # Note on Standard Library Usage
///
/// This design leverages Rust 1.80's `LazyCell` and `LazyLock` types,
/// which encapsulate the initialization-once logic.
pub trait LazyConfig: 'static {
	/// The lazy cell type for infallible memoization.
	type Lazy<'a, A: 'a>: Clone;

	/// The lazy cell type for fallible memoization.
	type TryLazy<'a, A: 'a, E: 'a>: Clone;

	/// The type of the initializer thunk.
	type Init<'a, A: 'a>: ?Sized;

	/// The type of the fallible initializer thunk.
	type TryInit<'a, A: 'a, E: 'a>: ?Sized;

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
	/// let lazy = RcLazyConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*RcLazyConfig::force(&lazy), 42);
	/// ```
	fn new_lazy<'a, A: 'a>(f: Box<Self::Init<'a, A>>) -> Self::Lazy<'a, A>;

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
	/// let lazy = RcLazyConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcLazyConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn new_try_lazy<'a, A: 'a, E: 'a>(f: Box<Self::TryInit<'a, A, E>>) -> Self::TryLazy<'a, A, E>;

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
	/// let lazy = RcLazyConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*RcLazyConfig::force(&lazy), 42);
	/// ```
	fn force<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A;

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
	/// let lazy = RcLazyConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcLazyConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn force_try<'a, 'b, A: 'a, E: 'a>(lazy: &'b Self::TryLazy<'a, A, E>) -> Result<&'b A, &'b E>;
}

/// Single-threaded memoization using `Rc<LazyCell>`.
///
/// Not thread-safe. Use [`ArcLazyConfig`] for multi-threaded contexts.
pub struct RcLazyConfig;

impl LazyConfig for RcLazyConfig {
	type Lazy<'a, A: 'a> = Rc<LazyCell<A, Box<dyn FnOnce() -> A + 'a>>>;
	type TryLazy<'a, A: 'a, E: 'a> =
		Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + 'a>>>;
	type Init<'a, A: 'a> = dyn FnOnce() -> A + 'a;
	type TryInit<'a, A: 'a, E: 'a> = dyn FnOnce() -> Result<A, E> + 'a;

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
	/// let lazy = RcLazyConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*RcLazyConfig::force(&lazy), 42);
	/// ```
	fn new_lazy<'a, A: 'a>(f: Box<Self::Init<'a, A>>) -> Self::Lazy<'a, A> {
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
	/// let lazy = RcLazyConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcLazyConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn new_try_lazy<'a, A: 'a, E: 'a>(f: Box<Self::TryInit<'a, A, E>>) -> Self::TryLazy<'a, A, E> {
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
	/// let lazy = RcLazyConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*RcLazyConfig::force(&lazy), 42);
	/// ```
	fn force<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A {
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
	/// let lazy = RcLazyConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcLazyConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn force_try<'a, 'b, A: 'a, E: 'a>(lazy: &'b Self::TryLazy<'a, A, E>) -> Result<&'b A, &'b E> {
		LazyCell::force(lazy).as_ref()
	}
}

/// Thread-safe memoization using `Arc<LazyLock>`.
///
/// Requires `A: Send + Sync` for the value type.
pub struct ArcLazyConfig;

impl LazyConfig for ArcLazyConfig {
	type Lazy<'a, A: 'a> = Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send + 'a>>>;
	type TryLazy<'a, A: 'a, E: 'a> =
		Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send + 'a>>>;
	type Init<'a, A: 'a> = dyn FnOnce() -> A + Send + 'a;
	type TryInit<'a, A: 'a, E: 'a> = dyn FnOnce() -> Result<A, E> + Send + 'a;

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
	/// let lazy = ArcLazyConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*ArcLazyConfig::force(&lazy), 42);
	/// ```
	fn new_lazy<'a, A: 'a>(f: Box<Self::Init<'a, A>>) -> Self::Lazy<'a, A> {
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
	/// let lazy = ArcLazyConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(ArcLazyConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn new_try_lazy<'a, A: 'a, E: 'a>(f: Box<Self::TryInit<'a, A, E>>) -> Self::TryLazy<'a, A, E> {
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
	/// let lazy = ArcLazyConfig::new_lazy(Box::new(|| 42));
	/// assert_eq!(*ArcLazyConfig::force(&lazy), 42);
	/// ```
	fn force<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A {
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
	/// let lazy = ArcLazyConfig::new_try_lazy(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(ArcLazyConfig::force_try(&lazy), Ok(&42));
	/// ```
	fn force_try<'a, 'b, A: 'a, E: 'a>(lazy: &'b Self::TryLazy<'a, A, E>) -> Result<&'b A, &'b E> {
		LazyLock::force(lazy).as_ref()
	}
}

/// A lazily-computed, memoized value with shared semantics.
///
/// The computation runs at most once; subsequent accesses return the cached value.
/// Cloning a `Lazy` shares the underlying cache - all clones see the same value.
///
/// ### Type Parameters
///
/// * `A`: The type of the computed value.
/// * `Config`: The memoization configuration (determines Rc vs Arc).
///
/// ### Fields
///
/// * `0`: The internal lazy cell.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let memo = Lazy::<_, RcLazyConfig>::new(|| 5);
/// let shared = memo.clone();
///
/// // First force computes and caches:
/// let value = memo.get();
///
/// // Second force returns cached value (shared sees same result):
/// assert_eq!(shared.get(), value);
/// ```
pub struct Lazy<'a, A, Config: LazyConfig = RcLazyConfig>(pub(crate) Config::Lazy<'a, A>)
where
	A: 'a;

impl<'a, A, Config: LazyConfig> Clone for Lazy<'a, A, Config>
where
	A: 'a,
{
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<'a, A, Config: LazyConfig> Lazy<'a, A, Config>
where
	A: 'a,
{
	/// Gets the memoized value, computing on first access.
	///
	/// ### Type Signature
	///
	/// `forall a. Lazy a -> a`
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
	/// let memo = Lazy::<_, RcLazyConfig>::new(|| 42);
	/// assert_eq!(*memo.get(), 42);
	/// ```
	pub fn get(&self) -> &A {
		Config::force(&self.0)
	}
}

impl<'a, A> Lazy<'a, A, RcLazyConfig>
where
	A: 'a,
{
	/// Creates a new Lazy that will run `f` on first access.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Lazy a`
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
	/// A new `Lazy` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = Lazy::<_, RcLazyConfig>::new(|| 42);
	/// assert_eq!(*memo.get(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'a,
	{
		Lazy(RcLazyConfig::new_lazy(Box::new(f)))
	}
}

impl<'a, A> From<Thunk<'a, A>> for Lazy<'a, A, RcLazyConfig> {
	fn from(eval: Thunk<'a, A>) -> Self {
		Self::new(move || eval.run())
	}
}

impl<'a, A> From<Trampoline<A>> for Lazy<'a, A, RcLazyConfig>
where
	A: Send,
{
	fn from(task: Trampoline<A>) -> Self {
		Self::new(move || task.run())
	}
}

impl<'a, A> Lazy<'a, A, ArcLazyConfig>
where
	A: 'a,
{
	/// Creates a new Lazy that will run `f` on first access.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Lazy a`
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
	/// A new `Lazy` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let memo = Lazy::<_, ArcLazyConfig>::new(|| 42);
	/// assert_eq!(*memo.get(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + Send + 'a,
	{
		Lazy(ArcLazyConfig::new_lazy(Box::new(f)))
	}
}

/// Single-threaded memoization alias.
pub type RcLazy<'a, A> = Lazy<'a, A, RcLazyConfig>;

/// Thread-safe memoization alias.
pub type ArcLazy<'a, A> = Lazy<'a, A, ArcLazyConfig>;

impl_kind! {
	impl<Config: LazyConfig> for LazyBrand<Config> {
		type Of<'a, A: 'a>: 'a = Lazy<'a, A, Config>;
	}
}

impl<'a, A> Defer<'a> for Lazy<'a, A, RcLazyConfig>
where
	A: Clone + 'a,
{
	fn defer<FnBrand: 'a + CloneableFn>(f: <FnBrand as CloneableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized,
	{
		RcLazy::new(move || f(()).get().clone())
	}
}

impl SendDefer for LazyBrand<ArcLazyConfig> {
	fn send_defer<'a, A>(
		thunk: impl 'a
		+ Fn() -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		+ Send
		+ Sync
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		A: Clone + Send + Sync + 'a,
	{
		ArcLazy::new(move || thunk().get().clone())
	}
}

impl RefFunctor for LazyBrand<RcLazyConfig> {
	/// Maps a function over the memoized value, where the function takes a reference.
	///
	/// ### Type Signature
	///
	/// `forall b a. RefFunctor (Lazy RcLazyConfig) => (a -> b, Lazy a RcLazyConfig) -> Lazy b RcLazyConfig`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result.
	/// * `A`: The type of the value.
	/// * `F`: The type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply.
	/// * `fa`: The memoized value.
	///
	/// ### Returns
	///
	/// A new memoized value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::*,
	///     classes::*,
	///     types::*,
	/// };
	///
	/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
	/// let mapped = LazyBrand::<RcLazyConfig>::map_ref(
	///     |x: &i32| *x * 2,
	///     memo,
	/// );
	/// assert_eq!(*mapped.get(), 20);
	/// ```
	fn map_ref<'a, B: 'a, A: 'a, F>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: FnOnce(&A) -> B + 'a,
	{
		let fa = fa.clone();
		let init: Box<dyn FnOnce() -> B + 'a> = Box::new(move || f(fa.get()));
		Lazy(RcLazyConfig::new_lazy(init))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::cell::RefCell;
	use std::rc::Rc;

	/// Tests that `Lazy` caches the result of the computation.
	///
	/// Verifies that the initializer is called only once.
	#[test]
	fn test_memo_caching() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo = RcLazy::new(move || {
			*counter_clone.borrow_mut() += 1;
			42
		});

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(*memo.get(), 42);
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(*memo.get(), 42);
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `Lazy` shares the cache across clones.
	///
	/// Verifies that clones see the same value and don't recompute.
	#[test]
	fn test_memo_sharing() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo = RcLazy::new(move || {
			*counter_clone.borrow_mut() += 1;
			42
		});
		let shared = memo.clone();

		assert_eq!(*memo.get(), 42);
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(*shared.get(), 42);
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests thread safety of `ArcLazy`.
	///
	/// Verifies that `ArcLazy` can be shared across threads and computes only once.
	#[test]
	fn test_arc_memo_thread_safety() {
		use std::sync::atomic::{AtomicUsize, Ordering};
		use std::thread;

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();
		let memo = ArcLazy::new(move || {
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

	/// Tests creation from `Thunk`.
	///
	/// Verifies `From<Thunk>` works correctly.
	#[test]
	fn test_memo_from_eval() {
		let eval = Thunk::new(|| 42);
		let memo = RcLazy::from(eval);
		assert_eq!(*memo.get(), 42);
	}

	/// Tests creation from `Trampoline`.
	///
	/// Verifies `From<Trampoline>` works correctly.
	#[test]
	fn test_memo_from_task() {
		// Trampoline requires Send, so we use a Send-compatible value
		let task = Trampoline::pure(42);
		let memo = RcLazy::from(task);
		assert_eq!(*memo.get(), 42);
	}

	/// Tests Defer implementation.
	#[test]
	fn test_defer() {
		use crate::brands::RcFnBrand;
		use crate::classes::defer::defer;
		use crate::functions::cloneable_fn_new;

		let memo: RcLazy<i32> =
			defer::<RcLazy<i32>, RcFnBrand>(cloneable_fn_new::<RcFnBrand, _, _>(|_| {
				RcLazy::new(|| 42)
			}));
		assert_eq!(*memo.get(), 42);
	}

	/// Tests SendDefer implementation.
	#[test]
	fn test_send_defer() {
		use crate::brands::LazyBrand;
		use crate::classes::send_defer::send_defer;

		let memo: ArcLazy<i32> =
			send_defer::<LazyBrand<ArcLazyConfig>, _, _>(|| ArcLazy::new(|| 42));
		assert_eq!(*memo.get(), 42);
	}
}
