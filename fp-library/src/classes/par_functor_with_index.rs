//! A `ParFunctor` with an additional index.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A `ParFunctor` with an additional index.
	///
	/// `ParFunctorWithIndex` is the parallel counterpart to
	/// [`FunctorWithIndex`](crate::classes::FunctorWithIndex). Implementors define
	/// [`par_map_with_index`][ParFunctorWithIndex::par_map_with_index] directly.
	///
	/// ### Laws
	///
	/// `ParFunctorWithIndex` instances must satisfy the same laws as `FunctorWithIndex`:
	/// * Identity: `par_map_with_index(|_, a| a, fa) = fa`.
	/// * Compatibility with `ParFunctor`:
	///   `par_map(f, fa) = par_map_with_index(|_, a| f(a), fa)`.
	///
	/// ### Thread Safety
	///
	/// The index type must satisfy `Self::Index: Send + Sync + Copy` when calling
	/// [`par_map_with_index`][ParFunctorWithIndex::par_map_with_index]. These bounds apply even
	/// when the `rayon` feature is disabled.
	#[document_examples]
	///
	/// ParFunctorWithIndex laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	classes::par_functor_with_index::ParFunctorWithIndex,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![10, 20, 30];
	///
	/// // Identity: par_map_with_index(|_, a| a, fa) = fa
	/// assert_eq!(VecBrand::par_map_with_index(|_, a: i32| a, xs.clone()), xs);
	///
	/// // Compatibility with ParFunctor:
	/// // par_map(f, fa) = par_map_with_index(|_, a| f(a), fa)
	/// let f = |a: i32| a * 2;
	/// assert_eq!(
	/// 	par_map::<VecBrand, _, _>(f, xs.clone()),
	/// 	VecBrand::par_map_with_index(|_, a| f(a), xs),
	/// );
	/// ```
	pub trait ParFunctorWithIndex: ParFunctor + FunctorWithIndex {
		/// Maps a function over the structure in parallel, providing the index of each element.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential indexed mapping.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The type of the result."
		)]
		#[document_parameters(
			"The function to apply to each element and its index. Must be `Send + Sync`.",
			"The structure to map over."
		)]
		#[document_returns("The mapped structure.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_functor_with_index::ParFunctorWithIndex,
		/// };
		///
		/// let result = VecBrand::par_map_with_index(|i, x: i32| x + i as i32, vec![10, 20, 30]);
		/// assert_eq!(result, vec![10, 21, 32]);
		/// ```
		fn par_map_with_index<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(Self::Index, A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Self::Index: Send + Sync + Copy + 'a;
	}

	/// Maps a function over the structure in parallel, providing the index of each element.
	///
	/// Free function version that dispatches to
	/// [`ParFunctorWithIndex::par_map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the structure.",
		"The type of the elements.",
		"The type of the result."
	)]
	#[document_parameters(
		"The function to apply to each element and its index. Must be `Send + Sync`.",
		"The structure to map over."
	)]
	#[document_returns("The mapped structure.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result: Vec<i32> =
	/// 	par_map_with_index::<VecBrand, _, _>(|i, x: i32| x + i as i32, vec![10, 20, 30]);
	/// assert_eq!(result, vec![10, 21, 32]);
	/// ```
	pub fn par_map_with_index<'a, Brand, A: 'a + Send, B: 'a + Send>(
		f: impl Fn(Brand::Index, A) -> B + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Brand: ParFunctorWithIndex,
		Brand::Index: Send + Sync + Copy + 'a, {
		Brand::par_map_with_index(f, fa)
	}
}

pub use inner::*;
