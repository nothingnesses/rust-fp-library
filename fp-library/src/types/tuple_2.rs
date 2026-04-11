//! Two-value tuple with [`Bifunctor`](crate::classes::Bifunctor) and dual [`Functor`](crate::classes::Functor) instances.
//!
//! Can be used as a bifunctor over both values, or as a functor/monad by fixing either the first value [`Tuple2FirstAppliedBrand`](crate::brands::Tuple2FirstAppliedBrand) or second value [`Tuple2SecondAppliedBrand`](crate::brands::Tuple2SecondAppliedBrand).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				Tuple2Brand,
				Tuple2FirstAppliedBrand,
				Tuple2SecondAppliedBrand,
			},
			classes::*,
			dispatch::Ref,
			impl_kind,
			kinds::*,
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	impl_kind! {
		for Tuple2Brand {
			type Of<First, Second> = (First, Second);
		}
	}

	impl_kind! {
		for Tuple2Brand {
			type Of<'a, First: 'a, Second: 'a>: 'a = (First, Second);
		}
	}

	impl Bifunctor for Tuple2Brand {
		/// Maps functions over the values in the tuple.
		///
		/// This method applies one function to the first value and another to the second value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the mapped first value.",
			"The type of the second value.",
			"The type of the mapped second value."
		)]
		///
		#[document_parameters(
			"The function to apply to the first value.",
			"The function to apply to the second value.",
			"The tuple to map over."
		)]
		///
		#[document_returns("A new tuple containing the mapped values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (1, 5);
		/// assert_eq!(bimap::<Tuple2Brand, _, _, _, _, _, _>((|a| a + 1, |b| b * 2), x), (2, 10));
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(A) -> B + 'a,
			g: impl Fn(C) -> D + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			(f(p.0), g(p.1))
		}
	}

	impl RefBifunctor for Tuple2Brand {
		/// Maps functions over references to the values in the tuple.
		///
		/// This method applies one function to a reference of the first value and another
		/// to a reference of the second value, producing a new tuple with mapped values.
		/// The original tuple is borrowed, not consumed.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the mapped first value.",
			"The type of the second value.",
			"The type of the mapped second value."
		)]
		///
		#[document_parameters(
			"The function to apply to a reference of the first value.",
			"The function to apply to a reference of the second value.",
			"The tuple to map over by reference."
		)]
		///
		#[document_returns("A new tuple containing the mapped values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_bifunctor::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (1, 5);
		/// assert_eq!(ref_bimap::<Tuple2Brand, _, _, _, _>(|a| *a + 1, |b| *b * 2, &x), (2, 10));
		/// ```
		fn ref_bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(&A) -> B + 'a,
			g: impl Fn(&C) -> D + 'a,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			(f(&p.0), g(&p.1))
		}
	}

	impl RefBifoldable for Tuple2Brand {
		/// Folds a tuple from right to left by reference using two step functions.
		///
		/// Applies `f` to a reference of the first value and `g` to a reference of
		/// the second value, folding `(a, b)` as `f(&a, g(&b, z))`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first element.",
			"The type of the second element.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function applied to a reference of the first element.",
			"The step function applied to a reference of the second element.",
			"The initial accumulator.",
			"The tuple to fold by reference."
		)]
		///
		#[document_returns("`f(&a, g(&b, z))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (3, 5);
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, Tuple2Brand, _, _, _, _, _>(
		/// 		(|a: &i32, acc| acc - *a, |b: &i32, acc| acc + *b),
		/// 		0,
		/// 		&x,
		/// 	),
		/// 	2
		/// );
		/// ```
		fn ref_bi_fold_right<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(&A, C) -> C + 'a,
			g: impl Fn(&B, C) -> C + 'a,
			z: C,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			f(&p.0, g(&p.1, z))
		}
	}

	impl RefBitraversable for Tuple2Brand {
		/// Traverses a tuple by reference with two effectful functions.
		///
		/// Applies `f` to a reference of the first element and `g` to a reference
		/// of the second element, combining the effects via `lift2`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the first element.",
			"The type of the second element.",
			"The output type for the first element.",
			"The output type for the second element.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function applied to a reference of the first element.",
			"The function applied to a reference of the second element.",
			"The tuple to traverse by reference."
		)]
		///
		#[document_returns("`lift2(|c, d| (c, d), f(&a), g(&b))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = (3, 5);
		/// assert_eq!(
		/// 	bi_traverse::<RcFnBrand, Tuple2Brand, _, _, _, _, OptionBrand, _, _>(
		/// 		(|a: &i32| Some(a + 1), |b: &i32| Some(b * 2)),
		/// 		&x,
		/// 	),
		/// 	Some((4, 10))
		/// );
		/// ```
		fn ref_bi_traverse<
			'a,
			FnBrand,
			A: 'a + Clone,
			B: 'a + Clone,
			C: 'a + Clone,
			D: 'a + Clone,
			F: Applicative,
		>(
			f: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(&B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>): Clone, {
			F::lift2(|c, d| (c, d), f(&p.0), g(&p.1))
		}
	}

	impl Bifoldable for Tuple2Brand {
		/// Folds a tuple using two step functions, right-associatively.
		///
		/// Applies `f` to the first value and `g` to the second value,
		/// folding `(a, b)` as `f(a, g(b, z))`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first element.",
			"The type of the second element.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function applied to the first element.",
			"The step function applied to the second element.",
			"The initial accumulator.",
			"The tuple to fold."
		)]
		///
		#[document_returns("`f(a, g(b, z))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, Tuple2Brand, _, _, _, _, _>(
		/// 		(|a: i32, acc| acc - a, |b: i32, acc| acc + b),
		/// 		0,
		/// 		(3, 5),
		/// 	),
		/// 	2
		/// );
		/// ```
		fn bi_fold_right<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(A, C) -> C + 'a,
			g: impl Fn(B, C) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			let (a, b) = p;
			f(a, g(b, z))
		}

		/// Folds a tuple using two step functions, left-associatively.
		///
		/// Applies `f` to the first value and `g` to the second value,
		/// folding `(a, b)` as `g(f(z, a), b)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first element.",
			"The type of the second element.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function applied to the first element.",
			"The step function applied to the second element.",
			"The initial accumulator.",
			"The tuple to fold."
		)]
		///
		#[document_returns("`g(f(z, a), b)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_left::<RcFnBrand, Tuple2Brand, _, _, _, _, _>(
		/// 		(|acc, a: i32| acc - a, |acc, b: i32| acc + b),
		/// 		0,
		/// 		(3, 5),
		/// 	),
		/// 	2
		/// );
		/// ```
		fn bi_fold_left<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(C, A) -> C + 'a,
			g: impl Fn(C, B) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			let (a, b) = p;
			g(f(z, a), b)
		}

		/// Maps both elements of a tuple to a monoid and combines the results.
		///
		/// Computes `M::append(f(a), g(b))` for a tuple `(a, b)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first element.",
			"The type of the second element.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping the first element to the monoid.",
			"The function mapping the second element to the monoid.",
			"The tuple to fold."
		)]
		///
		#[document_returns("`M::append(f(a), g(b))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_map::<RcFnBrand, Tuple2Brand, _, _, _, _, _>(
		/// 		(|a: i32| a.to_string(), |b: i32| b.to_string()),
		/// 		(3, 5),
		/// 	),
		/// 	"35".to_string()
		/// );
		/// ```
		fn bi_fold_map<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, M>(
			f: impl Fn(A) -> M + 'a,
			g: impl Fn(B) -> M + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> M
		where
			M: Monoid + 'a, {
			let (a, b) = p;
			M::append(f(a), g(b))
		}
	}

	impl Bitraversable for Tuple2Brand {
		/// Traverses a tuple with two effectful functions.
		///
		/// Applies `f` to the first element and `g` to the second element,
		/// combining the effects via `lift2`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first element.",
			"The type of the second element.",
			"The output type for the first element.",
			"The output type for the second element.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function applied to the first element.",
			"The function applied to the second element.",
			"The tuple to traverse."
		)]
		///
		#[document_returns("`lift2(|c, d| (c, d), f(a), g(b))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_traverse::<RcFnBrand, Tuple2Brand, _, _, _, _, OptionBrand, _, _>(
		/// 		(|a: i32| Some(a + 1), |b: i32| Some(b * 2)),
		/// 		(3, 5),
		/// 	),
		/// 	Some((4, 10))
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
			let (a, b) = p;
			F::lift2(|c, d| (c, d), f(a), g(b))
		}
	}

	// Tuple2FirstAppliedBrand<First> (Functor over Second)

	impl_kind! {
		impl<First: 'static> for Tuple2FirstAppliedBrand<First> {
			type Of<'a, A: 'a>: 'a = (First, A);
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: 'static> Functor for Tuple2FirstAppliedBrand<First> {
		/// Maps a function over the second value in the tuple.
		///
		/// This method applies a function to the second value inside the tuple, producing a new tuple with the transformed second value. The first value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the second value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the second value.",
			"The tuple to map over."
		)]
		///
		#[document_returns(
			"A new tuple containing the result of applying the function to the second value."
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
		/// 	map_explicit::<Tuple2FirstAppliedBrand<_>, _, _, _, _>(|x: i32| x * 2, (1, 5)),
		/// 	(1, 10)
		/// );
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(fa.0, func(fa.1))
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> Lift for Tuple2FirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Lifts a binary function into the tuple context (over second).
		///
		/// This method lifts a binary function to operate on the second values within the tuple context. The first values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first second value.",
			"The type of the second second value.",
			"The type of the result second value."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the second values.",
			"The first tuple.",
			"The second tuple."
		)]
		///
		#[document_returns(
			"A new tuple where the first values are combined using `Semigroup::append` and the second values are combined using `f`."
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
		/// 	lift2_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _, _, _>(
		/// 		|x, y| x + y,
		/// 		("a".to_string(), 1),
		/// 		("b".to_string(), 2)
		/// 	),
		/// 	("ab".to_string(), 3)
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
			(Semigroup::append(fa.0, fb.0), func(fa.1, fb.1))
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> Pointed for Tuple2FirstAppliedBrand<First>
	where
		First: Monoid,
	{
		/// Wraps a value in a tuple (with empty first).
		///
		/// This method wraps a value in a tuple, using the `Monoid::empty()` value for the first element.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A tuple containing the empty value of the first type and `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(pure::<Tuple2FirstAppliedBrand<String>, _>(5), ("".to_string(), 5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			(Monoid::empty(), a)
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + Semigroup + 'static> ApplyFirst for Tuple2FirstAppliedBrand<First> {}
	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + Semigroup + 'static> ApplySecond for Tuple2FirstAppliedBrand<First> {}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> Semiapplicative for Tuple2FirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Applies a wrapped function to a wrapped value (over second).
		///
		/// This method applies a function wrapped in a tuple to a value wrapped in a tuple. The first values are combined using their `Semigroup` implementation.
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
			"The tuple containing the function.",
			"The tuple containing the value."
		)]
		///
		#[document_returns(
			"A new tuple where the first values are combined and the function is applied to the second value."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = ("a".to_string(), lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(
		/// 	apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(f, ("b".to_string(), 5)),
		/// 	("ab".to_string(), 10)
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(Semigroup::append(ff.0, fa.0), ff.1(fa.1))
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> Semimonad for Tuple2FirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Chains tuple computations (over second).
		///
		/// This method chains two computations, where the second computation depends on the result of the first. The first values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters("The first tuple.", "The function to apply to the second value.")]
		///
		#[document_returns("A new tuple where the first values are combined.")]
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
		/// 	bind_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(("a".to_string(), 5), |x| (
		/// 		"b".to_string(),
		/// 		x * 2
		/// 	)),
		/// 	("ab".to_string(), 10)
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let (first, second) = ma;
			let (next_first, next_second) = func(second);
			(Semigroup::append(first, next_first), next_second)
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> MonadRec for Tuple2FirstAppliedBrand<First>
	where
		First: Monoid,
	{
		/// Performs tail-recursive monadic computation over a tuple (varying the second element).
		///
		/// Iteratively applies the step function, accumulating the first element
		/// via `Semigroup::append` at each iteration. When the step function returns
		/// `ControlFlow::Break`, the accumulated first element and the final result are returned.
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
			"A tuple with the accumulated first value and the result of the computation."
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
		/// let result = tail_rec_m::<Tuple2FirstAppliedBrand<String>, _, _>(
		/// 	|n| {
		/// 		if n < 3 {
		/// 			(format!("{n},"), ControlFlow::Continue(n + 1))
		/// 		} else {
		/// 			(format!("{n}"), ControlFlow::Break(n))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, ("0,1,2,3".to_string(), 3));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut acc: First = Monoid::empty();
			let mut current = initial;
			loop {
				let (first, step) = func(current);
				acc = Semigroup::append(acc, first);
				match step {
					ControlFlow::Continue(next) => current = next,
					ControlFlow::Break(b) => return (acc, b),
				}
			}
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: 'static> Foldable for Tuple2FirstAppliedBrand<First> {
		/// Folds the tuple from the right (over second).
		///
		/// This method performs a right-associative fold of the tuple (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The tuple to fold.")]
		///
		#[document_returns("`func(a, initial)`.")]
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
		/// 	fold_right_explicit::<RcFnBrand, Tuple2FirstAppliedBrand<()>, _, _, _, _>(
		/// 		|x, acc| x + acc,
		/// 		0,
		/// 		((), 5)
		/// 	),
		/// 	5
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			func(fa.1, initial)
		}

		/// Folds the tuple from the left (over second).
		///
		/// This method performs a left-associative fold of the tuple (over second).
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
			"The tuple to fold."
		)]
		///
		#[document_returns("`func(initial, a)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left_explicit::<RcFnBrand, Tuple2FirstAppliedBrand<()>, _, _, _, _>(
		/// 		|acc, x| acc + x,
		/// 		0,
		/// 		((), 5)
		/// 	),
		/// 	5
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			func(initial, fa.1)
		}

		/// Maps the value to a monoid and returns it (over second).
		///
		/// This method maps the element of the tuple to a monoid and then returns it (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The tuple to fold.")]
		///
		#[document_returns("`func(a)`.")]
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
		/// 	fold_map_explicit::<RcFnBrand, Tuple2FirstAppliedBrand<()>, _, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		((), 5)
		/// 	),
		/// 	"5".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			func(fa.1)
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> Traversable for Tuple2FirstAppliedBrand<First> {
		/// Traverses the tuple with an applicative function (over second).
		///
		/// This method maps the element of the tuple to a computation, evaluates it, and combines the result into an applicative context (over second).
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
			"The tuple to traverse."
		)]
		///
		#[document_returns("The tuple wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse_explicit::<RcFnBrand, Tuple2FirstAppliedBrand<()>, _, _, OptionBrand, _, _>(
		/// 		|x| Some(x * 2),
		/// 		((), 5)
		/// 	),
		/// 	Some(((), 10))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let (first, second) = ta;
			F::map(move |b| (first.clone(), b), func(second))
		}

		/// Sequences a tuple of applicative (over second).
		///
		/// This method evaluates the computation inside the tuple and accumulates the result into an applicative context (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The tuple containing the applicative value.")]
		///
		#[document_returns("The tuple wrapped in the applicative context.")]
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
		/// 	sequence::<Tuple2FirstAppliedBrand<()>, _, OptionBrand>(((), Some(5))),
		/// 	Some(((), 5))
		/// );
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			let (first, second) = ta;
			F::map(move |a| (first.clone(), a), second)
		}
	}

	// -- By-reference trait implementations for Tuple2FirstAppliedBrand --

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> RefFunctor for Tuple2FirstAppliedBrand<First> {
		/// Maps a function over the second value in the tuple by reference.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function.", "The tuple.")]
		#[document_returns("A new tuple with the mapped second value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// assert_eq!(
		/// 	map_explicit::<Tuple2FirstAppliedBrand<_>, _, _, _, _>(|x: &i32| *x * 2, &(1, 5)),
		/// 	(1, 10)
		/// );
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(fa.0.clone(), func(&fa.1))
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> RefFoldable for Tuple2FirstAppliedBrand<First> {
		/// Folds the tuple by reference (over second).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The element type.",
			"The monoid type."
		)]
		#[document_parameters("The mapping function.", "The tuple.")]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = fold_map_explicit::<RcFnBrand, Tuple2FirstAppliedBrand<()>, _, _, _, _>(
		/// 	|x: &i32| x.to_string(),
		/// 	&((), 5),
		/// );
		/// assert_eq!(result, "5");
		/// ```
		fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(&A) -> M + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: LiftFn + 'a,
			M: Monoid + 'a, {
			func(&fa.1)
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> RefTraversable for Tuple2FirstAppliedBrand<First> {
		/// Traverses the tuple by reference (over second).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The input type.",
			"The output type.",
			"The applicative."
		)]
		#[document_parameters("The function.", "The tuple.")]
		#[document_returns("The traversed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result: Option<((), String)> =
		/// 	ref_traverse::<Tuple2FirstAppliedBrand<()>, RcFnBrand, _, _, OptionBrand>(
		/// 		|x: &i32| Some(x.to_string()),
		/// 		&((), 42),
		/// 	);
		/// assert_eq!(result, Some(((), "42".to_string())));
		/// ```
		fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let first = ta.0.clone();
			F::map(move |b| (first.clone(), b), func(&ta.1))
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> RefPointed for Tuple2FirstAppliedBrand<First>
	where
		First: Monoid,
	{
		/// Creates a tuple from a reference by cloning (with empty first).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The value type.")]
		#[document_parameters("The reference to wrap.")]
		#[document_returns("A tuple containing `Monoid::empty()` and a clone of the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = 42;
		/// let result: (String, i32) = ref_pure::<Tuple2FirstAppliedBrand<String>, _>(&x);
		/// assert_eq!(result, ("".to_string(), 42));
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			(Monoid::empty(), a.clone())
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> RefLift for Tuple2FirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Combines two tuples with a by-reference binary function (over second).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "First input.", "Second input.", "Output.")]
		#[document_parameters("The binary function.", "The first tuple.", "The second tuple.")]
		#[document_returns("A tuple with combined first values and the function result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = lift2_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32| *a + *b,
		/// 	&("a".to_string(), 1),
		/// 	&("b".to_string(), 2),
		/// );
		/// assert_eq!(result, ("ab".to_string(), 3));
		/// ```
		fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
			func: impl Fn(&A, &B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			(Semigroup::append(fa.0.clone(), fb.0.clone()), func(&fa.1, &fb.1))
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> RefSemiapplicative for Tuple2FirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Applies a wrapped by-ref function to a tuple value (over second).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The function brand.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters(
			"The tuple containing the function.",
			"The tuple containing the value."
		)]
		#[document_returns("A tuple with combined first values and the function result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f: std::rc::Rc<dyn Fn(&i32) -> i32> = std::rc::Rc::new(|x: &i32| *x * 2);
		/// let result = ref_apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(
		/// 	&("a".to_string(), f),
		/// 	&("b".to_string(), 5),
		/// );
		/// assert_eq!(result, ("ab".to_string(), 10));
		/// ```
		fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, A: 'a, B: 'a>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(Semigroup::append(ff.0.clone(), fa.0.clone()), (*ff.1)(&fa.1))
		}
	}

	#[document_type_parameters("The type of the first value in the tuple.")]
	impl<First: Clone + 'static> RefSemimonad for Tuple2FirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Chains tuple computations by reference (over second).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The input tuple.", "The function to apply by reference.")]
		#[document_returns("A tuple with combined first values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result: (String, String) = bind_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(
		/// 	&("a".to_string(), 42),
		/// 	|x: &i32| ("b".to_string(), x.to_string()),
		/// );
		/// assert_eq!(result, ("ab".to_string(), "42".to_string()));
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let (next_first, next_second) = f(&fa.1);
			(Semigroup::append(fa.0.clone(), next_first), next_second)
		}
	}

	// Tuple2SecondAppliedBrand<Second> (Functor over First)

	impl_kind! {
		impl<Second: 'static> for Tuple2SecondAppliedBrand<Second> {
			type Of<'a, A: 'a>: 'a = (A, Second);
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: 'static> Functor for Tuple2SecondAppliedBrand<Second> {
		/// Maps a function over the first value in the tuple.
		///
		/// This method applies a function to the first value inside the tuple, producing a new tuple with the transformed first value. The second value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the first value.",
			"The tuple to map over."
		)]
		///
		#[document_returns(
			"A new tuple containing the result of applying the function to the first value."
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
		/// 	map_explicit::<Tuple2SecondAppliedBrand<_>, _, _, _, _>(|x: i32| x * 2, (5, 1)),
		/// 	(10, 1)
		/// );
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(func(fa.0), fa.1)
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> Lift for Tuple2SecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Lifts a binary function into the tuple context (over first).
		///
		/// This method lifts a binary function to operate on the first values within the tuple context. The second values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first first value.",
			"The type of the second first value.",
			"The type of the result first value."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the first values.",
			"The first tuple.",
			"The second tuple."
		)]
		///
		#[document_returns(
			"A new tuple where the first values are combined using `f` and the second values are combined using `Semigroup::append`."
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
		/// 	lift2_explicit::<Tuple2SecondAppliedBrand<String>, _, _, _, _, _, _>(
		/// 		|x, y| x + y,
		/// 		(1, "a".to_string()),
		/// 		(2, "b".to_string())
		/// 	),
		/// 	(3, "ab".to_string())
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
			(func(fa.0, fb.0), Semigroup::append(fa.1, fb.1))
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> Pointed for Tuple2SecondAppliedBrand<Second>
	where
		Second: Monoid,
	{
		/// Wraps a value in a tuple (with empty second).
		///
		/// This method wraps a value in a tuple, using the `Monoid::empty()` value for the second element.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A tuple containing `a` and the empty value of the second type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// assert_eq!(pure::<Tuple2SecondAppliedBrand<String>, _>(5), (5, "".to_string()));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			(a, Monoid::empty())
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + Semigroup + 'static> ApplyFirst for Tuple2SecondAppliedBrand<Second> {}
	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + Semigroup + 'static> ApplySecond for Tuple2SecondAppliedBrand<Second> {}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> Semiapplicative for Tuple2SecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Applies a wrapped function to a wrapped value (over first).
		///
		/// This method applies a function wrapped in a tuple to a value wrapped in a tuple. The second values are combined using their `Semigroup` implementation.
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
			"The tuple containing the function.",
			"The tuple containing the value."
		)]
		///
		#[document_returns(
			"A new tuple where the function is applied to the first value and the second values are combined."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = (lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2), "a".to_string());
		/// assert_eq!(
		/// 	apply::<RcFnBrand, Tuple2SecondAppliedBrand<String>, _, _>(f, (5, "b".to_string())),
		/// 	(10, "ab".to_string())
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(ff.0(fa.0), Semigroup::append(ff.1, fa.1))
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> Semimonad for Tuple2SecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Chains tuple computations (over first).
		///
		/// This method chains two computations, where the second computation depends on the result of the first. The second values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters("The first tuple.", "The function to apply to the first value.")]
		///
		#[document_returns("A new tuple where the second values are combined.")]
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
		/// 	bind_explicit::<Tuple2SecondAppliedBrand<String>, _, _, _, _>((5, "a".to_string()), |x| (
		/// 		x * 2,
		/// 		"b".to_string()
		/// 	)),
		/// 	(10, "ab".to_string())
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let (first, second) = ma;
			let (next_first, next_second) = func(first);
			(next_first, Semigroup::append(second, next_second))
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> MonadRec for Tuple2SecondAppliedBrand<Second>
	where
		Second: Monoid,
	{
		/// Performs tail-recursive monadic computation over a tuple (varying the first element).
		///
		/// Iteratively applies the step function, accumulating the second element
		/// via `Semigroup::append` at each iteration. When the step function returns
		/// `ControlFlow::Break`, the final result and the accumulated second element are returned.
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
			"A tuple with the result of the computation and the accumulated second value."
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
		/// let result = tail_rec_m::<Tuple2SecondAppliedBrand<String>, _, _>(
		/// 	|n| {
		/// 		if n < 3 {
		/// 			(ControlFlow::Continue(n + 1), format!("{n},"))
		/// 		} else {
		/// 			(ControlFlow::Break(n), format!("{n}"))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, (3, "0,1,2,3".to_string()));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut acc: Second = Monoid::empty();
			let mut current = initial;
			loop {
				let (step, second) = func(current);
				acc = Semigroup::append(acc, second);
				match step {
					ControlFlow::Continue(next) => current = next,
					ControlFlow::Break(b) => return (b, acc),
				}
			}
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: 'static> Foldable for Tuple2SecondAppliedBrand<Second> {
		/// Folds the tuple from the right (over first).
		///
		/// This method performs a right-associative fold of the tuple (over first).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The tuple to fold.")]
		///
		#[document_returns("`func(a, initial)`.")]
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
		/// 	fold_right_explicit::<RcFnBrand, Tuple2SecondAppliedBrand<()>, _, _, _, _>(
		/// 		|x, acc| x + acc,
		/// 		0,
		/// 		(5, ())
		/// 	),
		/// 	5
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			func(fa.0, initial)
		}

		/// Folds the tuple from the left (over first).
		///
		/// This method performs a left-associative fold of the tuple (over first).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The tuple to fold.")]
		///
		#[document_returns("`func(initial, a)`.")]
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
		/// 	fold_left_explicit::<RcFnBrand, Tuple2SecondAppliedBrand<()>, _, _, _, _>(
		/// 		|acc, x| acc + x,
		/// 		0,
		/// 		(5, ())
		/// 	),
		/// 	5
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			func(initial, fa.0)
		}

		/// Maps the value to a monoid and returns it (over first).
		///
		/// This method maps the element of the tuple to a monoid and then returns it (over first).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The tuple to fold.")]
		///
		#[document_returns("`func(a)`.")]
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
		/// 	fold_map_explicit::<RcFnBrand, Tuple2SecondAppliedBrand<()>, _, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		(5, ())
		/// 	),
		/// 	"5".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			func(fa.0)
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> Traversable for Tuple2SecondAppliedBrand<Second> {
		/// Traverses the tuple with an applicative function (over first).
		///
		/// This method maps the element of the tuple to a computation, evaluates it, and combines the result into an applicative context (over first).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The function to apply.", "The tuple to traverse.")]
		///
		#[document_returns("The tuple wrapped in the applicative context.")]
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
		/// 	traverse_explicit::<RcFnBrand, Tuple2SecondAppliedBrand<()>, _, _, OptionBrand, _, _>(
		/// 		|x| Some(x * 2),
		/// 		(5, ())
		/// 	),
		/// 	Some((10, ()))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let (first, second) = ta;
			F::map(move |b| (b, second.clone()), func(first))
		}

		/// Sequences a tuple of applicative (over first).
		///
		/// This method evaluates the computation inside the tuple and accumulates the result into an applicative context (over first).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The tuple containing the applicative value.")]
		///
		#[document_returns("The tuple wrapped in the applicative context.")]
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
		/// 	sequence::<Tuple2SecondAppliedBrand<()>, _, OptionBrand>((Some(5), ())),
		/// 	Some((5, ()))
		/// );
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			let (first, second) = ta;
			F::map(move |a| (a, second.clone()), first)
		}
	}
	// -- By-reference trait implementations for Tuple2SecondAppliedBrand --

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> RefFunctor for Tuple2SecondAppliedBrand<Second> {
		/// Maps a function over the first value in the tuple by reference.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function.", "The tuple.")]
		#[document_returns("A new tuple with the mapped first value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// assert_eq!(
		/// 	map_explicit::<Tuple2SecondAppliedBrand<_>, _, _, _, _>(|x: &i32| *x * 2, &(5, 1)),
		/// 	(10, 1)
		/// );
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			(func(&fa.0), fa.1.clone())
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> RefFoldable for Tuple2SecondAppliedBrand<Second> {
		/// Folds the tuple by reference (over first).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The element type.",
			"The monoid type."
		)]
		#[document_parameters("The mapping function.", "The tuple.")]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = fold_map_explicit::<RcFnBrand, Tuple2SecondAppliedBrand<()>, _, _, _, _>(
		/// 	|x: &i32| x.to_string(),
		/// 	&(5, ()),
		/// );
		/// assert_eq!(result, "5");
		/// ```
		fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(&A) -> M + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: LiftFn + 'a,
			M: Monoid + 'a, {
			func(&fa.0)
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> RefTraversable for Tuple2SecondAppliedBrand<Second> {
		/// Traverses the tuple by reference (over first).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The input type.",
			"The output type.",
			"The applicative."
		)]
		#[document_parameters("The function.", "The tuple.")]
		#[document_returns("The traversed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result: Option<(String, ())> =
		/// 	ref_traverse::<Tuple2SecondAppliedBrand<()>, RcFnBrand, _, _, OptionBrand>(
		/// 		|x: &i32| Some(x.to_string()),
		/// 		&(42, ()),
		/// 	);
		/// assert_eq!(result, Some(("42".to_string(), ())));
		/// ```
		fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let second = ta.1.clone();
			F::map(move |a| (a, second.clone()), func(&ta.0))
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> RefPointed for Tuple2SecondAppliedBrand<Second>
	where
		Second: Monoid,
	{
		/// Creates a tuple from a reference by cloning (with empty second).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The value type.")]
		#[document_parameters("The reference to wrap.")]
		#[document_returns("A tuple containing a clone of the value and `Monoid::empty()`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = 42;
		/// let result: (i32, String) = ref_pure::<Tuple2SecondAppliedBrand<String>, _>(&x);
		/// assert_eq!(result, (42, "".to_string()));
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			(a.clone(), Monoid::empty())
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> RefLift for Tuple2SecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Combines two tuples with a by-reference binary function (over first).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "First input.", "Second input.", "Output.")]
		#[document_parameters("The binary function.", "The first tuple.", "The second tuple.")]
		#[document_returns("A tuple with the function result and combined second values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = lift2_explicit::<Tuple2SecondAppliedBrand<String>, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32| *a + *b,
		/// 	&(1, "a".to_string()),
		/// 	&(2, "b".to_string()),
		/// );
		/// assert_eq!(result, (3, "ab".to_string()));
		/// ```
		fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
			func: impl Fn(&A, &B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			(func(&fa.0, &fb.0), Semigroup::append(fa.1.clone(), fb.1.clone()))
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> RefSemiapplicative for Tuple2SecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Applies a wrapped by-ref function to a tuple value (over first).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The function brand.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters(
			"The tuple containing the function.",
			"The tuple containing the value."
		)]
		#[document_returns("A tuple with the function result and combined second values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f: std::rc::Rc<dyn Fn(&i32) -> i32> = std::rc::Rc::new(|x: &i32| *x * 2);
		/// let result = ref_apply::<RcFnBrand, Tuple2SecondAppliedBrand<String>, _, _>(
		/// 	&(f, "a".to_string()),
		/// 	&(5, "b".to_string()),
		/// );
		/// assert_eq!(result, (10, "ab".to_string()));
		/// ```
		fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, A: 'a, B: 'a>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			((*ff.0)(&fa.0), Semigroup::append(ff.1.clone(), fa.1.clone()))
		}
	}

	#[document_type_parameters("The type of the second value in the tuple.")]
	impl<Second: Clone + 'static> RefSemimonad for Tuple2SecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Chains tuple computations by reference (over first).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The input tuple.", "The function to apply by reference.")]
		#[document_returns("A tuple with combined second values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result: (String, String) = bind_explicit::<Tuple2SecondAppliedBrand<String>, _, _, _, _>(
		/// 	&(42, "a".to_string()),
		/// 	|x: &i32| (x.to_string(), "b".to_string()),
		/// );
		/// assert_eq!(result, ("42".to_string(), "ab".to_string()));
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let (next_first, next_second) = f(&fa.0);
			(next_first, Semigroup::append(fa.1.clone(), next_second))
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
		core::ops::ControlFlow,
		quickcheck_macros::quickcheck,
	};

	// Bifunctor Tests

	/// Tests `bimap` on `Tuple2`.
	#[test]
	fn test_bimap() {
		let x = (1, 5);
		assert_eq!(bimap::<Tuple2Brand, _, _, _, _, _, _>((|a| a + 1, |b| b * 2), x), (2, 10));
	}

	// Bifunctor Laws

	/// Tests the identity law for Bifunctor.
	#[quickcheck]
	fn bifunctor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = (first, second);
		bimap::<Tuple2Brand, _, _, _, _, _, _>((identity, identity), x.clone()) == x
	}

	/// Tests the composition law for Bifunctor.
	#[quickcheck]
	fn bifunctor_composition(
		first: i32,
		second: i32,
	) -> bool {
		let x = (first, second);
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		let h = |x: i32| x.wrapping_sub(1);
		let i = |x: i32| if x == 0 { 0 } else { x.wrapping_div(2) };

		bimap::<Tuple2Brand, _, _, _, _, _, _>((compose(f, g), compose(h, i)), x)
			== bimap::<Tuple2Brand, _, _, _, _, _, _>(
				(f, h),
				bimap::<Tuple2Brand, _, _, _, _, _, _>((g, i), x),
			)
	}

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = (first, second);
		map_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(identity, x.clone()) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(
		first: String,
		second: i32,
	) -> bool {
		let x = (first, second);
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(compose(f, g), x.clone())
			== map_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(
				f,
				map_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(g, x),
			)
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(
		first: String,
		second: i32,
	) -> bool {
		let v = (first, second);
		apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(
			pure::<Tuple2FirstAppliedBrand<String>, _>(<RcFnBrand as LiftFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(
			pure::<Tuple2FirstAppliedBrand<String>, _>(<RcFnBrand as LiftFn>::new(f)),
			pure::<Tuple2FirstAppliedBrand<String>, _>(x),
		) == pure::<Tuple2FirstAppliedBrand<String>, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w_first: String,
		w_second: i32,
		u_seed: i32,
		v_seed: i32,
	) -> bool {
		let w = (w_first, w_second);

		let u_fn = <RcFnBrand as LiftFn>::new(move |x: i32| x.wrapping_add(u_seed));
		let u = pure::<Tuple2FirstAppliedBrand<String>, _>(u_fn);

		let v_fn = <RcFnBrand as LiftFn>::new(move |x: i32| x.wrapping_mul(v_seed));
		let v = pure::<Tuple2FirstAppliedBrand<String>, _>(v_fn);

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		let compose_fn = <RcFnBrand as LiftFn>::new(|f: std::rc::Rc<dyn Fn(i32) -> i32>| {
			let f = f.clone();
			<RcFnBrand as LiftFn>::new(move |g: std::rc::Rc<dyn Fn(i32) -> i32>| {
				let f = f.clone();
				let g = g.clone();
				<RcFnBrand as LiftFn>::new(move |x| f(g(x)))
			})
		});

		let pure_compose = pure::<Tuple2FirstAppliedBrand<String>, _>(compose_fn);
		let u_applied = apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(pure_compose, u);
		let uv = apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(u_applied, v);
		let lhs = apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(
		y: i32,
		u_seed: i32,
	) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = move |x: i32| x.wrapping_mul(u_seed);
		let u = pure::<Tuple2FirstAppliedBrand<String>, _>(<RcFnBrand as LiftFn>::new(f));

		let lhs = apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(
			u.clone(),
			pure::<Tuple2FirstAppliedBrand<String>, _>(y),
		);

		let rhs_fn = <RcFnBrand as LiftFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, Tuple2FirstAppliedBrand<String>, _, _>(
			pure::<Tuple2FirstAppliedBrand<String>, _>(rhs_fn),
			u,
		);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| ("f".to_string(), x.wrapping_mul(2));
		bind_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(
			pure::<Tuple2FirstAppliedBrand<String>, _>(a),
			f,
		) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(
		first: String,
		second: i32,
	) -> bool {
		let m = (first, second);
		bind_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(
			m.clone(),
			pure::<Tuple2FirstAppliedBrand<String>, _>,
		) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(
		first: String,
		second: i32,
	) -> bool {
		let m = (first, second);
		let f = |x: i32| ("f".to_string(), x.wrapping_mul(2));
		let g = |x: i32| ("g".to_string(), x.wrapping_add(1));
		bind_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(
			bind_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(m.clone(), f),
			g,
		) == bind_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(m, |x| {
			bind_explicit::<Tuple2FirstAppliedBrand<String>, _, _, _, _>(f(x), g)
		})
	}

	// MonadRec tests (Tuple2FirstAppliedBrand)

	/// Tests the MonadRec identity law for Tuple2FirstAppliedBrand:
	/// `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`.
	#[quickcheck]
	fn monad_rec_first_identity(x: i32) -> bool {
		tail_rec_m::<Tuple2FirstAppliedBrand<String>, _, _>(
			|a| (String::new(), ControlFlow::Break(a)),
			x,
		) == pure::<Tuple2FirstAppliedBrand<String>, _>(x)
	}

	/// Tests a recursive computation that accumulates the first element.
	#[test]
	fn monad_rec_first_accumulation() {
		let result = tail_rec_m::<Tuple2FirstAppliedBrand<String>, _, _>(
			|n: i32| {
				if n < 3 {
					(format!("{n},"), ControlFlow::Continue(n + 1))
				} else {
					(format!("{n}"), ControlFlow::Break(n))
				}
			},
			0,
		);
		assert_eq!(result, ("0,1,2,3".to_string(), 3));
	}

	/// Tests stack safety for Tuple2FirstAppliedBrand: `tail_rec_m` handles large iteration counts.
	#[test]
	fn monad_rec_first_stack_safety() {
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<Tuple2FirstAppliedBrand<String>, _, _>(
			|acc| {
				if acc < iterations {
					(String::new(), ControlFlow::Continue(acc + 1))
				} else {
					(String::new(), ControlFlow::Break(acc))
				}
			},
			0i64,
		);
		assert_eq!(result, (String::new(), iterations));
	}

	// MonadRec tests (Tuple2SecondAppliedBrand)

	/// Tests the MonadRec identity law for Tuple2SecondAppliedBrand:
	/// `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`.
	#[quickcheck]
	fn monad_rec_second_identity(x: i32) -> bool {
		tail_rec_m::<Tuple2SecondAppliedBrand<String>, _, _>(
			|a| (ControlFlow::Break(a), String::new()),
			x,
		) == pure::<Tuple2SecondAppliedBrand<String>, _>(x)
	}

	/// Tests a recursive computation that accumulates the second element.
	#[test]
	fn monad_rec_second_accumulation() {
		let result = tail_rec_m::<Tuple2SecondAppliedBrand<String>, _, _>(
			|n: i32| {
				if n < 3 {
					(ControlFlow::Continue(n + 1), format!("{n},"))
				} else {
					(ControlFlow::Break(n), format!("{n}"))
				}
			},
			0,
		);
		assert_eq!(result, (3, "0,1,2,3".to_string()));
	}

	/// Tests stack safety for Tuple2SecondAppliedBrand: `tail_rec_m` handles large iteration counts.
	#[test]
	fn monad_rec_second_stack_safety() {
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<Tuple2SecondAppliedBrand<String>, _, _>(
			|acc| {
				if acc < iterations {
					(ControlFlow::Continue(acc + 1), String::new())
				} else {
					(ControlFlow::Break(acc), String::new())
				}
			},
			0i64,
		);
		assert_eq!(result, (iterations, String::new()));
	}
}
