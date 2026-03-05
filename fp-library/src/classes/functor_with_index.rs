//! A `Functor` with an additional index.

use crate::classes::functor::Functor;

#[fp_macros::document_module]
mod inner {
	use super::*;

	/// A `Functor` with an additional index.
	///
	/// A `FunctorWithIndex` is a `Functor` that also allows you to access the
	/// index of each element when mapping over the structure.
	#[fp_macros::document_type_parameters("The index type.")]
	pub trait FunctorWithIndex<I>: Functor {
		/// Map a function over the structure, providing the index of each element.
		#[fp_macros::document_signature]
		#[fp_macros::document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The type of the result."
		)]
		#[fp_macros::document_parameters(
			"The function to apply to each element and its index.",
			"The structure to map over."
		)]
		#[fp_macros::document_returns("The mapped structure.")]
		fn map_with_index<'a, A: 'a, B: 'a>(
			f: impl Fn(I, A) -> B + 'a,
			fa: Self::Of<'a, A>,
		) -> Self::Of<'a, B>;
	}
}

pub use inner::*;
