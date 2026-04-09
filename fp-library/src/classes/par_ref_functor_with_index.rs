//! Parallel by-reference functor mapping with index.
//!
//! **User story:** "I want to map over a collection by reference with index in parallel."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::VecBrand,
//! 	classes::par_ref_functor_with_index::ParRefFunctorWithIndex,
//! };
//!
//! let v = vec![10, 20, 30];
//! let result = VecBrand::par_ref_map_with_index(|i, x: &i32| format!("{}:{}", i, x), &v);
//! assert_eq!(result, vec!["0:10", "1:20", "2:30"]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// Parallel by-reference functor mapping with index.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait ParRefFunctorWithIndex: ParRefFunctor + RefFunctorWithIndex {
		/// Maps a function over the structure by reference with index in parallel.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		#[document_parameters(
			"The function to apply to each element's index and reference. Must be `Send + Sync`.",
			"The structure to map over."
		)]
		#[document_returns("A new structure containing the mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_functor_with_index::ParRefFunctorWithIndex,
		/// };
		///
		/// let v = vec![10, 20, 30];
		/// let result = VecBrand::par_ref_map_with_index(|i, x: &i32| format!("{}:{}", i, x), &v);
		/// assert_eq!(result, vec!["0:10", "1:20", "2:30"]);
		/// ```
		fn par_ref_map_with_index<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(Self::Index, &A) -> B + Send + Sync + 'a,
			fa: &Self::Of<'a, A>,
		) -> Self::Of<'a, B>;
	}

	/// Maps a function over a structure by reference with index in parallel.
	///
	/// Free function version that dispatches to [the type class' associated function][`ParRefFunctorWithIndex::par_ref_map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The input element type.",
		"The output element type."
	)]
	#[document_parameters(
		"The function to apply to each element's index and reference. Must be `Send + Sync`.",
		"The structure to map over."
	)]
	#[document_returns("A new structure containing the mapped elements.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![10, 20, 30];
	/// let result = par_ref_map_with_index::<VecBrand, _, _>(|i, x: &i32| format!("{}:{}", i, x), &v);
	/// assert_eq!(result, vec!["0:10", "1:20", "2:30"]);
	/// ```
	pub fn par_ref_map_with_index<
		'a,
		Brand: ParRefFunctorWithIndex,
		A: Send + Sync + 'a,
		B: Send + 'a,
	>(
		f: impl Fn(Brand::Index, &A) -> B + Send + Sync + 'a,
		fa: &Brand::Of<'a, A>,
	) -> Brand::Of<'a, B> {
		Brand::par_ref_map_with_index(f, fa)
	}
}

pub use inner::*;
