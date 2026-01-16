//! Implementations for [`Result`].
//!
//! This module provides implementations of various type classes for the `Result` type.
//! `Result` can be treated as a functor/monad in two ways:
//! 1.  `ResultWithErrBrand<E>`: Functor over the success value `T` (fixing `E`). This is the standard `Result` monad.
//! 2.  `ResultWithOkBrand<T>`: Functor over the error value `E` (fixing `T`).

use crate::{
	Apply,
	brands::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		clonable_fn::ClonableFn, foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		par_foldable::ParFoldable, pointed::Pointed, semiapplicative::Semiapplicative,
		semimonad::Semimonad, send_clonable_fn::SendClonableFn, traversable::Traversable,
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
	/// This method applies a function to the value inside the result if it is `Ok`, producing a new result with the transformed value. If the result is `Err`, it is returned unchanged.
	///
	/// ### Type Signature
	///
	/// `forall a b e. Functor (Result e) => (a -> b, Result e a) -> Result e b`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the function to apply.
	/// * `A`: The type of the value inside the result.
	/// * `B`: The type of the result of applying the function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply.
	/// * `fa`: The result to map over.
	///
	/// ### Returns
	///
	/// A new result containing the result of applying the function, or the original error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::ResultWithErrBrand;
	///
	/// assert_eq!(map::<ResultWithErrBrand<()>, _, _, _>(|x: i32| x * 2, Ok(5)), Ok(10));
	/// assert_eq!(map::<ResultWithErrBrand<i32>, _, _, _>(|x: i32| x * 2, Err(1)), Err(1));
	/// ```
	fn map<'a, F, A: 'a, B: 'a>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
	where
		F: Fn(A) -> B + 'a,
	{
		fa.map(f)
	}
}

impl<E: Clone + 'static> Lift for ResultWithErrBrand<E> {
	/// Lifts a binary function into the result context.
	///
	/// This method lifts a binary function to operate on values within the result context.
	///
	/// ### Type Signature
	///
	/// `forall a b c e. Lift (Result e) => ((a, b) -> c, Result e a, Result e b) -> Result e c`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the binary function.
	/// * `A`: The type of the first value.
	/// * `B`: The type of the second value.
	/// * `C`: The type of the result.
	///
	/// ### Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first result.
	/// * `fb`: The second result.
	///
	/// ### Returns
	///
	/// `Ok(f(a, b))` if both results are `Ok`, otherwise the first error encountered.
	///
	/// ### Examples
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
	/// assert_eq!(
	///     lift2::<ResultWithErrBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Ok(2)),
	///     Err(1)
	/// );
	/// assert_eq!(
	///     lift2::<ResultWithErrBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
	///     Err(1)
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
	/// This method wraps a value in the `Ok` variant of a `Result`.
	///
	/// ### Type Signature
	///
	/// `forall a e. Pointed (Result e) => a -> Result e a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to wrap.
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// `Ok(a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::ResultWithErrBrand;
	///
	/// assert_eq!(pure::<ResultWithErrBrand<()>, _>(5), Ok(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(brand: Self, signature: ('a, A: 'a) -> 'a) {
		Ok(a)
	}
}

impl<E: Clone + 'static> ApplyFirst for ResultWithErrBrand<E> {}
impl<E: Clone + 'static> ApplySecond for ResultWithErrBrand<E> {}

impl<E: Clone + 'static> Semiapplicative for ResultWithErrBrand<E> {
	/// Applies a wrapped function to a wrapped value.
	///
	/// This method applies a function wrapped in a result to a value wrapped in a result.
	///
	/// ### Type Signature
	///
	/// `forall a b e. Semiapplicative (Result e) => (Result e (a -> b), Result e a) -> Result e b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function wrapper.
	/// * `A`: The type of the input value.
	/// * `B`: The type of the output value.
	///
	/// ### Parameters
	///
	/// * `ff`: The result containing the function.
	/// * `fa`: The result containing the value.
	///
	/// ### Returns
	///
	/// `Ok(f(a))` if both are `Ok`, otherwise the first error encountered.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::ResultWithErrBrand;
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let f = Ok(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<RcFnBrand, ResultWithErrBrand<()>, _, _>(f.clone(), Ok(5)), Ok(10));
	/// assert_eq!(apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(f, Err(1)), Err(1));
	///
	/// let f_err = Err(1);
	/// assert_eq!(apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(f_err, Ok(5)), Err(1));
	/// ```
	fn apply<'a, FnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(brand: Self, signature: ('a, Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, B)): 'a) -> 'a),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) {
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
	/// This method chains two computations, where the second computation depends on the result of the first.
	///
	/// ### Type Signature
	///
	/// `forall a b e. Semimonad (Result e) => (Result e a, a -> Result e b) -> Result e b`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the function to apply.
	/// * `A`: The type of the result of the first computation.
	/// * `B`: The type of the result of the second computation.
	///
	/// ### Parameters
	///
	/// * `ma`: The first result.
	/// * `f`: The function to apply to the value inside the result.
	///
	/// ### Returns
	///
	/// The result of applying `f` to the value if `ma` is `Ok`, otherwise the original error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::ResultWithErrBrand;
	///
	/// assert_eq!(
	///     bind::<ResultWithErrBrand<()>, _, _, _>(Ok(5), |x| Ok(x * 2)),
	///     Ok(10)
	/// );
	/// assert_eq!(
	///     bind::<ResultWithErrBrand<i32>, _, _, _>(Ok(5), |_| Err(1)),
	///     Err(1)
	/// );
	/// assert_eq!(
	///     bind::<ResultWithErrBrand<i32>, _, _, _>(Err(1), |x: i32| Ok(x * 2)),
	///     Err(1)
	/// );
	/// ```
	fn bind<'a, F, A: 'a, B: 'a>(
		ma: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		f: F,
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
	where
		F: Fn(A) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) + 'a,
	{
		ma.and_then(f)
	}
}

impl<E: 'static> Foldable for ResultWithErrBrand<E> {
	/// Folds the result from the right.
	///
	/// This method performs a right-associative fold of the result.
	///
	/// ### Type Signature
	///
	/// `forall a b e. Foldable (Result e) => ((a, b) -> b, b, Result e a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `F`: The type of the folding function.
	/// * `A`: The type of the elements in the structure.
	/// * `B`: The type of the accumulator.
	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// ### Returns
	///
	/// `func(a, initial)` if `fa` is `Ok(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::ResultWithErrBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_right::<RcFnBrand, ResultWithErrBrand<()>, _, _, _>(|x, acc| x + acc, 0, Ok(5)), 5);
	/// assert_eq!(fold_right::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Err(1)), 0);
	/// ```
	fn fold_right<'a, FnBrand, F, A: 'a, B: 'a>(
		func: F,
		initial: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Ok(a) => func(a, initial),
			Err(_) => initial,
		}
	}

	/// Folds the result from the left.
	///
	/// This method performs a left-associative fold of the result.
	///
	/// ### Type Signature
	///
	/// `forall a b e. Foldable (Result e) => ((b, a) -> b, b, Result e a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `Func`: The type of the folding function.
	/// * `A`: The type of the elements in the structure.
	/// * `B`: The type of the accumulator.
	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// ### Returns
	///
	/// `func(initial, a)` if `fa` is `Ok(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::ResultWithErrBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_left::<RcFnBrand, ResultWithErrBrand<()>, _, _, _>(|acc, x| acc + x, 0, Ok(5)), 5);
	/// assert_eq!(fold_left::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(|acc, x: i32| acc + x, 0, Err(1)), 0);
	/// ```
	fn fold_left<'a, FnBrand, F, A: 'a, B: 'a>(
		func: F,
		initial: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Ok(a) => func(initial, a),
			Err(_) => initial,
		}
	}

	/// Maps the value to a monoid and returns it.
	///
	/// This method maps the element of the result to a monoid and then returns it.
	///
	/// ### Type Signature
	///
	/// `forall a m e. (Foldable (Result e), Monoid m) => ((a) -> m, Result e a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `Func`: The type of the mapping function.
	/// * `A`: The type of the elements in the structure.
	/// * `M`: The type of the monoid.
	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The result to fold.
	///
	/// ### Returns
	///
	/// `func(a)` if `fa` is `Ok(a)`, otherwise `M::empty()`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::ResultWithErrBrand;
	/// use fp_library::types::string;
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, ResultWithErrBrand<()>, _, _, _>(|x: i32| x.to_string(), Ok(5)),
	///     "5".to_string()
	/// );
	/// assert_eq!(
	///     fold_map::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(|x: i32| x.to_string(), Err(1)),
	///     "".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, F, A: 'a, M>(
		func: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Ok(a) => func(a),
			Err(_) => M::empty(),
		}
	}
}

impl<E: Clone + 'static> Traversable for ResultWithErrBrand<E> {
	/// Traverses the result with an applicative function.
	///
	/// This method maps the element of the result to a computation, evaluates it, and combines the result into an applicative context.
	///
	/// ### Type Signature
	///
	/// `forall a b f e. (Traversable (Result e), Applicative f) => (a -> f b, Result e a) -> f (Result e b)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `Func`: The type of the function to apply.
	/// * `A`: The type of the elements in the traversable structure.
	/// * `B`: The type of the elements in the resulting traversable structure.
	///
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
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::{ResultWithErrBrand, OptionBrand};
	///
	/// assert_eq!(
	///     traverse::<ResultWithErrBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Ok(5)),
	///     Some(Ok(10))
	/// );
	/// assert_eq!(
	///     traverse::<ResultWithErrBrand<i32>, OptionBrand, _, _, _>(|x: i32| Some(x * 2), Err(1)),
	///     Some(Err(1))
	/// );
	/// assert_eq!(
	///     traverse::<ResultWithErrBrand<()>, OptionBrand, _, _, _>(|_| None::<i32>, Ok(5)),
	///     None
	/// );
	/// ```
	fn traverse<'a, F: Applicative, Func, A: 'a + Clone, B: 'a + Clone>(
		func: Func,
		ta: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): 'a) -> 'a)
	where
		Func: Fn(A) -> Apply!(brand: F, signature: ('a, B: 'a) -> 'a) + 'a,
		Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): Clone,
	{
		match ta {
			Ok(a) => F::map(|b| Ok(b), func(a)),
			Err(e) => F::pure(Err(e)),
		}
	}

	/// Sequences a result of applicative.
	///
	/// This method evaluates the computation inside the result and accumulates the result into an applicative context.
	///
	/// ### Type Signature
	///
	/// `forall a f e. (Traversable (Result e), Applicative f) => (Result e (f a)) -> f (Result e a)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `A`: The type of the elements in the traversable structure.
	///
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
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::{ResultWithErrBrand, OptionBrand};
	///
	/// assert_eq!(
	///     sequence::<ResultWithErrBrand<()>, OptionBrand, _>(Ok(Some(5))),
	///     Some(Ok(5))
	/// );
	/// assert_eq!(
	///     sequence::<ResultWithErrBrand<i32>, OptionBrand, _>(Err(1)),
	///     Some(Err(1))
	/// );
	/// assert_eq!(
	///     sequence::<ResultWithErrBrand<()>, OptionBrand, _>(Ok(None::<i32>)),
	///     None
	/// );
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(brand: Self, signature: ('a, Apply!(brand: F, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	where
		Apply!(brand: F, signature: ('a, A: 'a) -> 'a): Clone,
		Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): Clone,
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
	/// This method applies a function to the error value inside the result if it is `Err`, producing a new result with the transformed error. If the result is `Ok`, it is returned unchanged.
	///
	/// ### Type Signature
	///
	/// `forall a b t. Functor (Result' t) => (a -> b, Result t a) -> Result t b`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the function to apply.
	/// * `A`: The type of the error value inside the result.
	/// * `B`: The type of the result of applying the function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the error.
	/// * `fa`: The result to map over.
	///
	/// ### Returns
	///
	/// A new result containing the mapped error, or the original success value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(map::<ResultWithOkBrand<i32>, _, _, _>(|x: i32| x * 2, Err(5)), Err(10));
	/// assert_eq!(map::<ResultWithOkBrand<i32>, _, _, _>(|x: i32| x * 2, Ok(1)), Ok(1));
	/// ```
	fn map<'a, F, A: 'a, B: 'a>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
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
	/// This method lifts a binary function to operate on error values within the result context.
	///
	/// ### Type Signature
	///
	/// `forall a b c t. Lift (Result' t) => ((a, b) -> c, Result t a, Result t b) -> Result t c`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the binary function.
	/// * `A`: The type of the first error value.
	/// * `B`: The type of the second error value.
	/// * `C`: The type of the result error value.
	///
	/// ### Parameters
	///
	/// * `f`: The binary function to apply to the errors.
	/// * `fa`: The first result.
	/// * `fb`: The second result.
	///
	/// ### Returns
	///
	/// `Err(f(a, b))` if both results are `Err`, otherwise the first success encountered.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::lift::lift2;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(
	///     lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Err(2)),
	///     Err(3)
	/// );
	/// assert_eq!(
	///     lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Err(1), Ok(2)),
	///     Ok(2)
	/// );
	/// assert_eq!(
	///     lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Err(2)),
	///     Ok(1)
	/// );
	/// assert_eq!(
	///     lift2::<ResultWithOkBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Ok(1), Ok(2)),
	///     Ok(1)
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
	/// This method wraps a value in the `Err` variant of a `Result`.
	///
	/// ### Type Signature
	///
	/// `forall a t. Pointed (Result' t) => a -> Result t a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to wrap.
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// `Err(a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(pure::<ResultWithOkBrand<()>, _>(5), Err(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(brand: Self, signature: ('a, A: 'a) -> 'a) {
		Err(a)
	}
}

impl<T: Clone + 'static> ApplyFirst for ResultWithOkBrand<T> {}
impl<T: Clone + 'static> ApplySecond for ResultWithOkBrand<T> {}

impl<T: Clone + 'static> Semiapplicative for ResultWithOkBrand<T> {
	/// Applies a wrapped function to a wrapped value (over error).
	///
	/// This method applies a function wrapped in a result (as error) to a value wrapped in a result (as error).
	///
	/// ### Type Signature
	///
	/// `forall a b t. Semiapplicative (Result' t) => (Result t (a -> b), Result t a) -> Result t b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function wrapper.
	/// * `A`: The type of the input value.
	/// * `B`: The type of the output value.
	///
	/// ### Parameters
	///
	/// * `ff`: The result containing the function (in Err).
	/// * `fa`: The result containing the value (in Err).
	///
	/// ### Returns
	///
	/// `Err(f(a))` if both are `Err`, otherwise the first success encountered.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::ResultWithOkBrand;
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let f = Err(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<RcFnBrand, ResultWithOkBrand<()>, _, _>(f.clone(), Err(5)), Err(10));
	/// assert_eq!(apply::<RcFnBrand, ResultWithOkBrand<i32>, _, _>(f, Ok(1)), Ok(1));
	///
	/// let f_ok = Ok(1);
	/// assert_eq!(apply::<RcFnBrand, ResultWithOkBrand<i32>, _, _>(f_ok, Err(5)), Ok(1));
	/// ```
	fn apply<'a, FnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(brand: Self, signature: ('a, Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, B)): 'a) -> 'a),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) {
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
	/// This method chains two computations, where the second computation depends on the result of the first (over error).
	///
	/// ### Type Signature
	///
	/// `forall a b t. Semimonad (Result' t) => (Result t a, a -> Result t b) -> Result t b`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the function to apply.
	/// * `A`: The type of the result of the first computation.
	/// * `B`: The type of the result of the second computation.
	///
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
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::ResultWithOkBrand;
	///
	/// assert_eq!(
	///     bind::<ResultWithOkBrand<()>, _, _, _>(Err(5), |x| Err(x * 2)),
	///     Err(10)
	/// );
	/// assert_eq!(
	///     bind::<ResultWithOkBrand<i32>, _, _, _>(Err(5), |_| Ok(1)),
	///     Ok(1)
	/// );
	/// assert_eq!(
	///     bind::<ResultWithOkBrand<i32>, _, _, _>(Ok(1), |x: i32| Err(x * 2)),
	///     Ok(1)
	/// );
	/// ```
	fn bind<'a, F, A: 'a, B: 'a>(
		ma: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		f: F,
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
	where
		F: Fn(A) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) + 'a,
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
	/// This method performs a right-associative fold of the result (over error).
	///
	/// ### Type Signature
	///
	/// `forall a b t. Foldable (Result' t) => ((a, b) -> b, b, Result t a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `Func`: The type of the folding function.
	/// * `A`: The type of the elements in the structure.
	/// * `B`: The type of the accumulator.
	///
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
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::ResultWithOkBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_right::<RcFnBrand, ResultWithOkBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Err(1)), 1);
	/// assert_eq!(fold_right::<RcFnBrand, ResultWithOkBrand<()>, _, _, _>(|x: i32, acc| x + acc, 0, Ok(())), 0);
	/// ```
	fn fold_right<'a, FnBrand, F, A: 'a, B: 'a>(
		func: F,
		initial: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Err(e) => func(e, initial),
			Ok(_) => initial,
		}
	}

	/// Folds the result from the left (over error).
	///
	/// This method performs a left-associative fold of the result (over error).
	///
	/// ### Type Signature
	///
	/// `forall a b t. Foldable (Result' t) => ((b, a) -> b, b, Result t a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `Func`: The type of the folding function.
	/// * `A`: The type of the elements in the structure.
	/// * `B`: The type of the accumulator.
	///
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
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::ResultWithOkBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(fold_left::<RcFnBrand, ResultWithOkBrand<()>, _, _, _>(|acc, x: i32| acc + x, 0, Err(5)), 5);
	/// assert_eq!(fold_left::<RcFnBrand, ResultWithOkBrand<i32>, _, _, _>(|acc, x: i32| acc + x, 0, Ok(1)), 0);
	/// ```
	fn fold_left<'a, FnBrand, F, A: 'a, B: 'a>(
		func: F,
		initial: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Err(e) => func(initial, e),
			Ok(_) => initial,
		}
	}

	/// Maps the value to a monoid and returns it (over error).
	///
	/// This method maps the element of the result to a monoid and then returns it (over error).
	///
	/// ### Type Signature
	///
	/// `forall a m t. (Foldable (Result' t), Monoid m) => ((a) -> m, Result t a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `Func`: The type of the mapping function.
	/// * `A`: The type of the elements in the structure.
	/// * `M`: The type of the monoid.
	///
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
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::ResultWithOkBrand;
	/// use fp_library::types::string;
	/// use fp_library::brands::RcFnBrand;
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, ResultWithOkBrand<()>, _, _, _>(|x: i32| x.to_string(), Err(5)),
	///     "5".to_string()
	/// );
	/// assert_eq!(
	///     fold_map::<RcFnBrand, ResultWithOkBrand<i32>, _, _, _>(|x: i32| x.to_string(), Ok(1)),
	///     "".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, Func, A: 'a, M>(
		func: Func,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Err(e) => func(e),
			Ok(_) => M::empty(),
		}
	}
}

impl<T: Clone + 'static> Traversable for ResultWithOkBrand<T> {
	/// Traverses the result with an applicative function (over error).
	///
	/// This method maps the element of the result to a computation, evaluates it, and combines the result into an applicative context (over error).
	///
	/// ### Type Signature
	///
	/// `forall a b f t. (Traversable (Result' t), Applicative f) => (a -> f b, Result t a) -> f (Result t b)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `Func`: The type of the function to apply.
	/// * `A`: The type of the elements in the traversable structure.
	/// * `B`: The type of the elements in the resulting traversable structure.
	///
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
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::{ResultWithOkBrand, OptionBrand};
	///
	/// assert_eq!(
	///     traverse::<ResultWithOkBrand<()>, OptionBrand, _, _, _>(|x| Some(x * 2), Err(5)),
	///     Some(Err(10))
	/// );
	/// assert_eq!(
	///     traverse::<ResultWithOkBrand<i32>, OptionBrand, _, _, _>(|x: i32| Some(x * 2), Ok(1)),
	///     Some(Ok(1))
	/// );
	/// assert_eq!(
	///     traverse::<ResultWithOkBrand<()>, OptionBrand, _, _, _>(|_| None::<i32>, Err(5)),
	///     None
	/// );
	/// ```
	fn traverse<'a, F: Applicative, Func, A: 'a + Clone, B: 'a + Clone>(
		func: Func,
		ta: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): 'a) -> 'a)
	where
		Func: Fn(A) -> Apply!(brand: F, signature: ('a, B: 'a) -> 'a) + 'a,
		Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): Clone,
	{
		match ta {
			Err(e) => F::map(|b| Err(b), func(e)),
			Ok(t) => F::pure(Ok(t)),
		}
	}

	/// Sequences a result of applicative (over error).
	///
	/// This method evaluates the computation inside the result and accumulates the result into an applicative context (over error).
	///
	/// ### Type Signature
	///
	/// `forall a f t. (Traversable (Result' t), Applicative f) => (Result t (f a)) -> f (Result t a)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `A`: The type of the elements in the traversable structure.
	///
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
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::{ResultWithOkBrand, OptionBrand};
	///
	/// assert_eq!(
	///     sequence::<ResultWithOkBrand<()>, OptionBrand, _>(Err(Some(5))),
	///     Some(Err(5))
	/// );
	/// assert_eq!(
	///     sequence::<ResultWithOkBrand<i32>, OptionBrand, _>(Ok(1)),
	///     Some(Ok(1))
	/// );
	/// assert_eq!(
	///     sequence::<ResultWithOkBrand<()>, OptionBrand, _>(Err(None::<i32>)),
	///     None
	/// );
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(brand: Self, signature: ('a, Apply!(brand: F, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	where
		Apply!(brand: F, signature: ('a, A: 'a) -> 'a): Clone,
		Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): Clone,
	{
		match ta {
			Err(fe) => F::map(|e| Err(e), fe),
			Ok(t) => F::pure(Ok(t)),
		}
	}
}

impl<E: 'static, FnBrand: SendClonableFn> ParFoldable<FnBrand> for ResultWithErrBrand<E> {
	/// Maps the value to a monoid and returns it, or returns empty, in parallel.
	///
	/// This method maps the element of the result to a monoid and then returns it. The mapping operation may be executed in parallel.
	///
	/// ### Type Signature
	///
	/// `forall a m e. (ParFoldable (Result e), Monoid m, Send m, Sync m) => (f a m, Result e a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of thread-safe function to use (must implement `SendClonableFn`).
	/// * `A`: The element type (must be `Send + Sync`).
	/// * `M`: The monoid type (must be `Send + Sync`).
	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to map each element to a monoid.
	/// * `fa`: The result to fold.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::par_foldable::par_fold_map;
	/// use fp_library::brands::{ResultWithErrBrand, ArcFnBrand};
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
	/// use fp_library::types::string;
	///
	/// let x: Result<i32, ()> = Ok(5);
	/// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x.to_string());
	/// assert_eq!(par_fold_map::<ArcFnBrand, ResultWithErrBrand<()>, _, _>(f.clone(), x), "5".to_string());
	///
	/// let x_err: Result<i32, i32> = Err(1);
	/// assert_eq!(par_fold_map::<ArcFnBrand, ResultWithErrBrand<i32>, _, _>(f, x_err), "".to_string());
	/// ```
	fn par_fold_map<'a, A, M>(
		func: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		match fa {
			Ok(a) => func(a),
			Err(_) => M::empty(),
		}
	}

	/// Folds the result from the right in parallel.
	///
	/// This method folds the result by applying a function from right to left, potentially in parallel.
	///
	/// ### Type Signature
	///
	/// `forall a b e. ParFoldable (Result e) => (f (a, b) b, b, Result e a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of thread-safe function to use.
	/// * `A`: The element type (must be `Send + Sync`).
	/// * `B`: The accumulator type (must be `Send + Sync`).
	///
	/// ### Parameters
	///
	/// * `f`: The thread-safe function to apply to each element and the accumulator.
	/// * `init`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::par_foldable::par_fold_right;
	/// use fp_library::brands::{ResultWithErrBrand, ArcFnBrand};
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
	///
	/// let x: Result<i32, ()> = Ok(5);
	/// let f = <ArcFnBrand as SendClonableFn>::new_send(|(a, b): (i32, i32)| a + b);
	/// assert_eq!(par_fold_right::<ArcFnBrand, ResultWithErrBrand<()>, _, _>(f.clone(), 10, x), 15);
	///
	/// let x_err: Result<i32, i32> = Err(1);
	/// assert_eq!(par_fold_right::<ArcFnBrand, ResultWithErrBrand<i32>, _, _>(f, 10, x_err), 10);
	fn par_fold_right<'a, A, B>(
		func: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
		initial: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
	{
		match fa {
			Ok(a) => func((a, initial)),
			Err(_) => initial,
		}
	}
}

impl<T: 'static, FnBrand: SendClonableFn> ParFoldable<FnBrand> for ResultWithOkBrand<T> {
	/// Maps the value to a monoid and returns it, or returns empty, in parallel (over error).
	///
	/// This method maps the element of the result to a monoid and then returns it (over error). The mapping operation may be executed in parallel.
	///
	/// ### Type Signature
	///
	/// `forall a m t. (ParFoldable (Result' t), Monoid m, Send m, Sync m) => (f a m, Result t a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of thread-safe function to use (must implement `SendClonableFn`).
	/// * `A`: The element type (must be `Send + Sync`).
	/// * `M`: The monoid type (must be `Send + Sync`).
	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to map each element to a monoid.
	/// * `fa`: The result to fold.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::par_foldable::par_fold_map;
	/// use fp_library::brands::{ResultWithOkBrand, ArcFnBrand};
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
	/// use fp_library::types::string;
	///
	/// let x: Result<(), i32> = Err(5);
	/// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x.to_string());
	/// assert_eq!(par_fold_map::<ArcFnBrand, ResultWithOkBrand<()>, _, _>(f.clone(), x), "5".to_string());
	///
	/// let x_ok: Result<i32, i32> = Ok(1);
	/// assert_eq!(par_fold_map::<ArcFnBrand, ResultWithOkBrand<i32>, _, _>(f, x_ok), "".to_string());
	/// ```
	fn par_fold_map<'a, A, M>(
		func: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		match fa {
			Err(e) => func(e),
			Ok(_) => M::empty(),
		}
	}

	/// Folds the result from the right in parallel (over error).
	///
	/// This method folds the result by applying a function from right to left, potentially in parallel (over error).
	///
	/// ### Type Signature
	///
	/// `forall a b t. ParFoldable (Result' t) => (f (a, b) b, b, Result t a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of thread-safe function to use.
	/// * `A`: The element type (must be `Send + Sync`).
	/// * `B`: The accumulator type (must be `Send + Sync`).
	///
	/// ### Parameters
	///
	/// * `f`: The thread-safe function to apply to each element and the accumulator.
	/// * `init`: The initial value.
	/// * `fa`: The result to fold.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::par_foldable::par_fold_right;
	/// use fp_library::brands::{ResultWithOkBrand, ArcFnBrand};
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
	///
	/// let x: Result<(), i32> = Err(5);
	/// let f = <ArcFnBrand as SendClonableFn>::new_send(|(a, b): (i32, i32)| a + b);
	/// assert_eq!(par_fold_right::<ArcFnBrand, ResultWithOkBrand<()>, _, _>(f.clone(), 10, x), 15);
	///
	/// let x_ok: Result<i32, i32> = Ok(1);
	/// assert_eq!(par_fold_right::<ArcFnBrand, ResultWithOkBrand<i32>, _, _>(f, 10, x_ok), 10);
	fn par_fold_right<'a, A, B>(
		func: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: ((A, B), B)),
		initial: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
	{
		match fa {
			Err(e) => func((e, initial)),
			Ok(_) => initial,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::{ArcFnBrand, OptionBrand, RcFnBrand},
		classes::{
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
		apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(
			pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as ClonableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(
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
		let vw = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(u.clone(), vw);

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

		let lhs = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<ResultWithErrBrand<i32>, _>(<RcFnBrand as ClonableFn>::new(f));

		let lhs = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(
			u.clone(),
			pure::<ResultWithErrBrand<i32>, _>(y),
		);

		let rhs_fn = <RcFnBrand as ClonableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(
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
			crate::classes::foldable::fold_right::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(
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
			crate::classes::foldable::fold_left::<RcFnBrand, ResultWithErrBrand<i32>, _, _, _>(
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

	// ParFoldable Tests for ResultWithErrBrand

	/// Tests `par_fold_map` on `Ok`.
	#[test]
	fn par_fold_map_ok() {
		let x: Result<i32, ()> = Ok(5);
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, ResultWithErrBrand<()>, _, _>(f, x), "5".to_string());
	}

	/// Tests `par_fold_map` on `Err`.
	#[test]
	fn par_fold_map_err_val() {
		let x: Result<i32, i32> = Err(5);
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, ResultWithErrBrand<i32>, _, _>(f, x), "".to_string());
	}

	/// Tests `par_fold_right` on `Ok`.
	#[test]
	fn par_fold_right_ok() {
		let x: Result<i32, ()> = Ok(5);
		let f = new_send::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultWithErrBrand<()>, _, _>(f, 10, x), 15);
	}

	/// Tests `par_fold_right` on `Err`.
	#[test]
	fn par_fold_right_err_val() {
		let x: Result<i32, i32> = Err(5);
		let f = new_send::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultWithErrBrand<i32>, _, _>(f, 10, x), 10);
	}

	// ParFoldable Tests for ResultWithOkBrand

	/// Tests `par_fold_map` on `Err` (which holds the value for ResultWithOkBrand).
	#[test]
	fn par_fold_map_err_ok_brand() {
		let x: Result<(), i32> = Err(5);
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, ResultWithOkBrand<()>, _, _>(f, x), "5".to_string());
	}

	/// Tests `par_fold_map` on `Ok` (which is empty for ResultWithOkBrand).
	#[test]
	fn par_fold_map_ok_ok_brand() {
		let x: Result<i32, i32> = Ok(5);
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, ResultWithOkBrand<i32>, _, _>(f, x), "".to_string());
	}

	/// Tests `par_fold_right` on `Err` (which holds the value for ResultWithOkBrand).
	#[test]
	fn par_fold_right_err_ok_brand() {
		let x: Result<(), i32> = Err(5);
		let f = new_send::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultWithOkBrand<()>, _, _>(f, 10, x), 15);
	}

	/// Tests `par_fold_right` on `Ok` (which is empty for ResultWithOkBrand).
	#[test]
	fn par_fold_right_ok_ok_brand() {
		let x: Result<i32, i32> = Ok(5);
		let f = new_send::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, ResultWithOkBrand<i32>, _, _>(f, 10, x), 10);
	}
}
