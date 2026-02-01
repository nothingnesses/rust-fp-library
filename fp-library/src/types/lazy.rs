use crate::{
	Apply,
	brands::LazyBrand,
	classes::{CloneableFn, Deferrable, RefFunctor, SendDeferrable},
	impl_kind,
	kinds::*,
	types::{Thunk, Trampoline},
};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;
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
	type Thunk<'a, A: 'a>: ?Sized;

	/// The type of the fallible initializer thunk.
	type TryThunk<'a, A: 'a, E: 'a>: ?Sized;

	/// Creates a new lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Lazy a`
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the computation.", "The type of the value.")]
	///
	/// ### Parameters
	///
	#[doc_params("The initializer thunk.")]
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
	/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
	/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
	/// ```
	fn lazy_new<'a, A: 'a>(f: Box<Self::Thunk<'a, A>>) -> Self::Lazy<'a, A>;

	/// Creates a new fallible lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryLazy a e`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the value.",
		"The type of the error."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The initializer thunk.")]
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
	/// let lazy = RcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcLazyConfig::try_evaluate(&lazy), Ok(&42));
	/// ```
	fn try_lazy_new<'a, A: 'a, E: 'a>(f: Box<Self::TryThunk<'a, A, E>>) -> Self::TryLazy<'a, A, E>;

	/// Forces evaluation and returns a reference.
	///
	/// ### Type Signature
	///
	/// `forall a. Lazy a -> a`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The lifetime of the reference.",
		"The type of the value."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The lazy cell to evaluate.")]
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
	/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
	/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
	/// ```
	fn evaluate<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A;

	/// Forces evaluation and returns a reference to the result.
	///
	/// ### Type Signature
	///
	/// `forall e a. TryLazy a e -> Result a e`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The lifetime of the reference.",
		"The type of the value.",
		"The type of the error."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The fallible lazy cell to evaluate.")]
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
	/// let lazy = RcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcLazyConfig::try_evaluate(&lazy), Ok(&42));
	/// ```
	fn try_evaluate<'a, 'b, A: 'a, E: 'a>(
		lazy: &'b Self::TryLazy<'a, A, E>
	) -> Result<&'b A, &'b E>;
}

/// Single-threaded memoization using [`Rc<LazyCell>`].
///
/// Not thread-safe. Use [`ArcLazyConfig`] for multi-threaded contexts.
pub struct RcLazyConfig;

impl LazyConfig for RcLazyConfig {
	type Lazy<'a, A: 'a> = Rc<LazyCell<A, Box<dyn FnOnce() -> A + 'a>>>;
	type TryLazy<'a, A: 'a, E: 'a> =
		Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + 'a>>>;
	type Thunk<'a, A: 'a> = dyn FnOnce() -> A + 'a;
	type TryThunk<'a, A: 'a, E: 'a> = dyn FnOnce() -> Result<A, E> + 'a;

	/// Creates a new lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Lazy a`
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the computation.", "The type of the value.")]
	///
	/// ### Parameters
	///
	#[doc_params("The initializer thunk.")]
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
	/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
	/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
	/// ```
	fn lazy_new<'a, A: 'a>(f: Box<Self::Thunk<'a, A>>) -> Self::Lazy<'a, A> {
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
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the value.",
		"The type of the error."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The initializer thunk.")]
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
	/// let lazy = RcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcLazyConfig::try_evaluate(&lazy), Ok(&42));
	/// ```
	fn try_lazy_new<'a, A: 'a, E: 'a>(f: Box<Self::TryThunk<'a, A, E>>) -> Self::TryLazy<'a, A, E> {
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
	#[doc_type_params(
		"The lifetime of the computation.",
		"The lifetime of the reference.",
		"The type of the value."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The lazy cell to evaluate.")]
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
	/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
	/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
	/// ```
	fn evaluate<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A {
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
	#[doc_type_params(
		"The lifetime of the computation.",
		"The lifetime of the reference.",
		"The type of the value.",
		"The type of the error."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The fallible lazy cell to evaluate.")]
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
	/// let lazy = RcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(RcLazyConfig::try_evaluate(&lazy), Ok(&42));
	/// ```
	fn try_evaluate<'a, 'b, A: 'a, E: 'a>(
		lazy: &'b Self::TryLazy<'a, A, E>
	) -> Result<&'b A, &'b E> {
		LazyCell::force(lazy).as_ref()
	}
}

/// Thread-safe memoization using [`Arc<LazyLock>`].
///
/// Requires `A: Send + Sync` for the value type.
pub struct ArcLazyConfig;

impl LazyConfig for ArcLazyConfig {
	type Lazy<'a, A: 'a> = Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send + 'a>>>;
	type TryLazy<'a, A: 'a, E: 'a> =
		Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send + 'a>>>;
	type Thunk<'a, A: 'a> = dyn FnOnce() -> A + Send + 'a;
	type TryThunk<'a, A: 'a, E: 'a> = dyn FnOnce() -> Result<A, E> + Send + 'a;

	/// Creates a new lazy cell from an initializer.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Lazy a`
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the computation.", "The type of the value.")]
	///
	/// ### Parameters
	///
	#[doc_params("The initializer thunk.")]
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
	/// let lazy = ArcLazyConfig::lazy_new(Box::new(|| 42));
	/// assert_eq!(*ArcLazyConfig::evaluate(&lazy), 42);
	/// ```
	fn lazy_new<'a, A: 'a>(f: Box<Self::Thunk<'a, A>>) -> Self::Lazy<'a, A> {
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
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the value.",
		"The type of the error."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The initializer thunk.")]
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
	/// let lazy = ArcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(ArcLazyConfig::try_evaluate(&lazy), Ok(&42));
	/// ```
	fn try_lazy_new<'a, A: 'a, E: 'a>(f: Box<Self::TryThunk<'a, A, E>>) -> Self::TryLazy<'a, A, E> {
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
	#[doc_type_params(
		"The lifetime of the computation.",
		"The lifetime of the reference.",
		"The type of the value."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The lazy cell to evaluate.")]
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
	/// let lazy = ArcLazyConfig::lazy_new(Box::new(|| 42));
	/// assert_eq!(*ArcLazyConfig::evaluate(&lazy), 42);
	/// ```
	fn evaluate<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A {
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
	#[doc_type_params(
		"The lifetime of the computation.",
		"The lifetime of the reference.",
		"The type of the value.",
		"The type of the error."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The fallible lazy cell to evaluate.")]
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
	/// let lazy = ArcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
	/// assert_eq!(ArcLazyConfig::try_evaluate(&lazy), Ok(&42));
	/// ```
	fn try_evaluate<'a, 'b, A: 'a, E: 'a>(
		lazy: &'b Self::TryLazy<'a, A, E>
	) -> Result<&'b A, &'b E> {
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
/// let value = memo.evaluate();
///
/// // Second force returns cached value (shared sees same result):
/// assert_eq!(shared.evaluate(), value);
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
	/// assert_eq!(*memo.evaluate(), 42);
	/// ```
	pub fn evaluate(&self) -> &A {
		Config::evaluate(&self.0)
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
	#[doc_type_params("The type of the initializer closure.")]
	///
	/// ### Parameters
	///
	#[doc_params("The closure that produces the value.")]
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
	/// assert_eq!(*memo.evaluate(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'a,
	{
		Lazy(RcLazyConfig::lazy_new(Box::new(f)))
	}

	/// Creates a `Lazy` from an already-computed value.
	///
	/// The value is immediately available without any computation.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Lazy a`
	///
	/// ### Parameters
	///
	#[doc_params("The pre-computed value to wrap.")]
	///
	/// ### Returns
	///
	/// A new `Lazy` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
	/// assert_eq!(*lazy.evaluate(), 42);
	/// ```
	pub fn pure(a: A) -> Self {
		Lazy(RcLazyConfig::lazy_new(Box::new(move || a)))
	}
}

impl<'a, A> From<Thunk<'a, A>> for Lazy<'a, A, RcLazyConfig> {
	fn from(eval: Thunk<'a, A>) -> Self {
		Self::new(move || eval.evaluate())
	}
}

impl<'a, A> From<Trampoline<A>> for Lazy<'a, A, RcLazyConfig>
where
	A: Send,
{
	fn from(task: Trampoline<A>) -> Self {
		Self::new(move || task.evaluate())
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
	#[doc_type_params("The type of the initializer closure.")]
	///
	/// ### Parameters
	///
	#[doc_params("The closure that produces the value.")]
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
	/// let lazy = Lazy::<_, ArcLazyConfig>::new(|| 42);
	/// assert_eq!(*lazy.evaluate(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + Send + 'a,
	{
		Lazy(ArcLazyConfig::lazy_new(Box::new(f)))
	}

	/// Creates a `Lazy` from an already-computed value.
	///
	/// The value is immediately available without any computation.
	/// Requires `Send` since `ArcLazy` is thread-safe.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Lazy a`
	///
	/// ### Parameters
	///
	#[doc_params("The pre-computed value to wrap.")]
	///
	/// ### Returns
	///
	/// A new `Lazy` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = Lazy::<_, ArcLazyConfig>::pure(42);
	/// assert_eq!(*lazy.evaluate(), 42);
	/// ```
	pub fn pure(a: A) -> Self
	where
		A: Send,
	{
		Lazy(ArcLazyConfig::lazy_new(Box::new(move || a)))
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

impl<'a, A> Deferrable<'a> for Lazy<'a, A, RcLazyConfig>
where
	A: Clone + 'a,
{
	fn defer<FnBrand: 'a + CloneableFn>(f: <FnBrand as CloneableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized,
	{
		RcLazy::new(move || f(()).evaluate().clone())
	}
}

impl SendDeferrable for LazyBrand<ArcLazyConfig> {
	fn send_defer<'a, A>(
		thunk: impl 'a
		+ Fn() -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		+ Send
		+ Sync
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		A: Clone + Send + Sync + 'a,
	{
		ArcLazy::new(move || thunk().evaluate().clone())
	}
}

impl RefFunctor for LazyBrand<RcLazyConfig> {
	/// Maps a function over the memoized value, where the function takes a reference.
	///
	/// ### Type Signature
	///
	#[hm_signature(RefFunctor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the value.",
		"The type of the result.",
		"The type of the function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply.", "The memoized value.")]
	///
	/// ### Returns
	///
	/// A new memoized value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
	/// let mapped = LazyBrand::<RcLazyConfig>::ref_map(
	///     |x: &i32| *x * 2,
	///     memo,
	/// );
	/// assert_eq!(*mapped.evaluate(), 20);
	/// ```
	fn ref_map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: FnOnce(&A) -> B + 'a,
	{
		let fa = fa.clone();
		let init: Box<dyn FnOnce() -> B + 'a> = Box::new(move || f(fa.evaluate()));
		Lazy(RcLazyConfig::lazy_new(init))
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
		assert_eq!(*memo.evaluate(), 42);
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(*memo.evaluate(), 42);
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

		assert_eq!(*memo.evaluate(), 42);
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(*shared.evaluate(), 42);
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
				assert_eq!(*memo_clone.evaluate(), 42);
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
		assert_eq!(*memo.evaluate(), 42);
	}

	/// Tests creation from `Trampoline`.
	///
	/// Verifies `From<Trampoline>` works correctly.
	#[test]
	fn test_memo_from_task() {
		// Trampoline requires Send, so we use a Send-compatible value
		let task = Trampoline::pure(42);
		let memo = RcLazy::from(task);
		assert_eq!(*memo.evaluate(), 42);
	}

	/// Tests Defer implementation.
	#[test]
	fn test_defer() {
		use crate::brands::RcFnBrand;
		use crate::classes::deferrable::defer;
		use crate::functions::cloneable_fn_new;

		let memo: RcLazy<i32> =
			defer::<RcLazy<i32>, RcFnBrand>(cloneable_fn_new::<RcFnBrand, _, _>(|_| {
				RcLazy::new(|| 42)
			}));
		assert_eq!(*memo.evaluate(), 42);
	}

	/// Tests SendDefer implementation.
	#[test]
	fn test_send_defer() {
		use crate::brands::LazyBrand;
		use crate::classes::send_deferrable::send_defer;

		let memo: ArcLazy<i32> =
			send_defer::<LazyBrand<ArcLazyConfig>, _, _>(|| ArcLazy::new(|| 42));
		assert_eq!(*memo.evaluate(), 42);
	}

	/// Tests `RcLazy::pure`.
	///
	/// Verifies that `pure` creates a lazy value from a pre-computed value.
	#[test]
	fn test_rc_lazy_pure() {
		let lazy = RcLazy::pure(42);
		assert_eq!(*lazy.evaluate(), 42);

		// Verify it's still lazy (shares cache)
		let shared = lazy.clone();
		assert_eq!(*shared.evaluate(), 42);
	}

	/// Tests `ArcLazy::pure`.
	///
	/// Verifies that `pure` creates a thread-safe lazy value from a pre-computed value.
	#[test]
	fn test_arc_lazy_pure() {
		let lazy = ArcLazy::pure(42);
		assert_eq!(*lazy.evaluate(), 42);

		// Verify it's still lazy (shares cache)
		let shared = lazy.clone();
		assert_eq!(*shared.evaluate(), 42);
	}

	/// Tests `ArcLazy::pure` with thread safety.
	///
	/// Verifies that `pure` works across threads.
	#[test]
	fn test_arc_lazy_pure_thread_safety() {
		use std::thread;

		let lazy = ArcLazy::pure(42);

		let mut handles = vec![];
		for _ in 0..10 {
			let lazy_clone = lazy.clone();
			handles.push(thread::spawn(move || {
				assert_eq!(*lazy_clone.evaluate(), 42);
			}));
		}

		for handle in handles {
			handle.join().unwrap();
		}
	}
}
