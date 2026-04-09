//! Parallel by-reference functor mapping.
//!
//! **User story:** "I want to map over a collection by reference in parallel."
//!
//! This trait combines the by-reference access of [`RefFunctor`](crate::classes::RefFunctor) with
//! the parallelism of [`ParFunctor`](crate::classes::ParFunctor). The closure receives `&A`
//! (no consumption of elements) and must be `Send + Sync`. Elements must be `Send + Sync`
//! for rayon's `par_iter()`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::VecBrand,
//! 	classes::par_ref_functor::ParRefFunctor,
//! };
//!
//! let v = vec![1, 2, 3];
//! let result = VecBrand::par_ref_map(|x: &i32| x.to_string(), &v);
//! assert_eq!(result, vec!["1", "2", "3"]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// Parallel by-reference functor mapping.
	///
	/// Maps a `Send + Sync` function over a structure by reference, potentially
	/// using rayon for parallel execution. The closure receives `&A` instead
	/// of consuming `A`.
	///
	/// Elements must be `Send + Sync` because rayon's `par_iter()` requires
	/// `&A: Send`, which is equivalent to `A: Sync`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait ParRefFunctor: crate::classes::RefFunctor {
		/// Maps a function over the structure by reference in parallel.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		#[document_parameters(
			"The function to apply to each element reference. Must be `Send + Sync`.",
			"The structure to map over."
		)]
		#[document_returns("A new structure containing the mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_ref_functor::ParRefFunctor,
		/// };
		///
		/// let v = vec![1, 2, 3];
		/// let result = VecBrand::par_ref_map(|x: &i32| x * 2, &v);
		/// assert_eq!(result, vec![2, 4, 6]);
		/// ```
		fn par_ref_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
			f: impl Fn(&A) -> B + Send + Sync + 'a,
			fa: &Self::Of<'a, A>,
		) -> Self::Of<'a, B>;
	}

	/// Maps a function over a structure by reference in parallel.
	///
	/// Free function version that dispatches to [the type class' associated function][`ParRefFunctor::par_ref_map`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The input element type.",
		"The output element type."
	)]
	#[document_parameters(
		"The function to apply to each element reference. Must be `Send + Sync`.",
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
	/// let v = vec![1, 2, 3];
	/// let result = par_ref_map::<VecBrand, _, _>(|x: &i32| x * 2, &v);
	/// assert_eq!(result, vec![2, 4, 6]);
	/// ```
	pub fn par_ref_map<'a, Brand: ParRefFunctor, A: Send + Sync + 'a, B: Send + 'a>(
		f: impl Fn(&A) -> B + Send + Sync + 'a,
		fa: &Brand::Of<'a, A>,
	) -> Brand::Of<'a, B> {
		Brand::par_ref_map(f, fa)
	}
}

pub use inner::*;
