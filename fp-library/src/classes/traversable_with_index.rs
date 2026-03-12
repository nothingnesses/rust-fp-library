//! A `Traversable` with an additional index.

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A `Traversable` with an additional index.
	///
	/// A `TraversableWithIndex` is a `Traversable` that also allows you to access the
	/// index of each element when traversing the structure.
	///
	/// ### Laws
	///
	/// `TraversableWithIndex` instances must be compatible with their `Traversable` instance:
	/// * Compatibility with Traversable: `traverse(f, fa) = traverse_with_index(|_, a| f(a), fa)`.
	#[document_type_parameters("The index type.")]
	#[document_examples]
	///
	/// TraversableWithIndex laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		OptionBrand,
	/// 		VecBrand,
	/// 	},
	/// 	classes::traversable_with_index::TraversableWithIndex,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3];
	/// let f = |a: i32| if a > 0 { Some(a * 2) } else { None };
	///
	/// // Compatibility with Traversable:
	/// // traverse(f, fa) = traverse_with_index(|_, a| f(a), fa)
	/// assert_eq!(
	/// 	traverse::<VecBrand, _, _, OptionBrand>(f, xs.clone()),
	/// 	VecBrand::traverse_with_index::<i32, i32, OptionBrand>(|_, a| f(a), xs),
	/// );
	/// ```
	pub trait TraversableWithIndex<I>:
		Traversable + FoldableWithIndex<I> + FunctorWithIndex<I> {
		/// Traverse the structure with an effectful function, providing the index of each element.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements.",
			"The type of the result inside the applicative.",
			"The applicative type."
		)]
		#[document_parameters(
			"The function to apply to each element and its index.",
			"The structure to traverse."
		)]
		#[document_returns("The structure of results inside the applicative.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		OptionBrand,
		/// 		VecBrand,
		/// 	},
		/// 	classes::traversable_with_index::TraversableWithIndex,
		/// };
		///
		/// let result = VecBrand::traverse_with_index::<i32, i32, OptionBrand>(
		/// 	|_i, x| if x > 0 { Some(x * 2) } else { None },
		/// 	vec![1, 2, 3],
		/// );
		/// assert_eq!(result, Some(vec![2, 4, 6]));
		/// ```
		fn traverse_with_index<'a, A: 'a, B: 'a + Clone, M: Applicative>(
			f: impl Fn(I, A) -> M::Of<'a, B> + 'a,
			ta: Self::Of<'a, A>,
		) -> M::Of<'a, Self::Of<'a, B>>
		where
			Self::Of<'a, B>: Clone,
			M::Of<'a, B>: Clone, {
			Self::sequence::<B, M>(Self::map_with_index::<A, M::Of<'a, B>>(f, ta))
		}
	}
}

pub use inner::*;
