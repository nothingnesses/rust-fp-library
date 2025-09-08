//! Generic, helper free functions, combinators and re-exports of free versions
//! of [type class][crate::classes] functions that dispatch to associated
//! functions of type class instances.

use crate::classes::{ClonableFn, clonable_fn::ApplyClonableFn};
pub use crate::classes::{
	apply_first::apply_first,
	apply_second::apply_second,
	category::category_identity,
	foldable::{fold_left, fold_map, fold_right},
	functor::map,
	monoid::empty,
	pointed::pure,
	semiapplicative::apply,
	semigroup::append,
	semigroupoid::semigroupoid_compose,
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
/// use fp_library::{brands::RcFnBrand, functions::compose};
/// use std::rc::Rc;
///
/// let add_one = Rc::new(|x: i32| x + 1);
/// let times_two = Rc::new(|x: i32| x * 2);
/// let times_two_add_one = compose::<RcFnBrand, _, _, _>(add_one)(times_two);
///
/// // 3 * 2 + 1 = 7
/// assert_eq!(
///     times_two_add_one(3),
///     7
/// );
/// ```
pub fn compose<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a, C: 'a>(
	f: ApplyClonableFn<'a, ClonableFnBrand, B, C>
) -> ApplyClonableFn<
	'a,
	ClonableFnBrand,
	ApplyClonableFn<'a, ClonableFnBrand, A, B>,
	ApplyClonableFn<'a, ClonableFnBrand, A, C>,
> {
	<ClonableFnBrand as ClonableFn>::new(move |g: ApplyClonableFn<'a, ClonableFnBrand, _, _>| {
		let f = f.clone();
		<ClonableFnBrand as ClonableFn>::new(move |a| f(g(a)))
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
/// use fp_library::{brands::RcFnBrand, functions::constant};
///
/// assert_eq!(
///     constant::<RcFnBrand, _, _>(true)(false),
///     true
/// );
/// ```
pub fn constant<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
	a: A
) -> ApplyClonableFn<'a, ClonableFnBrand, B, A> {
	<ClonableFnBrand as ClonableFn>::new(move |_b| a.to_owned())
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
/// use fp_library::{brands::RcFnBrand, functions::flip, classes::clonable_fn::ApplyClonableFn};
/// use std::rc::Rc;
///
/// let subtract: ApplyClonableFn<RcFnBrand, _, ApplyClonableFn<RcFnBrand, _, _>> = Rc::new(|a| Rc::new(move |b| a - b));
///
/// // 0 - 1 = -1
/// assert_eq!(
///     flip::<RcFnBrand, _, _, _>(subtract)(1)(0),
///     -1
/// );
/// ```
pub fn flip<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a + Clone, C: 'a>(
	f: ApplyClonableFn<'a, ClonableFnBrand, A, ApplyClonableFn<'a, ClonableFnBrand, B, C>>
) -> ApplyClonableFn<'a, ClonableFnBrand, B, ApplyClonableFn<'a, ClonableFnBrand, A, C>> {
	<ClonableFnBrand as ClonableFn>::new(move |b: B| {
		let f = f.clone();
		<ClonableFnBrand as ClonableFn>::new(move |a| (f(a))(b.to_owned()))
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
/// assert_eq!(
///     identity(()),
///     ()
/// );
/// ```
pub fn identity<A>(a: A) -> A {
	a
}
