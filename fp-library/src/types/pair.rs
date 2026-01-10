use crate::{
	brands::{PairWithFirstBrand, PairWithSecondBrand},
	classes::{
		applicative::Applicative,
		apply_first::ApplyFirst,
		apply_second::ApplySecond,
		clonable_fn::{ApplyClonableFn, ClonableFn},
		foldable::Foldable,
		functor::Functor,
		lift::Lift,
		monoid::Monoid,
		pointed::Pointed,
		semiapplicative::Semiapplicative,
		semigroup::Semigroup,
		semimonad::Semimonad,
		traversable::Traversable,
	},
	hkt::{Apply1L1T, Kind1L1T},
};

use crate::hkt::Kind0L2T;

/// Wraps two values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<First, Second>(pub First, pub Second);

pub struct PairBrand;

impl Kind0L2T for PairBrand {
	type Output<A, B> = Pair<A, B>;
}

// PairWithFirstBrand<First> (Functor over Second)

impl<First: 'static> Kind1L1T for PairWithFirstBrand<First> {
	type Output<'a, A: 'a> = Pair<First, A>;
}

impl<First: 'static> Functor for PairWithFirstBrand<First> {
	/// Maps a function over the second value in the pair.
	///
	/// # Type Signature
	///
	/// `forall a b t. Functor (Pair t) => (a -> b, Pair t a) -> Pair t b`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply to the second value.
	/// * `fa`: The pair to map over.
	///
	/// # Returns
	///
	/// A new pair containing the result of applying the function to the second value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::PairWithFirstBrand;
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(map::<PairWithFirstBrand<_>, _, _, _>(|x: i32| x * 2, Pair(1, 5)), Pair(1, 10));
	/// ```
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply1L1T<'a, Self, A>,
	) -> Apply1L1T<'a, Self, B>
	where
		F: Fn(A) -> B + 'a,
	{
		Pair(fa.0, f(fa.1))
	}
}

impl<First: Clone + 'static> Lift for PairWithFirstBrand<First>
where
	First: Semigroup,
{
	/// Lifts a binary function into the pair context (over second).
	///
	/// # Type Signature
	///
	/// `forall a b c t. (Lift (Pair t), Semigroup t) => ((a, b) -> c, Pair t a, Pair t b) -> Pair t c`
	///
	/// # Parameters
	///
	/// * `f`: The binary function to apply to the second values.
	/// * `fa`: The first pair.
	/// * `fb`: The second pair.
	///
	/// # Returns
	///
	/// A new pair where the first values are combined using `Semigroup::append` and the second values are combined using `f`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::lift::lift2;
	/// use fp_library::brands::PairWithFirstBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::string;
	///
	/// assert_eq!(
	///     lift2::<PairWithFirstBrand<String>, _, _, _, _>(|x, y| x + y, Pair("a".to_string(), 1), Pair("b".to_string(), 2)),
	///     Pair("ab".to_string(), 3)
	/// );
	/// ```
	fn lift2<'a, A, B, C, F>(
		f: F,
		fa: Apply1L1T<'a, Self, A>,
		fb: Apply1L1T<'a, Self, B>,
	) -> Apply1L1T<'a, Self, C>
	where
		F: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
	{
		Pair(Semigroup::append(fa.0, fb.0), f(fa.1, fb.1))
	}
}

impl<First: Clone + 'static> Pointed for PairWithFirstBrand<First>
where
	First: Monoid,
{
	/// Wraps a value in a pair (with empty first).
	///
	/// # Type Signature
	///
	/// `forall a t. (Pointed (Pair t), Monoid t) => a -> Pair t a`
	///
	/// # Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// # Returns
	///
	/// A pair containing the empty value of the first type and `a`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::PairWithFirstBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::string;
	///
	/// assert_eq!(pure::<PairWithFirstBrand<String>, _>(5), Pair("".to_string(), 5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A> {
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
	/// # Type Signature
	///
	/// `forall a b t. (Semiapplicative (Pair t), Semigroup t) => (Pair t (a -> b), Pair t a) -> Pair t b`
	///
	/// # Parameters
	///
	/// * `ff`: The pair containing the function.
	/// * `fa`: The pair containing the value.
	///
	/// # Returns
	///
	/// A new pair where the first values are combined and the function is applied to the second value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::PairWithFirstBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::rc_fn::RcFnBrand;
	/// use fp_library::types::string;
	/// use std::rc::Rc;
	///
	/// let f = Pair("a".to_string(), <RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<PairWithFirstBrand<String>, _, _, RcFnBrand>(f, Pair("b".to_string(), 5)), Pair("ab".to_string(), 10));
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
		ff: Apply1L1T<'a, Self, ApplyClonableFn<'a, FnBrand, A, B>>,
		fa: Apply1L1T<'a, Self, A>,
	) -> Apply1L1T<'a, Self, B> {
		Pair(Semigroup::append(ff.0, fa.0), ff.1(fa.1))
	}
}

impl<First: Clone + 'static> Semimonad for PairWithFirstBrand<First>
where
	First: Semigroup,
{
	/// Chains pair computations (over second).
	///
	/// # Type Signature
	///
	/// `forall a b t. (Semimonad (Pair t), Semigroup t) => (Pair t a, a -> Pair t b) -> Pair t b`
	///
	/// # Parameters
	///
	/// * `ma`: The first pair.
	/// * `f`: The function to apply to the second value.
	///
	/// # Returns
	///
	/// A new pair where the first values are combined.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::PairWithFirstBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::string;
	///
	/// assert_eq!(
	///     bind::<PairWithFirstBrand<String>, _, _, _>(Pair("a".to_string(), 5), |x| Pair("b".to_string(), x * 2)),
	///     Pair("ab".to_string(), 10)
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, F>(
		ma: Apply1L1T<'a, Self, A>,
		f: F,
	) -> Apply1L1T<'a, Self, B>
	where
		F: Fn(A) -> Apply1L1T<'a, Self, B> + 'a,
	{
		let Pair(first, second) = ma;
		let Pair(next_first, next_second) = f(second);
		Pair(Semigroup::append(first, next_first), next_second)
	}
}

impl<First: 'static> Foldable for PairWithFirstBrand<First> {
	/// Folds the pair from the right (over second).
	///
	/// # Type Signature
	///
	/// `forall a b t. Foldable (Pair t) => ((a, b) -> b, b, Pair t a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// `f(a, init)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::PairWithFirstBrand;
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(fold_right::<PairWithFirstBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair((), 5)), 5);
	/// ```
	fn fold_right<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply1L1T<'a, Self, A>,
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
	{
		f(fa.1, init)
	}

	/// Folds the pair from the left (over second).
	///
	/// # Type Signature
	///
	/// `forall a b t. Foldable (Pair t) => ((b, a) -> b, b, Pair t a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// `f(init, a)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::PairWithFirstBrand;
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(fold_left::<PairWithFirstBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair((), 5)), 5);
	/// ```
	fn fold_left<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply1L1T<'a, Self, A>,
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
	{
		f(init, fa.1)
	}

	/// Maps the value to a monoid and returns it (over second).
	///
	/// # Type Signature
	///
	/// `forall a m t. (Foldable (Pair t), Monoid m) => ((a) -> m, Pair t a) -> m`
	///
	/// # Parameters
	///
	/// * `f`: The mapping function.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// `f(a)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::PairWithFirstBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::string;
	///
	/// assert_eq!(
	///     fold_map::<PairWithFirstBrand<()>, _, _, _>(|x: i32| x.to_string(), Pair((), 5)),
	///     "5".to_string()
	/// );
	/// ```
	fn fold_map<'a, A: 'a, M, F>(
		f: F,
		fa: Apply1L1T<'a, Self, A>,
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
	{
		f(fa.1)
	}
}

impl<First: Clone + 'static> Traversable for PairWithFirstBrand<First> {
	/// Traverses the pair with an applicative function (over second).
	///
	/// # Type Signature
	///
	/// `forall a b f t. (Traversable (Pair t), Applicative f) => (a -> f b, Pair t a) -> f (Pair t b)`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply.
	/// * `ta`: The pair to traverse.
	///
	/// # Returns
	///
	/// The pair wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::{PairWithFirstBrand, OptionBrand};
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(
	///     traverse::<PairWithFirstBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Pair((), 5)),
	///     Some(Pair((), 10))
	/// );
	/// ```
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
		f: Func,
		ta: Apply1L1T<'a, Self, A>,
	) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, B>>
	where
		Func: Fn(A) -> Apply1L1T<'a, F, B> + 'a,
		Apply1L1T<'a, Self, B>: Clone,
	{
		let Pair(first, second) = ta;
		F::map(move |b| Pair(first.clone(), b), f(second))
	}

	/// Sequences a pair of applicative (over second).
	///
	/// # Type Signature
	///
	/// `forall a f t. (Traversable (Pair t), Applicative f) => (Pair t (f a)) -> f (Pair t a)`
	///
	/// # Parameters
	///
	/// * `ta`: The pair containing the applicative value.
	///
	/// # Returns
	///
	/// The pair wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::{PairWithFirstBrand, OptionBrand};
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(
	///     sequence::<PairWithFirstBrand<()>, OptionBrand, _>(Pair((), Some(5))),
	///     Some(Pair((), 5))
	/// );
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply1L1T<'a, Self, Apply1L1T<'a, F, A>>
	) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, A>>
	where
		Apply1L1T<'a, F, A>: Clone,
		Apply1L1T<'a, Self, A>: Clone,
	{
		let Pair(first, second) = ta;
		F::map(move |a| Pair(first.clone(), a), second)
	}
}

// PairWithSecondBrand<Second> (Functor over First)

impl<Second: 'static> Kind1L1T for PairWithSecondBrand<Second> {
	type Output<'a, A: 'a> = Pair<A, Second>;
}

impl<Second: 'static> Functor for PairWithSecondBrand<Second> {
	/// Maps a function over the first value in the pair.
	///
	/// # Type Signature
	///
	/// `forall a b t. Functor (Pair' t) => (a -> b, Pair a t) -> Pair b t`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply to the first value.
	/// * `fa`: The pair to map over.
	///
	/// # Returns
	///
	/// A new pair containing the result of applying the function to the first value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::PairWithSecondBrand;
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(map::<PairWithSecondBrand<_>, _, _, _>(|x: i32| x * 2, Pair(5, 1)), Pair(10, 1));
	/// ```
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply1L1T<'a, Self, A>,
	) -> Apply1L1T<'a, Self, B>
	where
		F: Fn(A) -> B + 'a,
	{
		Pair(f(fa.0), fa.1)
	}
}

impl<Second: Clone + 'static> Lift for PairWithSecondBrand<Second>
where
	Second: Semigroup,
{
	/// Lifts a binary function into the pair context (over first).
	///
	/// # Type Signature
	///
	/// `forall a b c t. (Lift (Pair' t), Semigroup t) => ((a, b) -> c, Pair a t, Pair b t) -> Pair c t`
	///
	/// # Parameters
	///
	/// * `f`: The binary function to apply to the first values.
	/// * `fa`: The first pair.
	/// * `fb`: The second pair.
	///
	/// # Returns
	///
	/// A new pair where the first values are combined using `f` and the second values are combined using `Semigroup::append`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::lift::lift2;
	/// use fp_library::brands::PairWithSecondBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::string;
	///
	/// assert_eq!(
	///     lift2::<PairWithSecondBrand<String>, _, _, _, _>(|x, y| x + y, Pair(1, "a".to_string()), Pair(2, "b".to_string())),
	///     Pair(3, "ab".to_string())
	/// );
	/// ```
	fn lift2<'a, A, B, C, F>(
		f: F,
		fa: Apply1L1T<'a, Self, A>,
		fb: Apply1L1T<'a, Self, B>,
	) -> Apply1L1T<'a, Self, C>
	where
		F: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
	{
		Pair(f(fa.0, fb.0), Semigroup::append(fa.1, fb.1))
	}
}

impl<Second: Clone + 'static> Pointed for PairWithSecondBrand<Second>
where
	Second: Monoid,
{
	/// Wraps a value in a pair (with empty second).
	///
	/// # Type Signature
	///
	/// `forall a t. (Pointed (Pair' t), Monoid t) => a -> Pair a t`
	///
	/// # Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// # Returns
	///
	/// A pair containing `a` and the empty value of the second type.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::PairWithSecondBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::string;
	///
	/// assert_eq!(pure::<PairWithSecondBrand<String>, _>(5), Pair(5, "".to_string()));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A> {
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
	/// # Type Signature
	///
	/// `forall a b t. (Semiapplicative (Pair' t), Semigroup t) => (Pair (a -> b) t, Pair a t) -> Pair b t`
	///
	/// # Parameters
	///
	/// * `ff`: The pair containing the function.
	/// * `fa`: The pair containing the value.
	///
	/// # Returns
	///
	/// A new pair where the function is applied to the first value and the second values are combined.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::PairWithSecondBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::rc_fn::RcFnBrand;
	/// use fp_library::types::string;
	/// use std::rc::Rc;
	///
	/// let f = Pair(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2), "a".to_string());
	/// assert_eq!(apply::<PairWithSecondBrand<String>, _, _, RcFnBrand>(f, Pair(5, "b".to_string())), Pair(10, "ab".to_string()));
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
		ff: Apply1L1T<'a, Self, ApplyClonableFn<'a, FnBrand, A, B>>,
		fa: Apply1L1T<'a, Self, A>,
	) -> Apply1L1T<'a, Self, B> {
		Pair(ff.0(fa.0), Semigroup::append(ff.1, fa.1))
	}
}

impl<Second: Clone + 'static> Semimonad for PairWithSecondBrand<Second>
where
	Second: Semigroup,
{
	/// Chains pair computations (over first).
	///
	/// # Type Signature
	///
	/// `forall a b t. (Semimonad (Pair' t), Semigroup t) => (Pair a t, a -> Pair b t) -> Pair b t`
	///
	/// # Parameters
	///
	/// * `ma`: The first pair.
	/// * `f`: The function to apply to the first value.
	///
	/// # Returns
	///
	/// A new pair where the second values are combined.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::PairWithSecondBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::string;
	///
	/// assert_eq!(
	///     bind::<PairWithSecondBrand<String>, _, _, _>(Pair(5, "a".to_string()), |x| Pair(x * 2, "b".to_string())),
	///     Pair(10, "ab".to_string())
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, F>(
		ma: Apply1L1T<'a, Self, A>,
		f: F,
	) -> Apply1L1T<'a, Self, B>
	where
		F: Fn(A) -> Apply1L1T<'a, Self, B> + 'a,
	{
		let Pair(first, second) = ma;
		let Pair(next_first, next_second) = f(first);
		Pair(next_first, Semigroup::append(second, next_second))
	}
}

impl<Second: 'static> Foldable for PairWithSecondBrand<Second> {
	/// Folds the pair from the right (over first).
	///
	/// # Type Signature
	///
	/// `forall a b t. Foldable (Pair' t) => ((a, b) -> b, b, Pair a t) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// `f(a, init)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::PairWithSecondBrand;
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(fold_right::<PairWithSecondBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair(5, ())), 5);
	/// ```
	fn fold_right<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply1L1T<'a, Self, A>,
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
	{
		f(fa.0, init)
	}

	/// Folds the pair from the left (over first).
	///
	/// # Type Signature
	///
	/// `forall a b t. Foldable (Pair' t) => ((b, a) -> b, b, Pair a t) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// `f(init, a)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::PairWithSecondBrand;
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(fold_left::<PairWithSecondBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair(5, ())), 5);
	/// ```
	fn fold_left<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply1L1T<'a, Self, A>,
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
	{
		f(init, fa.0)
	}

	/// Maps the value to a monoid and returns it (over first).
	///
	/// # Type Signature
	///
	/// `forall a m t. (Foldable (Pair' t), Monoid m) => ((a) -> m, Pair a t) -> m`
	///
	/// # Parameters
	///
	/// * `f`: The mapping function.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// `f(a)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::PairWithSecondBrand;
	/// use fp_library::types::Pair;
	/// use fp_library::types::string;
	///
	/// assert_eq!(
	///     fold_map::<PairWithSecondBrand<()>, _, _, _>(|x: i32| x.to_string(), Pair(5, ())),
	///     "5".to_string()
	/// );
	/// ```
	fn fold_map<'a, A: 'a, M, F>(
		f: F,
		fa: Apply1L1T<'a, Self, A>,
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
	{
		f(fa.0)
	}
}

impl<Second: Clone + 'static> Traversable for PairWithSecondBrand<Second> {
	/// Traverses the pair with an applicative function (over first).
	///
	/// # Type Signature
	///
	/// `forall a b f t. (Traversable (Pair' t), Applicative f) => (a -> f b, Pair a t) -> f (Pair b t)`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply.
	/// * `ta`: The pair to traverse.
	///
	/// # Returns
	///
	/// The pair wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::{PairWithSecondBrand, OptionBrand};
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(
	///     traverse::<PairWithSecondBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Pair(5, ())),
	///     Some(Pair(10, ()))
	/// );
	/// ```
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
		f: Func,
		ta: Apply1L1T<'a, Self, A>,
	) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, B>>
	where
		Func: Fn(A) -> Apply1L1T<'a, F, B> + 'a,
		Apply1L1T<'a, Self, B>: Clone,
	{
		let Pair(first, second) = ta;
		F::map(move |b| Pair(b, second.clone()), f(first))
	}

	/// Sequences a pair of applicative (over first).
	///
	/// # Type Signature
	///
	/// `forall a f t. (Traversable (Pair' t), Applicative f) => (Pair (f a) t) -> f (Pair a t)`
	///
	/// # Parameters
	///
	/// * `ta`: The pair containing the applicative value.
	///
	/// # Returns
	///
	/// The pair wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::{PairWithSecondBrand, OptionBrand};
	/// use fp_library::types::Pair;
	///
	/// assert_eq!(
	///     sequence::<PairWithSecondBrand<()>, OptionBrand, _>(Pair(Some(5), ())),
	///     Some(Pair(5, ()))
	/// );
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply1L1T<'a, Self, Apply1L1T<'a, F, A>>
	) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, A>>
	where
		Apply1L1T<'a, F, A>: Clone,
		Apply1L1T<'a, Self, A>: Clone,
	{
		let Pair(first, second) = ta;
		F::map(move |a| Pair(a, second.clone()), first)
	}
}
