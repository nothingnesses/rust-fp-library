//! Parallel by-reference filterable.
//!
//! **User story:** "I want to filter and transform a collection by reference in parallel."
//!
//! The closure receives `&A` instead of consuming `A`, which avoids cloning elements
//! that get filtered out. Elements must be `Send + Sync` for rayon's `par_iter()`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::VecBrand,
//! 	classes::par_ref_filterable::ParRefFilterable,
//! };
//!
//! let v = vec![1, 2, 3, 4, 5];
//! let result =
//! 	VecBrand::par_ref_filter_map(|x: &i32| if *x > 2 { Some(x.to_string()) } else { None }, v);
//! assert_eq!(result, vec!["3", "4", "5"]);
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

	/// Parallel by-reference filtering and mapping.
	///
	/// Combines the by-reference access of [`RefFilterable`](crate::classes::RefFilterable)
	/// with the parallelism of [`ParFilterable`](crate::classes::ParFilterable).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait ParRefFilterable: ParRefFunctor + ParCompactable {
		/// Filters and maps elements by reference in parallel.
		///
		/// The closure receives `&A` and returns `Option<B>`. Elements for which
		/// the closure returns `None` are discarded. Only surviving elements
		/// are transformed.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		#[document_parameters(
			"The function to apply to each element reference. Must be `Send + Sync`.",
			"The structure to filter and map."
		)]
		#[document_returns("A new structure containing only the transformed elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_filterable::ParRefFilterable,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result =
		/// 	VecBrand::par_ref_filter_map(|x: &i32| if *x % 2 == 0 { Some(*x * 10) } else { None }, v);
		/// assert_eq!(result, vec![20, 40]);
		/// ```
		fn par_ref_filter_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(&A) -> Option<B> + Send + Sync + 'a,
			fa: Self::Of<'a, A>,
		) -> Self::Of<'a, B> {
			Self::par_compact(Self::par_ref_map(f, fa))
		}

		/// Filters elements by reference in parallel.
		///
		/// The predicate receives `&A`. Only elements that pass are kept.
		/// Unlike `par_filter`, this only clones elements that survive the filter.
		#[document_signature]
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		#[document_parameters(
			"The predicate to apply to each element reference. Must be `Send + Sync`.",
			"The structure to filter."
		)]
		#[document_returns("A new structure containing only the elements that pass the predicate.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_filterable::ParRefFilterable,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result = VecBrand::par_ref_filter(|x: &i32| *x > 3, v);
		/// assert_eq!(result, vec![4, 5]);
		/// ```
		fn par_ref_filter<'a, A: Send + Sync + Clone + 'a>(
			f: impl Fn(&A) -> bool + Send + Sync + 'a,
			fa: Self::Of<'a, A>,
		) -> Self::Of<'a, A> {
			Self::par_ref_filter_map(move |a| if f(a) { Some(a.clone()) } else { None }, fa)
		}
	}

	/// Filters and maps elements by reference in parallel.
	///
	/// Free function version that dispatches to [the type class' associated function][`ParRefFilterable::par_ref_filter_map`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The input element type.",
		"The output element type."
	)]
	#[document_parameters(
		"The function to apply to each element reference. Must be `Send + Sync`.",
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
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result = par_ref_filter_map::<VecBrand, _, _>(
	/// 	|x: &i32| if *x % 2 == 0 { Some(*x * 10) } else { None },
	/// 	v,
	/// );
	/// assert_eq!(result, vec![20, 40]);
	/// ```
	pub fn par_ref_filter_map<'a, Brand: ParRefFilterable, A: Send + Sync + 'a, B: Send + 'a>(
		f: impl Fn(&A) -> Option<B> + Send + Sync + 'a,
		fa: Brand::Of<'a, A>,
	) -> Brand::Of<'a, B> {
		Brand::par_ref_filter_map(f, fa)
	}

	/// Filters elements by reference in parallel.
	///
	/// Free function version that dispatches to [the type class' associated function][`ParRefFilterable::par_ref_filter`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The element type."
	)]
	#[document_parameters(
		"The predicate to apply to each element reference. Must be `Send + Sync`.",
		"The structure to filter."
	)]
	#[document_returns("A new structure containing only the elements that pass the predicate.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result = par_ref_filter::<VecBrand, _>(|x: &i32| *x > 3, v);
	/// assert_eq!(result, vec![4, 5]);
	/// ```
	pub fn par_ref_filter<'a, Brand: ParRefFilterable, A: Send + Sync + Clone + 'a>(
		f: impl Fn(&A) -> bool + Send + Sync + 'a,
		fa: Brand::Of<'a, A>,
	) -> Brand::Of<'a, A> {
		Brand::par_ref_filter(f, fa)
	}
}

pub use inner::*;
