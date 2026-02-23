//! Profunctors that support traversing structures.
//!
//! A `Wander` profunctor can lift a profunctor to operate on traversable structures.
//! This is the profunctor constraint that characterizes traversals.

use {
	crate::{
		Apply,
		classes::{Choice, Strong, applicative::Applicative},
		kinds::*,
	},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
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
	/// 		wander::TraversalFunc,
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

/// A type class for wandering profunctors.
///
/// A `Wander` profunctor can lift a profunctor to operate on traversable structures.
///
/// ### Hierarchy Unification
///
/// This trait inherits from [`Strong`] and [`Choice`].
pub trait Wander: Strong + Choice {
	/// Lift a profunctor to operate on a traversable structure.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the profunctor.",
		"The source type of the structure.",
		"The target type of the structure.",
		"The source type of the focus.",
		"The target type of the focus.",
		"The type of the traversal function."
	)]
	///
	#[document_parameters("The traversal function.", "The profunctor instance.")]
	///
	/// ### Returns
	///
	/// A new profunctor instance that operates on the structure.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	Apply,
	/// 	brands::*,
	/// 	classes::{
	/// 		Applicative,
	/// 		Traversable,
	/// 		Wander,
	/// 		wander::TraversalFunc,
	/// 	},
	/// 	kinds::*,
	/// };
	///
	/// struct ListTraversal;
	/// impl<'a, A: 'a> TraversalFunc<'a, Vec<A>, Vec<A>, A, A> for ListTraversal {
	/// 	fn apply<M: Applicative>(
	/// 		&self,
	/// 		_f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a>,
	/// 		_s: Vec<A>,
	/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
	/// 		unreachable!()
	/// 	}
	/// }
	///
	/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
	/// let _g = <RcFnBrand as Wander>::wander::<Vec<i32>, Vec<i32>, i32, i32, _>(ListTraversal, f);
	/// ```
	fn wander<'a, S: 'a, T: 'a, A: 'a, B: 'a, TFunc>(
		traversal: TFunc,
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	where
		TFunc: TraversalFunc<'a, S, T, A, B> + 'a;
}
