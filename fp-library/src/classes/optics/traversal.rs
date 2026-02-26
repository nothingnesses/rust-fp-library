//! Traversal optic traits.

use crate::{
	Apply,
	classes::applicative::Applicative,
	kinds::*,
};

/// A trait for traversal functions.
///
/// This trait represents a function that can traverse a structure `S` to produce `T`,
/// given a function `A -> F B` for any [`Applicative`] `F`.
pub trait TraversalFunc<'a, S, T, A, B> {
	/// Apply the traversal to a structure.
	///
	/// This is not object-safe because of the generic parameter `M`.
	/// To use this in an optic, it must be used with a concrete type or a wrapper.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	Apply,
	/// 	classes::{
	/// 		Applicative,
	/// 		optics::traversal::TraversalFunc,
	/// 		profunctor::*,
	/// 	},
	/// 	kinds::*,
	/// };
	///
	/// struct ListTraversal;
	/// impl<'a, A: 'a> TraversalFunc<'a, Vec<A>, Vec<A>, A, A> for ListTraversal {
	/// 	fn apply<M: Applicative>(
	/// 		&self,
	/// 		f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a>,
	/// 		s: Vec<A>,
	/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
	/// 		// Implementation would use M::sequence or similar
	/// 		todo!()
	/// 	}
	/// }
	/// ```
	fn apply<M: Applicative>(
		&self,
		f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, B>) + 'a>,
		s: S,
	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, T>);
}

impl<'a, S, T, A, B, TF> TraversalFunc<'a, S, T, A, B> for &TF
where
	TF: TraversalFunc<'a, S, T, A, B>,
{
	fn apply<M: Applicative>(
		&self,
		f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, B>) + 'a>,
		s: S,
	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, T>) {
		(**self).apply::<M>(f, s)
	}
}
