//! By-reference variant of [`FoldableWithIndex`](crate::classes::FoldableWithIndex).
//!
//! **User story:** "I want to fold over a memoized value by reference, with access to the index."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::ref_foldable_with_index::RefFoldableWithIndex,
//! 	types::*,
//! };
//!
//! let lazy = RcLazy::new(|| 42);
//! let result = <LazyBrand<RcLazyConfig> as RefFoldableWithIndex>::ref_fold_map_with_index(
//! 	|_, x: &i32| x.to_string(),
//! 	lazy,
//! );
//! assert_eq!(result, "42");
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

	/// By-reference folding with index over a structure.
	///
	/// Similar to [`FoldableWithIndex`], but the closure receives `&A` instead of `A`.
	/// This is the honest interface for memoized types like [`Lazy`](crate::types::Lazy)
	/// that internally hold a cached `&A`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefFoldableWithIndex: RefFoldable + WithIndex {
		/// Maps each element of the structure to a monoid by reference,
		/// providing the index, and combines the results.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to each element's index and reference.",
			"The structure to fold over."
		)]
		#[document_returns("The combined result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_foldable_with_index::RefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 42);
		/// let result = <LazyBrand<RcLazyConfig> as RefFoldableWithIndex>::ref_fold_map_with_index(
		/// 	|_, x: &i32| x.to_string(),
		/// 	lazy,
		/// );
		/// assert_eq!(result, "42");
		/// ```
		fn ref_fold_map_with_index<'a, A: 'a, R: Monoid>(
			f: impl Fn(Self::Index, &A) -> R + 'a,
			fa: Self::Of<'a, A>,
		) -> R;
	}

	/// Maps each element to a monoid by reference with its index and combines the results.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFoldableWithIndex::ref_fold_map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the structure.",
		"The type of the elements.",
		"The monoid type."
	)]
	#[document_parameters(
		"The function to apply to each element's index and reference.",
		"The structure to fold over."
	)]
	#[document_returns("The combined result.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let lazy = RcLazy::new(|| 42);
	/// let result =
	/// 	ref_fold_map_with_index::<LazyBrand<RcLazyConfig>, _, _>(|_, x: &i32| x.to_string(), lazy);
	/// assert_eq!(result, "42");
	/// ```
	pub fn ref_fold_map_with_index<'a, Brand: RefFoldableWithIndex, A: 'a, R: Monoid>(
		f: impl Fn(Brand::Index, &A) -> R + 'a,
		fa: Brand::Of<'a, A>,
	) -> R {
		Brand::ref_fold_map_with_index(f, fa)
	}
}

pub use inner::*;
