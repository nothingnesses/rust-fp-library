//! Generic, free helper functions, combinators and typeclass functions that
//! dispatch to instance methods.

pub use super::typeclasses::{
	bind::bind, empty::empty, functor::map, pure::pure, sequence::sequence,
};

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
