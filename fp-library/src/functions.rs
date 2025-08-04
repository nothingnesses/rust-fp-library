//! Generic, helper free functions, combinators and re-exports of [typeclass][crate::typeclasses]
//! free functions that dispatch to associated functions of typeclass instances.

use crate::aliases::ClonableFn;
pub use crate::typeclasses::{
	apply::apply,
	apply_first::apply_first,
	apply_second::apply_second,
	bind::bind,
	foldable::{fold_left, fold_map, fold_right},
	functor::map,
	monoid::empty,
	pure::pure,
	semigroup::append,
};
use std::sync::Arc;

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
/// use std::sync::Arc;
///
/// let add_one = Arc::new(|x: i32| x + 1);
/// let times_two = Arc::new(|x: i32| x * 2);
/// let times_two_add_one = compose(add_one)(times_two);
///
/// assert_eq!(times_two_add_one(3), 7); // 3 * 2 + 1 = 7
/// ```
pub fn compose<'a, A: 'a, B: 'a, C: 'a>(
	f: ClonableFn<'a, B, C>
) -> ClonableFn<'a, ClonableFn<'a, A, B>, ClonableFn<'a, A, C>> {
	Arc::new(move |g: ClonableFn<'a, A, B>| {
		let f = f.clone();
		Arc::new(move |a: A| f(g(a)))
	})
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
/// use fp_library::{functions::constant};
///
/// assert_eq!(constant(true)(false), true);
/// ```
pub fn constant<A, B>(a: A) -> impl Fn(B) -> A
where
	A: Clone,
	B: Clone,
{
	move |_b| a.to_owned()
}

/// Returns a version of the input curried binary function
/// with its arguments flipped.
///
/// # Type Signature
///
/// `forall a b c. (a -> b -> c) -> b -> a -> c`
///
/// # Parameters
///
/// * `f`: A curried binary function.
///
/// # Returns
///
/// A version of `f` that takes its arguments in reverse.
///
/// # Examples
///
/// ```rust
/// use fp_library::{aliases::ClonableFn, functions::flip};
/// use std::sync::Arc;
///
/// let subtract: ClonableFn<_, ClonableFn<_, _>> = Arc::new(|a| Arc::new(move |b| a - b));
///
/// assert_eq!(flip(subtract)(1)(0), -1); // 0 - 1 = -1
/// ```
pub fn flip<'a, A: 'a, B: 'a + Clone, C: 'a>(
	f: ClonableFn<'a, A, ClonableFn<'a, B, C>>
) -> ClonableFn<'a, B, ClonableFn<'a, A, C>> {
	Arc::new(move |b: B| {
		let f = f.clone();
		Arc::new(move |a: A| (f(a))(b.to_owned()))
	})
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
/// assert_eq!(identity(()), ());
/// ```
pub fn identity<A>(a: A) -> A {
	a
}
