//! Thread-safe by-reference variant of [`FoldableWithIndex`](crate::classes::FoldableWithIndex).
//!
//! **User story:** "I want to fold over a thread-safe memoized value by reference, with access to the index."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::send_ref_foldable_with_index::SendRefFoldableWithIndex,
//! 	types::*,
//! };
//!
//! let lazy = ArcLazy::new(|| 42);
//! let result =
//! 	<LazyBrand<ArcLazyConfig> as SendRefFoldableWithIndex>::send_ref_fold_map_with_index(
//! 		|_, x: &i32| x.to_string(),
//! 		lazy,
//! 	);
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

	/// Thread-safe by-reference folding with index over a structure.
	///
	/// Similar to [`RefFoldableWithIndex`], but closures and elements must be `Send + Sync`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefFoldableWithIndex: SendRefFoldable + WithIndex {
		/// Maps each element to a monoid by reference with index (thread-safe).
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
		/// 	classes::send_ref_foldable_with_index::SendRefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 42);
		/// let result =
		/// 	<LazyBrand<ArcLazyConfig> as SendRefFoldableWithIndex>::send_ref_fold_map_with_index(
		/// 		|_, x: &i32| x.to_string(),
		/// 		lazy,
		/// 	);
		/// assert_eq!(result, "42");
		/// ```
		fn send_ref_fold_map_with_index<'a, A: Send + Sync + 'a, R: Monoid>(
			f: impl Fn(Self::Index, &A) -> R + Send + Sync + 'a,
			fa: Self::Of<'a, A>,
		) -> R;
	}

	/// Maps each element to a monoid by reference with its index (thread-safe).
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefFoldableWithIndex::send_ref_fold_map_with_index`].
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
	/// let lazy = ArcLazy::new(|| 42);
	/// let result = send_ref_fold_map_with_index::<LazyBrand<ArcLazyConfig>, _, _>(
	/// 	|_, x: &i32| x.to_string(),
	/// 	lazy,
	/// );
	/// assert_eq!(result, "42");
	/// ```
	pub fn send_ref_fold_map_with_index<
		'a,
		Brand: SendRefFoldableWithIndex,
		A: Send + Sync + 'a,
		R: Monoid,
	>(
		f: impl Fn(Brand::Index, &A) -> R + Send + Sync + 'a,
		fa: Brand::Of<'a, A>,
	) -> R {
		Brand::send_ref_fold_map_with_index(f, fa)
	}
}

pub use inner::*;
