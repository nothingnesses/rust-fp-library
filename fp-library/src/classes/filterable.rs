//! Data structures that can be filtered and partitioned based on predicates or mapping functions.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(5);
//! let y = filter::<OptionBrand, _, _>(|a| a > 2, x);
//! assert_eq!(y, Some(5));
//! ```

use crate::{
	Apply,
	classes::{compactable::Compactable, functor::Functor},
	kinds::*,
	types::Pair,
};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A type class for data structures that can be filtered and partitioned.
///
/// `Filterable` extends [`Compactable`] and [`Functor`], adding methods for:
/// *   `filter`: Keeping elements that satisfy a predicate.
/// *   `filter_map`: Mapping and filtering in one step.
/// *   `partition`: Splitting elements based on a predicate.
/// *   `partition_map`: Mapping and partitioning in one step.
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
	///
	/// ### Type Signature
	///
	#[hm_signature(Filterable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The type of the elements in the input structure.",
		"The type of the success values.",
		"The type of the error values.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The function to apply to each element, returning a [`Result`].",
		"The data structure to partition."
	)]
	///
	/// ### Returns
	///
	/// A pair of data structures: the first containing the [`Ok`] values, and the second containing the [`Err`] values.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let x = Some(5);
	/// let Pair(oks, errs) = partition_map::<OptionBrand, _, _, _, _>(|a| if a > 2 { Ok(a) } else { Err(a) }, x);
	/// assert_eq!(oks, Some(5));
	/// assert_eq!(errs, None);
	/// ```
	fn partition_map<'a, A: 'a, O: 'a, E: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	>
	where
		Func: Fn(A) -> Result<O, E> + 'a,
	{
		Self::separate::<O, E>(Self::map::<A, Result<O, E>, Func>(func, fa))
	}

	/// Partitions a data structure based on a predicate.
	///
	/// The default implementation uses [`partition_map`].
	///
	/// **Note**: The return order is `(satisfied, not_satisfied)`, matching Rust's [`Iterator::partition`].
	/// This is achieved by mapping satisfied elements to [`Ok`] and unsatisfied elements to [`Err`] internally,
	/// as `separate` returns `(Oks, Errs)`.
	///
	/// ### Type Signature
	///
	#[hm_signature(Filterable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The type of the elements in the structure.",
		"The type of the predicate function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The predicate function.", "The data structure to partition.")]
	///
	/// ### Returns
	///
	/// A pair of data structures: the first containing elements that satisfy the predicate, and the second containing elements that do not.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let x = Some(5);
	/// let Pair(satisfied, not_satisfied) = partition::<OptionBrand, _, _>(|a| a > 2, x);
	/// assert_eq!(satisfied, Some(5));
	/// assert_eq!(not_satisfied, None);
	/// ```
	fn partition<'a, A: 'a + Clone, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	>
	where
		Func: Fn(A) -> bool + 'a,
	{
		Self::partition_map(move |a| if func(a.clone()) { Ok(a) } else { Err(a) }, fa)
	}

	/// Maps a function over a data structure and filters out [`None`] results.
	///
	/// The default implementation uses [`map`](crate::functions::map) and [`compact`](crate::functions::compact).
	///
	/// ### Type Signature
	/// ### Type Signature
	///
	#[hm_signature(Filterable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The type of the function to apply."
	)]
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning an [`Option`].
	/// * `fa`: The data structure to filter and map.
	///
	/// ### Returns
	///
	/// A new data structure containing only the values where the function returned [`Some`].
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(5);
	/// let y = filter_map::<OptionBrand, _, _, _>(|a| if a > 2 { Some(a * 2) } else { None }, x);
	/// assert_eq!(y, Some(10));
	/// ```
	fn filter_map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Option<B> + 'a,
	{
		Self::compact::<B>(Self::map::<A, Option<B>, Func>(func, fa))
	}

	/// Filters a data structure based on a predicate.
	///
	/// The default implementation uses [`filter_map`].
	///
	/// ### Type Signature
	///
	#[hm_signature(Filterable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The type of the elements in the structure.",
		"The type of the predicate function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The predicate function.", "The data structure to filter.")]
	///
	/// ### Returns
	///
	/// A new data structure containing only the elements that satisfy the predicate.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(5);
	/// let y = filter::<OptionBrand, _, _>(|a| a > 2, x);
	/// assert_eq!(y, Some(5));
	/// ```
	fn filter<'a, A: 'a + Clone, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Func: Fn(A) -> bool + 'a,
	{
		Self::filter_map(move |a| if func(a.clone()) { Some(a) } else { None }, fa)
	}
}

/// Partitions a data structure based on a function that returns a [`Result`].
///
/// Free function version that dispatches to [the type class' associated function][`Filterable::partition_map`].
///
/// ### Type Signature
///
#[hm_signature(Filterable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the filterable structure.",
	"The type of the elements in the input structure.",
	"The type of the success values.",
	"The type of the error values.",
	"The type of the function to apply."
)]
///
/// ### Parameters
///
#[doc_params(
	"The function to apply to each element, returning a [`Result`].",
	"The data structure to partition."
)]
///
/// ### Returns
///
/// A pair of data structures: the first containing the [`Ok`] values, and the second containing the [`Err`] values.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let x = Some(5);
/// let Pair(oks, errs) = partition_map::<OptionBrand, _, _, _, _>(|a| if a > 2 { Ok(a) } else { Err(a) }, x);
/// assert_eq!(oks, Some(5));
/// assert_eq!(errs, None);
/// ```
pub fn partition_map<'a, Brand: Filterable, A: 'a, O: 'a, E: 'a, Func>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Pair<
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
>
where
	Func: Fn(A) -> Result<O, E> + 'a,
{
	Brand::partition_map::<A, O, E, Func>(func, fa)
}

/// Partitions a data structure based on a predicate.
///
/// Free function version that dispatches to [the type class' associated function][`Filterable::partition`].
///
/// **Note**: The return order is `(satisfied, not_satisfied)`, matching Rust's [`Iterator::partition`].
///
/// ### Type Signature
///
#[hm_signature(Filterable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the filterable structure.",
	"The type of the elements in the structure.",
	"The type of the predicate function."
)]
///
/// ### Parameters
///
#[doc_params("The predicate function.", "The data structure to partition.")]
///
/// ### Returns
///
/// A pair of data structures: the first containing elements that satisfy the predicate, and the second containing elements that do not.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let x = Some(5);
/// let Pair(satisfied, not_satisfied) = partition::<OptionBrand, _, _>(|a| a > 2, x);
/// assert_eq!(satisfied, Some(5));
/// assert_eq!(not_satisfied, None);
/// ```
pub fn partition<'a, Brand: Filterable, A: 'a + Clone, Func>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Pair<
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
>
where
	Func: Fn(A) -> bool + 'a,
{
	Brand::partition(func, fa)
}

/// Maps a function over a data structure and filters out [`None`] results.
///
/// Free function version that dispatches to [the type class' associated function][`Filterable::filter_map`].
///
/// ### Type Signature
///
#[hm_signature(Filterable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the filterable structure.",
	"The type of the elements in the input structure.",
	"The type of the elements in the output structure.",
	"The type of the function to apply."
)]
///
/// ### Parameters
///
#[doc_params(
	"The function to apply to each element, returning an [`Option`].",
	"The data structure to filter and map."
)]
///
/// ### Returns
///
/// A new data structure containing only the values where the function returned [`Some`].
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(5);
/// let y = filter_map::<OptionBrand, _, _, _>(|a| if a > 2 { Some(a * 2) } else { None }, x);
/// assert_eq!(y, Some(10));
/// ```
pub fn filter_map<'a, Brand: Filterable, A: 'a, B: 'a, Func>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	Func: Fn(A) -> Option<B> + 'a,
{
	Brand::filter_map::<A, B, Func>(func, fa)
}

/// Filters a data structure based on a predicate.
///
/// Free function version that dispatches to [the type class' associated function][`Filterable::filter`].
///
/// ### Type Signature
///
#[hm_signature(Filterable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the filterable structure.",
	"The type of the elements in the structure.",
	"The type of the predicate function."
)]
///
/// ### Parameters
///
#[doc_params("The predicate function.", "The data structure to filter.")]
///
/// ### Returns
///
/// A new data structure containing only the elements that satisfy the predicate.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(5);
/// let y = filter::<OptionBrand, _, _>(|a| a > 2, x);
/// assert_eq!(y, Some(5));
/// ```
pub fn filter<'a, Brand: Filterable, A: 'a + Clone, Func>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
where
	Func: Fn(A) -> bool + 'a,
{
	Brand::filter(func, fa)
}
