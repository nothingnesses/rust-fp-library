//! Traversal optics for operating on multiple foci.
//!
//! A traversal represents a way to focus on zero or more values in a structure
//! and update them while maintaining the structure.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FnBrand,
			classes::{
				UnsizedCoercible,
				monoid::Monoid,
				optics::*,
				profunctor::Wander,
			},
			kinds::*,
			types::optics::ForgetBrand,
		},
		fp_macros::{
			document_parameters,
			document_type_parameters,
			document_return,
		},
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

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The type of the traversal function."
	)]
	impl<'a, P, S, T, A, B, F> Traversal<'a, P, S, T, A, B, F>
	where
		P: UnsizedCoercible,
		F: TraversalFunc<'a, S, T, A, B> + 'a,
	{
		/// Creates a new `Traversal` instance.
		#[document_signature]
		///
		#[document_parameters("The traversal function.")]
		///
		#[document_return("A new instance of the type.")]
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
		/// 		pointed::Pointed,
		/// 	},
		/// 	kinds::*,
		/// 	types::optics::Traversal,
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
		/// let traversal = Traversal::<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::new(ListTraversal);
		/// assert_eq!(
		/// 	traversal.traversal.apply::<OptionBrand>(Box::new(|x| Some(x + 1)), vec![1, 2]),
		/// 	Some(vec![2, 3])
		/// );
		/// ```
		pub fn new(traversal: F) -> Self {
			Traversal {
				traversal,
				_phantom: PhantomData,
			}
		}
	}

	/// A monomorphic traversal.
	///
	/// Matches PureScript's `Traversal' s a`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus.",
		"The type of the traversal function."
	)]
	pub struct TraversalPrime<'a, P, S, A, F>
	where
		P: UnsizedCoercible,
		F: TraversalFunc<'a, S, S, A, A> + 'a,
		S: 'a,
		A: 'a, {
		/// The traversal function.
		pub traversal: F,
		pub(crate) _phantom: PhantomData<(&'a (S, A), P)>,
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The type of the structure.",
		"The type of the focus.",
		"The type of the traversal function."
	)]
	impl<'a, P, S, A, F> TraversalPrime<'a, P, S, A, F>
	where
		P: UnsizedCoercible,
		F: TraversalFunc<'a, S, S, A, A> + 'a,
	{
		/// Creates a new `TraversalPrime` instance.
		#[document_signature]
		///
		#[document_parameters("The traversal function.")]
		///
		#[document_return("A new instance of the type.")]
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
		/// 		pointed::Pointed,
		/// 	},
		/// 	kinds::*,
		/// 	types::optics::TraversalPrime,
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
		/// let traversal = TraversalPrime::<'_, RcBrand, Vec<i32>, i32, _>::new(ListTraversal);
		/// assert_eq!(
		/// 	traversal.traversal.apply::<OptionBrand>(Box::new(|x| Some(x + 1)), vec![1, 2]),
		/// 	Some(vec![2, 3])
		/// );
		/// ```
		pub fn new(traversal: F) -> Self {
			TraversalPrime {
				traversal,
				_phantom: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The profunctor type.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The type of the traversal function."
	)]
	#[document_parameters("The traversal instance.")]
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
		#[document_signature]
		///
		#[document_parameters("The profunctor value to transform.")]
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
		/// 		optics::{
		/// 			traversal::TraversalFunc,
		/// 			*,
		/// 		},
		/// 		pointed::Pointed,
		/// 		profunctor::*,
		/// 	},
		/// 	kinds::*,
		/// 	types::optics::Traversal,
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
		/// let traversal = Traversal::<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::new(ListTraversal);
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	Optic::<'_, RcFnBrand, _, _, _, _>::evaluate(&traversal, f);
		/// assert_eq!(result(vec![1, 2]), vec![2, 3]);
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Q::wander(self.traversal.clone(), pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The type of the traversal function."
	)]
	#[document_parameters("The traversal instance.")]
	impl<'a, P, S, T, A, B, F> TraversalOptic<'a, S, T, A, B> for Traversal<'a, P, S, T, A, B, F>
	where
		P: UnsizedCoercible,
		F: TraversalFunc<'a, S, T, A, B> + Clone + 'a,
		S: 'a,
		T: 'a,
		A: 'a,
		B: 'a,
	{
		#[document_signature]
		#[document_type_parameters("The profunctor type.")]
		#[document_parameters("The profunctor value to transform.")]
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
		/// 		optics::{
		/// 			traversal::TraversalFunc,
		/// 			*,
		/// 		},
		/// 		pointed::Pointed,
		/// 		profunctor::*,
		/// 	},
		/// 	kinds::*,
		/// 	types::optics::Traversal,
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
		/// let traversal = Traversal::<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::new(ListTraversal);
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	TraversalOptic::evaluate::<RcFnBrand>(&traversal, f);
		/// assert_eq!(result(vec![1, 2]), vec![2, 3]);
		/// ```
		fn evaluate<Q: Wander>(
			&self,
			pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			Optic::<Q, S, T, A, B>::evaluate(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type.",
		"The source type of the structure.",
		"The source type of the focus.",
		"The type of the traversal function."
	)]
	#[document_parameters("The traversal instance.")]
	impl<'a, P, S, A, F> FoldOptic<'a, S, A> for Traversal<'a, P, S, S, A, A, F>
	where
		P: UnsizedCoercible,
		F: TraversalFunc<'a, S, S, A, A> + Clone + 'a,
		S: 'a,
		A: 'a,
	{
		#[document_signature]
		#[document_type_parameters(
			"The monoid type.",
			"The reference-counted pointer type for the Forget brand."
		)]
		#[document_parameters("The profunctor value to transform.")]
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
		/// 		optics::{
		/// 			traversal::TraversalFunc,
		/// 			*,
		/// 		},
		/// 		pointed::Pointed,
		/// 		profunctor::*,
		/// 	},
		/// 	kinds::*,
		/// 	types::optics::{
		/// 		Forget,
		/// 		Traversal,
		/// 	},
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
		/// let traversal = Traversal::<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::new(ListTraversal);
		/// let f = Forget::<RcBrand, String, i32, i32>::new(|x: i32| x.to_string());
		/// let result = FoldOptic::evaluate(&traversal, f);
		/// assert_eq!(result.run(vec![1, 2]), "12".to_string());
		/// ```
		fn evaluate<R: 'a + Monoid + 'static, Q: UnsizedCoercible + 'static>(
			&self,
			pab: Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
		) -> Apply!(<ForgetBrand<Q, R> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>)
		{
			TraversalOptic::evaluate::<ForgetBrand<Q, R>>(self, pab)
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The reference-counted pointer type for the setter brand.",
		"The reference-counted pointer type for the traversal.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update.",
		"The type of the traversal function."
	)]
	#[document_parameters("The traversal instance.")]
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
		#[document_signature]
		#[document_parameters("The profunctor value to transform.")]
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
		/// 		optics::{
		/// 			traversal::TraversalFunc,
		/// 			*,
		/// 		},
		/// 		pointed::Pointed,
		/// 		profunctor::*,
		/// 	},
		/// 	kinds::*,
		/// 	types::optics::Traversal,
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
		/// let traversal = Traversal::<'_, RcBrand, Vec<i32>, Vec<i32>, i32, i32, _>::new(ListTraversal);
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let result: std::rc::Rc<dyn Fn(Vec<i32>) -> Vec<i32>> =
		/// 	SetterOptic::<RcBrand, _, _, _, _>::evaluate(&traversal, f);
		/// assert_eq!(result(vec![1, 2]), vec![2, 3]);
		/// ```
		fn evaluate(
			&self,
			pab: Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
		) -> Apply!(<FnBrand<Q> as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
			TraversalOptic::evaluate::<FnBrand<Q>>(self, pab)
		}
	}
}
pub use inner::*;
