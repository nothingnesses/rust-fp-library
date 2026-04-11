//! By-reference indexed filtering and partitioning of structures.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let v = vec![10, 20, 30, 40, 50];
//! let result = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
//! 	|i, x: &i32| if i >= 2 { Some(*x) } else { None },
//! 	&v,
//! );
//! assert_eq!(result, vec![30, 40, 50]);
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

	/// By-reference indexed filtering of structures.
	///
	/// Similar to [`FilterableWithIndex`], but closures receive `&A` instead of `A`.
	/// Default implementations derive all methods from `ref_filter_map_with_index`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefFilterableWithIndex: RefFilterable + RefFunctorWithIndex + WithIndex {
		/// Maps a function with index over a structure by reference and filters out `None` results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements."
		)]
		///
		#[document_parameters(
			"The function to apply to each (index, element reference) pair.",
			"The structure to filter and map."
		)]
		///
		#[document_returns("The filtered structure.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30, 40, 50];
		/// let result = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
		/// 	|i, x: &i32| if i >= 2 { Some(*x) } else { None },
		/// 	&v,
		/// );
		/// assert_eq!(result, vec![30, 40, 50]);
		/// ```
		fn ref_filter_map_with_index<'a, A: 'a, B: 'a>(
			func: impl Fn(Self::Index, &A) -> Option<B> + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);

		/// Filters by reference with index using a predicate.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The predicate.", "The structure to filter.")]
		///
		#[document_returns("The filtered structure.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30, 40, 50];
		/// let result = filter_with_index_explicit::<VecBrand, _, _, _>(|i, _x: &i32| i >= 2, &v);
		/// assert_eq!(result, vec![30, 40, 50]);
		/// ```
		fn ref_filter_with_index<'a, A: 'a + Clone>(
			func: impl Fn(Self::Index, &A) -> bool + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Self::ref_filter_map_with_index(
				move |i, a: &A| {
					if func(i, a) { Some(a.clone()) } else { None }
				},
				fa,
			)
		}

		/// Partitions by reference with index using a function returning `Result`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The error type.",
			"The success type."
		)]
		///
		#[document_parameters("The partitioning function.", "The structure to partition.")]
		///
		#[document_returns("A pair of (errors, successes).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30, 40, 50];
		/// let (left, right) = partition_map_with_index_explicit::<VecBrand, _, _, _, _, _>(
		/// 	|i, x: &i32| if i >= 2 { Ok(*x) } else { Err(*x) },
		/// 	&v,
		/// );
		/// assert_eq!(left, vec![10, 20]);
		/// assert_eq!(right, vec![30, 40, 50]);
		/// ```
		fn ref_partition_map_with_index<'a, A: 'a, E: 'a, O: 'a>(
			func: impl Fn(Self::Index, &A) -> Result<O, E> + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Self::separate::<E, O>(Self::ref_map_with_index::<A, Result<O, E>>(func, fa))
		}

		/// Partitions by reference with index using a predicate.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The type of the elements.")]
		///
		#[document_parameters("The predicate.", "The structure to partition.")]
		///
		#[document_returns("A pair of (not satisfied, satisfied).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30, 40, 50];
		/// let (left, right) =
		/// 	partition_with_index_explicit::<VecBrand, _, _, _>(|i, _x: &i32| i >= 2, &v);
		/// assert_eq!(left, vec![10, 20]);
		/// assert_eq!(right, vec![30, 40, 50]);
		/// ```
		fn ref_partition_with_index<'a, A: 'a + Clone>(
			func: impl Fn(Self::Index, &A) -> bool + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			Self::ref_partition_map_with_index(
				move |i, a: &A| {
					if func(i, a) { Ok(a.clone()) } else { Err(a.clone()) }
				},
				fa,
			)
		}
	}

	/// Maps by reference with index and filters out `None` results.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFilterableWithIndex::ref_filter_map_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the input elements.",
		"The type of the output elements."
	)]
	///
	#[document_parameters("The filter-map function.", "The structure.")]
	///
	#[document_returns("The filtered structure.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![10, 20, 30, 40, 50];
	/// let result = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
	/// 	|i, x: &i32| if i >= 2 { Some(*x) } else { None },
	/// 	&v,
	/// );
	/// assert_eq!(result, vec![30, 40, 50]);
	/// ```
	pub fn ref_filter_map_with_index<'a, Brand: RefFilterableWithIndex, A: 'a, B: 'a>(
		func: impl Fn(Brand::Index, &A) -> Option<B> + 'a,
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::ref_filter_map_with_index(func, fa)
	}

	/// Filters by reference with index using a predicate.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFilterableWithIndex::ref_filter_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the elements."
	)]
	///
	#[document_parameters("The predicate.", "The structure.")]
	///
	#[document_returns("The filtered structure.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![10, 20, 30, 40, 50];
	/// let result = filter_with_index_explicit::<VecBrand, _, _, _>(|i, _x: &i32| i >= 2, &v);
	/// assert_eq!(result, vec![30, 40, 50]);
	/// ```
	pub fn ref_filter_with_index<'a, Brand: RefFilterableWithIndex, A: 'a + Clone>(
		func: impl Fn(Brand::Index, &A) -> bool + 'a,
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::ref_filter_with_index(func, fa)
	}

	/// Partitions by reference with index using a function returning `Result`.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFilterableWithIndex::ref_partition_map_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the input elements.",
		"The error type.",
		"The success type."
	)]
	///
	#[document_parameters("The partitioning function.", "The structure.")]
	///
	#[document_returns("A pair of (errors, successes).")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![10, 20, 30, 40, 50];
	/// let (left, right) = partition_map_with_index_explicit::<VecBrand, _, _, _, _, _>(
	/// 	|i, x: &i32| if i >= 2 { Ok(*x) } else { Err(*x) },
	/// 	&v,
	/// );
	/// assert_eq!(left, vec![10, 20]);
	/// assert_eq!(right, vec![30, 40, 50]);
	/// ```
	pub fn ref_partition_map_with_index<'a, Brand: RefFilterableWithIndex, A: 'a, E: 'a, O: 'a>(
		func: impl Fn(Brand::Index, &A) -> Result<O, E> + 'a,
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	) {
		Brand::ref_partition_map_with_index(func, fa)
	}

	/// Partitions by reference with index using a predicate.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFilterableWithIndex::ref_partition_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the elements."
	)]
	///
	#[document_parameters("The predicate.", "The structure.")]
	///
	#[document_returns("A pair of (not satisfied, satisfied).")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![10, 20, 30, 40, 50];
	/// let (left, right) =
	/// 	partition_with_index_explicit::<VecBrand, _, _, _>(|i, _x: &i32| i >= 2, &v);
	/// assert_eq!(left, vec![10, 20]);
	/// assert_eq!(right, vec![30, 40, 50]);
	/// ```
	pub fn ref_partition_with_index<'a, Brand: RefFilterableWithIndex, A: 'a + Clone>(
		func: impl Fn(Brand::Index, &A) -> bool + 'a,
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) {
		Brand::ref_partition_with_index(func, fa)
	}
}

pub use inner::*;
