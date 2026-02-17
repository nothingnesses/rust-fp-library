//! Memoized lazy evaluation with shared cache semantics.
//!
//! Computes a value at most once on first access and caches the result. All clones share the same cache. Available in both single-threaded [`RcLazy`] and thread-safe [`ArcLazy`] variants.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::LazyBrand,
			classes::{Deferrable, RefFunctor, SendDeferrable},
			impl_kind,
			kinds::*,
			types::{Thunk, Trampoline},
		},
		fp_macros::{
			document_fields, document_parameters, document_signature, document_type_parameters,
		},
		std::{
			cell::LazyCell,
			rc::Rc,
			sync::{Arc, LazyLock},
		},
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
		type Lazy<A>: Clone;

		/// The lazy cell type for fallible memoization.
		type TryLazy<A, E>: Clone;

		/// The type of the initializer thunk.
		type Thunk<A>: ?Sized;

		/// The type of the fallible initializer thunk.
		type TryThunk<A, E>: ?Sized;

		/// Creates a new lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters("The type of the value.")]
		///
		#[document_parameters("The initializer thunk.")]
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
		fn lazy_new<A>(f: Box<Self::Thunk<A>>) -> Self::Lazy<A>;

		/// Creates a new fallible lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The initializer thunk.")]
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
		fn try_lazy_new<A, E>(
			f: Box<Self::TryThunk<A, E>>
		) -> Self::TryLazy<A, E>;

		/// Forces evaluation and returns a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the reference.",
			"The type of the value."
		)]
		///
		#[document_parameters("The lazy cell to evaluate.")]
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
		fn evaluate<'b, A>(lazy: &'b Self::Lazy<A>) -> &'b A;

		/// Forces evaluation and returns a reference to the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the reference.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The fallible lazy cell to evaluate.")]
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
		fn try_evaluate<'b, A, E>(
			lazy: &'b Self::TryLazy<A, E>
		) -> Result<&'b A, &'b E>;
	}

	/// Single-threaded memoization using [`Rc<LazyCell>`].
	///
	/// Not thread-safe. Use [`ArcLazyConfig`] for multi-threaded contexts.
	pub struct RcLazyConfig;

	impl LazyConfig for RcLazyConfig {
		type Lazy<A> = Rc<LazyCell<A, Box<dyn FnOnce() -> A + 'static>>>;
		type Thunk<A> = dyn FnOnce() -> A + 'static;
		type TryLazy<A, E> =
			Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + 'static>>>;
		type TryThunk<A, E> = dyn FnOnce() -> Result<A, E> + 'static;

		/// Creates a new lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters("The type of the value.")]
		///
		#[document_parameters("The initializer thunk.")]
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
		fn lazy_new<A>(f: Box<Self::Thunk<A>>) -> Self::Lazy<A> {
			Rc::new(LazyCell::new(f))
		}

		/// Creates a new fallible lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The initializer thunk.")]
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
		fn try_lazy_new<A, E>(
			f: Box<Self::TryThunk<A, E>>
		) -> Self::TryLazy<A, E> {
			Rc::new(LazyCell::new(f))
		}

		/// Forces evaluation and returns a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the reference.",
			"The type of the value."
		)]
		///
		#[document_parameters("The lazy cell to evaluate.")]
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
		fn evaluate<'b, A>(lazy: &'b Self::Lazy<A>) -> &'b A {
			LazyCell::force(lazy)
		}

		/// Forces evaluation and returns a reference to the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the reference.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The fallible lazy cell to evaluate.")]
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
		fn try_evaluate<'b, A, E>(
			lazy: &'b Self::TryLazy<A, E>
		) -> Result<&'b A, &'b E> {
			LazyCell::force(lazy).as_ref()
		}
	}

	/// Thread-safe memoization using [`Arc<LazyLock>`].
	///
	/// Requires `A: Send + Sync` for the value type.
	pub struct ArcLazyConfig;

	impl LazyConfig for ArcLazyConfig {
		type Lazy<A> = Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send + 'static>>>;
		type Thunk<A> = dyn FnOnce() -> A + Send + 'static;
		type TryLazy<A, E> =
			Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send + 'static>>>;
		type TryThunk<A, E> = dyn FnOnce() -> Result<A, E> + Send + 'static;

		/// Creates a new lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters("The type of the value.")]
		///
		#[document_parameters("The initializer thunk.")]
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
		fn lazy_new<A>(f: Box<Self::Thunk<A>>) -> Self::Lazy<A> {
			Arc::new(LazyLock::new(f))
		}

		/// Creates a new fallible lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The initializer thunk.")]
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
		fn try_lazy_new<A, E>(
			f: Box<Self::TryThunk<A, E>>
		) -> Self::TryLazy<A, E> {
			Arc::new(LazyLock::new(f))
		}

		/// Forces evaluation and returns a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the reference.",
			"The type of the value."
		)]
		///
		#[document_parameters("The lazy cell to evaluate.")]
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
		fn evaluate<'b, A>(lazy: &'b Self::Lazy<A>) -> &'b A {
			LazyLock::force(lazy)
		}

		/// Forces evaluation and returns a reference to the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the reference.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The fallible lazy cell to evaluate.")]
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
		fn try_evaluate<'b, A, E>(
			lazy: &'b Self::TryLazy<A, E>
		) -> Result<&'b A, &'b E> {
			LazyLock::force(lazy).as_ref()
		}
	}

	/// A lazily-computed, memoized value with shared semantics.
	///
	/// The computation runs at most once; subsequent accesses return the cached value.
	/// Cloning a `Lazy` shares the underlying cache - all clones see the same value.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is [`LazyBrand<Config>`](crate::brands::LazyBrand),
	/// which is parameterized by the memoization configuration and is polymorphic over the computed value type.
	#[document_type_parameters(
		"The type of the computed value.",
		"The memoization configuration (determines Rc vs Arc)."
	)]
	///
	#[document_fields("The internal lazy cell.")]
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
	pub struct Lazy<A, Config: LazyConfig = RcLazyConfig>(pub(crate) Config::Lazy<A>);

	#[document_type_parameters(
		"The type of the computed value.",
		"The memoization configuration (determines Rc vs Arc)."
	)]
	#[document_parameters("The instance to clone.")]
	impl<A, Config: LazyConfig> Clone for Lazy<A, Config> {
		#[document_signature]
		fn clone(&self) -> Self {
			Self(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The type of the computed value.",
		"The memoization configuration (determines Rc vs Arc)."
	)]
	#[document_parameters("The lazy instance.")]
	impl<A, Config: LazyConfig> Lazy<A, Config> {
		/// Gets the memoized value, computing on first access.
		#[document_signature]
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

	#[document_type_parameters("The type of the computed value.")]
	impl<A> Lazy<A, RcLazyConfig> {
		/// Creates a new Lazy that will run `f` on first access.
		#[document_signature]
		///
		#[document_type_parameters("The type of the initializer closure.")]
		///
		#[document_parameters("The closure that produces the value.")]
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
			F: FnOnce() -> A + 'static,
		{
			Lazy(RcLazyConfig::lazy_new(Box::new(f)))
		}

		/// Creates a `Lazy` from an already-computed value.
		///
		/// The value is immediately available without any computation.
		#[document_signature]
		///
		#[document_parameters("The pre-computed value to wrap.")]
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

	#[document_type_parameters("The type of the computed value.")]
	impl<A> From<Thunk<'static, A>> for Lazy<A, RcLazyConfig> {
		#[document_signature]
		#[document_parameters("The thunk to convert.")]
		fn from(eval: Thunk<'static, A>) -> Self {
			Self::new(move || eval.evaluate())
		}
	}

	#[document_type_parameters("The type of the computed value.")]
	impl<A> From<Trampoline<A>> for Lazy<A, RcLazyConfig>
	where
		A: Send,
	{
		#[document_signature]
		#[document_parameters("The trampoline to convert.")]
		fn from(task: Trampoline<A>) -> Self {
			Self::new(move || task.evaluate())
		}
	}

	#[document_type_parameters("The type of the computed value.")]
	impl<A> Lazy<A, ArcLazyConfig> {
		/// Creates a new Lazy that will run `f` on first access.
		#[document_signature]
		///
		#[document_type_parameters("The type of the initializer closure.")]
		///
		#[document_parameters("The closure that produces the value.")]
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
			F: FnOnce() -> A + Send + 'static,
		{
			Lazy(ArcLazyConfig::lazy_new(Box::new(f)))
		}

		/// Creates a `Lazy` from an already-computed value.
		///
		/// The value is immediately available without any computation.
		/// Requires `Send` since `ArcLazy` is thread-safe.
		#[document_signature]
		///
		#[document_parameters("The pre-computed value to wrap.")]
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
	pub type RcLazy<A> = Lazy<A, RcLazyConfig>;

	/// Thread-safe memoization alias.
	pub type ArcLazy<A> = Lazy<A, ArcLazyConfig>;

	impl_kind! {
		impl<Config: LazyConfig> for LazyBrand<Config> {
			type Of<A> = Lazy<A, Config>;
		}
	}

	#[document_type_parameters("The type of the computed value.")]
	impl<A> Deferrable<'static> for Lazy<A, RcLazyConfig>
	where
		A: Clone,
	{
		/// Defers a computation that produces a `Lazy` value.
		///
		/// This flattens the nested structure: instead of `Lazy<Lazy<A>>`, we get `Lazy<A>`.
		/// The inner `Lazy` is computed only when the outer `Lazy` is evaluated.
		#[document_signature]
		///
		#[document_type_parameters("The type of the thunk.")]
		///
		#[document_parameters("The thunk that produces the lazy value.")]
		///
		/// ### Returns
		///
		/// A new `Lazy` value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = Lazy::<_, RcLazyConfig>::defer(|| RcLazy::pure(42));
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn defer<F>(f: F) -> Self
		where
			F: FnOnce() -> Self + 'static,
			Self: Sized,
		{
			RcLazy::new(move || f().evaluate().clone())
		}
	}

	#[document_type_parameters("The type of the computed value.")]
	impl<A> SendDeferrable<'static> for Lazy<A, ArcLazyConfig>
	where
		A: Clone + Send + Sync,
	{
		/// Defers a computation that produces a thread-safe `Lazy` value.
		///
		/// This flattens the nested structure: instead of `ArcLazy<ArcLazy<A>>`, we get `ArcLazy<A>`.
		/// The inner `Lazy` is computed only when the outer `Lazy` is evaluated.
		#[document_signature]
		///
		#[document_type_parameters("The type of the thunk.")]
		///
		#[document_parameters("The thunk that produces the lazy value.")]
		///
		/// ### Returns
		///
		/// A new `ArcLazy` value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::send_defer(|| ArcLazy::pure(42));
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn send_defer<F>(f: F) -> Self
		where
			F: FnOnce() -> Self + Send + Sync + 'static,
			Self: Sized,
		{
			ArcLazy::new(move || f().evaluate().clone())
		}
	}

	impl RefFunctor for LazyBrand<RcLazyConfig> {
		/// Maps a function over the memoized value, where the function takes a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the value.",
			"The type of the result.",
			"The type of the function."
		)]
		///
		#[document_parameters("The function to apply.", "The memoized value.")]
		///
		/// ### Returns
		///
		/// A new memoized value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
		/// let mapped = LazyBrand::<RcLazyConfig>::ref_map(|x: &i32| *x * 2, memo);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		fn ref_map<A, B, F>(
			f: F,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
		where
			F: FnOnce(&A) -> B + 'static,
		{
			let fa = fa.clone();
			let init: Box<dyn FnOnce() -> B + 'static> = Box::new(move || f(fa.evaluate()));
			Lazy(RcLazyConfig::lazy_new(init))
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::inner::*,
		crate::types::{Thunk, Trampoline},
		std::{cell::RefCell, rc::Rc, sync::Arc},
	};

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
		use std::{
			sync::atomic::{AtomicUsize, Ordering},
			thread,
		};

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
		use crate::classes::deferrable::defer;

		let memo: RcLazy<i32> = defer(|| RcLazy::new(|| 42));
		assert_eq!(*memo.evaluate(), 42);
	}

	/// Tests SendDefer implementation.
	#[test]
	fn test_send_defer() {
		use crate::classes::send_deferrable::send_defer;

		let memo: ArcLazy<i32> = send_defer(|| ArcLazy::new(|| 42));
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
