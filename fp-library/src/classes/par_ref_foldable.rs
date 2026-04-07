//! Parallel by-reference foldable.
//!
//! **User story:** "I want to fold over a collection by reference in parallel."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::VecBrand,
//! 	classes::par_ref_foldable::ParRefFoldable,
//! };
//!
//! let v = vec![1, 2, 3];
//! let result = VecBrand::par_ref_fold_map(|x: &i32| x.to_string(), v);
//! assert_eq!(result, "123");
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

	/// Parallel by-reference folding over a structure.
	///
	/// Maps each element by reference to a monoid using a `Send + Sync` function
	/// and combines the results. When the `rayon` feature is enabled, elements are
	/// processed across multiple threads.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait ParRefFoldable: RefFoldable {
		/// Maps each element by reference to a monoid and combines them in parallel.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The element type.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to map each element reference to a monoid. Must be `Send + Sync`.",
			"The structure to fold."
		)]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_foldable::ParRefFoldable,
		/// };
		///
		/// let v = vec![1, 2, 3];
		/// let result = VecBrand::par_ref_fold_map(|x: &i32| x.to_string(), v);
		/// assert_eq!(result, "123");
		/// ```
		fn par_ref_fold_map<'a, A: Send + Sync + 'a, M: Monoid + Send + 'a>(
			f: impl Fn(&A) -> M + Send + Sync + 'a,
			fa: Self::Of<'a, A>,
		) -> M;
	}

	/// Maps each element by reference to a monoid and combines them in parallel.
	///
	/// Free function version that dispatches to [the type class' associated function][`ParRefFoldable::par_ref_fold_map`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The element type.",
		"The monoid type."
	)]
	#[document_parameters(
		"The function to map each element reference to a monoid. Must be `Send + Sync`.",
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
	/// let v = vec![1, 2, 3];
	/// let result = par_ref_fold_map::<VecBrand, _, _>(|x: &i32| x.to_string(), v);
	/// assert_eq!(result, "123");
	/// ```
	pub fn par_ref_fold_map<
		'a,
		Brand: ParRefFoldable,
		A: Send + Sync + 'a,
		M: Monoid + Send + 'a,
	>(
		f: impl Fn(&A) -> M + Send + Sync + 'a,
		fa: Brand::Of<'a, A>,
	) -> M {
		Brand::par_ref_fold_map(f, fa)
	}
}

pub use inner::*;
