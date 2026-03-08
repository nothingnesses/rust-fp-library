//! Trait for indexed traversal functions.

use crate::{
	Apply,
	classes::applicative::Applicative,
	kinds::*,
};

/// A trait for indexed traversal functions.
pub trait IndexedTraversalFunc<'a, I, S, T, A, B> {
	/// Apply the indexed traversal function.
	fn apply<M: Applicative>(
		&self,
		f: Box<dyn Fn(I, A) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>) + 'a>,
		s: S,
	) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, T>)
	where
		Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>): Clone;
}
