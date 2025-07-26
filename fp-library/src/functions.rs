//! Generic, free helper functions, combinators and typeclass functions that
//! dispatch to instance methods.

pub use super::typeclasses::{
	apply::apply, apply_first::apply_first, apply_second::apply_second, bind::bind, functor::map,
	pure::pure,
};

/// forall a. a -> a
///
/// Returns its input.
///
/// Examples
///
/// ```rust
/// use fp_library::{functions::identity};
/// assert_eq!(identity(()), ());
/// ```
pub fn identity<A>(a: A) -> A {
	a
}

/// forall a b. a -> b -> a
///
/// Returns its first argument.
///
/// Examples
///
/// ```rust
/// use fp_library::{functions::constant};
/// assert_eq!(constant(true)(false), true);
/// ```
pub fn constant<A, B>(a: A) -> impl Fn(B) -> A
where
	A: Clone,
	B: Clone,
{
	move |_b| a.to_owned()
}
