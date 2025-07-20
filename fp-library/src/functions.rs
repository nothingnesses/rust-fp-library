//! Free generic helper functions, combinators and typeclass functions that dispatch to instance methods.

pub use super::typeclasses::{
	bind::bind, empty::empty, functor::map, pure::pure, sequence::sequence,
};

pub fn identity<A>(a: A) -> A {
	a
}
