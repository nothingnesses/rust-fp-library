//! By-reference filtering and partitioning of structures.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let v = vec![1, 2, 3, 4, 5];
//! let result =
//! 	ref_filter_map::<VecBrand, _, _>(|x: &i32| if *x > 3 { Some(*x) } else { None }, v);
//! assert_eq!(result, vec![4, 5]);
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

	/// By-reference filtering of structures.
	///
	/// Similar to [`Filterable`], but closures receive `&A` instead of `A`.
	/// This enables filtering collections by reference without consuming elements,
	/// or filtering memoized types that only provide `&A` access.
	///
	/// Default implementations derive:
	/// * `ref_partition_map` from `ref_map` + `separate` (via `Compactable`).
	/// * `ref_partition` from `ref_partition_map`.
	/// * `ref_filter_map` from `ref_map` + `compact` (via `Compactable`).
	/// * `ref_filter` from `ref_filter_map`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefFilterable: RefFunctor + Compactable {
		/// Partitions a structure by reference using a function that returns `Result`.
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
			"The function to apply to each element reference, returning a `Result`.",
			"The structure to partition."
		)]
		///
		#[document_returns("A pair of (errors, successes).")]
		#[document_examples]
		///
		/// ```ignore
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let (small, big) =
		/// 	ref_partition_map::<VecBrand, _, _, _>(|x: &i32| if *x > 3 { Ok(*x) } else { Err(*x) }, v);
		/// assert_eq!(big, vec![4, 5]);
		/// assert_eq!(small, vec![1, 2, 3]);
		/// ```ignore
		fn ref_partition_map<'a, A: 'a, E: 'a, O: 'a>(
			func: impl Fn(&A) -> Result<O, E> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Self::separate::<E, O>(Self::ref_map::<A, Result<O, E>>(func, fa))
		}

		/// Partitions a structure by reference using a predicate.
		///
		/// Returns `(not_satisfied, satisfied)`, matching Rust's `Iterator::partition`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the structure."
		)]
		///
		#[document_parameters("The predicate function.", "The structure to partition.")]
		///
		#[document_returns("A pair of (not satisfied, satisfied).")]
		#[document_examples]
		///
		/// ```ignore
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let (small, big) = ref_partition::<VecBrand, _>(|x: &i32| *x > 3, v);
		/// assert_eq!(big, vec![4, 5]);
		/// assert_eq!(small, vec![1, 2, 3]);
		/// ```ignore
		fn ref_partition<'a, A: 'a + Clone>(
			func: impl Fn(&A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			Self::ref_partition_map(
				move |a: &A| {
					if func(a) { Ok(a.clone()) } else { Err(a.clone()) }
				},
				fa,
			)
		}

		/// Maps a function over a structure by reference and filters out `None` results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the input structure.",
			"The type of the elements in the output structure."
		)]
		///
		#[document_parameters(
			"The function to apply to each element reference, returning an `Option`.",
			"The structure to filter and map."
		)]
		///
		#[document_returns("The structure with `None` results removed.")]
		#[document_examples]
		///
		/// ```ignore
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result =
		/// 	ref_filter_map::<VecBrand, _, _>(|x: &i32| if *x > 3 { Some(*x) } else { None }, v);
		/// assert_eq!(result, vec![4, 5]);
		/// ```ignore
		fn ref_filter_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> Option<B> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Self::compact::<B>(Self::ref_map::<A, Option<B>>(func, fa))
		}

		/// Filters a structure by reference using a predicate.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the structure."
		)]
		///
		#[document_parameters("The predicate function.", "The structure to filter.")]
		///
		#[document_returns("The structure with elements not satisfying the predicate removed.")]
		#[document_examples]
		///
		/// ```ignore
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result = ref_filter::<VecBrand, _>(|x: &i32| *x > 3, v);
		/// assert_eq!(result, vec![4, 5]);
		/// ```ignore
		fn ref_filter<'a, A: 'a + Clone>(
			func: impl Fn(&A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Self::ref_filter_map(move |a: &A| if func(a) { Some(a.clone()) } else { None }, fa)
		}
	}

	/// Partitions by reference using a function returning `Result`.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFilterable::ref_partition_map`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the elements.",
		"The error type.",
		"The success type."
	)]
	///
	#[document_parameters("The partitioning function.", "The structure to partition.")]
	///
	#[document_returns("A pair of (errors, successes).")]
	#[document_examples]
	///
	/// ```ignore
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let (small, big) =
	/// 	ref_partition_map::<VecBrand, _, _, _>(|x: &i32| if *x > 3 { Ok(*x) } else { Err(*x) }, v);
	/// assert_eq!(big, vec![4, 5]);
	/// assert_eq!(small, vec![1, 2, 3]);
	/// ```ignore
	pub fn ref_partition_map<'a, Brand: RefFilterable, A: 'a, E: 'a, O: 'a>(
		func: impl Fn(&A) -> Result<O, E> + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	) {
		Brand::ref_partition_map(func, fa)
	}

	/// Partitions by reference using a predicate.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFilterable::ref_partition`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the elements."
	)]
	///
	#[document_parameters("The predicate.", "The structure to partition.")]
	///
	#[document_returns("A pair of (not satisfied, satisfied).")]
	#[document_examples]
	///
	/// ```ignore
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let (small, big) = ref_partition::<VecBrand, _>(|x: &i32| *x > 3, v);
	/// assert_eq!(big, vec![4, 5]);
	/// assert_eq!(small, vec![1, 2, 3]);
	/// ```ignore
	pub fn ref_partition<'a, Brand: RefFilterable, A: 'a + Clone>(
		func: impl Fn(&A) -> bool + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) {
		Brand::ref_partition(func, fa)
	}

	/// Maps by reference and filters out `None` results.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFilterable::ref_filter_map`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the input elements.",
		"The type of the output elements."
	)]
	///
	#[document_parameters("The filter-map function.", "The structure to filter.")]
	///
	#[document_returns("The filtered structure.")]
	#[document_examples]
	///
	/// ```ignore
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result =
	/// 	ref_filter_map::<VecBrand, _, _>(|x: &i32| if *x > 3 { Some(*x) } else { None }, v);
	/// assert_eq!(result, vec![4, 5]);
	/// ```ignore
	pub fn ref_filter_map<'a, Brand: RefFilterable, A: 'a, B: 'a>(
		func: impl Fn(&A) -> Option<B> + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::ref_filter_map(func, fa)
	}

	/// Filters by reference using a predicate.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFilterable::ref_filter`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the elements."
	)]
	///
	#[document_parameters("The predicate.", "The structure to filter.")]
	///
	#[document_returns("The filtered structure.")]
	#[document_examples]
	///
	/// ```ignore
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result = ref_filter::<VecBrand, _>(|x: &i32| *x > 3, v);
	/// assert_eq!(result, vec![4, 5]);
	/// ```ignore
	pub fn ref_filter<'a, Brand: RefFilterable, A: 'a + Clone>(
		func: impl Fn(&A) -> bool + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::ref_filter(func, fa)
	}
}

pub use inner::*;
