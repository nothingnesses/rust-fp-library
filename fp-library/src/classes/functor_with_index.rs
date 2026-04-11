//! A `Functor` with an additional index.

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A `Functor` with an additional index.
	///
	/// A `FunctorWithIndex` is a `Functor` that also allows you to access the
	/// index of each element when mapping over the structure. The index type is
	/// uniquely determined by the implementing brand via the [`WithIndex`] supertype,
	/// encoding the functional dependency `f -> i` from PureScript.
	///
	/// ### Laws
	///
	/// `FunctorWithIndex` instances must satisfy:
	/// * Identity: `map_with_index(|_, a| a, fa) = fa`.
	/// * Compatibility with Functor: `map(f, fa) = map_with_index(|_, a| f(a), fa)`.
	#[document_examples]
	///
	/// FunctorWithIndex laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	classes::functor_with_index::FunctorWithIndex,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![10, 20, 30];
	///
	/// // Identity: map_with_index(|_, a| a, fa) = fa
	/// assert_eq!(VecBrand::map_with_index(|_, a: i32| a, xs.clone()), xs,);
	///
	/// // Compatibility with Functor: map(f, fa) = map_with_index(|_, a| f(a), fa)
	/// let f = |a: i32| a * 2;
	/// assert_eq!(
	/// 	map_explicit::<VecBrand, _, _, _, _>(f, xs.clone()),
	/// 	VecBrand::map_with_index(|_, a| f(a), xs),
	/// );
	/// ```
	pub trait FunctorWithIndex: Functor + WithIndex {
		/// Map a function over the structure, providing the index of each element.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The type of the result."
		)]
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The structure to map over."
		)]
		#[document_returns("The mapped structure.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::functor_with_index::FunctorWithIndex,
		/// };
		///
		/// let result = VecBrand::map_with_index(|i, x: i32| x + i as i32, vec![10, 20, 30]);
		/// assert_eq!(result, vec![10, 21, 32]);
		/// ```
		fn map_with_index<'a, A: 'a, B: 'a>(
			f: impl Fn(Self::Index, A) -> B + 'a,
			fa: Self::Of<'a, A>,
		) -> Self::Of<'a, B>;
	}

	/// Maps a function over a structure with access to the index of each element.
	///
	/// Free function version that dispatches to [the type class' associated function][`FunctorWithIndex::map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the structure.",
		"The type of the elements.",
		"The type of the result."
	)]
	#[document_parameters(
		"The function to apply to each element and its index.",
		"The structure to map over."
	)]
	#[document_returns("The mapped structure.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	functions::*,
	/// };
	///
	/// let result =
	/// 	map_with_index_explicit::<VecBrand, _, _, _, _>(|i, x: i32| x + i as i32, vec![10, 20, 30]);
	/// assert_eq!(result, vec![10, 21, 32]);
	/// ```
	pub fn map_with_index<'a, Brand: FunctorWithIndex, A: 'a, B: 'a>(
		f: impl Fn(Brand::Index, A) -> B + 'a,
		fa: Brand::Of<'a, A>,
	) -> Brand::Of<'a, B> {
		Brand::map_with_index(f, fa)
	}
}

pub use inner::*;
