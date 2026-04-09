//! Parallel by-reference foldable with index.
//!
//! **User story:** "I want to fold over a collection by reference with index in parallel."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::VecBrand,
//! 	classes::par_ref_foldable_with_index::ParRefFoldableWithIndex,
//! };
//!
//! let v = vec![10, 20, 30];
//! let result = VecBrand::par_ref_fold_map_with_index(|i, x: &i32| format!("{}:{}", i, x), v);
//! assert_eq!(result, "0:101:202:30");
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

	/// Parallel by-reference folding with index.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait ParRefFoldableWithIndex: ParRefFoldable + RefFoldableWithIndex {
		/// Maps each element by reference with index to a monoid and combines them in parallel.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The element type.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to each element's index and reference. Must be `Send + Sync`.",
			"The structure to fold."
		)]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_foldable_with_index::ParRefFoldableWithIndex,
		/// };
		///
		/// let v = vec![10, 20, 30];
		/// let result = VecBrand::par_ref_fold_map_with_index(|i, x: &i32| format!("{}:{}", i, x), v);
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn par_ref_fold_map_with_index<'a, A: Send + Sync + 'a, M: Monoid + Send + 'a>(
			f: impl Fn(Self::Index, &A) -> M + Send + Sync + 'a,
			fa: &Self::Of<'a, A>,
		) -> M;
	}

	/// Maps each element by reference with index to a monoid and combines them in parallel.
	///
	/// Free function version that dispatches to [the type class' associated function][`ParRefFoldableWithIndex::par_ref_fold_map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The element type.",
		"The monoid type."
	)]
	#[document_parameters(
		"The function to apply to each element's index and reference. Must be `Send + Sync`.",
		"The structure to fold."
	)]
	#[document_returns("The combined monoid value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![10, 20, 30];
	/// let result =
	/// 	par_ref_fold_map_with_index::<VecBrand, _, _>(|i, x: &i32| format!("{}:{}", i, x), v);
	/// assert_eq!(result, "0:101:202:30");
	/// ```
	pub fn par_ref_fold_map_with_index<
		'a,
		Brand: ParRefFoldableWithIndex,
		A: Send + Sync + 'a,
		M: Monoid + Send + 'a,
	>(
		f: impl Fn(Brand::Index, &A) -> M + Send + Sync + 'a,
		fa: &Brand::Of<'a, A>,
	) -> M {
		Brand::par_ref_fold_map_with_index(f, fa)
	}
}

pub use inner::*;
