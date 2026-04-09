//! Functional programming trait implementations for the standard library [`Option`] type.
//!
//! Extends `Option` with [`Functor`](crate::classes::Functor), [`Monad`](crate::classes::semimonad::Semimonad), [`Foldable`](crate::classes::Foldable), [`Traversable`](crate::classes::Traversable), [`Filterable`](crate::classes::Filterable), and [`Witherable`](crate::classes::Witherable) instances.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::OptionBrand,
			classes::{
				dispatch::Ref,
				*,
			},
			impl_kind,
			kinds::*,
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	impl_kind! {
		for OptionBrand {
			type Of<'a, A: 'a>: 'a = Option<A>;
		}
	}

	impl Functor for OptionBrand {
		/// Maps a function over the value in the option.
		///
		/// This method applies a function to the value inside the option, producing a new option with the transformed value. If the option is `None`, it returns `None`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the option.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply to the value.", "The option to map over.")]
		///
		#[document_returns(
			"A new option containing the result of applying the function, or `None`."
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
		/// let x = Some(5);
		/// let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
		/// assert_eq!(y, Some(10));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(func)
		}
	}

	impl Lift for OptionBrand {
		/// Lifts a binary function into the option context.
		///
		/// This method lifts a binary function to operate on values within the option context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first option's value.",
			"The type of the second option's value.",
			"The return type of the function."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first option.",
			"The second option."
		)]
		///
		#[document_returns("`Some(f(a, b))` if both options are `Some`, otherwise `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(1);
		/// let y = Some(2);
		/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, x, y);
		/// assert_eq!(z, Some(3));
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: 'a,
			B: 'a,
			C: 'a, {
			fa.zip(fb).map(|(a, b)| func(a, b))
		}
	}

	impl Pointed for OptionBrand {
		/// Wraps a value in an option.
		///
		/// This method wraps a value in an option context.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("`Some(a)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::OptionBrand,
		/// 	functions::*,
		/// };
		///
		/// let x = pure::<OptionBrand, _>(5);
		/// assert_eq!(x, Some(5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Some(a)
		}
	}

	impl ApplyFirst for OptionBrand {}
	impl ApplySecond for OptionBrand {}

	impl Semiapplicative for OptionBrand {
		/// Applies a wrapped function to a wrapped value.
		///
		/// This method applies a function wrapped in an option to a value wrapped in an option.
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
			"The option containing the function.",
			"The option containing the value."
		)]
		///
		#[document_returns("`Some(f(a))` if both are `Some`, otherwise `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let x = Some(5);
		/// let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
		/// assert_eq!(y, Some(10));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match (ff, fa) {
				(Some(f), Some(a)) => Some(f(a)),
				_ => None,
			}
		}
	}

	impl Semimonad for OptionBrand {
		/// Chains option computations.
		///
		/// This method chains two option computations, where the second computation depends on the result of the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first option.",
			"The function to apply to the value inside the option."
		)]
		///
		#[document_returns(
			"The result of applying `f` to the value if `ma` is `Some`, otherwise `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::OptionBrand,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
		/// assert_eq!(y, Some(10));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.and_then(func)
		}
	}

	impl Alt for OptionBrand {
		/// Chooses between two options.
		///
		/// Returns the first `Some` value, or `None` if both are `None`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the values.", "The type of the value.")]
		///
		#[document_parameters("The first option.", "The second option.")]
		///
		#[document_returns("The first `Some` value, or `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(alt::<OptionBrand, _>(None, Some(5)), Some(5));
		/// assert_eq!(alt::<OptionBrand, _>(Some(3), Some(5)), Some(3));
		/// assert_eq!(alt::<OptionBrand, _>(None::<i32>, None), None);
		/// ```
		fn alt<'a, A: 'a>(
			fa1: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fa2: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa1.or(fa2)
		}
	}

	impl Plus for OptionBrand {
		/// Returns `None`, the identity element for [`alt`](Alt::alt).
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value.")]
		///
		#[document_returns("`None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Option<i32> = plus_empty::<OptionBrand, i32>();
		/// assert_eq!(x, None);
		/// ```
		fn empty<'a, A: 'a>() -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			None
		}
	}

	impl Foldable for OptionBrand {
		/// Folds the option from the right.
		///
		/// This method performs a right-associative fold of the option. If the option is `Some(a)`, it applies the function to `a` and the initial value. If `None`, it returns the initial value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The option to fold.")]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Some(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = fold_right::<RcFnBrand, OptionBrand, _, _, _>(|a, b| a + b, 10, x);
		/// assert_eq!(y, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			match fa {
				Some(a) => func(a, initial),
				None => initial,
			}
		}

		/// Folds the option from the left.
		///
		/// This method performs a left-associative fold of the option. If the option is `Some(a)`, it applies the function to the initial value and `a`. If `None`, it returns the initial value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The option to fold."
		)]
		///
		#[document_returns("`f(initial, a)` if `fa` is `Some(a)`, otherwise `initial`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = fold_left::<RcFnBrand, OptionBrand, _, _, _>(|b, a| b + a, 10, x);
		/// assert_eq!(y, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			match fa {
				Some(a) => func(initial, a),
				None => initial,
			}
		}

		/// Maps the value to a monoid and returns it, or returns empty.
		///
		/// This method maps the element of the option to a monoid. If the option is `None`, it returns the monoid's identity element.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The option to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Some(a)`, otherwise `M::empty()`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = fold_map::<RcFnBrand, OptionBrand, _, _, _>(|a: i32| a.to_string(), x);
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			match fa {
				Some(a) => func(a),
				None => M::empty(),
			}
		}
	}

	impl Traversable for OptionBrand {
		/// Traverses the option with an applicative function.
		///
		/// This method maps the element of the option to a computation, evaluates it, and wraps the result in the applicative context. If `None`, it returns `pure(None)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a value in an applicative context.",
			"The option to traverse."
		)]
		///
		#[document_returns("The option wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _>(|a| Some(a * 2), x);
		/// assert_eq!(y, Some(Some(10)));
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			match ta {
				Some(a) => F::map(|b| Some(b), func(a)),
				None => F::pure(None),
			}
		}

		/// Sequences an option of applicative.
		///
		/// This method evaluates the computation inside the option and wraps the result in the applicative context. If `None`, it returns `pure(None)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The option containing the applicative value.")]
		///
		#[document_returns("The result of the traversal.")]
		///
		/// # Returns
		///
		/// The option wrapped in the applicative context.
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::OptionBrand,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(Some(5));
		/// let y = sequence::<OptionBrand, _, OptionBrand>(x);
		/// assert_eq!(y, Some(Some(5)));
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			match ta {
				Some(fa) => F::map(|a| Some(a), fa),
				None => F::pure(None),
			}
		}
	}

	impl WithIndex for OptionBrand {
		type Index = ();
	}

	impl FunctorWithIndex for OptionBrand {
		/// Maps a function over the value in the option, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the option.",
			"The type of the result of applying the function."
		)]
		#[document_parameters(
			"The function to apply to the value and its index.",
			"The option to map over."
		)]
		#[document_returns(
			"A new option containing the result of applying the function, or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::OptionBrand,
		/// 	classes::functor_with_index::FunctorWithIndex,
		/// 	functions::*,
		/// };
		/// let x = Some(5);
		/// let y = <OptionBrand as FunctorWithIndex>::map_with_index(|_, i| i * 2, x);
		/// assert_eq!(y, Some(10));
		/// ```
		fn map_with_index<'a, A: 'a, B: 'a>(
			f: impl Fn((), A) -> B + 'a,
			fa: Option<A>,
		) -> Option<B> {
			fa.map(|a| f((), a))
		}
	}

	impl FoldableWithIndex for OptionBrand {
		/// Folds the option using a monoid, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the value.",
			"The brand of the cloneable function to use.",
			"The type of the value inside the option.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to the value and its index.",
			"The option to fold."
		)]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// 	functions::*,
		/// };
		/// let x = Some(5);
		/// let y = <OptionBrand as FoldableWithIndex>::fold_map_with_index::<RcFnBrand, _, _>(
		/// 	|_, i: i32| i.to_string(),
		/// 	x,
		/// );
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid + 'a>(
			f: impl Fn((), A) -> R + 'a,
			fa: Option<A>,
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			match fa {
				Some(a) => f((), a),
				None => R::empty(),
			}
		}
	}

	impl TraversableWithIndex for OptionBrand {
		/// Traverses the option with an applicative function, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the option.",
			"The type of the result.",
			"The applicative context."
		)]
		#[document_parameters(
			"The function to apply to the value and its index, returning a value in an applicative context.",
			"The option to traverse."
		)]
		#[document_returns("The option wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::OptionBrand,
		/// 	classes::traversable_with_index::TraversableWithIndex,
		/// 	functions::*,
		/// };
		/// let x = Some(5);
		/// let y = <OptionBrand as TraversableWithIndex>::traverse_with_index::<i32, i32, OptionBrand>(
		/// 	|_, i| Some(i * 2),
		/// 	x,
		/// );
		/// assert_eq!(y, Some(Some(10)));
		/// ```
		fn traverse_with_index<'a, A: 'a, B: 'a + Clone, M: Applicative>(
			f: impl Fn((), A) -> M::Of<'a, B> + 'a,
			ta: Option<A>,
		) -> M::Of<'a, Option<B>> {
			match ta {
				Some(a) => M::map(|b| Some(b), f((), a)),
				None => M::pure(None),
			}
		}
	}

	impl Compactable for OptionBrand {
		/// Compacts a nested option.
		///
		/// This method flattens a nested option.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the values.", "The type of the elements.")]
		///
		#[document_parameters("The nested option.")]
		///
		#[document_returns("The flattened option.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::OptionBrand,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(Some(5));
		/// let y = compact::<OptionBrand, _>(x);
		/// assert_eq!(y, Some(5));
		/// ```
		fn compact<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.flatten()
		}

		/// Separates an option of result.
		///
		/// This method separates an option of result into a pair of options.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters("The option of result.")]
		///
		#[document_returns("A pair of options.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Option<Result<i32, &str>> = Some(Ok(5));
		/// let (errs, oks) = separate::<OptionBrand, _, _>(x);
		/// assert_eq!(oks, Some(5));
		/// assert_eq!(errs, None);
		/// ```
		fn separate<'a, E: 'a, O: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			match fa {
				Some(Ok(o)) => (None, Some(o)),
				Some(Err(e)) => (Some(e), None),
				None => (None, None),
			}
		}
	}

	impl Filterable for OptionBrand {
		/// Partitions an option based on a function that returns a result.
		///
		/// This method partitions an option based on a function that returns a result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the input value.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters("The function to apply.", "The option to partition.")]
		///
		#[document_returns("A pair of options.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let (errs, oks) =
		/// 	partition_map::<OptionBrand, _, _, _>(|a| if a > 2 { Ok(a) } else { Err(a) }, x);
		/// assert_eq!(oks, Some(5));
		/// assert_eq!(errs, None);
		/// ```
		fn partition_map<'a, A: 'a, E: 'a, O: 'a>(
			func: impl Fn(A) -> Result<O, E> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			match fa {
				Some(a) => match func(a) {
					Ok(o) => (None, Some(o)),
					Err(e) => (Some(e), None),
				},
				None => (None, None),
			}
		}

		/// Partitions an option based on a predicate.
		///
		/// This method partitions an option based on a predicate.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the values.", "The type of the elements.")]
		///
		#[document_parameters("The predicate.", "The option to partition.")]
		///
		#[document_returns("A pair of options.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let (not_satisfied, satisfied) = partition::<OptionBrand, _>(|a| a > 2, x);
		/// assert_eq!(satisfied, Some(5));
		/// assert_eq!(not_satisfied, None);
		/// ```
		fn partition<'a, A: 'a + Clone>(
			func: impl Fn(A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			match fa {
				Some(a) =>
					if func(a.clone()) {
						(None, Some(a))
					} else {
						(Some(a), None)
					},
				None => (None, None),
			}
		}

		/// Maps a function over an option and filters out `None` results.
		///
		/// This method maps a function over an option and filters out `None` results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the input value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply.", "The option to filter and map.")]
		///
		#[document_returns("The filtered and mapped option.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::OptionBrand,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = filter_map::<OptionBrand, _, _, _>(|a| if a > 2 { Some(a * 2) } else { None }, x);
		/// assert_eq!(y, Some(10));
		/// ```
		fn filter_map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> Option<B> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.and_then(func)
		}

		/// Filters an option based on a predicate.
		///
		/// This method filters an option based on a predicate.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the values.", "The type of the elements.")]
		///
		#[document_parameters("The predicate.", "The option to filter.")]
		///
		#[document_returns("The filtered option.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::OptionBrand,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = filter::<OptionBrand, _>(|a| a > 2, x);
		/// assert_eq!(y, Some(5));
		/// ```
		fn filter<'a, A: 'a + Clone>(
			func: impl Fn(A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.filter(|a| func(a.clone()))
		}
	}

	impl Witherable for OptionBrand {
		/// Partitions an option based on a function that returns a result in an applicative context.
		///
		/// This method partitions an option based on a function that returns a result in an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The applicative context.",
			"The type of the elements in the input structure.",
			"The type of the error values.",
			"The type of the success values."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a `Result` in an applicative context.",
			"The option to partition."
		)]
		///
		#[document_returns("The partitioned option wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y =
		/// 	wilt::<OptionBrand, OptionBrand, _, _, _>(|a| Some(if a > 2 { Ok(a) } else { Err(a) }), x);
		/// assert_eq!(y, Some((None, Some(5))));
		/// ```
		fn wilt<'a, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone>(
			func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
			+ 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		(
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		),
	>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone, {
			match ta {
				Some(a) => M::map(
					|res| match res {
						Ok(o) => (None, Some(o)),
						Err(e) => (Some(e), None),
					},
					func(a),
				),
				None => M::pure((None, None)),
			}
		}

		/// Maps a function over an option and filters out `None` results in an applicative context.
		///
		/// This method maps a function over an option and filters out `None` results in an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The applicative context.",
			"The type of the elements in the input structure.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning an `Option` in an applicative context.",
			"The option to filter and map."
		)]
		///
		#[document_returns("The filtered and mapped option wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = wither::<OptionBrand, OptionBrand, _, _>(
		/// 	|a| Some(if a > 2 { Some(a * 2) } else { None }),
		/// 	x,
		/// );
		/// assert_eq!(y, Some(Some(10)));
		/// ```
		fn wither<'a, M: Applicative, A: 'a + Clone, B: 'a + Clone>(
			func: impl Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone, {
			match ta {
				Some(a) => func(a),
				None => M::pure(None),
			}
		}
	}

	impl MonadRec for OptionBrand {
		/// Performs tail-recursive monadic computation over [`Option`].
		///
		/// Iteratively applies the step function. If the function returns [`None`],
		/// the computation short-circuits. If it returns `Some(ControlFlow::Continue(a))`, the
		/// loop continues with the new state. If it returns `Some(ControlFlow::Break(b))`,
		/// the computation completes with `Some(b)`.
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
			"The result of the computation, or `None` if the step function returned `None`."
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
		/// let result = tail_rec_m::<OptionBrand, _, _>(
		/// 	|n| {
		/// 		if n < 10 { Some(ControlFlow::Continue(n + 1)) } else { Some(ControlFlow::Break(n)) }
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, Some(10));
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
					None => return None,
					Some(ControlFlow::Continue(next)) => current = next,
					Some(ControlFlow::Break(b)) => return Some(b),
				}
			}
		}
	}

	// -- By-reference trait implementations --

	impl RefFunctor for OptionBrand {
		/// Maps a function over the option by reference.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function.", "The option.")]
		#[document_returns("The mapped option.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// assert_eq!(map::<OptionBrand, _, _, _>(|x: &i32| *x * 2, Some(5)), Some(10));
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.as_ref().map(func)
		}
	}

	impl RefFoldable for OptionBrand {
		/// Folds the option by reference.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The element type.",
			"The monoid type."
		)]
		#[document_parameters("The mapping function.", "The option.")]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = fold_map::<RcFnBrand, OptionBrand, _, _, _>(|x: &i32| x.to_string(), Some(5));
		/// assert_eq!(result, "5");
		/// ```
		fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(&A) -> M + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: LiftFn + 'a,
			M: Monoid + 'a, {
			match fa {
				Some(a) => func(a),
				None => Monoid::empty(),
			}
		}
	}

	impl RefFilterable for OptionBrand {
		/// Filters and maps the option by reference.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function.", "The option.")]
		#[document_returns("The filtered option.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = ref_filter_map::<OptionBrand, _, _>(
		/// 	|x: &i32| if *x > 3 { Some(*x) } else { None },
		/// 	Some(5),
		/// );
		/// assert_eq!(result, Some(5));
		/// ```
		fn ref_filter_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> Option<B> + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.as_ref().and_then(func)
		}
	}

	impl RefTraversable for OptionBrand {
		/// Traverses the option by reference.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The input type.",
			"The output type.",
			"The applicative."
		)]
		#[document_parameters("The function.", "The option.")]
		#[document_returns("The traversed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result: Vec<Option<String>> = ref_traverse::<OptionBrand, RcFnBrand, _, _, VecBrand>(
		/// 	|x: &i32| vec![x.to_string()],
		/// 	Some(5),
		/// );
		/// assert_eq!(result, vec![Some("5".to_string())]);
		/// ```
		fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			match ta {
				Some(a) => F::map(|b| Some(b), func(a)),
				None => F::pure(None),
			}
		}
	}

	impl RefWitherable for OptionBrand {}

	impl RefFunctorWithIndex for OptionBrand {
		/// Maps by reference with index (always `()`).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function.", "The option.")]
		#[document_returns("The mapped option.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// assert_eq!(ref_map_with_index::<OptionBrand, _, _>(|(), x: &i32| *x * 2, Some(5)), Some(10));
		/// ```
		fn ref_map_with_index<'a, A: 'a, B: 'a>(
			func: impl Fn((), &A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.as_ref().map(|a| func((), a))
		}
	}

	impl RefFoldableWithIndex for OptionBrand {
		/// Folds by reference with index (always `()`).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand of the cloneable function.",
			"The element type.",
			"The monoid type."
		)]
		#[document_parameters("The function.", "The option.")]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = ref_fold_map_with_index::<RcFnBrand, OptionBrand, _, _>(
		/// 	|(), x: &i32| x.to_string(),
		/// 	Some(5),
		/// );
		/// assert_eq!(result, "5");
		/// ```
		fn ref_fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid + 'a>(
			func: impl Fn((), &A) -> R + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			match fa {
				Some(a) => func((), a),
				None => Monoid::empty(),
			}
		}
	}

	impl RefTraversableWithIndex for OptionBrand {
		/// Traverses by reference with index (always `()`).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The input type.",
			"The output type.",
			"The applicative."
		)]
		#[document_parameters("The function.", "The option.")]
		#[document_returns("The traversed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result: Vec<Option<String>> = ref_traverse_with_index::<OptionBrand, _, _, VecBrand>(
		/// 	|(), x: &i32| vec![x.to_string()],
		/// 	Some(5),
		/// );
		/// assert_eq!(result, vec![Some("5".to_string())]);
		/// ```
		fn ref_traverse_with_index<'a, A: 'a + Clone, B: 'a + Clone, M: Applicative>(
			f: impl Fn((), &A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			match ta {
				Some(a) => M::map(|b| Some(b), f((), a)),
				None => M::pure(None),
			}
		}
	}

	// -- By-reference monadic trait implementations --

	impl RefPointed for OptionBrand {
		/// Creates a `Some` from a reference by cloning.
		#[document_signature]
		#[document_type_parameters("The lifetime of the value.", "The type of the value.")]
		#[document_parameters("The reference to the value to wrap.")]
		#[document_returns("A `Some` containing a clone of the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = 42;
		/// let result: Option<i32> = ref_pure::<OptionBrand, _>(&x);
		/// assert_eq!(result, Some(42));
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Some(a.clone())
		}
	}

	impl RefLift for OptionBrand {
		/// Combines two `Option` values with a by-reference binary function.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"First input type.",
			"Second input type.",
			"Output type."
		)]
		#[document_parameters("The binary function.", "The first option.", "The second option.")]
		#[document_returns("The combined result, or `None` if either input is `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = lift2::<OptionBrand, _, _, _, _>(|a: &i32, b: &i32| *a + *b, Some(1), Some(2));
		/// assert_eq!(result, Some(3));
		/// ```
		fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
			func: impl Fn(&A, &B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			match (fa.as_ref(), fb.as_ref()) {
				(Some(a), Some(b)) => Some(func(a, b)),
				_ => None,
			}
		}
	}

	impl RefSemiapplicative for OptionBrand {
		/// Applies a wrapped by-ref function to an `Option` value.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The function brand.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters(
			"The option containing the by-ref function.",
			"The option containing the value."
		)]
		#[document_returns("The result of applying the function, or `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f: std::rc::Rc<dyn Fn(&i32) -> i32> = std::rc::Rc::new(|x: &i32| *x + 1);
		/// let result = ref_apply::<RcFnBrand, OptionBrand, _, _>(Some(f), Some(5));
		/// assert_eq!(result, Some(6));
		/// ```
		fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, A: 'a, B: 'a>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match (ff, fa.as_ref()) {
				(Some(f), Some(a)) => Some((**f)(a)),
				_ => None,
			}
		}
	}

	impl RefSemimonad for OptionBrand {
		/// Chains `Option` computations by reference.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The input option.", "The function to apply by reference.")]
		#[document_returns("The result of applying the function, or `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result: Option<String> =
		/// 	bind::<OptionBrand, _, _, _>(Some(42), |x: &i32| Some(x.to_string()));
		/// assert_eq!(result, Some("42".to_string()));
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.as_ref().and_then(f)
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

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: Option<i32>) -> bool {
		map::<OptionBrand, _, _, _>(identity, x) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Option<i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<OptionBrand, _, _, _>(compose(f, g), x)
			== map::<OptionBrand, _, _, _>(f, map::<OptionBrand, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Option<i32>) -> bool {
		apply::<RcFnBrand, OptionBrand, _, _>(
			pure::<OptionBrand, _>(<RcFnBrand as LiftFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, OptionBrand, _, _>(
			pure::<OptionBrand, _>(<RcFnBrand as LiftFn>::new(f)),
			pure::<OptionBrand, _>(x),
		) == pure::<OptionBrand, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: Option<i32>,
		u_is_some: bool,
		v_is_some: bool,
	) -> bool {
		let v_fn = |x: i32| x.wrapping_mul(2);
		let u_fn = |x: i32| x.wrapping_add(1);

		let v =
			if v_is_some { pure::<OptionBrand, _>(<RcFnBrand as LiftFn>::new(v_fn)) } else { None };
		let u =
			if u_is_some { pure::<OptionBrand, _>(<RcFnBrand as LiftFn>::new(u_fn)) } else { None };

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, OptionBrand, _, _>(v.clone(), w);
		let rhs = apply::<RcFnBrand, OptionBrand, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let uv = match (u, v) {
			(Some(uf), Some(vf)) => {
				let composed = move |x| uf(vf(x));
				Some(<RcFnBrand as LiftFn>::new(composed))
			}
			_ => None,
		};

		let lhs = apply::<RcFnBrand, OptionBrand, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<OptionBrand, _>(<RcFnBrand as LiftFn>::new(f));

		let lhs = apply::<RcFnBrand, OptionBrand, _, _>(u.clone(), pure::<OptionBrand, _>(y));

		let rhs_fn = <RcFnBrand as LiftFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, OptionBrand, _, _>(pure::<OptionBrand, _>(rhs_fn), u);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| Some(x.wrapping_mul(2));
		bind::<OptionBrand, _, _, _>(pure::<OptionBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Option<i32>) -> bool {
		bind::<OptionBrand, _, _, _>(m, pure::<OptionBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Option<i32>) -> bool {
		let f = |x: i32| Some(x.wrapping_mul(2));
		let g = |x: i32| Some(x.wrapping_add(1));
		bind::<OptionBrand, _, _, _>(bind::<OptionBrand, _, _, _>(m, f), g)
			== bind::<OptionBrand, _, _, _>(m, |x| bind::<OptionBrand, _, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests `map` on `None`.
	#[test]
	fn map_none() {
		assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x + 1, None), None);
	}

	/// Tests `bind` on `None`.
	#[test]
	fn bind_none() {
		assert_eq!(bind::<OptionBrand, _, _, _>(None, |x: i32| Some(x + 1)), None);
	}

	/// Tests `bind` returning `None`.
	#[test]
	fn bind_returning_none() {
		assert_eq!(bind::<OptionBrand, _, _, _>(Some(5), |_| None::<i32>), None);
	}

	/// Tests `fold_right` on `None`.
	#[test]
	fn fold_right_none() {
		assert_eq!(
			crate::functions::fold_right::<RcFnBrand, OptionBrand, _, _, _>(
				|x: i32, acc| x + acc,
				0,
				None
			),
			0
		);
	}

	/// Tests `fold_left` on `None`.
	#[test]
	fn fold_left_none() {
		assert_eq!(
			crate::functions::fold_left::<RcFnBrand, OptionBrand, _, _, _>(
				|acc, x: i32| acc + x,
				0,
				None
			),
			0
		);
	}

	/// Tests `traverse` on `None`.
	#[test]
	fn traverse_none() {
		assert_eq!(
			crate::classes::traversable::traverse::<OptionBrand, _, _, OptionBrand>(
				|x: i32| Some(x + 1),
				None
			),
			Some(None)
		);
	}

	/// Tests `traverse` returning `None`.
	#[test]
	fn traverse_returning_none() {
		assert_eq!(
			crate::classes::traversable::traverse::<OptionBrand, _, _, OptionBrand>(
				|_: i32| None::<i32>,
				Some(5)
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
		tail_rec_m::<OptionBrand, _, _>(|a| Some(ControlFlow::Break(a)), x) == Some(x)
	}

	/// Tests a recursive computation that sums a range via `tail_rec_m`.
	#[test]
	fn monad_rec_sum_range() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		// Sum 1..=100 using tail_rec_m
		let result = tail_rec_m::<OptionBrand, _, _>(
			|(n, acc)| {
				if n == 0 {
					Some(ControlFlow::Break(acc))
				} else {
					Some(ControlFlow::Continue((n - 1, acc + n)))
				}
			},
			(100i64, 0i64),
		);
		assert_eq!(result, Some(5050));
	}

	/// Tests that `tail_rec_m` short-circuits on `None`.
	#[test]
	fn monad_rec_short_circuit() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result: Option<i32> = tail_rec_m::<OptionBrand, _, _>(
			|n| {
				if n == 5 { None } else { Some(ControlFlow::Continue(n + 1)) }
			},
			0,
		);
		assert_eq!(result, None);
	}

	/// Tests stack safety: `tail_rec_m` handles large iteration counts.
	#[test]
	fn monad_rec_stack_safety() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<OptionBrand, _, _>(
			|acc| {
				if acc < iterations {
					Some(ControlFlow::Continue(acc + 1))
				} else {
					Some(ControlFlow::Break(acc))
				}
			},
			0i64,
		);
		assert_eq!(result, Some(iterations));
	}

	// RefFunctor Laws

	/// Tests the identity law for RefFunctor: `ref_map(|x| *x, opt) == opt`.
	#[quickcheck]
	fn ref_functor_identity(opt: Option<i32>) -> bool {
		use crate::classes::ref_functor::RefFunctor;
		OptionBrand::ref_map(|x: &i32| *x, opt) == opt
	}

	/// Tests the composition law for RefFunctor.
	#[quickcheck]
	fn ref_functor_composition(opt: Option<i32>) -> bool {
		use crate::classes::ref_functor::RefFunctor;
		let f = |x: &i32| x.wrapping_add(1);
		let g = |x: &i32| x.wrapping_mul(2);
		OptionBrand::ref_map(|x: &i32| f(&g(x)), opt)
			== OptionBrand::ref_map(f, OptionBrand::ref_map(g, opt))
	}

	// RefSemimonad Laws

	/// Tests the left identity law for RefSemimonad: `ref_bind(Some(x), |a| Some(*a)) == Some(x)`.
	#[quickcheck]
	fn ref_semimonad_left_identity(x: i32) -> bool {
		use crate::classes::ref_semimonad::RefSemimonad;
		OptionBrand::ref_bind(Some(x), |a: &i32| Some(*a)) == Some(x)
	}

	// RefFoldable Laws

	/// Tests RefFoldable fold_map on Option with Additive monoid.
	#[quickcheck]
	fn ref_foldable_fold_map(opt: Option<i32>) -> bool {
		use crate::{
			classes::ref_foldable::RefFoldable,
			types::Additive,
		};
		let result = OptionBrand::ref_fold_map::<RcFnBrand, _, _>(|x: &i32| Additive(*x), opt);
		let expected = match opt {
			Some(v) => Additive(v),
			None => Additive(0),
		};
		result == expected
	}

	// RefSemimonad Laws (continued)

	/// Tests the right identity law for RefSemimonad:
	/// `ref_bind(m, |a| ref_pure(a)) == m`.
	#[quickcheck]
	fn ref_semimonad_right_identity(opt: Option<i32>) -> bool {
		use crate::classes::{
			ref_pointed::RefPointed,
			ref_semimonad::RefSemimonad,
		};
		OptionBrand::ref_bind(opt, |a: &i32| OptionBrand::ref_pure(a)) == opt
	}

	/// Tests the associativity law for RefSemimonad.
	#[quickcheck]
	fn ref_semimonad_associativity(opt: Option<i32>) -> bool {
		use crate::classes::ref_semimonad::RefSemimonad;
		let f = |a: &i32| if *a > 0 { Some(a.wrapping_mul(2)) } else { None };
		let g = |b: &i32| Some(b.wrapping_add(10));
		let lhs = OptionBrand::ref_bind(OptionBrand::ref_bind(opt, f), g);
		let rhs = OptionBrand::ref_bind(opt, |a: &i32| OptionBrand::ref_bind(f(a), g));
		lhs == rhs
	}

	// RefLift Laws

	/// Tests the identity law for RefLift:
	/// `ref_lift2(|_, b| *b, pure(unit), fa) == fa`.
	#[quickcheck]
	fn ref_lift_identity(opt: Option<i32>) -> bool {
		use crate::classes::ref_lift::RefLift;
		OptionBrand::ref_lift2(|_: &(), b: &i32| *b, Some(()), opt) == opt
	}

	/// Tests commutativity of ref_lift2 (for Option specifically):
	/// `ref_lift2(f, a, b) == ref_lift2(|b, a| f(a, b), b, a)`.
	#[quickcheck]
	fn ref_lift2_commutativity(
		a: Option<i32>,
		b: Option<i32>,
	) -> bool {
		use crate::classes::ref_lift::RefLift;
		let lhs = OptionBrand::ref_lift2(|x: &i32, y: &i32| x.wrapping_add(*y), a, b);
		let rhs = OptionBrand::ref_lift2(|y: &i32, x: &i32| x.wrapping_add(*y), b, a);
		lhs == rhs
	}

	// RefTraversable Laws

	/// Tests the identity law for RefTraversable:
	/// `ref_traverse(|a| Identity(*a), ta) == Identity(ta)`.
	#[quickcheck]
	fn ref_traversable_identity(opt: Option<i32>) -> bool {
		use crate::{
			classes::ref_traversable::RefTraversable,
			types::Identity,
		};
		let result: Identity<Option<i32>> =
			OptionBrand::ref_traverse::<RcFnBrand, _, _, IdentityBrand>(
				|a: &i32| Identity(*a),
				opt,
			);
		result == Identity(opt)
	}

	/// Tests RefTraversable naturality: ref_traverse(f, ta) produces
	/// the same result as traverse(|a| f(&a), ta).
	#[quickcheck]
	fn ref_traversable_consistent_with_traverse(opt: Option<i32>) -> bool {
		use crate::classes::{
			ref_traversable::RefTraversable,
			traversable::Traversable,
		};
		let ref_result: Option<Option<String>> =
			OptionBrand::ref_traverse::<RcFnBrand, _, _, OptionBrand>(
				|a: &i32| Some(a.to_string()),
				opt,
			);
		let val_result: Option<Option<String>> =
			OptionBrand::traverse::<i32, String, OptionBrand>(|a: i32| Some(a.to_string()), opt);
		ref_result == val_result
	}
}
