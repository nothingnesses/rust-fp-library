use fp_macros::doc_type_params;
use crate::{
	Apply,
	brands::{PairBrand, PairWithFirstBrand, PairWithSecondBrand},
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		bifunctor::Bifunctor, cloneable_fn::CloneableFn, foldable::Foldable, functor::Functor,
		lift::Lift, monoid::Monoid, par_foldable::ParFoldable, pointed::Pointed,
		semiapplicative::Semiapplicative, semigroup::Semigroup, semimonad::Semimonad,
		send_cloneable_fn::SendCloneableFn, traversable::Traversable,
	},
	impl_kind,
	kinds::*,
};
use fp_macros::hm_signature;

/// Wraps two values.
///
/// A simple tuple struct that holds two values of potentially different types.
///
/// ### Type Parameters
///
/// * `First`: The type of the first value.
/// * `Second`: The type of the second value.
///
/// ### Fields
///
/// * `0`: The first value.
/// * `1`: The second value.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let p = Pair(1, "hello");
/// assert_eq!(p.0, 1);
/// assert_eq!(p.1, "hello");
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<First, Second>(pub First, pub Second);

impl_kind! {
	for PairBrand {
		type Of<A, B> = Pair<A, B>;
	}
}

impl_kind! {
	for PairBrand {
		type Of<'a, A: 'a, B: 'a>: 'a = Pair<A, B>;
	}
}

impl Bifunctor for PairBrand {
	/// Maps functions over the values in the pair.
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
		"Undocumented",
		"The type of the first value.",
		"The type of the mapped first value.",
		"The type of the second value.",
		"The type of the mapped second value.",
		"The type of the function to apply to the first value.",
		"The type of the function to apply to the second value."
	)]	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the first value.
	/// * `g`: The function to apply to the second value.
	/// * `p`: The pair to map over.
	///
	/// ### Returns
	///
	/// A new pair containing the mapped values.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::bifunctor::*, functions::*, types::*};
	///
	/// let x = Pair(1, 5);
	/// assert_eq!(bimap::<PairBrand, _, _, _, _, _, _>(|a| a + 1, |b| b * 2, x), Pair(2, 10));
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
		let Pair(a, c) = p;
		Pair(f(a), g(c))
	}
}

// PairWithFirstBrand<First> (Functor over Second)

impl_kind! {
	impl<First: 'static> for PairWithFirstBrand<First> {
		type Of<'a, A: 'a>: 'a = Pair<First, A>;
	}
}

impl<First: 'static> Functor for PairWithFirstBrand<First> {
	/// Maps a function over the second value in the pair.
	///
	/// This method applies a function to the second value inside the pair, producing a new pair with the transformed second value. The first value remains unchanged.
	///
	/// ### Type Signature
	///
	#[hm_signature(Functor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the second value.",
		"The type of the result of applying the function.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the second value.
	/// * `fa`: The pair to map over.
	///
	/// ### Returns
	///
	/// A new pair containing the result of applying the function to the second value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(map::<PairWithFirstBrand<_>, _, _, _>(|x: i32| x * 2, Pair(1, 5)), Pair(1, 10));
	/// ```
	fn map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> B + 'a,
	{
		Pair(fa.0, func(fa.1))
	}
}

impl<First: Clone + 'static> Lift for PairWithFirstBrand<First>
where
	First: Semigroup,
{
	/// Lifts a binary function into the pair context (over second).
	///
	/// This method lifts a binary function to operate on the second values within the pair context. The first values are combined using their `Semigroup` implementation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Lift)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the first second value.",
		"The type of the second second value.",
		"The type of the result second value.",
		"The type of the binary function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The binary function to apply to the second values.
	/// * `fa`: The first pair.
	/// * `fb`: The second pair.
	///
	/// ### Returns
	///
	/// A new pair where the first values are combined using `Semigroup::append` and the second values are combined using `f`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     lift2::<PairWithFirstBrand<String>, _, _, _, _>(|x, y| x + y, Pair("a".to_string(), 1), Pair("b".to_string(), 2)),
	///     Pair("ab".to_string(), 3)
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
		Pair(Semigroup::append(fa.0, fb.0), func(fa.1, fb.1))
	}
}

impl<First: Clone + 'static> Pointed for PairWithFirstBrand<First>
where
	First: Monoid,
{
	/// Wraps a value in a pair (with empty first).
	///
	/// This method wraps a value in a pair, using the `Monoid::empty()` value for the first element.
	///
	/// ### Type Signature
	///
	#[hm_signature(Pointed)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the value to wrap."
	)]	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A pair containing the empty value of the first type and `a`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(pure::<PairWithFirstBrand<String>, _>(5), Pair("".to_string(), 5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Pair(Monoid::empty(), a)
	}
}

impl<First: Clone + Semigroup + 'static> ApplyFirst for PairWithFirstBrand<First> {}
impl<First: Clone + Semigroup + 'static> ApplySecond for PairWithFirstBrand<First> {}

impl<First: Clone + 'static> Semiapplicative for PairWithFirstBrand<First>
where
	First: Semigroup,
{
	/// Applies a wrapped function to a wrapped value (over second).
	///
	/// This method applies a function wrapped in a pair to a value wrapped in a pair. The first values are combined using their `Semigroup` implementation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semiapplicative)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function wrapper.",
		"The type of the input value.",
		"The type of the output value."
	)]	///
	/// ### Parameters
	///
	/// * `ff`: The pair containing the function.
	/// * `fa`: The pair containing the value.
	///
	/// ### Returns
	///
	/// A new pair where the first values are combined and the function is applied to the second value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let f = Pair("a".to_string(), cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// assert_eq!(apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(f, Pair("b".to_string(), 5)), Pair("ab".to_string(), 10));
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Pair(Semigroup::append(ff.0, fa.0), ff.1(fa.1))
	}
}

impl<First: Clone + 'static> Semimonad for PairWithFirstBrand<First>
where
	First: Semigroup,
{
	/// Chains pair computations (over second).
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
		"Undocumented",
		"The type of the result of the first computation.",
		"The type of the result of the second computation.",
		("A", "The type of the result of the first computation.")
	)]	///
	/// ### Parameters
	///
	/// * `ma`: The first pair.
	/// * `f`: The function to apply to the second value.
	///
	/// ### Returns
	///
	/// A new pair where the first values are combined.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     bind::<PairWithFirstBrand<String>, _, _, _>(Pair("a".to_string(), 5), |x| Pair("b".to_string(), x * 2)),
	///     Pair("ab".to_string(), 10)
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, Func>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		let Pair(first, second) = ma;
		let Pair(next_first, next_second) = func(second);
		Pair(Semigroup::append(first, next_first), next_second)
	}
}

impl<First: 'static> Foldable for PairWithFirstBrand<First> {
	/// Folds the pair from the right (over second).
	///
	/// This method performs a right-associative fold of the pair (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The pair to fold.
	///
	/// ### Returns
	///
	/// `func(a, initial)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(fold_right::<RcFnBrand, PairWithFirstBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair((), 5)), 5);
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

	/// Folds the pair from the left (over second).
	///
	/// This method performs a left-associative fold of the pair (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the accumulator and each element.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The identity to fold.
	///
	/// ### Returns
	///
	/// `func(initial, a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(fold_left::<RcFnBrand, PairWithFirstBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair((), 5)), 5);
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
	/// This method maps the element of the pair to a monoid and then returns it (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the monoid.",
		"The type of the mapping function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The pair to fold.
	///
	/// ### Returns
	///
	/// `func(a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, PairWithFirstBrand<()>, _, _, _>(|x: i32| x.to_string(), Pair((), 5)),
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

impl<First: Clone + 'static> Traversable for PairWithFirstBrand<First> {
	/// Traverses the pair with an applicative function (over second).
	///
	/// This method maps the element of the pair to a computation, evaluates it, and combines the result into an applicative context (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a value in an applicative context.
	/// * `ta`: The pair to traverse.
	///
	/// ### Returns
	///
	/// The pair wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     traverse::<PairWithFirstBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Pair((), 5)),
	///     Some(Pair((), 10))
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
		let Pair(first, second) = ta;
		F::map(move |b| Pair(first.clone(), b), func(second))
	}
	/// Sequences a pair of applicative (over second).
	///
	/// This method evaluates the computation inside the pair and accumulates the result into an applicative context (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]	///
	/// ### Parameters
	///
	/// * `ta`: The pair containing the applicative value.
	///
	/// ### Returns
	///
	/// The pair wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     sequence::<PairWithFirstBrand<()>, _, OptionBrand>(Pair((), Some(5))),
	///     Some(Pair((), 5))
	/// );
	/// ```
	fn sequence<'a, A: 'a + Clone, F: Applicative>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		let Pair(first, second) = ta;
		F::map(move |a| Pair(first.clone(), a), second)
	}
}

impl<First: 'static> ParFoldable for PairWithFirstBrand<First> {
	/// Maps the value to a monoid and returns it in parallel (over second).
	///
	/// This method maps the element of the pair to a monoid and then returns it (over second). The mapping operation may be executed in parallel.
	///
	/// ### Type Signature
	///
	#[hm_signature(SendCloneableFn)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		("A", "The element type (must be `Send + Sync`)."),
		"The element type (must be `Send + Sync`).",
		"The monoid type (must be `Send + Sync`)."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to map each element to a monoid.
	/// * `fa`: The pair to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let x = Pair("a".to_string(), 1);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// assert_eq!(
	///     par_fold_map::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, x),
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

	/// Folds the pair from the right in parallel (over second).
	///
	/// This method folds the pair by applying a function from right to left, potentially in parallel (over second).
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		("B", "The accumulator type (must be `Send + Sync`)."),
		"The element type (must be `Send + Sync`).",
		"The accumulator type (must be `Send + Sync`)."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to apply to each element and the accumulator.
	/// * `initial`: The initial value.
	/// * `fa`: The pair to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let x = Pair("a".to_string(), 1);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
	/// assert_eq!(par_fold_right::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, 10, x), 11);
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
// PairWithSecondBrand<Second> (Functor over First)

impl_kind! {
	impl<Second: 'static> for PairWithSecondBrand<Second> {
		type Of<'a, A: 'a>: 'a = Pair<A, Second>;
	}
}

impl<Second: 'static> Functor for PairWithSecondBrand<Second> {
	/// Maps a function over the first value in the pair.
	///
	/// This method applies a function to the first value inside the pair, producing a new pair with the transformed first value. The second value remains unchanged.
	///
	/// ### Type Signature
	///
	#[hm_signature(Functor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the first value.",
		"The type of the result of applying the function.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the first value.
	/// * `fa`: The pair to map over.
	///
	/// ### Returns
	///
	/// A new pair containing the result of applying the function to the first value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(map::<PairWithSecondBrand<_>, _, _, _>(|x: i32| x * 2, Pair(5, 1)), Pair(10, 1));
	/// ```
	fn map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> B + 'a,
	{
		Pair(func(fa.0), fa.1)
	}
}

impl<Second: Clone + 'static> Lift for PairWithSecondBrand<Second>
where
	Second: Semigroup,
{
	/// Lifts a binary function into the pair context (over first).
	///
	/// This method lifts a binary function to operate on the first values within the pair context. The second values are combined using their `Semigroup` implementation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Lift)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the first first value.",
		"The type of the second first value.",
		"The type of the result first value.",
		"The type of the binary function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The binary function to apply to the first values.
	/// * `fa`: The first pair.
	/// * `fb`: The second pair.
	///
	/// ### Returns
	///
	/// A new pair where the first values are combined using `f` and the second values are combined using `Semigroup::append`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     lift2::<PairWithSecondBrand<String>, _, _, _, _>(|x, y| x + y, Pair(1, "a".to_string()), Pair(2, "b".to_string())),
	///     Pair(3, "ab".to_string())
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
		Pair(func(fa.0, fb.0), Semigroup::append(fa.1, fb.1))
	}
}

impl<Second: Clone + 'static> Pointed for PairWithSecondBrand<Second>
where
	Second: Monoid,
{
	/// Wraps a value in a pair (with empty second).
	///
	/// This method wraps a value in a pair, using the `Monoid::empty()` value for the second element.
	///
	/// ### Type Signature
	///
	#[hm_signature(Pointed)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the value to wrap."
	)]	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A pair containing `a` and the empty value of the second type.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(pure::<PairWithSecondBrand<String>, _>(5), Pair(5, "".to_string()));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Pair(a, Monoid::empty())
	}
}

impl<Second: Clone + Semigroup + 'static> ApplyFirst for PairWithSecondBrand<Second> {}
impl<Second: Clone + Semigroup + 'static> ApplySecond for PairWithSecondBrand<Second> {}

impl<Second: Clone + 'static> Semiapplicative for PairWithSecondBrand<Second>
where
	Second: Semigroup,
{
	/// Applies a wrapped function to a wrapped value (over first).
	///
	/// This method applies a function wrapped in a result (as error) to a value wrapped in a result (as error).
	///
	/// ### Type Signature
	///
	#[hm_signature(Semiapplicative)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function wrapper.",
		"The type of the input value.",
		"The type of the output value."
	)]	///
	/// ### Parameters
	///
	/// * `ff`: The pair containing the function (in Err).
	/// * `fa`: The pair containing the value (in Err).
	///
	/// ### Returns
	///
	/// `Err(f(a))` if both are `Err`, otherwise the first success encountered.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let f = Pair(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2), "a".to_string());
	/// assert_eq!(apply::<RcFnBrand, PairWithSecondBrand<String>, _, _>(f, Pair(5, "b".to_string())), Pair(10, "ab".to_string()));
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Pair(ff.0(fa.0), Semigroup::append(ff.1, fa.1))
	}
}

impl<Second: Clone + 'static> Semimonad for PairWithSecondBrand<Second>
where
	Second: Semigroup,
{
	/// Chains pair computations (over first).
	///
	/// This method chains two computations, where the second computation depends on the result of the first (over error).
	///
	/// ### Type Signature
	///
	#[hm_signature(Semimonad)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the result of the first computation.",
		"The type of the result of the second computation.",
		("A", "The type of the result of the first computation.")
	)]	///
	/// ### Parameters
	///
	/// * `ma`: The first result.
	/// * `f`: The function to apply to the error value.
	///
	/// ### Returns
	///
	/// The result of applying `f` to the error if `ma` is `Err`, otherwise the original success.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     bind::<PairWithSecondBrand<String>, _, _, _>(Pair(5, "a".to_string()), |x| Pair(x * 2, "b".to_string())),
	///     Pair(10, "ab".to_string())
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, Func>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		let Pair(first, second) = ma;
		let Pair(next_first, next_second) = func(first);
		Pair(next_first, Semigroup::append(second, next_second))
	}
}

impl<Second: 'static> Foldable for PairWithSecondBrand<Second> {
	/// Folds the pair from the right (over first).
	///
	/// This method performs a right-associative fold of the result (over error).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// ### Returns
	///
	/// `func(a, initial)` if `fa` is `Err(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(fold_right::<RcFnBrand, PairWithSecondBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair(5, ())), 5);
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

	/// Folds the pair from the left (over first).
	///
	/// This method performs a left-associative fold of the result (over error).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// ### Returns
	///
	/// `func(initial, a)` if `fa` is `Err(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(fold_left::<RcFnBrand, PairWithSecondBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair(5, ())), 5);
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
	/// This method maps the element of the result to a monoid and then returns it (over error).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the monoid.",
		"The type of the mapping function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The result to fold.
	///
	/// ### Returns
	///
	/// `func(a)` if `fa` is `Err(a)`, otherwise `M::empty()`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, PairWithSecondBrand<()>, _, _, _>(|x: i32| x.to_string(), Pair(5, ())),
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

impl<Second: Clone + 'static> Traversable for PairWithSecondBrand<Second> {
	/// Traverses the pair with an applicative function (over first).
	///
	/// This method maps the element of the result to a computation, evaluates it, and combines the result into an applicative context (over error).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `ta`: The result to traverse.
	///
	/// ### Returns
	///
	/// The result wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     traverse::<PairWithSecondBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Pair(5, ())),
	///     Some(Pair(10, ()))
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
		let Pair(first, second) = ta;
		F::map(move |b| Pair(b, second.clone()), func(first))
	}

	/// Sequences a pair of applicative (over first).
	///
	/// This method evaluates the computation inside the result and accumulates the result into an applicative context (over error).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]	///
	/// ### Parameters
	///
	/// * `ta`: The result containing the applicative value.
	///
	/// ### Returns
	///
	/// The result wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     sequence::<PairWithSecondBrand<()>, _, OptionBrand>(Pair(Some(5), ())),
	///     Some(Pair(5, ()))
	/// );
	/// ```
	fn sequence<'a, A: 'a + Clone, F: Applicative>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		let Pair(first, second) = ta;
		F::map(move |a| Pair(a, second.clone()), first)
	}
}

impl<Second: 'static> ParFoldable for PairWithSecondBrand<Second> {
	/// Maps the value to a monoid and returns it in parallel (over first).
	///
	/// This method maps the element of the pair to a monoid and then returns it (over first). The mapping operation may be executed in parallel.
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		("M", "The monoid type (must be `Send + Sync`)."),
		"The element type (must be `Send + Sync`).",
		"The monoid type (must be `Send + Sync`)."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to map each element to a monoid.
	/// * `fa`: The pair to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let x = Pair(1, "a".to_string());
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// assert_eq!(
	///     par_fold_map::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, x),
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

	/// Folds the pair from the right in parallel (over first).
	///
	/// This method folds the pair by applying a function from right to left, potentially in parallel (over first).
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		("B", "The accumulator type (must be `Send + Sync`)."),
		"The element type (must be `Send + Sync`).",
		"The accumulator type (must be `Send + Sync`)."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to apply to each element and the accumulator.
	/// * `initial`: The initial value.
	/// * `fa`: The pair to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let x = Pair(1, "a".to_string());
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
	/// assert_eq!(par_fold_right::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, 10, x), 11);
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

	/// Tests `bimap` on `Pair`.
	#[test]
	fn test_bimap() {
		let x = Pair(1, 5);
		assert_eq!(bimap::<PairBrand, _, _, _, _, _, _>(|a| a + 1, |b| b * 2, x), Pair(2, 10));
	}

	// Bifunctor Laws

	/// Tests the identity law for Bifunctor.
	#[quickcheck]
	fn bifunctor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		bimap::<PairBrand, _, _, _, _, _, _>(identity, identity, x.clone()) == x
	}

	/// Tests the composition law for Bifunctor.
	#[quickcheck]
	fn bifunctor_composition(
		first: i32,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		let h = |x: i32| x.wrapping_sub(1);
		let i = |x: i32| if x == 0 { 0 } else { x.wrapping_div(2) };

		bimap::<PairBrand, _, _, _, _, _, _>(compose(f, g), compose(h, i), x.clone())
			== bimap::<PairBrand, _, _, _, _, _, _>(
				f,
				h,
				bimap::<PairBrand, _, _, _, _, _, _>(g, i, x),
			)
	}

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		map::<PairWithFirstBrand<String>, _, _, _>(identity, x.clone()) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(
		first: String,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<PairWithFirstBrand<String>, _, _, _>(compose(f, g), x.clone())
			== map::<PairWithFirstBrand<String>, _, _, _>(
				f,
				map::<PairWithFirstBrand<String>, _, _, _>(g, x),
			)
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(
		first: String,
		second: i32,
	) -> bool {
		let v = Pair(first, second);
		apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			pure::<PairWithFirstBrand<String>, _>(<RcFnBrand as CloneableFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			pure::<PairWithFirstBrand<String>, _>(<RcFnBrand as CloneableFn>::new(f)),
			pure::<PairWithFirstBrand<String>, _>(x),
		) == pure::<PairWithFirstBrand<String>, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w_first: String,
		w_second: i32,
		u_seed: i32,
		v_seed: i32,
	) -> bool {
		let w = Pair(w_first, w_second);

		let u_fn = <RcFnBrand as CloneableFn>::new(move |x: i32| x.wrapping_add(u_seed));
		let u = pure::<PairWithFirstBrand<String>, _>(u_fn);

		let v_fn = <RcFnBrand as CloneableFn>::new(move |x: i32| x.wrapping_mul(v_seed));
		let v = pure::<PairWithFirstBrand<String>, _>(v_fn);

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		let compose_fn = <RcFnBrand as CloneableFn>::new(|f: std::rc::Rc<dyn Fn(i32) -> i32>| {
			let f = f.clone();
			<RcFnBrand as CloneableFn>::new(move |g: std::rc::Rc<dyn Fn(i32) -> i32>| {
				let f = f.clone();
				let g = g.clone();
				<RcFnBrand as CloneableFn>::new(move |x| f(g(x)))
			})
		});

		let pure_compose = pure::<PairWithFirstBrand<String>, _>(compose_fn);
		let u_applied = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(pure_compose, u);
		let uv = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(u_applied, v);
		let lhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(uv, w);

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
		let u = pure::<PairWithFirstBrand<String>, _>(<RcFnBrand as CloneableFn>::new(f));

		let lhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			u.clone(),
			pure::<PairWithFirstBrand<String>, _>(y),
		);

		let rhs_fn =
			<RcFnBrand as CloneableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			pure::<PairWithFirstBrand<String>, _>(rhs_fn),
			u,
		);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| Pair("f".to_string(), x.wrapping_mul(2));
		bind::<PairWithFirstBrand<String>, _, _, _>(pure::<PairWithFirstBrand<String>, _>(a), f)
			== f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(
		first: String,
		second: i32,
	) -> bool {
		let m = Pair(first, second);
		bind::<PairWithFirstBrand<String>, _, _, _>(
			m.clone(),
			pure::<PairWithFirstBrand<String>, _>,
		) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(
		first: String,
		second: i32,
	) -> bool {
		let m = Pair(first, second);
		let f = |x: i32| Pair("f".to_string(), x.wrapping_mul(2));
		let g = |x: i32| Pair("g".to_string(), x.wrapping_add(1));
		bind::<PairWithFirstBrand<String>, _, _, _>(
			bind::<PairWithFirstBrand<String>, _, _, _>(m.clone(), f),
			g,
		) == bind::<PairWithFirstBrand<String>, _, _, _>(m, |x| {
			bind::<PairWithFirstBrand<String>, _, _, _>(f(x), g)
		})
	}

	// ParFoldable Tests for PairWithFirstBrand (Functor over Second)

	/// Tests `par_fold_map` on `PairWithFirstBrand`.
	#[test]
	fn par_fold_map_pair_with_first() {
		let x = Pair("a".to_string(), 1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `PairWithFirstBrand`.
	#[test]
	fn par_fold_right_pair_with_first() {
		let x = Pair("a".to_string(), 1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, 10, x), 11);
	}

	// ParFoldable Tests for PairWithSecondBrand (Functor over First)

	/// Tests `par_fold_map` on `PairWithSecondBrand`.
	#[test]
	fn par_fold_map_pair_with_second() {
		let x = Pair(1, "a".to_string());
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `PairWithSecondBrand`.
	#[test]
	fn par_fold_right_pair_with_second() {
		let x = Pair(1, "a".to_string());
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, 10, x), 11);
	}
}
