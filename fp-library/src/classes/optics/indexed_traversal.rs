//! Trait for indexed traversal functions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A trait for indexed traversal functions.
	///
	/// This trait represents a function that can traverse a structure `S` to produce `T`,
	/// given an indexed function `(I, A) -> F B` for any [`Applicative`] `F`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The index type.",
		"The source type of the structure.",
		"The target type of the structure after an update.",
		"The source type of the focus.",
		"The target type of the focus after an update."
	)]
	#[document_parameters("The indexed traversal function itself.")]
	pub trait IndexedTraversalFunc<'a, I, S, T, A, B: 'a> {
		/// Apply the indexed traversal to a structure.
		#[document_signature]
		///
		#[document_type_parameters("The applicative context.")]
		///
		#[document_parameters("The indexed mapping function.", "The structure to traverse.")]
		///
		#[document_returns("The traversed structure wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	classes::{
		/// 		Applicative,
		/// 		optics::indexed_traversal::IndexedTraversalFunc,
		/// 	},
		/// 	kinds::*,
		/// };
		///
		/// struct IndexedListTraversal;
		/// impl<'a, A: 'a + Clone> IndexedTraversalFunc<'a, usize, Vec<A>, Vec<A>, A, A>
		/// 	for IndexedListTraversal
		/// {
		/// 	fn apply<M: Applicative>(
		/// 		&self,
		/// 		f: impl Fn(usize, A) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, A>) + 'a,
		/// 		s: Vec<A>,
		/// 	) -> Apply!(<M as Kind!( type Of<'b, U: 'b>: 'b; )>::Of<'a, Vec<A>>) {
		/// 		s.into_iter().enumerate().fold(M::pure(vec![]), |acc, (i, a)| {
		/// 			M::lift2(
		/// 				|mut v: Vec<A>, x: A| {
		/// 					v.push(x);
		/// 					v
		/// 				},
		/// 				acc,
		/// 				f(i, a),
		/// 			)
		/// 		})
		/// 	}
		/// }
		///
		/// use fp_library::brands::OptionBrand;
		/// let t = IndexedListTraversal;
		/// let result = t.apply::<OptionBrand>(|i: usize, x: i32| Some(x + i as i32), vec![10, 20, 30]);
		/// assert_eq!(result, Some(vec![10, 21, 32]));
		/// ```
		fn apply<M: Applicative>(
			&self,
			f: impl Fn(I, A) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>) + 'a,
			s: S,
		) -> Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, T>)
		where
			Apply!(<M as Kind!( type Of<'c, U: 'c>: 'c; )>::Of<'a, B>): Clone;
	}
}

pub use inner::*;
