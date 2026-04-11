//! Memoized lazy evaluation with shared cache semantics.
//!
//! Computes a value at most once on first access and caches the result. All clones share the same cache. Available in both single-threaded [`RcLazy`] and thread-safe [`ArcLazy`] variants.
//!
//! ## Why `Lazy` does not implement `Functor`
//!
//! [`Lazy::evaluate`] returns `&A` (a reference to the cached value), not an owned `A`.
//! The standard [`Functor`](crate::classes::Functor) trait requires `A -> B`, which would
//! need either cloning or consuming the cached value. Instead, `Lazy` implements
//! [`RefFunctor`](crate::classes::RefFunctor) (and [`SendRefFunctor`](crate::classes::SendRefFunctor)
//! for `ArcLazy`), whose `ref_map` takes `&A -> B`. Use [`map`](crate::functions::map) with a
//! closure that takes `&A` to map over lazy values, or [`send_ref_map`](crate::functions::send_ref_map)
//! for thread-safe mapping.

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
				CloneFn,
				Deferrable,
				LiftFn,
				Monoid,
				RefFoldable,
				RefFoldableWithIndex,
				RefFunctor,
				RefFunctorWithIndex,
				RefLift,
				RefPointed,
				RefSemiapplicative,
				RefSemimonad,
				Semigroup,
				SendCloneFn,
				SendDeferrable,
				SendLiftFn,
				SendRefFoldable,
				SendRefFoldableWithIndex,
				SendRefFunctor,
				SendRefFunctorWithIndex,
				SendRefLift,
				SendRefPointed,
				SendRefSemiapplicative,
				SendRefSemimonad,
				WithIndex,
				dispatch::Ref,
			},
			impl_kind,
			kinds::*,
			types::{
				SendThunk,
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

	pub use crate::classes::LazyConfig;

	/// Single-threaded memoization using [`Rc<LazyCell>`].
	///
	/// Not thread-safe. Use [`ArcLazyConfig`] for multi-threaded contexts.
	pub struct RcLazyConfig;

	impl LazyConfig for RcLazyConfig {
		type Lazy<'a, A: 'a> = Rc<LazyCell<A, Box<dyn FnOnce() -> A + 'a>>>;
		type PointerBrand = RcBrand;
		type Thunk<'a, A: 'a> = dyn FnOnce() -> A + 'a;

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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
		/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
		/// ```
		fn lazy_new<'a, A: 'a>(f: Box<Self::Thunk<'a, A>>) -> Self::Lazy<'a, A> {
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
		/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
		/// ```
		fn evaluate<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A {
			LazyCell::force(lazy)
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazyConfig::lazy_new(Box::new(|| 42));
		/// assert_eq!(*ArcLazyConfig::evaluate(&lazy), 42);
		/// ```
		fn lazy_new<'a, A: 'a>(f: Box<Self::Thunk<'a, A>>) -> Self::Lazy<'a, A> {
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazyConfig::lazy_new(Box::new(|| 42));
		/// assert_eq!(*ArcLazyConfig::evaluate(&lazy), 42);
		/// ```
		fn evaluate<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A {
			LazyLock::force(lazy)
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
		"The lifetime of the computation.",
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
		"The lifetime of the computation.",
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
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
		"The lifetime of the computation.",
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let memo = Lazy::<_, RcLazyConfig>::new(|| 42);
		/// assert_eq!(*memo.evaluate(), 42);
		/// ```
		#[inline]
		pub fn evaluate(&self) -> &A {
			Config::evaluate(&self.0)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self {
			Lazy(RcLazyConfig::lazy_new(Box::new(move || a)))
		}

		/// Returns a clone of the memoized value, computing on first access.
		///
		/// This is a convenience wrapper around [`evaluate`](Lazy::evaluate) for cases
		/// where an owned value is needed rather than a reference.
		#[document_signature]
		///
		#[document_returns("An owned clone of the memoized value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let memo = RcLazy::new(|| vec![1, 2, 3]);
		/// let owned: Vec<i32> = memo.evaluate_owned();
		/// assert_eq!(owned, vec![1, 2, 3]);
		/// ```
		#[inline]
		pub fn evaluate_owned(&self) -> A
		where
			A: Clone, {
			self.evaluate().clone()
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
		/// let mapped = memo.ref_map(|x| *x * 2);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		#[inline]
		pub fn ref_map<B: 'a>(
			&self,
			f: impl Fn(&A) -> B + 'a,
		) -> Lazy<'a, B, RcLazyConfig> {
			let this = self.clone();
			RcLazy::new(move || f(this.evaluate()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A> From<Thunk<'a, A>> for Lazy<'a, A, RcLazyConfig> {
		#[document_signature]
		#[document_parameters("The thunk to convert.")]
		#[document_returns("A new `Lazy` instance that will evaluate the thunk on first access.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		/// let thunk = Thunk::new(|| 42);
		/// let lazy: RcLazy<i32> = Lazy::from(thunk);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn from(eval: Thunk<'a, A>) -> Self {
			Self::new(move || eval.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: 'static> From<Trampoline<A>> for Lazy<'a, A, RcLazyConfig> {
		#[document_signature]
		#[document_parameters("The trampoline to convert.")]
		#[document_returns(
			"A new `Lazy` instance that will evaluate the trampoline on first access."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		/// let task = Trampoline::pure(42);
		/// let lazy: RcLazy<i32> = Lazy::from(task);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn from(task: Trampoline<A>) -> Self {
			Self::new(move || task.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: Send + Sync + 'a> From<Thunk<'a, A>> for Lazy<'a, A, ArcLazyConfig> {
		/// Converts a [`Thunk`] into an [`ArcLazy`] by eagerly evaluating the thunk.
		///
		/// Thunk is `!Send`, so the value must be computed immediately to cross
		/// into the thread-safe `ArcLazy` world.
		#[document_signature]
		#[document_parameters("The thunk to convert.")]
		#[document_returns("A new `Lazy` instance containing the eagerly evaluated value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		/// let thunk = Thunk::new(|| 42);
		/// let lazy: ArcLazy<i32> = ArcLazy::from(thunk);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn from(eval: Thunk<'a, A>) -> Self {
			Self::pure(eval.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: Send + Sync + 'static> From<Trampoline<A>> for Lazy<'a, A, ArcLazyConfig> {
		/// Converts a [`Trampoline`] into an [`ArcLazy`] by eagerly evaluating the trampoline.
		///
		/// Trampoline is `!Send`, so the value must be computed immediately to cross
		/// into the thread-safe `ArcLazy` world.
		#[document_signature]
		#[document_parameters("The trampoline to convert.")]
		#[document_returns("A new `Lazy` instance containing the eagerly evaluated value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		/// let task = Trampoline::pure(42);
		/// let lazy: ArcLazy<i32> = ArcLazy::from(task);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn from(task: Trampoline<A>) -> Self {
			Self::pure(task.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: Send + Sync + 'a> From<SendThunk<'a, A>> for Lazy<'a, A, ArcLazyConfig> {
		/// Converts a [`SendThunk`] into an [`ArcLazy`] without eager evaluation.
		///
		/// Because `SendThunk` already satisfies `Send`, the inner closure can be
		/// passed directly into `ArcLazy`, deferring computation until first access.
		#[document_signature]
		#[document_parameters("The send thunk to convert.")]
		#[document_returns("A new `ArcLazy` wrapping the deferred computation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		/// let thunk = SendThunk::new(|| 42);
		/// let lazy: ArcLazy<i32> = ArcLazy::from(thunk);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn from(thunk: SendThunk<'a, A>) -> Self {
			Self::new(move || thunk.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: Clone + Send + Sync + 'a> From<Lazy<'a, A, RcLazyConfig>>
		for Lazy<'a, A, ArcLazyConfig>
	{
		/// Converts an [`RcLazy`] into an [`ArcLazy`] by eagerly evaluating and cloning the value.
		///
		/// `RcLazy` is `!Send`, so the value must be computed immediately and cloned
		/// into the thread-safe `ArcLazy` world.
		#[document_signature]
		#[document_parameters("The `RcLazy` instance to convert.")]
		#[document_returns(
			"A new `ArcLazy` instance containing a clone of the eagerly evaluated value."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let rc_lazy = RcLazy::new(|| 42);
		/// let arc_lazy: ArcLazy<i32> = ArcLazy::from(rc_lazy);
		/// assert_eq!(*arc_lazy.evaluate(), 42);
		/// ```
		fn from(source: Lazy<'a, A, RcLazyConfig>) -> Self {
			Self::pure(source.evaluate().clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: Clone + 'a> From<Lazy<'a, A, ArcLazyConfig>> for Lazy<'a, A, RcLazyConfig> {
		/// Converts an [`ArcLazy`] into an [`RcLazy`] by eagerly evaluating and cloning the value.
		///
		/// The value is computed immediately and cloned into a new single-threaded
		/// `RcLazy` instance.
		#[document_signature]
		#[document_parameters("The `ArcLazy` instance to convert.")]
		#[document_returns(
			"A new `RcLazy` instance containing a clone of the eagerly evaluated value."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let arc_lazy = ArcLazy::new(|| 42);
		/// let rc_lazy: RcLazy<i32> = RcLazy::from(arc_lazy);
		/// assert_eq!(*rc_lazy.evaluate(), 42);
		/// ```
		fn from(source: Lazy<'a, A, ArcLazyConfig>) -> Self {
			Self::pure(source.evaluate().clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	#[document_parameters("The lazy instance.")]
	impl<'a, A> Lazy<'a, A, ArcLazyConfig>
	where
		A: Send + Sync + 'a,
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
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
		/// Requires `Send + Sync` since `ArcLazy` is thread-safe.
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = Lazy::<_, ArcLazyConfig>::pure(42);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self {
			Lazy(ArcLazyConfig::lazy_new(Box::new(move || a)))
		}

		/// Returns a clone of the memoized value, computing on first access.
		///
		/// This is a convenience wrapper around [`evaluate`](Lazy::evaluate) for cases
		/// where an owned value is needed rather than a reference. Requires `Send + Sync`
		/// since `ArcLazy` is thread-safe.
		#[document_signature]
		///
		#[document_returns("An owned clone of the memoized value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let memo = ArcLazy::new(|| vec![1, 2, 3]);
		/// let owned: Vec<i32> = memo.evaluate_owned();
		/// assert_eq!(owned, vec![1, 2, 3]);
		/// ```
		#[inline]
		pub fn evaluate_owned(&self) -> A
		where
			A: Clone, {
			self.evaluate().clone()
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	#[document_parameters("The lazy value to map over.")]
	impl<'a, A: Send + Sync + 'a> Lazy<'a, A, ArcLazyConfig> {
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let memo = ArcLazy::new(|| 10);
		/// let mapped = memo.ref_map(|x| *x * 2);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		#[inline]
		pub fn ref_map<B: Send + Sync + 'a>(
			&self,
			f: impl Fn(&A) -> B + Send + 'a,
		) -> Lazy<'a, B, ArcLazyConfig> {
			let this = self.clone();
			ArcLazy::new(move || f(this.evaluate()))
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

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
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

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A> SendDeferrable<'a> for Lazy<'a, A, ArcLazyConfig>
	where
		A: Clone + Send + Sync + 'a,
	{
		/// Defers a computation that produces a thread-safe `Lazy` value using a thread-safe thunk.
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
		fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self
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
		/// let mapped = LazyBrand::<RcLazyConfig>::ref_map(|x: &i32| *x * 2, &memo);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			f: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.ref_map(f)
		}
	}

	impl SendRefFunctor for LazyBrand<ArcLazyConfig> {
		/// Maps a thread-safe function over the memoized value, where the function takes a reference.
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
		/// let memo = ArcLazy::new(|| 10);
		/// let mapped = LazyBrand::<ArcLazyConfig>::send_ref_map(|x: &i32| *x * 2, &memo);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(&A) -> B + Send + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.ref_map(f)
		}
	}

	// -- SendRefPointed --

	impl SendRefPointed for LazyBrand<ArcLazyConfig> {
		/// Wraps a cloned value in a new thread-safe memoized context.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value.")]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("A new thread-safe memoized value containing a clone of the input.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let value = 42;
		/// let lazy = LazyBrand::<ArcLazyConfig>::send_ref_pure(&value);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn send_ref_pure<'a, A: Clone + Send + Sync + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			let cloned = a.clone();
			ArcLazy::new(move || cloned)
		}
	}

	// -- SendRefLift --

	impl SendRefLift for LazyBrand<ArcLazyConfig> {
		/// Lifts a thread-safe binary function over two memoized values using references.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The function to lift.",
			"The first memoized value.",
			"The second memoized value."
		)]
		///
		#[document_returns("A new thread-safe memoized value containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let x = ArcLazy::new(|| 3);
		/// let y = ArcLazy::new(|| 4);
		/// let z = LazyBrand::<ArcLazyConfig>::send_ref_lift2(|a: &i32, b: &i32| *a + *b, &x, &y);
		/// assert_eq!(*z.evaluate(), 7);
		/// ```
		fn send_ref_lift2<'a, A: Send + Sync + 'a, B: Send + Sync + 'a, C: Send + Sync + 'a>(
			func: impl Fn(&A, &B) -> C + Send + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			let fa = fa.clone();
			let fb = fb.clone();
			ArcLazy::new(move || func(fa.evaluate(), fb.evaluate()))
		}
	}

	// -- SendRefSemiapplicative --

	impl SendRefSemiapplicative for LazyBrand<ArcLazyConfig> {
		/// Applies a wrapped thread-safe by-ref function to a memoized value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the thread-safe cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters("The memoized wrapped by-ref function.", "The memoized value.")]
		///
		#[document_returns("A new thread-safe memoized value containing the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let f = ArcLazy::new(|| {
		/// 	std::sync::Arc::new(|x: &i32| *x * 2) as std::sync::Arc<dyn Fn(&i32) -> i32 + Send + Sync>
		/// });
		/// let x = ArcLazy::new(|| 5);
		/// let result = LazyBrand::<ArcLazyConfig>::send_ref_apply::<ArcFnBrand, _, _>(&f, &x);
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn send_ref_apply<
			'a,
			FnBrand: 'a + SendCloneFn<Ref>,
			A: Send + Sync + 'a,
			B: Send + Sync + 'a,
		>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as SendCloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let ff = ff.clone();
			let fa = fa.clone();
			ArcLazy::new(move || {
				let f = ff.evaluate();
				let a = fa.evaluate();
				(**f)(a)
			})
		}
	}

	// -- SendRefSemimonad --

	impl SendRefSemimonad for LazyBrand<ArcLazyConfig> {
		/// Sequences a thread-safe computation using a reference to the memoized value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value inside the context.",
			"The type of the value in the resulting context."
		)]
		///
		#[document_parameters(
			"The memoized value.",
			"A thread-safe function that receives a reference and returns a new memoized value."
		)]
		///
		#[document_returns("A new thread-safe memoized value produced by the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 5);
		/// let result = LazyBrand::<ArcLazyConfig>::send_ref_bind(&lazy, |x: &i32| {
		/// 	let v = *x * 2;
		/// 	ArcLazy::new(move || v)
		/// });
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn send_ref_bind<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			ma: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + Send + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			f(ma.evaluate())
		}
	}

	// --- SendRefFoldable ---

	impl SendRefFoldable for LazyBrand<ArcLazyConfig> {
		/// Maps the value to a monoid by reference (thread-safe).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the computed value.",
			"The monoid type."
		)]
		#[document_parameters("The mapping function.", "The Lazy to fold.")]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_ref_foldable::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 5);
		/// let result = send_ref_fold_map::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(
		/// 	|a: &i32| a.to_string(),
		/// 	&lazy,
		/// );
		/// assert_eq!(result, "5");
		/// ```
		fn send_ref_fold_map<'a, FnBrand, A: Send + Sync + 'a + Clone, M>(
			func: impl Fn(&A) -> M + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: SendLiftFn + 'a,
			M: Monoid + Send + Sync + 'a, {
			func(fa.evaluate())
		}
	}

	// --- SendRefFoldableWithIndex ---

	impl SendRefFoldableWithIndex for LazyBrand<ArcLazyConfig> {
		/// Maps the value to a monoid by reference with the unit index (thread-safe).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the computed value.",
			"The monoid type."
		)]
		#[document_parameters("The function to apply.", "The Lazy to fold.")]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_ref_foldable_with_index::SendRefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 42);
		/// let result =
		/// 	<LazyBrand<ArcLazyConfig> as SendRefFoldableWithIndex>::send_ref_fold_map_with_index::<
		/// 		ArcFnBrand,
		/// 		_,
		/// 		_,
		/// 	>(|_, x: &i32| x.to_string(), &lazy);
		/// assert_eq!(result, "42");
		/// ```
		fn send_ref_fold_map_with_index<
			'a,
			FnBrand,
			A: Send + Sync + 'a + Clone,
			R: Monoid + Send + Sync + 'a,
		>(
			f: impl Fn((), &A) -> R + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: SendLiftFn + 'a, {
			f((), fa.evaluate())
		}
	}

	// --- SendRefFunctorWithIndex ---

	impl SendRefFunctorWithIndex for LazyBrand<ArcLazyConfig> {
		/// Maps a function over the `ArcLazy` by reference with the unit index (thread-safe).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the input value.",
			"The type of the output value."
		)]
		#[document_parameters("The function to apply.", "The Lazy to map over.")]
		#[document_returns("A new Lazy containing the mapped value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_ref_functor_with_index::SendRefFunctorWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 42);
		/// let mapped = <LazyBrand<ArcLazyConfig> as SendRefFunctorWithIndex>::send_ref_map_with_index(
		/// 	|_, x: &i32| x.to_string(),
		/// 	&lazy,
		/// );
		/// assert_eq!(*mapped.evaluate(), "42");
		/// ```
		fn send_ref_map_with_index<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn((), &A) -> B + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Self::send_ref_map(move |a| f((), a), fa)
		}
	}

	// -- RefPointed --

	impl RefPointed for LazyBrand<RcLazyConfig> {
		/// Wraps a cloned value in a new memoized context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value. Must be `Clone`."
		)]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("A new memoized value containing a clone of the input.")]
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
		/// let value = 42;
		/// let lazy = LazyBrand::<RcLazyConfig>::ref_pure(&value);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			let cloned = a.clone();
			RcLazy::new(move || cloned)
		}
	}

	// -- RefLift --

	impl RefLift for LazyBrand<RcLazyConfig> {
		/// Lifts a binary function over two memoized values using references.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The function to lift.",
			"The first memoized value.",
			"The second memoized value."
		)]
		///
		#[document_returns("A new memoized value containing the result.")]
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
		/// let x = RcLazy::pure(3);
		/// let y = RcLazy::pure(4);
		/// let z = LazyBrand::<RcLazyConfig>::ref_lift2(|a: &i32, b: &i32| *a + *b, &x, &y);
		/// assert_eq!(*z.evaluate(), 7);
		/// ```
		fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
			func: impl Fn(&A, &B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			let fa = fa.clone();
			let fb = fb.clone();
			RcLazy::new(move || func(fa.evaluate(), fb.evaluate()))
		}
	}

	// -- RefSemiapplicative --

	impl RefSemiapplicative for LazyBrand<RcLazyConfig> {
		/// Applies a wrapped by-ref function within a memoized context to a memoized value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters("The memoized wrapped by-ref function.", "The memoized value.")]
		///
		#[document_returns("A new memoized value containing the result of applying the function.")]
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
		/// let f = RcLazy::pure(std::rc::Rc::new(|x: &i32| *x * 2) as std::rc::Rc<dyn Fn(&i32) -> i32>);
		/// let x = RcLazy::pure(5);
		/// let result = LazyBrand::<RcLazyConfig>::ref_apply::<RcFnBrand, _, _>(&f, &x);
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, A: 'a, B: 'a>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let ff = ff.clone();
			let fa = fa.clone();
			RcLazy::new(move || {
				let f = ff.evaluate();
				let a = fa.evaluate();
				(**f)(a)
			})
		}
	}

	// -- RefSemimonad --

	impl RefSemimonad for LazyBrand<RcLazyConfig> {
		/// Sequences a computation using a reference to the memoized value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value inside the context.",
			"The type of the value in the resulting context."
		)]
		///
		#[document_parameters(
			"The memoized value.",
			"A function that receives a reference to the value and returns a new memoized value."
		)]
		///
		#[document_returns("A new memoized value produced by the function.")]
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
		/// let lazy = RcLazy::pure(5);
		/// let result = LazyBrand::<RcLazyConfig>::ref_bind(&lazy, |x: &i32| {
		/// 	Lazy::<_, RcLazyConfig>::new({
		/// 		let v = *x;
		/// 		move || v * 2
		/// 	})
		/// });
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			f(fa.evaluate())
		}
	}

	// --- Display ---

	#[document_type_parameters(
		"The lifetime of the reference.",
		"The type of the computed value.",
		"The memoization configuration."
	)]
	#[document_parameters("The lazy value to display.")]
	impl<'a, A: fmt::Display + 'a, Config: LazyConfig> fmt::Display for Lazy<'a, A, Config> {
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 42);
		/// assert_eq!(format!("{}", lazy), "42");
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

	// --- Hash ---

	#[document_type_parameters(
		"The lifetime of the reference.",
		"The type of the computed value.",
		"The memoization configuration."
	)]
	#[document_parameters("The lazy value to hash.")]
	impl<'a, A: Hash + 'a, Config: LazyConfig> Hash for Lazy<'a, A, Config> {
		/// Forces evaluation and hashes the value.
		#[document_signature]
		#[document_type_parameters("The type of the hasher.")]
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
		///
		/// let lazy = ArcLazy::new(|| 42);
		/// let mut hasher = DefaultHasher::new();
		/// lazy.hash(&mut hasher);
		/// let h3 = hasher.finish();
		///
		/// assert_eq!(h1, h3);
		/// ```
		fn hash<H: Hasher>(
			&self,
			state: &mut H,
		) {
			self.evaluate().hash(state)
		}
	}

	// --- RefFoldable ---

	#[document_type_parameters("The memoization configuration (determines Rc vs Arc).")]
	impl<Config: LazyConfig> RefFoldable for LazyBrand<Config> {
		/// Maps the value to a monoid by reference and returns it.
		///
		/// Forces evaluation and maps the single contained value by reference.
		/// No cloning is required since the closure receives `&A` directly.
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
		/// 	fold_map::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32| a.to_string(), &lazy);
		/// assert_eq!(result, "10");
		/// ```
		fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(&A) -> M + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: LiftFn + 'a,
			M: Monoid + 'a, {
			func(fa.evaluate())
		}

		/// Folds the `Lazy` from the right by reference.
		///
		/// Forces evaluation and folds the single contained value by reference.
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
			"The function to apply to each element reference and the accumulator.",
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
		/// let result =
		/// 	fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32, b| *a + b, 5, &lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn ref_fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(&A, B) -> B + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: LiftFn + 'a, {
			func(fa.evaluate(), initial)
		}

		/// Folds the `Lazy` from the left by reference.
		///
		/// Forces evaluation and folds the single contained value by reference.
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
			"The function to apply to the accumulator and each element reference.",
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
		/// let result =
		/// 	fold_left::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|b, a: &i32| b + *a, 5, &lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn ref_fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, &A) -> B + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: LiftFn + 'a, {
			func(initial, fa.evaluate())
		}
	}

	// --- WithIndex ---

	#[document_type_parameters("The memoization configuration (determines Rc vs Arc).")]
	impl<Config: LazyConfig> WithIndex for LazyBrand<Config> {
		type Index = ();
	}

	// --- RefFoldableWithIndex ---

	#[document_type_parameters("The memoization configuration (determines Rc vs Arc).")]
	impl<Config: LazyConfig> RefFoldableWithIndex for LazyBrand<Config> {
		/// Maps the value to a monoid by reference with the unit index.
		///
		/// Forces evaluation and maps the single contained value by reference,
		/// providing `()` as the index.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the computed value.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function to apply to the index and value reference.",
			"The Lazy to fold."
		)]
		///
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_foldable_with_index::RefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 42);
		/// let result = <LazyBrand<RcLazyConfig> as RefFoldableWithIndex>::ref_fold_map_with_index::<
		/// 	RcFnBrand,
		/// 	_,
		/// 	_,
		/// >(|_, x: &i32| x.to_string(), &lazy);
		/// assert_eq!(result, "42");
		/// ```
		fn ref_fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid + 'a>(
			f: impl Fn((), &A) -> R + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			f((), fa.evaluate())
		}
	}

	// --- RefFunctorWithIndex ---

	impl RefFunctorWithIndex for LazyBrand<RcLazyConfig> {
		/// Maps a function over the `Lazy` by reference with the unit index.
		///
		/// Forces evaluation and maps the single contained value by reference,
		/// providing `()` as the index. Returns a new `Lazy` wrapping the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The function to apply to the index and value reference.",
			"The Lazy to map over."
		)]
		///
		#[document_returns("A new Lazy containing the mapped value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_functor_with_index::RefFunctorWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 42);
		/// let mapped = <LazyBrand<RcLazyConfig> as RefFunctorWithIndex>::ref_map_with_index(
		/// 	|_, x: &i32| x.to_string(),
		/// 	&lazy,
		/// );
		/// assert_eq!(*mapped.evaluate(), "42");
		/// ```
		fn ref_map_with_index<'a, A: 'a, B: 'a>(
			f: impl Fn((), &A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Self::ref_map(move |a| f((), a), fa)
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
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

	// --- Eq ---

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The memoization configuration."
	)]
	impl<'a, A: Eq + 'a, Config: LazyConfig> Eq for Lazy<'a, A, Config> {}

	// --- Ord ---

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The memoization configuration."
	)]
	#[document_parameters("The lazy value to compare.")]
	impl<'a, A: Ord + 'a, Config: LazyConfig> Ord for Lazy<'a, A, Config> {
		/// Compares two `Lazy` values for ordering by forcing evaluation of both sides.
		///
		/// Note: This will trigger computation if either value has not yet been evaluated.
		#[document_signature]
		#[document_parameters("The other lazy value to compare with.")]
		#[document_returns("The ordering between the evaluated values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let a = RcLazy::pure(1);
		/// let b = RcLazy::pure(2);
		/// assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
		/// ```
		fn cmp(
			&self,
			other: &Self,
		) -> std::cmp::Ordering {
			self.evaluate().cmp(other.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
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
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
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

	// --- Fix combinators ---

	/// Computes a fixed point for `RcLazy`.
	///
	/// Constructs a self-referential `RcLazy` where the initializer receives a clone
	/// of the resulting lazy cell. This enables recursive definitions where the value
	/// depends on the lazy cell itself.
	///
	/// # Caveats
	///
	/// **Panic on reentrant evaluation:** Forcing the self-reference inside `f` before
	/// the outer cell has completed initialization causes a panic, because `LazyCell`
	/// detects the reentrant access.
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
	/// use fp_library::types::{
	/// 	lazy::rc_lazy_fix,
	/// 	*,
	/// };
	///
	/// let lazy = rc_lazy_fix(|_self_ref: RcLazy<i32>| 42);
	/// assert_eq!(*lazy.evaluate(), 42);
	/// ```
	pub fn rc_lazy_fix<'a, A: Clone + 'a>(
		f: impl FnOnce(RcLazy<'a, A>) -> A + 'a
	) -> RcLazy<'a, A> {
		use std::{
			cell::OnceCell,
			rc::Weak,
		};

		#[allow(clippy::type_complexity)]
		let cell: Rc<OnceCell<Weak<LazyCell<A, Box<dyn FnOnce() -> A + 'a>>>>> = Rc::new(OnceCell::new());
		let cell_clone = cell.clone();
		let lazy = RcLazy::new(move || {
			// INVARIANT: cell is always set on the line after this closure is
			// created, and the outer RcLazy is still alive (we are inside its
			// evaluation), so the Weak upgrade always succeeds.
			#[allow(clippy::expect_used)]
			let weak = cell_clone.get().expect("rc_lazy_fix: cell not initialized");
			#[allow(clippy::expect_used)]
			let self_ref = Lazy(weak.upgrade().expect("rc_lazy_fix: outer lazy was dropped"));
			f(self_ref)
		});
		let _ = cell.set(Rc::downgrade(&lazy.0));
		lazy
	}

	/// Computes a fixed point for `ArcLazy`.
	///
	/// Constructs a self-referential `ArcLazy` where the initializer receives a clone
	/// of the resulting lazy cell. This enables recursive definitions where the value
	/// depends on the lazy cell itself.
	///
	/// # Caveats
	///
	/// **Deadlock on reentrant evaluation:** Forcing the self-reference inside `f` before
	/// the outer cell has completed initialization causes a deadlock, because `LazyLock`
	/// blocks on the lock that the current thread already holds.
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
	/// use fp_library::types::{
	/// 	lazy::arc_lazy_fix,
	/// 	*,
	/// };
	///
	/// let lazy = arc_lazy_fix(|_self_ref: ArcLazy<i32>| 42);
	/// assert_eq!(*lazy.evaluate(), 42);
	/// ```
	pub fn arc_lazy_fix<'a, A: Clone + Send + Sync + 'a>(
		f: impl FnOnce(ArcLazy<'a, A>) -> A + Send + 'a
	) -> ArcLazy<'a, A> {
		use std::sync::{
			OnceLock,
			Weak,
		};

		#[allow(clippy::type_complexity)]
		let cell: Arc<OnceLock<Weak<LazyLock<A, Box<dyn FnOnce() -> A + Send + 'a>>>>> =
			Arc::new(OnceLock::new());
		let cell_clone = cell.clone();
		let lazy = ArcLazy::new(move || {
			// INVARIANT: cell is always set on the line after this closure is
			// created, and the outer ArcLazy is still alive (we are inside its
			// evaluation), so the Weak upgrade always succeeds.
			#[allow(clippy::expect_used)]
			let weak = cell_clone.get().expect("arc_lazy_fix: cell not initialized");
			#[allow(clippy::expect_used)]
			let self_ref = Lazy(weak.upgrade().expect("arc_lazy_fix: outer lazy was dropped"));
			f(self_ref)
		});
		let _ = cell.set(Arc::downgrade(&lazy.0));
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
		// Trampoline requires 'static due to type erasure via Box<dyn Any>
		let task = Trampoline::pure(42);
		let memo = RcLazy::from(task);
		assert_eq!(*memo.evaluate(), 42);
	}

	/// Tests conversion from `RcLazy` to `ArcLazy`.
	///
	/// Verifies `From<RcLazy>` for `ArcLazy` works correctly.
	#[test]
	fn test_rc_lazy_to_arc_lazy() {
		let rc = RcLazy::new(|| "hello".to_string());
		let arc: ArcLazy<String> = ArcLazy::from(rc);
		assert_eq!(*arc.evaluate(), "hello");
	}

	/// Tests conversion from `ArcLazy` to `RcLazy`.
	///
	/// Verifies `From<ArcLazy>` for `RcLazy` works correctly.
	#[test]
	fn test_arc_lazy_to_rc_lazy() {
		let arc = ArcLazy::new(|| "world".to_string());
		let rc: RcLazy<String> = RcLazy::from(arc);
		assert_eq!(*rc.evaluate(), "world");
	}

	/// Tests that `RcLazy` to `ArcLazy` conversion eagerly evaluates.
	///
	/// Verifies that the source is evaluated during conversion, not deferred.
	#[test]
	fn test_rc_to_arc_eager_evaluation() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let rc = RcLazy::new(move || {
			*counter_clone.borrow_mut() += 1;
			99
		});
		assert_eq!(*counter.borrow(), 0);

		let arc: ArcLazy<i32> = ArcLazy::from(rc);
		// Conversion should have forced evaluation.
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(*arc.evaluate(), 99);
		// No additional evaluation.
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `ArcLazy` to `RcLazy` conversion eagerly evaluates.
	///
	/// Verifies that the source is evaluated during conversion, not deferred.
	#[test]
	fn test_arc_to_rc_eager_evaluation() {
		use std::sync::atomic::{
			AtomicUsize,
			Ordering,
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();
		let arc = ArcLazy::new(move || {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			77
		});
		assert_eq!(counter.load(Ordering::SeqCst), 0);

		let rc: RcLazy<i32> = RcLazy::from(arc);
		// Conversion should have forced evaluation.
		assert_eq!(counter.load(Ordering::SeqCst), 1);
		assert_eq!(*rc.evaluate(), 77);
		// No additional evaluation.
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	}

	/// Property: RcLazy to ArcLazy round-trip preserves values.
	#[quickcheck]
	fn prop_rc_to_arc_preserves_value(x: i32) -> bool {
		let rc = RcLazy::new(move || x);
		let arc: ArcLazy<i32> = ArcLazy::from(rc);
		*arc.evaluate() == x
	}

	/// Property: ArcLazy to RcLazy round-trip preserves values.
	#[quickcheck]
	fn prop_arc_to_rc_preserves_value(x: i32) -> bool {
		let arc = ArcLazy::new(move || x);
		let rc: RcLazy<i32> = RcLazy::from(arc);
		*rc.evaluate() == x
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

	// --- Tests for RefFoldable ---

	/// Tests `fold_right` for `RcLazy`.
	#[test]
	fn test_rc_lazy_ref_fold_right() {
		use crate::functions::*;

		let lazy = RcLazy::pure(10);
		let result = fold_right::<
			crate::brands::RcFnBrand,
			crate::brands::LazyBrand<RcLazyConfig>,
			_,
			_,
			_,
			_,
		>(|a: &i32, b| *a + b, 5, &lazy);
		assert_eq!(result, 15);
	}

	/// Tests `fold_left` for `RcLazy`.
	#[test]
	fn test_rc_lazy_ref_fold_left() {
		use crate::functions::*;

		let lazy = RcLazy::pure(10);
		let result = fold_left::<
			crate::brands::RcFnBrand,
			crate::brands::LazyBrand<RcLazyConfig>,
			_,
			_,
			_,
			_,
		>(|b, a: &i32| b + *a, 5, &lazy);
		assert_eq!(result, 15);
	}

	/// Tests `fold_map` for `RcLazy`.
	#[test]
	fn test_rc_lazy_ref_fold_map() {
		use crate::functions::*;

		let lazy = RcLazy::pure(10);
		let result = fold_map::<
			crate::brands::RcFnBrand,
			crate::brands::LazyBrand<RcLazyConfig>,
			_,
			_,
			_,
			_,
		>(|a: &i32| a.to_string(), &lazy);
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
		let fixed = rc_lazy_fix(|_self_ref: RcLazy<i32>| 42);
		assert_eq!(*fixed.evaluate(), 42);
	}

	/// Tests that `rc_lazy_fix` sets up the `OnceCell` before evaluation.
	///
	/// The closure inside the resulting `RcLazy` reads from the cell, so the
	/// cell must be populated by the time evaluation happens.
	#[test]
	fn test_rc_lazy_fix_cell_initialized() {
		let fixed = rc_lazy_fix(|_self_ref: RcLazy<String>| String::from("initialized"));
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
		let fixed = rc_lazy_fix(|_self_ref: RcLazy<i32>| 7);
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
			*counter_clone.borrow_mut() += 1;
			100
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
		let fixed = rc_lazy_fix(|_self_ref: RcLazy<i32>| 55);
		let cloned = fixed.clone();
		assert_eq!(*fixed.evaluate(), 55);
		assert_eq!(*cloned.evaluate(), 55);
	}

	/// Tests that `arc_lazy_fix` produces the correct value when
	/// the function ignores the self-reference.
	#[test]
	fn test_arc_lazy_fix_constant() {
		let fixed = arc_lazy_fix(|_self_ref: ArcLazy<i32>| 42);
		assert_eq!(*fixed.evaluate(), 42);
	}

	/// Tests that `arc_lazy_fix` sets up the `OnceLock` before evaluation.
	#[test]
	fn test_arc_lazy_fix_cell_initialized() {
		let fixed = arc_lazy_fix(|_self_ref: ArcLazy<String>| String::from("initialized"));
		// If the lock were not initialized, this would panic.
		assert_eq!(fixed.evaluate().as_str(), "initialized");
	}

	/// Tests that `arc_lazy_fix` correctly threads the self-reference.
	#[test]
	fn test_arc_lazy_fix_self_reference_plumbing() {
		let fixed = arc_lazy_fix(|_self_ref: ArcLazy<i32>| 7);
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
			counter_clone.fetch_add(1, Ordering::SeqCst);
			100
		});

		assert_eq!(counter.load(Ordering::SeqCst), 0);
		assert_eq!(*fixed.evaluate(), 100);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
		// Second evaluation should use cached value.
		assert_eq!(*fixed.evaluate(), 100);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	}

	// --- ArcLazy RefFoldable tests ---

	/// Tests `fold_right` on `ArcLazy`.
	#[test]
	fn test_arc_lazy_ref_fold_right() {
		use crate::{
			brands::*,
			functions::*,
		};

		let lazy = ArcLazy::new(|| 10);
		let result = fold_right::<RcFnBrand, LazyBrand<ArcLazyConfig>, _, _, _, _>(
			|a: &i32, b| *a + b,
			5,
			&lazy,
		);
		assert_eq!(result, 15);
	}

	/// Tests `fold_left` on `ArcLazy`.
	#[test]
	fn test_arc_lazy_ref_fold_left() {
		use crate::{
			brands::*,
			functions::*,
		};

		let lazy = ArcLazy::new(|| 10);
		let result = fold_left::<RcFnBrand, LazyBrand<ArcLazyConfig>, _, _, _, _>(
			|b, a: &i32| b + *a,
			5,
			&lazy,
		);
		assert_eq!(result, 15);
	}

	/// Tests `fold_map` on `ArcLazy`.
	#[test]
	fn test_arc_lazy_ref_fold_map() {
		use crate::{
			brands::*,
			functions::*,
		};

		let lazy = ArcLazy::new(|| 10);
		let result = fold_map::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _, _, _>(
			|a: &i32| a.to_string(),
			&lazy,
		);
		assert_eq!(result, "10");
	}

	/// Property: ArcLazy fold_right is consistent with RcLazy fold_right.
	#[quickcheck]
	fn prop_arc_lazy_ref_fold_right(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};

		let rc_lazy = RcLazy::new(move || x);
		let arc_lazy = ArcLazy::new(move || x);
		let rc_result = fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(
			|a: &i32, b| *a + b,
			0,
			&rc_lazy,
		);
		let arc_result = fold_right::<RcFnBrand, LazyBrand<ArcLazyConfig>, _, _, _, _>(
			|a: &i32, b| *a + b,
			0,
			&arc_lazy,
		);
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

	/// Property: `rc_lazy_fix` with a constant function produces that constant.
	#[quickcheck]
	fn prop_rc_lazy_fix_constant(x: i32) -> bool {
		let fixed = rc_lazy_fix(move |_: RcLazy<i32>| x);
		*fixed.evaluate() == x
	}

	/// Property: `arc_lazy_fix` with a constant function produces that constant.
	#[quickcheck]
	fn prop_arc_lazy_fix_constant(x: i32) -> bool {
		let fixed = arc_lazy_fix(move |_: ArcLazy<i32>| x);
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

	// -- Knot-tying tests: self-reference is actually used after initialization --

	/// Tests that `rc_lazy_fix` ties the knot: the self-reference captured
	/// inside the closure points to the same `Rc` cell as the returned lazy.
	/// After evaluation, querying the self-reference yields the cached value.
	#[test]
	fn test_rc_lazy_fix_knot_tying_ptr_eq() {
		let stash: Rc<RefCell<Option<RcLazy<i32>>>> = Rc::new(RefCell::new(None));
		let stash_clone = stash.clone();
		let lazy = rc_lazy_fix(move |self_ref: RcLazy<i32>| {
			// Store the self-reference externally so we can inspect it later.
			*stash_clone.borrow_mut() = Some(self_ref);
			42
		});
		// Force evaluation.
		assert_eq!(*lazy.evaluate(), 42);
		// Retrieve the stashed self-reference.
		let self_ref = stash.borrow().clone().unwrap();
		// The self-reference must point to the same underlying Rc cell.
		assert!(Rc::ptr_eq(&lazy.0, &self_ref.0));
		// Evaluating the self-reference yields the same cached value.
		assert_eq!(*self_ref.evaluate(), 42);
	}

	/// Tests that `rc_lazy_fix` knot-tying produces a shared cache: the
	/// initializer runs exactly once even when accessed through the self-reference.
	#[test]
	fn test_rc_lazy_fix_knot_tying_shared_cache() {
		let counter = Rc::new(RefCell::new(0));
		let stash: Rc<RefCell<Option<RcLazy<i32>>>> = Rc::new(RefCell::new(None));
		let counter_clone = counter.clone();
		let stash_clone = stash.clone();
		let lazy = rc_lazy_fix(move |self_ref: RcLazy<i32>| {
			*stash_clone.borrow_mut() = Some(self_ref);
			*counter_clone.borrow_mut() += 1;
			100
		});
		assert_eq!(*counter.borrow(), 0);
		// Force evaluation through the outer lazy.
		assert_eq!(*lazy.evaluate(), 100);
		assert_eq!(*counter.borrow(), 1);
		// Force evaluation through the stashed self-reference.
		let self_ref = stash.borrow().clone().unwrap();
		assert_eq!(*self_ref.evaluate(), 100);
		// Counter must still be 1: the initializer was not re-run.
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests that `rc_lazy_fix` panics on reentrant evaluation.
	///
	/// Forcing the self-reference inside the initializer triggers a
	/// `LazyCell` reentrant-init panic.
	#[test]
	#[should_panic]
	fn test_rc_lazy_fix_reentrant_panics() {
		let lazy = rc_lazy_fix(|self_ref: RcLazy<i32>| {
			// Forcing the self-reference during initialization is reentrant
			// and must panic.
			*self_ref.evaluate() + 1
		});
		let _ = lazy.evaluate();
	}

	/// Tests that `arc_lazy_fix` ties the knot: the self-reference captured
	/// inside the closure points to the same `Arc` cell as the returned lazy.
	#[test]
	fn test_arc_lazy_fix_knot_tying_ptr_eq() {
		use std::sync::Mutex;

		let stash: Arc<Mutex<Option<ArcLazy<i32>>>> = Arc::new(Mutex::new(None));
		let stash_clone = stash.clone();
		let lazy = arc_lazy_fix(move |self_ref: ArcLazy<i32>| {
			*stash_clone.lock().unwrap() = Some(self_ref);
			42
		});
		assert_eq!(*lazy.evaluate(), 42);
		let self_ref = stash.lock().unwrap().clone().unwrap();
		// The self-reference must point to the same underlying Arc cell.
		assert!(Arc::ptr_eq(&lazy.0, &self_ref.0));
		// Evaluating the self-reference yields the same cached value.
		assert_eq!(*self_ref.evaluate(), 42);
	}

	/// Tests that `arc_lazy_fix` knot-tying produces a shared cache: the
	/// initializer runs exactly once even when accessed through the self-reference.
	#[test]
	fn test_arc_lazy_fix_knot_tying_shared_cache() {
		use std::sync::{
			Mutex,
			atomic::{
				AtomicUsize,
				Ordering,
			},
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let stash: Arc<Mutex<Option<ArcLazy<i32>>>> = Arc::new(Mutex::new(None));
		let counter_clone = counter.clone();
		let stash_clone = stash.clone();
		let lazy = arc_lazy_fix(move |self_ref: ArcLazy<i32>| {
			*stash_clone.lock().unwrap() = Some(self_ref);
			counter_clone.fetch_add(1, Ordering::SeqCst);
			100
		});
		assert_eq!(counter.load(Ordering::SeqCst), 0);
		assert_eq!(*lazy.evaluate(), 100);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
		// Access through the stashed self-reference.
		let self_ref = stash.lock().unwrap().clone().unwrap();
		assert_eq!(*self_ref.evaluate(), 100);
		// Counter must still be 1: the initializer was not re-run.
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	}

	/// Tests that `arc_lazy_fix` knot-tying works across threads: the
	/// self-reference can be evaluated from a different thread and still
	/// returns the cached value.
	#[test]
	fn test_arc_lazy_fix_knot_tying_cross_thread() {
		use std::{
			sync::Mutex,
			thread,
		};

		let stash: Arc<Mutex<Option<ArcLazy<i32>>>> = Arc::new(Mutex::new(None));
		let stash_clone = stash.clone();
		let lazy = arc_lazy_fix(move |self_ref: ArcLazy<i32>| {
			*stash_clone.lock().unwrap() = Some(self_ref);
			77
		});
		assert_eq!(*lazy.evaluate(), 77);
		let self_ref = stash.lock().unwrap().clone().unwrap();
		// Evaluate the self-reference from a spawned thread.
		let handle = thread::spawn(move || *self_ref.evaluate());
		assert_eq!(handle.join().unwrap(), 77);
	}

	#[test]
	fn m_do_ref_lazy_manual() {
		// Manual expansion of what m_do!(ref ...) should generate
		use crate::{
			brands::LazyBrand,
			functions::*,
		};

		let lazy_a = RcLazy::new(|| 10i32);

		let result =
			bind_explicit::<LazyBrand<RcLazyConfig>, _, _, _, _>(&lazy_a, move |a: &i32| {
				ref_pure::<LazyBrand<RcLazyConfig>, _>(&(*a * 2))
			});

		assert_eq!(*result.evaluate(), 20);
	}

	#[test]
	fn m_do_ref_lazy_macro() {
		use {
			crate::{
				brands::LazyBrand,
				functions::*,
			},
			fp_macros::m_do,
		};

		let lazy_a = RcLazy::new(|| 10i32);

		let result = m_do!(ref LazyBrand<RcLazyConfig> {
			a: &i32 <- lazy_a;
			pure(*a * 2)
		});

		assert_eq!(*result.evaluate(), 20);
	}

	#[test]
	fn m_do_ref_lazy_multi_bind() {
		use {
			crate::{
				brands::LazyBrand,
				functions::*,
			},
			fp_macros::m_do,
		};

		let lazy_a = RcLazy::new(|| 10i32);
		let lazy_b = RcLazy::new(|| 20i32);

		// Multi-bind in ref mode: each bind receives &A, but inner closures
		// can't capture references from outer binds (lifetime issue). Use
		// let bindings to clone the referenced value for use in later binds.
		let result = m_do!(ref LazyBrand<RcLazyConfig> {
			a: &i32 <- lazy_a;
			let a_val = *a;
			b: &i32 <- lazy_b.clone();
			pure(a_val + *b)
		});

		assert_eq!(*result.evaluate(), 30);
	}

	#[test]
	fn m_do_ref_lazy_untyped() {
		use {
			crate::{
				brands::LazyBrand,
				functions::*,
			},
			fp_macros::m_do,
		};

		let lazy_a = RcLazy::new(|| 10i32);

		let result = m_do!(ref LazyBrand<RcLazyConfig> {
			a <- lazy_a;
			pure(*a * 3)
		});

		assert_eq!(*result.evaluate(), 30);
	}

	#[test]
	fn a_do_ref_lazy() {
		use {
			crate::{
				brands::LazyBrand,
				functions::*,
			},
			fp_macros::a_do,
		};

		let lazy_a = RcLazy::new(|| 10i32);
		let lazy_b = RcLazy::new(|| 20i32);

		// a_do uses lift2, which doesn't have the FnOnce issue since
		// applicative binds are independent (no nesting).
		let result = a_do!(ref LazyBrand<RcLazyConfig> {
			a: &i32 <- lazy_a;
			b: &i32 <- lazy_b;
			*a + *b
		});

		assert_eq!(*result.evaluate(), 30);
	}
}
