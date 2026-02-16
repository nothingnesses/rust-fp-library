//! Functional programming trait implementations for the standard library [`Result`] type.
//!
//! Extends `Result` with dual functor/monad instances: [`ResultWithErrBrand`](crate::brands::ResultWithErrBrand) (standard Result monad) functors over the success value, while [`ResultWithOkBrand`](crate::brands::ResultWithOkBrand) functors over the error value.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
			classes::{
				Applicative, ApplyFirst, ApplySecond, Bifunctor, CloneableFn, Foldable, Functor,
				Lift, Monoid, ParFoldable, Pointed, Semiapplicative, Semimonad, SendCloneableFn,
				Traversable,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::{document_parameters, document_type_parameters},
	};

	impl_kind! {
		/// HKT branding for the `Result` type.
		///
		/// The type parameters for `Of` are ordered `E`, then `A` (Error, then Success).
		/// This follows functional programming conventions (like Haskell's `Either e a`)
		/// where the right-most type parameter is the "success" value, allowing the
		/// type to form a `Monad` over the success type by fixing the error type.
		for ResultBrand {
			type Of<A, B> = Result<B, A>;
		}
	}

	impl Bifunctor for ResultBrand {
		/// Maps functions over the values in the result.
		///
		/// This method applies one function to the error value and another to the success value.
		#[document_signature]
		///
		#[document_type_parameters(
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
			"The result to map over."
		)]
		///
		/// ### Returns
		///
		/// A new result containing the mapped values.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::bifunctor::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Result<i32, i32> = Ok(5);
		/// assert_eq!(bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x), Ok(10));
		///
		/// let y: Result<i32, i32> = Err(5);
		/// assert_eq!(bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, y), Err(6));
		/// ```
		fn bimap<A, B, C, D, F, G>(
			f: F,
			g: G,
			p: Apply!(<Self as Kind!( type Of<A, B>; )>::Of<A, C>),
		) -> Apply!(<Self as Kind!( type Of<A, B>; )>::Of<B, D>)
		where
			F: Fn(A) -> B,
			G: Fn(C) -> D,
		{
			match p {
				Ok(c) => Ok(g(c)),
				Err(a) => Err(f(a)),
			}
		}
	}

	// ResultWithErrBrand<E> (Functor over T)

	impl_kind! {
		#[document_type_parameters("The error type.")]
		impl<E: 'static> for ResultWithErrBrand<E> {
			type Of<A> = Result<A, E>;
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Functor for ResultWithErrBrand<E> {
		/// Maps a function over the value in the result.
		///
		/// This method applies a function to the value inside the result if it is `Ok`, producing a new result with the transformed value. If the result is `Err`, it is returned unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the value inside the result.",
			"The type of the result of applying the function.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply.", "The result to map over.")]
		///
		/// ### Returns
		///
		/// A new result containing the result of applying the function, or the original error.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(map::<ResultWithErrBrand<()>, _, _, _>(|x: i32| x * 2, Ok(5)), Ok(10));
		/// assert_eq!(map::<ResultWithErrBrand<i32>, _, _, _>(|x: i32| x * 2, Err(1)), Err(1));
		/// ```
		fn map<A, B, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
		where
			Func: Fn(A) -> B,
		{
			fa.map(func)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> Lift for ResultWithErrBrand<E> {
		/// Lifts a binary function into the result context.
		///
		/// This method lifts a binary function to operate on values within the result context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result.",
			"The type of the binary function."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first result.",
			"The second result."
		)]
		///
		/// ### Returns
		///
		/// `Ok(f(a, b))` if both results are `Ok`, otherwise the first error encountered.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	lift2::<ResultWithErrBrand<()>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Ok(2)),
		/// 	Ok(3)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultWithErrBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Err(2)),
		/// 	Err(2)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultWithErrBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Ok(2)),
		/// 	Err(1)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultWithErrBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
		/// 	Err(1)
		/// );
		/// ```
		fn lift2<A, B, C, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
			fb: Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<C>)
		where
			Func: Fn(A, B) -> C,
		{
			match (fa, fb) {
				(Ok(a), Ok(b)) => Ok(func(a, b)),
				(Err(e), _) => Err(e),
				(_, Err(e)) => Err(e),
			}
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Pointed for ResultWithErrBrand<E> {
		/// Wraps a value in a result.
		///
		/// This method wraps a value in the `Ok` variant of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters("The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		/// ### Returns
		///
		/// `Ok(a)`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ResultWithErrBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(pure::<ResultWithErrBrand<()>, _>(5), Ok(5));
		/// ```
		fn pure<A>(a: A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<A>) {
			Ok(a)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> ApplyFirst for ResultWithErrBrand<E> {}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> ApplySecond for ResultWithErrBrand<E> {}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> Semiapplicative for ResultWithErrBrand<E> {
		/// Applies a wrapped function to a wrapped value.
		///
		/// This method applies a function wrapped in a result to a value wrapped in a result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The result containing the function.",
			"The result containing the value."
		)]
		///
		/// ### Returns
		///
		/// `Ok(f(a))` if both are `Ok`, otherwise the first error encountered.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	Kind,
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f: Result<_, ()> = Ok(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(apply::<RcFnBrand, ResultWithErrBrand<()>, _, _>(f, Ok(5)), Ok(10));
		/// let f: Result<_, i32> = Ok(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(f, Err(1)), Err(1));
		///
		/// let f_err: Result<_, i32> = Err(1);
		/// assert_eq!(apply::<RcFnBrand, ResultWithErrBrand<i32>, i32, i32>(f_err, Ok(5)), Err(1));
		/// ```
		fn apply<FnBrand: CloneableFn, A: Clone, B>(
			ff: Apply!(<Self as Kind!( type Of<T>; )>::Of<<FnBrand as CloneableFn>::Of<A, B>>),
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>) {
			match (ff, fa) {
				(Ok(f), Ok(a)) => Ok(f(a)),
				(Err(e), _) => Err(e),
				(_, Err(e)) => Err(e),
			}
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> Semimonad for ResultWithErrBrand<E> {
		/// Chains result computations.
		///
		/// This method chains two computations, where the second computation depends on the result of the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the result of the first computation.",
			"The type of the result of the second computation.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters(
			"The first result.",
			"The function to apply to the value inside the result."
		)]
		///
		/// ### Returns
		///
		/// The result of applying `f` to the value if `ma` is `Ok`, otherwise the original error.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ResultWithErrBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(bind::<ResultWithErrBrand<()>, _, _, _>(Ok(5), |x| Ok(x * 2)), Ok(10));
		/// assert_eq!(bind::<ResultWithErrBrand<i32>, _, _, _>(Ok(5), |_| Err::<i32, _>(1)), Err(1));
		/// assert_eq!(bind::<ResultWithErrBrand<i32>, _, _, _>(Err(1), |x: i32| Ok(x * 2)), Err(1));
		/// ```
		fn bind<A, B, Func>(
			ma: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
		where
			Func: Fn(A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
		{
			ma.and_then(func)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Foldable for ResultWithErrBrand<E> {
		/// Folds the result from the right.
		///
		/// This method performs a right-associative fold of the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		/// ### Returns
		///
		/// `func(a, initial)` if `fa` is `Ok(a)`, otherwise `initial`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ResultWithErrBrand<()>, _, _, _>(|x, acc| x + acc, 0, Ok(5)),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Err(1)),
		/// 	0
		/// );
		/// ```
		fn fold_right<FnBrand, A, B, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			F: Fn(A, B) -> B,
			FnBrand: CloneableFn,
		{
			match fa {
				Ok(a) => func(a, initial),
				Err(_) => initial,
			}
		}

		/// Folds the result from the left.
		///
		/// This method performs a left-associative fold of the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		/// ### Returns
		///
		/// `func(initial, a)` if `fa` is `Ok(a)`, otherwise `initial`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ResultWithErrBrand<()>, _, _, _>(|acc, x| acc + x, 0, Ok(5)),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(|acc, x: i32| acc + x, 0, Err(1)),
		/// 	0
		/// );
		/// ```
		fn fold_left<FnBrand, A, B, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			F: Fn(B, A) -> B,
			FnBrand: CloneableFn,
		{
			match fa {
				Ok(a) => func(initial, a),
				Err(_) => initial,
			}
		}

		/// Maps the value to a monoid and returns it.
		///
		/// This method maps the element of the result to a monoid and then returns it.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid.",
			"The type of the mapping function."
		)]
		///
		#[document_parameters("The mapping function.", "The result to fold.")]
		///
		/// ### Returns
		///
		/// `func(a)` if `fa` is `Ok(a)`, otherwise `M::empty()`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ResultWithErrBrand<()>, _, _, _>(|x: i32| x.to_string(), Ok(5)),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(|x: i32| x.to_string(), Err(1)),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<FnBrand, A, M, F>(
			func: F,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> M
		where
			M: Monoid,
			F: Fn(A) -> M,
			FnBrand: CloneableFn,
		{
			match fa {
				Ok(a) => func(a),
				Err(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> Traversable for ResultWithErrBrand<E> {
		/// Traverses the result with an applicative function.
		///
		/// This method maps the element of the result to a computation, evaluates it, and combines the result into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply.", "The result to traverse.")]
		///
		/// ### Returns
		///
		/// The result wrapped in the applicative context.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		ResultWithErrBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<ResultWithErrBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Ok(5)),
		/// 	Some(Ok(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<ResultWithErrBrand<i32>, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Err(1)),
		/// 	Some(Err(1))
		/// );
		/// assert_eq!(
		/// 	traverse::<ResultWithErrBrand<()>, _, _, OptionBrand, _>(|_| None::<i32>, Ok(5)),
		/// 	None
		/// );
		/// ```
		fn traverse<A: Clone, B: Clone, F: Applicative, Func>(
			func: Func,
			ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)>)
		where
			Func: Fn(A) -> Apply!(<F as Kind!( type Of<T>; )>::Of<B>),
			Apply!(<Self as Kind!( type Of<T>; )>::Of<B>): Clone,
		{
			match ta {
				Ok(a) => F::map(|b| Ok(b), func(a)),
				Err(e) => F::pure(Err(e)),
			}
		}

		/// Sequences a result of applicative.
		///
		/// This method evaluates the computation inside the result and accumulates the result into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The result containing the applicative value.")]
		///
		/// ### Returns
		///
		/// The result wrapped in the applicative context.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		ResultWithErrBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(sequence::<ResultWithErrBrand<()>, _, OptionBrand>(Ok(Some(5))), Some(Ok(5)));
		/// assert_eq!(
		/// 	sequence::<ResultWithErrBrand<i32>, i32, OptionBrand>(Err::<Option<i32>, _>(1)),
		/// 	Some(Err::<i32, i32>(1))
		/// );
		/// assert_eq!(sequence::<ResultWithErrBrand<()>, _, OptionBrand>(Ok(None::<i32>)), None);
		/// ```
		fn sequence<A: Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<Apply!(<F as Kind!( type Of<T>; )>::Of<A>)>)
		) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<A>)>)
		where
			Apply!(<F as Kind!( type Of<T>; )>::Of<A>): Clone,
			Apply!(<Self as Kind!( type Of<T>; )>::Of<A>): Clone,
		{
			match ta {
				Ok(fa) => F::map(|a| Ok(a), fa),
				Err(e) => F::pure(Err(e)),
			}
		}
	}

	// ResultWithOkBrand<T> (Functor over E)

	impl_kind! {
		#[document_type_parameters("The success type.")]
		impl<T: 'static> for ResultWithOkBrand<T> {
			type Of<A> = Result<T, A>;
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: 'static> Functor for ResultWithOkBrand<T> {
		/// Maps a function over the error value in the result.
		///
		/// This method applies a function to the error value inside the result if it is `Err`, producing a new result with the transformed error. If the result is `Ok`, it is returned unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the error value inside the result.",
			"The type of the result of applying the function.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply to the error.", "The result to map over.")]
		///
		/// ### Returns
		///
		/// A new result containing the mapped error, or the original success value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(map::<ResultWithOkBrand<i32>, _, _, _>(|x: i32| x * 2, Err(5)), Err(10));
		/// assert_eq!(map::<ResultWithOkBrand<i32>, _, _, _>(|x: i32| x * 2, Ok(1)), Ok(1));
		/// ```
		fn map<A, B, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
		where
			Func: Fn(A) -> B,
		{
			match fa {
				Ok(t) => Ok(t),
				Err(e) => Err(func(e)),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> Lift for ResultWithOkBrand<T> {
		/// Lifts a binary function into the result context (over error).
		///
		/// This method lifts a binary function to operate on error values within the result context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the first error value.",
			"The type of the second error value.",
			"The type of the result error value.",
			"The type of the binary function."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the errors.",
			"The first result.",
			"The second result."
		)]
		///
		/// ### Returns
		///
		/// `Err(f(a, b))` if both results are `Err`, otherwise the first success encountered.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
		/// 	Err(3)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Ok(2)),
		/// 	Ok(2)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Err(2)),
		/// 	Ok(1)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Ok(2)),
		/// 	Ok(1)
		/// );
		/// ```
		fn lift2<A, B, C, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
			fb: Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<C>)
		where
			Func: Fn(A, B) -> C,
		{
			match (fa, fb) {
				(Err(a), Err(b)) => Err(func(a, b)),
				(Ok(t), _) => Ok(t),
				(_, Ok(t)) => Ok(t),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: 'static> Pointed for ResultWithOkBrand<T> {
		/// Wraps a value in a result (as error).
		///
		/// This method wraps a value in the `Err` variant of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters("The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		/// ### Returns
		///
		/// `Err(a)`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ResultWithOkBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(pure::<ResultWithOkBrand<()>, _>(5), Err(5));
		/// ```
		fn pure<A>(a: A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<A>) {
			Err(a)
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> ApplyFirst for ResultWithOkBrand<T> {}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> ApplySecond for ResultWithOkBrand<T> {}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> Semiapplicative for ResultWithOkBrand<T> {
		/// Applies a wrapped function to a wrapped value (over error).
		///
		/// This method applies a function wrapped in a result (as error) to a value wrapped in a result (as error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The result containing the function (in Err).",
			"The result containing the value (in Err)."
		)]
		///
		/// ### Returns
		///
		/// `Err(f(a))` if both are `Err`, otherwise the first success encountered.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	Kind,
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f: Result<(), _> = Err(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(apply::<RcFnBrand, ResultWithOkBrand<()>, _, _>(f, Err(5)), Err(10));
		/// let f: Result<i32, _> = Err(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(apply::<RcFnBrand, ResultWithOkBrand<i32>, _, _>(f, Ok(1)), Ok(1));
		///
		/// let f_ok: Result<i32, _> = Ok(1);
		/// assert_eq!(apply::<RcFnBrand, ResultWithOkBrand<i32>, i32, i32>(f_ok, Err(5)), Ok(1));
		/// ```
		fn apply<FnBrand: CloneableFn, A: Clone, B>(
			ff: Apply!(<Self as Kind!( type Of<T>; )>::Of<<FnBrand as CloneableFn>::Of<A, B>>),
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>) {
			match (ff, fa) {
				(Err(f), Err(a)) => Err(f(a)),
				(Ok(t), _) => Ok(t),
				(_, Ok(t)) => Ok(t),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> Semimonad for ResultWithOkBrand<T> {
		/// Chains result computations (over error).
		///
		/// This method chains two computations, where the second computation depends on the result of the first (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the result of the first computation.",
			"The type of the result of the second computation.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The first result.", "The function to apply to the error value.")]
		///
		/// ### Returns
		///
		/// The result of applying `f` to the error if `ma` is `Err`, otherwise the original success.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ResultWithOkBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(bind::<ResultWithOkBrand<()>, _, _, _>(Err(5), |x| Err(x * 2)), Err(10));
		/// assert_eq!(bind::<ResultWithOkBrand<i32>, _, _, _>(Err(5), |_| Ok::<_, i32>(1)), Ok(1));
		/// assert_eq!(bind::<ResultWithOkBrand<i32>, _, _, _>(Ok(1), |x: i32| Err(x * 2)), Ok(1));
		/// ```
		fn bind<A, B, Func>(
			ma: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
		where
			Func: Fn(A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
		{
			match ma {
				Ok(t) => Ok(t),
				Err(e) => func(e),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: 'static> Foldable for ResultWithOkBrand<T> {
		/// Folds the result from the right (over error).
		///
		/// This method performs a right-associative fold of the result (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		/// ### Returns
		///
		/// `func(a, initial)` if `fa` is `Err(a)`, otherwise `initial`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ResultWithOkBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Err(1)),
		/// 	1
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ResultWithOkBrand<()>, _, _, _>(|x: i32, acc| x + acc, 0, Ok(())),
		/// 	0
		/// );
		/// ```
		fn fold_right<FnBrand, A, B, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			F: Fn(A, B) -> B,
			FnBrand: CloneableFn,
		{
			match fa {
				Err(e) => func(e, initial),
				Ok(_) => initial,
			}
		}

		/// Folds the result from the left (over error).
		///
		/// This method performs a left-associative fold of the result (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		/// ### Returns
		///
		/// `func(initial, a)` if `fa` is `Err(a)`, otherwise `initial`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ResultWithOkBrand<()>, _, _, _>(|acc, x: i32| acc + x, 0, Err(5)),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ResultWithOkBrand<i32>, _, _, _>(|acc, x: i32| acc + x, 0, Ok(1)),
		/// 	0
		/// );
		/// ```
		fn fold_left<FnBrand, A, B, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			F: Fn(B, A) -> B,
			FnBrand: CloneableFn,
		{
			match fa {
				Err(e) => func(initial, e),
				Ok(_) => initial,
			}
		}

		/// Maps the value to a monoid and returns it (over error).
		///
		/// This method maps the element of the result to a monoid and then returns it (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid.",
			"The type of the mapping function."
		)]
		///
		#[document_parameters("The mapping function.", "The result to fold.")]
		///
		/// ### Returns
		///
		/// `func(a)` if `fa` is `Err(a)`, otherwise `M::empty()`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ResultWithOkBrand<()>, _, _, _>(|x: i32| x.to_string(), Err(5)),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ResultWithOkBrand<i32>, _, _, _>(|x: i32| x.to_string(), Ok(1)),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<FnBrand, A, M, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> M
		where
			M: Monoid,
			Func: Fn(A) -> M,
			FnBrand: CloneableFn,
		{
			match fa {
				Err(e) => func(e),
				Ok(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> Traversable for ResultWithOkBrand<T> {
		/// Traverses the result with an applicative function (over error).
		///
		/// This method maps the element of the result to a computation, evaluates it, and combines the result into an applicative context (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply.", "The result to traverse.")]
		///
		/// ### Returns
		///
		/// The result wrapped in the applicative context.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		ResultWithOkBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<ResultWithOkBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Err(5)),
		/// 	Some(Err(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<ResultWithOkBrand<i32>, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Ok(1)),
		/// 	Some(Ok(1))
		/// );
		/// assert_eq!(
		/// 	traverse::<ResultWithOkBrand<()>, _, _, OptionBrand, _>(|_| None::<i32>, Err(5)),
		/// 	None
		/// );
		/// ```
		fn traverse<A: Clone, B: Clone, F: Applicative, Func>(
			func: Func,
			ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)>)
		where
			Func: Fn(A) -> Apply!(<F as Kind!( type Of<T>; )>::Of<B>),
			Apply!(<Self as Kind!( type Of<T>; )>::Of<B>): Clone,
		{
			match ta {
				Err(e) => F::map(|b| Err(b), func(e)),
				Ok(t) => F::pure(Ok(t)),
			}
		}

		/// Sequences a result of applicative (over error).
		///
		/// This method evaluates the computation inside the result and accumulates the result into an applicative context (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The result containing the applicative value.")]
		///
		/// ### Returns
		///
		/// The result wrapped in the applicative context.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		ResultWithOkBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(sequence::<ResultWithOkBrand<()>, _, OptionBrand>(Err(Some(5))), Some(Err(5)));
		/// assert_eq!(
		/// 	sequence::<ResultWithOkBrand<i32>, i32, OptionBrand>(Ok::<_, Option<i32>>(1)),
		/// 	Some(Ok::<i32, i32>(1))
		/// );
		/// assert_eq!(sequence::<ResultWithOkBrand<()>, _, OptionBrand>(Err(None::<i32>)), None);
		/// ```
		fn sequence<A: Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<Apply!(<F as Kind!( type Of<T>; )>::Of<A>)>)
		) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<A>)>)
		where
			Apply!(<F as Kind!( type Of<T>; )>::Of<A>): Clone,
			Apply!(<Self as Kind!( type Of<T>; )>::Of<A>): Clone,
		{
			match ta {
				Err(fe) => F::map(|e| Err(e), fe),
				Ok(t) => F::pure(Ok(t)),
			}
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> ParFoldable for ResultWithErrBrand<E> {
		/// Maps the value to a monoid and returns it, or returns empty, in parallel.
		///
		/// This method maps the element of the result to a monoid and then returns it. The mapping operation may be executed in parallel.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The thread-safe function to map each element to a monoid.",
			"The result to fold."
		)]
		///
		/// ### Returns
		///
		/// The combined monoid value.
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
		/// let x: Result<i32, ()> = Ok(5);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		/// assert_eq!(
		/// 	par_fold_map::<ArcFnBrand, ResultWithErrBrand<()>, _, _>(f.clone(), x),
		/// 	"5".to_string()
		/// );
		///
		/// let x_err: Result<i32, i32> = Err(1);
		/// assert_eq!(par_fold_map::<ArcFnBrand, ResultWithErrBrand<i32>, _, _>(f, x_err), "".to_string());
		/// ```
		fn par_fold_map<FnBrand, A, M>(
			func: <FnBrand as SendCloneableFn>::SendOf<A, M>,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> M
		where
			FnBrand: SendCloneableFn,
			A: Clone + Send + Sync,
			M: Monoid + Send + Sync,
		{
			match fa {
				Ok(a) => func(a),
				Err(_) => M::empty(),
			}
		}

		/// Folds the result from the right in parallel.
		///
		/// This method folds the result by applying a function from right to left, potentially in parallel.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The element type.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The thread-safe function to apply to each element and the accumulator.",
			"The initial value.",
			"The result to fold."
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
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Result<i32, ()> = Ok(5);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		/// assert_eq!(par_fold_right::<ArcFnBrand, ResultWithErrBrand<()>, _, _>(f.clone(), 10, x), 15);
		///
		/// let x_err: Result<i32, i32> = Err(1);
		/// assert_eq!(par_fold_right::<ArcFnBrand, ResultWithErrBrand<i32>, _, _>(f, 10, x_err), 10);
		/// ```
		fn par_fold_right<FnBrand, A, B>(
			func: <FnBrand as SendCloneableFn>::SendOf<(A, B), B>,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			FnBrand: SendCloneableFn,
			A: Clone + Send + Sync,
			B: Send + Sync,
		{
			match fa {
				Ok(a) => func((a, initial)),
				Err(_) => initial,
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: 'static> ParFoldable for ResultWithOkBrand<T> {
		/// Maps the value to a monoid and returns it, or returns empty, in parallel (over error).
		///
		/// This method maps the element of the result to a monoid and then returns it (over error). The mapping operation may be executed in parallel.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The thread-safe function to map each element to a monoid.",
			"The result to fold."
		)]
		///
		/// ### Returns
		///
		/// The combined monoid value.
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
		/// let x: Result<(), i32> = Err(5);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		/// assert_eq!(
		/// 	par_fold_map::<ArcFnBrand, ResultWithOkBrand<()>, _, _>(f.clone(), x),
		/// 	"5".to_string()
		/// );
		///
		/// let x_ok: Result<i32, i32> = Ok(1);
		/// assert_eq!(par_fold_map::<ArcFnBrand, ResultWithOkBrand<i32>, _, _>(f, x_ok), "".to_string());
		/// ```
		fn par_fold_map<FnBrand, A, M>(
			func: <FnBrand as SendCloneableFn>::SendOf<A, M>,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> M
		where
			FnBrand: SendCloneableFn,
			A: Clone + Send + Sync,
			M: Monoid + Send + Sync,
		{
			match fa {
				Err(e) => func(e),
				Ok(_) => M::empty(),
			}
		}

		/// Folds the result from the right in parallel (over error).
		///
		/// This method folds the result by applying a function from right to left, potentially in parallel (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The element type.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The thread-safe function to apply to each element and the accumulator.",
			"The initial value.",
			"The result to fold."
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
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Result<(), i32> = Err(5);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		/// assert_eq!(par_fold_right::<ArcFnBrand, ResultWithOkBrand<()>, _, _>(f.clone(), 10, x), 15);
		///
		/// let x_ok: Result<i32, i32> = Ok(1);
		/// assert_eq!(par_fold_right::<ArcFnBrand, ResultWithOkBrand<i32>, _, _>(f, 10, x_ok), 10);
		/// ```
		fn par_fold_right<FnBrand, A, B>(
			func: <FnBrand as SendCloneableFn>::SendOf<(A, B), B>,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			FnBrand: SendCloneableFn,
			A: Clone + Send + Sync,
			B: Send + Sync,
		{
			match fa {
				Err(e) => func((e, initial)),
				Ok(_) => initial,
			}
		}
	}
}

#[cfg(test)]
mod tests {

	use {
		crate::{
			brands::*,
			classes::{CloneableFn, bifunctor::*},
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	// Bifunctor Tests

	/// Tests `bimap` on `Ok` and `Err`.
	#[test]
	fn test_bimap() {
		let x: Result<i32, i32> = Ok(5);
		assert_eq!(bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x), Ok(10));

		let y: Result<i32, i32> = Err(5);
		assert_eq!(bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, y), Err(6));
	}

	// Bifunctor Laws

	/// Tests the identity law for Bifunctor.
	#[quickcheck]
	fn bifunctor_identity(x: Result<i32, i32>) -> bool {
		bimap::<ResultBrand, _, _, _, _, _, _>(identity, identity, x) == x
	}

	/// Tests the composition law for Bifunctor.
	#[quickcheck]
	fn bifunctor_composition(x: Result<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		let h = |x: i32| x.wrapping_sub(1);
		let i = |x: i32| if x == 0 { 0 } else { x.wrapping_div(2) };

		bimap::<ResultBrand, _, _, _, _, _, _>(compose(f, g), compose(h, i), x)
			== bimap::<ResultBrand, _, _, _, _, _, _>(
				f,
				h,
				bimap::<ResultBrand, _, _, _, _, _, _>(g, i, x),
			)
	}

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: Result<i32, i32>) -> bool {
		map::<ResultWithErrBrand<i32>, _, _, _>(identity, x) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Result<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<ResultWithErrBrand<i32>, _, _, _>(compose(f, g), x)
			== map::<ResultWithErrBrand<i32>, _, _, _>(
				f,
				map::<ResultWithErrBrand<i32>, _, _, _>(g, x),
			)
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Result<i32, i32>) -> bool {
		apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(
			pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(
			pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(f)),
			pure::<ResultWithErrBrand<i32>, _>(x),
		) == pure::<ResultWithErrBrand<i32>, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: Result<i32, i32>,
		u_is_ok: bool,
		v_is_ok: bool,
	) -> bool {
		let v_fn = |x: i32| x.wrapping_mul(2);
		let u_fn = |x: i32| x.wrapping_add(1);

		let v = if v_is_ok {
			pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(v_fn))
		} else {
			Err(100)
		};
		let u = if u_is_ok {
			pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(u_fn))
		} else {
			Err(200)
		};

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(v.clone(), w);
		let rhs = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let uv = match (u, v) {
			(Ok(uf), Ok(vf)) => {
				let composed = move |x| uf(vf(x));
				Ok(<RcFnBrand as CloneableFn>::new(composed))
			}
			(Err(e), _) => Err(e),
			(_, Err(e)) => Err(e),
		};

		let lhs = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(f));

		let lhs = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(
			u.clone(),
			pure::<ResultWithErrBrand<i32>, _>(y),
		);

		let rhs_fn =
			<RcFnBrand as CloneableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(
			pure::<ResultWithErrBrand<i32>, _>(rhs_fn),
			u,
		);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| -> Result<i32, i32> { Err(x.wrapping_mul(2)) };
		bind::<ResultWithErrBrand<i32>, _, _, _>(pure::<ResultWithErrBrand<i32>, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Result<i32, i32>) -> bool {
		bind::<ResultWithErrBrand<i32>, _, _, _>(m, pure::<ResultWithErrBrand<i32>, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Result<i32, i32>) -> bool {
		let f = |x: i32| -> Result<i32, i32> { Err(x.wrapping_mul(2)) };
		let g = |x: i32| -> Result<i32, i32> { Err(x.wrapping_add(1)) };
		bind::<ResultWithErrBrand<i32>, _, _, _>(bind::<ResultWithErrBrand<i32>, _, _, _>(m, f), g)
			== bind::<ResultWithErrBrand<i32>, _, _, _>(m, |x| {
				bind::<ResultWithErrBrand<i32>, _, _, _>(f(x), g)
			})
	}

	// Edge Cases

	/// Tests `map` on `Err`.
	#[test]
	fn map_err() {
		assert_eq!(
			map::<ResultWithErrBrand<i32>, _, _, _>(|x: i32| x + 1, Err::<i32, i32>(1)),
			Err(1)
		);
	}

	/// Tests `bind` on `Err`.
	#[test]
	fn bind_err() {
		assert_eq!(
			bind::<ResultWithErrBrand<i32>, _, _, _>(Err::<i32, i32>(1), |x: i32| Ok(x + 1)),
			Err(1)
		);
	}

	/// Tests `bind` returning `Err`.
	#[test]
	fn bind_returning_err() {
		assert_eq!(bind::<ResultWithErrBrand<i32>, _, _, _>(Ok(1), |_| Err::<i32, i32>(2)), Err(2));
	}

	/// Tests `fold_right` on `Err`.
	#[test]
	fn fold_right_err() {
		assert_eq!(
			crate::classes::foldable::fold_right::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(
				|x: i32, acc| x + acc,
				0,
				Err(1)
			),
			0
		);
	}

	/// Tests `fold_left` on `Err`.
	#[test]
	fn fold_left_err() {
		assert_eq!(
			crate::classes::foldable::fold_left::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(
				|acc, x: i32| acc + x,
				0,
				Err(1)
			),
			0
		);
	}

	/// Tests `traverse` on `Err`.
	#[test]
	fn traverse_err() {
		assert_eq!(
			crate::classes::traversable::traverse::<ResultWithErrBrand<i32>, _, _, OptionBrand, _>(
				|x: i32| Some(x + 1),
				Err(1)
			),
			Some(Err(1))
		);
	}

	/// Tests `traverse` returning `Err`.
	#[test]
	fn traverse_returning_err() {
		assert_eq!(
			crate::classes::traversable::traverse::<ResultWithErrBrand<i32>, _, _, OptionBrand, _>(
				|_: i32| None::<i32>,
				Ok(1)
			),
			None
		);
	}

	// ParFoldable Tests for ResultWithErrBrand

	/// Tests `par_fold_map` on `Ok`.
	#[test]
	fn par_fold_map_ok() {
		let x: Result<i32, ()> = Ok(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, ResultWithErrBrand<()>, _, _>(f, x), "5".to_string());
	}

	/// Tests `par_fold_map` on `Err`.
	#[test]
	fn par_fold_map_err_val() {
		let x: Result<i32, i32> = Err(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, ResultWithErrBrand<i32>, _, _>(f, x), "".to_string());
	}

	/// Tests `par_fold_right` on `Ok`.
	#[test]
	fn par_fold_right_ok() {
		let x: Result<i32, ()> = Ok(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultWithErrBrand<()>, _, _>(f, 10, x), 15);
	}

	/// Tests `par_fold_right` on `Err`.
	#[test]
	fn par_fold_right_err_val() {
		let x: Result<i32, i32> = Err(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultWithErrBrand<i32>, _, _>(f, 10, x), 10);
	}

	// ParFoldable Tests for ResultWithOkBrand

	/// Tests `par_fold_map` on `Err` (which holds the value for ResultWithOkBrand).
	#[test]
	fn par_fold_map_err_ok_brand() {
		let x: Result<(), i32> = Err(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, ResultWithOkBrand<()>, _, _>(f.clone(), x),
			"5".to_string()
		);
	}

	/// Tests `par_fold_map` on `Ok` (which is empty for ResultWithOkBrand).
	#[test]
	fn par_fold_map_ok_ok_brand() {
		let x: Result<i32, i32> = Ok(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, ResultWithOkBrand<i32>, _, _>(f, x), "".to_string());
	}

	/// Tests `par_fold_right` on `Err` (which holds the value for ResultWithOkBrand).
	#[test]
	fn par_fold_right_err_ok_brand() {
		let x: Result<(), i32> = Err(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultWithOkBrand<()>, _, _>(f.clone(), 10, x), 15);
	}

	/// Tests `par_fold_right` on `Ok` (which is empty for ResultWithOkBrand).
	#[test]
	fn par_fold_right_ok_ok_brand() {
		let x: Result<i32, i32> = Ok(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultWithOkBrand<i32>, _, _>(f, 10, x), 10);
	}
}
