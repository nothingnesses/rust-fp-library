//! Deferred, non-memoized fallible computation with higher-kinded type support.
//!
//! The fallible counterpart to [`Thunk`](crate::types::Thunk). Each call to [`TryThunk::evaluate`] re-executes the computation and returns a [`Result`]. Supports borrowing and lifetime polymorphism.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				TryThunkBrand,
				TryThunkErrAppliedBrand,
				TryThunkOkAppliedBrand,
			},
			classes::{
				ApplyFirst,
				ApplySecond,
				Bifoldable,
				Bifunctor,
				CloneableFn,
				Deferrable,
				Foldable,
				Functor,
				Lift,
				MonadRec,
				Monoid,
				Pointed,
				Semiapplicative,
				Semigroup,
				Semimonad,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcLazyConfig,
				ArcTryLazy,
				Lazy,
				LazyConfig,
				RcLazyConfig,
				RcTryLazy,
				Step,
				Thunk,
				TryLazy,
				TryTrampoline,
			},
		},
		fp_macros::*,
		std::fmt,
	};

	/// A deferred computation that may fail with error type `E`.
	///
	/// This is [`Thunk<'a, Result<A, E>>`] with ergonomic combinators for error handling.
	/// Like [`Thunk`], this is NOT memoized. Each [`TryThunk::evaluate`] re-executes.
	/// Unlike [`Thunk`], the result is [`Result<A, E>`].
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation on success.",
		"The type of the error produced by the computation on failure."
	)]
	///
	/// ### Higher-Kinded Type Representation
	///
	/// This type has multiple higher-kinded representations:
	/// - [`TryThunkBrand`](crate::brands::TryThunkBrand): fully polymorphic over both error and success types (bifunctor).
	/// - [`TryThunkErrAppliedBrand<E>`](crate::brands::TryThunkErrAppliedBrand): the error type is fixed, polymorphic over the success type (functor over `Ok`).
	/// - [`TryThunkOkAppliedBrand<A>`](crate::brands::TryThunkOkAppliedBrand): the success type is fixed, polymorphic over the error type (functor over `Err`).
	pub struct TryThunk<'a, A, E>(
		/// The internal `Thunk` wrapping a `Result`.
		Thunk<'a, Result<A, E>>,
	);

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	#[document_parameters("The `TryThunk` instance.")]
	impl<'a, A: 'a, E: 'a> TryThunk<'a, A, E> {
		/// Creates a new `TryThunk` from a thunk.
		#[document_signature]
		///
		#[document_parameters("The thunk to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::new(|| Ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn new(f: impl FnOnce() -> Result<A, E> + 'a) -> Self {
			TryThunk(Thunk::new(f))
		}

		/// Returns a pure value (already computed).
		#[deprecated(
			since = "0.14.0",
			note = "Use `ok` instead for consistency with TryTrampoline and TryLazy"
		)]
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// #[allow(deprecated)]
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self
		where
			A: 'a, {
			TryThunk(Thunk::pure(Ok(a)))
		}

		/// Defers a computation that returns a TryThunk.
		#[document_signature]
		///
		#[document_parameters("The thunk that returns a `TryThunk`.")]
		///
		#[document_returns("A new `TryThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::defer(|| TryThunk::ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn defer(f: impl FnOnce() -> TryThunk<'a, A, E> + 'a) -> Self {
			TryThunk(Thunk::defer(move || f().0))
		}

		/// Creates a successful computation.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn ok(a: A) -> Self
		where
			A: 'a, {
			TryThunk(Thunk::pure(Ok(a)))
		}

		/// Returns a pure error.
		#[document_signature]
		///
		#[document_parameters("The error to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, &str> = TryThunk::err("error");
		/// assert_eq!(try_thunk.evaluate(), Err("error"));
		/// ```
		#[inline]
		pub fn err(e: E) -> Self
		where
			E: 'a, {
			TryThunk(Thunk::pure(Err(e)))
		}

		/// Monadic bind: chains computations.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `TryThunk` instance representing the chained computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(21).bind(|x| TryThunk::ok(x * 2));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn bind<B: 'a>(
			self,
			f: impl FnOnce(A) -> TryThunk<'a, B, E> + 'a,
		) -> TryThunk<'a, B, E> {
			TryThunk(self.0.bind(|result| match result {
				Ok(a) => f(a).0,
				Err(e) => Thunk::pure(Err(e)),
			}))
		}

		/// Functor map: transforms the result.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the transformation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `TryThunk` instance with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(21).map(|x| x * 2);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn map<B: 'a>(
			self,
			func: impl FnOnce(A) -> B + 'a,
		) -> TryThunk<'a, B, E> {
			TryThunk(self.0.map(|result| result.map(func)))
		}

		/// Map error: transforms the error.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new error.")]
		///
		#[document_parameters("The function to apply to the error.")]
		///
		#[document_returns("A new `TryThunk` instance with the transformed error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, i32> = TryThunk::err(21).map_err(|x| x * 2);
		/// assert_eq!(try_thunk.evaluate(), Err(42));
		/// ```
		#[inline]
		pub fn map_err<E2: 'a>(
			self,
			f: impl FnOnce(E) -> E2 + 'a,
		) -> TryThunk<'a, A, E2> {
			TryThunk(self.0.map(|result| result.map_err(f)))
		}

		/// Recovers from an error.
		#[document_signature]
		///
		#[document_parameters("The function to apply to the error value.")]
		///
		#[document_returns("A new `TryThunk` that attempts to recover from failure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, &str> = TryThunk::err("error").catch(|_| TryThunk::ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn catch(
			self,
			f: impl FnOnce(E) -> TryThunk<'a, A, E> + 'a,
		) -> Self {
			TryThunk(self.0.bind(|result| match result {
				Ok(a) => Thunk::pure(Ok(a)),
				Err(e) => f(e).0,
			}))
		}

		/// Forces evaluation and returns the result.
		#[document_signature]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn evaluate(self) -> Result<A, E> {
			self.0.evaluate()
		}

		/// Combines two `TryThunk`s, running both and combining their results.
		///
		/// Short-circuits on error: if `self` fails, `other` is never evaluated.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the second computation's success value.",
			"The type of the combined result."
		)]
		///
		#[document_parameters("The second computation.", "The function to combine the results.")]
		///
		#[document_returns("A new `TryThunk` producing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1: TryThunk<i32, String> = TryThunk::ok(10);
		/// let t2: TryThunk<i32, String> = TryThunk::ok(20);
		/// let t3 = t1.lift2(t2, |a, b| a + b);
		/// assert_eq!(t3.evaluate(), Ok(30));
		///
		/// let t4: TryThunk<i32, String> = TryThunk::err("fail".to_string());
		/// let t5: TryThunk<i32, String> = TryThunk::ok(20);
		/// let t6 = t4.lift2(t5, |a, b| a + b);
		/// assert_eq!(t6.evaluate(), Err("fail".to_string()));
		/// ```
		pub fn lift2<B: 'a, C: 'a>(
			self,
			other: TryThunk<'a, B, E>,
			f: impl FnOnce(A, B) -> C + 'a,
		) -> TryThunk<'a, C, E> {
			self.bind(move |a| other.map(move |b| f(a, b)))
		}

		/// Sequences two `TryThunk`s, discarding the first result.
		///
		/// Short-circuits on error: if `self` fails, `other` is never evaluated.
		#[document_signature]
		///
		#[document_type_parameters("The type of the second computation's success value.")]
		///
		#[document_parameters("The second computation.")]
		///
		#[document_returns(
			"A new `TryThunk` that runs both computations and returns the result of the second."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1: TryThunk<i32, String> = TryThunk::ok(10);
		/// let t2: TryThunk<i32, String> = TryThunk::ok(20);
		/// let t3 = t1.then(t2);
		/// assert_eq!(t3.evaluate(), Ok(20));
		///
		/// let t4: TryThunk<i32, String> = TryThunk::err("fail".to_string());
		/// let t5: TryThunk<i32, String> = TryThunk::ok(20);
		/// let t6 = t4.then(t5);
		/// assert_eq!(t6.evaluate(), Err("fail".to_string()));
		/// ```
		pub fn then<B: 'a>(
			self,
			other: TryThunk<'a, B, E>,
		) -> TryThunk<'a, B, E> {
			self.bind(move |_| other)
		}

		/// Converts this `TryThunk` into a memoized [`RcTryLazy`].
		///
		/// The resulting `RcTryLazy` will evaluate the computation on first
		/// access and cache the result for subsequent accesses.
		#[document_signature]
		///
		#[document_returns("A memoized `RcTryLazy` wrapping this computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// let lazy: RcTryLazy<i32, ()> = thunk.memoize();
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		pub fn memoize(self) -> RcTryLazy<'a, A, E> {
			RcTryLazy::from(self)
		}

		/// Evaluates this `TryThunk` and wraps the result in a thread-safe [`ArcTryLazy`].
		///
		/// The thunk is evaluated eagerly because its inner closure is not
		/// `Send`. The result is stored in an `ArcTryLazy` for thread-safe sharing.
		#[document_signature]
		///
		#[document_returns("A thread-safe memoized `ArcTryLazy` wrapping this computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// let lazy: ArcTryLazy<i32, ()> = thunk.memoize_arc();
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		pub fn memoize_arc(self) -> ArcTryLazy<'a, A, E>
		where
			A: Send + Sync + 'a,
			E: Send + Sync + 'a, {
			let result = self.evaluate();
			ArcTryLazy::new(move || result)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: 'a> TryThunk<'a, A, String> {
		/// Creates a `TryThunk` that catches unwinds (panics).
		///
		/// The closure is executed when the thunk is evaluated. If the closure
		/// panics, the panic payload is converted to a `String` error. If the
		/// closure returns normally, the value is wrapped in `Ok`.
		#[document_signature]
		///
		#[document_parameters("The closure that might panic.")]
		///
		#[document_returns(
			"A new `TryThunk` instance where panics are converted to `Err(String)`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = TryThunk::<i32, String>::catch_unwind(|| {
		/// 	if true {
		/// 		panic!("oops")
		/// 	}
		/// 	42
		/// });
		/// assert_eq!(thunk.evaluate(), Err("oops".to_string()));
		/// ```
		pub fn catch_unwind(f: impl FnOnce() -> A + std::panic::UnwindSafe + 'a) -> Self {
			TryThunk::new(move || {
				std::panic::catch_unwind(f).map_err(crate::utils::panic_payload_to_string)
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value.",
		"The memoization configuration."
	)]
	impl<'a, A, E, Config> From<Lazy<'a, A, Config>> for TryThunk<'a, A, E>
	where
		A: Clone + 'a,
		E: 'a,
		Config: LazyConfig,
	{
		#[document_signature]
		#[document_parameters("The lazy value to convert.")]
		#[document_returns("A new `TryThunk` instance that wraps the lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
		/// let thunk: TryThunk<i32, ()> = TryThunk::from(lazy);
		/// assert_eq!(thunk.evaluate(), Ok(42));
		/// ```
		fn from(memo: Lazy<'a, A, Config>) -> Self {
			TryThunk::new(move || Ok(memo.evaluate().clone()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value.",
		"The memoization configuration."
	)]
	impl<'a, A, E, Config> From<TryLazy<'a, A, E, Config>> for TryThunk<'a, A, E>
	where
		A: Clone + 'a,
		E: Clone + 'a,
		Config: LazyConfig,
	{
		/// Converts a [`TryLazy`] value into a [`TryThunk`] by cloning the memoized result.
		///
		/// This conversion clones both the success and error values on each evaluation.
		/// The cost depends on the [`Clone`] implementations of `A` and `E`.
		#[document_signature]
		#[document_parameters("The fallible lazy value to convert.")]
		#[document_returns("A new `TryThunk` instance that wraps the fallible lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// let thunk = TryThunk::from(lazy);
		/// assert_eq!(thunk.evaluate(), Ok(42));
		/// ```
		fn from(memo: TryLazy<'a, A, E, Config>) -> Self {
			TryThunk::new(move || memo.evaluate().cloned().map_err(Clone::clone))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A: 'a, E: 'a> From<Thunk<'a, A>> for TryThunk<'a, A, E> {
		#[document_signature]
		#[document_parameters("The thunk to convert.")]
		#[document_returns("A new `TryThunk` instance that wraps the thunk.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = Thunk::new(|| 42);
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::from(thunk);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		fn from(eval: Thunk<'a, A>) -> Self {
			TryThunk(eval.map(Ok))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A: 'a, E: 'a> From<Result<A, E>> for TryThunk<'a, A, E> {
		#[document_signature]
		#[document_parameters("The result to convert.")]
		#[document_returns("A new `TryThunk` instance that produces the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let ok_thunk: TryThunk<i32, String> = TryThunk::from(Ok(42));
		/// assert_eq!(ok_thunk.evaluate(), Ok(42));
		///
		/// let err_thunk: TryThunk<i32, String> = TryThunk::from(Err("error".to_string()));
		/// assert_eq!(err_thunk.evaluate(), Err("error".to_string()));
		/// ```
		fn from(result: Result<A, E>) -> Self {
			TryThunk(Thunk::pure(result))
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	impl<A: 'static, E: 'static> From<TryTrampoline<A, E>> for TryThunk<'static, A, E> {
		/// Converts a [`TryTrampoline`] into a `TryThunk`.
		///
		/// The resulting `TryThunk` will evaluate the trampoline when forced.
		#[document_signature]
		#[document_parameters("The fallible trampoline to convert.")]
		#[document_returns("A new `TryThunk` instance that evaluates the trampoline.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let tramp: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		/// let thunk: TryThunk<i32, String> = TryThunk::from(tramp);
		/// assert_eq!(thunk.evaluate(), Ok(42));
		/// ```
		fn from(tramp: TryTrampoline<A, E>) -> Self {
			TryThunk::new(move || tramp.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A, E> Deferrable<'a> for TryThunk<'a, A, E>
	where
		A: 'a,
		E: 'a,
	{
		/// Creates a `TryThunk` from a computation that produces it.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the try thunk.")]
		///
		#[document_returns("The deferred try thunk.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::Deferrable,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let task: TryThunk<i32, ()> = Deferrable::defer(|| TryThunk::ok(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized, {
			TryThunk::defer(f)
		}
	}

	impl_kind! {
		impl<E: 'static> for TryThunkErrAppliedBrand<E> {
			#[document_default]
			type Of<'a, A: 'a>: 'a = TryThunk<'a, A, E>;
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Functor for TryThunkErrAppliedBrand<E> {
		/// Maps a function over the result of a `TryThunk` computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value inside the `TryThunk`.",
			"The type of the result of the transformation."
		)]
		///
		#[document_parameters(
			"The function to apply to the result of the computation.",
			"The `TryThunk` instance."
		)]
		///
		#[document_returns("A new `TryThunk` instance with the transformed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let mapped = map::<TryThunkErrAppliedBrand<()>, _, _>(|x| x * 2, try_thunk);
		/// assert_eq!(mapped.evaluate(), Ok(20));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(func)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Pointed for TryThunkErrAppliedBrand<E> {
		/// Wraps a value in a `TryThunk` context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value to wrap."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the value.")]
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
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			TryThunk::ok(a)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Lift for TryThunkErrAppliedBrand<E> {
		/// Lifts a binary function into the `TryThunk` context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first `TryThunk`.",
			"The second `TryThunk`."
		)]
		///
		#[document_returns(
			"A new `TryThunk` instance containing the result of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let eval1: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let eval2: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(20);
		/// let result = lift2::<TryThunkErrAppliedBrand<()>, _, _, _>(|a, b| a + b, eval1, eval2);
		/// assert_eq!(result.evaluate(), Ok(30));
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			fa.bind(move |a| fb.map(move |b| func(a, b)))
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> ApplyFirst for TryThunkErrAppliedBrand<E> {}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> ApplySecond for TryThunkErrAppliedBrand<E> {}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Semiapplicative for TryThunkErrAppliedBrand<E> {
		/// Applies a function wrapped in `TryThunk` to a value wrapped in `TryThunk`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function wrapper.",
			"The type of the input.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The `TryThunk` containing the function.",
			"The `TryThunk` containing the value."
		)]
		///
		#[document_returns(
			"A new `TryThunk` instance containing the result of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let func: TryThunk<_, ()> =
		/// 	pure::<TryThunkErrAppliedBrand<()>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let val: TryThunk<_, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(21);
		/// let result = apply::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _>(func, val);
		/// assert_eq!(result.evaluate(), Ok(42));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ff.bind(move |f| {
				fa.map(
					#[allow(clippy::redundant_closure)] // Required for move semantics
					move |a| f(a),
				)
			})
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Semimonad for TryThunkErrAppliedBrand<E> {
		/// Chains `TryThunk` computations.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the result of the first computation.",
			"The type of the result of the new computation."
		)]
		///
		#[document_parameters(
			"The first `TryThunk`.",
			"The function to apply to the result of the computation."
		)]
		///
		#[document_returns("A new `TryThunk` instance representing the chained computation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let result = bind::<TryThunkErrAppliedBrand<()>, _, _>(try_thunk, |x| {
		/// 	pure::<TryThunkErrAppliedBrand<()>, _>(x * 2)
		/// });
		/// assert_eq!(result.evaluate(), Ok(20));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind(func)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> MonadRec for TryThunkErrAppliedBrand<E> {
		/// Performs tail-recursive monadic computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("The result of the computation.")]
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
		/// let result = tail_rec_m::<TryThunkErrAppliedBrand<()>, _, _>(
		/// 	|x| {
		/// 		pure::<TryThunkErrAppliedBrand<()>, _>(
		/// 			if x < 1000 { Step::Loop(x + 1) } else { Step::Done(x) },
		/// 		)
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), Ok(1000));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>)
			+ Clone
			+ 'a,
			a: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			TryThunk::new(move || {
				let mut current = a;
				loop {
					match f(current).evaluate() {
						Ok(Step::Loop(next)) => current = next,
						Ok(Step::Done(res)) => break Ok(res),
						Err(e) => break Err(e),
					}
				}
			})
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Foldable for TryThunkErrAppliedBrand<E> {
		/// Folds the `TryThunk` from the right.
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
			"The `TryThunk` to fold."
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
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let result =
		/// 	fold_right::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _>(|a, b| a + b, 5, try_thunk);
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
				Ok(a) => func(a, initial),
				Err(_) => initial,
			}
		}

		/// Folds the `TryThunk` from the left.
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
			"The `TryThunk` to fold."
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
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let result =
		/// 	fold_left::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _>(|b, a| b + a, 5, try_thunk);
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
				Ok(a) => func(initial, a),
				Err(_) => initial,
			}
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
		#[document_parameters("The mapping function.", "The Thunk to fold.")]
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
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let result =
		/// 	fold_map::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _>(|a| a.to_string(), try_thunk);
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			match fa.evaluate() {
				Ok(a) => func(a),
				Err(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The success value type.",
		"The error value type."
	)]
	impl<'a, A: Semigroup + 'a, E: 'a> Semigroup for TryThunk<'a, A, E> {
		/// Combines two `TryThunk`s by combining their results.
		#[document_signature]
		///
		#[document_parameters("The first `TryThunk`.", "The second `TryThunk`.")]
		///
		#[document_returns("A new `TryThunk` containing the combined result.")]
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
		/// let t1: TryThunk<String, ()> = pure::<TryThunkErrAppliedBrand<()>, _>("Hello".to_string());
		/// let t2: TryThunk<String, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(" World".to_string());
		/// let t3 = append::<_>(t1, t2);
		/// assert_eq!(t3.evaluate(), Ok("Hello World".to_string()));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			TryThunk::new(move || match (a.evaluate(), b.evaluate()) {
				(Ok(a_val), Ok(b_val)) => Ok(Semigroup::append(a_val, b_val)),
				(Err(e), _) => Err(e),
				(_, Err(e)) => Err(e),
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The success value type.",
		"The error value type."
	)]
	impl<'a, A: Monoid + 'a, E: 'a> Monoid for TryThunk<'a, A, E> {
		/// Returns the identity `TryThunk`.
		#[document_signature]
		///
		#[document_returns("A `TryThunk` producing the identity value of `A`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let t: TryThunk<String, ()> = TryThunk::empty();
		/// assert_eq!(t.evaluate(), Ok("".to_string()));
		/// ```
		fn empty() -> Self {
			TryThunk(Thunk::pure(Ok(Monoid::empty())))
		}
	}

	impl_kind! {
		/// HKT branding for the `TryThunk` type.
		///
		/// The type parameters for `Of` are ordered `E`, then `A` (Error, then Success).
		/// This follows the same convention as `ResultBrand`, matching functional
		/// programming expectations (like Haskell's `Either e a`) where the success
		/// type is the last parameter.
		for TryThunkBrand {
			type Of<'a, E: 'a, A: 'a>: 'a = TryThunk<'a, A, E>;
		}
	}

	impl Bifunctor for TryThunkBrand {
		/// Maps functions over the values in the `TryThunk`.
		///
		/// This method applies one function to the error value and another to the success value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the error value.",
			"The type of the mapped error value.",
			"The type of the success value.",
			"The type of the mapped success value."
		)]
		///
		#[document_parameters(
			"The function to apply to the error.",
			"The function to apply to the success.",
			"The `TryThunk` to map over."
		)]
		///
		#[document_returns("A new `TryThunk` containing the mapped values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::bifunctor::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x: TryThunk<i32, i32> = TryThunk::ok(5);
		/// assert_eq!(bimap::<TryThunkBrand, _, _, _, _>(|e| e + 1, |s| s * 2, x).evaluate(), Ok(10));
		///
		/// let y: TryThunk<i32, i32> = TryThunk::err(5);
		/// assert_eq!(bimap::<TryThunkBrand, _, _, _, _>(|e| e + 1, |s| s * 2, y).evaluate(), Err(6));
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(A) -> B + 'a,
			g: impl Fn(C) -> D + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			TryThunk(p.0.map(move |result| match result {
				Ok(c) => Ok(g(c)),
				Err(a) => Err(f(a)),
			}))
		}
	}

	impl Bifoldable for TryThunkBrand {
		/// Folds a `TryThunk` using two step functions, right-associatively.
		///
		/// Dispatches to `f` for error values and `g` for success values.
		/// The thunk is evaluated to determine which branch to fold.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The error type (first position).",
			"The success type (second position).",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function applied to the error value.",
			"The step function applied to the success value.",
			"The initial accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("`f(e, z)` for `Err(e)`, or `g(a, z)` for `Ok(a)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, TryThunkBrand, _, _, _>(
		/// 		|e: i32, acc| acc - e,
		/// 		|s: i32, acc| acc + s,
		/// 		10,
		/// 		TryThunk::err(3),
		/// 	),
		/// 	7
		/// );
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, TryThunkBrand, _, _, _>(
		/// 		|e: i32, acc| acc - e,
		/// 		|s: i32, acc| acc + s,
		/// 		10,
		/// 		TryThunk::ok(5),
		/// 	),
		/// 	15
		/// );
		/// ```
		fn bi_fold_right<'a, FnBrand: CloneableFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(A, C) -> C + 'a,
			g: impl Fn(B, C) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			match p.evaluate() {
				Err(a) => f(a, z),
				Ok(b) => g(b, z),
			}
		}

		/// Folds a `TryThunk` using two step functions, left-associatively.
		///
		/// Dispatches to `f` for error values and `g` for success values.
		/// The thunk is evaluated to determine which branch to fold.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The error type (first position).",
			"The success type (second position).",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function applied to the error value.",
			"The step function applied to the success value.",
			"The initial accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("`f(z, e)` for `Err(e)`, or `g(z, a)` for `Ok(a)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_left::<RcFnBrand, TryThunkBrand, _, _, _>(
		/// 		|acc, e: i32| acc - e,
		/// 		|acc, s: i32| acc + s,
		/// 		10,
		/// 		TryThunk::err(3),
		/// 	),
		/// 	7
		/// );
		/// assert_eq!(
		/// 	bi_fold_left::<RcFnBrand, TryThunkBrand, _, _, _>(
		/// 		|acc, e: i32| acc - e,
		/// 		|acc, s: i32| acc + s,
		/// 		10,
		/// 		TryThunk::ok(5),
		/// 	),
		/// 	15
		/// );
		/// ```
		fn bi_fold_left<'a, FnBrand: CloneableFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(C, A) -> C + 'a,
			g: impl Fn(C, B) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			match p.evaluate() {
				Err(a) => f(z, a),
				Ok(b) => g(z, b),
			}
		}

		/// Maps a `TryThunk`'s value to a monoid using two functions and returns the result.
		///
		/// Dispatches to `f` for error values and `g` for success values,
		/// returning the monoid value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The error type (first position).",
			"The success type (second position).",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping the error to the monoid.",
			"The function mapping the success to the monoid.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("`f(e)` for `Err(e)`, or `g(a)` for `Ok(a)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_map::<RcFnBrand, TryThunkBrand, _, _, _>(
		/// 		|e: i32| e.to_string(),
		/// 		|s: i32| s.to_string(),
		/// 		TryThunk::err(3),
		/// 	),
		/// 	"3".to_string()
		/// );
		/// assert_eq!(
		/// 	bi_fold_map::<RcFnBrand, TryThunkBrand, _, _, _>(
		/// 		|e: i32| e.to_string(),
		/// 		|s: i32| s.to_string(),
		/// 		TryThunk::ok(5),
		/// 	),
		/// 	"5".to_string()
		/// );
		/// ```
		fn bi_fold_map<'a, FnBrand: CloneableFn + 'a, A: 'a + Clone, B: 'a + Clone, M>(
			f: impl Fn(A) -> M + 'a,
			g: impl Fn(B) -> M + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> M
		where
			M: Monoid + 'a, {
			match p.evaluate() {
				Err(a) => f(a),
				Ok(b) => g(b),
			}
		}
	}

	impl_kind! {
		impl<A: 'static> for TryThunkOkAppliedBrand<A> {
			#[document_default]
			type Of<'a, E: 'a>: 'a = TryThunk<'a, A, E>;
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Functor for TryThunkOkAppliedBrand<A> {
		/// Maps a function over the error value in the `TryThunk`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the error value inside the `TryThunk`.",
			"The type of the result of the transformation."
		)]
		///
		#[document_parameters("The function to apply to the error.", "The `TryThunk` instance.")]
		///
		#[document_returns("A new `TryThunk` instance with the transformed error.")]
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
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let mapped = map::<TryThunkOkAppliedBrand<i32>, _, _>(|x| x * 2, try_thunk);
		/// assert_eq!(mapped.evaluate(), Err(20));
		/// ```
		fn map<'a, E: 'a, E2: 'a>(
			func: impl Fn(E) -> E2 + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) {
			fa.map_err(func)
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Pointed for TryThunkOkAppliedBrand<A> {
		/// Wraps a value in a `TryThunk` context (as error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value to wrap."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the value as an error.")]
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
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(42);
		/// assert_eq!(try_thunk.evaluate(), Err(42));
		/// ```
		fn pure<'a, E: 'a>(e: E) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>) {
			TryThunk::err(e)
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Lift for TryThunkOkAppliedBrand<A> {
		/// Lifts a binary function into the `TryThunk` context (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the first error value.",
			"The type of the second error value.",
			"The type of the result error value."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the errors.",
			"The first `TryThunk`.",
			"The second `TryThunk`."
		)]
		///
		#[document_returns(
			"A new `TryThunk` instance containing the result of applying the function to the errors."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let eval1: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let eval2: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(20);
		/// let result = lift2::<TryThunkOkAppliedBrand<i32>, _, _, _>(|a, b| a + b, eval1, eval2);
		/// assert_eq!(result.evaluate(), Err(30));
		/// ```
		fn lift2<'a, E1, E2, E3>(
			func: impl Fn(E1, E2) -> E3 + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E1>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E3>)
		where
			E1: Clone + 'a,
			E2: Clone + 'a,
			E3: 'a, {
			TryThunk::new(move || match (fa.evaluate(), fb.evaluate()) {
				(Err(e1), Err(e2)) => Err(func(e1, e2)),
				(Ok(a), _) => Ok(a),
				(_, Ok(a)) => Ok(a),
			})
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> ApplyFirst for TryThunkOkAppliedBrand<A> {}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> ApplySecond for TryThunkOkAppliedBrand<A> {}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Semiapplicative for TryThunkOkAppliedBrand<A> {
		/// Applies a function wrapped in `TryThunk` (as error) to a value wrapped in `TryThunk` (as error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function wrapper.",
			"The type of the input error.",
			"The type of the result error."
		)]
		///
		#[document_parameters(
			"The `TryThunk` containing the function (in Err).",
			"The `TryThunk` containing the value (in Err)."
		)]
		///
		#[document_returns(
			"A new `TryThunk` instance containing the result of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let func: TryThunk<i32, _> =
		/// 	pure::<TryThunkOkAppliedBrand<i32>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let val: TryThunk<i32, _> = pure::<TryThunkOkAppliedBrand<i32>, _>(21);
		/// let result = apply::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _>(func, val);
		/// assert_eq!(result.evaluate(), Err(42));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneableFn, E1: 'a + Clone, E2: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, E1, E2>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E1>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) {
			TryThunk::new(move || match (ff.evaluate(), fa.evaluate()) {
				(Err(f), Err(e)) => Err(f(e)),
				(Ok(a), _) => Ok(a),
				(_, Ok(a)) => Ok(a),
			})
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Semimonad for TryThunkOkAppliedBrand<A> {
		/// Chains `TryThunk` computations (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the result of the first computation (error).",
			"The type of the result of the new computation (error)."
		)]
		///
		#[document_parameters(
			"The first `TryThunk`.",
			"The function to apply to the error result of the computation."
		)]
		///
		#[document_returns("A new `TryThunk` instance representing the chained computation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let result = bind::<TryThunkOkAppliedBrand<i32>, _, _>(try_thunk, |x| {
		/// 	pure::<TryThunkOkAppliedBrand<i32>, _>(x * 2)
		/// });
		/// assert_eq!(result.evaluate(), Err(20));
		/// ```
		fn bind<'a, E1: 'a, E2: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E1>),
			func: impl Fn(E1) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) {
			TryThunk::new(move || match ma.evaluate() {
				Ok(a) => Ok(a),
				Err(e) => func(e).evaluate(),
			})
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> MonadRec for TryThunkOkAppliedBrand<A> {
		/// Performs tail-recursive monadic computation over the error channel.
		///
		/// The step function returns `TryThunk<A, Step<E, E2>>`. The loop
		/// continues while the error is `Step::Loop`, terminates with the
		/// final error on `Step::Done`, and short-circuits on `Ok`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("The result of the computation.")]
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
		/// let result = tail_rec_m::<TryThunkOkAppliedBrand<i32>, _, _>(
		/// 	|x| {
		/// 		pure::<TryThunkOkAppliedBrand<i32>, _>(
		/// 			if x < 1000 { Step::Loop(x + 1) } else { Step::Done(x) },
		/// 		)
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), Err(1000));
		/// ```
		fn tail_rec_m<'a, E: 'a, E2: 'a>(
			f: impl Fn(E) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<E, E2>>)
			+ Clone
			+ 'a,
			e: E,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) {
			TryThunk::new(move || {
				let mut current = e;
				loop {
					match f(current).evaluate() {
						Err(Step::Loop(next)) => current = next,
						Err(Step::Done(res)) => break Err(res),
						Ok(a) => break Ok(a),
					}
				}
			})
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Foldable for TryThunkOkAppliedBrand<A> {
		/// Folds the `TryThunk` from the right (over error).
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
			"The `TryThunk` to fold."
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
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let result =
		/// 	fold_right::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _>(|a, b| a + b, 5, try_thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_right<'a, FnBrand, E: 'a + Clone, B: 'a>(
			func: impl Fn(E, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			match fa.evaluate() {
				Err(e) => func(e, initial),
				Ok(_) => initial,
			}
		}

		/// Folds the `TryThunk` from the left (over error).
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
			"The `TryThunk` to fold."
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
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let result =
		/// 	fold_left::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _>(|b, a| b + a, 5, try_thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_left<'a, FnBrand, E: 'a + Clone, B: 'a>(
			func: impl Fn(B, E) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			match fa.evaluate() {
				Err(e) => func(initial, e),
				Ok(_) => initial,
			}
		}

		/// Maps the value to a monoid and returns it (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The Thunk to fold.")]
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
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let result =
		/// 	fold_map::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _>(|a| a.to_string(), try_thunk);
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, E: 'a + Clone, M>(
			func: impl Fn(E) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			match fa.evaluate() {
				Err(e) => func(e),
				Ok(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	#[document_parameters("The try-thunk to format.")]
	impl<'a, A, E> fmt::Debug for TryThunk<'a, A, E> {
		/// Formats the try-thunk without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = TryThunk::new(|| Ok::<i32, ()>(42));
		/// assert_eq!(format!("{:?}", thunk), "TryThunk(<unevaluated>)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("TryThunk(<unevaluated>)")
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::Thunk,
		quickcheck_macros::quickcheck,
	};

	/// Tests success path.
	///
	/// Verifies that `TryThunk::ok` creates a successful computation.
	#[test]
	fn test_success() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests that `TryThunk::pure` still works but is deprecated.
	#[test]
	#[allow(deprecated)]
	fn test_pure_deprecated() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::pure(42);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests failure path.
	///
	/// Verifies that `TryThunk::err` creates a failed computation.
	#[test]
	fn test_failure() {
		let try_thunk: TryThunk<i32, &str> = TryThunk::err("error");
		assert_eq!(try_thunk.evaluate(), Err("error"));
	}

	/// Tests `TryThunk::map`.
	///
	/// Verifies that `map` transforms the success value.
	#[test]
	fn test_map() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(21).map(|x| x * 2);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunk::map_err`.
	///
	/// Verifies that `map_err` transforms the error value.
	#[test]
	fn test_map_err() {
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(21).map_err(|x| x * 2);
		assert_eq!(try_thunk.evaluate(), Err(42));
	}

	/// Tests `TryThunk::bind`.
	///
	/// Verifies that `bind` chains computations.
	#[test]
	fn test_bind() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(21).bind(|x| TryThunk::ok(x * 2));
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests borrowing in TryThunk.
	///
	/// Verifies that `TryThunk` can capture references.
	#[test]
	fn test_borrowing() {
		let x = 42;
		let try_thunk: TryThunk<&i32, ()> = TryThunk::new(|| Ok(&x));
		assert_eq!(try_thunk.evaluate(), Ok(&42));
	}

	/// Tests `TryThunk::bind` failure propagation.
	///
	/// Verifies that if the first computation fails, the second one is not executed.
	#[test]
	fn test_bind_failure() {
		let try_thunk = TryThunk::<i32, &str>::err("error").bind(|x| TryThunk::ok(x * 2));
		assert_eq!(try_thunk.evaluate(), Err("error"));
	}

	/// Tests `TryThunk::map` failure propagation.
	///
	/// Verifies that `map` is not executed if the computation fails.
	#[test]
	fn test_map_failure() {
		let try_thunk = TryThunk::<i32, &str>::err("error").map(|x| x * 2);
		assert_eq!(try_thunk.evaluate(), Err("error"));
	}

	/// Tests `TryThunk::map_err` success propagation.
	///
	/// Verifies that `map_err` is not executed if the computation succeeds.
	#[test]
	fn test_map_err_success() {
		let try_thunk = TryThunk::<i32, &str>::pure(42).map_err(|_| "new error");
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `From<Lazy>`.
	#[test]
	fn test_try_thunk_from_memo() {
		use crate::types::RcLazy;
		let memo = RcLazy::new(|| 42);
		let try_thunk: TryThunk<i32, ()> = TryThunk::from(memo);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `From<TryLazy>`.
	#[test]
	fn test_try_thunk_from_try_memo() {
		use crate::types::RcTryLazy;
		let memo = RcTryLazy::new(|| Ok(42));
		let try_thunk: TryThunk<i32, ()> = TryThunk::from(memo);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `Thunk::into_try`.
	///
	/// Verifies that `From<Thunk>` converts a `Thunk` into a `TryThunk` that succeeds.
	#[test]
	fn test_try_thunk_from_eval() {
		let eval = Thunk::pure(42);
		let try_thunk: TryThunk<i32, ()> = TryThunk::from(eval);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunk::defer`.
	#[test]
	fn test_defer() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::defer(|| TryThunk::ok(42));
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunk::catch`.
	///
	/// Verifies that `catch` recovers from failure.
	#[test]
	fn test_catch() {
		let try_thunk: TryThunk<i32, &str> = TryThunk::err("error").catch(|_| TryThunk::ok(42));
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunkErrAppliedBrand` (Functor over Success).
	#[test]
	fn test_try_thunk_with_err_brand() {
		use crate::{
			brands::*,
			functions::*,
		};

		// Functor (map over success)
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(10);
		let mapped = map::<TryThunkErrAppliedBrand<()>, _, _>(|x| x * 2, try_thunk);
		assert_eq!(mapped.evaluate(), Ok(20));

		// Pointed (pure -> ok)
		let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(42);
		assert_eq!(try_thunk.evaluate(), Ok(42));

		// Semimonad (bind over success)
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(10);
		let bound = bind::<TryThunkErrAppliedBrand<()>, _, _>(try_thunk, |x| {
			pure::<TryThunkErrAppliedBrand<()>, _>(x * 2)
		});
		assert_eq!(bound.evaluate(), Ok(20));

		// Foldable (fold over success)
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(10);
		let folded = fold_right::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _>(
			|x, acc| x + acc,
			5,
			try_thunk,
		);
		assert_eq!(folded, 15);
	}

	/// Tests `Bifunctor` for `TryThunkBrand`.
	#[test]
	fn test_bifunctor() {
		use crate::{
			brands::*,
			classes::bifunctor::*,
		};

		let x: TryThunk<i32, i32> = TryThunk::ok(5);
		assert_eq!(bimap::<TryThunkBrand, _, _, _, _>(|e| e + 1, |s| s * 2, x).evaluate(), Ok(10));

		let y: TryThunk<i32, i32> = TryThunk::err(5);
		assert_eq!(bimap::<TryThunkBrand, _, _, _, _>(|e| e + 1, |s| s * 2, y).evaluate(), Err(6));
	}

	/// Tests `Bifoldable` for `TryThunkBrand` with `bi_fold_right`.
	///
	/// Verifies that error values are folded with `f` and success values with `g`.
	#[test]
	fn test_bifoldable_right() {
		use crate::{
			brands::*,
			functions::*,
		};

		// Error case: f(3, 10) = 10 - 3 = 7
		assert_eq!(
			bi_fold_right::<RcFnBrand, TryThunkBrand, _, _, _>(
				|e: i32, acc| acc - e,
				|s: i32, acc| acc + s,
				10,
				TryThunk::err(3),
			),
			7
		);

		// Success case: g(5, 10) = 10 + 5 = 15
		assert_eq!(
			bi_fold_right::<RcFnBrand, TryThunkBrand, _, _, _>(
				|e: i32, acc| acc - e,
				|s: i32, acc| acc + s,
				10,
				TryThunk::ok(5),
			),
			15
		);
	}

	/// Tests `Bifoldable` for `TryThunkBrand` with `bi_fold_left`.
	///
	/// Verifies left-associative folding over both error and success values.
	#[test]
	fn test_bifoldable_left() {
		use crate::{
			brands::*,
			functions::*,
		};

		assert_eq!(
			bi_fold_left::<RcFnBrand, TryThunkBrand, _, _, _>(
				|acc, e: i32| acc - e,
				|acc, s: i32| acc + s,
				10,
				TryThunk::err(3),
			),
			7
		);

		assert_eq!(
			bi_fold_left::<RcFnBrand, TryThunkBrand, _, _, _>(
				|acc, e: i32| acc - e,
				|acc, s: i32| acc + s,
				10,
				TryThunk::ok(5),
			),
			15
		);
	}

	/// Tests `Bifoldable` for `TryThunkBrand` with `bi_fold_map`.
	///
	/// Verifies that both error and success values can be mapped to a monoid.
	#[test]
	fn test_bifoldable_map() {
		use crate::{
			brands::*,
			functions::*,
		};

		assert_eq!(
			bi_fold_map::<RcFnBrand, TryThunkBrand, _, _, _>(
				|e: i32| e.to_string(),
				|s: i32| s.to_string(),
				TryThunk::err(3),
			),
			"3".to_string()
		);

		assert_eq!(
			bi_fold_map::<RcFnBrand, TryThunkBrand, _, _, _>(
				|e: i32| e.to_string(),
				|s: i32| s.to_string(),
				TryThunk::ok(5),
			),
			"5".to_string()
		);
	}

	/// Tests `MonadRec` for `TryThunkOkAppliedBrand` (tail recursion over error).
	///
	/// Verifies that the loop continues on `Step::Loop` and terminates on `Step::Done`.
	#[test]
	fn test_monad_rec_ok_applied() {
		use crate::{
			brands::*,
			functions::*,
			types::Step,
		};

		let result = tail_rec_m::<TryThunkOkAppliedBrand<i32>, _, _>(
			|x| {
				pure::<TryThunkOkAppliedBrand<i32>, _>(
					if x < 100 { Step::Loop(x + 1) } else { Step::Done(x) },
				)
			},
			0,
		);
		assert_eq!(result.evaluate(), Err(100));
	}

	/// Tests `MonadRec` for `TryThunkOkAppliedBrand` short-circuits on `Ok`.
	///
	/// Verifies that encountering an `Ok` value terminates the loop immediately.
	#[test]
	fn test_monad_rec_ok_applied_short_circuit() {
		use crate::{
			brands::*,
			functions::*,
			types::Step,
		};

		let result = tail_rec_m::<TryThunkOkAppliedBrand<i32>, _, _>(
			|x: i32| {
				if x == 5 {
					TryThunk::ok(42)
				} else {
					pure::<TryThunkOkAppliedBrand<i32>, _>(Step::<i32, i32>::Loop(x + 1))
				}
			},
			0,
		);
		assert_eq!(result.evaluate(), Ok(42));
	}

	/// Tests `catch_unwind` on `TryThunk`.
	///
	/// Verifies that panics are caught and converted to `Err(String)`.
	#[test]
	fn test_catch_unwind() {
		let thunk = TryThunk::<i32, String>::catch_unwind(|| {
			if true {
				panic!("oops")
			}
			42
		});
		assert_eq!(thunk.evaluate(), Err("oops".to_string()));
	}

	/// Tests `catch_unwind` on `TryThunk` with a non-panicking closure.
	///
	/// Verifies that a successful closure wraps the value in `Ok`.
	#[test]
	fn test_catch_unwind_success() {
		let thunk = TryThunk::<i32, String>::catch_unwind(|| 42);
		assert_eq!(thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunkOkAppliedBrand` (Functor over Error).
	#[test]
	fn test_try_thunk_with_ok_brand() {
		use crate::{
			brands::*,
			functions::*,
		};

		// Functor (map over error)
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(10);
		let mapped = map::<TryThunkOkAppliedBrand<i32>, _, _>(|x| x * 2, try_thunk);
		assert_eq!(mapped.evaluate(), Err(20));

		// Pointed (pure -> err)
		let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(42);
		assert_eq!(try_thunk.evaluate(), Err(42));

		// Semimonad (bind over error)
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(10);
		let bound = bind::<TryThunkOkAppliedBrand<i32>, _, _>(try_thunk, |x| {
			pure::<TryThunkOkAppliedBrand<i32>, _>(x * 2)
		});
		assert_eq!(bound.evaluate(), Err(20));

		// Foldable (fold over error)
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(10);
		let folded = fold_right::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _>(
			|x, acc| x + acc,
			5,
			try_thunk,
		);
		assert_eq!(folded, 15);
	}

	/// Tests `From<Result>` with `Ok`.
	///
	/// Verifies that converting an `Ok` result produces a successful `TryThunk`.
	#[test]
	fn test_try_thunk_from_result_ok() {
		let try_thunk: TryThunk<i32, String> = TryThunk::from(Ok(42));
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `From<Result>` with `Err`.
	///
	/// Verifies that converting an `Err` result produces a failed `TryThunk`.
	#[test]
	fn test_try_thunk_from_result_err() {
		let try_thunk: TryThunk<i32, String> = TryThunk::from(Err("error".to_string()));
		assert_eq!(try_thunk.evaluate(), Err("error".to_string()));
	}

	// QuickCheck Law Tests

	// Functor Laws (via HKT, TryThunkErrAppliedBrand)

	/// Functor identity: `map(id, t).evaluate() == t.evaluate()`.
	#[quickcheck]
	fn functor_identity(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let t: TryThunk<i32, i32> = TryThunk::ok(x);
		map::<TryThunkErrAppliedBrand<i32>, _, _>(|a| a, t).evaluate() == Ok(x)
	}

	/// Functor composition: `map(f . g, t) == map(f, map(g, t))`.
	#[quickcheck]
	fn functor_composition(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let f = |a: i32| a.wrapping_add(1);
		let g = |a: i32| a.wrapping_mul(2);
		let lhs =
			map::<TryThunkErrAppliedBrand<i32>, _, _>(move |a| f(g(a)), TryThunk::ok(x)).evaluate();
		let rhs = map::<TryThunkErrAppliedBrand<i32>, _, _>(
			f,
			map::<TryThunkErrAppliedBrand<i32>, _, _>(g, TryThunk::ok(x)),
		)
		.evaluate();
		lhs == rhs
	}

	// Monad Laws (via HKT, TryThunkErrAppliedBrand)

	/// Monad left identity: `pure(a).bind(f) == f(a)` (for Ok values).
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let f = |x: i32| pure::<TryThunkErrAppliedBrand<i32>, _>(x.wrapping_mul(2));
		let lhs = bind::<TryThunkErrAppliedBrand<i32>, _, _>(
			pure::<TryThunkErrAppliedBrand<i32>, _>(a),
			f,
		)
		.evaluate();
		let rhs = f(a).evaluate();
		lhs == rhs
	}

	/// Monad right identity: `t.bind(pure) == t`.
	#[quickcheck]
	fn monad_right_identity(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let lhs = bind::<TryThunkErrAppliedBrand<i32>, _, _>(
			pure::<TryThunkErrAppliedBrand<i32>, _>(x),
			pure::<TryThunkErrAppliedBrand<i32>, _>,
		)
		.evaluate();
		lhs == Ok(x)
	}

	/// Monad associativity: `m.bind(f).bind(g) == m.bind(|a| f(a).bind(g))`.
	#[quickcheck]
	fn monad_associativity(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let f = |a: i32| pure::<TryThunkErrAppliedBrand<i32>, _>(a.wrapping_add(1));
		let g = |a: i32| pure::<TryThunkErrAppliedBrand<i32>, _>(a.wrapping_mul(3));
		let m: TryThunk<i32, i32> = TryThunk::ok(x);
		let m2: TryThunk<i32, i32> = TryThunk::ok(x);
		let lhs = bind::<TryThunkErrAppliedBrand<i32>, _, _>(
			bind::<TryThunkErrAppliedBrand<i32>, _, _>(m, f),
			g,
		)
		.evaluate();
		let rhs = bind::<TryThunkErrAppliedBrand<i32>, _, _>(m2, move |a| {
			bind::<TryThunkErrAppliedBrand<i32>, _, _>(f(a), g)
		})
		.evaluate();
		lhs == rhs
	}

	/// Error short-circuit: `TryThunk::err(e).bind(f).evaluate() == Err(e)`.
	#[quickcheck]
	fn error_short_circuit(e: i32) -> bool {
		let t: TryThunk<i32, i32> = TryThunk::err(e);
		t.bind(|x| TryThunk::ok(x.wrapping_add(1))).evaluate() == Err(e)
	}
}
