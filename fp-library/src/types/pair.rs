//! Implementations for [`Pair`], a type that wraps two values.

use crate::{
	Apply,
	brands::{PairBrand, PairWithFirstBrand, PairWithSecondBrand},
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		clonable_fn::ClonableFn, foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		par_foldable::ParFoldable, pointed::Pointed, semiapplicative::Semiapplicative,
		semigroup::Semigroup, semimonad::Semimonad, send_clonable_fn::SendClonableFn,
		traversable::Traversable,
	},
	impl_kind,
	kinds::*,
};

/// Wraps two values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<First, Second>(pub First, pub Second);

impl_kind! {
	for PairBrand {
		type Of<A, B> = Pair<A, B>;
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
	fn map<'a, F, A: 'a, B: 'a>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
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
	fn lift2<'a, F, A, B, C>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		fb: Apply!(brand: Self, signature: ('a, B: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, C: 'a) -> 'a)
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
	fn pure<'a, A: 'a>(a: A) -> Apply!(brand: Self, signature: ('a, A: 'a) -> 'a) {
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
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::types::string;
	/// use std::rc::Rc;
	///
	/// let f = Pair("a".to_string(), <RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(f, Pair("b".to_string(), 5)), Pair("ab".to_string(), 10));
	/// ```
	fn apply<'a, FnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(brand: Self, signature: ('a, Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, B)): 'a) -> 'a),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) {
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
	fn bind<'a, F, A: 'a, B: 'a>(
		ma: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		f: F,
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
	where
		F: Fn(A) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) + 'a,
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
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_right::<RcFnBrand, PairWithFirstBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair((), 5)), 5);
	/// ```
	fn fold_right<'a, FnBrand, F, A: 'a, B: 'a>(
		f: F,
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
		FnBrand: ClonableFn + 'a,
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
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_left::<RcFnBrand, PairWithFirstBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair((), 5)), 5);
	/// ```
	fn fold_left<'a, FnBrand, F, A: 'a, B: 'a>(
		f: F,
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
		FnBrand: ClonableFn + 'a,
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
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, PairWithFirstBrand<()>, _, _, _>(|x: i32| x.to_string(), Pair((), 5)),
	///     "5".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, F, A: 'a, M>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
		FnBrand: ClonableFn + 'a,
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
	fn traverse<'a, F: Applicative, Func, A: 'a + Clone, B: 'a + Clone>(
		f: Func,
		ta: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): 'a) -> 'a)
	where
		Func: Fn(A) -> Apply!(brand: F, signature: ('a, B: 'a) -> 'a) + 'a,
		Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): Clone,
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
		ta: Apply!(brand: Self, signature: ('a, Apply!(brand: F, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	where
		Apply!(brand: F, signature: ('a, A: 'a) -> 'a): Clone,
		Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): Clone,
	{
		let Pair(first, second) = ta;
		F::map(move |a| Pair(first.clone(), a), second)
	}
}

impl<First: 'static, FnBrand: SendClonableFn> ParFoldable<FnBrand> for PairWithFirstBrand<First> {
	/// Maps the value to a monoid and returns it in parallel (over second).
	///
	/// # Type Signature
	///
	/// `forall a m t. (ParFoldable (Pair t), Monoid m, Send m, Sync m) => (f a m, Pair t a) -> m`
	///
	/// # Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// The combined monoid value.
	fn par_fold_map<'a, A, M>(
		func: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		func(fa.1)
	}

	/// Folds the pair from the right in parallel (over second).
	///
	/// # Type Signature
	///
	/// `forall a b t. ParFoldable (Pair t) => (f (a, b) b, b, Pair t a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// The final accumulator value.
	fn par_fold_right<'a, A, B>(
		f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
	{
		f((fa.1, init))
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
	fn map<'a, F, A: 'a, B: 'a>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
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
	fn lift2<'a, F, A, B, C>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		fb: Apply!(brand: Self, signature: ('a, B: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, C: 'a) -> 'a)
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
	fn pure<'a, A: 'a>(a: A) -> Apply!(brand: Self, signature: ('a, A: 'a) -> 'a) {
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
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::types::string;
	/// use std::rc::Rc;
	///
	/// let f = Pair(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2), "a".to_string());
	/// assert_eq!(apply::<RcFnBrand, PairWithSecondBrand<String>, _, _>(f, Pair(5, "b".to_string())), Pair(10, "ab".to_string()));
	/// ```
	fn apply<'a, FnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(brand: Self, signature: ('a, Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, B)): 'a) -> 'a),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) {
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
	fn bind<'a, F, A: 'a, B: 'a>(
		ma: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		f: F,
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
	where
		F: Fn(A) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) + 'a,
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
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_right::<RcFnBrand, PairWithSecondBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair(5, ())), 5);
	/// ```
	fn fold_right<'a, FnBrand, F, A: 'a, B: 'a>(
		f: F,
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
		FnBrand: ClonableFn + 'a,
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
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_left::<RcFnBrand, PairWithSecondBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair(5, ())), 5);
	/// ```
	fn fold_left<'a, FnBrand, F, A: 'a, B: 'a>(
		f: F,
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
		FnBrand: ClonableFn + 'a,
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
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, PairWithSecondBrand<()>, _, _, _>(|x: i32| x.to_string(), Pair(5, ())),
	///     "5".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, F, A: 'a, M>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
		FnBrand: ClonableFn + 'a,
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
	fn traverse<'a, F: Applicative, Func, A: 'a + Clone, B: 'a + Clone>(
		f: Func,
		ta: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): 'a) -> 'a)
	where
		Func: Fn(A) -> Apply!(brand: F, signature: ('a, B: 'a) -> 'a) + 'a,
		Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): Clone,
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
		ta: Apply!(brand: Self, signature: ('a, Apply!(brand: F, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	where
		Apply!(brand: F, signature: ('a, A: 'a) -> 'a): Clone,
		Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): Clone,
	{
		let Pair(first, second) = ta;
		F::map(move |a| Pair(a, second.clone()), first)
	}
}

impl<Second: 'static, FnBrand: SendClonableFn> ParFoldable<FnBrand>
	for PairWithSecondBrand<Second>
{
	/// Maps the value to a monoid and returns it in parallel (over first).
	///
	/// # Type Signature
	///
	/// `forall a m t. (ParFoldable (Pair' t), Monoid m, Send m, Sync m) => (f a m, Pair a t) -> m`
	///
	/// # Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// The combined monoid value.
	fn par_fold_map<'a, A, M>(
		func: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		func(fa.0)
	}

	/// Folds the pair from the right in parallel (over first).
	///
	/// # Type Signature
	///
	/// `forall a b t. ParFoldable (Pair' t) => (f (a, b) b, b, Pair a t) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The pair to fold.
	///
	/// # Returns
	///
	/// The final accumulator value.
	fn par_fold_right<'a, A, B>(
		f: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
	{
		f((fa.0, init))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::{ArcFnBrand, PairWithFirstBrand, PairWithSecondBrand, RcFnBrand},
		classes::{
			clonable_fn::ClonableFn,
			functor::map,
			par_foldable::{par_fold_map, par_fold_right},
			pointed::pure,
			semiapplicative::apply,
			semimonad::bind,
			send_clonable_fn::new_send,
		},
		functions::{compose, identity},
	};
	use quickcheck_macros::quickcheck;

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
			pure::<PairWithFirstBrand<String>, _>(<RcFnBrand as ClonableFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			pure::<PairWithFirstBrand<String>, _>(<RcFnBrand as ClonableFn>::new(f)),
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

		let u_fn = <RcFnBrand as ClonableFn>::new(move |x: i32| x.wrapping_add(u_seed));
		let u = pure::<PairWithFirstBrand<String>, _>(u_fn);

		let v_fn = <RcFnBrand as ClonableFn>::new(move |x: i32| x.wrapping_mul(v_seed));
		let v = pure::<PairWithFirstBrand<String>, _>(v_fn);

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		let compose_fn = <RcFnBrand as ClonableFn>::new(|f: std::rc::Rc<dyn Fn(i32) -> i32>| {
			let f = f.clone();
			<RcFnBrand as ClonableFn>::new(move |g: std::rc::Rc<dyn Fn(i32) -> i32>| {
				let f = f.clone();
				let g = g.clone();
				<RcFnBrand as ClonableFn>::new(move |x| f(g(x)))
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
		let u = pure::<PairWithFirstBrand<String>, _>(<RcFnBrand as ClonableFn>::new(f));

		let lhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			u.clone(),
			pure::<PairWithFirstBrand<String>, _>(y),
		);

		let rhs_fn = <RcFnBrand as ClonableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
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
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `PairWithFirstBrand`.
	#[test]
	fn par_fold_right_pair_with_first() {
		let x = Pair("a".to_string(), 1);
		let f = new_send::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, 10, x), 11);
	}

	// ParFoldable Tests for PairWithSecondBrand (Functor over First)

	/// Tests `par_fold_map` on `PairWithSecondBrand`.
	#[test]
	fn par_fold_map_pair_with_second() {
		let x = Pair(1, "a".to_string());
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `PairWithSecondBrand`.
	#[test]
	fn par_fold_right_pair_with_second() {
		let x = Pair(1, "a".to_string());
		let f = new_send::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, 10, x), 11);
	}
}
