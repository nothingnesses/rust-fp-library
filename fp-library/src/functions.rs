//! Generic, helper free functions and re-exports of free versions
//! of type class functions.

pub use crate::classes::{
	apply_first::apply_first,
	apply_second::apply_second,
	category::identity as category_identity,
	foldable::{fold_left, fold_map, fold_right},
	functor::map,
	monoid::empty,
	pointed::pure,
	semiapplicative::apply,
	semigroup::append,
	semigroupoid::compose as semigroupoid_compose,
	semimonad::bind,
	traversable::{sequence, traverse},
};

/// Takes functions `f` and `g` and returns the function `f . g` (`f` composed with `g`).
///
/// # Type Signature
///
/// `forall a b c. (b -> c) -> (a -> b) -> a -> c`
///
/// # Parameters
///
/// * `f`: A function from values of type `B` to values of type `C`.
/// * `g`: A function from values of type `A` to values of type `B`.
///
/// # Returns
///
/// A function from values of type `A` to values of type `C`.
///
/// # Examples
///
/// ```rust
/// use fp_library::functions::compose;
///
/// let add_one = |x: i32| x + 1;
/// let times_two = |x: i32| x * 2;
/// let times_two_add_one = compose(add_one, times_two);
///
/// // 3 * 2 + 1 = 7
/// assert_eq!(
///     times_two_add_one(3),
///     7
/// );
/// ```
pub fn compose<A, B, C, F, G>(
	f: F,
	g: G,
) -> impl Fn(A) -> C
where
	F: Fn(B) -> C,
	G: Fn(A) -> B,
{
	move |a| f(g(a))
}

/// Returns its first argument.
///
/// # Type Signature
///
/// `forall a b. a -> b -> a`
///
/// # Parameters
///
/// * `a`: A value.
/// * `b`: Some other value.
///
/// # Returns
///
/// The first value.
///
/// # Examples
///
/// ```rust
/// use fp_library::functions::constant;
///
/// assert_eq!(
///     constant(true)(false),
///     true
/// );
/// ```
pub fn constant<A: Clone, B>(a: A) -> impl Fn(B) -> A {
	move |_| a.clone()
}

/// Returns a version of the input binary function with its arguments flipped.
///
/// # Type Signature
///
/// `forall a b c. (a -> b -> c) -> b -> a -> c`
///
/// # Parameters
///
/// * `f`: A binary function.
///
/// # Returns
///
/// A version of `f` that takes its arguments in reverse.
///
/// # Examples
///
/// ```rust
/// use fp_library::functions::flip;
///
/// let subtract = |a, b| a - b;
///
/// // 0 - 1 = -1
/// assert_eq!(
///     flip(subtract)(1, 0),
///     -1
/// );
/// ```
pub fn flip<A, B, C, F>(f: F) -> impl Fn(B, A) -> C
where
	F: Fn(A, B) -> C,
{
	move |b, a| f(a, b)
}

/// Returns its input.
///
/// # Type Signature
///
/// `forall a. a -> a`
///
/// # Parameters
///
/// * `a`: A value.
///
/// # Returns
///
/// The same value.
///
/// # Examples
///
/// ```rust
/// use fp_library::functions::identity;
///
/// assert_eq!(
///     identity(()),
///     ()
/// );
/// ```
pub fn identity<A>(a: A) -> A {
	a
}
