//! A `Foldable` with an additional index.

use crate::classes::{
	foldable::Foldable,
	monoid::Monoid,
};

#[fp_macros::document_module]
mod inner {
	use super::*;

	/// A `Foldable` with an additional index.
	///
	/// A `FoldableWithIndex` is a `Foldable` that also allows you to access the
	/// index of each element when folding over the structure.
	#[fp_macros::document_type_parameters("The index type.")]
	pub trait FoldableWithIndex<I>: Foldable {
		/// Map each element of the structure to a monoid, and combine the results,
		/// providing the index of each element.
		#[fp_macros::document_signature]
		#[fp_macros::document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The monoid type."
		)]
		#[fp_macros::document_parameters(
			"The function to apply to each element and its index.",
			"The structure to fold over."
		)]
		#[fp_macros::document_returns("The combined result.")]
		fn fold_map_with_index<'a, A: 'a, R: Monoid>(
			f: impl Fn(I, A) -> R + 'a,
			fa: Self::Of<'a, A>,
		) -> R;
	}
}

pub use inner::*;
