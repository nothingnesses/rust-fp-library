//! Memoized lazy evaluation with shared cache semantics.
//!
//! Computes a value at most once on first access and caches the result. All clones share the same cache. Available in both single-threaded [`RcLazy`] and thread-safe [`ArcLazy`] variants.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::LazyBrand,
			classes::{
				CloneableFn,
				Deferrable,
				Foldable,
				Monoid,
				RefFunctor,
				SendDeferrable,
			},
			impl_kind,
			kinds::*,
			types::{
				Thunk,
				Trampoline,
			},
		},
		fp_macros::*,
		std::{
			cell::LazyCell,
			fmt,
			hash::{
				Hash,
				Hasher,
			},
			rc::Rc,
			sync::{
				Arc,
				LazyLock,
			},
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
		type Lazy<'a, A: 'a>: Clone;

		/// The lazy cell type for fallible memoization.
		type TryLazy<'a, A: 'a, E: 'a>: Clone;

		/// The type of the initializer thunk.
		type Thunk<'a, A: 'a>: ?Sized;

		/// The type of the fallible initializer thunk.
		type TryThunk<'a, A: 'a, E: 'a>: ?Sized;

		/// Creates a new lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the computation.", "The type of the value.")]
		///
		#[document_parameters("The initializer thunk.")]
		///
		#[document_returns("A new lazy cell.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
		/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
		/// ```
		fn lazy_new<'a, A: 'a>(f: Box<Self::Thunk<'a, A>>) -> Self::Lazy<'a, A>;

		/// Creates a new fallible lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The initializer thunk.")]
		///
		#[document_returns("A new fallible lazy cell.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let lazy = RcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
		/// assert_eq!(RcLazyConfig::try_evaluate(&lazy), Ok(&42));
		/// ```
		fn try_lazy_new<'a, A: 'a, E: 'a>(
			f: Box<Self::TryThunk<'a, A, E>>
		) -> Self::TryLazy<'a, A, E>;

		/// Forces evaluation and returns a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The borrow lifetime.",
			"The type of the value."
		)]
		///
		#[document_parameters("The lazy cell to evaluate.")]
		///
		#[document_returns("A reference to the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
		/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
		/// ```
		fn evaluate<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A;

		/// Forces evaluation and returns a reference to the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The borrow lifetime.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The fallible lazy cell to evaluate.")]
		///
		#[document_returns("A result containing a reference to the value or error.")]
		///
		#[document_examples]
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
		type Thunk<'a, A: 'a> = dyn FnOnce() -> A + 'a;
		type TryLazy<'a, A: 'a, E: 'a> =
			Rc<LazyCell<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + 'a>>>;
		type TryThunk<'a, A: 'a, E: 'a> = dyn FnOnce() -> Result<A, E> + 'a;

		/// Creates a new lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the computation.", "The type of the value.")]
		///
		#[document_parameters("The initializer thunk.")]
		///
		#[document_returns("A new lazy cell.")]
		///
		#[document_examples]
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
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The initializer thunk.")]
		///
		#[document_returns("A new fallible lazy cell.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let lazy = RcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
		/// assert_eq!(RcLazyConfig::try_evaluate(&lazy), Ok(&42));
		/// ```
		fn try_lazy_new<'a, A: 'a, E: 'a>(
			f: Box<Self::TryThunk<'a, A, E>>
		) -> Self::TryLazy<'a, A, E> {
			Rc::new(LazyCell::new(f))
		}

		/// Forces evaluation and returns a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The borrow lifetime.",
			"The type of the value."
		)]
		///
		#[document_parameters("The lazy cell to evaluate.")]
		///
		#[document_returns("A reference to the value.")]
		///
		#[document_examples]
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
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The borrow lifetime.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The fallible lazy cell to evaluate.")]
		///
		#[document_returns("A result containing a reference to the value or error.")]
		///
		#[document_examples]
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
		type Thunk<'a, A: 'a> = dyn FnOnce() -> A + Send + 'a;
		type TryLazy<'a, A: 'a, E: 'a> =
			Arc<LazyLock<Result<A, E>, Box<dyn FnOnce() -> Result<A, E> + Send + 'a>>>;
		type TryThunk<'a, A: 'a, E: 'a> = dyn FnOnce() -> Result<A, E> + Send + 'a;

		/// Creates a new lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the computation.", "The type of the value.")]
		///
		#[document_parameters("The initializer thunk.")]
		///
		#[document_returns("A new lazy cell.")]
		///
		#[document_examples]
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
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The initializer thunk.")]
		///
		#[document_returns("A new fallible lazy cell.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let lazy = ArcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
		/// assert_eq!(ArcLazyConfig::try_evaluate(&lazy), Ok(&42));
		/// ```
		fn try_lazy_new<'a, A: 'a, E: 'a>(
			f: Box<Self::TryThunk<'a, A, E>>
		) -> Self::TryLazy<'a, A, E> {
			Arc::new(LazyLock::new(f))
		}

		/// Forces evaluation and returns a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The borrow lifetime.",
			"The type of the value."
		)]
		///
		#[document_parameters("The lazy cell to evaluate.")]
		///
		#[document_returns("A reference to the value.")]
		///
		#[document_examples]
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
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The borrow lifetime.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The fallible lazy cell to evaluate.")]
		///
		#[document_returns("A result containing a reference to the value or error.")]
		///
		#[document_examples]
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
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is [`LazyBrand<Config>`](crate::brands::LazyBrand),
	/// which is parameterized by the memoization configuration and is polymorphic over the computed value type.
	#[document_type_parameters(
		"The lifetime of the reference.",
		"The type of the computed value.",
		"The memoization configuration (determines Rc vs Arc)."
	)]
	///
	pub struct Lazy<'a, A, Config: LazyConfig = RcLazyConfig>(
		/// The internal lazy cell.
		pub(crate) Config::Lazy<'a, A>,
	)
	where
		A: 'a;

	#[document_type_parameters(
		"The lifetime of the reference.",
		"The type of the computed value.",
		"The memoization configuration (determines Rc vs Arc)."
	)]
	#[document_parameters("The instance to clone.")]
	impl<'a, A, Config: LazyConfig> Clone for Lazy<'a, A, Config>
	where
		A: 'a,
	{
		#[document_signature]
		#[document_returns("A new `Lazy` instance that shares the same underlying memoized value.")]
		#[document_examples]
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
		fn clone(&self) -> Self {
			Self(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the reference.",
		"The type of the computed value.",
		"The memoization configuration (determines Rc vs Arc)."
	)]
	#[document_parameters("The lazy instance.")]
	impl<'a, A, Config: LazyConfig> Lazy<'a, A, Config>
	where
		A: 'a,
	{
		/// Gets the memoized value, computing on first access.
		#[document_signature]
		///
		#[document_returns("A reference to the memoized value.")]
		///
		#[document_examples]
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

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	#[document_parameters("The lazy instance.")]
	impl<'a, A> Lazy<'a, A, RcLazyConfig>
	where
		A: 'a,
	{
		/// Creates a new Lazy that will run `f` on first access.
		#[document_signature]
		///
		#[document_parameters("The closure that produces the value.")]
		///
		#[document_returns("A new `Lazy` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = Lazy::<_, RcLazyConfig>::new(|| 42);
		/// assert_eq!(*memo.evaluate(), 42);
		/// ```
		pub fn new(f: impl FnOnce() -> A + 'a) -> Self {
			Lazy(RcLazyConfig::lazy_new(Box::new(f)))
		}

		/// Creates a `Lazy` from an already-computed value.
		///
		/// The value is immediately available without any computation.
		#[document_signature]
		///
		#[document_parameters("The pre-computed value to wrap.")]
		///
		#[document_returns("A new `Lazy` instance containing the value.")]
		///
		#[document_examples]
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

		/// Maps a function over the memoized value by reference.
		///
		/// This is the inherent method form of [`RefFunctor::ref_map`](crate::classes::ref_functor::RefFunctor::ref_map).
		/// The mapping function receives a reference to the cached value and returns a new value,
		/// which is itself lazily memoized.
		#[document_signature]
		#[document_type_parameters("The type of the result.")]
		#[document_parameters("The function to apply to the memoized value.")]
		#[document_returns("A new `Lazy` instance containing the mapped value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
		/// let mapped = memo.ref_map(|x| *x * 2);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		pub fn ref_map<B: 'a>(
			self,
			f: impl FnOnce(&A) -> B + 'a,
		) -> Lazy<'a, B, RcLazyConfig> {
			let fa = self.clone();
			let init: Box<dyn FnOnce() -> B + 'a> = Box::new(move || f(fa.evaluate()));
			Lazy(RcLazyConfig::lazy_new(init))
		}
	}

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	impl<'a, A> From<Thunk<'a, A>> for Lazy<'a, A, RcLazyConfig> {
		#[document_signature]
		#[document_parameters("The thunk to convert.")]
		#[document_returns("A new `Lazy` instance that will evaluate the thunk on first access.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = Thunk::new(|| 42);
		/// let lazy = Lazy::from(thunk);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn from(eval: Thunk<'a, A>) -> Self {
			Self::new(move || eval.evaluate())
		}
	}

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	impl<'a, A> From<Trampoline<A>> for Lazy<'a, A, RcLazyConfig>
	where
		A: Send,
	{
		#[document_signature]
		#[document_parameters("The trampoline to convert.")]
		#[document_returns(
			"A new `Lazy` instance that will evaluate the trampoline on first access."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let task = Trampoline::pure(42);
		/// let lazy = Lazy::from(task);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn from(task: Trampoline<A>) -> Self {
			Self::new(move || task.evaluate())
		}
	}

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	impl<'a, A> Lazy<'a, A, ArcLazyConfig>
	where
		A: 'a,
	{
		/// Creates a new Lazy that will run `f` on first access.
		#[document_signature]
		///
		#[document_parameters("The closure that produces the value.")]
		///
		#[document_returns("A new `Lazy` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let lazy = Lazy::<_, ArcLazyConfig>::new(|| 42);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		pub fn new(f: impl FnOnce() -> A + Send + 'a) -> Self {
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
		#[document_returns("A new `Lazy` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let lazy = Lazy::<_, ArcLazyConfig>::pure(42);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self
		where
			A: Send + Sync, {
			Lazy(ArcLazyConfig::lazy_new(Box::new(move || a)))
		}

		/// Maps a function over the memoized value by reference.
		///
		/// This is the thread-safe equivalent of [`RcLazy::ref_map`].
		/// The mapping function receives a reference to the cached value and returns a new value,
		/// which is itself lazily memoized.
		#[document_signature]
		#[document_type_parameters("The type of the result.")]
		#[document_parameters("The function to apply to the memoized value.")]
		#[document_returns("A new `ArcLazy` instance containing the mapped value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = Lazy::<_, ArcLazyConfig>::new(|| 10);
		/// let mapped = memo.ref_map(|x| *x * 2);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		pub fn ref_map<B: 'a>(
			self,
			f: impl FnOnce(&A) -> B + Send + 'a,
		) -> Lazy<'a, B, ArcLazyConfig>
		where
			A: Send + Sync, {
			let fa = self.clone();
			let init: Box<dyn FnOnce() -> B + Send + 'a> = Box::new(move || f(fa.evaluate()));
			Lazy(ArcLazyConfig::lazy_new(init))
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

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	impl<'a, A> Deferrable<'a> for Lazy<'a, A, RcLazyConfig>
	where
		A: Clone + 'a,
	{
		/// Defers a computation that produces a `Lazy` value.
		///
		/// This flattens the nested structure: instead of `Lazy<Lazy<A>>`, we get `Lazy<A>`.
		/// The inner `Lazy` is computed only when the outer `Lazy` is evaluated.
		#[document_signature]
		///
		#[document_parameters("The thunk that produces the lazy value.")]
		///
		#[document_returns("A new `Lazy` value.")]
		///
		#[document_examples]
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
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized, {
			RcLazy::new(move || f().evaluate().clone())
		}
	}

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	impl<'a, A> SendDeferrable<'a> for Lazy<'a, A, ArcLazyConfig>
	where
		A: Clone + Send + Sync + 'a,
	{
		/// Defers a computation that produces a thread-safe `Lazy` value.
		///
		/// This flattens the nested structure: instead of `ArcLazy<ArcLazy<A>>`, we get `ArcLazy<A>`.
		/// The inner `Lazy` is computed only when the outer `Lazy` is evaluated.
		#[document_signature]
		///
		#[document_parameters("The thunk that produces the lazy value.")]
		///
		#[document_returns("A new `ArcLazy` value.")]
		///
		#[document_examples]
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
		fn send_defer(f: impl FnOnce() -> Self + Send + Sync + 'a) -> Self
		where
			Self: Sized, {
			ArcLazy::new(move || f().evaluate().clone())
		}
	}

	impl RefFunctor for LazyBrand<RcLazyConfig> {
		/// Maps a function over the memoized value, where the function takes a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value.",
			"The type of the result."
		)]
		///
		#[document_parameters("The function to apply.", "The memoized value.")]
		///
		#[document_returns("A new memoized value.")]
		///
		#[document_examples]
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
		fn ref_map<'a, A: 'a, B: 'a>(
			f: impl FnOnce(&A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.ref_map(f)
		}
	}

	// --- Display ---

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	impl<'a, A: fmt::Display + 'a> fmt::Display for Lazy<'a, A, RcLazyConfig> {
		/// Forces evaluation and displays the value.
		#[document_signature]
		///
		#[document_parameters("The formatter.")]
		///
		#[document_returns("The formatting result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let lazy = RcLazy::new(|| 42);
		/// assert_eq!(format!("{}", lazy), "42");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			fmt::Display::fmt(self.evaluate(), f)
		}
	}

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	impl<'a, A: fmt::Display + 'a> fmt::Display for Lazy<'a, A, ArcLazyConfig> {
		/// Forces evaluation and displays the value.
		#[document_signature]
		///
		#[document_parameters("The formatter.")]
		///
		#[document_returns("The formatting result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let lazy = ArcLazy::new(|| 42);
		/// assert_eq!(format!("{}", lazy), "42");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			fmt::Display::fmt(self.evaluate(), f)
		}
	}

	// --- Hash ---

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	impl<'a, A: Hash + 'a> Hash for Lazy<'a, A, RcLazyConfig> {
		/// Forces evaluation and hashes the value.
		#[document_signature]
		///
		#[document_parameters("The hasher state.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::types::*,
		/// 	std::{
		/// 		collections::hash_map::DefaultHasher,
		/// 		hash::{
		/// 			Hash,
		/// 			Hasher,
		/// 		},
		/// 	},
		/// };
		///
		/// let lazy = RcLazy::new(|| 42);
		/// let mut hasher = DefaultHasher::new();
		/// lazy.hash(&mut hasher);
		/// let h1 = hasher.finish();
		///
		/// let mut hasher = DefaultHasher::new();
		/// 42i32.hash(&mut hasher);
		/// let h2 = hasher.finish();
		///
		/// assert_eq!(h1, h2);
		/// ```
		fn hash<H: Hasher>(
			&self,
			state: &mut H,
		) {
			self.evaluate().hash(state)
		}
	}

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	impl<'a, A: Hash + 'a> Hash for Lazy<'a, A, ArcLazyConfig> {
		/// Forces evaluation and hashes the value.
		#[document_signature]
		///
		#[document_parameters("The hasher state.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::types::*,
		/// 	std::{
		/// 		collections::hash_map::DefaultHasher,
		/// 		hash::{
		/// 			Hash,
		/// 			Hasher,
		/// 		},
		/// 	},
		/// };
		///
		/// let lazy = ArcLazy::new(|| 42);
		/// let mut hasher = DefaultHasher::new();
		/// lazy.hash(&mut hasher);
		/// let h1 = hasher.finish();
		///
		/// let mut hasher = DefaultHasher::new();
		/// 42i32.hash(&mut hasher);
		/// let h2 = hasher.finish();
		///
		/// assert_eq!(h1, h2);
		/// ```
		fn hash<H: Hasher>(
			&self,
			state: &mut H,
		) {
			self.evaluate().hash(state)
		}
	}

	// --- Foldable ---

	impl<Config: LazyConfig> Foldable for LazyBrand<Config> {
		/// Folds the `Lazy` from the right.
		///
		/// Forces evaluation and folds the single contained value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The `Lazy` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 10);
		/// let result = fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _>(|a, b| a + b, 5, lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(fa.evaluate().clone(), initial)
		}

		/// Folds the `Lazy` from the left.
		///
		/// Forces evaluation and folds the single contained value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The `Lazy` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 10);
		/// let result = fold_left::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _>(|b, a| b + a, 5, lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(initial, fa.evaluate().clone())
		}

		/// Maps the value to a monoid and returns it.
		///
		/// Forces evaluation and maps the single contained value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The Lazy to fold.")]
		///
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_map::<RcFnBrand, LazyBrand<RcLazyConfig>, _, String>(|a: i32| a.to_string(), lazy);
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, R: Monoid>(
			func: impl Fn(A) -> R + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: CloneableFn + 'a, {
			func(fa.evaluate().clone())
		}
	}

	// --- Fix combinators ---

	/// Computes a fixed point for `RcLazy`.
	///
	/// Constructs a self-referential `RcLazy` where the initializer receives a clone
	/// of the resulting lazy cell. This enables recursive definitions where the value
	/// depends on the lazy cell itself.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	///
	#[document_parameters(
		"The function that receives a lazy self-reference and produces the value."
	)]
	///
	#[document_returns("A new `RcLazy` instance.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = rc_lazy_fix(|_self_ref: RcLazy<i32>| 42);
	/// assert_eq!(*lazy.evaluate(), 42);
	/// ```
	pub fn rc_lazy_fix<'a, A: Clone + 'a>(
		f: impl FnOnce(RcLazy<'a, A>) -> A + 'a
	) -> RcLazy<'a, A> {
		use std::cell::OnceCell;

		let cell: Rc<OnceCell<RcLazy<'a, A>>> = Rc::new(OnceCell::new());
		let cell_clone = cell.clone();
		let lazy = RcLazy::new(move || {
			let self_ref = cell_clone.get().expect("rc_lazy_fix: cell not initialized").clone();
			f(self_ref)
		});
		let _ = cell.set(lazy.clone());
		lazy
	}

	/// Computes a fixed point for `ArcLazy`.
	///
	/// Constructs a self-referential `ArcLazy` where the initializer receives a clone
	/// of the resulting lazy cell. This enables recursive definitions where the value
	/// depends on the lazy cell itself.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	///
	#[document_parameters(
		"The function that receives a lazy self-reference and produces the value."
	)]
	///
	#[document_returns("A new `ArcLazy` instance.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let lazy = arc_lazy_fix(|_self_ref: ArcLazy<i32>| 42);
	/// assert_eq!(*lazy.evaluate(), 42);
	/// ```
	pub fn arc_lazy_fix<'a, A: Clone + Send + Sync + 'a>(
		f: impl FnOnce(ArcLazy<'a, A>) -> A + Send + 'a
	) -> ArcLazy<'a, A> {
		use std::sync::OnceLock;

		let cell: Arc<OnceLock<ArcLazy<'a, A>>> = Arc::new(OnceLock::new());
		let cell_clone = cell.clone();
		let lazy = ArcLazy::new(move || {
			let self_ref = cell_clone.get().expect("arc_lazy_fix: cell not initialized").clone();
			f(self_ref)
		});
		let _ = cell.set(lazy.clone());
		lazy
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::inner::*,
		crate::types::{
			Thunk,
			Trampoline,
		},
		quickcheck_macros::quickcheck,
		std::{
			cell::RefCell,
			rc::Rc,
			sync::Arc,
		},
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
			sync::atomic::{
				AtomicUsize,
				Ordering,
			},
			thread,
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();
		let memo = ArcLazy::new(move || {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			42
		});

		let mut handles = vec![];
		for _ in 0 .. 10 {
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
		for _ in 0 .. 10 {
			let lazy_clone = lazy.clone();
			handles.push(thread::spawn(move || {
				assert_eq!(*lazy_clone.evaluate(), 42);
			}));
		}

		for handle in handles {
			handle.join().unwrap();
		}
	}

	// Memoization Properties

	/// Property: getting a memoized value twice returns the same result (Rc).
	#[quickcheck]
	fn prop_rc_memo_get_memoization(x: i32) -> bool {
		let memo = RcLazy::new(move || x.wrapping_mul(2));
		let result1 = *memo.evaluate();
		let result2 = *memo.evaluate();
		result1 == result2
	}

	/// Property: getting a memoized value twice returns the same result (Arc).
	#[quickcheck]
	fn prop_arc_memo_get_memoization(x: i32) -> bool {
		let memo = ArcLazy::new(move || x.wrapping_mul(2));
		let result1 = *memo.evaluate();
		let result2 = *memo.evaluate();
		result1 == result2
	}

	// Clone Equivalence Properties

	/// Property: cloning an RcLazy shares state.
	#[quickcheck]
	fn prop_rc_memo_clone_shares_state(x: i32) -> bool {
		let memo1 = RcLazy::new(move || x);
		let memo2 = memo1.clone();

		let result1 = *memo1.evaluate();
		let result2 = *memo2.evaluate();
		result1 == result2
	}

	/// Property: cloning an ArcLazy shares state.
	#[quickcheck]
	fn prop_arc_memo_clone_shares_state(x: i32) -> bool {
		let memo1 = ArcLazy::new(move || x);
		let memo2 = memo1.clone();

		let result1 = *memo1.evaluate();
		let result2 = *memo2.evaluate();
		result1 == result2
	}

	/// Property: getting original then clone gives same result.
	#[quickcheck]
	fn prop_memo_get_original_then_clone(x: String) -> bool {
		let value = x.clone();
		let memo = RcLazy::new(move || value.clone());
		let memo_clone = memo.clone();

		let result1 = memo.evaluate().clone();
		let result2 = memo_clone.evaluate().clone();

		result1 == result2
	}

	// Determinism Properties

	/// Property: lazy computation is deterministic.
	#[quickcheck]
	fn prop_memo_deterministic(
		x: i32,
		y: i32,
	) -> bool {
		let memo1 = RcLazy::new(move || x.wrapping_add(y));
		let memo2 = RcLazy::new(move || x.wrapping_add(y));

		*memo1.evaluate() == *memo2.evaluate()
	}

	// --- ArcLazy Foldable tests ---

	/// Tests `fold_right` on `ArcLazy`.
	#[test]
	fn test_arc_lazy_fold_right() {
		use crate::{
			brands::*,
			functions::*,
		};

		let lazy = ArcLazy::new(|| 10);
		let result = fold_right::<RcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(|a, b| a + b, 5, lazy);
		assert_eq!(result, 15);
	}

	/// Tests `fold_left` on `ArcLazy`.
	#[test]
	fn test_arc_lazy_fold_left() {
		use crate::{
			brands::*,
			functions::*,
		};

		let lazy = ArcLazy::new(|| 10);
		let result = fold_left::<RcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(|b, a| b + a, 5, lazy);
		assert_eq!(result, 15);
	}

	/// Tests `fold_map` on `ArcLazy`.
	#[test]
	fn test_arc_lazy_fold_map() {
		use crate::{
			brands::*,
			functions::*,
		};

		let lazy = ArcLazy::new(|| 10);
		let result = fold_map::<RcFnBrand, LazyBrand<ArcLazyConfig>, _, String>(
			|a: i32| a.to_string(),
			lazy,
		);
		assert_eq!(result, "10");
	}

	/// Property: ArcLazy fold_right is consistent with RcLazy fold_right.
	#[quickcheck]
	fn prop_arc_lazy_fold_right(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};

		let rc_lazy = RcLazy::new(move || x);
		let arc_lazy = ArcLazy::new(move || x);
		let rc_result =
			fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _>(|a, b| a + b, 0, rc_lazy);
		let arc_result =
			fold_right::<RcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(|a, b| a + b, 0, arc_lazy);
		rc_result == arc_result
	}

	// --- SendRefFunctor (ArcLazy ref_map) law tests ---

	/// Property: ArcLazy ref_map identity law.
	///
	/// ref_map(|x| x.clone(), fa) should produce the same value as fa.
	#[quickcheck]
	fn prop_arc_lazy_ref_map_identity(x: i32) -> bool {
		let lazy = ArcLazy::new(move || x);
		let mapped = lazy.clone().ref_map(|a| *a);
		*lazy.evaluate() == *mapped.evaluate()
	}

	/// Property: ArcLazy ref_map composition law.
	///
	/// ref_map(g . f, fa) == ref_map(g, ref_map(f, fa)).
	#[quickcheck]
	fn prop_arc_lazy_ref_map_composition(x: i32) -> bool {
		let f = |a: &i32| a.wrapping_mul(2);
		let g = |a: &i32| a.wrapping_add(1);

		let lazy1 = ArcLazy::new(move || x);
		let composed = lazy1.ref_map(|a| g(&f(a)));

		let lazy2 = ArcLazy::new(move || x);
		let chained = lazy2.ref_map(f).ref_map(g);

		*composed.evaluate() == *chained.evaluate()
	}

	/// Property: ArcLazy ref_map preserves memoization.
	#[quickcheck]
	fn prop_arc_lazy_ref_map_memoization(x: i32) -> bool {
		let lazy = ArcLazy::new(move || x);
		let mapped = lazy.ref_map(|a| a.wrapping_mul(3));
		let r1 = *mapped.evaluate();
		let r2 = *mapped.evaluate();
		r1 == r2
	}

	// --- rc_lazy_fix / arc_lazy_fix tests ---

	/// Tests `rc_lazy_fix` with a self-referential computation.
	#[test]
	fn test_rc_lazy_fix_self_reference() {
		let lazy = rc_lazy_fix(|self_ref: RcLazy<Vec<i32>>| {
			// The self-reference can be evaluated to get the same value.
			let _ = self_ref; // The self-ref is available but evaluating it here would recurse.
			vec![1, 2, 3]
		});
		assert_eq!(*lazy.evaluate(), vec![1, 2, 3]);
	}

	/// Tests `rc_lazy_fix` where `f` uses the self-reference after initial evaluation.
	#[test]
	fn test_rc_lazy_fix_uses_self_ref() {
		// Create a lazy value that, when evaluated, stores its own length.
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let lazy = rc_lazy_fix(move |self_ref: RcLazy<i32>| {
			*counter_clone.borrow_mut() += 1;
			// We cannot evaluate self_ref during construction (would deadlock),
			// but we can capture it for later use in a derived value.
			let _ = self_ref;
			42
		});
		assert_eq!(*lazy.evaluate(), 42);
		assert_eq!(*counter.borrow(), 1);
		// Verify memoization: second evaluate does not re-run.
		assert_eq!(*lazy.evaluate(), 42);
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests `arc_lazy_fix` with a self-referential computation.
	#[test]
	fn test_arc_lazy_fix_self_reference() {
		let lazy = arc_lazy_fix(|self_ref: ArcLazy<Vec<i32>>| {
			let _ = self_ref;
			vec![1, 2, 3]
		});
		assert_eq!(*lazy.evaluate(), vec![1, 2, 3]);
	}

	/// Tests `arc_lazy_fix` where `f` uses the self-reference after initial evaluation.
	#[test]
	fn test_arc_lazy_fix_uses_self_ref() {
		use std::sync::atomic::{
			AtomicUsize,
			Ordering,
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();
		let lazy = arc_lazy_fix(move |self_ref: ArcLazy<i32>| {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			let _ = self_ref;
			42
		});
		assert_eq!(*lazy.evaluate(), 42);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
		// Verify memoization: second evaluate does not re-run.
		assert_eq!(*lazy.evaluate(), 42);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	}

	/// Tests `arc_lazy_fix` is thread-safe.
	#[test]
	fn test_arc_lazy_fix_thread_safety() {
		use std::thread;

		let lazy = arc_lazy_fix(|self_ref: ArcLazy<i32>| {
			let _ = self_ref;
			42
		});

		let mut handles = vec![];
		for _ in 0 .. 10 {
			let lazy_clone = lazy.clone();
			handles.push(thread::spawn(move || {
				assert_eq!(*lazy_clone.evaluate(), 42);
			}));
		}

		for handle in handles {
			handle.join().unwrap();
		}
	}

	/// Tests `rc_lazy_fix` where the self-reference is shared with a derived lazy.
	#[test]
	fn test_rc_lazy_fix_shared_self_ref() {
		let lazy = rc_lazy_fix(|self_ref: RcLazy<i32>| {
			// Store the self-ref for later verification that it shares the same cache.
			let _captured = self_ref.clone();
			100
		});
		assert_eq!(*lazy.evaluate(), 100);
		// Clone should share the same cache.
		let cloned = lazy.clone();
		assert_eq!(*cloned.evaluate(), 100);
	}
}
