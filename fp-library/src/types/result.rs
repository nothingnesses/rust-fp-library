//! Functional programming trait implementations for the standard library [`Result`] type.
//!
//! Extends `Result` with dual functor/monad instances: [`ResultErrAppliedBrand`](crate::brands::ResultErrAppliedBrand) (standard Result monad) functors over the success value, while [`ResultOkAppliedBrand`](crate::brands::ResultOkAppliedBrand) functors over the error value.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ResultBrand,
				ResultErrAppliedBrand,
				ResultOkAppliedBrand,
			},
			classes::*,
			impl_kind,
			kinds::*,
		},
		core::ops::ControlFlow,
		fp_macros::*,
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

	impl_kind! {
		/// HKT branding for the `Result` type with lifetimes.
		///
		/// The type parameters for `Of` are ordered `E`, then `A` (Error, then Success).
		for ResultBrand {
			type Of<'a, A: 'a, B: 'a>: 'a = Result<B, A>;
		}
	}

	impl Bifunctor for ResultBrand {
		/// Maps functions over the values in the result.
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
			"The result to map over."
		)]
		///
		#[document_returns("A new result containing the mapped values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::bifunctor::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Result<i32, i32> = Ok(5);
		/// assert_eq!(bimap::<ResultBrand, _, _, _, _>(|e| e + 1, |s| s * 2, x), Ok(10));
		///
		/// let y: Result<i32, i32> = Err(5);
		/// assert_eq!(bimap::<ResultBrand, _, _, _, _>(|e| e + 1, |s| s * 2, y), Err(6));
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(A) -> B + 'a,
			g: impl Fn(C) -> D + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			match p {
				Ok(c) => Ok(g(c)),
				Err(a) => Err(f(a)),
			}
		}
	}

	impl Bifoldable for ResultBrand {
		/// Folds a result using two step functions, right-associatively.
		///
		/// Dispatches to `f` for `Err(a)` values and `g` for `Ok(b)` values.
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
			"The result to fold."
		)]
		///
		#[document_returns("`f(a, z)` for `Err(a)`, or `g(b, z)` for `Ok(b)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, ResultBrand, _, _, _>(
		/// 		|e: i32, acc| acc - e,
		/// 		|s: i32, acc| acc + s,
		/// 		10,
		/// 		Err(3),
		/// 	),
		/// 	7
		/// );
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, ResultBrand, _, _, _>(
		/// 		|e: i32, acc| acc - e,
		/// 		|s: i32, acc| acc + s,
		/// 		10,
		/// 		Ok(5),
		/// 	),
		/// 	15
		/// );
		/// ```
		fn bi_fold_right<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(A, C) -> C + 'a,
			g: impl Fn(B, C) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			match p {
				Err(a) => f(a, z),
				Ok(b) => g(b, z),
			}
		}

		/// Folds a result using two step functions, left-associatively.
		///
		/// Dispatches to `f` for `Err(a)` values and `g` for `Ok(b)` values.
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
			"The result to fold."
		)]
		///
		#[document_returns("`f(z, a)` for `Err(a)`, or `g(z, b)` for `Ok(b)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_left::<RcFnBrand, ResultBrand, _, _, _>(
		/// 		|acc, e: i32| acc - e,
		/// 		|acc, s: i32| acc + s,
		/// 		10,
		/// 		Err(3),
		/// 	),
		/// 	7
		/// );
		/// assert_eq!(
		/// 	bi_fold_left::<RcFnBrand, ResultBrand, _, _, _>(
		/// 		|acc, e: i32| acc - e,
		/// 		|acc, s: i32| acc + s,
		/// 		10,
		/// 		Ok(5),
		/// 	),
		/// 	15
		/// );
		/// ```
		fn bi_fold_left<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(C, A) -> C + 'a,
			g: impl Fn(C, B) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			match p {
				Err(a) => f(z, a),
				Ok(b) => g(z, b),
			}
		}

		/// Maps a result's value to a monoid using two functions and returns the result.
		///
		/// Dispatches to `f` for `Err(a)` and `g` for `Ok(b)`, returning the monoid value.
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
			"The result to fold."
		)]
		///
		#[document_returns("`f(a)` for `Err(a)`, or `g(b)` for `Ok(b)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_map::<RcFnBrand, ResultBrand, _, _, _>(
		/// 		|e: i32| e.to_string(),
		/// 		|s: i32| s.to_string(),
		/// 		Err::<i32, i32>(3),
		/// 	),
		/// 	"3".to_string()
		/// );
		/// assert_eq!(
		/// 	bi_fold_map::<RcFnBrand, ResultBrand, _, _, _>(
		/// 		|e: i32| e.to_string(),
		/// 		|s: i32| s.to_string(),
		/// 		Ok::<i32, i32>(5),
		/// 	),
		/// 	"5".to_string()
		/// );
		/// ```
		fn bi_fold_map<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, M>(
			f: impl Fn(A) -> M + 'a,
			g: impl Fn(B) -> M + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> M
		where
			M: Monoid + 'a, {
			match p {
				Err(a) => f(a),
				Ok(b) => g(b),
			}
		}
	}

	impl Bitraversable for ResultBrand {
		/// Traverses a result with two effectful functions.
		///
		/// Dispatches to `f` for `Err(a)` values and `g` for `Ok(b)` values,
		/// wrapping the result in the applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The error type (first position).",
			"The success type (second position).",
			"The output error type.",
			"The output success type.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function applied to the error value.",
			"The function applied to the success value.",
			"The result to traverse."
		)]
		///
		#[document_returns(
			"`f(a)` wrapped in context for `Err(a)`, or `g(b)` wrapped in context for `Ok(b)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_traverse::<ResultBrand, _, _, _, _, OptionBrand>(
		/// 		|e: i32| Some(e + 1),
		/// 		|s: i32| Some(s * 2),
		/// 		Err::<i32, i32>(3),
		/// 	),
		/// 	Some(Err(4))
		/// );
		/// assert_eq!(
		/// 	bi_traverse::<ResultBrand, _, _, _, _, OptionBrand>(
		/// 		|e: i32| Some(e + 1),
		/// 		|s: i32| Some(s * 2),
		/// 		Ok::<i32, i32>(5),
		/// 	),
		/// 	Some(Ok(10))
		/// );
		/// ```
		fn bi_traverse<
			'a,
			A: 'a + Clone,
			B: 'a + Clone,
			C: 'a + Clone,
			D: 'a + Clone,
			F: Applicative,
		>(
			f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		{
			match p {
				Err(a) => F::map(|c| Err(c), f(a)),
				Ok(b) => F::map(|d| Ok(d), g(b)),
			}
		}
	}

	// ResultErrAppliedBrand<E> (Functor over T)

	impl_kind! {
		#[document_type_parameters("The error type.")]
		impl<E: 'static> for ResultErrAppliedBrand<E> {
			type Of<'a, A: 'a>: 'a = Result<A, E>;
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Functor for ResultErrAppliedBrand<E> {
		/// Maps a function over the value in the result.
		///
		/// This method applies a function to the value inside the result if it is `Ok`, producing a new result with the transformed value. If the result is `Err`, it is returned unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value inside the result.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply.", "The result to map over.")]
		///
		#[document_returns(
			"A new result containing the result of applying the function, or the original error."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(map::<ResultErrAppliedBrand<()>, _, _, _>(|x: i32| x * 2, Ok(5)), Ok(10));
		/// assert_eq!(map::<ResultErrAppliedBrand<i32>, _, _, _>(|x: i32| x * 2, Err(1)), Err(1));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(func)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> Lift for ResultErrAppliedBrand<E> {
		/// Lifts a binary function into the result context.
		///
		/// This method lifts a binary function to operate on values within the result context.
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
			"The binary function to apply.",
			"The first result.",
			"The second result."
		)]
		///
		#[document_returns(
			"`Ok(f(a, b))` if both results are `Ok`, otherwise the first error encountered."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	lift2::<ResultErrAppliedBrand<()>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Ok(2)),
		/// 	Ok(3)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultErrAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Err(2)),
		/// 	Err(2)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultErrAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Ok(2)),
		/// 	Err(1)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultErrAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
		/// 	Err(1)
		/// );
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
			match (fa, fb) {
				(Ok(a), Ok(b)) => Ok(func(a, b)),
				(Err(e), _) => Err(e),
				(_, Err(e)) => Err(e),
			}
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Pointed for ResultErrAppliedBrand<E> {
		/// Wraps a value in a result.
		///
		/// This method wraps a value in the `Ok` variant of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("`Ok(a)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ResultErrAppliedBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(pure::<ResultErrAppliedBrand<()>, _>(5), Ok(5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Ok(a)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> ApplyFirst for ResultErrAppliedBrand<E> {}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> ApplySecond for ResultErrAppliedBrand<E> {}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> Semiapplicative for ResultErrAppliedBrand<E> {
		/// Applies a wrapped function to a wrapped value.
		///
		/// This method applies a function wrapped in a result to a value wrapped in a result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
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
		#[document_returns("`Ok(f(a))` if both are `Ok`, otherwise the first error encountered.")]
		#[document_examples]
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
		/// let f: Result<_, ()> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(apply::<RcFnBrand, ResultErrAppliedBrand<()>, _, _>(f, Ok(5)), Ok(10));
		/// let f: Result<_, i32> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(f, Err(1)), Err(1));
		///
		/// let f_err: Result<_, i32> = Err(1);
		/// assert_eq!(apply::<RcFnBrand, ResultErrAppliedBrand<i32>, i32, i32>(f_err, Ok(5)), Err(1));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match (ff, fa) {
				(Ok(f), Ok(a)) => Ok(f(a)),
				(Err(e), _) => Err(e),
				(_, Err(e)) => Err(e),
			}
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> Semimonad for ResultErrAppliedBrand<E> {
		/// Chains result computations.
		///
		/// This method chains two computations, where the second computation depends on the result of the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first result.",
			"The function to apply to the value inside the result."
		)]
		///
		#[document_returns(
			"The result of applying `f` to the value if `ma` is `Ok`, otherwise the original error."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ResultErrAppliedBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(bind::<ResultErrAppliedBrand<()>, _, _, _>(Ok(5), |x| Ok(x * 2)), Ok(10));
		/// assert_eq!(bind::<ResultErrAppliedBrand<i32>, _, _, _>(Ok(5), |_| Err::<i32, _>(1)), Err(1));
		/// assert_eq!(bind::<ResultErrAppliedBrand<i32>, _, _, _>(Err(1), |x: i32| Ok(x * 2)), Err(1));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.and_then(func)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Foldable for ResultErrAppliedBrand<E> {
		/// Folds the result from the right.
		///
		/// This method performs a right-associative fold of the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Ok(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ResultErrAppliedBrand<()>, _, _>(|x, acc| x + acc, 0, Ok(5)),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(|x: i32, acc| x + acc, 0, Err(1)),
		/// 	0
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
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
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(initial, a)` if `fa` is `Ok(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ResultErrAppliedBrand<()>, _, _>(|acc, x| acc + x, 0, Ok(5)),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(|acc, x: i32| acc + x, 0, Err(1)),
		/// 	0
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
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
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The result to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Ok(a)`, otherwise `M::empty()`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ResultErrAppliedBrand<()>, _, _>(|x: i32| x.to_string(), Ok(5)),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(|x: i32| x.to_string(), Err(1)),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			match fa {
				Ok(a) => func(a),
				Err(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> Traversable for ResultErrAppliedBrand<E> {
		/// Traverses the result with an applicative function.
		///
		/// This method maps the element of the result to a computation, evaluates it, and combines the result into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The function to apply.", "The result to traverse.")]
		///
		#[document_returns("The result wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		ResultErrAppliedBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<ResultErrAppliedBrand<()>, _, _, OptionBrand>(|x| Some(x * 2), Ok(5)),
		/// 	Some(Ok(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<ResultErrAppliedBrand<i32>, _, _, OptionBrand>(|x: i32| Some(x * 2), Err(1)),
		/// 	Some(Err(1))
		/// );
		/// assert_eq!(
		/// 	traverse::<ResultErrAppliedBrand<()>, _, _, OptionBrand>(|_| None::<i32>, Ok(5)),
		/// 	None
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
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
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The result containing the applicative value.")]
		///
		#[document_returns("The result wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		ResultErrAppliedBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(sequence::<ResultErrAppliedBrand<()>, _, OptionBrand>(Ok(Some(5))), Some(Ok(5)));
		/// assert_eq!(
		/// 	sequence::<ResultErrAppliedBrand<i32>, i32, OptionBrand>(Err::<Option<i32>, _>(1)),
		/// 	Some(Err::<i32, i32>(1))
		/// );
		/// assert_eq!(sequence::<ResultErrAppliedBrand<()>, _, OptionBrand>(Ok(None::<i32>)), None);
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			match ta {
				Ok(fa) => F::map(|a| Ok(a), fa),
				Err(e) => F::pure(Err(e)),
			}
		}
	}

	// ResultOkAppliedBrand<T> (Functor over E)

	impl_kind! {
		#[document_type_parameters("The success type.")]
		impl<T: 'static> for ResultOkAppliedBrand<T> {
			type Of<'a, A: 'a>: 'a = Result<T, A>;
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: 'static> Functor for ResultOkAppliedBrand<T> {
		/// Maps a function over the error value in the result.
		///
		/// This method applies a function to the error value inside the result if it is `Err`, producing a new result with the transformed error. If the result is `Ok`, it is returned unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the error value inside the result.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply to the error.", "The result to map over.")]
		///
		#[document_returns(
			"A new result containing the mapped error, or the original success value."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(map::<ResultOkAppliedBrand<i32>, _, _, _>(|x: i32| x * 2, Err(5)), Err(10));
		/// assert_eq!(map::<ResultOkAppliedBrand<i32>, _, _, _>(|x: i32| x * 2, Ok(1)), Ok(1));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Ok(t) => Ok(t),
				Err(e) => Err(func(e)),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> Lift for ResultOkAppliedBrand<T> {
		/// Lifts a binary function into the result context (over error).
		///
		/// This method lifts a binary function to operate on error values within the result context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first error value.",
			"The type of the second error value.",
			"The type of the result error value."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the errors.",
			"The first result.",
			"The second result."
		)]
		///
		#[document_returns(
			"`Err(f(a, b))` if both results are `Err`, otherwise the first success encountered."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	lift2::<ResultOkAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
		/// 	Err(3)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultOkAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Ok(2)),
		/// 	Ok(2)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultOkAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Err(2)),
		/// 	Ok(1)
		/// );
		/// assert_eq!(
		/// 	lift2::<ResultOkAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Ok(2)),
		/// 	Ok(1)
		/// );
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
			match (fa, fb) {
				(Err(a), Err(b)) => Err(func(a, b)),
				(Ok(t), _) => Ok(t),
				(_, Ok(t)) => Ok(t),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: 'static> Pointed for ResultOkAppliedBrand<T> {
		/// Wraps a value in a result (as error).
		///
		/// This method wraps a value in the `Err` variant of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("`Err(a)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ResultOkAppliedBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(pure::<ResultOkAppliedBrand<()>, _>(5), Err(5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Err(a)
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> ApplyFirst for ResultOkAppliedBrand<T> {}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> ApplySecond for ResultOkAppliedBrand<T> {}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> Semiapplicative for ResultOkAppliedBrand<T> {
		/// Applies a wrapped function to a wrapped value (over error).
		///
		/// This method applies a function wrapped in a result (as error) to a value wrapped in a result (as error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
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
		#[document_returns(
			"`Err(f(a))` if both are `Err`, otherwise the first success encountered."
		)]
		#[document_examples]
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
		/// let f: Result<(), _> = Err(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(apply::<RcFnBrand, ResultOkAppliedBrand<()>, _, _>(f, Err(5)), Err(10));
		/// let f: Result<i32, _> = Err(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(apply::<RcFnBrand, ResultOkAppliedBrand<i32>, _, _>(f, Ok(1)), Ok(1));
		///
		/// let f_ok: Result<i32, _> = Ok(1);
		/// assert_eq!(apply::<RcFnBrand, ResultOkAppliedBrand<i32>, i32, i32>(f_ok, Err(5)), Ok(1));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match (ff, fa) {
				(Err(f), Err(a)) => Err(f(a)),
				(Ok(t), _) => Ok(t),
				(_, Ok(t)) => Ok(t),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> Semimonad for ResultOkAppliedBrand<T> {
		/// Chains result computations (over error).
		///
		/// This method chains two computations, where the second computation depends on the result of the first (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters("The first result.", "The function to apply to the error value.")]
		///
		#[document_returns(
			"The result of applying `f` to the error if `ma` is `Err`, otherwise the original success."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ResultOkAppliedBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(bind::<ResultOkAppliedBrand<()>, _, _, _>(Err(5), |x| Err(x * 2)), Err(10));
		/// assert_eq!(bind::<ResultOkAppliedBrand<i32>, _, _, _>(Err(5), |_| Ok::<_, i32>(1)), Ok(1));
		/// assert_eq!(bind::<ResultOkAppliedBrand<i32>, _, _, _>(Ok(1), |x: i32| Err(x * 2)), Ok(1));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match ma {
				Ok(t) => Ok(t),
				Err(e) => func(e),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: 'static> Foldable for ResultOkAppliedBrand<T> {
		/// Folds the result from the right (over error).
		///
		/// This method performs a right-associative fold of the result (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Err(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ResultOkAppliedBrand<i32>, _, _>(|x: i32, acc| x + acc, 0, Err(1)),
		/// 	1
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, ResultOkAppliedBrand<()>, _, _>(|x: i32, acc| x + acc, 0, Ok(())),
		/// 	0
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
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
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(initial, a)` if `fa` is `Err(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ResultOkAppliedBrand<()>, _, _>(|acc, x: i32| acc + x, 0, Err(5)),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, ResultOkAppliedBrand<i32>, _, _>(|acc, x: i32| acc + x, 0, Ok(1)),
		/// 	0
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
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
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The result to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Err(a)`, otherwise `M::empty()`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ResultOkAppliedBrand<()>, _, _>(|x: i32| x.to_string(), Err(5)),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, ResultOkAppliedBrand<i32>, _, _>(|x: i32| x.to_string(), Ok(1)),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			match fa {
				Err(e) => func(e),
				Ok(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> Traversable for ResultOkAppliedBrand<T> {
		/// Traverses the result with an applicative function (over error).
		///
		/// This method maps the element of the result to a computation, evaluates it, and combines the result into an applicative context (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The function to apply.", "The result to traverse.")]
		///
		#[document_returns("The result wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		ResultOkAppliedBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<ResultOkAppliedBrand<()>, _, _, OptionBrand>(|x| Some(x * 2), Err(5)),
		/// 	Some(Err(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<ResultOkAppliedBrand<i32>, _, _, OptionBrand>(|x: i32| Some(x * 2), Ok(1)),
		/// 	Some(Ok(1))
		/// );
		/// assert_eq!(
		/// 	traverse::<ResultOkAppliedBrand<()>, _, _, OptionBrand>(|_| None::<i32>, Err(5)),
		/// 	None
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
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
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The result containing the applicative value.")]
		///
		#[document_returns("The result wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		ResultOkAppliedBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(sequence::<ResultOkAppliedBrand<()>, _, OptionBrand>(Err(Some(5))), Some(Err(5)));
		/// assert_eq!(
		/// 	sequence::<ResultOkAppliedBrand<i32>, i32, OptionBrand>(Ok::<_, Option<i32>>(1)),
		/// 	Some(Ok::<i32, i32>(1))
		/// );
		/// assert_eq!(sequence::<ResultOkAppliedBrand<()>, _, OptionBrand>(Err(None::<i32>)), None);
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			match ta {
				Err(fe) => F::map(|e| Err(e), fe),
				Ok(t) => F::pure(Ok(t)),
			}
		}
	}

	/// [`MonadRec`] implementation for [`ResultOkAppliedBrand`].
	#[document_type_parameters("The success type.")]
	impl<T: Clone + 'static> MonadRec for ResultOkAppliedBrand<T> {
		/// Performs tail-recursive monadic computation over [`Result`] (error channel).
		///
		/// Iteratively applies the step function. If the function returns [`Ok`],
		/// the computation short-circuits with that success value. If it returns
		/// `Err(ControlFlow::Continue(a))`, the loop continues with the new state. If it returns
		/// `Err(ControlFlow::Break(b))`, the computation completes with `Err(b)`.
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
		#[document_returns(
			"The result of the computation, or a success if the step function returned `Ok`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<ResultOkAppliedBrand<&str>, _, _>(
		/// 	|n| {
		/// 		if n < 10 { Err(ControlFlow::Continue(n + 1)) } else { Err(ControlFlow::Break(n)) }
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, Err(10));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut current = initial;
			loop {
				match func(current) {
					Ok(t) => return Ok(t),
					Err(ControlFlow::Continue(next)) => current = next,
					Err(ControlFlow::Break(b)) => return Err(b),
				}
			}
		}
	}

	/// [`MonadRec`] implementation for [`ResultErrAppliedBrand`].
	#[document_type_parameters("The error type.")]
	impl<E: Clone + 'static> MonadRec for ResultErrAppliedBrand<E> {
		/// Performs tail-recursive monadic computation over [`Result`].
		///
		/// Iteratively applies the step function. If the function returns [`Err`],
		/// the computation short-circuits with that error. If it returns
		/// `Ok(ControlFlow::Continue(a))`, the loop continues with the new state. If it returns
		/// `Ok(ControlFlow::Break(b))`, the computation completes with `Ok(b)`.
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
		#[document_returns(
			"The result of the computation, or an error if the step function returned `Err`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<ResultErrAppliedBrand<&str>, _, _>(
		/// 	|n| {
		/// 		if n < 10 { Ok(ControlFlow::Continue(n + 1)) } else { Ok(ControlFlow::Break(n)) }
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, Ok(10));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut current = initial;
			loop {
				match func(current) {
					Err(e) => return Err(e),
					Ok(ControlFlow::Continue(next)) => current = next,
					Ok(ControlFlow::Break(b)) => return Ok(b),
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {

	use {
		crate::{
			brands::*,
			classes::*,
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	// Bifunctor Tests

	/// Tests `bimap` on `Ok` and `Err`.
	#[test]
	fn test_bimap() {
		let x: Result<i32, i32> = Ok(5);
		assert_eq!(bimap::<ResultBrand, _, _, _, _>(|e| e + 1, |s| s * 2, x), Ok(10));

		let y: Result<i32, i32> = Err(5);
		assert_eq!(bimap::<ResultBrand, _, _, _, _>(|e| e + 1, |s| s * 2, y), Err(6));
	}

	// Bifunctor Laws

	/// Tests the identity law for Bifunctor.
	#[quickcheck]
	fn bifunctor_identity(x: Result<i32, i32>) -> bool {
		bimap::<ResultBrand, _, _, _, _>(identity, identity, x) == x
	}

	/// Tests the composition law for Bifunctor.
	#[quickcheck]
	fn bifunctor_composition(x: Result<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		let h = |x: i32| x.wrapping_sub(1);
		let i = |x: i32| if x == 0 { 0 } else { x.wrapping_div(2) };

		bimap::<ResultBrand, _, _, _, _>(compose(f, g), compose(h, i), x)
			== bimap::<ResultBrand, _, _, _, _>(f, h, bimap::<ResultBrand, _, _, _, _>(g, i, x))
	}

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: Result<i32, i32>) -> bool {
		map::<ResultErrAppliedBrand<i32>, _, _, _>(identity, x) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Result<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<ResultErrAppliedBrand<i32>, _, _, _>(compose(f, g), x)
			== map::<ResultErrAppliedBrand<i32>, _, _, _>(
				f,
				map::<ResultErrAppliedBrand<i32>, _, _, _>(g, x),
			)
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Result<i32, i32>) -> bool {
		apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(
			pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as LiftFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(
			pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as LiftFn>::new(f)),
			pure::<ResultErrAppliedBrand<i32>, _>(x),
		) == pure::<ResultErrAppliedBrand<i32>, _>(f(x))
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
			pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as LiftFn>::new(v_fn))
		} else {
			Err(100)
		};
		let u = if u_is_ok {
			pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as LiftFn>::new(u_fn))
		} else {
			Err(200)
		};

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(v.clone(), w);
		let rhs = apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let uv = match (u, v) {
			(Ok(uf), Ok(vf)) => {
				let composed = move |x| uf(vf(x));
				Ok(<RcFnBrand as LiftFn>::new(composed))
			}
			(Err(e), _) => Err(e),
			(_, Err(e)) => Err(e),
		};

		let lhs = apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as LiftFn>::new(f));

		let lhs = apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(
			u.clone(),
			pure::<ResultErrAppliedBrand<i32>, _>(y),
		);

		let rhs_fn = <RcFnBrand as LiftFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(
			pure::<ResultErrAppliedBrand<i32>, _>(rhs_fn),
			u,
		);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| -> Result<i32, i32> { Err(x.wrapping_mul(2)) };
		bind::<ResultErrAppliedBrand<i32>, _, _, _>(pure::<ResultErrAppliedBrand<i32>, _>(a), f)
			== f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Result<i32, i32>) -> bool {
		bind::<ResultErrAppliedBrand<i32>, _, _, _>(m, pure::<ResultErrAppliedBrand<i32>, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Result<i32, i32>) -> bool {
		let f = |x: i32| -> Result<i32, i32> { Err(x.wrapping_mul(2)) };
		let g = |x: i32| -> Result<i32, i32> { Err(x.wrapping_add(1)) };
		bind::<ResultErrAppliedBrand<i32>, _, _, _>(
			bind::<ResultErrAppliedBrand<i32>, _, _, _>(m, f),
			g,
		) == bind::<ResultErrAppliedBrand<i32>, _, _, _>(m, |x| {
			bind::<ResultErrAppliedBrand<i32>, _, _, _>(f(x), g)
		})
	}

	// Edge Cases

	/// Tests `map` on `Err`.
	#[test]
	fn map_err() {
		assert_eq!(
			map::<ResultErrAppliedBrand<i32>, _, _, _>(|x: i32| x + 1, Err::<i32, i32>(1)),
			Err(1)
		);
	}

	/// Tests `bind` on `Err`.
	#[test]
	fn bind_err() {
		assert_eq!(
			bind::<ResultErrAppliedBrand<i32>, _, _, _>(Err::<i32, i32>(1), |x: i32| Ok(x + 1)),
			Err(1)
		);
	}

	/// Tests `bind` returning `Err`.
	#[test]
	fn bind_returning_err() {
		assert_eq!(
			bind::<ResultErrAppliedBrand<i32>, _, _, _>(Ok(1), |_| Err::<i32, i32>(2)),
			Err(2)
		);
	}

	/// Tests `fold_right` on `Err`.
	#[test]
	fn fold_right_err() {
		assert_eq!(
			crate::classes::foldable::fold_right::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(
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
			crate::classes::foldable::fold_left::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(
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
			crate::classes::traversable::traverse::<ResultErrAppliedBrand<i32>, _, _, OptionBrand>(
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
			crate::classes::traversable::traverse::<ResultErrAppliedBrand<i32>, _, _, OptionBrand>(
				|_: i32| None::<i32>,
				Ok(1)
			),
			None
		);
	}

	// MonadRec tests

	/// Tests the MonadRec identity law: `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`.
	#[quickcheck]
	fn monad_rec_identity(x: i32) -> bool {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		tail_rec_m::<ResultErrAppliedBrand<()>, _, _>(|a| Ok(ControlFlow::Break(a)), x) == Ok(x)
	}

	/// Tests a recursive computation that sums a range via `tail_rec_m`.
	#[test]
	fn monad_rec_sum_range() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result = tail_rec_m::<ResultErrAppliedBrand<&str>, _, _>(
			|(n, acc)| {
				if n == 0 {
					Ok(ControlFlow::Break(acc))
				} else {
					Ok(ControlFlow::Continue((n - 1, acc + n)))
				}
			},
			(100i64, 0i64),
		);
		assert_eq!(result, Ok(5050));
	}

	/// Tests that `tail_rec_m` short-circuits on `Err`.
	#[test]
	fn monad_rec_short_circuit() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result: Result<i32, &str> = tail_rec_m::<ResultErrAppliedBrand<&str>, _, _>(
			|n| {
				if n == 5 { Err("stopped") } else { Ok(ControlFlow::Continue(n + 1)) }
			},
			0,
		);
		assert_eq!(result, Err("stopped"));
	}

	/// Tests stack safety: `tail_rec_m` handles large iteration counts.
	#[test]
	fn monad_rec_stack_safety() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<ResultErrAppliedBrand<()>, _, _>(
			|acc| {
				if acc < iterations {
					Ok(ControlFlow::Continue(acc + 1))
				} else {
					Ok(ControlFlow::Break(acc))
				}
			},
			0i64,
		);
		assert_eq!(result, Ok(iterations));
	}

	// MonadRec tests for ResultOkAppliedBrand

	/// Tests the MonadRec identity law for `ResultOkAppliedBrand`:
	/// `tail_rec_m(|a| Err(Done(a)), x) == Err(x)`.
	#[quickcheck]
	fn monad_rec_ok_applied_identity(x: i32) -> bool {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		tail_rec_m::<ResultOkAppliedBrand<()>, _, _>(|a| Err(ControlFlow::Break(a)), x) == Err(x)
	}

	/// Tests a recursive computation that sums a range via `tail_rec_m` on the error channel.
	#[test]
	fn monad_rec_ok_applied_sum_range() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result = tail_rec_m::<ResultOkAppliedBrand<&str>, _, _>(
			|(n, acc)| {
				if n == 0 {
					Err(ControlFlow::Break(acc))
				} else {
					Err(ControlFlow::Continue((n - 1, acc + n)))
				}
			},
			(100i64, 0i64),
		);
		assert_eq!(result, Err(5050));
	}

	/// Tests that `tail_rec_m` on `ResultOkAppliedBrand` short-circuits on `Ok`.
	#[test]
	fn monad_rec_ok_applied_short_circuit() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result: Result<&str, i32> = tail_rec_m::<ResultOkAppliedBrand<&str>, _, _>(
			|n| {
				if n == 5 { Ok("stopped") } else { Err(ControlFlow::Continue(n + 1)) }
			},
			0,
		);
		assert_eq!(result, Ok("stopped"));
	}

	/// Tests stack safety: `tail_rec_m` on `ResultOkAppliedBrand` handles large iteration counts.
	#[test]
	fn monad_rec_ok_applied_stack_safety() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<ResultOkAppliedBrand<()>, _, _>(
			|acc| {
				if acc < iterations {
					Err(ControlFlow::Continue(acc + 1))
				} else {
					Err(ControlFlow::Break(acc))
				}
			},
			0i64,
		);
		assert_eq!(result, Err(iterations));
	}
}
