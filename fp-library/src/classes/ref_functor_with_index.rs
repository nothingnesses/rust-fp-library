//! By-reference variant of [`FunctorWithIndex`](crate::classes::FunctorWithIndex).
//!
//! **User story:** "I want to map over a memoized value by reference, with access to the index."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::ref_functor_with_index::RefFunctorWithIndex,
//! 	types::*,
//! };
//!
//! let lazy = RcLazy::new(|| 42);
//! let mapped = <LazyBrand<RcLazyConfig> as RefFunctorWithIndex>::ref_map_with_index(
//! 	|_, x: &i32| x.to_string(),
//! 	&lazy,
//! );
//! assert_eq!(*mapped.evaluate(), "42");
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

	/// By-reference mapping with index over a structure.
	///
	/// Similar to [`FunctorWithIndex`], but the closure receives `&A` instead of `A`.
	/// This is the honest interface for memoized types like [`Lazy`](crate::types::Lazy)
	/// that internally hold a cached `&A`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefFunctorWithIndex: RefFunctor + WithIndex {
		/// Maps a function over the structure by reference, providing the index.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The type of the result."
		)]
		#[document_parameters(
			"The function to apply to each element's index and reference.",
			"The structure to map over."
		)]
		#[document_returns("The mapped structure.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_functor_with_index::RefFunctorWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 42);
		/// let mapped = <LazyBrand<RcLazyConfig> as RefFunctorWithIndex>::ref_map_with_index(
		/// 	|_, x: &i32| x.to_string(),
		/// 	&lazy,
		/// );
		/// assert_eq!(*mapped.evaluate(), "42");
		/// ```
		fn ref_map_with_index<'a, A: 'a, B: 'a>(
			f: impl Fn(Self::Index, &A) -> B + 'a,
			fa: &Self::Of<'a, A>,
		) -> Self::Of<'a, B>;
	}

	/// Maps a function over a structure by reference with access to the index.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFunctorWithIndex::ref_map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the structure.",
		"The type of the elements.",
		"The type of the result."
	)]
	#[document_parameters(
		"The function to apply to each element's index and reference.",
		"The structure to map over."
	)]
	#[document_returns("The mapped structure.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::explicit::*,
	/// 	types::*,
	/// };
	///
	/// let lazy = RcLazy::new(|| 42);
	/// let mapped =
	/// 	map_with_index::<LazyBrand<RcLazyConfig>, _, _, _, _>(|_, x: &i32| x.to_string(), &lazy);
	/// assert_eq!(*mapped.evaluate(), "42");
	/// ```
	pub fn ref_map_with_index<'a, Brand: RefFunctorWithIndex, A: 'a, B: 'a>(
		f: impl Fn(Brand::Index, &A) -> B + 'a,
		fa: &Brand::Of<'a, A>,
	) -> Brand::Of<'a, B> {
		Brand::ref_map_with_index(f, fa)
	}
}

pub use inner::*;
