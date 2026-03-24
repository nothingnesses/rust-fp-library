//! Memoized lazy evaluation with shared cache semantics.
//!
//! Computes a value at most once on first access and caches the result. All clones share the same cache. Available in both single-threaded [`RcLazy`] and thread-safe [`ArcLazy`] variants.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				LazyBrand,
				RcBrand,
			},
			classes::{
				CloneableFn,
				Deferrable,
				Foldable,
				Monoid,
				RefFunctor,
				Semigroup,
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
			cell::{
				LazyCell,
				OnceCell,
			},
			fmt,
			rc::Rc,
			sync::{
				Arc,
				LazyLock,
				OnceLock,
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
	///
	/// # Extensibility
	///
	/// This trait is open for third-party implementations. You can define a custom
	/// `LazyConfig` to use alternative lazy cell or pointer types (for example,
	/// `parking_lot`-based locks or async-aware cells). Implement the four associated
	/// types ([`Lazy`](LazyConfig::Lazy), [`TryLazy`](LazyConfig::TryLazy),
	/// [`Thunk`](LazyConfig::Thunk), [`TryThunk`](LazyConfig::TryThunk)) and the
	/// four methods ([`lazy_new`](LazyConfig::lazy_new),
	/// [`try_lazy_new`](LazyConfig::try_lazy_new), [`evaluate`](LazyConfig::evaluate),
	/// [`try_evaluate`](LazyConfig::try_evaluate)), then use your config as the
	/// `Config` parameter on [`Lazy`] and [`TryLazy`](crate::types::TryLazy).
	pub trait LazyConfig: 'static {
		/// The pointer brand used by this configuration.
		///
		/// Links the lazy configuration to the pointer hierarchy, enabling
		/// generic code to obtain the underlying pointer brand from a
		/// `LazyConfig` without hard-coding `RcBrand` or `ArcBrand`.
		type PointerBrand: crate::classes::RefCountedPointer;

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
			"The lifetime of the reference.",
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
			"The lifetime of the reference.",
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
		type PointerBrand = RcBrand;
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
			"The lifetime of the reference.",
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
			"The lifetime of the reference.",
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
		type PointerBrand = ArcBrand;
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
			"The lifetime of the reference.",
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
			"The lifetime of the reference.",
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
	/// Cloning a `Lazy` shares the underlying cache, so all clones see the same value.
	///
	/// ### Panics
	///
	/// If the initializer closure panics, the underlying [`LazyCell`](std::cell::LazyCell)
	/// (for [`RcLazyConfig`]) or [`LazyLock`](std::sync::LazyLock) (for [`ArcLazyConfig`])
	/// is poisoned. Any subsequent call to [`evaluate`](Lazy::evaluate) on the same instance
	/// or any of its clones will panic again. For panic-safe memoization, use
	/// [`TryLazy`](crate::types::TryLazy) with a closure that catches panics via
	/// [`std::panic::catch_unwind`].
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
		#[inline]
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
		#[inline]
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
		#[inline]
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
		#[inline]
		pub fn ref_map<B: 'a>(
			self,
			f: impl FnOnce(&A) -> B + 'a,
		) -> Lazy<'a, B, RcLazyConfig> {
			let init: Box<dyn FnOnce() -> B + 'a> = Box::new(move || f(self.evaluate()));
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
	impl<'a, A: 'static> From<Trampoline<A>> for Lazy<'a, A, RcLazyConfig> {
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
		#[inline]
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
		#[inline]
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
			A: Send, {
			Lazy(ArcLazyConfig::lazy_new(Box::new(move || a)))
		}
	}

	#[document_type_parameters("The lifetime of the reference.", "The type of the computed value.")]
	#[document_parameters("The lazy value to map over.")]
	impl<'a, A: 'a> Lazy<'a, A, ArcLazyConfig> {
		/// Maps a function over the memoized value by reference.
		///
		/// This is the `ArcLazy` counterpart of [`RcLazy::ref_map`](Lazy::ref_map).
		/// The mapping function receives a reference to the cached value and returns a new value,
		/// which is itself lazily memoized.
		///
		/// Note: A blanket `RefFunctor` trait impl is not provided for `LazyBrand<ArcLazyConfig>`
		/// because the `RefFunctor` trait does not require `Send` on the mapping function, but
		/// `ArcLazy::new` requires `Send`. This inherent method adds the necessary `Send` bounds
		/// explicitly.
		#[document_signature]
		#[document_type_parameters("The type of the result.")]
		#[document_parameters("The function to apply to the memoized value.")]
		#[document_returns("A new `ArcLazy` instance containing the mapped value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = ArcLazy::new(|| 10);
		/// let mapped = memo.ref_map(|x| *x * 2);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		#[inline]
		pub fn ref_map<B: 'a>(
			self,
			f: impl FnOnce(&A) -> B + Send + 'a,
		) -> Lazy<'a, B, ArcLazyConfig>
		where
			A: Send + Sync, {
			ArcLazy::new(move || f(self.evaluate()))
		}
	}

	/// Single-threaded memoization alias.
	pub type RcLazy<'a, A> = Lazy<'a, A, RcLazyConfig>;

	/// Thread-safe memoization alias.
	pub type ArcLazy<'a, A> = Lazy<'a, A, ArcLazyConfig>;

	/// Creates a lazy value as the fixed point of a function (single-threaded variant).
	///
	/// `rc_lazy_fix(f)` produces an [`RcLazy`] value `x` such that evaluating `x` is
	/// equivalent to evaluating `f(x)`. This ties a recursive knot lazily: the
	/// function `f` receives a clone of the not-yet-evaluated result, and when
	/// that clone is eventually forced it triggers the same `f`-based computation.
	///
	/// This is the Rust analogue of PureScript's `fix :: (l -> l) -> l` from
	/// `Control.Lazy`, specialized to [`RcLazy`].
	///
	/// Internally, an [`Rc<OnceCell>`] breaks the self-referential cycle: the cell
	/// is allocated empty, the result [`RcLazy`] is constructed with a closure that
	/// reads from the cell, and then the cell is populated with the result before
	/// it is returned.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	///
	#[document_parameters("A function that takes a lazy self-reference and returns a lazy value.")]
	///
	#[document_returns("A lazy value that is the fixed point of `f`.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // Fixed point of a function that ignores its argument and returns 42.
	/// let fixed = rc_lazy_fix(|_self_ref: RcLazy<i32>| RcLazy::pure(42));
	/// assert_eq!(*fixed.evaluate(), 42);
	///
	/// // Fixed point where the function returns a new lazy value.
	/// let fixed = rc_lazy_fix(|_self_ref: RcLazy<i32>| RcLazy::new(move || 7));
	/// assert_eq!(*fixed.evaluate(), 7);
	/// ```
	pub fn rc_lazy_fix<'a, A: Clone + 'a>(
		f: impl Fn(RcLazy<'a, A>) -> RcLazy<'a, A> + 'a
	) -> RcLazy<'a, A> {
		let cell: Rc<OnceCell<RcLazy<'a, A>>> = Rc::new(OnceCell::new());
		let cell_ref = cell.clone();
		let result = RcLazy::new(move || {
			// SAFETY (logical): The cell is always populated before this closure
			// can be invoked, because `cell.set(result.clone())` runs immediately
			// after construction, before the `RcLazy` is returned to the caller.
			#[allow(clippy::unwrap_used)]
			let go = cell_ref.get().unwrap().clone();
			f(go).evaluate().clone()
		});
		cell.set(result.clone()).ok();
		result
	}

	/// Creates a lazy value as the fixed point of a function (thread-safe variant).
	///
	/// `arc_lazy_fix(f)` produces an [`ArcLazy`] value `x` such that evaluating `x` is
	/// equivalent to evaluating `f(x)`. This ties a recursive knot lazily: the
	/// function `f` receives a clone of the not-yet-evaluated result, and when
	/// that clone is eventually forced it triggers the same `f`-based computation.
	///
	/// This is the thread-safe analogue of [`rc_lazy_fix`], using [`Arc<OnceLock>`]
	/// instead of [`Rc<OnceCell>`] to break the self-referential cycle.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	///
	#[document_parameters("A function that takes a lazy self-reference and returns a lazy value.")]
	///
	#[document_returns("A lazy value that is the fixed point of `f`.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // Fixed point of a function that ignores its argument and returns 42.
	/// let fixed = arc_lazy_fix(|_self_ref: ArcLazy<i32>| ArcLazy::pure(42));
	/// assert_eq!(*fixed.evaluate(), 42);
	///
	/// // Thread-safe fixed point.
	/// let fixed = arc_lazy_fix(|_self_ref: ArcLazy<i32>| ArcLazy::new(|| 99));
	/// assert_eq!(*fixed.evaluate(), 99);
	/// ```
	pub fn arc_lazy_fix<'a, A: Clone + Send + Sync + 'a>(
		f: impl Fn(ArcLazy<'a, A>) -> ArcLazy<'a, A> + Send + Sync + 'a
	) -> ArcLazy<'a, A> {
		let cell: Arc<OnceLock<ArcLazy<'a, A>>> = Arc::new(OnceLock::new());
		let cell_ref = cell.clone();
		let result = ArcLazy::new(move || {
			// SAFETY (logical): The lock is always populated before this closure
			// can be invoked, because `cell.set(result.clone())` runs immediately
			// after construction, before the `ArcLazy` is returned to the caller.
			#[allow(clippy::unwrap_used)]
			let go = cell_ref.get().unwrap().clone();
			f(go).evaluate().clone()
		});
		cell.set(result.clone()).ok();
		result
	}

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

	// --- Semigroup ---

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: Semigroup + Clone + 'a> Semigroup for Lazy<'a, A, RcLazyConfig> {
		/// Combines two `RcLazy` values by combining their results using the inner type's `Semigroup`.
		///
		/// Both sides are forced on evaluation.
		#[document_signature]
		///
		#[document_parameters("The first lazy value.", "The second lazy value.")]
		///
		#[document_returns("A new `RcLazy` containing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let a = RcLazy::pure("Hello".to_string());
		/// let b = RcLazy::pure(" World".to_string());
		/// let c = append(a, b);
		/// assert_eq!(*c.evaluate(), "Hello World");
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			RcLazy::new(move || Semigroup::append(a.evaluate().clone(), b.evaluate().clone()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: Semigroup + Clone + Send + Sync + 'a> Semigroup for Lazy<'a, A, ArcLazyConfig> {
		/// Combines two `ArcLazy` values by combining their results using the inner type's `Semigroup`.
		///
		/// Both sides are forced on evaluation.
		#[document_signature]
		///
		#[document_parameters("The first lazy value.", "The second lazy value.")]
		///
		#[document_returns("A new `ArcLazy` containing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let a = ArcLazy::pure("Hello".to_string());
		/// let b = ArcLazy::pure(" World".to_string());
		/// let c = append(a, b);
		/// assert_eq!(*c.evaluate(), "Hello World");
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			ArcLazy::new(move || Semigroup::append(a.evaluate().clone(), b.evaluate().clone()))
		}
	}

	// --- Monoid ---

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: Monoid + Clone + 'a> Monoid for Lazy<'a, A, RcLazyConfig> {
		/// Returns the identity `RcLazy`.
		#[document_signature]
		///
		#[document_returns("An `RcLazy` producing the identity value of `A`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let t: RcLazy<String> = empty();
		/// assert_eq!(*t.evaluate(), "");
		/// ```
		fn empty() -> Self {
			RcLazy::new(|| Monoid::empty())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: Monoid + Clone + Send + Sync + 'a> Monoid for Lazy<'a, A, ArcLazyConfig> {
		/// Returns the identity `ArcLazy`.
		#[document_signature]
		///
		#[document_returns("An `ArcLazy` producing the identity value of `A`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let t: ArcLazy<String> = empty();
		/// assert_eq!(*t.evaluate(), "");
		/// ```
		fn empty() -> Self {
			ArcLazy::new(|| Monoid::empty())
		}
	}

	// --- Foldable ---

	impl Foldable for LazyBrand<RcLazyConfig> {
		/// Folds the `RcLazy` from the right.
		///
		/// Forces evaluation of the lazy value and applies the folding function to the cloned
		/// result and the initial accumulator. Since `Lazy` contains exactly one element, this is
		/// equivalent to applying the function once.
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
			"The `RcLazy` to fold."
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
		/// let lazy = RcLazy::pure(10);
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

		/// Folds the `RcLazy` from the left.
		///
		/// Forces evaluation and applies the folding function with the accumulator on the left.
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
			"The `RcLazy` to fold."
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
		/// let lazy = RcLazy::pure(10);
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
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The `RcLazy` to fold.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::pure(10);
		/// let result = fold_map::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _>(|a| a.to_string(), lazy);
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			func(fa.evaluate().clone())
		}
	}

	impl Foldable for LazyBrand<ArcLazyConfig> {
		/// Folds the `ArcLazy` from the right.
		///
		/// Forces evaluation of the lazy value and applies the folding function to the cloned
		/// result and the initial accumulator. Since `Lazy` contains exactly one element, this is
		/// equivalent to applying the function once.
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
			"The `ArcLazy` to fold."
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
		/// let lazy = ArcLazy::pure(10);
		/// let result = fold_right::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(|a, b| a + b, 5, lazy);
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

		/// Folds the `ArcLazy` from the left.
		///
		/// Forces evaluation and applies the folding function with the accumulator on the left.
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
			"The `ArcLazy` to fold."
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
		/// let lazy = ArcLazy::pure(10);
		/// let result = fold_left::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(|b, a| b + a, 5, lazy);
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
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The `ArcLazy` to fold.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::pure(10);
		/// let result = fold_map::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(|a| a.to_string(), lazy);
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			func(fa.evaluate().clone())
		}
	}

	// --- PartialEq ---

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The memoization configuration."
	)]
	#[document_parameters("The lazy value to compare.")]
	impl<'a, A: PartialEq + 'a, Config: LazyConfig> PartialEq for Lazy<'a, A, Config> {
		/// Compares two `Lazy` values for equality by forcing evaluation of both sides.
		///
		/// Note: This will trigger computation if either value has not yet been evaluated.
		#[document_signature]
		#[document_parameters("The other lazy value to compare with.")]
		#[document_returns("`true` if the evaluated values are equal.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let a = RcLazy::pure(42);
		/// let b = RcLazy::pure(42);
		/// assert!(a == b);
		/// ```
		fn eq(
			&self,
			other: &Self,
		) -> bool {
			self.evaluate() == other.evaluate()
		}
	}

	// --- PartialOrd ---

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The memoization configuration."
	)]
	#[document_parameters("The lazy value to compare.")]
	impl<'a, A: PartialOrd + 'a, Config: LazyConfig> PartialOrd for Lazy<'a, A, Config> {
		/// Compares two `Lazy` values for ordering by forcing evaluation of both sides.
		///
		/// Note: This will trigger computation if either value has not yet been evaluated.
		#[document_signature]
		#[document_parameters("The other lazy value to compare with.")]
		#[document_returns(
			"The ordering between the evaluated values, or `None` if not comparable."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let a = RcLazy::pure(1);
		/// let b = RcLazy::pure(2);
		/// assert!(a < b);
		/// ```
		fn partial_cmp(
			&self,
			other: &Self,
		) -> Option<std::cmp::Ordering> {
			self.evaluate().partial_cmp(other.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the reference.",
		"The type of the computed value.",
		"The memoization configuration."
	)]
	#[document_parameters("The lazy value to format.")]
	impl<'a, A, Config: LazyConfig> fmt::Debug for Lazy<'a, A, Config>
	where
		A: 'a,
	{
		/// Formats the lazy value without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
		/// assert_eq!(format!("{:?}", lazy), "Lazy(..)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("Lazy(..)")
		}
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
		// Trampoline requires 'static due to type erasure via Box<dyn Any>
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

	// --- Tests for ArcLazy::ref_map ---

	/// Tests `ArcLazy::ref_map`.
	///
	/// Verifies that `ref_map` applies a function to the memoized value by reference.
	#[test]
	fn test_arc_lazy_ref_map() {
		let memo = ArcLazy::new(|| 10);
		let mapped = memo.ref_map(|x| *x * 2);
		assert_eq!(*mapped.evaluate(), 20);
	}

	/// Tests `ArcLazy::ref_map` thread safety.
	///
	/// Verifies that the mapped value can be shared across threads.
	#[test]
	fn test_arc_lazy_ref_map_thread_safety() {
		use std::thread;

		let memo = ArcLazy::new(|| 10);
		let mapped = memo.ref_map(|x| *x * 3);

		let mut handles = vec![];
		for _ in 0 .. 5 {
			let m = mapped.clone();
			handles.push(thread::spawn(move || {
				assert_eq!(*m.evaluate(), 30);
			}));
		}

		for handle in handles {
			handle.join().unwrap();
		}
	}

	// --- Tests for Semigroup ---

	/// Tests the `Semigroup` implementation for `RcLazy`.
	///
	/// Verifies that `append` correctly combines two lazy values.
	#[test]
	fn test_rc_lazy_semigroup() {
		use crate::classes::semigroup::append;

		let a = RcLazy::pure("Hello".to_string());
		let b = RcLazy::pure(" World".to_string());
		let c = append(a, b);
		assert_eq!(*c.evaluate(), "Hello World");
	}

	/// Tests the `Semigroup` implementation for `ArcLazy`.
	///
	/// Verifies that `append` correctly combines two thread-safe lazy values.
	#[test]
	fn test_arc_lazy_semigroup() {
		use crate::classes::semigroup::append;

		let a = ArcLazy::pure("Hello".to_string());
		let b = ArcLazy::pure(" World".to_string());
		let c = append(a, b);
		assert_eq!(*c.evaluate(), "Hello World");
	}

	/// Tests `Semigroup` associativity for `RcLazy`.
	#[test]
	fn test_rc_lazy_semigroup_associativity() {
		use crate::classes::semigroup::append;

		let a = RcLazy::pure("a".to_string());
		let b = RcLazy::pure("b".to_string());
		let c = RcLazy::pure("c".to_string());

		let ab_c = append(append(a.clone(), b.clone()), c.clone());
		let a_bc = append(a, append(b, c));

		assert_eq!(*ab_c.evaluate(), *a_bc.evaluate());
	}

	// --- Tests for Monoid ---

	/// Tests the `Monoid` implementation for `RcLazy`.
	///
	/// Verifies that `empty` returns the identity element.
	#[test]
	fn test_rc_lazy_monoid() {
		use crate::classes::monoid::empty;

		let t: RcLazy<String> = empty();
		assert_eq!(*t.evaluate(), "");
	}

	/// Tests the `Monoid` implementation for `ArcLazy`.
	///
	/// Verifies that `empty` returns the identity element.
	#[test]
	fn test_arc_lazy_monoid() {
		use crate::classes::monoid::empty;

		let t: ArcLazy<String> = empty();
		assert_eq!(*t.evaluate(), "");
	}

	/// Tests `Monoid` identity laws for `RcLazy`.
	#[test]
	fn test_rc_lazy_monoid_identity() {
		use crate::classes::{
			monoid::empty,
			semigroup::append,
		};

		let a = RcLazy::pure("hello".to_string());

		// Left identity: append(empty(), a) == a
		let left: RcLazy<String> = append(empty(), a.clone());
		assert_eq!(*left.evaluate(), *a.evaluate());

		// Right identity: append(a, empty()) == a
		let right: RcLazy<String> = append(a.clone(), empty());
		assert_eq!(*right.evaluate(), *a.evaluate());
	}

	// --- Tests for Foldable ---

	/// Tests `fold_right` for `RcLazy`.
	#[test]
	fn test_rc_lazy_fold_right() {
		use crate::functions::*;

		let lazy = RcLazy::pure(10);
		let result = fold_right::<
			crate::brands::RcFnBrand,
			crate::brands::LazyBrand<RcLazyConfig>,
			_,
			_,
		>(|a, b| a + b, 5, lazy);
		assert_eq!(result, 15);
	}

	/// Tests `fold_left` for `RcLazy`.
	#[test]
	fn test_rc_lazy_fold_left() {
		use crate::functions::*;

		let lazy = RcLazy::pure(10);
		let result = fold_left::<
			crate::brands::RcFnBrand,
			crate::brands::LazyBrand<RcLazyConfig>,
			_,
			_,
		>(|b, a| b + a, 5, lazy);
		assert_eq!(result, 15);
	}

	/// Tests `fold_map` for `RcLazy`.
	#[test]
	fn test_rc_lazy_fold_map() {
		use crate::functions::*;

		let lazy = RcLazy::pure(10);
		let result =
			fold_map::<crate::brands::RcFnBrand, crate::brands::LazyBrand<RcLazyConfig>, _, _>(
				|a: i32| a.to_string(),
				lazy,
			);
		assert_eq!(result, "10");
	}

	// --- Tests for PartialEq ---

	/// Tests `PartialEq` for `RcLazy`.
	///
	/// Verifies that equality comparison forces evaluation.
	#[test]
	fn test_rc_lazy_partial_eq() {
		let a = RcLazy::pure(42);
		let b = RcLazy::pure(42);
		let c = RcLazy::pure(99);

		assert!(a == b);
		assert!(b != c);
	}

	/// Tests `PartialEq` for `ArcLazy`.
	///
	/// Verifies that equality comparison forces evaluation.
	#[test]
	fn test_arc_lazy_partial_eq() {
		let a = ArcLazy::pure(42);
		let b = ArcLazy::pure(42);
		let c = ArcLazy::pure(99);

		assert!(a == b);
		assert!(b != c);
	}

	// --- Tests for PartialOrd ---

	/// Tests `PartialOrd` for `RcLazy`.
	///
	/// Verifies that ordering comparison forces evaluation.
	#[test]
	fn test_rc_lazy_partial_ord() {
		let a = RcLazy::pure(1);
		let b = RcLazy::pure(2);
		let c = RcLazy::pure(2);

		assert!(a < b);
		assert!(b > a);
		assert!(b >= c);
		assert!(c <= b);
	}

	/// Tests `PartialOrd` for `ArcLazy`.
	///
	/// Verifies that ordering comparison forces evaluation.
	#[test]
	fn test_arc_lazy_partial_ord() {
		let a = ArcLazy::pure(1);
		let b = ArcLazy::pure(2);

		assert!(a < b);
		assert!(b > a);
	}

	// fix combinator tests

	/// Tests that `rc_lazy_fix` produces the correct value when
	/// the function ignores the self-reference.
	#[test]
	fn test_rc_lazy_fix_constant() {
		let fixed = rc_lazy_fix(|_self_ref: RcLazy<i32>| RcLazy::pure(42));
		assert_eq!(*fixed.evaluate(), 42);
	}

	/// Tests that `rc_lazy_fix` sets up the `OnceCell` before evaluation.
	///
	/// The closure inside the resulting `RcLazy` reads from the cell, so the
	/// cell must be populated by the time evaluation happens.
	#[test]
	fn test_rc_lazy_fix_cell_initialized() {
		let fixed =
			rc_lazy_fix(|_self_ref: RcLazy<String>| RcLazy::new(|| String::from("initialized")));
		// If the cell were not initialized, this would panic.
		assert_eq!(fixed.evaluate().as_str(), "initialized");
	}

	/// Tests that `rc_lazy_fix` correctly threads the self-reference.
	///
	/// The function `f` receives a lazy self-reference. When `f` does not
	/// force it (avoiding infinite recursion), we can verify the plumbing
	/// by having `f` return a value derived from a closure that captures
	/// (but does not evaluate) the self-reference.
	#[test]
	fn test_rc_lazy_fix_self_reference_plumbing() {
		// f returns a value without forcing the self-reference.
		let fixed = rc_lazy_fix(|_self_ref: RcLazy<i32>| RcLazy::new(|| 7));
		assert_eq!(*fixed.evaluate(), 7);
	}

	/// Tests that `rc_lazy_fix` memoizes the result.
	///
	/// Multiple evaluations should return the same cached value.
	#[test]
	fn test_rc_lazy_fix_memoization() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let fixed = rc_lazy_fix(move |_self_ref: RcLazy<i32>| {
			let c = counter_clone.clone();
			RcLazy::new(move || {
				*c.borrow_mut() += 1;
				100
			})
		});

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(*fixed.evaluate(), 100);
		assert_eq!(*counter.borrow(), 1);
		// Second evaluation should use cached value.
		assert_eq!(*fixed.evaluate(), 100);
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `rc_lazy_fix` works with cloned results sharing the cache.
	#[test]
	fn test_rc_lazy_fix_clone_sharing() {
		let fixed = rc_lazy_fix(|_self_ref: RcLazy<i32>| RcLazy::pure(55));
		let cloned = fixed.clone();
		assert_eq!(*fixed.evaluate(), 55);
		assert_eq!(*cloned.evaluate(), 55);
	}

	/// Tests that `arc_lazy_fix` produces the correct value when
	/// the function ignores the self-reference.
	#[test]
	fn test_arc_lazy_fix_constant() {
		let fixed = arc_lazy_fix(|_self_ref: ArcLazy<i32>| ArcLazy::pure(42));
		assert_eq!(*fixed.evaluate(), 42);
	}

	/// Tests that `arc_lazy_fix` sets up the `OnceLock` before evaluation.
	#[test]
	fn test_arc_lazy_fix_cell_initialized() {
		let fixed =
			arc_lazy_fix(|_self_ref: ArcLazy<String>| ArcLazy::new(|| String::from("initialized")));
		// If the lock were not initialized, this would panic.
		assert_eq!(fixed.evaluate().as_str(), "initialized");
	}

	/// Tests that `arc_lazy_fix` correctly threads the self-reference.
	#[test]
	fn test_arc_lazy_fix_self_reference_plumbing() {
		let fixed = arc_lazy_fix(|_self_ref: ArcLazy<i32>| ArcLazy::new(|| 7));
		assert_eq!(*fixed.evaluate(), 7);
	}

	/// Tests that `arc_lazy_fix` memoizes the result.
	#[test]
	fn test_arc_lazy_fix_memoization() {
		use std::sync::atomic::{
			AtomicUsize,
			Ordering,
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();
		let fixed = arc_lazy_fix(move |_self_ref: ArcLazy<i32>| {
			let c = counter_clone.clone();
			ArcLazy::new(move || {
				c.fetch_add(1, Ordering::SeqCst);
				100
			})
		});

		assert_eq!(counter.load(Ordering::SeqCst), 0);
		assert_eq!(*fixed.evaluate(), 100);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
		// Second evaluation should use cached value.
		assert_eq!(*fixed.evaluate(), 100);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	}

	/// Tests that `arc_lazy_fix` is thread-safe.
	///
	/// The fixed-point value can be shared across threads and all
	/// threads see the same result.
	#[test]
	fn test_arc_lazy_fix_thread_safety() {
		use std::thread;

		let fixed = arc_lazy_fix(|_self_ref: ArcLazy<i32>| ArcLazy::pure(77));

		let mut handles = vec![];
		for _ in 0 .. 10 {
			let fixed_clone = fixed.clone();
			handles.push(thread::spawn(move || {
				assert_eq!(*fixed_clone.evaluate(), 77);
			}));
		}

		for handle in handles {
			handle.join().unwrap();
		}
	}

	/// Property: `rc_lazy_fix` with a constant function produces that constant.
	#[quickcheck]
	fn prop_rc_lazy_fix_constant(x: i32) -> bool {
		let fixed = rc_lazy_fix(move |_: RcLazy<i32>| RcLazy::pure(x));
		*fixed.evaluate() == x
	}

	/// Property: `arc_lazy_fix` with a constant function produces that constant.
	#[quickcheck]
	fn prop_arc_lazy_fix_constant(x: i32) -> bool {
		let fixed = arc_lazy_fix(move |_: ArcLazy<i32>| ArcLazy::pure(x));
		*fixed.evaluate() == x
	}

	// QuickCheck Law Tests

	// RefFunctor Laws

	/// RefFunctor identity: `ref_map(|v| v.clone(), fa)` evaluates to the same value as `fa`.
	#[quickcheck]
	fn ref_functor_identity(x: i32) -> bool {
		let lazy = RcLazy::pure(x);
		*lazy.clone().ref_map(|v| *v).evaluate() == *lazy.evaluate()
	}

	/// RefFunctor composition: `ref_map(|v| f(&g(v)), fa) == ref_map(f, ref_map(g, fa))`.
	#[quickcheck]
	fn ref_functor_composition(x: i32) -> bool {
		let f = |a: &i32| a.wrapping_add(1);
		let g = |a: &i32| a.wrapping_mul(2);
		let lazy = RcLazy::pure(x);
		let lhs = *lazy.clone().ref_map(move |v| f(&g(v))).evaluate();
		let rhs = *lazy.ref_map(g).ref_map(f).evaluate();
		lhs == rhs
	}

	// Deferrable Laws

	/// Deferrable transparency: `defer(|| pure(x))` evaluates to the same value as `pure(x)`.
	#[quickcheck]
	fn deferrable_transparency(x: i32) -> bool {
		let lazy = RcLazy::pure(x);
		let deferred: RcLazy<i32> = crate::classes::Deferrable::defer(move || RcLazy::pure(x));
		*deferred.evaluate() == *lazy.evaluate()
	}

	// Semigroup Laws

	/// Semigroup associativity for `RcLazy`: `append(append(a, b), c) == append(a, append(b, c))`.
	#[quickcheck]
	fn rc_lazy_semigroup_associativity(
		a: String,
		b: String,
		c: String,
	) -> bool {
		use crate::classes::semigroup::append;

		let la = RcLazy::pure(a.clone());
		let lb = RcLazy::pure(b.clone());
		let lc = RcLazy::pure(c.clone());
		let la2 = RcLazy::pure(a);
		let lb2 = RcLazy::pure(b);
		let lc2 = RcLazy::pure(c);
		let lhs = append(append(la, lb), lc).evaluate().clone();
		let rhs = append(la2, append(lb2, lc2)).evaluate().clone();
		lhs == rhs
	}

	/// Semigroup associativity for `ArcLazy`: `append(append(a, b), c) == append(a, append(b, c))`.
	#[quickcheck]
	fn arc_lazy_semigroup_associativity(
		a: String,
		b: String,
		c: String,
	) -> bool {
		use crate::classes::semigroup::append;

		let la = ArcLazy::pure(a.clone());
		let lb = ArcLazy::pure(b.clone());
		let lc = ArcLazy::pure(c.clone());
		let la2 = ArcLazy::pure(a);
		let lb2 = ArcLazy::pure(b);
		let lc2 = ArcLazy::pure(c);
		let lhs = append(append(la, lb), lc).evaluate().clone();
		let rhs = append(la2, append(lb2, lc2)).evaluate().clone();
		lhs == rhs
	}

	// Monoid Laws

	/// Monoid left identity for `RcLazy`: `append(empty(), a) == a`.
	#[quickcheck]
	fn rc_lazy_monoid_left_identity(x: String) -> bool {
		use crate::classes::{
			monoid::empty,
			semigroup::append,
		};

		let a = RcLazy::pure(x.clone());
		let lhs: RcLazy<String> = append(empty(), a);
		*lhs.evaluate() == x
	}

	/// Monoid right identity for `RcLazy`: `append(a, empty()) == a`.
	#[quickcheck]
	fn rc_lazy_monoid_right_identity(x: String) -> bool {
		use crate::classes::{
			monoid::empty,
			semigroup::append,
		};

		let a = RcLazy::pure(x.clone());
		let rhs: RcLazy<String> = append(a, empty());
		*rhs.evaluate() == x
	}

	/// Monoid left identity for `ArcLazy`: `append(empty(), a) == a`.
	#[quickcheck]
	fn arc_lazy_monoid_left_identity(x: String) -> bool {
		use crate::classes::{
			monoid::empty,
			semigroup::append,
		};

		let a = ArcLazy::pure(x.clone());
		let lhs: ArcLazy<String> = append(empty(), a);
		*lhs.evaluate() == x
	}

	/// Monoid right identity for `ArcLazy`: `append(a, empty()) == a`.
	#[quickcheck]
	fn arc_lazy_monoid_right_identity(x: String) -> bool {
		use crate::classes::{
			monoid::empty,
			semigroup::append,
		};

		let a = ArcLazy::pure(x.clone());
		let rhs: ArcLazy<String> = append(a, empty());
		*rhs.evaluate() == x
	}

	// --- Tests for LazyConfig::PointerBrand ---

	/// Tests that `RcLazyConfig::PointerBrand` is `RcBrand`.
	#[test]
	fn test_rc_lazy_config_pointer_brand() {
		fn assert_brand_is_rc<C: LazyConfig<PointerBrand = crate::brands::RcBrand>>() {}
		assert_brand_is_rc::<RcLazyConfig>();
	}

	/// Tests that `ArcLazyConfig::PointerBrand` is `ArcBrand`.
	#[test]
	fn test_arc_lazy_config_pointer_brand() {
		fn assert_brand_is_arc<C: LazyConfig<PointerBrand = crate::brands::ArcBrand>>() {}
		assert_brand_is_arc::<ArcLazyConfig>();
	}

	// SC-2: Panic poisoning test for Lazy

	/// Tests that a panicking initializer poisons the RcLazy.
	///
	/// Verifies that subsequent evaluate calls also panic after
	/// the initializer panics.
	#[test]
	fn test_panic_poisoning() {
		use std::panic;

		let memo: RcLazy<i32> = RcLazy::new(|| {
			panic!("initializer panic");
		});

		let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
			let _ = memo.evaluate();
		}));
		assert!(result.is_err(), "First evaluate should panic");

		let result2 = panic::catch_unwind(panic::AssertUnwindSafe(|| {
			let _ = memo.evaluate();
		}));
		assert!(result2.is_err(), "Second evaluate should also panic (poisoned)");
	}
}
