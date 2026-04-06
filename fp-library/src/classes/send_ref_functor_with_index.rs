//! Thread-safe by-reference variant of [`FunctorWithIndex`](crate::classes::FunctorWithIndex).
//!
//! **User story:** "I want to map over a thread-safe memoized value by reference, with access to the index."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::send_ref_functor_with_index::SendRefFunctorWithIndex,
//! 	types::*,
//! };
//!
//! let lazy = ArcLazy::new(|| 42);
//! let mapped = <LazyBrand<ArcLazyConfig> as SendRefFunctorWithIndex>::send_ref_map_with_index(
//! 	|_, x: &i32| x.to_string(),
//! 	lazy,
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

	/// Thread-safe by-reference mapping with index over a structure.
	///
	/// Similar to [`RefFunctorWithIndex`], but closures and elements must be `Send + Sync`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefFunctorWithIndex: SendRefFunctor + WithIndex {
		/// Maps a function over the structure by reference with index (thread-safe).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The type of the result."
		)]
		#[document_parameters(
			"The function to apply to each element's index and reference. Must be `Send + Sync`.",
			"The structure to map over."
		)]
		#[document_returns("The mapped structure.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_ref_functor_with_index::SendRefFunctorWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 42);
		/// let mapped = <LazyBrand<ArcLazyConfig> as SendRefFunctorWithIndex>::send_ref_map_with_index(
		/// 	|_, x: &i32| x.to_string(),
		/// 	lazy,
		/// );
		/// assert_eq!(*mapped.evaluate(), "42");
		/// ```
		fn send_ref_map_with_index<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(Self::Index, &A) -> B + Send + Sync + 'a,
			fa: Self::Of<'a, A>,
		) -> Self::Of<'a, B>;
	}
}

pub use inner::*;
