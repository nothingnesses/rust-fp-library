//! Two-value tuple with [`Bifunctor`] and dual [`Functor`] instances.
//!
//! Can be used as a bifunctor over both values, or as a functor/monad by fixing either the first value [`Tuple2WithFirstBrand`] or second value [`Tuple2WithSecondBrand`].

use crate::{
	Apply,
	brands::{Tuple2Brand, Tuple2WithFirstBrand, Tuple2WithSecondBrand},
	classes::{
		Applicative, ApplyFirst, ApplySecond, Bifunctor, CloneableFn, Foldable, Functor, Lift,
		Monoid, ParFoldable, Pointed, Semiapplicative, Semigroup, Semimonad, SendCloneableFn,
		Traversable,
	},
	impl_kind,
	kinds::*,
};
use fp_macros::{doc_params, doc_type_params, hm_signature};

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
	///
	/// ### Type Signature
	///
	#[hm_signature(Bifunctor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the first value.",
		"The type of the mapped first value.",
		"The type of the second value.",
		"The type of the mapped second value.",
		"The type of the function to apply to the first value.",
		"The type of the function to apply to the second value."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The function to apply to the first value.",
		"The function to apply to the second value.",
		"The tuple to map over."
	)]
	///
	/// ### Returns
	///
	/// A new tuple containing the mapped values.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::bifunctor::*, functions::*};
	///
	/// let x = (1, 5);
	/// assert_eq!(bimap::<Tuple2Brand, _, _, _, _, _, _>(|a| a + 1, |b| b * 2, x), (2, 10));
	/// ```
	fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, F, G>(
		f: F,
		g: G,
		p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
	) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
	where
		F: Fn(A) -> B + 'a,
		G: Fn(C) -> D + 'a,
	{
		(f(p.0), g(p.1))
	}
}

// Tuple2WithFirstBrand<First> (Functor over Second)

impl_kind! {
	impl<First: 'static> for Tuple2WithFirstBrand<First> {
		type Of<'a, A: 'a>: 'a = (First, A);
	}
}

impl<First: 'static> Functor for Tuple2WithFirstBrand<First> {
	/// Maps a function over the second value in the tuple.
	///
	/// This method applies a function to the second value inside the tuple, producing a new tuple with the transformed second value. The first value remains unchanged.
	///
	/// ### Type Signature
	///
	#[hm_signature(Functor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the second value.",
		"The type of the result of applying the function.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply to the second value.", "The tuple to map over.")]
	///
	/// ### Returns
	///
	/// A new tuple containing the result of applying the function to the second value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(map::<Tuple2WithFirstBrand<_>, _, _, _>(|x: i32| x * 2, (1, 5)), (1, 10));
	/// ```
	fn map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> B + 'a,
	{
		(fa.0, func(fa.1))
	}
}

impl<First: Clone + 'static> Lift for Tuple2WithFirstBrand<First>
where
	First: Semigroup,
{
	/// Lifts a binary function into the tuple context (over second).
	///
	/// This method lifts a binary function to operate on the second values within the tuple context. The first values are combined using their `Semigroup` implementation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Lift)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the first second value.",
		"The type of the second second value.",
		"The type of the result second value.",
		"The type of the binary function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The binary function to apply to the second values.",
		"The first tuple.",
		"The second tuple."
	)]
	///
	/// ### Returns
	///
	/// A new tuple where the first values are combined using `Semigroup::append` and the second values are combined using `f`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     lift2::<Tuple2WithFirstBrand<String>, _, _, _, _>(|x, y| x + y, ("a".to_string(), 1), ("b".to_string(), 2)),
	///     ("ab".to_string(), 3)
	/// );
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
		C: 'a,
	{
		(Semigroup::append(fa.0, fb.0), func(fa.1, fb.1))
	}
}

impl<First: Clone + 'static> Pointed for Tuple2WithFirstBrand<First>
where
	First: Monoid,
{
	/// Wraps a value in a tuple (with empty first).
	///
	/// This method wraps a value in a tuple, using the `Monoid::empty()` value for the first element.
	///
	/// ### Type Signature
	///
	#[hm_signature(Pointed)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the value.", "The type of the value to wrap.")]
	///
	/// ### Parameters
	///
	#[doc_params("The value to wrap.")]
	///
	/// ### Returns
	///
	/// A tuple containing the empty value of the first type and `a`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(pure::<Tuple2WithFirstBrand<String>, _>(5), ("".to_string(), 5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		(Monoid::empty(), a)
	}
}

impl<First: Clone + Semigroup + 'static> ApplyFirst for Tuple2WithFirstBrand<First> {}
impl<First: Clone + Semigroup + 'static> ApplySecond for Tuple2WithFirstBrand<First> {}

impl<First: Clone + 'static> Semiapplicative for Tuple2WithFirstBrand<First>
where
	First: Semigroup,
{
	/// Applies a wrapped function to a wrapped value (over second).
	///
	/// This method applies a function wrapped in a tuple to a value wrapped in a tuple. The first values are combined using their `Semigroup` implementation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semiapplicative)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function wrapper.",
		"The type of the input value.",
		"The type of the output value."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The tuple containing the function.", "The tuple containing the value.")]
	///
	/// ### Returns
	///
	/// A new tuple where the first values are combined and the function is applied to the second value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = ("a".to_string(), cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// assert_eq!(apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(f, ("b".to_string(), 5)), ("ab".to_string(), 10));
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		(Semigroup::append(ff.0, fa.0), ff.1(fa.1))
	}
}

impl<First: Clone + 'static> Semimonad for Tuple2WithFirstBrand<First>
where
	First: Semigroup,
{
	/// Chains tuple computations (over second).
	///
	/// This method chains two computations, where the second computation depends on the result of the first. The first values are combined using their `Semigroup` implementation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semimonad)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the result of the first computation.",
		"The type of the result of the second computation.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The first tuple.", "The function to apply to the second value.")]
	///
	/// ### Returns
	///
	/// A new tuple where the first values are combined.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     bind::<Tuple2WithFirstBrand<String>, _, _, _>(("a".to_string(), 5), |x| ("b".to_string(), x * 2)),
	///     ("ab".to_string(), 10)
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, Func>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		let (first, second) = ma;
		let (next_first, next_second) = func(second);
		(Semigroup::append(first, next_first), next_second)
	}
}

impl<First: 'static> Foldable for Tuple2WithFirstBrand<First> {
	/// Folds the tuple from the right (over second).
	///
	/// This method performs a right-associative fold of the tuple (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The folding function.", "The initial value.", "The tuple to fold.")]
	///
	/// ### Returns
	///
	/// `func(a, initial)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(fold_right::<RcFnBrand, Tuple2WithFirstBrand<()>, _, _, _>(|x, acc| x + acc, 0, ((), 5)), 5);
	/// ```
	fn fold_right<'a, FnBrand, A: 'a, B: 'a, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(A, B) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(fa.1, initial)
	}

	/// Folds the tuple from the left (over second).
	///
	/// This method performs a left-associative fold of the tuple (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The function to apply to the accumulator and each element.",
		"The initial value of the accumulator.",
		"The tuple to fold."
	)]
	///
	/// ### Returns
	///
	/// `func(initial, a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(fold_left::<RcFnBrand, Tuple2WithFirstBrand<()>, _, _, _>(|acc, x| acc + x, 0, ((), 5)), 5);
	/// ```
	fn fold_left<'a, FnBrand, A: 'a, B: 'a, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(B, A) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(initial, fa.1)
	}

	/// Maps the value to a monoid and returns it (over second).
	///
	/// This method maps the element of the tuple to a monoid and then returns it (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the monoid.",
		"The type of the mapping function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The mapping function.", "The tuple to fold.")]
	///
	/// ### Returns
	///
	/// `func(a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, Tuple2WithFirstBrand<()>, _, _, _>(|x: i32| x.to_string(), ((), 5)),
	///     "5".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, A: 'a, M, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(fa.1)
	}
}

impl<First: Clone + 'static> Traversable for Tuple2WithFirstBrand<First> {
	/// Traverses the tuple with an applicative function (over second).
	///
	/// This method maps the element of the tuple to a computation, evaluates it, and combines the result into an applicative context (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The function to apply to each element, returning a value in an applicative context.",
		"The tuple to traverse."
	)]
	///
	/// ### Returns
	///
	/// The tuple wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     traverse::<Tuple2WithFirstBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), ((), 5)),
	///     Some(((), 10))
	/// );
	/// ```
	fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		let (first, second) = ta;
		F::map(move |b| (first.clone(), b), func(second))
	}
	/// Sequences a tuple of applicative (over second).
	///
	/// This method evaluates the computation inside the tuple and accumulates the result into an applicative context (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The tuple containing the applicative value.")]
	///
	/// ### Returns
	///
	/// The tuple wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     sequence::<Tuple2WithFirstBrand<()>, _, OptionBrand>(((), Some(5))),
	///     Some(((), 5))
	/// );
	/// ```
	fn sequence<'a, A: 'a + Clone, F: Applicative>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		let (first, second) = ta;
		F::map(move |a| (first.clone(), a), second)
	}
}

impl<First: 'static> ParFoldable for Tuple2WithFirstBrand<First> {
	/// Maps the value to a monoid and returns it in parallel (over second).
	///
	/// This method maps the element of the tuple to a monoid and then returns it (over second). The mapping operation may be executed in parallel.
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function wrapper.",
		"The element type.",
		"The monoid type."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The thread-safe function to map each element to a monoid.", "The tuple to fold.")]
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = ("a".to_string(), 1);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// assert_eq!(
	///     par_fold_map::<ArcFnBrand, Tuple2WithFirstBrand<String>, _, _>(f, x),
	///     "1".to_string()
	/// );
	/// ```
	fn par_fold_map<'a, FnBrand, A, M>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		func(fa.1)
	}

	/// Folds the tuple from the right in parallel (over second).
	///
	/// This method folds the tuple by applying a function from right to left, potentially in parallel (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function wrapper.",
		"The element type.",
		"The accumulator type."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The thread-safe function to apply to each element and the accumulator.",
		"The initial value.",
		"The tuple to fold."
	)]
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = ("a".to_string(), 1);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
	/// assert_eq!(par_fold_right::<ArcFnBrand, Tuple2WithFirstBrand<String>, _, _>(f, 10, x), 11);
	/// ```
	fn par_fold_right<'a, FnBrand, A, B>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
	{
		func((fa.1, initial))
	}
}

// Tuple2WithSecondBrand<Second> (Functor over First)

impl_kind! {
	impl<Second: 'static> for Tuple2WithSecondBrand<Second> {
		type Of<'a, A: 'a>: 'a = (A, Second);
	}
}

impl<Second: 'static> Functor for Tuple2WithSecondBrand<Second> {
	/// Maps a function over the first value in the tuple.
	///
	/// This method applies a function to the first value inside the tuple, producing a new tuple with the transformed first value. The second value remains unchanged.
	///
	/// ### Type Signature
	///
	#[hm_signature(Functor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the first value.",
		"The type of the result of applying the function.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply to the first value.", "The tuple to map over.")]
	///
	/// ### Returns
	///
	/// A new tuple containing the result of applying the function to the first value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(map::<Tuple2WithSecondBrand<_>, _, _, _>(|x: i32| x * 2, (5, 1)), (10, 1));
	/// ```
	fn map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> B + 'a,
	{
		(func(fa.0), fa.1)
	}
}

impl<Second: Clone + 'static> Lift for Tuple2WithSecondBrand<Second>
where
	Second: Semigroup,
{
	/// Lifts a binary function into the tuple context (over first).
	///
	/// This method lifts a binary function to operate on the first values within the tuple context. The second values are combined using their `Semigroup` implementation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Lift)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the first first value.",
		"The type of the second first value.",
		"The type of the result first value.",
		"The type of the binary function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The binary function to apply to the first values.",
		"The first tuple.",
		"The second tuple."
	)]
	///
	/// ### Returns
	///
	/// A new tuple where the first values are combined using `f` and the second values are combined using `Semigroup::append`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     lift2::<Tuple2WithSecondBrand<String>, _, _, _, _>(|x, y| x + y, (1, "a".to_string()), (2, "b".to_string())),
	///     (3, "ab".to_string())
	/// );
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
		C: 'a,
	{
		(func(fa.0, fb.0), Semigroup::append(fa.1, fb.1))
	}
}

impl<Second: Clone + 'static> Pointed for Tuple2WithSecondBrand<Second>
where
	Second: Monoid,
{
	/// Wraps a value in a tuple (with empty second).
	///
	/// This method wraps a value in a tuple, using the `Monoid::empty()` value for the second element.
	///
	/// ### Type Signature
	///
	#[hm_signature(Pointed)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the value.", "The type of the value to wrap.")]
	///
	/// ### Parameters
	///
	#[doc_params("The value to wrap.")]
	///
	/// ### Returns
	///
	/// A tuple containing `a` and the empty value of the second type.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(pure::<Tuple2WithSecondBrand<String>, _>(5), (5, "".to_string()));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		(a, Monoid::empty())
	}
}

impl<Second: Clone + Semigroup + 'static> ApplyFirst for Tuple2WithSecondBrand<Second> {}
impl<Second: Clone + Semigroup + 'static> ApplySecond for Tuple2WithSecondBrand<Second> {}

impl<Second: Clone + 'static> Semiapplicative for Tuple2WithSecondBrand<Second>
where
	Second: Semigroup,
{
	/// Applies a wrapped function to a wrapped value (over first).
	///
	/// This method applies a function wrapped in a tuple to a value wrapped in a tuple. The second values are combined using their `Semigroup` implementation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semiapplicative)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function wrapper.",
		"The type of the input value.",
		"The type of the output value."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The tuple containing the function.", "The tuple containing the value.")]
	///
	/// ### Returns
	///
	/// A new tuple where the function is applied to the first value and the second values are combined.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = (cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2), "a".to_string());
	/// assert_eq!(apply::<RcFnBrand, Tuple2WithSecondBrand<String>, _, _>(f, (5, "b".to_string())), (10, "ab".to_string()));
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		(ff.0(fa.0), Semigroup::append(ff.1, fa.1))
	}
}

impl<Second: Clone + 'static> Semimonad for Tuple2WithSecondBrand<Second>
where
	Second: Semigroup,
{
	/// Chains tuple computations (over first).
	///
	/// This method chains two computations, where the second computation depends on the result of the first. The second values are combined using their `Semigroup` implementation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semimonad)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the result of the first computation.",
		"The type of the result of the second computation.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The first tuple.", "The function to apply to the first value.")]
	///
	/// ### Returns
	///
	/// A new tuple where the second values are combined.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     bind::<Tuple2WithSecondBrand<String>, _, _, _>((5, "a".to_string()), |x| (x * 2, "b".to_string())),
	///     (10, "ab".to_string())
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, Func>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		let (first, second) = ma;
		let (next_first, next_second) = func(first);
		(next_first, Semigroup::append(second, next_second))
	}
}

impl<Second: 'static> Foldable for Tuple2WithSecondBrand<Second> {
	/// Folds the tuple from the right (over first).
	///
	/// This method performs a right-associative fold of the tuple (over first).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The folding function.", "The initial value.", "The tuple to fold.")]
	///
	/// ### Returns
	///
	/// `func(a, initial)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(fold_right::<RcFnBrand, Tuple2WithSecondBrand<()>, _, _, _>(|x, acc| x + acc, 0, (5, ())), 5);
	/// ```
	fn fold_right<'a, FnBrand, A: 'a, B: 'a, F>(
		func: F,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(fa.0, initial)
	}

	/// Folds the tuple from the left (over first).
	///
	/// This method performs a left-associative fold of the tuple (over first).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The folding function.", "The initial value.", "The tuple to fold.")]
	///
	/// ### Returns
	///
	/// `func(initial, a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(fold_left::<RcFnBrand, Tuple2WithSecondBrand<()>, _, _, _>(|acc, x| acc + x, 0, (5, ())), 5);
	/// ```
	fn fold_left<'a, FnBrand, A: 'a, B: 'a, F>(
		func: F,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(initial, fa.0)
	}

	/// Maps the value to a monoid and returns it (over first).
	///
	/// This method maps the element of the tuple to a monoid and then returns it (over first).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the monoid.",
		"The type of the mapping function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The mapping function.", "The tuple to fold.")]
	///
	/// ### Returns
	///
	/// `func(a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, Tuple2WithSecondBrand<()>, _, _, _>(|x: i32| x.to_string(), (5, ())),
	///     "5".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, A: 'a, M, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(fa.0)
	}
}

impl<Second: Clone + 'static> Traversable for Tuple2WithSecondBrand<Second> {
	/// Traverses the tuple with an applicative function (over first).
	///
	/// This method maps the element of the tuple to a computation, evaluates it, and combines the result into an applicative context (over first).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply.", "The tuple to traverse.")]
	///
	/// ### Returns
	///
	/// The tuple wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     traverse::<Tuple2WithSecondBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), (5, ())),
	///     Some((10, ()))
	/// );
	/// ```
	fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		let (first, second) = ta;
		F::map(move |b| (b, second.clone()), func(first))
	}

	/// Sequences a tuple of applicative (over first).
	///
	/// This method evaluates the computation inside the tuple and accumulates the result into an applicative context (over first).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The tuple containing the applicative value.")]
	///
	/// ### Returns
	///
	/// The tuple wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// assert_eq!(
	///     sequence::<Tuple2WithSecondBrand<()>, _, OptionBrand>((Some(5), ())),
	///     Some((5, ()))
	/// );
	/// ```
	fn sequence<'a, A: 'a + Clone, F: Applicative>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		let (first, second) = ta;
		F::map(move |a| (a, second.clone()), first)
	}
}

impl<Second: 'static> ParFoldable for Tuple2WithSecondBrand<Second> {
	/// Maps the value to a monoid and returns it in parallel (over first).
	///
	/// This method maps the element of the tuple to a monoid and then returns it (over first). The mapping operation may be executed in parallel.
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function wrapper.",
		"The element type.",
		"The monoid type."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The thread-safe function to map each element to a monoid.", "The tuple to fold.")]
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = (1, "a".to_string());
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// assert_eq!(
	///     par_fold_map::<ArcFnBrand, Tuple2WithSecondBrand<String>, _, _>(f, x),
	///     "1".to_string()
	/// );
	/// ```
	fn par_fold_map<'a, FnBrand, A, M>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		func(fa.0)
	}

	/// Folds the tuple from the right in parallel (over first).
	///
	/// This method folds the tuple by applying a function from right to left, potentially in parallel (over first).
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function wrapper.",
		"The element type.",
		"The accumulator type."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The thread-safe function to apply to each element and the accumulator.",
		"The initial value.",
		"The tuple to fold."
	)]
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = (1, "a".to_string());
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
	/// assert_eq!(par_fold_right::<ArcFnBrand, Tuple2WithSecondBrand<String>, _, _>(f, 10, x), 11);
	/// ```
	fn par_fold_right<'a, FnBrand, A, B>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
	{
		func((fa.0, initial))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{brands::*, classes::bifunctor::*, functions::*};
	use quickcheck_macros::quickcheck;

	// Bifunctor Tests

	/// Tests `bimap` on `Tuple2`.
	#[test]
	fn test_bimap() {
		let x = (1, 5);
		assert_eq!(bimap::<Tuple2Brand, _, _, _, _, _, _>(|a| a + 1, |b| b * 2, x), (2, 10));
	}

	// Bifunctor Laws

	/// Tests the identity law for Bifunctor.
	#[quickcheck]
	fn bifunctor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = (first, second);
		bimap::<Tuple2Brand, _, _, _, _, _, _>(identity, identity, x.clone()) == x
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

		bimap::<Tuple2Brand, _, _, _, _, _, _>(compose(f, g), compose(h, i), x.clone())
			== bimap::<Tuple2Brand, _, _, _, _, _, _>(
				f,
				h,
				bimap::<Tuple2Brand, _, _, _, _, _, _>(g, i, x),
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
		map::<Tuple2WithFirstBrand<String>, _, _, _>(identity, x.clone()) == x
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
		map::<Tuple2WithFirstBrand<String>, _, _, _>(compose(f, g), x.clone())
			== map::<Tuple2WithFirstBrand<String>, _, _, _>(
				f,
				map::<Tuple2WithFirstBrand<String>, _, _, _>(g, x),
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
		apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(
			pure::<Tuple2WithFirstBrand<String>, _>(<RcFnBrand as CloneableFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(
			pure::<Tuple2WithFirstBrand<String>, _>(<RcFnBrand as CloneableFn>::new(f)),
			pure::<Tuple2WithFirstBrand<String>, _>(x),
		) == pure::<Tuple2WithFirstBrand<String>, _>(f(x))
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

		let u_fn = <RcFnBrand as CloneableFn>::new(move |x: i32| x.wrapping_add(u_seed));
		let u = pure::<Tuple2WithFirstBrand<String>, _>(u_fn);

		let v_fn = <RcFnBrand as CloneableFn>::new(move |x: i32| x.wrapping_mul(v_seed));
		let v = pure::<Tuple2WithFirstBrand<String>, _>(v_fn);

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		let compose_fn = <RcFnBrand as CloneableFn>::new(|f: std::rc::Rc<dyn Fn(i32) -> i32>| {
			let f = f.clone();
			<RcFnBrand as CloneableFn>::new(move |g: std::rc::Rc<dyn Fn(i32) -> i32>| {
				let f = f.clone();
				let g = g.clone();
				<RcFnBrand as CloneableFn>::new(move |x| f(g(x)))
			})
		});

		let pure_compose = pure::<Tuple2WithFirstBrand<String>, _>(compose_fn);
		let u_applied = apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(pure_compose, u);
		let uv = apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(u_applied, v);
		let lhs = apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(uv, w);

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
		let u = pure::<Tuple2WithFirstBrand<String>, _>(<RcFnBrand as CloneableFn>::new(f));

		let lhs = apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(
			u.clone(),
			pure::<Tuple2WithFirstBrand<String>, _>(y),
		);

		let rhs_fn =
			<RcFnBrand as CloneableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, Tuple2WithFirstBrand<String>, _, _>(
			pure::<Tuple2WithFirstBrand<String>, _>(rhs_fn),
			u,
		);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| ("f".to_string(), x.wrapping_mul(2));
		bind::<Tuple2WithFirstBrand<String>, _, _, _>(pure::<Tuple2WithFirstBrand<String>, _>(a), f)
			== f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(
		first: String,
		second: i32,
	) -> bool {
		let m = (first, second);
		bind::<Tuple2WithFirstBrand<String>, _, _, _>(
			m.clone(),
			pure::<Tuple2WithFirstBrand<String>, _>,
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
		bind::<Tuple2WithFirstBrand<String>, _, _, _>(
			bind::<Tuple2WithFirstBrand<String>, _, _, _>(m.clone(), f),
			g,
		) == bind::<Tuple2WithFirstBrand<String>, _, _, _>(m, |x| {
			bind::<Tuple2WithFirstBrand<String>, _, _, _>(f(x), g)
		})
	}

	// ParFoldable Tests for Tuple2WithFirstBrand (Functor over Second)

	/// Tests `par_fold_map` on `Tuple2WithFirstBrand`.
	#[test]
	fn par_fold_map_tuple2_with_first() {
		let x = ("a".to_string(), 1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, Tuple2WithFirstBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `Tuple2WithFirstBrand`.
	#[test]
	fn par_fold_right_tuple2_with_first() {
		let x = ("a".to_string(), 1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, Tuple2WithFirstBrand<String>, _, _>(f, 10, x), 11);
	}

	// ParFoldable Tests for Tuple2WithSecondBrand (Functor over First)

	/// Tests `par_fold_map` on `Tuple2WithSecondBrand`.
	#[test]
	fn par_fold_map_tuple2_with_second() {
		let x = (1, "a".to_string());
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, Tuple2WithSecondBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `Tuple2WithSecondBrand`.
	#[test]
	fn par_fold_right_tuple2_with_second() {
		let x = (1, "a".to_string());
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, Tuple2WithSecondBrand<String>, _, _>(f, 10, x), 11);
	}
}
