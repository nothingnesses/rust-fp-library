//! Profunctors that support traversing structures.
//!
//! A `Wander` profunctor can lift a profunctor to operate on traversable structures.
//! This is the profunctor constraint that characterizes traversals.

use {
	crate::{
		Apply,
		classes::profunctor::{
			Choice,
			Strong,
		},
		kinds::*,
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
};

pub use crate::classes::optics::traversal::TraversalFunc;

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
	/// ### `B: Clone` Requirement
	///
	/// The focus replacement type `B` must implement [`Clone`] because traversal internally
	/// uses [`sequence`](crate::classes::traversable::Traversable::sequence), which collects
	/// applicative values via [`lift2`](crate::classes::lift::Lift::lift2). `lift2` requires
	/// cloning the second applicative argument when combining results across multiple foci
	/// (e.g., building `Vec<B>` from individual `M<B>` values). This propagates from
	/// [`TraversalFunc::apply`]'s `M::Of<'a, B>: Clone` bound.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	Apply,
	/// 	brands::*,
	/// 	classes::{
	/// 		Applicative,
	/// 		lift::Lift,
	/// 		optics::traversal::TraversalFunc,
	/// 		profunctor::*,
	/// 	},
	/// 	kinds::*,
	/// };
	///
	/// #[derive(Clone)]
	/// struct ListTraversal;
	/// impl<'a, A: 'a + Clone> TraversalFunc<'a, Vec<A>, Vec<A>, A, A> for ListTraversal {
	/// 	fn apply<M: Applicative>(
	/// 		&self,
	/// 		f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a>,
	/// 		s: Vec<A>,
	/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
	/// 		s.into_iter().fold(M::pure(vec![]), |acc, a| {
	/// 			M::lift2(|mut v: Vec<A>, x: A| { v.push(x); v }, acc, f(a))
	/// 		})
	/// 	}
	/// }
	///
	/// let double = std::rc::Rc::new(|x: i32| x * 2) as std::rc::Rc<dyn Fn(i32) -> i32>;
	/// let map_all: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
	/// 	<RcFnBrand as Wander>::wander::<Vec<i32>, Vec<i32>, i32, i32, _>(ListTraversal, double);
	/// assert_eq!(map_all(vec![1, 2, 3]), vec![2, 4, 6]);
	/// assert_eq!(map_all(vec![]), vec![]);
	/// ```
	fn wander<'a, S: 'a, T: 'a, A: 'a, B: 'a + Clone, TFunc>(
		traversal: TFunc,
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	where
		TFunc: TraversalFunc<'a, S, T, A, B> + 'a;
}

/// Lift a profunctor to operate on a traversable structure.
///
/// Free function version that dispatches to [the type class' associated function][`Wander::wander`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the profunctor.",
	"The brand of the wander profunctor.",
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
/// 		lift::Lift,
/// 		optics::traversal::TraversalFunc,
/// 		profunctor::*,
/// 	},
/// 	kinds::*,
/// };
///
/// #[derive(Clone)]
/// struct ListTraversal;
/// impl<'a, A: 'a + Clone> TraversalFunc<'a, Vec<A>, Vec<A>, A, A> for ListTraversal {
/// 	fn apply<M: Applicative>(
/// 		&self,
/// 		f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a>,
/// 		s: Vec<A>,
/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
/// 		s.into_iter().fold(M::pure(vec![]), |acc, a| {
/// 			M::lift2(|mut v: Vec<A>, x: A| { v.push(x); v }, acc, f(a))
/// 		})
/// 	}
/// }
///
/// let double = std::rc::Rc::new(|x: i32| x * 2) as std::rc::Rc<dyn Fn(i32) -> i32>;
/// let map_all: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
/// 	wander::<RcFnBrand, Vec<i32>, Vec<i32>, i32, i32, _>(ListTraversal, double);
/// assert_eq!(map_all(vec![1, 2, 3]), vec![2, 4, 6]);
/// assert_eq!(map_all(vec![]), vec![]);
/// ```
pub fn wander<'a, Brand: Wander, S: 'a, T: 'a, A: 'a, B: 'a + Clone, TFunc>(
	traversal: TFunc,
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
where
	TFunc: TraversalFunc<'a, S, T, A, B> + 'a, {
	Brand::wander(traversal, pab)
}
