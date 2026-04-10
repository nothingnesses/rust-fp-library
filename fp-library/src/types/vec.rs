//! Functional programming trait implementations for the standard library [`Vec`] type.
//!
//! Extends `Vec` with [`Functor`](crate::classes::Functor), [`Monad`](crate::classes::semimonad::Semimonad), [`Foldable`](crate::classes::Foldable), [`Traversable`](crate::classes::Traversable), [`Extend`](crate::classes::Extend), [`Filterable`](crate::classes::Filterable), [`Witherable`](crate::classes::Witherable), and parallel folding instances.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				OptionBrand,
				VecBrand,
			},
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
		for VecBrand {
			type Of<'a, A: 'a>: 'a = Vec<A>;
		}
	}

	impl VecBrand {
		/// Constructs a new vector by prepending a value to an existing vector.
		///
		/// This method creates a new vector with the given head element followed by the elements of the tail vector.
		#[document_signature]
		///
		#[document_type_parameters("The type of the elements in the vector.")]
		///
		#[document_parameters(
			"A value to prepend to the vector.",
			"A vector to prepend the value to."
		)]
		///
		#[document_returns(
			"A new vector consisting of the `head` element prepended to the `tail` vector."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let head = 1;
		/// let tail = vec![2, 3];
		/// let new_vec = VecBrand::construct(head, tail);
		/// assert_eq!(new_vec, vec![1, 2, 3]);
		///
		/// let empty_tail = vec![];
		/// let single_element = VecBrand::construct(42, empty_tail);
		/// assert_eq!(single_element, vec![42]);
		/// ```
		pub fn construct<A>(
			head: A,
			tail: Vec<A>,
		) -> Vec<A>
		where
			A: Clone, {
			[vec![head], tail].concat()
		}

		/// Deconstructs a slice into its head element and tail vector.
		///
		/// This method splits a slice into its first element and the rest of the elements as a new vector.
		#[document_signature]
		///
		#[document_type_parameters("The type of the elements in the vector.")]
		///
		#[document_parameters("The vector slice to deconstruct.")]
		///
		#[document_returns(
			"An [`Option`] containing a tuple of the head element and the remaining tail vector, or [`None`] if the slice is empty."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let vec = vec![1, 2, 3];
		/// let deconstructed = VecBrand::deconstruct(&vec);
		/// assert_eq!(deconstructed, Some((1, vec![2, 3])));
		///
		/// let empty: Vec<i32> = vec![];
		/// assert_eq!(VecBrand::deconstruct(&empty), None);
		/// ```
		pub fn deconstruct<A>(slice: &[A]) -> Option<(A, Vec<A>)>
		where
			A: Clone, {
			match slice {
				[] => None,
				[head, tail @ ..] => Some((head.clone(), tail.to_vec())),
			}
		}
	}

	impl Functor for VecBrand {
		/// Maps a function over the vector.
		///
		/// This method applies a function to each element of the vector, producing a new vector with the transformed values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the vector.",
			"The type of the elements in the resulting vector."
		)]
		///
		#[document_parameters("The function to apply to each element.", "The vector to map over.")]
		///
		#[document_returns("A new vector containing the results of applying the function.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(map::<VecBrand, _, _, _, _>(|x: i32| x * 2, vec![1, 2, 3]), vec![2, 4, 6]);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.into_iter().map(func).collect()
		}
	}

	impl Lift for VecBrand {
		/// Lifts a binary function into the vector context (Cartesian product).
		///
		/// This method applies a binary function to all pairs of elements from two vectors, producing a new vector containing the results (Cartesian product).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the first vector.",
			"The type of the elements in the second vector.",
			"The type of the elements in the resulting vector."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first vector.",
			"The second vector."
		)]
		///
		#[document_returns(
			"A new vector containing the results of applying the function to all pairs of elements."
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
		/// 	lift2::<VecBrand, _, _, _, _, _, _>(|x, y| x + y, vec![1, 2], vec![10, 20]),
		/// 	vec![11, 21, 12, 22]
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
			fa.iter().flat_map(|a| fb.iter().map(|b| func(a.clone(), b.clone()))).collect()
		}
	}

	impl Pointed for VecBrand {
		/// Wraps a value in a vector.
		///
		/// This method creates a new vector containing the single given value.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A vector containing the single value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(pure::<VecBrand, _>(5), vec![5]);
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			vec![a]
		}
	}

	impl ApplyFirst for VecBrand {}
	impl ApplySecond for VecBrand {}

	impl Semiapplicative for VecBrand {
		/// Applies wrapped functions to wrapped values (Cartesian product).
		///
		/// This method applies each function in the first vector to each value in the second vector, producing a new vector containing all the results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input values.",
			"The type of the output values."
		)]
		///
		#[document_parameters(
			"The vector containing the functions.",
			"The vector containing the values."
		)]
		///
		#[document_returns(
			"A new vector containing the results of applying each function to each value."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let funcs = vec![
		/// 	lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1),
		/// 	lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2),
		/// ];
		/// assert_eq!(apply::<RcFnBrand, VecBrand, _, _>(funcs, vec![1, 2]), vec![2, 3, 2, 4]);
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ff.iter().flat_map(|f| fa.iter().map(move |a| f(a.clone()))).collect()
		}
	}

	impl Semimonad for VecBrand {
		/// Chains vector computations (`flat_map`).
		///
		/// This method applies a function that returns a vector to each element of the input vector, and then flattens the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the input vector.",
			"The type of the elements in the output vector."
		)]
		///
		#[document_parameters(
			"The first vector.",
			"The function to apply to each element, returning a vector."
		)]
		///
		#[document_returns("A new vector containing the flattened results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(bind::<VecBrand, _, _, _, _>(vec![1, 2], |x| vec![x, x * 2]), vec![1, 2, 2, 4]);
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.into_iter().flat_map(func).collect()
		}
	}

	impl Alt for VecBrand {
		/// Concatenates two vectors.
		///
		/// This is the same as [`Semigroup::append`] for `Vec`, providing an
		/// associative choice operation for the `Vec` type constructor.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The first vector.", "The second vector.")]
		///
		#[document_returns("The concatenated vector.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let x = vec![1, 2];
		/// let y = vec![3, 4];
		/// assert_eq!(alt::<VecBrand, _>(x, y), vec![1, 2, 3, 4]);
		/// ```
		fn alt<'a, A: 'a>(
			fa1: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fa2: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			let mut result = fa1;
			result.extend(fa2);
			result
		}
	}

	impl RefAlt for VecBrand {
		/// Concatenates two vectors by reference.
		///
		/// Both input vectors are borrowed and their elements are cloned to
		/// construct a new vector containing all elements from both inputs.
		/// This is the by-reference counterpart of [`Alt::alt`] for `Vec`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The first vector.", "The second vector.")]
		///
		#[document_returns("A new vector containing cloned elements from both inputs.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let x = vec![1, 2];
		/// let y = vec![3, 4];
		/// assert_eq!(ref_alt::<VecBrand, _>(&x, &y), vec![1, 2, 3, 4]);
		/// ```
		fn ref_alt<'a, A: 'a + Clone>(
			fa1: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fa2: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa1.iter().chain(fa2.iter()).cloned().collect()
		}
	}

	impl Plus for VecBrand {
		/// Returns an empty vector, the identity element for [`alt`](Alt::alt).
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_returns("An empty vector.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Vec<i32> = plus_empty::<VecBrand, i32>();
		/// assert_eq!(x, vec![]);
		/// ```
		fn empty<'a, A: 'a>() -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Vec::new()
		}
	}

	impl Foldable for VecBrand {
		/// Folds the vector from the right.
		///
		/// This method performs a right-associative fold of the vector.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the vector.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The vector to fold.")]
		///
		#[document_returns("The final accumulator value.")]
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
		/// 	fold_right::<RcFnBrand, VecBrand, _, _, _, _>(|x: i32, acc| x + acc, 0, vec![1, 2, 3]),
		/// 	6
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			fa.into_iter().rev().fold(initial, |acc, x| func(x, acc))
		}

		/// Folds the vector from the left.
		///
		/// This method performs a left-associative fold of the vector.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the vector.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The vector to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, VecBrand, _, _, _, _>(|acc, x: i32| acc + x, 0, vec![1, 2, 3]),
		/// 	6
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			fa.into_iter().fold(initial, func)
		}

		/// Maps the values to a monoid and combines them.
		///
		/// This method maps each element of the vector to a monoid and then combines the results using the monoid's `append` operation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the vector.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The vector to fold.")]
		///
		#[document_returns("The combined monoid value.")]
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
		/// 	fold_map::<RcFnBrand, VecBrand, _, _, _, _>(|x: i32| x.to_string(), vec![1, 2, 3]),
		/// 	"123".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			fa.into_iter().map(func).fold(M::empty(), |acc, x| M::append(acc, x))
		}
	}

	impl Traversable for VecBrand {
		/// Traverses the vector with an applicative function.
		///
		/// This method maps each element of the vector to a computation, evaluates them, and combines the results into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a value in an applicative context.",
			"The vector to traverse."
		)]
		///
		#[document_returns("The vector wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(|x| Some(x * 2), vec![1, 2, 3]),
		/// 	Some(vec![2, 4, 6])
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let len = ta.len();
			ta.into_iter().fold(F::pure(Vec::with_capacity(len)), |acc, x| {
				F::lift2(
					|mut v, b| {
						v.push(b);
						v
					},
					acc,
					func(x),
				)
			})
		}

		/// Sequences a vector of applicative.
		///
		/// This method evaluates the computations inside the vector and accumulates the results into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The vector containing the applicative values.")]
		///
		#[document_returns("The vector wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		VecBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(sequence::<VecBrand, _, OptionBrand>(vec![Some(1), Some(2)]), Some(vec![1, 2]));
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			let len = ta.len();
			ta.into_iter().fold(F::pure(Vec::with_capacity(len)), |acc, x| {
				F::lift2(
					|mut v, a| {
						v.push(a);
						v
					},
					acc,
					x,
				)
			})
		}
	}

	impl WithIndex for VecBrand {
		type Index = usize;
	}

	impl FunctorWithIndex for VecBrand {
		/// Maps a function over the vector, providing the index of each element.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the vector.",
			"The type of the elements in the resulting vector."
		)]
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The vector to map over."
		)]
		#[document_returns("A new vector containing the results of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		/// let v = vec![10, 20, 30];
		/// // Use `map_with_index` via the method on the trait, or a helper function if one existed.
		/// // Since there's no helper function in `functions.rs` yet, we use explicit syntax or call it via trait.
		/// use fp_library::classes::functor_with_index::FunctorWithIndex;
		/// let mapped = <VecBrand as FunctorWithIndex>::map_with_index(|i, x| x + i as i32, v);
		/// assert_eq!(mapped, vec![10, 21, 32]);
		/// ```
		fn map_with_index<'a, A: 'a, B: 'a>(
			f: impl Fn(usize, A) -> B + 'a,
			fa: Vec<A>,
		) -> Vec<B> {
			fa.into_iter().enumerate().map(|(i, a)| f(i, a)).collect()
		}
	}

	impl FoldableWithIndex for VecBrand {
		/// Folds the vector using a monoid, providing the index of each element.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the vector.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The vector to fold."
		)]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// 	functions::*,
		/// };
		/// let v = vec![10, 20, 30];
		/// let s = <VecBrand as FoldableWithIndex>::fold_map_with_index::<RcFnBrand, _, _>(
		/// 	|i, x| format!("{}:{}", i, x),
		/// 	v,
		/// );
		/// assert_eq!(s, "0:101:202:30");
		/// ```
		fn fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid + 'a>(
			f: impl Fn(usize, A) -> R + 'a,
			fa: Vec<A>,
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			fa.into_iter()
				.enumerate()
				.map(|(i, a)| f(i, a))
				.fold(R::empty(), |acc, x| R::append(acc, x))
		}
	}

	impl TraversableWithIndex for VecBrand {
		/// Traverses the vector with an applicative function, providing the index of each element.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the vector.",
			"The type of the elements in the resulting vector.",
			"The applicative context."
		)]
		#[document_parameters(
			"The function to apply to each element and its index, returning a value in an applicative context.",
			"The vector to traverse."
		)]
		#[document_returns("The vector wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		VecBrand,
		/// 	},
		/// 	classes::traversable_with_index::TraversableWithIndex,
		/// 	functions::*,
		/// };
		/// let v = vec![10, 20, 30];
		/// let t = <VecBrand as TraversableWithIndex>::traverse_with_index::<i32, i32, OptionBrand>(
		/// 	|i, x| Some(x + i as i32),
		/// 	v,
		/// );
		/// assert_eq!(t, Some(vec![10, 21, 32]));
		/// ```
		fn traverse_with_index<'a, A: 'a, B: 'a + Clone, M: Applicative>(
			f: impl Fn(usize, A) -> M::Of<'a, B> + 'a,
			ta: Vec<A>,
		) -> M::Of<'a, Vec<B>> {
			let len = ta.len();
			ta.into_iter().enumerate().fold(M::pure(Vec::with_capacity(len)), |acc, (i, x)| {
				M::lift2(
					|mut v, b| {
						v.push(b);
						v
					},
					acc,
					f(i, x),
				)
			})
		}
	}

	#[document_type_parameters("The type of the elements in the vector.")]
	impl<A: Clone> Semigroup for Vec<A> {
		/// Appends one vector to another.
		///
		/// This method concatenates two vectors.
		#[document_signature]
		///
		#[document_parameters("The first vector.", "The second vector.")]
		///
		#[document_returns("The concatenated vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::functions::*;
		///
		/// assert_eq!(append(vec![1, 2], vec![3, 4]), vec![1, 2, 3, 4]);
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			[a, b].concat()
		}
	}

	#[document_type_parameters("The type of the elements in the vector.")]
	impl<A: Clone> Monoid for Vec<A> {
		/// Returns an empty vector.
		///
		/// This method returns a new, empty vector.
		#[document_signature]
		///
		#[document_returns("An empty vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::functions::*;
		///
		/// assert_eq!(empty::<Vec<i32>>(), vec![]);
		/// ```
		fn empty() -> Self {
			Vec::new()
		}
	}

	impl VecBrand {
		/// Maps a function over the vector in parallel.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential mapping.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each element. Must be `Send + Sync`.",
			"The vector to map over."
		)]
		///
		#[document_returns("A new vector containing the mapped elements.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let result = VecBrand::par_map(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(result, vec![2, 4, 6]);
		/// ```
		pub fn par_map<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Vec<A>,
		) -> Vec<B> {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.into_par_iter().map(f).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.into_iter().map(f).collect()
		}

		/// Compacts a vector of options in parallel, discarding `None` values.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential compaction.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters("The vector of options.")]
		///
		#[document_returns("A new vector containing the unwrapped `Some` values.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let result = VecBrand::par_compact(vec![Some(1), None, Some(3)]);
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		pub fn par_compact<'a, A: 'a + Send>(fa: Vec<Option<A>>) -> Vec<A> {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.into_par_iter().flatten().collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.into_iter().flatten().collect()
		}

		/// Separates a vector of results into `(errors, oks)` in parallel.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential separation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The error type.",
			"The success type."
		)]
		///
		#[document_parameters("The vector of results.")]
		///
		#[document_returns(
			"A pair `(errs, oks)` where `errs` contains the `Err` values and `oks` the `Ok` values."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let v: Vec<Result<i32, &str>> = vec![Ok(1), Err("a"), Ok(3)];
		/// let (errs, oks): (Vec<&str>, Vec<i32>) = VecBrand::par_separate(v);
		/// assert_eq!(errs, vec!["a"]);
		/// assert_eq!(oks, vec![1, 3]);
		/// ```
		pub fn par_separate<'a, E: 'a + Send, O: 'a + Send>(
			fa: Vec<Result<O, E>>
		) -> (Vec<E>, Vec<O>) {
			#[cfg(feature = "rayon")]
			{
				use rayon::{
					iter::Either,
					prelude::*,
				};
				fa.into_par_iter().partition_map(|r| match r {
					Ok(o) => Either::Right(o),
					Err(e) => Either::Left(e),
				})
			}
			#[cfg(not(feature = "rayon"))]
			{
				let mut errs = Vec::new();
				let mut oks = Vec::new();
				for result in fa {
					match result {
						Ok(o) => oks.push(o),
						Err(e) => errs.push(e),
					}
				}
				(errs, oks)
			}
		}

		/// Maps and filters a vector in parallel, discarding elements where `f` returns `None`.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential filter-mapping.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply. Must be `Send + Sync`.",
			"The vector to filter and map."
		)]
		///
		#[document_returns("A new vector containing the `Some` results of applying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let result = VecBrand::par_filter_map(
		/// 	|x: i32| if x % 2 == 0 { Some(x * 10) } else { None },
		/// 	vec![1, 2, 3, 4, 5],
		/// );
		/// assert_eq!(result, vec![20, 40]);
		/// ```
		pub fn par_filter_map<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(A) -> Option<B> + Send + Sync + 'a,
			fa: Vec<A>,
		) -> Vec<B> {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.into_par_iter().filter_map(f).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.into_iter().filter_map(f).collect()
		}

		/// Filters a vector in parallel, retaining only elements satisfying `f`.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential filtering.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters("The predicate. Must be `Send + Sync`.", "The vector to filter.")]
		///
		#[document_returns("A new vector containing only the elements satisfying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let result = VecBrand::par_filter(|x: &i32| x % 2 == 0, vec![1, 2, 3, 4, 5]);
		/// assert_eq!(result, vec![2, 4]);
		/// ```
		pub fn par_filter<'a, A: 'a + Send>(
			f: impl Fn(&A) -> bool + Send + Sync + 'a,
			fa: Vec<A>,
		) -> Vec<A> {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.into_par_iter().filter(|a| f(a)).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.into_iter().filter(|a| f(a)).collect()
		}

		/// Maps each element to a [`Monoid`] value and combines them in parallel.
		///
		/// When the `rayon` feature is enabled, mapping and reduction happen across multiple threads.
		/// Otherwise falls back to sequential fold-map.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping each element to a monoid value. Must be `Send + Sync`.",
			"The vector to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let result = VecBrand::par_fold_map(|x: i32| x.to_string(), vec![1, 2, 3]);
		/// assert_eq!(result, "123");
		/// ```
		pub fn par_fold_map<'a, A: 'a + Send, M: Monoid + Send + 'a>(
			f: impl Fn(A) -> M + Send + Sync + 'a,
			fa: Vec<A>,
		) -> M {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.into_par_iter().map(f).reduce(M::empty, |acc, m| M::append(acc, m))
			}
			#[cfg(not(feature = "rayon"))]
			fa.into_iter().map(f).fold(M::empty(), |acc, m| M::append(acc, m))
		}

		/// Maps a function over the vector in parallel, providing each element's index.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential indexed mapping.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each index and element. Must be `Send + Sync`.",
			"The vector to map over."
		)]
		///
		#[document_returns("A new vector containing the mapped elements.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let result = VecBrand::par_map_with_index(|i, x: i32| x + i as i32, vec![10, 20, 30]);
		/// assert_eq!(result, vec![10, 21, 32]);
		/// ```
		pub fn par_map_with_index<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(usize, A) -> B + Send + Sync + 'a,
			fa: Vec<A>,
		) -> Vec<B> {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.into_par_iter().enumerate().map(|(i, a)| f(i, a)).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.into_iter().enumerate().map(|(i, a)| f(i, a)).collect()
		}

		/// Maps each element and its index to a [`Monoid`] value and combines them in parallel.
		///
		/// When the `rayon` feature is enabled, mapping and reduction happen across multiple threads.
		/// Otherwise falls back to sequential indexed fold-map.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping each index and element to a monoid value. Must be `Send + Sync`.",
			"The vector to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let result =
		/// 	VecBrand::par_fold_map_with_index(|i, x: i32| format!("{i}:{x}"), vec![10, 20, 30]);
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		pub fn par_fold_map_with_index<'a, A: 'a + Send, M: Monoid + Send + 'a>(
			f: impl Fn(usize, A) -> M + Send + Sync + 'a,
			fa: Vec<A>,
		) -> M {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.into_par_iter()
					.enumerate()
					.map(|(i, a)| f(i, a))
					.reduce(M::empty, |acc, m| M::append(acc, m))
			}
			#[cfg(not(feature = "rayon"))]
			fa.into_iter()
				.enumerate()
				.map(|(i, a)| f(i, a))
				.fold(M::empty(), |acc, m| M::append(acc, m))
		}

		/// Maps and filters a vector in parallel with the index, discarding elements where
		/// `f` returns `None`.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential indexed filter-map.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each index and element. Must be `Send + Sync`.",
			"The vector to filter and map."
		)]
		///
		#[document_returns("A new vector containing the `Some` results of applying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let result = VecBrand::par_filter_map_with_index(
		/// 	|i, x: i32| if i < 3 { Some(x * 10) } else { None },
		/// 	vec![1, 2, 3, 4, 5],
		/// );
		/// assert_eq!(result, vec![10, 20, 30]);
		/// ```
		pub fn par_filter_map_with_index<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(usize, A) -> Option<B> + Send + Sync + 'a,
			fa: Vec<A>,
		) -> Vec<B> {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.into_par_iter().enumerate().filter_map(|(i, a)| f(i, a)).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.into_iter().enumerate().filter_map(|(i, a)| f(i, a)).collect()
		}

		/// Filters a vector in parallel with the index, retaining only elements satisfying `f`.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential indexed filtering.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters(
			"The predicate receiving the index and a reference to the element. Must be `Send + Sync`.",
			"The vector to filter."
		)]
		///
		#[document_returns("A new vector containing only the elements satisfying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::brands::VecBrand;
		///
		/// let result =
		/// 	VecBrand::par_filter_with_index(|i, x: &i32| i < 3 && x % 2 != 0, vec![1, 2, 3, 4, 5]);
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		pub fn par_filter_with_index<'a, A: 'a + Send>(
			f: impl Fn(usize, &A) -> bool + Send + Sync + 'a,
			fa: Vec<A>,
		) -> Vec<A> {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.into_par_iter().enumerate().filter(|(i, a)| f(*i, a)).map(|(_, a)| a).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.into_iter().enumerate().filter(|(i, a)| f(*i, a)).map(|(_, a)| a).collect()
		}
	}

	impl ParFunctor for VecBrand {
		/// Maps a function over the vector in parallel.
		///
		/// Delegates to [`VecBrand::par_map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each element. Must be `Send + Sync`.",
			"The vector to map over."
		)]
		///
		#[document_returns("A new vector containing the mapped elements.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_functor::ParFunctor,
		/// };
		///
		/// let result = VecBrand::par_map(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(result, vec![2, 4, 6]);
		/// ```
		fn par_map<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			VecBrand::par_map(f, fa)
		}
	}

	impl ParCompactable for VecBrand {
		/// Compacts a vector of options in parallel, discarding `None` values.
		///
		/// Delegates to [`VecBrand::par_compact`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters("The vector of options.")]
		///
		#[document_returns("A new vector containing the unwrapped `Some` values.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_compactable::ParCompactable,
		/// };
		///
		/// let result = VecBrand::par_compact(vec![Some(1), None, Some(3)]);
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		fn par_compact<'a, A: 'a + Send>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			VecBrand::par_compact(fa)
		}

		/// Separates a vector of results into `(errors, oks)` in parallel.
		///
		/// Delegates to [`VecBrand::par_separate`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The error type.",
			"The success type."
		)]
		///
		#[document_parameters("The vector of results.")]
		///
		#[document_returns(
			"A pair `(errs, oks)` where `errs` contains the `Err` values and `oks` the `Ok` values."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_compactable::ParCompactable,
		/// };
		///
		/// let v: Vec<Result<i32, &str>> = vec![Ok(1), Err("a"), Ok(3)];
		/// let (errs, oks): (Vec<&str>, Vec<i32>) = VecBrand::par_separate(v);
		/// assert_eq!(errs, vec!["a"]);
		/// assert_eq!(oks, vec![1, 3]);
		/// ```
		fn par_separate<'a, E: 'a + Send, O: 'a + Send>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			VecBrand::par_separate(fa)
		}
	}

	impl ParFilterable for VecBrand {
		/// Maps and filters a vector in parallel, discarding elements where `f` returns `None`.
		///
		/// Single-pass implementation using rayon's `filter_map`. Delegates to
		/// [`VecBrand::par_filter_map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply. Must be `Send + Sync`.",
			"The vector to filter and map."
		)]
		///
		#[document_returns("A new vector containing the `Some` results of applying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_filterable::ParFilterable,
		/// };
		///
		/// let result = VecBrand::par_filter_map(
		/// 	|x: i32| if x % 2 == 0 { Some(x * 10) } else { None },
		/// 	vec![1, 2, 3, 4, 5],
		/// );
		/// assert_eq!(result, vec![20, 40]);
		/// ```
		fn par_filter_map<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(A) -> Option<B> + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			VecBrand::par_filter_map(f, fa)
		}

		/// Filters a vector in parallel, retaining only elements satisfying `f`.
		///
		/// Single-pass implementation using rayon's `filter`. Delegates to
		/// [`VecBrand::par_filter`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters("The predicate. Must be `Send + Sync`.", "The vector to filter.")]
		///
		#[document_returns("A new vector containing only the elements satisfying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_filterable::ParFilterable,
		/// };
		///
		/// let result = VecBrand::par_filter(|x: &i32| x % 2 == 0, vec![1, 2, 3, 4, 5]);
		/// assert_eq!(result, vec![2, 4]);
		/// ```
		fn par_filter<'a, A: 'a + Send>(
			f: impl Fn(&A) -> bool + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			VecBrand::par_filter(f, fa)
		}
	}

	impl ParFoldable for VecBrand {
		/// Maps each element to a [`Monoid`] value and combines them in parallel.
		///
		/// Delegates to [`VecBrand::par_fold_map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping each element to a monoid value. Must be `Send + Sync`.",
			"The vector to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_foldable::ParFoldable,
		/// };
		///
		/// let result = VecBrand::par_fold_map(|x: i32| x.to_string(), vec![1, 2, 3]);
		/// assert_eq!(result, "123");
		/// ```
		fn par_fold_map<'a, A: 'a + Send, M: Monoid + Send + 'a>(
			f: impl Fn(A) -> M + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			VecBrand::par_fold_map(f, fa)
		}
	}

	impl ParFunctorWithIndex for VecBrand {
		/// Maps a function over the vector in parallel, providing each element's index.
		///
		/// Delegates to [`VecBrand::par_map_with_index`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each index and element. Must be `Send + Sync`.",
			"The vector to map over."
		)]
		///
		#[document_returns("A new vector containing the mapped elements.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_functor_with_index::ParFunctorWithIndex,
		/// };
		///
		/// let result = VecBrand::par_map_with_index(|i, x: i32| x + i as i32, vec![10, 20, 30]);
		/// assert_eq!(result, vec![10, 21, 32]);
		/// ```
		fn par_map_with_index<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(usize, A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			usize: Send + Sync + Copy + 'a, {
			VecBrand::par_map_with_index(f, fa)
		}
	}

	impl ParFoldableWithIndex for VecBrand {
		/// Maps each element and its index to a [`Monoid`] value and combines them in parallel.
		///
		/// Delegates to [`VecBrand::par_fold_map_with_index`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping each index and element to a monoid value. Must be `Send + Sync`.",
			"The vector to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_foldable_with_index::ParFoldableWithIndex,
		/// };
		///
		/// let result =
		/// 	VecBrand::par_fold_map_with_index(|i, x: i32| format!("{i}:{x}"), vec![10, 20, 30]);
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn par_fold_map_with_index<'a, A: 'a + Send, M: Monoid + Send + 'a>(
			f: impl Fn(usize, A) -> M + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			usize: Send + Sync + Copy + 'a, {
			VecBrand::par_fold_map_with_index(f, fa)
		}
	}

	impl Compactable for VecBrand {
		/// Compacts a vector of options.
		///
		/// This method flattens a vector of options, discarding `None` values.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The vector of options.")]
		///
		#[document_returns("The flattened vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		///
		/// let x = vec![Some(1), None, Some(2)];
		/// let y = compact::<VecBrand, _>(x);
		/// assert_eq!(y, vec![1, 2]);
		/// ```
		fn compact<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.into_iter().flatten().collect()
		}

		/// Separates a vector of results.
		///
		/// This method separates a vector of results into a pair of vectors.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters("The vector of results.")]
		///
		#[document_returns("A pair of vectors.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = vec![Ok(1), Err("error"), Ok(2)];
		/// let (errs, oks) = separate::<VecBrand, _, _>(x);
		/// assert_eq!(oks, vec![1, 2]);
		/// assert_eq!(errs, vec!["error"]);
		/// ```
		fn separate<'a, E: 'a, O: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			let mut oks = Vec::new();
			let mut errs = Vec::new();
			for result in fa {
				match result {
					Ok(o) => oks.push(o),
					Err(e) => errs.push(e),
				}
			}
			(errs, oks)
		}
	}

	impl RefCompactable for VecBrand {
		/// Compacts a borrowed vector of options, discarding [`None`] values and cloning [`Some`] values.
		///
		/// This method iterates over a borrowed vector of options, keeping only the [`Some`] values
		/// by cloning them into a new vector.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the [`Option`]. Must be [`Clone`] because elements are extracted from a borrowed container."
		)]
		///
		#[document_parameters("A reference to the vector containing [`Option`] values.")]
		///
		#[document_returns(
			"A new vector containing only the cloned values from the [`Some`] variants."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![Some(1), None, Some(3)];
		/// let result = ref_compact::<VecBrand, _>(&v);
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		fn ref_compact<'a, A: 'a + Clone>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<A>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.iter().filter_map(|opt| opt.as_ref().cloned()).collect()
		}

		/// Separates a borrowed vector of results into two vectors: one containing the cloned [`Err`] values and one containing the cloned [`Ok`] values.
		///
		/// This method iterates over a borrowed vector of results, cloning each value into the
		/// appropriate output vector.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the error values. Must be [`Clone`] because elements are extracted from a borrowed container.",
			"The type of the success values. Must be [`Clone`] because elements are extracted from a borrowed container."
		)]
		///
		#[document_parameters("A reference to the vector containing [`Result`] values.")]
		///
		#[document_returns(
			"A pair of vectors: the first containing the cloned [`Err`] values, and the second containing the cloned [`Ok`] values."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		///
		/// let v: Vec<Result<i32, &str>> = vec![Ok(1), Err("bad"), Ok(3)];
		/// let (errs, oks) = ref_separate::<VecBrand, _, _>(&v);
		/// assert_eq!(oks, vec![1, 3]);
		/// assert_eq!(errs, vec!["bad"]);
		/// ```
		fn ref_separate<'a, E: 'a + Clone, O: 'a + Clone>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			let mut errs = Vec::new();
			let mut oks = Vec::new();
			for result in fa.iter() {
				match result {
					Ok(o) => oks.push(o.clone()),
					Err(e) => errs.push(e.clone()),
				}
			}
			(errs, oks)
		}
	}

	impl Filterable for VecBrand {
		/// Partitions a vector based on a function that returns a result.
		///
		/// This method partitions a vector based on a function that returns a result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input value.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters("The function to apply.", "The vector to partition.")]
		///
		#[document_returns("A pair of vectors.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = vec![1, 2, 3, 4];
		/// let (errs, oks) =
		/// 	partition_map::<VecBrand, _, _, _, _, _>(|a| if a % 2 == 0 { Ok(a) } else { Err(a) }, x);
		/// assert_eq!(oks, vec![2, 4]);
		/// assert_eq!(errs, vec![1, 3]);
		/// ```
		fn partition_map<'a, A: 'a, E: 'a, O: 'a>(
			func: impl Fn(A) -> Result<O, E> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			let mut oks = Vec::new();
			let mut errs = Vec::new();
			for a in fa {
				match func(a) {
					Ok(o) => oks.push(o),
					Err(e) => errs.push(e),
				}
			}
			(errs, oks)
		}

		/// Partitions a vector based on a predicate.
		///
		/// This method partitions a vector based on a predicate.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The predicate.", "The vector to partition.")]
		///
		#[document_returns("A pair of vectors.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = vec![1, 2, 3, 4];
		/// let (not_satisfied, satisfied) = partition::<VecBrand, _, _, _>(|a| a % 2 == 0, x);
		/// assert_eq!(satisfied, vec![2, 4]);
		/// assert_eq!(not_satisfied, vec![1, 3]);
		/// ```
		fn partition<'a, A: 'a + Clone>(
			func: impl Fn(A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			let (satisfied, not_satisfied): (Vec<A>, Vec<A>) =
				fa.into_iter().partition(|a| func(a.clone()));
			(not_satisfied, satisfied)
		}

		/// Maps a function over a vector and filters out `None` results.
		///
		/// This method maps a function over a vector and filters out `None` results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply.", "The vector to filter and map.")]
		///
		#[document_returns("The filtered and mapped vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		///
		/// let x = vec![1, 2, 3, 4];
		/// let y = filter_map::<VecBrand, _, _, _, _>(|a| if a % 2 == 0 { Some(a * 2) } else { None }, x);
		/// assert_eq!(y, vec![4, 8]);
		/// ```
		fn filter_map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> Option<B> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.into_iter().filter_map(func).collect()
		}

		/// Filters a vector based on a predicate.
		///
		/// This method filters a vector based on a predicate.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The predicate.", "The vector to filter.")]
		///
		#[document_returns("The filtered vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		///
		/// let x = vec![1, 2, 3, 4];
		/// let y = filter::<VecBrand, _, _, _>(|a| a % 2 == 0, x);
		/// assert_eq!(y, vec![2, 4]);
		/// ```
		fn filter<'a, A: 'a + Clone>(
			func: impl Fn(A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			fa.into_iter().filter(|a| func(a.clone())).collect()
		}
	}

	impl FilterableWithIndex for VecBrand {
		/// Partitions a vector based on a function that receives the index and returns a [`Result`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input value.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The vector to partition."
		)]
		///
		#[document_returns("A pair of vectors.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let xs = vec![1, 2, 3, 4];
		/// let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
		/// 	|i, a: i32| if i < 2 { Ok(a) } else { Err(a) },
		/// 	xs,
		/// );
		/// assert_eq!(oks, vec![1, 2]);
		/// assert_eq!(errs, vec![3, 4]);
		/// ```
		fn partition_map_with_index<'a, A: 'a, E: 'a, O: 'a>(
			func: impl Fn(usize, A) -> Result<O, E> + 'a,
			fa: Vec<A>,
		) -> (Vec<E>, Vec<O>) {
			let mut oks = Vec::new();
			let mut errs = Vec::new();
			for (i, a) in fa.into_iter().enumerate() {
				match func(i, a) {
					Ok(o) => oks.push(o),
					Err(e) => errs.push(e),
				}
			}
			(errs, oks)
		}

		/// Partitions a vector based on a predicate that receives the index.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters(
			"The predicate receiving the index and element.",
			"The vector to partition."
		)]
		///
		#[document_returns("A pair of vectors.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let xs = vec![1, 2, 3, 4];
		/// let (not_satisfied, satisfied) =
		/// 	partition_with_index::<VecBrand, _, _, _>(|i, _a: i32| i < 2, xs);
		/// assert_eq!(satisfied, vec![1, 2]);
		/// assert_eq!(not_satisfied, vec![3, 4]);
		/// ```
		fn partition_with_index<'a, A: 'a + Clone>(
			func: impl Fn(usize, A) -> bool + 'a,
			fa: Vec<A>,
		) -> (Vec<A>, Vec<A>) {
			let mut satisfied = Vec::new();
			let mut not_satisfied = Vec::new();
			for (i, a) in fa.into_iter().enumerate() {
				if func(i, a.clone()) {
					satisfied.push(a);
				} else {
					not_satisfied.push(a);
				}
			}
			(not_satisfied, satisfied)
		}

		/// Maps a function over a vector with the index and filters out [`None`] results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The vector to filter and map."
		)]
		///
		#[document_returns("The filtered and mapped vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		///
		/// let xs = vec![1, 2, 3, 4];
		/// let result = filter_map_with_index::<VecBrand, _, _, _, _>(
		/// 	|i, a: i32| if i % 2 == 0 { Some(a * 2) } else { None },
		/// 	xs,
		/// );
		/// assert_eq!(result, vec![2, 6]);
		/// ```
		fn filter_map_with_index<'a, A: 'a, B: 'a>(
			func: impl Fn(usize, A) -> Option<B> + 'a,
			fa: Vec<A>,
		) -> Vec<B> {
			fa.into_iter().enumerate().filter_map(|(i, a)| func(i, a)).collect()
		}

		/// Filters a vector based on a predicate that receives the index.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters(
			"The predicate receiving the index and element.",
			"The vector to filter."
		)]
		///
		#[document_returns("The filtered vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	functions::*,
		/// };
		///
		/// let xs = vec![1, 2, 3, 4];
		/// let result = filter_with_index::<VecBrand, _, _, _>(|i, _a: i32| i < 2, xs);
		/// assert_eq!(result, vec![1, 2]);
		/// ```
		fn filter_with_index<'a, A: 'a + Clone>(
			func: impl Fn(usize, A) -> bool + 'a,
			fa: Vec<A>,
		) -> Vec<A> {
			fa.into_iter()
				.enumerate()
				.filter(|(i, a)| func(*i, a.clone()))
				.map(|(_, a)| a)
				.collect()
		}
	}

	impl ParFilterableWithIndex for VecBrand {
		/// Maps and filters a vector in parallel with the index, discarding elements where
		/// `f` returns `None`.
		///
		/// Single-pass implementation using rayon's `enumerate` + `filter_map`. Delegates to
		/// [`VecBrand::par_filter_map_with_index`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each index and element. Must be `Send + Sync`.",
			"The vector to filter and map."
		)]
		///
		#[document_returns("A new vector containing the `Some` results of applying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_filterable_with_index::ParFilterableWithIndex,
		/// };
		///
		/// let result = VecBrand::par_filter_map_with_index(
		/// 	|i, x: i32| if i < 3 { Some(x * 10) } else { None },
		/// 	vec![1, 2, 3, 4, 5],
		/// );
		/// assert_eq!(result, vec![10, 20, 30]);
		/// ```
		fn par_filter_map_with_index<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(usize, A) -> Option<B> + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			usize: Send + Sync + Copy + 'a, {
			VecBrand::par_filter_map_with_index(f, fa)
		}

		/// Filters a vector in parallel with the index, retaining only elements satisfying `f`.
		///
		/// Single-pass implementation using rayon's `enumerate` + `filter`. Delegates to
		/// [`VecBrand::par_filter_with_index`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters(
			"The predicate receiving the index and a reference to the element. Must be `Send + Sync`.",
			"The vector to filter."
		)]
		///
		#[document_returns("A new vector containing only the elements satisfying `f`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_filterable_with_index::ParFilterableWithIndex,
		/// };
		///
		/// let result =
		/// 	VecBrand::par_filter_with_index(|i, x: &i32| i < 3 && x % 2 != 0, vec![1, 2, 3, 4, 5]);
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		fn par_filter_with_index<'a, A: 'a + Send>(
			f: impl Fn(usize, &A) -> bool + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			usize: Send + Sync + Copy + 'a, {
			VecBrand::par_filter_with_index(f, fa)
		}
	}

	impl Witherable for VecBrand {
		/// Partitions a vector based on a function that returns a result in an applicative context.
		///
		/// This method partitions a vector based on a function that returns a result in an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The applicative context.",
			"The type of the input value.",
			"The type of the error value.",
			"The type of the success value."
		)]
		///
		#[document_parameters("The function to apply.", "The vector to partition.")]
		///
		#[document_returns("The partitioned vector wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = vec![1, 2, 3, 4];
		/// let y = wilt::<VecBrand, OptionBrand, _, _, _>(
		/// 	|a| Some(if a % 2 == 0 { Ok(a) } else { Err(a) }),
		/// 	x,
		/// );
		/// assert_eq!(y, Some((vec![1, 3], vec![2, 4])));
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
			ta.into_iter().fold(M::pure((Vec::new(), Vec::new())), |acc, x| {
				M::lift2(
					|mut pair, res| {
						match res {
							Ok(o) => pair.1.push(o),
							Err(e) => pair.0.push(e),
						}
						pair
					},
					acc,
					func(x),
				)
			})
		}

		/// Maps a function over a vector and filters out `None` results in an applicative context.
		///
		/// This method maps a function over a vector and filters out `None` results in an applicative context.
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
			"The vector to filter and map."
		)]
		///
		#[document_returns("The filtered and mapped vector wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		VecBrand,
		/// 	},
		/// 	functions::*,
		/// };
		///
		/// let x = vec![1, 2, 3, 4];
		/// let y = wither::<VecBrand, OptionBrand, _, _>(
		/// 	|a| Some(if a % 2 == 0 { Some(a * 2) } else { None }),
		/// 	x,
		/// );
		/// assert_eq!(y, Some(vec![4, 8]));
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
			ta.into_iter().fold(M::pure(Vec::new()), |acc, x| {
				M::lift2(
					|mut v, opt_b| {
						if let Some(b) = opt_b {
							v.push(b);
						}
						v
					},
					acc,
					func(x),
				)
			})
		}
	}

	/// Cooperative extension for [`Vec`], ported from PureScript's `Extend Array` instance.
	///
	/// `extend(f, vec)` produces a new vector where each element at index `i` is
	/// `f` applied to the suffix `vec[i..]`. This is the dual of [`Semimonad::bind`]:
	/// where `bind` feeds each element into a function that produces a new context,
	/// `extend` feeds each suffix (context) into a function that produces a single value.
	///
	/// Requires `A: Clone` because suffixes are materialized as owned vectors.
	impl Extend for VecBrand {
		/// Extends a local context-dependent computation to a global computation over
		/// [`Vec`].
		///
		/// Applies `f` to every suffix of the input vector. For a vector
		/// `[a, b, c]`, the result is `[f([a, b, c]), f([b, c]), f([c])]`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the vector.",
			"The result type of the extension function."
		)]
		///
		#[document_parameters(
			"The function that consumes a suffix vector and produces a value.",
			"The vector to extend over."
		)]
		///
		#[document_returns(
			"A new vector containing the results of applying the function to each suffix."
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
		/// let result = extend::<VecBrand, _, _>(|v: Vec<i32>| v.iter().sum::<i32>(), vec![1, 2, 3]);
		/// assert_eq!(result, vec![6, 5, 3]);
		/// ```
		fn extend<'a, A: 'a + Clone, B: 'a>(
			f: impl Fn(Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> B + 'a,
			wa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(0 .. wa.len()).map(|i| f(wa.get(i ..).unwrap_or_default().to_vec())).collect()
		}
	}

	impl MonadRec for VecBrand {
		/// Performs tail-recursive monadic computation over [`Vec`].
		///
		/// Since `Vec` represents nondeterminism, this performs a breadth-first
		/// expansion: each iteration maps all current `Loop` states through the step
		/// function, collecting `Done` results as they appear. The computation
		/// terminates when no `Loop` values remain.
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
		#[document_returns("A vector of all completed results.")]
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
		/// // Branch into two paths, each running until done
		/// let result = tail_rec_m::<VecBrand, _, _>(
		/// 	|n| {
		/// 		if n < 3 {
		/// 			vec![ControlFlow::Continue(n + 1), ControlFlow::Break(n * 10)]
		/// 		} else {
		/// 			vec![ControlFlow::Break(n * 10)]
		/// 		}
		/// 	},
		/// 	0,
		/// );
		/// // Starting from 0: branches at 0,1,2; done at 3
		/// assert_eq!(result, vec![0, 10, 20, 30]);
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut done: Vec<B> = Vec::new();
			let mut pending: Vec<A> = vec![initial];
			while !pending.is_empty() {
				let mut next_pending: Vec<A> = Vec::new();
				for a in pending {
					for step in func(a) {
						match step {
							ControlFlow::Continue(next) => next_pending.push(next),
							ControlFlow::Break(b) => done.push(b),
						}
					}
				}
				pending = next_pending;
			}
			done
		}
	}

	// -- By-reference trait implementations --

	impl RefFunctor for VecBrand {
		/// Maps a function over the vector by reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the vector.",
			"The type of the elements in the resulting vector."
		)]
		///
		#[document_parameters(
			"The function to apply to each element reference.",
			"The vector to map over."
		)]
		///
		#[document_returns("A new vector containing the results.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(map::<VecBrand, _, _, _, _>(|x: &i32| *x * 2, &vec![1, 2, 3]), vec![2, 4, 6]);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().map(func).collect()
		}
	}

	impl RefFoldable for VecBrand {
		/// Folds the vector by reference using a monoid.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function wrapper.",
			"The type of the elements.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function to map each element reference to a monoid.",
			"The vector to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3];
		/// let result = fold_map::<RcFnBrand, VecBrand, _, _, _, _>(|x: &i32| x.to_string(), &v);
		/// assert_eq!(result, "123");
		/// ```
		fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(&A) -> M + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: LiftFn + 'a,
			M: Monoid + 'a, {
			fa.iter().fold(Monoid::empty(), |acc, a| Semigroup::append(acc, func(a)))
		}
	}

	impl RefFilterable for VecBrand {
		/// Filters and maps the vector by reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements."
		)]
		///
		#[document_parameters("The filter-map function.", "The vector to filter.")]
		///
		#[document_returns("The filtered vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result =
		/// 	ref_filter_map::<VecBrand, _, _>(|x: &i32| if *x > 3 { Some(*x) } else { None }, &v);
		/// assert_eq!(result, vec![4, 5]);
		/// ```
		fn ref_filter_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> Option<B> + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().filter_map(func).collect()
		}
	}

	impl RefTraversable for VecBrand {
		/// Traverses the vector by reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function wrapper.",
			"The type of the input elements.",
			"The type of the output elements.",
			"The applicative functor brand."
		)]
		///
		#[document_parameters(
			"The function to apply to each element reference.",
			"The vector to traverse."
		)]
		///
		#[document_returns("The combined result in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3];
		/// let result: Option<Vec<String>> =
		/// 	ref_traverse::<VecBrand, RcFnBrand, _, _, OptionBrand>(|x: &i32| Some(x.to_string()), &v);
		/// assert_eq!(result, Some(vec!["1".to_string(), "2".to_string(), "3".to_string()]));
		/// ```
		fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let len = ta.len();
			ta.iter().fold(
				F::pure::<Vec<B>>(Vec::with_capacity(len)),
				|acc: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Vec<B>>), a| {
					F::lift2(
						|mut v: Vec<B>, b: B| {
							v.push(b);
							v
						},
						acc,
						func(a),
					)
				},
			)
		}
	}

	impl RefWitherable for VecBrand {}

	impl RefFunctorWithIndex for VecBrand {
		/// Maps a function with index over the vector by reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements."
		)]
		///
		#[document_parameters("The function to apply.", "The vector to map over.")]
		///
		#[document_returns("The mapped vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30];
		/// let result = ref_map_with_index::<VecBrand, _, _>(|i, x: &i32| format!("{}:{}", i, x), &v);
		/// assert_eq!(result, vec!["0:10", "1:20", "2:30"]);
		/// ```
		fn ref_map_with_index<'a, A: 'a, B: 'a>(
			func: impl Fn(usize, &A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().enumerate().map(|(i, a)| func(i, a)).collect()
		}
	}

	impl RefFoldableWithIndex for VecBrand {
		/// Folds the vector by reference with index using a monoid.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function to map each (index, element reference) pair.",
			"The vector to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30];
		/// let result = ref_fold_map_with_index::<RcFnBrand, VecBrand, _, _>(
		/// 	|i, x: &i32| format!("{}:{}", i, x),
		/// 	&v,
		/// );
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn ref_fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid + 'a>(
			func: impl Fn(usize, &A) -> R + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			fa.iter()
				.enumerate()
				.fold(Monoid::empty(), |acc, (i, a)| Semigroup::append(acc, func(i, a)))
		}
	}

	impl RefFilterableWithIndex for VecBrand {
		/// Filters and maps the vector by reference with index.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements."
		)]
		///
		#[document_parameters("The filter-map function.", "The vector to filter.")]
		///
		#[document_returns("The filtered vector.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30, 40, 50];
		/// let result = ref_filter_map_with_index::<VecBrand, _, _>(
		/// 	|i, x: &i32| if i >= 2 { Some(*x) } else { None },
		/// 	&v,
		/// );
		/// assert_eq!(result, vec![30, 40, 50]);
		/// ```
		fn ref_filter_map_with_index<'a, A: 'a, B: 'a>(
			func: impl Fn(usize, &A) -> Option<B> + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().enumerate().filter_map(|(i, a)| func(i, a)).collect()
		}
	}

	impl RefTraversableWithIndex for VecBrand {
		/// Traverses the vector by reference with index.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements.",
			"The applicative functor brand."
		)]
		///
		#[document_parameters("The function to apply.", "The vector to traverse.")]
		///
		#[document_returns("The combined result in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30];
		/// let result: Option<Vec<String>> = ref_traverse_with_index::<VecBrand, _, _, OptionBrand>(
		/// 	|i, x: &i32| Some(format!("{}:{}", i, x)),
		/// 	&v,
		/// );
		/// assert_eq!(result, Some(vec!["0:10".to_string(), "1:20".to_string(), "2:30".to_string()]));
		/// ```
		fn ref_traverse_with_index<'a, A: 'a + Clone, B: 'a + Clone, M: Applicative>(
			f: impl Fn(usize, &A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let len = ta.len();
			ta.iter().enumerate().fold(
				M::pure::<Vec<B>>(Vec::with_capacity(len)),
				|acc: Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Vec<B>>), (i, a)| {
					M::lift2(
						|mut v: Vec<B>, b: B| {
							v.push(b);
							v
						},
						acc,
						f(i, a),
					)
				},
			)
		}
	}

	// -- By-reference monadic trait implementations --

	impl RefPointed for VecBrand {
		/// Creates a singleton vector from a reference by cloning.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value.")]
		///
		#[document_parameters("The reference to the value to wrap.")]
		///
		#[document_returns("A singleton vector containing a clone of the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = 42;
		/// let v: Vec<i32> = ref_pure::<VecBrand, _>(&x);
		/// assert_eq!(v, vec![42]);
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			vec![a.clone()]
		}
	}

	impl RefLift for VecBrand {
		/// Combines two vectors with a by-reference binary function (Cartesian product).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first input.",
			"The type of the second input.",
			"The type of the output."
		)]
		///
		#[document_parameters(
			"The binary function receiving references.",
			"The first vector.",
			"The second vector."
		)]
		///
		#[document_returns("A new vector with the combined results.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = lift2::<VecBrand, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &String| format!("{}{}", a, b),
		/// 	&vec![1, 2],
		/// 	&vec!["a".to_string(), "b".to_string()],
		/// );
		/// assert_eq!(v, vec!["1a".to_string(), "1b".to_string(), "2a".to_string(), "2b".to_string()]);
		/// ```
		fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
			func: impl Fn(&A, &B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			let func = &func;
			fa.iter().flat_map(|a| fb.iter().map(move |b| func(a, b))).collect()
		}
	}

	impl RefSemiapplicative for VecBrand {
		/// Applies wrapped by-ref functions to values (Cartesian product).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input values.",
			"The type of the output values."
		)]
		///
		#[document_parameters(
			"The vector containing the by-ref functions.",
			"The vector containing the values."
		)]
		///
		#[document_returns("A new vector with each function applied to each value by reference.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f1: std::rc::Rc<dyn Fn(&i32) -> i32> = std::rc::Rc::new(|x: &i32| *x + 1);
		/// let f2: std::rc::Rc<dyn Fn(&i32) -> i32> = std::rc::Rc::new(|x: &i32| *x * 2);
		/// let result = ref_apply::<RcFnBrand, VecBrand, _, _>(&vec![f1, f2], &vec![10, 20]);
		/// assert_eq!(result, vec![11, 21, 20, 40]);
		/// ```
		fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, A: 'a, B: 'a>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ff.iter().flat_map(|f| fa.iter().map(move |a| (**f)(a))).collect()
		}
	}

	impl RefSemimonad for VecBrand {
		/// Chains vector computations by reference (`flat_map` with `&A`).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the input values.",
			"The type of the output values."
		)]
		///
		#[document_parameters(
			"The input vector.",
			"The function to apply to each element by reference."
		)]
		///
		#[document_returns("A new vector with the results flattened.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3];
		/// let result: Vec<i32> = bind::<VecBrand, _, _, _, _>(&v, |x: &i32| vec![*x, *x * 10]);
		/// assert_eq!(result, vec![1, 10, 2, 20, 3, 30]);
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.iter().flat_map(f).collect()
		}
	}

	// -- Parallel by-reference trait implementations --

	impl ParRefFunctor for VecBrand {
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The input element type.",
			"The output element type."
		)]
		#[document_parameters("The function. Must be `Send + Sync`.", "The vector.")]
		#[document_returns("A new vector with mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_functor::ParRefFunctor,
		/// };
		/// let result = VecBrand::par_ref_map(|x: &i32| x * 2, &vec![1, 2, 3]);
		/// assert_eq!(result, vec![2, 4, 6]);
		/// ```
		fn par_ref_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(&A) -> B + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.par_iter().map(f).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.iter().map(f).collect()
		}
	}

	impl ParRefFoldable for VecBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The element type.", "The monoid type.")]
		#[document_parameters("The function. Must be `Send + Sync`.", "The vector.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_foldable::ParRefFoldable,
		/// };
		/// let result = VecBrand::par_ref_fold_map(|x: &i32| x.to_string(), &vec![1, 2, 3]);
		/// assert_eq!(result, "123");
		/// ```
		fn par_ref_fold_map<'a, A: Send + Sync + 'a, M: Monoid + Send + 'a>(
			f: impl Fn(&A) -> M + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.par_iter().map(f).reduce(Monoid::empty, Semigroup::append)
			}
			#[cfg(not(feature = "rayon"))]
			fa.iter().map(f).fold(Monoid::empty(), |acc, m| Semigroup::append(acc, m))
		}
	}

	impl ParRefFilterable for VecBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function. Must be `Send + Sync`.", "The vector.")]
		#[document_returns("A new vector with filtered and mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_filterable::ParRefFilterable,
		/// };
		/// let result = VecBrand::par_ref_filter_map(
		/// 	|x: &i32| if *x > 2 { Some(x.to_string()) } else { None },
		/// 	&vec![1, 2, 3, 4],
		/// );
		/// assert_eq!(result, vec!["3", "4"]);
		/// ```
		fn par_ref_filter_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(&A) -> Option<B> + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.par_iter().filter_map(f).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.iter().filter_map(f).collect()
		}
	}

	impl ParRefFunctorWithIndex for VecBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function with index. Must be `Send + Sync`.", "The vector.")]
		#[document_returns("A new vector with mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_functor_with_index::ParRefFunctorWithIndex,
		/// };
		/// let result =
		/// 	VecBrand::par_ref_map_with_index(|i, x: &i32| format!("{}:{}", i, x), &vec![10, 20]);
		/// assert_eq!(result, vec!["0:10", "1:20"]);
		/// ```
		fn par_ref_map_with_index<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(usize, &A) -> B + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.par_iter().enumerate().map(|(i, a)| f(i, a)).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.iter().enumerate().map(|(i, a)| f(i, a)).collect()
		}
	}

	impl ParRefFoldableWithIndex for VecBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The element type.", "The monoid type.")]
		#[document_parameters("The function with index. Must be `Send + Sync`.", "The vector.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_foldable_with_index::ParRefFoldableWithIndex,
		/// };
		/// let result =
		/// 	VecBrand::par_ref_fold_map_with_index(|i, x: &i32| format!("{}:{}", i, x), &vec![10, 20]);
		/// assert_eq!(result, "0:101:20");
		/// ```
		fn par_ref_fold_map_with_index<'a, A: Send + Sync + 'a, M: Monoid + Send + 'a>(
			f: impl Fn(usize, &A) -> M + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.par_iter()
					.enumerate()
					.map(|(i, a)| f(i, a))
					.reduce(Monoid::empty, Semigroup::append)
			}
			#[cfg(not(feature = "rayon"))]
			fa.iter()
				.enumerate()
				.map(|(i, a)| f(i, a))
				.fold(Monoid::empty(), |acc, m| Semigroup::append(acc, m))
		}
	}

	impl ParRefFilterableWithIndex for VecBrand {
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function with index. Must be `Send + Sync`.", "The vector.")]
		#[document_returns("A new vector with filtered and mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_filterable_with_index::ParRefFilterableWithIndex,
		/// };
		/// let v = vec![10, 20, 30, 40, 50];
		/// let result = VecBrand::par_ref_filter_map_with_index(
		/// 	|i, x: &i32| if i % 2 == 0 { Some(x.to_string()) } else { None },
		/// 	&v,
		/// );
		/// assert_eq!(result, vec!["10", "30", "50"]);
		/// ```
		fn par_ref_filter_map_with_index<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(usize, &A) -> Option<B> + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			#[cfg(feature = "rayon")]
			{
				use rayon::prelude::*;
				fa.par_iter().enumerate().filter_map(|(i, a)| f(i, a)).collect()
			}
			#[cfg(not(feature = "rayon"))]
			fa.iter().enumerate().filter_map(|(i, a)| f(i, a)).collect()
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
	fn functor_identity(x: Vec<i32>) -> bool {
		map::<VecBrand, _, _, _, _>(identity, x.clone()) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Vec<i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<VecBrand, _, _, _, _>(compose(f, g), x.clone())
			== map::<VecBrand, _, _, _, _>(f, map::<VecBrand, _, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Vec<i32>) -> bool {
		apply::<RcFnBrand, VecBrand, _, _>(
			pure::<VecBrand, _>(<RcFnBrand as LiftFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, VecBrand, _, _>(
			pure::<VecBrand, _>(<RcFnBrand as LiftFn>::new(f)),
			pure::<VecBrand, _>(x),
		) == pure::<VecBrand, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: Vec<i32>,
		u_seeds: Vec<i32>,
		v_seeds: Vec<i32>,
	) -> bool {
		let u_fns: Vec<_> = u_seeds
			.iter()
			.map(|&i| <RcFnBrand as LiftFn>::new(move |x: i32| x.wrapping_add(i)))
			.collect();
		let v_fns: Vec<_> = v_seeds
			.iter()
			.map(|&i| <RcFnBrand as LiftFn>::new(move |x: i32| x.wrapping_mul(i)))
			.collect();

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, VecBrand, _, _>(v_fns.clone(), w.clone());
		let rhs = apply::<RcFnBrand, VecBrand, _, _>(u_fns.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		// We construct (u . v) manually as the cartesian product of compositions
		let uv_fns: Vec<_> = u_fns
			.iter()
			.flat_map(|uf| {
				v_fns.iter().map(move |vf| {
					let uf = uf.clone();
					let vf = vf.clone();
					<RcFnBrand as LiftFn>::new(move |x| uf(vf(x)))
				})
			})
			.collect();

		let lhs = apply::<RcFnBrand, VecBrand, _, _>(uv_fns, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = vec![<RcFnBrand as LiftFn>::new(f)];

		let lhs = apply::<RcFnBrand, VecBrand, _, _>(u.clone(), pure::<VecBrand, _>(y));

		let rhs_fn = <RcFnBrand as LiftFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, VecBrand, _, _>(pure::<VecBrand, _>(rhs_fn), u);

		lhs == rhs
	}

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(
		a: Vec<i32>,
		b: Vec<i32>,
		c: Vec<i32>,
	) -> bool {
		append(a.clone(), append(b.clone(), c.clone())) == append(append(a, b), c)
	}

	// Monoid Laws

	/// Tests the left identity law for Monoid.
	#[quickcheck]
	fn monoid_left_identity(a: Vec<i32>) -> bool {
		append(empty::<Vec<i32>>(), a.clone()) == a
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(a: Vec<i32>) -> bool {
		append(a.clone(), empty::<Vec<i32>>()) == a
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| vec![x.wrapping_mul(2)];
		bind::<VecBrand, _, _, _, _>(pure::<VecBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Vec<i32>) -> bool {
		bind::<VecBrand, _, _, _, _>(m.clone(), pure::<VecBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Vec<i32>) -> bool {
		let f = |x: i32| vec![x.wrapping_mul(2)];
		let g = |x: i32| vec![x.wrapping_add(1)];
		bind::<VecBrand, _, _, _, _>(bind::<VecBrand, _, _, _, _>(m.clone(), f), g)
			== bind::<VecBrand, _, _, _, _>(m, |x| bind::<VecBrand, _, _, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests `map` on an empty vector.
	#[test]
	fn map_empty() {
		assert_eq!(
			map::<VecBrand, _, _, _, _>(|x: i32| x + 1, vec![] as Vec<i32>),
			vec![] as Vec<i32>
		);
	}

	/// Tests `bind` on an empty vector.
	#[test]
	fn bind_empty() {
		assert_eq!(
			bind::<VecBrand, _, _, _, _>(vec![] as Vec<i32>, |x: i32| vec![x + 1]),
			vec![] as Vec<i32>
		);
	}

	/// Tests `bind` returning an empty vector.
	#[test]
	fn bind_returning_empty() {
		assert_eq!(
			bind::<VecBrand, _, _, _, _>(vec![1, 2, 3], |_| vec![] as Vec<i32>),
			vec![] as Vec<i32>
		);
	}

	/// Tests `fold_right` on an empty vector.
	#[test]
	fn fold_right_empty() {
		assert_eq!(
			crate::functions::fold_right::<RcFnBrand, VecBrand, _, _, _, _>(
				|x: i32, acc| x + acc,
				0,
				vec![]
			),
			0
		);
	}

	/// Tests `fold_left` on an empty vector.
	#[test]
	fn fold_left_empty() {
		assert_eq!(
			crate::functions::fold_left::<RcFnBrand, VecBrand, _, _, _, _>(
				|acc, x: i32| acc + x,
				0,
				vec![]
			),
			0
		);
	}

	/// Tests `traverse` on an empty vector.
	#[test]
	fn traverse_empty() {
		use crate::brands::OptionBrand;
		assert_eq!(
			crate::classes::traversable::traverse::<VecBrand, _, _, OptionBrand>(
				|x: i32| Some(x + 1),
				vec![]
			),
			Some(vec![])
		);
	}

	/// Tests `traverse` returning an empty vector.
	#[test]
	fn traverse_returning_empty() {
		use crate::brands::OptionBrand;
		assert_eq!(
			crate::classes::traversable::traverse::<VecBrand, _, _, OptionBrand>(
				|_: i32| None::<i32>,
				vec![1, 2, 3]
			),
			None
		);
	}

	/// Tests `construct` with an empty tail.
	#[test]
	fn construct_empty_tail() {
		assert_eq!(VecBrand::construct(1, vec![]), vec![1]);
	}

	/// Tests `deconstruct` on an empty slice.
	#[test]
	fn deconstruct_empty() {
		assert_eq!(VecBrand::deconstruct::<i32>(&[]), None);
	}

	// Parallel Trait Tests

	/// Tests `par_map` on a vector.
	#[test]
	fn par_map_basic() {
		let v = vec![1, 2, 3];
		let result: Vec<i32> = par_map::<VecBrand, _, _>(|x: i32| x * 2, v);
		assert_eq!(result, vec![2, 4, 6]);
	}

	/// Tests `par_filter` on a vector.
	#[test]
	fn par_filter_basic() {
		let v = vec![1, 2, 3, 4, 5];
		let result: Vec<i32> = par_filter::<VecBrand, _>(|x: &i32| x % 2 == 0, v);
		assert_eq!(result, vec![2, 4]);
	}

	/// Tests `par_filter_map` on a vector.
	#[test]
	fn par_filter_map_basic() {
		let v = vec![1, 2, 3, 4, 5];
		let result: Vec<i32> = par_filter_map::<VecBrand, _, _>(
			|x: i32| if x % 2 == 0 { Some(x * 10) } else { None },
			v,
		);
		assert_eq!(result, vec![20, 40]);
	}

	/// Tests `par_compact` on a vector of options.
	#[test]
	fn par_compact_basic() {
		let v = vec![Some(1), None, Some(3), None, Some(5)];
		let result: Vec<i32> = par_compact::<VecBrand, _>(v);
		assert_eq!(result, vec![1, 3, 5]);
	}

	/// Tests `par_separate` on a vector of results.
	#[test]
	fn par_separate_basic() {
		let v: Vec<Result<i32, &str>> = vec![Ok(1), Err("a"), Ok(3), Err("b")];
		let (errs, oks): (Vec<&str>, Vec<i32>) = par_separate::<VecBrand, _, _>(v);
		assert_eq!(errs, vec!["a", "b"]);
		assert_eq!(oks, vec![1, 3]);
	}

	/// Tests `par_map_with_index` on a vector.
	#[test]
	fn par_map_with_index_basic() {
		let v = vec![10, 20, 30];
		let result: Vec<i32> = par_map_with_index::<VecBrand, _, _>(|i, x: i32| x + i as i32, v);
		assert_eq!(result, vec![10, 21, 32]);
	}

	/// Tests `par_fold_map` on an empty vector.
	#[test]
	fn par_fold_map_empty() {
		let v: Vec<i32> = vec![];
		assert_eq!(par_fold_map::<VecBrand, _, _>(|x: i32| x.to_string(), v), "".to_string());
	}

	/// Tests `par_fold_map` on multiple elements.
	#[test]
	fn par_fold_map_multiple() {
		let v = vec![1, 2, 3];
		assert_eq!(par_fold_map::<VecBrand, _, _>(|x: i32| x.to_string(), v), "123".to_string());
	}

	/// Tests `par_fold_map_with_index` on a vector.
	#[test]
	fn par_fold_map_with_index_basic() {
		let v = vec![10, 20, 30];
		let result: String =
			par_fold_map_with_index::<VecBrand, _, _>(|i, x: i32| format!("{i}:{x}"), v);
		assert_eq!(result, "0:101:202:30");
	}

	// Filterable Laws

	/// Tests `filterMap identity ≡ compact`.
	#[quickcheck]
	fn filterable_filter_map_identity(x: Vec<Option<i32>>) -> bool {
		filter_map::<VecBrand, _, _, _, _>(identity, x.clone()) == compact::<VecBrand, _>(x)
	}

	/// Tests `filterMap Just ≡ identity`.
	#[quickcheck]
	fn filterable_filter_map_just(x: Vec<i32>) -> bool {
		filter_map::<VecBrand, _, _, _, _>(Some, x.clone()) == x
	}

	/// Tests `filterMap (l <=< r) ≡ filterMap l <<< filterMap r`.
	#[quickcheck]
	fn filterable_filter_map_composition(x: Vec<i32>) -> bool {
		let r = |i: i32| if i % 2 == 0 { Some(i) } else { None };
		let l = |i: i32| if i > 5 { Some(i) } else { None };
		let composed = |i| bind::<OptionBrand, _, _, _, _>(r(i), l);

		filter_map::<VecBrand, _, _, _, _>(composed, x.clone())
			== filter_map::<VecBrand, _, _, _, _>(l, filter_map::<VecBrand, _, _, _, _>(r, x))
	}

	/// Tests `filter ≡ filterMap <<< maybeBool`.
	#[quickcheck]
	fn filterable_filter_consistency(x: Vec<i32>) -> bool {
		let p = |i: i32| i % 2 == 0;
		let maybe_bool = |i| if p(i) { Some(i) } else { None };

		filter::<VecBrand, _, _, _>(p, x.clone())
			== filter_map::<VecBrand, _, _, _, _>(maybe_bool, x)
	}

	/// Tests `partitionMap identity ≡ separate`.
	#[quickcheck]
	fn filterable_partition_map_identity(x: Vec<Result<i32, i32>>) -> bool {
		partition_map::<VecBrand, _, _, _, _, _>(identity, x.clone())
			== separate::<VecBrand, _, _>(x)
	}

	/// Tests `partitionMap Right ≡ identity` (on the right side).
	#[quickcheck]
	fn filterable_partition_map_right_identity(x: Vec<i32>) -> bool {
		let (_, oks) = partition_map::<VecBrand, _, _, _, _, _>(Ok::<_, i32>, x.clone());
		oks == x
	}

	/// Tests `partitionMap Left ≡ identity` (on the left side).
	#[quickcheck]
	fn filterable_partition_map_left_identity(x: Vec<i32>) -> bool {
		let (errs, _) = partition_map::<VecBrand, _, _, _, _, _>(Err::<i32, _>, x.clone());
		errs == x
	}

	/// Tests `f <<< partition ≡ partitionMap <<< eitherBool`.
	#[quickcheck]
	fn filterable_partition_consistency(x: Vec<i32>) -> bool {
		let p = |i: i32| i % 2 == 0;
		let either_bool = |i| if p(i) { Ok(i) } else { Err(i) };

		let (not_satisfied, satisfied) = partition::<VecBrand, _, _, _>(p, x.clone());
		let (errs, oks) = partition_map::<VecBrand, _, _, _, _, _>(either_bool, x);

		satisfied == oks && not_satisfied == errs
	}

	// Witherable Laws

	/// Tests `wither (pure <<< Just) ≡ pure`.
	#[quickcheck]
	fn witherable_identity(x: Vec<i32>) -> bool {
		wither::<VecBrand, OptionBrand, _, _>(|i| Some(Some(i)), x.clone()) == Some(x)
	}

	/// Tests `wilt p ≡ map separate <<< traverse p`.
	#[quickcheck]
	fn witherable_wilt_consistency(x: Vec<i32>) -> bool {
		let p = |i: i32| Some(if i % 2 == 0 { Ok(i) } else { Err(i) });

		let lhs = wilt::<VecBrand, OptionBrand, _, _, _>(p, x.clone());
		let rhs = crate::classes::dispatch::map::<OptionBrand, _, _, _, _>(
			separate::<VecBrand, _, _>,
			traverse::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(p, x),
		);

		lhs == rhs
	}

	/// Tests `wither p ≡ map compact <<< traverse p`.
	#[quickcheck]
	fn witherable_wither_consistency(x: Vec<i32>) -> bool {
		let p = |i: i32| Some(if i % 2 == 0 { Some(i) } else { None });

		let lhs = wither::<VecBrand, OptionBrand, _, _>(p, x.clone());
		let rhs = crate::classes::dispatch::map::<OptionBrand, _, _, _, _>(
			compact::<VecBrand, _>,
			traverse::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(p, x),
		);

		lhs == rhs
	}

	// Alt Laws

	/// Tests the associativity law for Alt.
	#[quickcheck]
	fn alt_associativity(
		x: Vec<i32>,
		y: Vec<i32>,
		z: Vec<i32>,
	) -> bool {
		alt::<VecBrand, _>(alt::<VecBrand, _>(x.clone(), y.clone()), z.clone())
			== alt::<VecBrand, _>(x, alt::<VecBrand, _>(y, z))
	}

	/// Tests the distributivity law for Alt.
	#[quickcheck]
	fn alt_distributivity(
		x: Vec<i32>,
		y: Vec<i32>,
	) -> bool {
		let f = |i: i32| i.wrapping_mul(2).wrapping_add(1);
		map::<VecBrand, _, _, _, _>(f, alt::<VecBrand, _>(x.clone(), y.clone()))
			== alt::<VecBrand, _>(
				map::<VecBrand, _, _, _, _>(f, x),
				map::<VecBrand, _, _, _, _>(f, y),
			)
	}

	// Plus Laws

	/// Tests the left identity law for Plus.
	#[quickcheck]
	fn plus_left_identity(x: Vec<i32>) -> bool {
		alt::<VecBrand, _>(plus_empty::<VecBrand, i32>(), x.clone()) == x
	}

	/// Tests the right identity law for Plus.
	#[quickcheck]
	fn plus_right_identity(x: Vec<i32>) -> bool {
		alt::<VecBrand, _>(x.clone(), plus_empty::<VecBrand, i32>()) == x
	}

	/// Tests the annihilation law for Plus.
	#[test]
	fn plus_annihilation() {
		let f = |i: i32| i.wrapping_mul(2);
		assert_eq!(
			map::<VecBrand, _, _, _, _>(f, plus_empty::<VecBrand, i32>()),
			plus_empty::<VecBrand, i32>(),
		);
	}

	// Compactable Laws (Plus-dependent)

	/// Tests the functor identity law for Compactable.
	#[quickcheck]
	fn compactable_functor_identity(fa: Vec<i32>) -> bool {
		compact::<VecBrand, _>(map::<VecBrand, _, _, _, _>(Some, fa.clone())) == fa
	}

	/// Tests the Plus annihilation (empty) law for Compactable.
	#[test]
	fn compactable_plus_annihilation_empty() {
		assert_eq!(
			compact::<VecBrand, _>(plus_empty::<VecBrand, Option<i32>>()),
			plus_empty::<VecBrand, i32>(),
		);
	}

	/// Tests the Plus annihilation (map) law for Compactable.
	#[quickcheck]
	fn compactable_plus_annihilation_map(xs: Vec<i32>) -> bool {
		compact::<VecBrand, _>(map::<VecBrand, _, _, _, _>(|_: i32| None::<i32>, xs))
			== plus_empty::<VecBrand, i32>()
	}

	// Edge Cases

	/// Tests `compact` on an empty vector.
	#[test]
	fn compact_empty() {
		assert_eq!(compact::<VecBrand, i32>(vec![] as Vec<Option<i32>>), vec![] as Vec<i32>);
	}

	/// Tests `compact` on a vector with `None`.
	#[test]
	fn compact_with_none() {
		assert_eq!(compact::<VecBrand, i32>(vec![Some(1), None, Some(2)]), vec![1, 2]);
	}

	/// Tests `separate` on an empty vector.
	#[test]
	fn separate_empty() {
		let (errs, oks) = separate::<VecBrand, i32, i32>(vec![] as Vec<Result<i32, i32>>);
		assert_eq!(oks, vec![] as Vec<i32>);
		assert_eq!(errs, vec![] as Vec<i32>);
	}

	/// Tests `separate` on a vector with `Ok` and `Err`.
	#[test]
	fn separate_mixed() {
		let (errs, oks) = separate::<VecBrand, i32, i32>(vec![Ok(1), Err(2), Ok(3)]);
		assert_eq!(oks, vec![1, 3]);
		assert_eq!(errs, vec![2]);
	}

	/// Tests `partition_map` on an empty vector.
	#[test]
	fn partition_map_empty() {
		let (errs, oks) =
			partition_map::<VecBrand, _, _, _, _, _>(|x: i32| Ok::<i32, i32>(x), vec![]);
		assert_eq!(oks, vec![] as Vec<i32>);
		assert_eq!(errs, vec![] as Vec<i32>);
	}

	/// Tests `partition` on an empty vector.
	#[test]
	fn partition_empty() {
		let (not_satisfied, satisfied) = partition::<VecBrand, _, _, _>(|x: i32| x > 0, vec![]);
		assert_eq!(satisfied, vec![] as Vec<i32>);
		assert_eq!(not_satisfied, vec![] as Vec<i32>);
	}

	/// Tests `filter_map` on an empty vector.
	#[test]
	fn filter_map_empty() {
		assert_eq!(
			filter_map::<VecBrand, i32, _, _, _>(|x: i32| Some(x), vec![]),
			vec![] as Vec<i32>
		);
	}

	/// Tests `filter` on an empty vector.
	#[test]
	fn filter_empty() {
		assert_eq!(filter::<VecBrand, _, _, _>(|x: i32| x > 0, vec![]), vec![] as Vec<i32>);
	}

	/// Tests `wilt` on an empty vector.
	#[test]
	fn wilt_empty() {
		let res = wilt::<VecBrand, OptionBrand, _, _, _>(|x: i32| Some(Ok::<i32, i32>(x)), vec![]);
		assert_eq!(res, Some((vec![], vec![])));
	}

	/// Tests `wither` on an empty vector.
	#[test]
	fn wither_empty() {
		let res = wither::<VecBrand, OptionBrand, _, _>(|x: i32| Some(Some(x)), vec![]);
		assert_eq!(res, Some(vec![]));
	}

	// Parallel Trait Laws

	/// Verifies that `par_fold_map` correctly sums a large vector (100,000 elements).
	#[test]
	fn test_large_vector_par_fold_map() {
		use crate::types::Additive;

		let xs: Vec<i32> = (0 .. 100000).collect();
		let res = par_fold_map::<VecBrand, _, _>(|x: i32| Additive(x as i64), xs);
		assert_eq!(res, Additive(4999950000));
	}

	/// Property: `par_map` agrees with sequential `map`.
	#[quickcheck]
	fn prop_par_map_equals_map(xs: Vec<i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let seq_res = map::<VecBrand, _, _, _, _>(f, xs.clone());
		let par_res = par_map::<VecBrand, _, _>(f, xs);
		seq_res == par_res
	}

	/// Property: `par_fold_map` agrees with sequential `fold_map`.
	#[quickcheck]
	fn prop_par_fold_map_equals_fold_map(xs: Vec<i32>) -> bool {
		use crate::types::Additive;

		let f = |x: i32| Additive(x as i64);
		let seq_res = crate::functions::fold_map::<crate::brands::RcFnBrand, VecBrand, _, _, _, _>(
			f,
			xs.clone(),
		);
		let par_res = par_fold_map::<VecBrand, _, _>(f, xs);
		seq_res == par_res
	}

	/// Property: `par_fold_map` on an empty vector returns the Monoid's empty value.
	#[quickcheck]
	fn prop_par_fold_map_empty_is_empty(xs: Vec<i32>) -> bool {
		use crate::types::Additive;

		if !xs.is_empty() {
			return true;
		}
		let par_res = par_fold_map::<VecBrand, _, _>(|x: i32| Additive(x as i64), xs);
		par_res == empty::<Additive<i64>>()
	}

	// MonadRec tests

	/// Tests the MonadRec identity law: `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`.
	#[quickcheck]
	fn monad_rec_identity(x: i32) -> bool {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		tail_rec_m::<VecBrand, _, _>(|a| vec![ControlFlow::Break(a)], x) == vec![x]
	}

	/// Tests a simple linear recursion via `tail_rec_m`.
	#[test]
	fn monad_rec_linear() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		// Count up to 5
		let result = tail_rec_m::<VecBrand, _, _>(
			|n| {
				if n < 5 { vec![ControlFlow::Continue(n + 1)] } else { vec![ControlFlow::Break(n)] }
			},
			0,
		);
		assert_eq!(result, vec![5]);
	}

	/// Tests branching nondeterminism via `tail_rec_m`.
	#[test]
	fn monad_rec_branching() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		// Each step either finishes or continues
		let result = tail_rec_m::<VecBrand, _, _>(
			|n: i32| {
				if n < 2 {
					vec![ControlFlow::Continue(n + 1), ControlFlow::Break(n * 100)]
				} else {
					vec![ControlFlow::Break(n * 100)]
				}
			},
			0,
		);
		// n=0: Loop(1), Done(0)
		// n=1: Loop(2), Done(100)
		// n=2: Done(200)
		assert_eq!(result, vec![0, 100, 200]);
	}

	/// Tests that `tail_rec_m` handles an empty result from the step function.
	#[test]
	fn monad_rec_empty() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result: Vec<i32> =
			tail_rec_m::<VecBrand, _, _>(|_n| Vec::<ControlFlow<i32, i32>>::new(), 0);
		assert_eq!(result, Vec::<i32>::new());
	}

	// Extend Laws

	/// Tests basic `extend` on `Vec`: sum of suffixes.
	#[test]
	fn extend_sum_of_suffixes() {
		use crate::classes::extend::extend;
		let result = extend::<VecBrand, _, _>(|v: Vec<i32>| v.iter().sum::<i32>(), vec![1, 2, 3]);
		assert_eq!(result, vec![6, 5, 3]);
	}

	/// Extend associativity: `extend(f, extend(g, w)) == extend(|w| f(extend(g, w)), w)`.
	#[quickcheck]
	fn extend_associativity(w: Vec<i32>) -> bool {
		use crate::classes::extend::extend;
		let g = |v: Vec<i32>| v.iter().fold(0i32, |a, b| a.wrapping_mul(2).wrapping_add(*b));
		let f = |v: Vec<i32>| v.iter().fold(0i32, |a, b| a.wrapping_add(b.wrapping_add(1)));
		let lhs = extend::<VecBrand, _, _>(f, extend::<VecBrand, _, _>(g, w.clone()));
		let rhs = extend::<VecBrand, _, _>(|w: Vec<i32>| f(extend::<VecBrand, _, _>(g, w)), w);
		lhs == rhs
	}

	/// Tests that `duplicate` produces suffixes.
	#[test]
	fn extend_duplicate_suffixes() {
		use crate::classes::extend::duplicate;
		let result = duplicate::<VecBrand, _>(vec![1, 2, 3]);
		assert_eq!(result, vec![vec![1, 2, 3], vec![2, 3], vec![3]]);
	}

	/// Tests `extend` on an empty vector.
	#[test]
	fn extend_empty() {
		use crate::classes::extend::extend;
		let result =
			extend::<VecBrand, _, _>(|v: Vec<i32>| v.iter().sum::<i32>(), Vec::<i32>::new());
		assert_eq!(result, Vec::<i32>::new());
	}

	/// Tests `extend` on a singleton vector.
	#[test]
	fn extend_singleton() {
		use crate::classes::extend::extend;
		let result = extend::<VecBrand, _, _>(|v: Vec<i32>| v.iter().sum::<i32>(), vec![42]);
		assert_eq!(result, vec![42]);
	}

	// -- Ref trait laws --

	/// Tests the identity law for RefFunctor: ref_map(deref, v) == v.
	#[quickcheck]
	fn ref_functor_identity(v: Vec<i32>) -> bool {
		use crate::classes::ref_functor::RefFunctor;
		VecBrand::ref_map(|x: &i32| *x, &v) == v
	}

	/// Tests the composition law for RefFunctor:
	/// ref_map(|x| g(&f(x)), v) == ref_map(g, ref_map(f, v)).
	#[quickcheck]
	fn ref_functor_composition(v: Vec<i32>) -> bool {
		use crate::classes::ref_functor::RefFunctor;
		let f = |x: &i32| x.wrapping_add(1);
		let g = |x: &i32| x.wrapping_mul(2);
		VecBrand::ref_map(|x: &i32| g(&f(x)), &v) == VecBrand::ref_map(g, &VecBrand::ref_map(f, &v))
	}

	/// Tests RefFoldable with Additive monoid: ref_fold_map matches iter().sum().
	#[quickcheck]
	fn ref_foldable_additive(v: Vec<i32>) -> bool {
		use crate::{
			brands::RcFnBrand,
			classes::ref_foldable::RefFoldable,
			types::Additive,
		};
		let result: Additive<i32> =
			VecBrand::ref_fold_map::<RcFnBrand, _, _>(|x: &i32| Additive(*x), &v);
		result.0 == v.iter().copied().fold(0i32, |a, b| a.wrapping_add(b))
	}

	/// Tests the left identity law for RefSemimonad:
	/// ref_bind(vec![x], |a| vec![*a]) == vec![x].
	#[quickcheck]
	fn ref_semimonad_left_identity(x: i32) -> bool {
		use crate::classes::ref_semimonad::RefSemimonad;
		VecBrand::ref_bind(&vec![x], |a: &i32| vec![*a]) == vec![x]
	}

	/// Tests the associativity law for RefSemimonad:
	/// ref_bind(ref_bind(v, f), g) == ref_bind(v, |a| ref_bind(f(a), g)).
	#[quickcheck]
	fn ref_semimonad_associativity(v: Vec<i32>) -> bool {
		use crate::classes::ref_semimonad::RefSemimonad;
		let f = |a: &i32| vec![a.wrapping_add(1), a.wrapping_mul(2)];
		let g = |b: &i32| vec![b.wrapping_add(10)];
		let lhs = VecBrand::ref_bind(&VecBrand::ref_bind(&v, f), g);
		let rhs = VecBrand::ref_bind(&v, |a: &i32| VecBrand::ref_bind(&f(a), g));
		lhs == rhs
	}

	/// Tests ParRefFunctor equivalence: par_ref_map(f, v) == ref_map(f, v).
	#[quickcheck]
	fn par_ref_functor_equivalence(v: Vec<i32>) -> bool {
		use crate::classes::{
			par_ref_functor::ParRefFunctor,
			ref_functor::RefFunctor,
		};
		let f = |x: &i32| x.wrapping_mul(3).wrapping_add(7);
		VecBrand::par_ref_map(f, &v) == VecBrand::ref_map(f, &v)
	}

	// RefSemimonad Laws (continued)

	/// Tests the right identity law for RefSemimonad:
	/// `ref_bind(v, |a| ref_pure(a)) == v`.
	#[quickcheck]
	fn ref_semimonad_right_identity(v: Vec<i32>) -> bool {
		use crate::classes::{
			ref_pointed::RefPointed,
			ref_semimonad::RefSemimonad,
		};
		VecBrand::ref_bind(&v, |a: &i32| VecBrand::ref_pure(a)) == v
	}

	// RefLift Laws

	/// Tests the identity law for RefLift:
	/// `ref_lift2(|_, b| *b, pure(unit), fa) == fa`.
	#[quickcheck]
	fn ref_lift_identity(v: Vec<i32>) -> bool {
		use crate::classes::ref_lift::RefLift;
		VecBrand::ref_lift2(|_: &(), b: &i32| *b, &vec![()], &v) == v
	}

	// RefTraversable Laws

	/// Tests the identity law for RefTraversable:
	/// `ref_traverse(|a| Identity(*a), ta) == Identity(ta)`.
	#[quickcheck]
	fn ref_traversable_identity(v: Vec<i32>) -> bool {
		use crate::{
			classes::ref_traversable::RefTraversable,
			types::Identity,
		};
		let result: Identity<Vec<i32>> =
			VecBrand::ref_traverse::<RcFnBrand, _, _, IdentityBrand>(|a: &i32| Identity(*a), &v);
		result == Identity(v)
	}

	/// Tests RefTraversable naturality: ref_traverse(f, ta) produces
	/// the same result as traverse(|a| f(&a), ta).
	#[quickcheck]
	fn ref_traversable_consistent_with_traverse(v: Vec<i32>) -> bool {
		use crate::classes::{
			ref_traversable::RefTraversable,
			traversable::Traversable,
		};
		let ref_result: Option<Vec<String>> = VecBrand::ref_traverse::<RcFnBrand, _, _, OptionBrand>(
			|a: &i32| Some(a.to_string()),
			&v,
		);
		let val_result: Option<Vec<String>> =
			VecBrand::traverse::<i32, String, OptionBrand>(|a: i32| Some(a.to_string()), v);
		ref_result == val_result
	}

	// RefCompactable Laws

	/// RefCompactable identity: ref_compact of ref_map(Some, &v) preserves values
	#[quickcheck]
	fn ref_compactable_identity(v: Vec<i32>) -> bool {
		use crate::classes::ref_compactable::ref_compact;
		let mapped: Vec<Option<i32>> = v.iter().map(|a| Some(*a)).collect();
		ref_compact::<VecBrand, _>(&mapped) == v
	}

	// RefAlt Laws

	/// RefAlt associativity
	#[quickcheck]
	fn ref_alt_associativity(
		x: Vec<i32>,
		y: Vec<i32>,
		z: Vec<i32>,
	) -> bool {
		use crate::classes::ref_alt::ref_alt;
		ref_alt::<VecBrand, _>(&ref_alt::<VecBrand, _>(&x, &y), &z)
			== ref_alt::<VecBrand, _>(&x, &ref_alt::<VecBrand, _>(&y, &z))
	}

	/// RefAlt distributivity with RefFunctor
	#[quickcheck]
	fn ref_alt_distributivity(
		x: Vec<i32>,
		y: Vec<i32>,
	) -> bool {
		use crate::classes::ref_alt::ref_alt;
		let f = |a: &i32| a.wrapping_mul(2);
		map::<VecBrand, _, _, _, _>(f, &ref_alt::<VecBrand, _>(&x, &y))
			== ref_alt::<VecBrand, _>(
				&map::<VecBrand, _, _, _, _>(f, &x),
				&map::<VecBrand, _, _, _, _>(f, &y),
			)
	}
}
