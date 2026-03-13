//! A `ParFoldable` with an additional index.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A `ParFoldable` with an additional index.
	///
	/// `ParFoldableWithIndex` is the parallel counterpart to
	/// [`FoldableWithIndex`](crate::classes::FoldableWithIndex). Implementors define
	/// [`par_fold_map_with_index`][ParFoldableWithIndex::par_fold_map_with_index] directly.
	///
	/// ### Laws
	///
	/// `ParFoldableWithIndex` instances must be compatible with their `ParFoldable` instance:
	/// * Compatibility with `ParFoldable`:
	///   `par_fold_map(f, fa) = par_fold_map_with_index(|_, a| f(a), fa)`.
	///
	/// ### Thread Safety
	///
	/// The index type `I` must satisfy `I: Send + Sync + Copy` when calling
	/// [`par_fold_map_with_index`][ParFoldableWithIndex::par_fold_map_with_index]. These bounds
	/// apply even when the `rayon` feature is disabled.
	#[document_type_parameters("The index type.")]
	#[document_examples]
	///
	/// ParFoldableWithIndex laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::par_foldable_with_index::ParFoldableWithIndex,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3];
	/// let f = |a: i32| a.to_string();
	///
	/// // Compatibility with ParFoldable:
	/// // par_fold_map(f, fa) = par_fold_map_with_index(|_, a| f(a), fa)
	/// assert_eq!(
	/// 	par_fold_map::<VecBrand, _, _>(f, xs.clone()),
	/// 	VecBrand::par_fold_map_with_index(|_, a| f(a), xs),
	/// );
	/// ```
	pub trait ParFoldableWithIndex<I>: ParFoldable + FoldableWithIndex<I> {
		/// Maps each element and its index to a [`Monoid`] value and combines them in parallel.
		///
		/// When the `rayon` feature is enabled, the mapping and reduction are done across multiple
		/// threads. Otherwise falls back to a sequential indexed fold.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to each element and its index. Must be `Send + Sync`.",
			"The structure to fold over."
		)]
		#[document_returns("The combined result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_foldable_with_index::ParFoldableWithIndex,
		/// };
		///
		/// let result =
		/// 	VecBrand::par_fold_map_with_index(|i, x: i32| format!("{i}:{x}"), vec![10, 20, 30]);
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn par_fold_map_with_index<'a, A: 'a + Send, M: Monoid + Send + 'a>(
			f: impl Fn(I, A) -> M + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			I: Send + Sync + Copy + 'a;
	}

	/// Maps each element and its index to a [`Monoid`] value and combines them in parallel.
	///
	/// Free function version that dispatches to
	/// [`ParFoldableWithIndex::par_fold_map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the structure.",
		"The index type.",
		"The type of the elements.",
		"The monoid type."
	)]
	#[document_parameters(
		"The function to apply to each element and its index. Must be `Send + Sync`.",
		"The structure to fold over."
	)]
	#[document_returns("The combined result.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result: String = par_fold_map_with_index::<VecBrand, usize, _, _>(
	/// 	|i, x: i32| format!("{i}:{x}"),
	/// 	vec![10, 20, 30],
	/// );
	/// assert_eq!(result, "0:101:202:30");
	/// ```
	pub fn par_fold_map_with_index<'a, Brand, I, A: 'a + Send, M: Monoid + Send + 'a>(
		f: impl Fn(I, A) -> M + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		Brand: ParFoldableWithIndex<I>,
		I: Send + Sync + Copy + 'a, {
		Brand::par_fold_map_with_index(f, fa)
	}
}

pub use inner::*;
