//! Memoized lazy evaluation for fallible computations.
//!
//! Computes a [`Result`] at most once and caches either the success value or error. All clones
//! share the same cache. Available in both single-threaded [`RcTryLazy`] and thread-safe
//! [`ArcTryLazy`] variants.
//!
//! # `Foldable` and error discarding
//!
//! The [`Foldable`](crate::classes::Foldable) implementation for
//! [`TryLazyBrand<E, Config>`](crate::brands::TryLazyBrand) treats `TryLazy` as a container of
//! zero or one success values. If the computation produces an `Err`, the fold sees an empty
//! container and returns the initial accumulator unchanged. The error value is silently discarded.
//! Use [`evaluate`](TryLazy::evaluate) directly if you need to inspect or handle errors.
//!
//! # Choosing between `TryLazy`, `Lazy<Result<A, E>>`, and `Result<Lazy, E>`
//!
//! - **[`TryLazy<A, E>`](TryLazy)**: Use when the computation itself may fail, and you want
//!   memoization of the entire outcome (success or error). The result is computed at most once;
//!   subsequent accesses return the cached `Ok` or `Err`.
//! - **`Lazy<Result<A, E>>`**: Equivalent in caching behavior, but does not participate in the
//!   `TryLazy` combinator API (`map_err`, `and_then`, `or_else`). Prefer `TryLazy` for
//!   ergonomics when working with fallible computations.
//! - **`Result<Lazy<A>, E>`**: The error is known eagerly (before any lazy computation). Use
//!   this when failure is detected up front and only the success path involves deferred work.
//!
//! # `TryLazy::map` vs `Lazy::ref_map` naming
//!
//! [`Lazy::ref_map`](crate::types::Lazy::ref_map) is named `ref_map` because it receives a
//! `&A` reference (the memoized value is borrowed, not moved). [`TryLazy::map`](TryLazy::map)
//! follows the same convention internally (the closure receives `&A` from the cached result), but
//! uses the name `map` to mirror [`Result::map`] and the other fallible-type combinators
//! (`map_err`, `and_then`, `or_else`). The mapping closure must clone or derive a new `B` from
//! the reference.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			Kind,
			brands::TryLazyBrand,
			classes::{
				CloneableFn,
				Deferrable,
				Foldable,
				Semigroup,
				SendDeferrable,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcLazyConfig,
				Lazy,
				LazyConfig,
				RcLazyConfig,
				TryThunk,
				TryTrampoline,
			},
		},
		fp_macros::*,
	};

	/// A lazily-computed, memoized value that may fail.
	///
	/// The computation runs at most once. If it succeeds, the value is cached.
	/// If it fails, the error is cached. Subsequent accesses return the cached result.
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error.",
		"The memoization configuration."
	)]
	///
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is [`TryLazyBrand<E, Config>`](crate::brands::TryLazyBrand),
	/// which is parameterized by both the error type and the `LazyConfig`, and is polymorphic over the success value type.
	pub struct TryLazy<'a, A, E, Config: LazyConfig = RcLazyConfig>(
		/// The internal lazy cell.
		pub(crate) Config::TryLazy<'a, A, E>,
	)
	where
		A: 'a,
		E: 'a;

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error.",
		"The memoization configuration."
	)]
	#[document_parameters("The instance to clone.")]
	impl<'a, A, E, Config: LazyConfig> Clone for TryLazy<'a, A, E, Config>
	where
		A: 'a,
		E: 'a,
	{
		#[document_signature]
		#[document_returns(
			"A new `TryLazy` instance that shares the same underlying memoized result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let memo = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// let cloned = memo.clone();
		/// assert_eq!(cloned.evaluate(), Ok(&42));
		/// ```
		fn clone(&self) -> Self {
			Self(self.0.clone())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error.",
		"The memoization configuration."
	)]
	#[document_parameters("The `TryLazy` instance.")]
	impl<'a, A, E, Config: LazyConfig> TryLazy<'a, A, E, Config>
	where
		A: 'a,
		E: 'a,
	{
		/// Gets the memoized result, computing on first access.
		#[document_signature]
		///
		#[document_returns("A result containing a reference to the value or error.")]
		///
		#[document_examples]
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

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> TryLazy<'a, A, E, RcLazyConfig>
	where
		A: 'a,
		E: 'a,
	{
		/// Creates a new `TryLazy` that will run `f` on first access.
		#[document_signature]
		///
		#[document_parameters("The closure that produces the result.")]
		///
		#[document_returns("A new `TryLazy` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		pub fn new(f: impl FnOnce() -> Result<A, E> + 'a) -> Self {
			TryLazy(RcLazyConfig::try_lazy_new(Box::new(f)))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> From<TryThunk<'a, A, E>> for TryLazy<'a, A, E, RcLazyConfig> {
		#[document_signature]
		#[document_parameters("The fallible thunk to convert.")]
		#[document_returns(
			"A new `TryLazy` instance that will evaluate the thunk on first access."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = TryThunk::new(|| Ok::<i32, ()>(42));
		/// let memo = TryLazy::from(thunk);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		fn from(eval: TryThunk<'a, A, E>) -> Self {
			Self::new(move || eval.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> From<TryTrampoline<A, E>> for TryLazy<'a, A, E, RcLazyConfig>
	where
		A: Send,
		E: Send,
	{
		#[document_signature]
		#[document_parameters("The fallible trampoline to convert.")]
		#[document_returns(
			"A new `TryLazy` instance that will evaluate the trampoline on first access."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let task = TryTrampoline::<_, ()>::ok(42);
		/// let memo: TryLazy<_, (), _> = TryLazy::from(task);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		fn from(task: TryTrampoline<A, E>) -> Self {
			Self::new(move || task.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> From<Lazy<'a, A, ArcLazyConfig>> for TryLazy<'a, A, E, ArcLazyConfig>
	where
		A: Clone + Send + Sync + 'a,
		E: Send + Sync + 'a,
	{
		#[document_signature]
		#[document_parameters("The thread-safe lazy value to convert.")]
		#[document_returns("A new `TryLazy` instance that wraps the lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, ArcLazyConfig>::pure(42);
		/// let memo: TryLazy<_, (), _> = TryLazy::from(lazy);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		fn from(memo: Lazy<'a, A, ArcLazyConfig>) -> Self {
			Self::new(move || Ok(memo.evaluate().clone()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> From<Lazy<'a, A, RcLazyConfig>> for TryLazy<'a, A, E, RcLazyConfig>
	where
		A: Clone + 'a,
		E: 'a,
	{
		#[document_signature]
		#[document_parameters("The lazy value to convert.")]
		#[document_returns("A new `TryLazy` instance that wraps the lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
		/// let memo: TryLazy<_, (), _> = TryLazy::from(lazy);
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		fn from(memo: Lazy<'a, A, RcLazyConfig>) -> Self {
			Self::new(move || Ok(memo.evaluate().clone()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A> TryLazy<'a, A, String, RcLazyConfig>
	where
		A: 'a,
	{
		/// Creates a `TryLazy` that catches unwinds (panics).
		#[document_signature]
		///
		#[document_parameters("The closure that might panic.")]
		///
		#[document_returns("A new `TryLazy` instance where panics are converted to `Err(String)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = TryLazy::<_, String, RcLazyConfig>::catch_unwind(|| {
		/// 	if true {
		/// 		panic!("oops")
		/// 	}
		/// 	42
		/// });
		/// assert_eq!(memo.evaluate(), Err(&"oops".to_string()));
		/// ```
		pub fn catch_unwind(f: impl FnOnce() -> A + std::panic::UnwindSafe + 'a) -> Self {
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

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	#[document_parameters("The `TryLazy` instance.")]
	impl<'a, A, E> TryLazy<'a, A, E, RcLazyConfig>
	where
		A: Clone + 'a,
		E: Clone + 'a,
	{
		/// Transforms the success value, producing a new memoized `TryLazy`.
		///
		/// The mapping closure receives a `&A` reference to the cached value. If the
		/// computation produced an `Err`, the error is cloned into the new `TryLazy`
		/// unchanged.
		#[document_signature]
		///
		#[document_type_parameters("The type of the mapped value.")]
		///
		#[document_parameters("The function to apply to the success value.")]
		///
		#[document_returns("A new `TryLazy` containing the mapped result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = RcTryLazy::new(|| Ok::<i32, ()>(21));
		/// let mapped = memo.map(|x| x * 2);
		/// assert_eq!(mapped.evaluate(), Ok(&42));
		/// ```
		pub fn map<B: 'a>(
			self,
			f: impl FnOnce(&A) -> B + 'a,
		) -> TryLazy<'a, B, E, RcLazyConfig> {
			let fa = self.clone();
			TryLazy::<'a, B, E, RcLazyConfig>::new(move || match fa.evaluate() {
				Ok(a) => Ok(f(a)),
				Err(e) => Err(e.clone()),
			})
		}

		/// Transforms the error value, producing a new memoized `TryLazy`.
		///
		/// The mapping closure receives a `&E` reference to the cached error. If the
		/// computation produced an `Ok`, the success value is cloned into the new
		/// `TryLazy` unchanged.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new error.")]
		///
		#[document_parameters("The function to apply to the error value.")]
		///
		#[document_returns("A new `TryLazy` with the transformed error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo: RcTryLazy<i32, i32> = RcTryLazy::new(|| Err(21));
		/// let mapped = memo.map_err(|e| e * 2);
		/// assert_eq!(mapped.evaluate(), Err(&42));
		/// ```
		pub fn map_err<E2: 'a>(
			self,
			f: impl FnOnce(&E) -> E2 + 'a,
		) -> TryLazy<'a, A, E2, RcLazyConfig> {
			let fa = self.clone();
			TryLazy::<'a, A, E2, RcLazyConfig>::new(move || match fa.evaluate() {
				Ok(a) => Ok(a.clone()),
				Err(e) => Err(f(e)),
			})
		}

		/// Chains a fallible operation on the success value.
		///
		/// If this `TryLazy` succeeds, applies `f` to the cached `&A` and returns a new
		/// `TryLazy` that evaluates the result of `f`. If this `TryLazy` fails, the
		/// error is cloned into the new `TryLazy`.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new success value.")]
		///
		#[document_parameters("The fallible function to apply to the success value.")]
		///
		#[document_returns("A new `TryLazy` containing the chained result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = RcTryLazy::new(|| Ok::<i32, String>(21));
		/// let chained = memo.and_then(|x| if *x > 0 { Ok(x * 2) } else { Err("negative".into()) });
		/// assert_eq!(chained.evaluate(), Ok(&42));
		/// ```
		pub fn and_then<B: 'a>(
			self,
			f: impl FnOnce(&A) -> Result<B, E> + 'a,
		) -> TryLazy<'a, B, E, RcLazyConfig> {
			let fa = self.clone();
			TryLazy::<'a, B, E, RcLazyConfig>::new(move || match fa.evaluate() {
				Ok(a) => f(a),
				Err(e) => Err(e.clone()),
			})
		}

		/// Provides a fallback on the error value.
		///
		/// If this `TryLazy` fails, applies `f` to the cached `&E` and returns a new
		/// `TryLazy` that evaluates the result of `f`. If this `TryLazy` succeeds, the
		/// success value is cloned into the new `TryLazy`.
		#[document_signature]
		///
		#[document_parameters("The recovery function to apply to the error value.")]
		///
		#[document_returns("A new `TryLazy` containing the recovered result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo: RcTryLazy<i32, String> = RcTryLazy::new(|| Err("oops".into()));
		/// let recovered = memo.or_else(|_| Ok(42));
		/// assert_eq!(recovered.evaluate(), Ok(&42));
		/// ```
		pub fn or_else(
			self,
			f: impl FnOnce(&E) -> Result<A, E> + 'a,
		) -> TryLazy<'a, A, E, RcLazyConfig> {
			let fa = self.clone();
			TryLazy::<'a, A, E, RcLazyConfig>::new(move || match fa.evaluate() {
				Ok(a) => Ok(a.clone()),
				Err(e) => f(e),
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	#[document_parameters("The `TryLazy` instance.")]
	impl<'a, A, E> TryLazy<'a, A, E, ArcLazyConfig>
	where
		A: Clone + Send + Sync + 'a,
		E: Clone + Send + Sync + 'a,
	{
		/// Transforms the success value, producing a new thread-safe memoized `TryLazy`.
		///
		/// The mapping closure receives a `&A` reference to the cached value. If the
		/// computation produced an `Err`, the error is cloned into the new `TryLazy`
		/// unchanged.
		#[document_signature]
		///
		#[document_type_parameters("The type of the mapped value.")]
		///
		#[document_parameters("The function to apply to the success value.")]
		///
		#[document_returns("A new `ArcTryLazy` containing the mapped result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = ArcTryLazy::new(|| Ok::<i32, ()>(21));
		/// let mapped = memo.map(|x| x * 2);
		/// assert_eq!(mapped.evaluate(), Ok(&42));
		/// ```
		pub fn map<B: Send + Sync + 'a>(
			self,
			f: impl FnOnce(&A) -> B + Send + 'a,
		) -> TryLazy<'a, B, E, ArcLazyConfig> {
			let fa = self.clone();
			TryLazy::<'a, B, E, ArcLazyConfig>::new(move || match fa.evaluate() {
				Ok(a) => Ok(f(a)),
				Err(e) => Err(e.clone()),
			})
		}

		/// Transforms the error value, producing a new thread-safe memoized `TryLazy`.
		///
		/// The mapping closure receives a `&E` reference to the cached error. If the
		/// computation produced an `Ok`, the success value is cloned into the new
		/// `TryLazy` unchanged.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new error.")]
		///
		#[document_parameters("The function to apply to the error value.")]
		///
		#[document_returns("A new `ArcTryLazy` with the transformed error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo: ArcTryLazy<i32, i32> = ArcTryLazy::new(|| Err(21));
		/// let mapped = memo.map_err(|e| e * 2);
		/// assert_eq!(mapped.evaluate(), Err(&42));
		/// ```
		pub fn map_err<E2: Send + Sync + 'a>(
			self,
			f: impl FnOnce(&E) -> E2 + Send + 'a,
		) -> TryLazy<'a, A, E2, ArcLazyConfig> {
			let fa = self.clone();
			TryLazy::<'a, A, E2, ArcLazyConfig>::new(move || match fa.evaluate() {
				Ok(a) => Ok(a.clone()),
				Err(e) => Err(f(e)),
			})
		}

		/// Chains a fallible operation on the success value (thread-safe variant).
		///
		/// If this `ArcTryLazy` succeeds, applies `f` to the cached `&A` and returns a
		/// new `ArcTryLazy` that evaluates the result of `f`. If this `ArcTryLazy`
		/// fails, the error is cloned into the new `ArcTryLazy`.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new success value.")]
		///
		#[document_parameters("The fallible function to apply to the success value.")]
		///
		#[document_returns("A new `ArcTryLazy` containing the chained result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = ArcTryLazy::new(|| Ok::<i32, String>(21));
		/// let chained = memo.and_then(|x| if *x > 0 { Ok(x * 2) } else { Err("negative".into()) });
		/// assert_eq!(chained.evaluate(), Ok(&42));
		/// ```
		pub fn and_then<B: Send + Sync + 'a>(
			self,
			f: impl FnOnce(&A) -> Result<B, E> + Send + 'a,
		) -> TryLazy<'a, B, E, ArcLazyConfig> {
			let fa = self.clone();
			TryLazy::<'a, B, E, ArcLazyConfig>::new(move || match fa.evaluate() {
				Ok(a) => f(a),
				Err(e) => Err(e.clone()),
			})
		}

		/// Provides a fallback on the error value (thread-safe variant).
		///
		/// If this `ArcTryLazy` fails, applies `f` to the cached `&E` and returns a
		/// new `ArcTryLazy` that evaluates the result of `f`. If this `ArcTryLazy`
		/// succeeds, the success value is cloned into the new `ArcTryLazy`.
		#[document_signature]
		///
		#[document_parameters("The recovery function to apply to the error value.")]
		///
		#[document_returns("A new `ArcTryLazy` containing the recovered result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo: ArcTryLazy<i32, String> = ArcTryLazy::new(|| Err("oops".into()));
		/// let recovered = memo.or_else(|_| Ok(42));
		/// assert_eq!(recovered.evaluate(), Ok(&42));
		/// ```
		pub fn or_else(
			self,
			f: impl FnOnce(&E) -> Result<A, E> + Send + 'a,
		) -> TryLazy<'a, A, E, ArcLazyConfig> {
			let fa = self.clone();
			TryLazy::<'a, A, E, ArcLazyConfig>::new(move || match fa.evaluate() {
				Ok(a) => Ok(a.clone()),
				Err(e) => f(e),
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> TryLazy<'a, A, E, ArcLazyConfig>
	where
		A: 'a,
		E: 'a,
	{
		/// Creates a new `TryLazy` that will run `f` on first access.
		#[document_signature]
		///
		#[document_parameters("The closure that produces the result.")]
		///
		#[document_returns("A new `TryLazy` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let memo = TryLazy::<_, _, ArcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// assert_eq!(memo.evaluate(), Ok(&42));
		/// ```
		pub fn new(f: impl FnOnce() -> Result<A, E> + Send + 'a) -> Self {
			TryLazy(ArcLazyConfig::try_lazy_new(Box::new(f)))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> Deferrable<'a> for TryLazy<'a, A, E, RcLazyConfig>
	where
		A: Clone + 'a,
		E: Clone + 'a,
	{
		/// Defers a computation that produces a `TryLazy` value.
		///
		/// This flattens the nested structure: instead of `TryLazy<TryLazy<A, E>, E>`, we get `TryLazy<A, E>`.
		/// The inner `TryLazy` is computed only when the outer `TryLazy` is evaluated.
		#[document_signature]
		///
		#[document_parameters("The thunk that produces the lazy value.")]
		///
		#[document_returns("A new `TryLazy` value.")]
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
		/// let lazy = TryLazy::<_, (), RcLazyConfig>::defer(|| RcTryLazy::new(|| Ok(42)));
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized, {
			Self::new(move || f().evaluate().cloned().map_err(Clone::clone))
		}
	}

	impl_kind! {
		impl<E: 'static, Config: LazyConfig> for TryLazyBrand<E, Config> {
			#[document_default]
			type Of<'a, A: 'a>: 'a = TryLazy<'a, A, E, Config>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The success value type.",
		"The type of the error."
	)]
	impl<'a, A: Semigroup + Clone + 'a, E: Clone + 'a> Semigroup for TryLazy<'a, A, E, RcLazyConfig> {
		/// Combines two `RcTryLazy` values by combining their success values.
		///
		/// Evaluates `a` first. If `a` is `Err`, returns the error immediately without
		/// evaluating `b`. If both succeed, combines the values using `Semigroup::append`.
		#[document_signature]
		///
		#[document_parameters("The first `TryLazy`.", "The second `TryLazy`.")]
		///
		#[document_returns("A new `RcTryLazy` containing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let a: RcTryLazy<String, ()> = RcTryLazy::new(|| Ok("Hello".to_string()));
		/// let b: RcTryLazy<String, ()> = RcTryLazy::new(|| Ok(" World".to_string()));
		/// let c = append::<_>(a, b);
		/// assert_eq!(c.evaluate(), Ok(&"Hello World".to_string()));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			RcTryLazy::new(move || {
				let a_val = match a.evaluate() {
					Ok(v) => v.clone(),
					Err(e) => return Err(e.clone()),
				};
				let b_val = match b.evaluate() {
					Ok(v) => v.clone(),
					Err(e) => return Err(e.clone()),
				};
				Ok(Semigroup::append(a_val, b_val))
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The success value type.",
		"The type of the error."
	)]
	impl<'a, A: Semigroup + Clone + Send + Sync + 'a, E: Clone + Send + Sync + 'a> Semigroup
		for TryLazy<'a, A, E, ArcLazyConfig>
	{
		/// Combines two `ArcTryLazy` values by combining their success values.
		///
		/// Evaluates `a` first. If `a` is `Err`, returns the error immediately without
		/// evaluating `b`. If both succeed, combines the values using `Semigroup::append`.
		#[document_signature]
		///
		#[document_parameters("The first `ArcTryLazy`.", "The second `ArcTryLazy`.")]
		///
		#[document_returns("A new `ArcTryLazy` containing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let a: ArcTryLazy<String, ()> = ArcTryLazy::new(|| Ok("Hello".to_string()));
		/// let b: ArcTryLazy<String, ()> = ArcTryLazy::new(|| Ok(" World".to_string()));
		/// let c = append::<_>(a, b);
		/// assert_eq!(c.evaluate(), Ok(&"Hello World".to_string()));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			ArcTryLazy::new(move || {
				let a_val = match a.evaluate() {
					Ok(v) => v.clone(),
					Err(e) => return Err(e.clone()),
				};
				let b_val = match b.evaluate() {
					Ok(v) => v.clone(),
					Err(e) => return Err(e.clone()),
				};
				Ok(Semigroup::append(a_val, b_val))
			})
		}
	}

	#[document_type_parameters("The error type.", "The memoization configuration.")]
	impl<E: 'static, Config: LazyConfig> Foldable for TryLazyBrand<E, Config> {
		/// Folds the `TryLazy` from the right.
		///
		/// If the computation succeeded, applies `func` to the success value and the
		/// initial accumulator. If it failed, returns the initial accumulator unchanged
		/// (the error is discarded).
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
			"The `TryLazy` to fold."
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
		/// let memo: RcTryLazy<i32, ()> = RcTryLazy::new(|| Ok(10));
		/// let result =
		/// 	fold_right::<RcFnBrand, TryLazyBrand<(), RcLazyConfig>, _, _>(|a, b| a + b, 5, memo);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			match fa.evaluate() {
				Ok(a) => func(a.clone(), initial),
				Err(_) => initial,
			}
		}

		/// Folds the `TryLazy` from the left.
		///
		/// If the computation succeeded, applies `func` to the initial accumulator and
		/// the success value. If it failed, returns the initial accumulator unchanged
		/// (the error is discarded).
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
			"The `TryLazy` to fold."
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
		/// let memo: RcTryLazy<i32, ()> = RcTryLazy::new(|| Ok(10));
		/// let result =
		/// 	fold_left::<RcFnBrand, TryLazyBrand<(), RcLazyConfig>, _, _>(|b, a| b + a, 5, memo);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			match fa.evaluate() {
				Ok(a) => func(initial, a.clone()),
				Err(_) => initial,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error."
	)]
	impl<'a, A, E> SendDeferrable<'a> for TryLazy<'a, A, E, ArcLazyConfig>
	where
		A: Clone + Send + Sync + 'a,
		E: Clone + Send + Sync + 'a,
	{
		/// Defers a computation that produces a thread-safe `TryLazy` value.
		///
		/// This flattens the nested structure: instead of `ArcTryLazy<ArcTryLazy<A, E>, E>`, we get `ArcTryLazy<A, E>`.
		/// The inner `TryLazy` is computed only when the outer `TryLazy` is evaluated.
		#[document_signature]
		///
		#[document_parameters("The thunk that produces the lazy value.")]
		///
		#[document_returns("A new `ArcTryLazy` value.")]
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
		/// let lazy: ArcTryLazy<i32, ()> = ArcTryLazy::send_defer(|| ArcTryLazy::new(|| Ok(42)));
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		fn send_defer(f: impl FnOnce() -> Self + Send + Sync + 'a) -> Self
		where
			Self: Sized, {
			Self::new(move || f().evaluate().cloned().map_err(Clone::clone))
		}
	}

	/// Single-threaded fallible memoization alias.
	pub type RcTryLazy<'a, A, E> = TryLazy<'a, A, E, RcLazyConfig>;

	/// Thread-safe fallible memoization alias.
	pub type ArcTryLazy<'a, A, E> = TryLazy<'a, A, E, ArcLazyConfig>;
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::{
			RcLazy,
			TryThunk,
			TryTrampoline,
		},
		std::{
			cell::RefCell,
			rc::Rc,
		},
	};

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

		let memo: ArcTryLazy<i32, ()> = send_defer(|| ArcTryLazy::new(|| Ok(42)));
		assert_eq!(memo.evaluate(), Ok(&42));
	}

	/// Tests `Semigroup::append` where the first operand is `Err`.
	///
	/// Verifies that the second operand is not evaluated (short-circuit behavior).
	#[test]
	fn test_semigroup_append_first_err_short_circuits() {
		use {
			crate::classes::Semigroup,
			std::cell::Cell,
		};

		let counter = Rc::new(Cell::new(0u32));
		let counter_clone = counter.clone();

		let a: RcTryLazy<String, String> = RcTryLazy::new(|| Err("first failed".into()));
		let b: RcTryLazy<String, String> = RcTryLazy::new(move || {
			counter_clone.set(counter_clone.get() + 1);
			Ok("second".into())
		});

		let result = Semigroup::append(a, b);
		assert_eq!(result.evaluate(), Err(&"first failed".to_string()));
		assert_eq!(counter.get(), 0, "second operand should not be evaluated");
	}

	/// Tests `Semigroup::append` where the second operand fails but the first succeeds.
	///
	/// Verifies that the error from the second operand is propagated.
	#[test]
	fn test_semigroup_append_second_err() {
		use crate::classes::Semigroup;

		let a: RcTryLazy<String, String> = RcTryLazy::new(|| Ok("hello".into()));
		let b: RcTryLazy<String, String> = RcTryLazy::new(|| Err("second failed".into()));

		let result = Semigroup::append(a, b);
		assert_eq!(result.evaluate(), Err(&"second failed".to_string()));
	}

	/// Tests `Semigroup::append` where both operands succeed.
	#[test]
	fn test_semigroup_append_both_ok() {
		use crate::classes::Semigroup;

		let a: RcTryLazy<String, ()> = RcTryLazy::new(|| Ok("Hello".into()));
		let b: RcTryLazy<String, ()> = RcTryLazy::new(|| Ok(" World".into()));

		let result = Semigroup::append(a, b);
		assert_eq!(result.evaluate(), Ok(&"Hello World".to_string()));
	}

	/// Tests `map` on a successful `TryLazy`.
	#[test]
	fn test_map_ok() {
		let memo = RcTryLazy::new(|| Ok::<i32, ()>(21));
		let mapped = memo.map(|x| x * 2);
		assert_eq!(mapped.evaluate(), Ok(&42));
	}

	/// Tests `map` on a failed `TryLazy`.
	#[test]
	fn test_map_err() {
		let memo: RcTryLazy<i32, String> = RcTryLazy::new(|| Err("oops".into()));
		let mapped = memo.map(|x| x * 2);
		assert_eq!(mapped.evaluate(), Err(&"oops".to_string()));
	}

	/// Tests `map_err` on a failed `TryLazy`.
	#[test]
	fn test_map_err_on_err() {
		let memo: RcTryLazy<i32, i32> = RcTryLazy::new(|| Err(21));
		let mapped = memo.map_err(|e| e * 2);
		assert_eq!(mapped.evaluate(), Err(&42));
	}

	/// Tests `map_err` on a successful `TryLazy`.
	#[test]
	fn test_map_err_on_ok() {
		let memo: RcTryLazy<i32, i32> = RcTryLazy::new(|| Ok(42));
		let mapped = memo.map_err(|e| e * 2);
		assert_eq!(mapped.evaluate(), Ok(&42));
	}

	/// Tests `and_then` on a successful `TryLazy`.
	#[test]
	fn test_and_then_ok() {
		let memo = RcTryLazy::new(|| Ok::<i32, String>(21));
		let chained = memo.and_then(|x| Ok(x * 2));
		assert_eq!(chained.evaluate(), Ok(&42));
	}

	/// Tests `and_then` where the chained operation fails.
	#[test]
	fn test_and_then_chained_err() {
		let memo = RcTryLazy::new(|| Ok::<i32, String>(21));
		let chained = memo.and_then(|_| Err("chained failure".into()));
		assert_eq!(chained.evaluate(), Err(&"chained failure".to_string()));
	}

	/// Tests `and_then` on a failed `TryLazy`.
	#[test]
	fn test_and_then_initial_err() {
		let memo: RcTryLazy<i32, String> = RcTryLazy::new(|| Err("initial".into()));
		let chained = memo.and_then(|x| Ok(x * 2));
		assert_eq!(chained.evaluate(), Err(&"initial".to_string()));
	}

	/// Tests `or_else` on a failed `TryLazy`.
	#[test]
	fn test_or_else_recovers() {
		let memo: RcTryLazy<i32, String> = RcTryLazy::new(|| Err("oops".into()));
		let recovered = memo.or_else(|_| Ok(42));
		assert_eq!(recovered.evaluate(), Ok(&42));
	}

	/// Tests `or_else` on a successful `TryLazy`.
	#[test]
	fn test_or_else_noop_on_ok() {
		let memo: RcTryLazy<i32, String> = RcTryLazy::new(|| Ok(42));
		let recovered = memo.or_else(|_| Ok(99));
		assert_eq!(recovered.evaluate(), Ok(&42));
	}

	/// Tests `or_else` where recovery itself fails.
	#[test]
	fn test_or_else_recovery_fails() {
		let memo: RcTryLazy<i32, String> = RcTryLazy::new(|| Err("first".into()));
		let recovered = memo.or_else(|_| Err("second".into()));
		assert_eq!(recovered.evaluate(), Err(&"second".to_string()));
	}

	/// Tests `Foldable` fold_right on a successful `TryLazy`.
	#[test]
	fn test_foldable_ok() {
		use crate::{
			brands::*,
			functions::*,
		};

		let memo: RcTryLazy<i32, ()> = RcTryLazy::new(|| Ok(10));
		let result =
			fold_right::<RcFnBrand, TryLazyBrand<(), RcLazyConfig>, _, _>(|a, b| a + b, 5, memo);
		assert_eq!(result, 15);
	}

	/// Tests `Foldable` fold_right on a failed `TryLazy`.
	#[test]
	fn test_foldable_err() {
		use crate::{
			brands::*,
			functions::*,
		};

		let memo: RcTryLazy<i32, String> = RcTryLazy::new(|| Err("oops".into()));
		let result = fold_right::<RcFnBrand, TryLazyBrand<String, RcLazyConfig>, _, _>(
			|a, b| a + b,
			5,
			memo,
		);
		assert_eq!(result, 5);
	}
}
