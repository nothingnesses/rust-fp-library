//! Implementations for [`Result`].

use crate::{
	Apply,
	brands::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		clonable_fn::ClonableFn, foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		pointed::Pointed, semiapplicative::Semiapplicative, semimonad::Semimonad,
		traversable::Traversable,
	},
	impl_kind,
	kinds::*,
};

impl_kind! {
	for ResultBrand {
		type Of<A, B> = Result<B, A>;
	}
}

// ResultWithErrBrand<E> (Functor over T)

impl_kind! {
	impl<E: 'static> for ResultWithErrBrand<E> {
		type Of<'a, A: 'a>: 'a = Result<A, E>;
	}
}

impl<E: 'static> Functor for ResultWithErrBrand<E> {
	/// Maps a function over the value in the result.
	///
	/// # Type Signature
	///
	/// `forall a b e. Functor (Result e) => (a -> b, Result e a) -> Result e b`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply.
	/// * `fa`: The result to map over.
	///
	/// # Returns
	///
	/// A new result containing the result of applying the function, or the original error.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::ResultWithErrBrand;
	///
	/// assert_eq!(map::<ResultWithErrBrand<()>, _, _, _>(|x: i32| x * 2, Ok(5)), Ok(10));
	/// assert_eq!(map::<ResultWithErrBrand<i32>, _, _, _>(|x: i32| x * 2, Err(1)), Err(1));
	/// ```
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B))
	where
		F: Fn(A) -> B + 'a,
	{
		fa.map(f)
	}
}

impl<E: Clone + 'static> Lift for ResultWithErrBrand<E> {
	/// Lifts a binary function into the result context.
	///
	/// # Type Signature
	///
	/// `forall a b c e. Lift (Result e) => ((a, b) -> c, Result e a, Result e b) -> Result e c`
	///
	/// # Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first result.
	/// * `fb`: The second result.
	///
	/// # Returns
	///
	/// `Ok(f(a, b))` if both results are `Ok`, otherwise the first error encountered.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::lift::lift2;
	/// use fp_library::brands::ResultWithErrBrand;
	///
	/// assert_eq!(
	///     lift2::<ResultWithErrBrand<()>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Ok(2)),
	///     Ok(3)
	/// );
	/// assert_eq!(
	///     lift2::<ResultWithErrBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Err(2)),
	///     Err(2)
	/// );
	/// ```
	fn lift2<'a, A, B, C, F>(
		f: F,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
		fb: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B)),
	) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (C))
	where
		F: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
	{
		match (fa, fb) {
			(Ok(a), Ok(b)) => Ok(f(a, b)),
			(Err(e), _) => Err(e),
			(_, Err(e)) => Err(e),
		}
	}
}

impl<E: 'static> Pointed for ResultWithErrBrand<E> {
	/// Wraps a value in a result.
	///
	/// # Type Signature
	///
	/// `forall a e. Pointed (Result e) => a -> Result e a`
	///
	/// # Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// # Returns
	///
	/// `Ok(a)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::ResultWithErrBrand;
	///
	/// assert_eq!(pure::<ResultWithErrBrand<()>, _>(5), Ok(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)) {
		Ok(a)
	}
}

impl<E: Clone + 'static> ApplyFirst for ResultWithErrBrand<E> {}
impl<E: Clone + 'static> ApplySecond for ResultWithErrBrand<E> {}

impl<E: Clone + 'static> Semiapplicative for ResultWithErrBrand<E> {
	/// Applies a wrapped function to a wrapped value.
	///
	/// # Type Signature
	///
	/// `forall a b e. Semiapplicative (Result e) => (Result e (a -> b), Result e a) -> Result e b`
	///
	/// # Parameters
	///
	/// * `ff`: The result containing the function.
	/// * `fa`: The result containing the value.
	///
	/// # Returns
	///
	/// `Ok(f(a))` if both are `Ok`, otherwise the first error encountered.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::ResultWithErrBrand;
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let f = Ok(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<ResultWithErrBrand<()>, _, _, RcFnBrand>(f, Ok(5)), Ok(10));
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
		ff: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (Apply!(FnBrand, ClonableFn, ('a), (A, B)))),
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B)) {
		match (ff, fa) {
			(Ok(f), Ok(a)) => Ok(f(a)),
			(Err(e), _) => Err(e),
			(_, Err(e)) => Err(e),
		}
	}
}

impl<E: Clone + 'static> Semimonad for ResultWithErrBrand<E> {
	/// Chains result computations.
	///
	/// # Type Signature
	///
	/// `forall a b e. Semimonad (Result e) => (Result e a, a -> Result e b) -> Result e b`
	///
	/// # Parameters
	///
	/// * `ma`: The first result.
	/// * `f`: The function to apply to the value inside the result.
	///
	/// # Returns
	///
	/// The result of applying `f` to the value if `ma` is `Ok`, otherwise the original error.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::ResultWithErrBrand;
	///
	/// assert_eq!(
	///     bind::<ResultWithErrBrand<()>, _, _, _>(Ok(5), |x| Ok(x * 2)),
	///     Ok(10)
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, F>(
		ma: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
		f: F,
	) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B))
	where
		F: Fn(A) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B)) + 'a,
	{
		ma.and_then(f)
	}
}

impl<E: 'static> Foldable for ResultWithErrBrand<E> {
	/// Folds the result from the right.
	///
	/// # Type Signature
	///
	/// `forall a b e. Foldable (Result e) => ((a, b) -> b, b, Result e a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// # Returns
	///
	/// `f(a, init)` if `fa` is `Ok(a)`, otherwise `init`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::ResultWithErrBrand;
	///
	/// assert_eq!(fold_right::<ResultWithErrBrand<()>, _, _, _>(|x, acc| x + acc, 0, Ok(5)), 5);
	/// assert_eq!(fold_right::<ResultWithErrBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Err(1)), 0);
	/// ```
	fn fold_right<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
	{
		match fa {
			Ok(a) => f(a, init),
			Err(_) => init,
		}
	}

	/// Folds the result from the left.
	///
	/// # Type Signature
	///
	/// `forall a b e. Foldable (Result e) => ((b, a) -> b, b, Result e a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// # Returns
	///
	/// `f(init, a)` if `fa` is `Ok(a)`, otherwise `init`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::ResultWithErrBrand;
	///
	/// assert_eq!(fold_left::<ResultWithErrBrand<()>, _, _, _>(|acc, x| acc + x, 0, Ok(5)), 5);
	/// ```
	fn fold_left<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
	{
		match fa {
			Ok(a) => f(init, a),
			Err(_) => init,
		}
	}

	/// Maps the value to a monoid and returns it.
	///
	/// # Type Signature
	///
	/// `forall a m e. (Foldable (Result e), Monoid m) => ((a) -> m, Result e a) -> m`
	///
	/// # Parameters
	///
	/// * `f`: The mapping function.
	/// * `fa`: The result to fold.
	///
	/// # Returns
	///
	/// `f(a)` if `fa` is `Ok(a)`, otherwise `M::empty()`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::ResultWithErrBrand;
	/// use fp_library::types::string;
	///
	/// assert_eq!(
	///     fold_map::<ResultWithErrBrand<()>, _, _, _>(|x: i32| x.to_string(), Ok(5)),
	///     "5".to_string()
	/// );
	/// ```
	fn fold_map<'a, A: 'a, M, F>(
		f: F,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
	{
		match fa {
			Ok(a) => f(a),
			Err(_) => M::empty(),
		}
	}
}

impl<E: Clone + 'static> Traversable for ResultWithErrBrand<E> {
	/// Traverses the result with an applicative function.
	///
	/// # Type Signature
	///
	/// `forall a b f e. (Traversable (Result e), Applicative f) => (a -> f b, Result e a) -> f (Result e b)`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply.
	/// * `ta`: The result to traverse.
	///
	/// # Returns
	///
	/// The result wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::{ResultWithErrBrand, OptionBrand};
	///
	/// assert_eq!(
	///     traverse::<ResultWithErrBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Ok(5)),
	///     Some(Ok(10))
	/// );
	/// ```
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
		f: Func,
		ta: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> Apply!(F, Kind_c3c3610c70409ee6, ('a), (Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B))))
	where
		Func: Fn(A) -> Apply!(F, Kind_c3c3610c70409ee6, ('a), (B)) + 'a,
		Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B)): Clone,
	{
		match ta {
			Ok(a) => F::map(|b| Ok(b), f(a)),
			Err(e) => F::pure(Err(e)),
		}
	}

	/// Sequences a result of applicative.
	///
	/// # Type Signature
	///
	/// `forall a f e. (Traversable (Result e), Applicative f) => (Result e (f a)) -> f (Result e a)`
	///
	/// # Parameters
	///
	/// * `ta`: The result containing the applicative value.
	///
	/// # Returns
	///
	/// The result wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::{ResultWithErrBrand, OptionBrand};
	///
	/// assert_eq!(
	///     sequence::<ResultWithErrBrand<()>, OptionBrand, _>(Ok(Some(5))),
	///     Some(Ok(5))
	/// );
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (Apply!(F, Kind_c3c3610c70409ee6, ('a), (A))))
	) -> Apply!(F, Kind_c3c3610c70409ee6, ('a), (Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A))))
	where
		Apply!(F, Kind_c3c3610c70409ee6, ('a), (A)): Clone,
		Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)): Clone,
	{
		match ta {
			Ok(fa) => F::map(|a| Ok(a), fa),
			Err(e) => F::pure(Err(e)),
		}
	}
}

// ResultWithOkBrand<T> (Functor over E)

impl_kind! {
	impl<T: 'static> for ResultWithOkBrand<T> {
		type Of<'a, A: 'a>: 'a = Result<T, A>;
	}
}

impl<T: 'static> Functor for ResultWithOkBrand<T> {
	/// Maps a function over the error value in the result.
	///
	/// # Type Signature
	///
	/// `forall a b t. Functor (Result' t) => (a -> b, Result t a) -> Result t b`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply to the error.
	/// * `fa`: The result to map over.
	///
	/// # Returns
	///
	/// A new result containing the mapped error, or the original success value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(map::<ResultWithOkBrand<i32>, _, _, _>(|x: i32| x * 2, Err(5)), Err(10));
	/// assert_eq!(map::<ResultWithOkBrand<i32>, _, _, _>(|x: i32| x * 2, Ok(1)), Ok(1));
	/// ```
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B))
	where
		F: Fn(A) -> B + 'a,
	{
		match fa {
			Ok(t) => Ok(t),
			Err(e) => Err(f(e)),
		}
	}
}

impl<T: Clone + 'static> Lift for ResultWithOkBrand<T> {
	/// Lifts a binary function into the result context (over error).
	///
	/// # Type Signature
	///
	/// `forall a b c t. Lift (Result' t) => ((a, b) -> c, Result t a, Result t b) -> Result t c`
	///
	/// # Parameters
	///
	/// * `f`: The binary function to apply to the errors.
	/// * `fa`: The first result.
	/// * `fb`: The second result.
	///
	/// # Returns
	///
	/// `Err(f(a, b))` if both results are `Err`, otherwise the first success encountered.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::lift::lift2;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(
	///     lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
	///     Err(3)
	/// );
	/// ```
	fn lift2<'a, A, B, C, F>(
		f: F,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
		fb: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B)),
	) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (C))
	where
		F: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
	{
		match (fa, fb) {
			(Err(a), Err(b)) => Err(f(a, b)),
			(Ok(t), _) => Ok(t),
			(_, Ok(t)) => Ok(t),
		}
	}
}

impl<T: 'static> Pointed for ResultWithOkBrand<T> {
	/// Wraps a value in a result (as error).
	///
	/// # Type Signature
	///
	/// `forall a t. Pointed (Result' t) => a -> Result t a`
	///
	/// # Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// # Returns
	///
	/// `Err(a)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(pure::<ResultWithOkBrand<()>, _>(5), Err(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)) {
		Err(a)
	}
}

impl<T: Clone + 'static> ApplyFirst for ResultWithOkBrand<T> {}
impl<T: Clone + 'static> ApplySecond for ResultWithOkBrand<T> {}

impl<T: Clone + 'static> Semiapplicative for ResultWithOkBrand<T> {
	/// Applies a wrapped function to a wrapped value (over error).
	///
	/// # Type Signature
	///
	/// `forall a b t. Semiapplicative (Result' t) => (Result t (a -> b), Result t a) -> Result t b`
	///
	/// # Parameters
	///
	/// * `ff`: The result containing the function (in Err).
	/// * `fa`: The result containing the value (in Err).
	///
	/// # Returns
	///
	/// `Err(f(a))` if both are `Err`, otherwise the first success encountered.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::ResultWithOkBrand;
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let f = Err(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<ResultWithOkBrand<()>, _, _, RcFnBrand>(f, Err(5)), Err(10));
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
		ff: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (Apply!(FnBrand, ClonableFn, ('a), (A, B)))),
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B)) {
		match (ff, fa) {
			(Err(f), Err(a)) => Err(f(a)),
			(Ok(t), _) => Ok(t),
			(_, Ok(t)) => Ok(t),
		}
	}
}

impl<T: Clone + 'static> Semimonad for ResultWithOkBrand<T> {
	/// Chains result computations (over error).
	///
	/// # Type Signature
	///
	/// `forall a b t. Semimonad (Result' t) => (Result t a, a -> Result t b) -> Result t b`
	///
	/// # Parameters
	///
	/// * `ma`: The first result.
	/// * `f`: The function to apply to the error value.
	///
	/// # Returns
	///
	/// The result of applying `f` to the error if `ma` is `Err`, otherwise the original success.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(
	///     bind::<ResultWithOkBrand<()>, _, _, _>(Err(5), |x| Err(x * 2)),
	///     Err(10)
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, F>(
		ma: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
		f: F,
	) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B))
	where
		F: Fn(A) -> Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B)) + 'a,
	{
		match ma {
			Ok(t) => Ok(t),
			Err(e) => f(e),
		}
	}
}

impl<T: 'static> Foldable for ResultWithOkBrand<T> {
	/// Folds the result from the right (over error).
	///
	/// # Type Signature
	///
	/// `forall a b t. Foldable (Result' t) => ((a, b) -> b, b, Result t a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// # Returns
	///
	/// `f(a, init)` if `fa` is `Err(a)`, otherwise `init`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(fold_right::<ResultWithOkBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Err(1)), 1);
	/// assert_eq!(fold_right::<ResultWithOkBrand<()>, _, _, _>(|x: i32, acc| x + acc, 0, Ok(())), 0);
	/// ```
	fn fold_right<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
	{
		match fa {
			Err(e) => f(e, init),
			Ok(_) => init,
		}
	}

	/// Folds the result from the left (over error).
	///
	/// # Type Signature
	///
	/// `forall a b t. Foldable (Result' t) => ((b, a) -> b, b, Result t a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// # Returns
	///
	/// `f(init, a)` if `fa` is `Err(a)`, otherwise `init`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(fold_left::<ResultWithOkBrand<()>, _, _, _>(|acc, x| acc + x, 0, Err(5)), 5);
	/// ```
	fn fold_left<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
	{
		match fa {
			Err(e) => f(init, e),
			Ok(_) => init,
		}
	}

	/// Maps the value to a monoid and returns it (over error).
	///
	/// # Type Signature
	///
	/// `forall a m t. (Foldable (Result' t), Monoid m) => ((a) -> m, Result t a) -> m`
	///
	/// # Parameters
	///
	/// * `f`: The mapping function.
	/// * `fa`: The result to fold.
	///
	/// # Returns
	///
	/// `f(a)` if `fa` is `Err(a)`, otherwise `M::empty()`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::ResultWithOkBrand;
	/// use fp_library::types::string;
	///
	/// assert_eq!(
	///     fold_map::<ResultWithOkBrand<()>, _, _, _>(|x: i32| x.to_string(), Err(5)),
	///     "5".to_string()
	/// );
	/// ```
	fn fold_map<'a, A: 'a, M, F>(
		f: F,
		fa: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
	{
		match fa {
			Err(e) => f(e),
			Ok(_) => M::empty(),
		}
	}
}

impl<T: Clone + 'static> Traversable for ResultWithOkBrand<T> {
	/// Traverses the result with an applicative function (over error).
	///
	/// # Type Signature
	///
	/// `forall a b f t. (Traversable (Result' t), Applicative f) => (a -> f b, Result t a) -> f (Result t b)`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply.
	/// * `ta`: The result to traverse.
	///
	/// # Returns
	///
	/// The result wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::{ResultWithOkBrand, OptionBrand};
	///
	/// assert_eq!(
	///     traverse::<ResultWithOkBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Err(5)),
	///     Some(Err(10))
	/// );
	/// ```
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
		f: Func,
		ta: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)),
	) -> Apply!(F, Kind_c3c3610c70409ee6, ('a), (Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B))))
	where
		Func: Fn(A) -> Apply!(F, Kind_c3c3610c70409ee6, ('a), (B)) + 'a,
		Apply!(Self, Kind_c3c3610c70409ee6, ('a), (B)): Clone,
	{
		match ta {
			Err(e) => F::map(|b| Err(b), f(e)),
			Ok(t) => F::pure(Ok(t)),
		}
	}

	/// Sequences a result of applicative (over error).
	///
	/// # Type Signature
	///
	/// `forall a f t. (Traversable (Result' t), Applicative f) => (Result t (f a)) -> f (Result t a)`
	///
	/// # Parameters
	///
	/// * `ta`: The result containing the applicative value.
	///
	/// # Returns
	///
	/// The result wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::{ResultWithOkBrand, OptionBrand};
	///
	/// assert_eq!(
	///     sequence::<ResultWithOkBrand<()>, OptionBrand, _>(Err(Some(5))),
	///     Some(Err(5))
	/// );
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(Self, Kind_c3c3610c70409ee6, ('a), (Apply!(F, Kind_c3c3610c70409ee6, ('a), (A))))
	) -> Apply!(F, Kind_c3c3610c70409ee6, ('a), (Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A))))
	where
		Apply!(F, Kind_c3c3610c70409ee6, ('a), (A)): Clone,
		Apply!(Self, Kind_c3c3610c70409ee6, ('a), (A)): Clone,
	{
		match ta {
			Err(fe) => F::map(|e| Err(e), fe),
			Ok(t) => F::pure(Ok(t)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::{OptionBrand, RcFnBrand},
		classes::{functor::map, pointed::pure, semiapplicative::apply, semimonad::bind},
		functions::{compose, identity},
	};
	use quickcheck_macros::quickcheck;

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
		apply::<ResultWithErrBrand<i32>, _, _, RcFnBrand>(
			pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as ClonableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<ResultWithErrBrand<i32>, _, _, RcFnBrand>(
			pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as ClonableFn>::new(f)),
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
			pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as ClonableFn>::new(v_fn))
		} else {
			Err(100)
		};
		let u = if u_is_ok {
			pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as ClonableFn>::new(u_fn))
		} else {
			Err(200)
		};

		// RHS: u <*> (v <*> w)
		let vw = apply::<ResultWithErrBrand<i32>, _, _, RcFnBrand>(v.clone(), w.clone());
		let rhs = apply::<ResultWithErrBrand<i32>, _, _, RcFnBrand>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let uv = match (u, v) {
			(Ok(uf), Ok(vf)) => {
				let composed = move |x| uf(vf(x));
				Ok(<RcFnBrand as ClonableFn>::new(composed))
			}
			(Err(e), _) => Err(e),
			(_, Err(e)) => Err(e),
		};

		let lhs = apply::<ResultWithErrBrand<i32>, _, _, RcFnBrand>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as ClonableFn>::new(f));

		let lhs = apply::<ResultWithErrBrand<i32>, _, _, RcFnBrand>(
			u.clone(),
			pure::<ResultWithErrBrand<i32>, _>(y),
		);

		let rhs_fn = <RcFnBrand as ClonableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<ResultWithErrBrand<i32>, _, _, RcFnBrand>(
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
			crate::classes::foldable::fold_right::<ResultWithErrBrand<i32>, _, _, _>(
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
			crate::classes::foldable::fold_left::<ResultWithErrBrand<i32>, _, _, _>(
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
			crate::classes::traversable::traverse::<ResultWithErrBrand<i32>, OptionBrand, _, _, _>(
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
			crate::classes::traversable::traverse::<ResultWithErrBrand<i32>, OptionBrand, _, _, _>(
				|_: i32| None::<i32>,
				Ok(1)
			),
			None
		);
	}
}
