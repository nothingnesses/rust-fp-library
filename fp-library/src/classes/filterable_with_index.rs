//! A `Filterable` with an additional index.

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A `Filterable` with an additional index.
	///
	/// A `FilterableWithIndex` is a `Filterable` that also allows you to access the
	/// index of each element when filtering, partitioning, or mapping over the structure.
	/// The index type is uniquely determined by the implementing brand via the [`WithIndex`]
	/// supertype, encoding the functional dependency `f -> i` from PureScript.
	///
	/// ### Laws
	///
	/// `FilterableWithIndex` instances must be compatible with their `Filterable` instance:
	/// * Compatibility (filter_map): `filter_map(f, fa) = filter_map_with_index(|_, a| f(a), fa)`.
	/// * Compatibility (partition_map): `partition_map(f, fa) = partition_map_with_index(|_, a| f(a), fa)`.
	#[document_examples]
	///
	/// FilterableWithIndex laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	classes::filterable_with_index::FilterableWithIndex,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3, 4, 5];
	///
	/// // Compatibility (filter_map):
	/// // filter_map(f, fa) = filter_map_with_index(|_, a| f(a), fa)
	/// let f = |a: i32| if a > 3 { Some(a * 10) } else { None };
	/// assert_eq!(
	/// 	filter_map::<VecBrand, _, _, _, _>(f, xs.clone()),
	/// 	VecBrand::filter_map_with_index(|_, a| f(a), xs.clone()),
	/// );
	///
	/// // Compatibility (partition_map):
	/// // partition_map(f, fa) = partition_map_with_index(|_, a| f(a), fa)
	/// let g = |a: i32| if a > 3 { Ok(a) } else { Err(a) };
	/// assert_eq!(
	/// 	partition_map::<VecBrand, _, _, _, _, _>(g, xs.clone()),
	/// 	VecBrand::partition_map_with_index(|_, a| g(a), xs),
	/// );
	/// ```
	///
	/// ### Minimal Implementation
	///
	/// A minimal implementation of `FilterableWithIndex` requires no specific method
	/// implementations, as all methods have default implementations based on
	/// [`Filterable`], [`FunctorWithIndex`], and [`Compactable`].
	///
	/// However, it is recommended to implement [`FilterableWithIndex::partition_map_with_index`]
	/// and [`FilterableWithIndex::filter_map_with_index`] to avoid the intermediate structure
	/// created by the default implementations (which use [`map_with_index`](FunctorWithIndex::map_with_index)
	/// followed by [`separate`](Compactable::separate) or [`compact`](Compactable::compact)).
	///
	/// * If [`FilterableWithIndex::partition_map_with_index`] is implemented,
	///   [`FilterableWithIndex::partition_with_index`] is derived from it.
	/// * If [`FilterableWithIndex::filter_map_with_index`] is implemented,
	///   [`FilterableWithIndex::filter_with_index`] is derived from it.
	pub trait FilterableWithIndex: Filterable + FunctorWithIndex {
		/// Partitions a data structure based on a function that receives the index
		/// and returns a [`Result`].
		///
		/// The default implementation uses [`map_with_index`](FunctorWithIndex::map_with_index)
		/// and [`separate`](Compactable::separate).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the input structure.",
			"The type of the error values.",
			"The type of the success values."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index, returning a [`Result`].",
			"The data structure to partition."
		)]
		///
		#[document_returns(
			"A pair of data structures: the first containing the [`Err`] values, and the second containing the [`Ok`] values."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::filterable_with_index::FilterableWithIndex,
		/// };
		///
		/// let xs = vec![10, 20, 30, 40];
		/// let (errs, oks) =
		/// 	VecBrand::partition_map_with_index(|i, a: i32| if i < 2 { Ok(a) } else { Err(a) }, xs);
		/// assert_eq!(oks, vec![10, 20]);
		/// assert_eq!(errs, vec![30, 40]);
		/// ```
		fn partition_map_with_index<'a, A: 'a, E: 'a, O: 'a>(
			func: impl Fn(Self::Index, A) -> Result<O, E> + 'a,
			fa: Self::Of<'a, A>,
		) -> (Self::Of<'a, E>, Self::Of<'a, O>) {
			Self::separate::<E, O>(Self::map_with_index::<A, Result<O, E>>(func, fa))
		}

		/// Partitions a data structure based on a predicate that receives the index.
		///
		/// The default implementation uses [`partition_map_with_index`](FilterableWithIndex::partition_map_with_index).
		///
		/// **Note**: The return order is `(not_satisfied, satisfied)`, matching
		/// [`Filterable::partition`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the structure."
		)]
		///
		#[document_parameters(
			"The predicate function receiving the index and element.",
			"The data structure to partition."
		)]
		///
		#[document_returns(
			"A pair of data structures: the first containing elements that do not satisfy the predicate, and the second containing elements that do."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::filterable_with_index::FilterableWithIndex,
		/// };
		///
		/// let xs = vec![10, 20, 30, 40];
		/// let (not_satisfied, satisfied) = VecBrand::partition_with_index(|i, _a: i32| i < 2, xs);
		/// assert_eq!(satisfied, vec![10, 20]);
		/// assert_eq!(not_satisfied, vec![30, 40]);
		/// ```
		fn partition_with_index<'a, A: 'a + Clone>(
			func: impl Fn(Self::Index, A) -> bool + 'a,
			fa: Self::Of<'a, A>,
		) -> (Self::Of<'a, A>, Self::Of<'a, A>) {
			Self::partition_map_with_index(
				move |i, a| {
					if func(i, a.clone()) { Ok(a) } else { Err(a) }
				},
				fa,
			)
		}

		/// Maps a function over a data structure with the index and filters out [`None`] results.
		///
		/// The default implementation uses [`map_with_index`](FunctorWithIndex::map_with_index)
		/// and [`compact`](Compactable::compact).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the input structure.",
			"The type of the elements in the output structure."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index, returning an [`Option`].",
			"The data structure to filter and map."
		)]
		///
		#[document_returns(
			"A new data structure containing only the values where the function returned [`Some`]."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::filterable_with_index::FilterableWithIndex,
		/// };
		///
		/// let xs = vec![10, 20, 30, 40];
		/// let result = VecBrand::filter_map_with_index(
		/// 	|i, a: i32| if i % 2 == 0 { Some(a * 2) } else { None },
		/// 	xs,
		/// );
		/// assert_eq!(result, vec![20, 60]);
		/// ```
		fn filter_map_with_index<'a, A: 'a, B: 'a>(
			func: impl Fn(Self::Index, A) -> Option<B> + 'a,
			fa: Self::Of<'a, A>,
		) -> Self::Of<'a, B> {
			Self::compact::<B>(Self::map_with_index::<A, Option<B>>(func, fa))
		}

		/// Filters a data structure based on a predicate that receives the index.
		///
		/// The default implementation uses [`filter_map_with_index`](FilterableWithIndex::filter_map_with_index).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the structure."
		)]
		///
		#[document_parameters(
			"The predicate function receiving the index and element.",
			"The data structure to filter."
		)]
		///
		#[document_returns(
			"A new data structure containing only the elements that satisfy the predicate."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::filterable_with_index::FilterableWithIndex,
		/// };
		///
		/// let xs = vec![10, 20, 30, 40];
		/// let result = VecBrand::filter_with_index(|i, _a: i32| i < 2, xs);
		/// assert_eq!(result, vec![10, 20]);
		/// ```
		fn filter_with_index<'a, A: 'a + Clone>(
			func: impl Fn(Self::Index, A) -> bool + 'a,
			fa: Self::Of<'a, A>,
		) -> Self::Of<'a, A> {
			Self::filter_map_with_index(
				move |i, a| {
					if func(i, a.clone()) { Some(a) } else { None }
				},
				fa,
			)
		}
	}

	/// Partitions a data structure based on a function that receives the index
	/// and returns a [`Result`].
	///
	/// Free function version that dispatches to
	/// [the type class' associated function][`FilterableWithIndex::partition_map_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the filterable structure.",
		"The type of the elements in the input structure.",
		"The type of the error values.",
		"The type of the success values."
	)]
	///
	#[document_parameters(
		"The function to apply to each element and its index, returning a [`Result`].",
		"The data structure to partition."
	)]
	///
	#[document_returns(
		"A pair of data structures: the first containing the [`Err`] values, and the second containing the [`Ok`] values."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![10, 20, 30, 40];
	/// let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
	/// 	|i, a: i32| if i < 2 { Ok(a) } else { Err(a) },
	/// 	xs,
	/// );
	/// assert_eq!(oks, vec![10, 20]);
	/// assert_eq!(errs, vec![30, 40]);
	/// ```
	pub fn partition_map_with_index<'a, Brand: FilterableWithIndex, A: 'a, E: 'a, O: 'a>(
		func: impl Fn(Brand::Index, A) -> Result<O, E> + 'a,
		fa: Brand::Of<'a, A>,
	) -> (Brand::Of<'a, E>, Brand::Of<'a, O>) {
		Brand::partition_map_with_index(func, fa)
	}

	/// Partitions a data structure based on a predicate that receives the index.
	///
	/// Free function version that dispatches to
	/// [the type class' associated function][`FilterableWithIndex::partition_with_index`].
	///
	/// **Note**: The return order is `(not_satisfied, satisfied)`, matching
	/// [`Filterable::partition`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the filterable structure.",
		"The type of the elements in the structure."
	)]
	///
	#[document_parameters(
		"The predicate function receiving the index and element.",
		"The data structure to partition."
	)]
	///
	#[document_returns(
		"A pair of data structures: the first containing elements that do not satisfy the predicate, and the second containing elements that do."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![10, 20, 30, 40];
	/// let (not_satisfied, satisfied) =
	/// 	partition_with_index::<VecBrand, _, _, _>(|i, _a: i32| i < 2, xs);
	/// assert_eq!(satisfied, vec![10, 20]);
	/// assert_eq!(not_satisfied, vec![30, 40]);
	/// ```
	pub fn partition_with_index<'a, Brand: FilterableWithIndex, A: 'a + Clone>(
		func: impl Fn(Brand::Index, A) -> bool + 'a,
		fa: Brand::Of<'a, A>,
	) -> (Brand::Of<'a, A>, Brand::Of<'a, A>) {
		Brand::partition_with_index(func, fa)
	}

	/// Maps a function over a data structure with the index and filters out [`None`] results.
	///
	/// Free function version that dispatches to
	/// [the type class' associated function][`FilterableWithIndex::filter_map_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the filterable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure."
	)]
	///
	#[document_parameters(
		"The function to apply to each element and its index, returning an [`Option`].",
		"The data structure to filter and map."
	)]
	///
	#[document_returns(
		"A new data structure containing only the values where the function returned [`Some`]."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![10, 20, 30, 40];
	/// let result = filter_map_with_index::<VecBrand, _, _, _, _>(
	/// 	|i, a: i32| if i % 2 == 0 { Some(a * 2) } else { None },
	/// 	xs,
	/// );
	/// assert_eq!(result, vec![20, 60]);
	/// ```
	pub fn filter_map_with_index<'a, Brand: FilterableWithIndex, A: 'a, B: 'a>(
		func: impl Fn(Brand::Index, A) -> Option<B> + 'a,
		fa: Brand::Of<'a, A>,
	) -> Brand::Of<'a, B> {
		Brand::filter_map_with_index(func, fa)
	}

	/// Filters a data structure based on a predicate that receives the index.
	///
	/// Free function version that dispatches to
	/// [the type class' associated function][`FilterableWithIndex::filter_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the filterable structure.",
		"The type of the elements in the structure."
	)]
	///
	#[document_parameters(
		"The predicate function receiving the index and element.",
		"The data structure to filter."
	)]
	///
	#[document_returns(
		"A new data structure containing only the elements that satisfy the predicate."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![10, 20, 30, 40];
	/// let result = filter_with_index::<VecBrand, _, _, _>(|i, _a: i32| i < 2, xs);
	/// assert_eq!(result, vec![10, 20]);
	/// ```
	pub fn filter_with_index<'a, Brand: FilterableWithIndex, A: 'a + Clone>(
		func: impl Fn(Brand::Index, A) -> bool + 'a,
		fa: Brand::Of<'a, A>,
	) -> Brand::Of<'a, A> {
		Brand::filter_with_index(func, fa)
	}
}

pub use inner::*;
