//! Profunctors that support traversing structures.
//!
//! A `Wander` profunctor can lift a profunctor to operate on traversable structures.
//! This is the profunctor constraint that characterizes traversals.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				profunctor::*,
				*,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// A trait for traversal functions.
	///
	/// This trait represents a function that can traverse a structure `S` to produce `T`,
	/// given a function `A -> F B` for any [`Applicative`] `F`.
	#[document_type_parameters(
		"The lifetime of the traversal.",
		"The source structure type.",
		"The target structure type.",
		"The focus type within the source.",
		"The replacement type within the target."
	)]
	#[document_parameters("The traversal function itself.")]
	pub trait TraversalFunc<'a, S, T, A, B> {
		/// Apply the traversal to a structure.
		///
		/// This is not object-safe because of the generic parameter `M`.
		/// To use this in an optic, it must be used with a concrete type or a wrapper.
		#[document_signature]
		///
		#[document_type_parameters("The applicative functor brand.")]
		///
		#[document_parameters(
			"The function to apply to each focus.",
			"The source structure to traverse."
		)]
		///
		#[document_returns("The traversed structure wrapped in the applicative.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	classes::{
		/// 		Applicative,
		/// 		profunctor::wander::TraversalFunc,
		/// 	},
		/// 	kinds::*,
		/// };
		///
		/// struct ListTraversal;
		/// impl<'a, A: 'a + Clone> TraversalFunc<'a, Vec<A>, Vec<A>, A, A> for ListTraversal {
		/// 	fn apply<M: Applicative>(
		/// 		&self,
		/// 		f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a>,
		/// 		s: Vec<A>,
		/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
		/// 		s.into_iter().fold(M::pure(vec![]), |acc, a| {
		/// 			M::lift2(
		/// 				|mut v: Vec<A>, x: A| {
		/// 					v.push(x);
		/// 					v
		/// 				},
		/// 				acc,
		/// 				f(a),
		/// 			)
		/// 		})
		/// 	}
		/// }
		///
		/// use fp_library::brands::OptionBrand;
		/// let t = ListTraversal;
		/// let result = t.apply::<OptionBrand>(Box::new(|x: i32| Some(x + 1)), vec![1, 2, 3]);
		/// assert_eq!(result, Some(vec![2, 3, 4]));
		/// ```
		fn apply<M: Applicative>(
			&self,
			f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, B>) + 'a>,
			s: S,
		) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, T>)
		where
			Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, B>): Clone;
	}

	/// Blanket implementation of [`TraversalFunc`] for references to traversal functions.
	#[document_type_parameters(
		"The lifetime of the traversal.",
		"The source structure type.",
		"The target structure type.",
		"The focus type within the source.",
		"The replacement type within the target.",
		"The underlying traversal function type."
	)]
	#[document_parameters("A reference to the traversal function.")]
	impl<'a, S, T, A, B, TF> TraversalFunc<'a, S, T, A, B> for &TF
	where
		TF: TraversalFunc<'a, S, T, A, B>,
	{
		/// Delegates to the underlying traversal function's `apply` method.
		#[document_signature]
		///
		#[document_type_parameters("The applicative functor brand.")]
		///
		#[document_parameters(
			"The function to apply to each focus.",
			"The source structure to traverse."
		)]
		///
		#[document_returns("The traversed structure wrapped in the applicative.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	classes::{
		/// 		Applicative,
		/// 		profunctor::wander::TraversalFunc,
		/// 	},
		/// 	kinds::*,
		/// };
		///
		/// struct ListTraversal;
		/// impl<'a, A: 'a + Clone> TraversalFunc<'a, Vec<A>, Vec<A>, A, A> for ListTraversal {
		/// 	fn apply<M: Applicative>(
		/// 		&self,
		/// 		f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a>,
		/// 		s: Vec<A>,
		/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
		/// 		s.into_iter().fold(M::pure(vec![]), |acc, a| {
		/// 			M::lift2(
		/// 				|mut v: Vec<A>, x: A| {
		/// 					v.push(x);
		/// 					v
		/// 				},
		/// 				acc,
		/// 				f(a),
		/// 			)
		/// 		})
		/// 	}
		/// }
		///
		/// use fp_library::brands::OptionBrand;
		/// let t = ListTraversal;
		/// let t_ref = &t;
		/// let result = t_ref.apply::<OptionBrand>(Box::new(|x: i32| Some(x + 1)), vec![1, 2, 3]);
		/// assert_eq!(result, Some(vec![2, 3, 4]));
		/// ```
		fn apply<M: Applicative>(
			&self,
			f: Box<dyn Fn(A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, B>) + 'a>,
			s: S,
		) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, T>)
		where
			Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, B>): Clone, {
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
			"The target type of the focus."
		)]
		///
		#[document_parameters("The traversal function.", "The profunctor instance.")]
		///
		#[document_returns("A new profunctor instance that operates on the structure.")]
		/// ### `B: Clone` Requirement
		///
		/// The focus replacement type `B` must implement [`Clone`] because traversal internally
		/// uses [`sequence`](crate::classes::traversable::Traversable::sequence), which collects
		/// applicative values via [`lift2`](crate::classes::lift::Lift::lift2). `lift2` requires
		/// cloning the second applicative argument when combining results across multiple foci
		/// (e.g., building `Vec<B>` from individual `M<B>` values). This propagates from
		/// [`TraversalFunc::apply`]'s `M::Of<'a, B>: Clone` bound.
		#[document_examples]
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
		/// 			M::lift2(
		/// 				|mut v: Vec<A>, x: A| {
		/// 					v.push(x);
		/// 					v
		/// 				},
		/// 				acc,
		/// 				f(a),
		/// 			)
		/// 		})
		/// 	}
		/// }
		///
		/// let double = std::rc::Rc::new(|x: i32| x * 2) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let map_all: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	<RcFnBrand as Wander>::wander::<Vec<i32>, Vec<i32>, i32, i32>(ListTraversal, double);
		/// assert_eq!(map_all(vec![1, 2, 3]), vec![2, 4, 6]);
		/// assert_eq!(map_all(vec![]), vec![]);
		/// ```
		fn wander<'a, S: 'a, T: 'a, A: 'a, B: 'a + Clone>(
			traversal: impl TraversalFunc<'a, S, T, A, B> + 'a,
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>);
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
		"The target type of the focus."
	)]
	///
	#[document_parameters("The traversal function.", "The profunctor instance.")]
	///
	#[document_returns("A new profunctor instance that operates on the structure.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	Apply,
	/// 	brands::*,
	/// 	classes::{
	/// 		Applicative,
	/// 		lift::Lift,
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
	/// 			M::lift2(
	/// 				|mut v: Vec<A>, x: A| {
	/// 					v.push(x);
	/// 					v
	/// 				},
	/// 				acc,
	/// 				f(a),
	/// 			)
	/// 		})
	/// 	}
	/// }
	///
	/// let double = std::rc::Rc::new(|x: i32| x * 2) as std::rc::Rc<dyn Fn(i32) -> i32>;
	/// let map_all: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
	/// 	wander::<RcFnBrand, Vec<i32>, Vec<i32>, i32, i32>(ListTraversal, double);
	/// assert_eq!(map_all(vec![1, 2, 3]), vec![2, 4, 6]);
	/// assert_eq!(map_all(vec![]), vec![]);
	/// ```
	pub fn wander<'a, Brand: Wander, S: 'a, T: 'a, A: 'a, B: 'a + Clone>(
		traversal: impl TraversalFunc<'a, S, T, A, B> + 'a,
		pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>) {
		Brand::wander(traversal, pab)
	}
}

pub use inner::*;
