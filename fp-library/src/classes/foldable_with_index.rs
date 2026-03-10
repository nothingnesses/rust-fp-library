//! A `Foldable` with an additional index.

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A `Foldable` with an additional index.
	///
	/// A `FoldableWithIndex` is a `Foldable` that also allows you to access the
	/// index of each element when folding over the structure.
	#[document_type_parameters("The index type.")]
	pub trait FoldableWithIndex<I>: Foldable {
		/// Map each element of the structure to a monoid, and combine the results,
		/// providing the index of each element.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The structure to fold over."
		)]
		#[document_returns("The combined result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// };
		///
		/// let result = VecBrand::fold_map_with_index(|i, x: i32| format!("{i}:{x}"), vec![10, 20, 30]);
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn fold_map_with_index<'a, A: 'a, R: Monoid>(
			f: impl Fn(I, A) -> R + 'a,
			fa: Self::Of<'a, A>,
		) -> R;
	}
}

pub use inner::*;
