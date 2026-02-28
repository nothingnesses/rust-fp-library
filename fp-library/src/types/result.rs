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
			classes::{
				Applicative,
				ApplyFirst,
				ApplySecond,
				Bifunctor,
				CloneableFn,
				Foldable,
				Functor,
				Lift,
				Monoid,
				ParFoldable,
				Pointed,
				Semiapplicative,
				Semimonad,
				SendCloneableFn,
				Traversable,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::{
			document_examples,
			document_parameters,
			document_returns,
			document_type_parameters,
		},
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
		#[document_returns("A new result containing the mapped values.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::bifunctor::*,
	functions::*,
};

let x: Result<i32, i32> = Ok(5);
assert_eq!(bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x), Ok(10));

let y: Result<i32, i32> = Err(5);
assert_eq!(bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, y), Err(6));"#
		)]
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, F, G>(
			f: F,
			g: G,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
		where
			F: Fn(A) -> B + 'a,
			G: Fn(C) -> D + 'a, {
			match p {
				Ok(c) => Ok(g(c)),
				Err(a) => Err(f(a)),
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
			"The type of the result of applying the function.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply.", "The result to map over.")]
		///
		#[document_returns(
			"A new result containing the result of applying the function, or the original error."
		)]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(map::<ResultErrAppliedBrand<()>, _, _, _>(|x: i32| x * 2, Ok(5)), Ok(10));
assert_eq!(map::<ResultErrAppliedBrand<i32>, _, _, _>(|x: i32| x * 2, Err(1)), Err(1));"#
		)]
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
		#[document_returns(
			"`Ok(f(a, b))` if both results are `Ok`, otherwise the first error encountered."
		)]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(
	lift2::<ResultErrAppliedBrand<()>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Ok(2)),
	Ok(3)
);
assert_eq!(
	lift2::<ResultErrAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Err(2)),
	Err(2)
);
assert_eq!(
	lift2::<ResultErrAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Ok(2)),
	Err(1)
);
assert_eq!(
	lift2::<ResultErrAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
	Err(1)
);"#
		)]
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
		#[document_examples(
			r#"use fp_library::{
	brands::ResultErrAppliedBrand,
	functions::*,
};

assert_eq!(pure::<ResultErrAppliedBrand<()>, _>(5), Ok(5));"#
		)]
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
		#[document_examples(
			r#"use fp_library::{
	Apply,
	Kind,
	brands::*,
	classes::*,
	functions::*,
};

let f: Result<_, ()> = Ok(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
assert_eq!(apply::<RcFnBrand, ResultErrAppliedBrand<()>, _, _>(f, Ok(5)), Ok(10));
let f: Result<_, i32> = Ok(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
assert_eq!(apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(f, Err(1)), Err(1));

let f_err: Result<_, i32> = Err(1);
assert_eq!(apply::<RcFnBrand, ResultErrAppliedBrand<i32>, i32, i32>(f_err, Ok(5)), Err(1));"#
		)]
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
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
			"The type of the result of the second computation.",
			"The type of the function to apply."
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
		#[document_examples(
			r#"use fp_library::{
	brands::ResultErrAppliedBrand,
	functions::*,
};

assert_eq!(bind::<ResultErrAppliedBrand<()>, _, _, _>(Ok(5), |x| Ok(x * 2)), Ok(10));
assert_eq!(bind::<ResultErrAppliedBrand<i32>, _, _, _>(Ok(5), |_| Err::<i32, _>(1)), Err(1));
assert_eq!(bind::<ResultErrAppliedBrand<i32>, _, _, _>(Err(1), |x: i32| Ok(x * 2)), Err(1));"#
		)]
		fn bind<'a, A: 'a, B: 'a, Func>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a, {
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
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Ok(a)`, otherwise `initial`.")]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(
	fold_right::<RcFnBrand, ResultErrAppliedBrand<()>, _, _, _>(|x, acc| x + acc, 0, Ok(5)),
	5
);
assert_eq!(
	fold_right::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _, _>(
		|x: i32, acc| x + acc,
		0,
		Err(1)
	),
	0
);"#
		)]
		fn fold_right<'a, FnBrand, A: 'a, B: 'a, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			F: Fn(A, B) -> B + 'a,
			FnBrand: CloneableFn + 'a, {
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
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(initial, a)` if `fa` is `Ok(a)`, otherwise `initial`.")]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(
	fold_left::<RcFnBrand, ResultErrAppliedBrand<()>, _, _, _>(|acc, x| acc + x, 0, Ok(5)),
	5
);
assert_eq!(
	fold_left::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _, _>(
		|acc, x: i32| acc + x,
		0,
		Err(1)
	),
	0
);"#
		)]
		fn fold_left<'a, FnBrand, A: 'a, B: 'a, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			F: Fn(B, A) -> B + 'a,
			FnBrand: CloneableFn + 'a, {
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
			"The type of the monoid.",
			"The type of the mapping function."
		)]
		///
		#[document_parameters("The mapping function.", "The result to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Ok(a)`, otherwise `M::empty()`.")]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(
	fold_map::<RcFnBrand, ResultErrAppliedBrand<()>, _, _, _>(|x: i32| x.to_string(), Ok(5)),
	"5".to_string()
);
assert_eq!(
	fold_map::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _, _>(|x: i32| x.to_string(), Err(1)),
	"".to_string()
);"#
		)]
		fn fold_map<'a, FnBrand, A: 'a, M, F>(
			func: F,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			F: Fn(A) -> M + 'a,
			FnBrand: CloneableFn + 'a, {
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
			"The applicative context.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply.", "The result to traverse.")]
		///
		#[document_returns("The result wrapped in the applicative context.")]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::{
		OptionBrand,
		ResultErrAppliedBrand,
	},
	functions::*,
};

assert_eq!(
	traverse::<ResultErrAppliedBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Ok(5)),
	Some(Ok(10))
);
assert_eq!(
	traverse::<ResultErrAppliedBrand<i32>, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Err(1)),
	Some(Err(1))
);
assert_eq!(
	traverse::<ResultErrAppliedBrand<()>, _, _, OptionBrand, _>(|_| None::<i32>, Ok(5)),
	None
);"#
		)]
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
			func: Func,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
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
		#[document_examples(
			r#"use fp_library::{
	brands::{
		OptionBrand,
		ResultErrAppliedBrand,
	},
	functions::*,
};

assert_eq!(sequence::<ResultErrAppliedBrand<()>, _, OptionBrand>(Ok(Some(5))), Some(Ok(5)));
assert_eq!(
	sequence::<ResultErrAppliedBrand<i32>, i32, OptionBrand>(Err::<Option<i32>, _>(1)),
	Some(Err::<i32, i32>(1))
);
assert_eq!(sequence::<ResultErrAppliedBrand<()>, _, OptionBrand>(Ok(None::<i32>)), None);"#
		)]
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
			"The type of the result of applying the function.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply to the error.", "The result to map over.")]
		///
		#[document_returns(
			"A new result containing the mapped error, or the original success value."
		)]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(map::<ResultOkAppliedBrand<i32>, _, _, _>(|x: i32| x * 2, Err(5)), Err(10));
assert_eq!(map::<ResultOkAppliedBrand<i32>, _, _, _>(|x: i32| x * 2, Ok(1)), Ok(1));"#
		)]
		fn map<'a, A: 'a, B: 'a, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Func: Fn(A) -> B + 'a, {
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
		#[document_returns(
			"`Err(f(a, b))` if both results are `Err`, otherwise the first success encountered."
		)]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(
	lift2::<ResultOkAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
	Err(3)
);
assert_eq!(
	lift2::<ResultOkAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Ok(2)),
	Ok(2)
);
assert_eq!(
	lift2::<ResultOkAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Err(2)),
	Ok(1)
);
assert_eq!(
	lift2::<ResultOkAppliedBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Ok(2)),
	Ok(1)
);"#
		)]
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
		#[document_examples(
			r#"use fp_library::{
	brands::ResultOkAppliedBrand,
	functions::*,
};

assert_eq!(pure::<ResultOkAppliedBrand<()>, _>(5), Err(5));"#
		)]
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
		#[document_examples(
			r#"use fp_library::{
	Apply,
	Kind,
	brands::*,
	classes::*,
	functions::*,
};

let f: Result<(), _> = Err(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
assert_eq!(apply::<RcFnBrand, ResultOkAppliedBrand<()>, _, _>(f, Err(5)), Err(10));
let f: Result<i32, _> = Err(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
assert_eq!(apply::<RcFnBrand, ResultOkAppliedBrand<i32>, _, _>(f, Ok(1)), Ok(1));

let f_ok: Result<i32, _> = Ok(1);
assert_eq!(apply::<RcFnBrand, ResultOkAppliedBrand<i32>, i32, i32>(f_ok, Err(5)), Ok(1));"#
		)]
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
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
			"The type of the result of the second computation.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The first result.", "The function to apply to the error value.")]
		///
		#[document_returns(
			"The result of applying `f` to the error if `ma` is `Err`, otherwise the original success."
		)]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::ResultOkAppliedBrand,
	functions::*,
};

assert_eq!(bind::<ResultOkAppliedBrand<()>, _, _, _>(Err(5), |x| Err(x * 2)), Err(10));
assert_eq!(bind::<ResultOkAppliedBrand<i32>, _, _, _>(Err(5), |_| Ok::<_, i32>(1)), Ok(1));
assert_eq!(bind::<ResultOkAppliedBrand<i32>, _, _, _>(Ok(1), |x: i32| Err(x * 2)), Ok(1));"#
		)]
		fn bind<'a, A: 'a, B: 'a, Func>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a, {
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
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Err(a)`, otherwise `initial`.")]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(
	fold_right::<RcFnBrand, ResultOkAppliedBrand<i32>, _, _, _>(
		|x: i32, acc| x + acc,
		0,
		Err(1)
	),
	1
);
assert_eq!(
	fold_right::<RcFnBrand, ResultOkAppliedBrand<()>, _, _, _>(
		|x: i32, acc| x + acc,
		0,
		Ok(())
	),
	0
);"#
		)]
		fn fold_right<'a, FnBrand, A: 'a, B: 'a, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			F: Fn(A, B) -> B + 'a,
			FnBrand: CloneableFn + 'a, {
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
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(initial, a)` if `fa` is `Err(a)`, otherwise `initial`.")]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(
	fold_left::<RcFnBrand, ResultOkAppliedBrand<()>, _, _, _>(|acc, x: i32| acc + x, 0, Err(5)),
	5
);
assert_eq!(
	fold_left::<RcFnBrand, ResultOkAppliedBrand<i32>, _, _, _>(|acc, x: i32| acc + x, 0, Ok(1)),
	0
);"#
		)]
		fn fold_left<'a, FnBrand, A: 'a, B: 'a, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			F: Fn(B, A) -> B + 'a,
			FnBrand: CloneableFn + 'a, {
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
			"The type of the monoid.",
			"The type of the mapping function."
		)]
		///
		#[document_parameters("The mapping function.", "The result to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Err(a)`, otherwise `M::empty()`.")]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	functions::*,
};

assert_eq!(
	fold_map::<RcFnBrand, ResultOkAppliedBrand<()>, _, _, _>(|x: i32| x.to_string(), Err(5)),
	"5".to_string()
);
assert_eq!(
	fold_map::<RcFnBrand, ResultOkAppliedBrand<i32>, _, _, _>(|x: i32| x.to_string(), Ok(1)),
	"".to_string()
);"#
		)]
		fn fold_map<'a, FnBrand, A: 'a, M, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			Func: Fn(A) -> M + 'a,
			FnBrand: CloneableFn + 'a, {
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
			"The applicative context.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply.", "The result to traverse.")]
		///
		#[document_returns("The result wrapped in the applicative context.")]
		///
		#[document_examples(
			r#"use fp_library::{
	brands::{
		OptionBrand,
		ResultOkAppliedBrand,
	},
	functions::*,
};

assert_eq!(
	traverse::<ResultOkAppliedBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Err(5)),
	Some(Err(10))
);
assert_eq!(
	traverse::<ResultOkAppliedBrand<i32>, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Ok(1)),
	Some(Ok(1))
);
assert_eq!(
	traverse::<ResultOkAppliedBrand<()>, _, _, OptionBrand, _>(|_| None::<i32>, Err(5)),
	None
);"#
		)]
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
			func: Func,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
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
		#[document_examples(
			r#"use fp_library::{
	brands::{
		OptionBrand,
		ResultOkAppliedBrand,
	},
	functions::*,
};

assert_eq!(sequence::<ResultOkAppliedBrand<()>, _, OptionBrand>(Err(Some(5))), Some(Err(5)));
assert_eq!(
	sequence::<ResultOkAppliedBrand<i32>, i32, OptionBrand>(Ok::<_, Option<i32>>(1)),
	Some(Ok::<i32, i32>(1))
);
assert_eq!(sequence::<ResultOkAppliedBrand<()>, _, OptionBrand>(Err(None::<i32>)), None);"#
		)]
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

	#[document_type_parameters("The error type.")]
	impl<E: 'static> ParFoldable for ResultErrAppliedBrand<E> {
		/// Maps the value to a monoid and returns it, or returns empty, in parallel.
		///
		/// This method maps the element of the result to a monoid and then returns it. The mapping operation may be executed in parallel.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
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
		#[document_returns("The combined monoid value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::*,
	functions::*,
	types::*,
};

let x: Result<i32, ()> = Ok(5);
let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
assert_eq!(
	par_fold_map::<ArcFnBrand, ResultErrAppliedBrand<()>, _, _>(f.clone(), x),
	"5".to_string()
);

let x_err: Result<i32, i32> = Err(1);
assert_eq!(
	par_fold_map::<ArcFnBrand, ResultErrAppliedBrand<i32>, _, _>(f, x_err),
	"".to_string()
);"#
		)]
		fn par_fold_map<'a, FnBrand, A, M>(
			func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: 'a + SendCloneableFn,
			A: 'a + Clone + Send + Sync,
			M: Monoid + Send + Sync + 'a, {
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
			"The lifetime of the values.",
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
		#[document_returns("The final accumulator value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::*,
	functions::*,
};

let x: Result<i32, ()> = Ok(5);
let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
assert_eq!(par_fold_right::<ArcFnBrand, ResultErrAppliedBrand<()>, _, _>(f.clone(), 10, x), 15);

let x_err: Result<i32, i32> = Err(1);
assert_eq!(par_fold_right::<ArcFnBrand, ResultErrAppliedBrand<i32>, _, _>(f, 10, x_err), 10);"#
		)]
		fn par_fold_right<'a, FnBrand, A, B>(
			func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: 'a + SendCloneableFn,
			A: 'a + Clone + Send + Sync,
			B: Send + Sync + 'a, {
			match fa {
				Ok(a) => func((a, initial)),
				Err(_) => initial,
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<T: 'static> ParFoldable for ResultOkAppliedBrand<T> {
		/// Maps the value to a monoid and returns it, or returns empty, in parallel (over error).
		///
		/// This method maps the element of the result to a monoid and then returns it (over error). The mapping operation may be executed in parallel.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
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
		#[document_returns("The combined monoid value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::*,
	functions::*,
	types::*,
};

let x: Result<(), i32> = Err(5);
let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
assert_eq!(
	par_fold_map::<ArcFnBrand, ResultOkAppliedBrand<()>, _, _>(f.clone(), x),
	"5".to_string()
);

let x_ok: Result<i32, i32> = Ok(1);
assert_eq!(
	par_fold_map::<ArcFnBrand, ResultOkAppliedBrand<i32>, _, _>(f, x_ok),
	"".to_string()
);"#
		)]
		fn par_fold_map<'a, FnBrand, A, M>(
			func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: 'a + SendCloneableFn,
			A: 'a + Clone + Send + Sync,
			M: Monoid + Send + Sync + 'a, {
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
			"The lifetime of the values.",
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
		#[document_returns("The final accumulator value.")]
		#[document_examples(
			r#"use fp_library::{
	brands::*,
	classes::*,
	functions::*,
};

let x: Result<(), i32> = Err(5);
let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
assert_eq!(par_fold_right::<ArcFnBrand, ResultOkAppliedBrand<()>, _, _>(f.clone(), 10, x), 15);

let x_ok: Result<i32, i32> = Ok(1);
assert_eq!(par_fold_right::<ArcFnBrand, ResultOkAppliedBrand<i32>, _, _>(f, 10, x_ok), 10);"#
		)]
		fn par_fold_right<'a, FnBrand, A, B>(
			func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: 'a + SendCloneableFn,
			A: 'a + Clone + Send + Sync,
			B: Send + Sync + 'a, {
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
			classes::{
				CloneableFn,
				bifunctor::*,
			},
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
			pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(
			pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(f)),
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
			pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(v_fn))
		} else {
			Err(100)
		};
		let u = if u_is_ok {
			pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(u_fn))
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
				Ok(<RcFnBrand as CloneableFn>::new(composed))
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
		let u = pure::<ResultErrAppliedBrand<i32>, _>(<RcFnBrand as CloneableFn>::new(f));

		let lhs = apply::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _>(
			u.clone(),
			pure::<ResultErrAppliedBrand<i32>, _>(y),
		);

		let rhs_fn =
			<RcFnBrand as CloneableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
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
			crate::classes::foldable::fold_right::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _, _>(
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
			crate::classes::foldable::fold_left::<RcFnBrand, ResultErrAppliedBrand<i32>, _, _, _>(
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
			crate::classes::traversable::traverse::<ResultErrAppliedBrand<i32>, _, _, OptionBrand, _>(
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
			crate::classes::traversable::traverse::<ResultErrAppliedBrand<i32>, _, _, OptionBrand, _>(
				|_: i32| None::<i32>,
				Ok(1)
			),
			None
		);
	}

	// ParFoldable Tests for ResultErrAppliedBrand

	/// Tests `par_fold_map` on `Ok`.
	#[test]
	fn par_fold_map_ok() {
		let x: Result<i32, ()> = Ok(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, ResultErrAppliedBrand<()>, _, _>(f, x),
			"5".to_string()
		);
	}

	/// Tests `par_fold_map` on `Err`.
	#[test]
	fn par_fold_map_err_val() {
		let x: Result<i32, i32> = Err(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, ResultErrAppliedBrand<i32>, _, _>(f, x),
			"".to_string()
		);
	}

	/// Tests `par_fold_right` on `Ok`.
	#[test]
	fn par_fold_right_ok() {
		let x: Result<i32, ()> = Ok(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultErrAppliedBrand<()>, _, _>(f, 10, x), 15);
	}

	/// Tests `par_fold_right` on `Err`.
	#[test]
	fn par_fold_right_err_val() {
		let x: Result<i32, i32> = Err(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultErrAppliedBrand<i32>, _, _>(f, 10, x), 10);
	}

	// ParFoldable Tests for ResultOkAppliedBrand

	/// Tests `par_fold_map` on `Err` (which holds the value for ResultOkAppliedBrand).
	#[test]
	fn par_fold_map_err_ok_brand() {
		let x: Result<(), i32> = Err(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, ResultOkAppliedBrand<()>, _, _>(f.clone(), x),
			"5".to_string()
		);
	}

	/// Tests `par_fold_map` on `Ok` (which is empty for ResultOkAppliedBrand).
	#[test]
	fn par_fold_map_ok_ok_brand() {
		let x: Result<i32, i32> = Ok(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, ResultOkAppliedBrand<i32>, _, _>(f, x),
			"".to_string()
		);
	}

	/// Tests `par_fold_right` on `Err` (which holds the value for ResultOkAppliedBrand).
	#[test]
	fn par_fold_right_err_ok_brand() {
		let x: Result<(), i32> = Err(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(
			par_fold_right::<ArcFnBrand, ResultOkAppliedBrand<()>, _, _>(f.clone(), 10, x),
			15
		);
	}

	/// Tests `par_fold_right` on `Ok` (which is empty for ResultOkAppliedBrand).
	#[test]
	fn par_fold_right_ok_ok_brand() {
		let x: Result<i32, i32> = Ok(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultOkAppliedBrand<i32>, _, _>(f, 10, x), 10);
	}
}
