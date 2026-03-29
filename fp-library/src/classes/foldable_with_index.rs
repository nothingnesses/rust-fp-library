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
	/// index of each element when folding over the structure. The index type is
	/// uniquely determined by the implementing brand via the [`WithIndex`] supertype.
	///
	/// ### Laws
	///
	/// `FoldableWithIndex` instances must be compatible with their `Foldable` instance:
	/// * Compatibility with Foldable: `fold_map(f, fa) = fold_map_with_index(|_, a| f(a), fa)`.
	#[document_examples]
	///
	/// FoldableWithIndex laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::foldable_with_index::FoldableWithIndex,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3];
	/// let f = |a: i32| a.to_string();
	///
	/// // Compatibility with Foldable:
	/// // fold_map(f, fa) = fold_map_with_index(|_, a| f(a), fa)
	/// assert_eq!(
	/// 	fold_map::<RcFnBrand, VecBrand, _, _>(f, xs.clone()),
	/// 	VecBrand::fold_map_with_index(|_, a| f(a), xs),
	/// );
	/// ```
	pub trait FoldableWithIndex: Foldable + WithIndex {
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
		fn fold_map_with_index<'a, A: 'a + Clone, R: Monoid>(
			f: impl Fn(Self::Index, A) -> R + 'a,
			fa: Self::Of<'a, A>,
		) -> R;
	}
}

pub use inner::*;
