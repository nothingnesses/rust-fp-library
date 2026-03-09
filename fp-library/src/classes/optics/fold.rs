//! Fold optic traits.

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A trait for fold functions.
	///
	/// A `FoldFunc` represents a way to iterate over the focuses of a structure
	/// and combine them using a monoid, without requiring an intermediate collection.
	/// This is the non-allocating equivalent of `Fn(S) -> Vec<A>`.
	#[document_type_parameters(
		"The lifetime of the function.",
		"The source type of the structure.",
		"The type of the focuses."
	)]
	#[document_parameters("The fold function itself.")]
	pub trait FoldFunc<'a, S, A> {
		/// Apply the fold by mapping each focus to a monoid value and combining.
		#[document_signature]
		///
		#[document_type_parameters("The monoid type to fold into.", "The mapping function type.")]
		///
		#[document_parameters("The mapping function.", "The structure to fold.")]
		///
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::optics::*,
		/// 	classes::monoid::Monoid,
		/// 	types::optics::{
		/// 		FoldFunc,
		/// 		IterableFoldFn,
		/// 	},
		/// };
		///
		/// let fold = IterableFoldFn(|v: Vec<i32>| v);
		/// let result = fold.apply::<String, _>(|x| x.to_string(), vec![1, 2, 3]);
		/// assert_eq!(result, "123".to_string());
		/// ```
		fn apply<R: Monoid, F: Fn(A) -> R + 'a>(
			&self,
			f: F,
			s: S,
		) -> R;
	}
}

pub use inner::*;
