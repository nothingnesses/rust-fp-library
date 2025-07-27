//! Generic, helper free functions, combinators and re-exports of [typeclass][crate::typeclasses]
//! functions that dispatch to instance methods.

pub use crate::typeclasses::{
	apply::apply, apply_first::apply_first, apply_second::apply_second, bind::bind, functor::map,
	pure::pure,
};

/// Takes a function `f`, returns a new function that takes a function `g`,
/// then returns the final composed function `f . g`.
///
/// forall a b c. (b -> c) -> (a -> b) -> a -> c
///
/// # Examples
///
/// ```rust
/// use fp_library::functions::compose;
///
/// let add_one = |x: i32| x + 1;
/// let times_two = |x: i32| x * 2;
/// let times_two_add_one = compose(add_one)(times_two);
///
/// assert_eq!(times_two_add_one(3), 7); // 3 * 2 + 1 = 7
/// ```
pub fn compose<'a, A, B, C, F, G>(f: F) -> impl Fn(G) -> Box<dyn Fn(A) -> C + 'a>
where
	F: Fn(B) -> C + Clone + 'a,
	G: Fn(A) -> B + 'a,
{
	move |g| {
		let f = f.to_owned();
		Box::new(move |a: A| f(g(a)))
	}
}

/// Returns its first argument.
///
/// forall a b. a -> b -> a
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
/// forall a b c. (a -> b -> c) -> b -> a -> c
///
/// # Examples
///
/// ```rust
/// use fp_library::functions::flip;
///
/// let subtract = |a| move |b| a - b;
///
/// assert_eq!(flip(subtract)(1)(0), -1); // 0 - 1 = -1
/// ```
pub fn flip<'a, A, B, C, F, G>(f: F) -> impl Fn(B) -> Box<dyn Fn(A) -> C + 'a>
where
	B: Clone + 'a,
	F: Fn(A) -> G + Clone + 'a,
	G: Fn(B) -> C,
{
	move |b| {
		let f = f.to_owned();
		Box::new(move |a| (f(a))(b.to_owned()))
	}
}

/// Returns its input.
///
/// forall a. a -> a
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
