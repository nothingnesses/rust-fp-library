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
				Lazy,
				LazyConfig,
				Step,
				Thunk,
				TryLazy,
			},
		},
		fp_macros::{
			document_fields,
			document_parameters,
			document_type_parameters,
		},
	};

	/// A deferred computation that may fail with error type `E`.
	///
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
	#[document_fields("The closure that performs the computation.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let computation: TryThunk<i32, &str> = TryThunk::new(|| Ok(42));
	///
	/// match computation.evaluate() {
	/// 	Ok(val) => assert_eq!(val, 42),
	/// 	Err(_) => panic!("Should not fail"),
	/// }
	/// ```
	pub struct TryThunk<'a, A, E>(Box<dyn FnOnce() -> Result<A, E> + 'a>);

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
		#[document_type_parameters("The type of the thunk.")]
		///
		#[document_parameters("The thunk to wrap.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::new(|| Ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn new<F>(f: F) -> Self
		where
			F: FnOnce() -> Result<A, E> + 'a, {
			TryThunk(Box::new(f))
		}

		/// Returns a pure value (already computed).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance containing the value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn pure(a: A) -> Self
		where
			A: 'a, {
			TryThunk::new(move || Ok(a))
		}

		/// Defers a computation that returns a TryThunk.
		#[document_signature]
		///
		#[document_type_parameters("The type of the thunk.")]
		///
		#[document_parameters("The thunk that returns a `TryThunk`.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::defer(|| TryThunk::pure(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn defer<F>(f: F) -> Self
		where
			F: FnOnce() -> TryThunk<'a, A, E> + 'a, {
			TryThunk::new(move || f().evaluate())
		}

		/// Alias for [`pure`](Self::pure).
		///
		/// Creates a successful computation.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance containing the value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn ok(a: A) -> Self
		where
			A: 'a, {
			Self::pure(a)
		}

		/// Returns a pure error.
		#[document_signature]
		///
		#[document_parameters("The error to wrap.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance containing the error.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, &str> = TryThunk::err("error");
		/// assert_eq!(try_thunk.evaluate(), Err("error"));
		/// ```
		pub fn err(e: E) -> Self
		where
			E: 'a, {
			TryThunk::new(move || Err(e))
		}

		/// Monadic bind: chains computations.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the result of the new computation.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance representing the chained computation.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(21).bind(|x| TryThunk::pure(x * 2));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn bind<B: 'a, F>(
			self,
			f: F,
		) -> TryThunk<'a, B, E>
		where
			F: FnOnce(A) -> TryThunk<'a, B, E> + 'a, {
			TryThunk::new(move || match (self.0)() {
				Ok(a) => (f(a).0)(),
				Err(e) => Err(e),
			})
		}

		/// Functor map: transforms the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the result of the transformation.",
			"The type of the transformation function."
		)]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance with the transformed result.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(21).map(|x| x * 2);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn map<B: 'a, Func>(
			self,
			func: Func,
		) -> TryThunk<'a, B, E>
		where
			Func: FnOnce(A) -> B + 'a, {
			TryThunk::new(move || (self.0)().map(func))
		}

		/// Map error: transforms the error.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the new error.",
			"The type of the transformation function."
		)]
		///
		#[document_parameters("The function to apply to the error.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance with the transformed error.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, i32> = TryThunk::err(21).map_err(|x| x * 2);
		/// assert_eq!(try_thunk.evaluate(), Err(42));
		/// ```
		pub fn map_err<E2: 'a, F>(
			self,
			f: F,
		) -> TryThunk<'a, A, E2>
		where
			F: FnOnce(E) -> E2 + 'a, {
			TryThunk::new(move || (self.0)().map_err(f))
		}

		/// Recovers from an error.
		#[document_signature]
		///
		#[document_type_parameters("The type of the recovery function.")]
		///
		#[document_parameters("The function to apply to the error value.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` that attempts to recover from failure.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, &str> = TryThunk::err("error").catch(|_| TryThunk::pure(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn catch<F>(
			self,
			f: F,
		) -> Self
		where
			F: FnOnce(E) -> TryThunk<'a, A, E> + 'a, {
			TryThunk::new(move || match (self.0)() {
				Ok(a) => Ok(a),
				Err(e) => (f(e).0)(),
			})
		}

		/// Forces evaluation and returns the result.
		#[document_signature]
		///
		/// ### Returns
		///
		/// The result of the computation.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		pub fn evaluate(self) -> Result<A, E> {
			(self.0)()
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
		#[document_signature]
		#[document_parameters("The fallible lazy value to convert.")]
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
		fn from(eval: Thunk<'a, A>) -> Self {
			TryThunk::new(move || Ok(eval.evaluate()))
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
		#[document_type_parameters("The type of the thunk.")]
		///
		#[document_parameters("A thunk that produces the try thunk.")]
		///
		/// ### Returns
		///
		/// The deferred try thunk.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::Deferrable,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let task: TryThunk<i32, ()> = Deferrable::defer(|| TryThunk::pure(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		fn defer<F>(f: F) -> Self
		where
			F: FnOnce() -> Self + 'a,
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
			"The type of the result of the transformation.",
			"The type of the transformation function."
		)]
		///
		#[document_parameters(
			"The function to apply to the result of the computation.",
			"The `TryThunk` instance."
		)]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance with the transformed result.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let mapped = map::<TryThunkErrAppliedBrand<()>, _, _, _>(|x| x * 2, try_thunk);
		/// assert_eq!(mapped.evaluate(), Ok(20));
		/// ```
		fn map<'a, A: 'a, B: 'a, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Func: Fn(A) -> B + 'a, {
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
		/// ### Returns
		///
		/// A new `TryThunk` instance containing the value.
		///
		/// ### Examples
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
			TryThunk::pure(a)
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
			"The type of the result.",
			"The type of the binary function."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first `TryThunk`.",
			"The second `TryThunk`."
		)]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance containing the result of applying the function.
		///
		/// ### Examples
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
		/// let result = lift2::<TryThunkErrAppliedBrand<()>, _, _, _, _>(|a, b| a + b, eval1, eval2);
		/// assert_eq!(result.evaluate(), Ok(30));
		/// ```
		fn lift2<'a, A, B, C, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			Func: Fn(A, B) -> C + 'a,
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
		/// ### Returns
		///
		/// A new `TryThunk` instance containing the result of applying the function.
		///
		/// ### Examples
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
			"The type of the result of the new computation.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters(
			"The first `TryThunk`.",
			"The function to apply to the result of the computation."
		)]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance representing the chained computation.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let result = bind::<TryThunkErrAppliedBrand<()>, _, _, _>(try_thunk, |x| {
		/// 	pure::<TryThunkErrAppliedBrand<()>, _>(x * 2)
		/// });
		/// assert_eq!(result.evaluate(), Ok(20));
		/// ```
		fn bind<'a, A: 'a, B: 'a, Func>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a, {
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
			"The type of the result.",
			"The type of the step function."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		/// ### Returns
		///
		/// The result of the computation.
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
		/// let result = tail_rec_m::<TryThunkErrAppliedBrand<()>, _, _, _>(
		/// 	|x| {
		/// 		pure::<TryThunkErrAppliedBrand<()>, _>(
		/// 			if x < 1000 { Step::Loop(x + 1) } else { Step::Done(x) },
		/// 		)
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), Ok(1000));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a, F>(
			f: F,
			a: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>)
				+ Clone
				+ 'a, {
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
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		/// ### Returns
		///
		/// The final accumulator value.
		///
		/// ### Examples
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
		/// 	fold_right::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _, _>(|a, b| a + b, 5, try_thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a, B: 'a, Func>(
			func: Func,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			Func: Fn(A, B) -> B + 'a,
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
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		/// ### Returns
		///
		/// The final accumulator value.
		///
		/// ### Examples
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
		/// 	fold_left::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _, _>(|b, a| b + a, 5, try_thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a, B: 'a, Func>(
			func: Func,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			Func: Fn(B, A) -> B + 'a,
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
			"The type of the monoid.",
			"The type of the mapping function."
		)]
		///
		#[document_parameters("The mapping function.", "The Thunk to fold.")]
		///
		/// ### Returns
		///
		/// The monoid value.
		///
		/// ### Examples
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
		/// 	fold_map::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _, _>(|a| a.to_string(), try_thunk);
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, A: 'a, M, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			Func: Fn(A) -> M + 'a,
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
		/// ### Returns
		///
		/// A new `TryThunk` containing the combined result.
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
		/// ### Returns
		///
		/// A `TryThunk` producing the identity value of `A`.
		///
		/// ### Examples
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
			TryThunk::new(|| Ok(Monoid::empty()))
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
			"The type of the mapped success value.",
			"The type of the function to apply to the error.",
			"The type of the function to apply to the success."
		)]
		///
		#[document_parameters(
			"The function to apply to the error.",
			"The function to apply to the success.",
			"The `TryThunk` to map over."
		)]
		///
		/// ### Returns
		///
		/// A new `TryThunk` containing the mapped values.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::bifunctor::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x: TryThunk<i32, i32> = TryThunk::pure(5);
		/// assert_eq!(
		/// 	bimap::<TryThunkBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x).evaluate(),
		/// 	Ok(10)
		/// );
		///
		/// let y: TryThunk<i32, i32> = TryThunk::err(5);
		/// assert_eq!(
		/// 	bimap::<TryThunkBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, y).evaluate(),
		/// 	Err(6)
		/// );
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, F, G>(
			f: F,
			g: G,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
		where
			F: Fn(A) -> B + 'a,
			G: Fn(C) -> D + 'a, {
			TryThunk::new(move || match p.evaluate() {
				Ok(c) => Ok(g(c)),
				Err(a) => Err(f(a)),
			})
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
			"The type of the result of the transformation.",
			"The type of the transformation function."
		)]
		///
		#[document_parameters("The function to apply to the error.", "The `TryThunk` instance.")]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance with the transformed error.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let mapped = map::<TryThunkOkAppliedBrand<i32>, _, _, _>(|x| x * 2, try_thunk);
		/// assert_eq!(mapped.evaluate(), Err(20));
		/// ```
		fn map<'a, E: 'a, E2: 'a, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>)
		where
			Func: Fn(E) -> E2 + 'a, {
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
		/// ### Returns
		///
		/// A new `TryThunk` instance containing the value as an error.
		///
		/// ### Examples
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
			"The type of the result error value.",
			"The type of the binary function."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the errors.",
			"The first `TryThunk`.",
			"The second `TryThunk`."
		)]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance containing the result of applying the function to the errors.
		///
		/// ### Examples
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
		/// let result = lift2::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(|a, b| a + b, eval1, eval2);
		/// assert_eq!(result.evaluate(), Err(30));
		/// ```
		fn lift2<'a, E1, E2, E3, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E1>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E3>)
		where
			Func: Fn(E1, E2) -> E3 + 'a,
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
		/// ### Returns
		///
		/// A new `TryThunk` instance containing the result of applying the function.
		///
		/// ### Examples
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
			"The type of the result of the new computation (error).",
			"The type of the function to apply."
		)]
		///
		#[document_parameters(
			"The first `TryThunk`.",
			"The function to apply to the error result of the computation."
		)]
		///
		/// ### Returns
		///
		/// A new `TryThunk` instance representing the chained computation.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let result = bind::<TryThunkOkAppliedBrand<i32>, _, _, _>(try_thunk, |x| {
		/// 	pure::<TryThunkOkAppliedBrand<i32>, _>(x * 2)
		/// });
		/// assert_eq!(result.evaluate(), Err(20));
		/// ```
		fn bind<'a, E1: 'a, E2: 'a, Func>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E1>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>)
		where
			Func: Fn(E1) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) + 'a, {
			TryThunk::new(move || match ma.evaluate() {
				Ok(a) => Ok(a),
				Err(e) => func(e).evaluate(),
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
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		/// ### Returns
		///
		/// The final accumulator value.
		///
		/// ### Examples
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
		/// 	fold_right::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _, _>(|a, b| a + b, 5, try_thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_right<'a, FnBrand, E: 'a, B: 'a, Func>(
			func: Func,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> B
		where
			Func: Fn(E, B) -> B + 'a,
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
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		/// ### Returns
		///
		/// The final accumulator value.
		///
		/// ### Examples
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
		/// 	fold_left::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _, _>(|b, a| b + a, 5, try_thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_left<'a, FnBrand, E: 'a, B: 'a, Func>(
			func: Func,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> B
		where
			Func: Fn(B, E) -> B + 'a,
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
			"The type of the monoid.",
			"The type of the mapping function."
		)]
		///
		#[document_parameters("The mapping function.", "The Thunk to fold.")]
		///
		/// ### Returns
		///
		/// The monoid value.
		///
		/// ### Examples
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
		/// 	fold_map::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _, _>(|a| a.to_string(), try_thunk);
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, E: 'a, M, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> M
		where
			M: Monoid + 'a,
			Func: Fn(E) -> M + 'a,
			FnBrand: CloneableFn + 'a, {
			match fa.evaluate() {
				Err(e) => func(e),
				Ok(_) => M::empty(),
			}
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::Thunk,
	};

	/// Tests success path.
	///
	/// Verifies that `TryThunk::pure` creates a successful computation.
	#[test]
	fn test_success() {
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
		let try_thunk: TryThunk<i32, ()> = TryThunk::pure(21).map(|x| x * 2);
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
		let try_thunk: TryThunk<i32, ()> = TryThunk::pure(21).bind(|x| TryThunk::pure(x * 2));
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
		let try_thunk = TryThunk::<i32, &str>::err("error").bind(|x| TryThunk::pure(x * 2));
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
		let try_thunk: TryThunk<i32, ()> = TryThunk::defer(|| TryThunk::pure(42));
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunk::catch`.
	///
	/// Verifies that `catch` recovers from failure.
	#[test]
	fn test_catch() {
		let try_thunk: TryThunk<i32, &str> = TryThunk::err("error").catch(|_| TryThunk::pure(42));
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
		let try_thunk: TryThunk<i32, ()> = TryThunk::pure(10);
		let mapped = map::<TryThunkErrAppliedBrand<()>, _, _, _>(|x| x * 2, try_thunk);
		assert_eq!(mapped.evaluate(), Ok(20));

		// Pointed (pure -> ok)
		let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(42);
		assert_eq!(try_thunk.evaluate(), Ok(42));

		// Semimonad (bind over success)
		let try_thunk: TryThunk<i32, ()> = TryThunk::pure(10);
		let bound = bind::<TryThunkErrAppliedBrand<()>, _, _, _>(try_thunk, |x| {
			pure::<TryThunkErrAppliedBrand<()>, _>(x * 2)
		});
		assert_eq!(bound.evaluate(), Ok(20));

		// Foldable (fold over success)
		let try_thunk: TryThunk<i32, ()> = TryThunk::pure(10);
		let folded = fold_right::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _, _>(
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

		let x: TryThunk<i32, i32> = TryThunk::pure(5);
		assert_eq!(
			bimap::<TryThunkBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x).evaluate(),
			Ok(10)
		);

		let y: TryThunk<i32, i32> = TryThunk::err(5);
		assert_eq!(
			bimap::<TryThunkBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, y).evaluate(),
			Err(6)
		);
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
		let mapped = map::<TryThunkOkAppliedBrand<i32>, _, _, _>(|x| x * 2, try_thunk);
		assert_eq!(mapped.evaluate(), Err(20));

		// Pointed (pure -> err)
		let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(42);
		assert_eq!(try_thunk.evaluate(), Err(42));

		// Semimonad (bind over error)
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(10);
		let bound = bind::<TryThunkOkAppliedBrand<i32>, _, _, _>(try_thunk, |x| {
			pure::<TryThunkOkAppliedBrand<i32>, _>(x * 2)
		});
		assert_eq!(bound.evaluate(), Err(20));

		// Foldable (fold over error)
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(10);
		let folded = fold_right::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _, _>(
			|x, acc| x + acc,
			5,
			try_thunk,
		);
		assert_eq!(folded, 15);
	}
}
