//! Parallel by-reference filterable with index.
//!
//! **User story:** "I want to filter and transform a collection by reference with index in parallel."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::VecBrand,
//! 	classes::par_ref_filterable_with_index::ParRefFilterableWithIndex,
//! };
//!
//! let v = vec![10, 20, 30, 40, 50];
//! let result = VecBrand::par_ref_filter_map_with_index(
//! 	|i, x: &i32| if i % 2 == 0 { Some(x.to_string()) } else { None },
//! 	v,
//! );
//! assert_eq!(result, vec!["10", "30", "50"]);
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

	/// Parallel by-reference filtering and mapping with index.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait ParRefFilterableWithIndex: ParRefFilterable + RefFilterableWithIndex {
		/// Filters and maps elements by reference with index in parallel.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		#[document_parameters(
			"The function to apply to each element's index and reference. Must be `Send + Sync`.",
			"The structure to filter and map."
		)]
		#[document_returns("A new structure containing only the transformed elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_filterable_with_index::ParRefFilterableWithIndex,
		/// };
		///
		/// let v = vec![10, 20, 30, 40, 50];
		/// let result = VecBrand::par_ref_filter_map_with_index(
		/// 	|i, x: &i32| if i % 2 == 0 { Some(x.to_string()) } else { None },
		/// 	v,
		/// );
		/// assert_eq!(result, vec!["10", "30", "50"]);
		/// ```
		fn par_ref_filter_map_with_index<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(Self::Index, &A) -> Option<B> + Send + Sync + 'a,
			fa: &Self::Of<'a, A>,
		) -> Self::Of<'a, B>;

		/// Filters elements by reference with index in parallel.
		#[document_signature]
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		#[document_parameters(
			"The predicate to apply to each element's index and reference. Must be `Send + Sync`.",
			"The structure to filter."
		)]
		#[document_returns("A new structure containing only the elements that pass.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_filterable_with_index::ParRefFilterableWithIndex,
		/// };
		///
		/// let v = vec![10, 20, 30, 40, 50];
		/// let result = VecBrand::par_ref_filter_with_index(|i, _: &i32| i >= 2, v);
		/// assert_eq!(result, vec![30, 40, 50]);
		/// ```
		fn par_ref_filter_with_index<'a, A: Send + Sync + Clone + 'a>(
			f: impl Fn(Self::Index, &A) -> bool + Send + Sync + 'a,
			fa: &Self::Of<'a, A>,
		) -> Self::Of<'a, A> {
			Self::par_ref_filter_map_with_index(
				move |i, a| {
					if f(i, a) { Some(a.clone()) } else { None }
				},
				fa,
			)
		}
	}

	/// Filters and maps elements by reference with index in parallel.
	///
	/// Free function version that dispatches to [the type class' associated function][`ParRefFilterableWithIndex::par_ref_filter_map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The input element type.",
		"The output element type."
	)]
	#[document_parameters(
		"The function to apply to each element's index and reference. Must be `Send + Sync`.",
		"The structure to filter and map."
	)]
	#[document_returns("A new structure containing only the transformed elements.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![10, 20, 30, 40, 50];
	/// let result = par_ref_filter_map_with_index::<VecBrand, _, _>(
	/// 	|i, x: &i32| if i % 2 == 0 { Some(x.to_string()) } else { None },
	/// 	v,
	/// );
	/// assert_eq!(result, vec!["10", "30", "50"]);
	/// ```
	pub fn par_ref_filter_map_with_index<
		'a,
		Brand: ParRefFilterableWithIndex,
		A: Send + Sync + 'a,
		B: Send + 'a,
	>(
		f: impl Fn(Brand::Index, &A) -> Option<B> + Send + Sync + 'a,
		fa: &Brand::Of<'a, A>,
	) -> Brand::Of<'a, B> {
		Brand::par_ref_filter_map_with_index(f, fa)
	}

	/// Filters elements by reference with index in parallel.
	///
	/// Free function version that dispatches to [the type class' associated function][`ParRefFilterableWithIndex::par_ref_filter_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The element type."
	)]
	#[document_parameters(
		"The predicate to apply to each element's index and reference. Must be `Send + Sync`.",
		"The structure to filter."
	)]
	#[document_returns("A new structure containing only the elements that pass.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![10, 20, 30, 40, 50];
	/// let result = par_ref_filter_with_index::<VecBrand, _>(|i, _: &i32| i >= 2, v);
	/// assert_eq!(result, vec![30, 40, 50]);
	/// ```
	pub fn par_ref_filter_with_index<
		'a,
		Brand: ParRefFilterableWithIndex,
		A: Send + Sync + Clone + 'a,
	>(
		f: impl Fn(Brand::Index, &A) -> bool + Send + Sync + 'a,
		fa: &Brand::Of<'a, A>,
	) -> Brand::Of<'a, A> {
		Brand::par_ref_filter_with_index(f, fa)
	}
}

pub use inner::*;
