//! Traversal optics for operating on multiple foci.
//!
//! A traversal represents a way to focus on zero or more values in a structure
//! and update them while maintaining the structure.

use {
	super::forget::ForgetBrand,
	crate::{
		Apply,
		brands::FnBrand,
		classes::{
			UnsizedCoercible,
			Wander,
			monoid::Monoid,
			wander::TraversalFunc,
		},
		kinds::*,
		types::optics::base::{
			FoldOptic,
			Optic,
			SetterOptic,
			TraversalOptic,
		},
	},
	fp_macros::document_type_parameters,
	std::marker::PhantomData,
};

/// A polymorphic traversal.
///
/// Matches PureScript's `Traversal s t a b`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The source type of the structure.",
	"The target type of the structure after an update.",
	"The source type of the focus.",
	"The target type of the focus after an update.",
	"The type of the traversal function."
)]
pub struct Traversal<'a, P, S, T, A, B, F>
where
	P: UnsizedCoercible,
	F: TraversalFunc<'a, S, T, A, B> + 'a,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a, {
	/// The traversal function.
	///
	/// In PureScript this is `(forall f. Applicative f => (a -> f b) -> s -> f t)`.
	pub traversal: F,
	pub(crate) _phantom: PhantomData<(&'a (S, T, A, B), P)>,
}

impl<'a, P, S, T, A, B, F> Traversal<'a, P, S, T, A, B, F>
where
	P: UnsizedCoercible,
	F: TraversalFunc<'a, S, T, A, B> + 'a,
{
	/// Creates a new `Traversal` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	Apply,
	/// 	brands::*,
	/// 	classes::{
	/// 		Applicative,
	/// 		wander::TraversalFunc,
	/// 	},
	/// 	kinds::*,
	/// 	types::optics::Traversal,
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
	/// let traversal = Traversal::<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::new(ListTraversal);
	/// ```
	pub fn new(traversal: F) -> Self {
		Traversal {
			traversal,
			_phantom: PhantomData,
		}
	}
}

impl<'a, Q, P, S, T, A, B, F> Optic<'a, Q, S, T, A, B> for Traversal<'a, P, S, T, A, B, F>
where
	Q: Wander,
	P: UnsizedCoercible,
	F: TraversalFunc<'a, S, T, A, B> + Clone + 'a,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	/// Evaluates the traversal with a profunctor.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	Apply,
	/// 	brands::*,
	/// 	classes::{
	/// 		Applicative,
	/// 		Wander,
	/// 		wander::TraversalFunc,
	/// 	},
	/// 	kinds::*,
	/// 	types::optics::{
	/// 		Optic,
	/// 		Traversal,
	/// 	},
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
	/// impl Clone for ListTraversal {
	/// 	fn clone(&self) -> Self {
	/// 		ListTraversal
	/// 	}
	/// }
	///
	/// let traversal = Traversal::<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::new(ListTraversal);
	/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
	/// let _result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
	/// 	Optic::<'_, RcFnBrand, _, _, _, _>::evaluate(&traversal, f);
	/// ```
	fn evaluate(
		&self,
		pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
		Q::wander(self.traversal.clone(), pab)
	}
}

impl<'a, P, S, T, A, B, F> TraversalOptic<'a, S, T, A, B> for Traversal<'a, P, S, T, A, B, F>
where
	P: UnsizedCoercible,
	F: TraversalFunc<'a, S, T, A, B> + Clone + 'a,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	fn evaluate<Q: Wander>(
		&self,
		pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
		Optic::<Q, S, T, A, B>::evaluate(self, pab)
	}
}

impl<'a, P, S, A, F> FoldOptic<'a, S, A> for Traversal<'a, P, S, S, A, A, F>
where
	P: UnsizedCoercible,
	F: TraversalFunc<'a, S, S, A, A> + Clone + 'a,
	S: 'a,
	A: 'a,
{
	fn evaluate<R: 'a + Monoid + 'static>(
		&self,
		pab: Apply!(<ForgetBrand<R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<ForgetBrand<R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
		TraversalOptic::evaluate::<ForgetBrand<R>>(self, pab)
	}
}

impl<'a, Q, P, S, T, A, B, F> SetterOptic<'a, Q, S, T, A, B> for Traversal<'a, P, S, T, A, B, F>
where
	P: UnsizedCoercible,
	Q: UnsizedCoercible,
	F: TraversalFunc<'a, S, T, A, B> + Clone + 'a,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	fn evaluate(
		&self,
		pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
		TraversalOptic::evaluate::<FnBrand<Q>>(self, pab)
	}
}
