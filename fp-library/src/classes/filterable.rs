//! Data structures that can be filtered and partitioned based on predicates or mapping functions.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(5);
//! let y = filter::<OptionBrand, _>(|a| a > 2, x);
//! assert_eq!(y, Some(5));
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

	/// A type class for data structures that can be filtered and partitioned.
	///
	/// `Filterable` extends [`Compactable`] and [`Functor`], adding methods for:
	/// *   `filter`: Keeping elements that satisfy a predicate.
	/// *   `filter_map`: Mapping and filtering in one step.
	/// *   `partition`: Splitting elements based on a predicate.
	/// *   `partition_map`: Mapping and partitioning in one step.
	///
	/// ### Laws
	///
	/// `Filterable` instances must satisfy the following laws:
	/// * Distributivity: `filter_map(identity, fa) = compact(fa)`.
	/// * Distributivity: `partition_map(identity, fa) = separate(fa)`.
	/// * Identity: `filter_map(Some, fa) = fa`.
	/// * Composition: `filter_map(|a| r(a).and_then(l), fa) = filter_map(l, filter_map(r, fa))`.
	/// * Consistency (`filter`/`filter_map`): `filter(p, fa) = filter_map(|a| if p(a) { Some(a) } else { None }, fa)`.
	/// * Consistency (`partition`/`partition_map`): `partition(p, fa) = partition_map(|a| if p(a) { Ok(a) } else { Err(a) }, fa)`.
	#[document_examples]
	///
	/// Filterable laws for [`Option`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Distributivity: filter_map(identity, fa) = compact(fa)
	/// let fa: Option<Option<i32>> = Some(Some(5));
	/// assert_eq!(
	/// 	filter_map::<OptionBrand, _, _>(identity, fa),
	/// 	compact::<OptionBrand, _>(fa),
	/// );
	///
	/// // Distributivity: partition_map(identity, fa) = separate(fa)
	/// let fa: Option<Result<i32, &str>> = Some(Ok(5));
	/// assert_eq!(
	/// 	partition_map::<OptionBrand, _, _, _>(identity, fa),
	/// 	separate::<OptionBrand, _, _>(fa),
	/// );
	///
	/// // Identity: filter_map(Some, fa) = fa
	/// assert_eq!(filter_map::<OptionBrand, _, _>(Some, Some(5)), Some(5));
	/// assert_eq!(filter_map::<OptionBrand, _, _>(Some, None::<i32>), None);
	///
	/// // Composition: filter_map(|a| r(a).and_then(l), fa) = filter_map(l, filter_map(r, fa))
	/// let l = |x: i32| if x > 3 { Some(x * 10) } else { None };
	/// let r = |x: i32| if x > 1 { Some(x + 1) } else { None };
	/// assert_eq!(
	/// 	filter_map::<OptionBrand, _, _>(|a| r(a).and_then(l), Some(5)),
	/// 	filter_map::<OptionBrand, _, _>(l, filter_map::<OptionBrand, _, _>(r, Some(5))),
	/// );
	///
	/// // Consistency (filter/filter_map): filter(p, fa) = filter_map(|a| if p(a) { Some(a) } else { None }, fa)
	/// let p = |x: i32| x > 3;
	/// assert_eq!(
	/// 	filter::<OptionBrand, _>(p, Some(5)),
	/// 	filter_map::<OptionBrand, _, _>(|a: i32| if p(a) { Some(a) } else { None }, Some(5)),
	/// );
	/// ```
	///
	/// Filterable laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Distributivity: filter_map(identity, fa) = compact(fa)
	/// let fa: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
	/// assert_eq!(
	/// 	filter_map::<VecBrand, _, _>(identity, fa.clone()),
	/// 	compact::<VecBrand, _>(fa),
	/// );
	///
	/// // Identity: filter_map(Some, fa) = fa
	/// assert_eq!(
	/// 	filter_map::<VecBrand, _, _>(Some, vec![1, 2, 3]),
	/// 	vec![1, 2, 3],
	/// );
	///
	/// // Composition: filter_map(|a| r(a).and_then(l), fa) = filter_map(l, filter_map(r, fa))
	/// let l = |x: i32| if x > 3 { Some(x * 10) } else { None };
	/// let r = |x: i32| if x > 1 { Some(x + 1) } else { None };
	/// assert_eq!(
	/// 	filter_map::<VecBrand, _, _>(|a| r(a).and_then(l), vec![1, 2, 3, 4, 5]),
	/// 	filter_map::<VecBrand, _, _>(l, filter_map::<VecBrand, _, _>(r, vec![1, 2, 3, 4, 5])),
	/// );
	///
	/// // Consistency (filter/filter_map): filter(p, fa) = filter_map(|a| if p(a) { Some(a) } else { None }, fa)
	/// let p = |x: i32| x > 3;
	/// assert_eq!(
	/// 	filter::<VecBrand, _>(p, vec![1, 2, 3, 4, 5]),
	/// 	filter_map::<VecBrand, _, _>(|a: i32| if p(a) { Some(a) } else { None }, vec![1, 2, 3, 4, 5]),
	/// );
	/// ```
	///
	/// ### Minimal Implementation
	///
	/// A minimal implementation of `Filterable` requires no specific method implementations, as all methods have default implementations based on [`Compactable`] and [`Functor`].
	///
	/// However, it is recommended to implement [`Filterable::partition_map`] and [`Filterable::filter_map`] to avoid the intermediate structure created by the default implementations (which use [`map`](crate::functions::map) followed by [`separate`](crate::functions::separate) or [`compact`](crate::functions::compact)).
	///
	/// *   If [`Filterable::partition_map`] is implemented, [`Filterable::partition`] is derived from it.
	/// *   If [`Filterable::filter_map`] is implemented, [`Filterable::filter`] is derived from it.
	pub trait Filterable: Compactable + Functor {
		/// Partitions a data structure based on a function that returns a [`Result`].
		///
		/// The default implementation uses [`map`](crate::functions::map) and [`separate`](crate::functions::separate).
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
			"The function to apply to each element, returning a [`Result`].",
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
		/// let x = Some(5);
		/// let (errs, oks) =
		/// 	partition_map::<OptionBrand, _, _, _>(|a| if a > 2 { Ok(a) } else { Err(a) }, x);
		/// assert_eq!(oks, Some(5));
		/// assert_eq!(errs, None);
		/// ```
		fn partition_map<'a, A: 'a, E: 'a, O: 'a>(
			func: impl Fn(A) -> Result<O, E> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Self::separate::<E, O>(Self::map::<A, Result<O, E>>(func, fa))
		}

		/// Partitions a data structure based on a predicate.
		///
		/// The default implementation uses [`partition_map`].
		///
		/// **Note**: The return order is `(satisfied, not_satisfied)`, matching Rust's [`Iterator::partition`].
		/// This is achieved by mapping satisfied elements to [`Ok`] and unsatisfied elements to [`Err`] internally,
		/// as `separate` returns `(Oks, Errs)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the structure."
		)]
		///
		#[document_parameters("The predicate function.", "The data structure to partition.")]
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
		/// let x = Some(5);
		/// let (not_satisfied, satisfied) = partition::<OptionBrand, _>(|a| a > 2, x);
		/// assert_eq!(satisfied, Some(5));
		/// assert_eq!(not_satisfied, None);
		/// ```
		fn partition<'a, A: 'a + Clone>(
			func: impl Fn(A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			Self::partition_map(move |a| if func(a.clone()) { Ok(a) } else { Err(a) }, fa)
		}

		/// Maps a function over a data structure and filters out [`None`] results.
		///
		/// The default implementation uses [`map`](crate::functions::map) and [`compact`](crate::functions::compact).
		///
		/// ### Type Signature
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the input structure.",
			"The type of the elements in the output structure."
		)]
		#[document_parameters(
			"The function to apply to each element, returning an [`Option`].",
			"The data structure to filter and map."
		)]
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
		/// let x = Some(5);
		/// let y = filter_map::<OptionBrand, _, _>(|a| if a > 2 { Some(a * 2) } else { None }, x);
		/// assert_eq!(y, Some(10));
		/// ```
		fn filter_map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> Option<B> + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Self::compact::<B>(Self::map::<A, Option<B>>(func, fa))
		}

		/// Filters a data structure based on a predicate.
		///
		/// The default implementation uses [`filter_map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the structure."
		)]
		///
		#[document_parameters("The predicate function.", "The data structure to filter.")]
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
		/// let x = Some(5);
		/// let y = filter::<OptionBrand, _>(|a| a > 2, x);
		/// assert_eq!(y, Some(5));
		/// ```
		fn filter<'a, A: 'a + Clone>(
			func: impl Fn(A) -> bool + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Self::filter_map(move |a| if func(a.clone()) { Some(a) } else { None }, fa)
		}
	}

	/// Partitions a data structure based on a function that returns a [`Result`].
	///
	/// Free function version that dispatches to [the type class' associated function][`Filterable::partition_map`].
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
		"The function to apply to each element, returning a [`Result`].",
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
	/// let x = Some(5);
	/// let (errs, oks) =
	/// 	partition_map::<OptionBrand, _, _, _>(|a| if a > 2 { Ok(a) } else { Err(a) }, x);
	/// assert_eq!(oks, Some(5));
	/// assert_eq!(errs, None);
	/// ```
	pub fn partition_map<'a, Brand: Filterable, A: 'a, E: 'a, O: 'a>(
		func: impl Fn(A) -> Result<O, E> + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	) {
		Brand::partition_map(func, fa)
	}

	/// Partitions a data structure based on a predicate.
	///
	/// Free function version that dispatches to [the type class' associated function][`Filterable::partition`].
	///
	/// **Note**: The return order is `(satisfied, not_satisfied)`, matching Rust's [`Iterator::partition`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the filterable structure.",
		"The type of the elements in the structure."
	)]
	///
	#[document_parameters("The predicate function.", "The data structure to partition.")]
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
	/// let x = Some(5);
	/// let (not_satisfied, satisfied) = partition::<OptionBrand, _>(|a| a > 2, x);
	/// assert_eq!(satisfied, Some(5));
	/// assert_eq!(not_satisfied, None);
	/// ```
	pub fn partition<'a, Brand: Filterable, A: 'a + Clone>(
		func: impl Fn(A) -> bool + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) {
		Brand::partition(func, fa)
	}

	/// Maps a function over a data structure and filters out [`None`] results.
	///
	/// Free function version that dispatches to [the type class' associated function][`Filterable::filter_map`].
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
		"The function to apply to each element, returning an [`Option`].",
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
	/// let x = Some(5);
	/// let y = filter_map::<OptionBrand, _, _>(|a| if a > 2 { Some(a * 2) } else { None }, x);
	/// assert_eq!(y, Some(10));
	/// ```
	pub fn filter_map<'a, Brand: Filterable, A: 'a, B: 'a>(
		func: impl Fn(A) -> Option<B> + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::filter_map(func, fa)
	}

	/// Filters a data structure based on a predicate.
	///
	/// Free function version that dispatches to [the type class' associated function][`Filterable::filter`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the filterable structure.",
		"The type of the elements in the structure."
	)]
	///
	#[document_parameters("The predicate function.", "The data structure to filter.")]
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
	/// let x = Some(5);
	/// let y = filter::<OptionBrand, _>(|a| a > 2, x);
	/// assert_eq!(y, Some(5));
	/// ```
	pub fn filter<'a, Brand: Filterable, A: 'a + Clone>(
		func: impl Fn(A) -> bool + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::filter(func, fa)
	}
}

pub use inner::*;
