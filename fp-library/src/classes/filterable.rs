//! Filterable type class.
//!
//! This module defines the [`Filterable`] trait, which represents data structures that can be filtered and partitioned.

use crate::{
	Apply,
	classes::{compactable::Compactable, functor::Functor},
	kinds::*,
	types::Pair,
};

/// A type class for data structures that can be filtered and partitioned.
///
/// `Filterable` extends [`Compactable`] and [`Functor`], adding methods for:
/// *   `filter`: Keeping elements that satisfy a predicate.
/// *   `filter_map`: Mapping and filtering in one step.
/// *   `partition`: Splitting elements based on a predicate.
/// *   `partition_map`: Mapping and partitioning in one step.
pub trait Filterable: Compactable + Functor {
	/// Partitions a data structure based on a function that returns a `Result`.
	///
	/// The default implementation uses [`Functor::map`] and [`Compactable::separate`].
	///
	/// ### Type Signature
	///
	/// `forall a e o f. Filterable f => (a -> Result o e) -> f a -> (f o, f e)`
	///
	/// ### Type Parameters
	///
	/// * `Func`: The type of the function to apply.
	/// * `A`: The type of the elements in the input structure.
	/// * `E`: The type of the error values.
	/// * `O`: The type of the success values.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a `Result`.
	/// * `fa`: The data structure to partition.
	///
	/// ### Returns
	///
	/// A pair of data structures: the first containing the `Ok` values, and the second containing the `Err` values.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::Pair;
	///
	/// let x = Some(5);
	/// let Pair(oks, errs) = OptionBrand::partition_map(|a| if a > 2 { Ok(a) } else { Err(a) }, x);
	/// assert_eq!(oks, Some(5));
	/// assert_eq!(errs, None);
	/// ```
	fn partition_map<'a, Func, A: 'a, E: 'a, O: 'a>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	>
	where
		Func: Fn(A) -> Result<O, E> + 'a,
	{
		Self::separate(Self::map(func, fa))
	}

	/// Partitions a data structure based on a predicate.
	///
	/// The default implementation uses [`partition_map`].
	///
	/// **Note**: The return order is `(satisfied, not_satisfied)`, matching Rust's `Iterator::partition`.
	/// This is achieved by mapping satisfied elements to `Ok` and unsatisfied elements to `Err` internally,
	/// as `separate` returns `(Oks, Errs)`.
	///
	/// ### Type Signature
	///
	/// `forall a f. Filterable f => (a -> bool) -> f a -> (f a, f a)`
	///
	/// ### Type Parameters
	///
	/// * `Func`: The type of the predicate function.
	/// * `A`: The type of the elements in the structure.
	///
	/// ### Parameters
	///
	/// * `func`: The predicate function.
	/// * `fa`: The data structure to partition.
	///
	/// ### Returns
	///
	/// A pair of data structures: the first containing elements that satisfy the predicate, and the second containing elements that do not.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::Pair;
	///
	/// let x = Some(5);
	/// let Pair(satisfied, not_satisfied) = OptionBrand::partition(|a| a > 2, x);
	/// assert_eq!(satisfied, Some(5));
	/// assert_eq!(not_satisfied, None);
	/// ```
	fn partition<'a, Func, A: 'a + Clone>(
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

	/// Maps a function over a data structure and filters out `None` results.
	///
	/// The default implementation uses [`Functor::map`] and [`Compactable::compact`].
	///
	/// ### Type Signature
	///
	/// `forall a b f. Filterable f => (a -> Option b) -> f a -> f b`
	///
	/// ### Type Parameters
	///
	/// * `Func`: The type of the function to apply.
	/// * `A`: The type of the elements in the input structure.
	/// * `B`: The type of the elements in the output structure.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning an `Option`.
	/// * `fa`: The data structure to filter and map.
	///
	/// ### Returns
	///
	/// A new data structure containing only the values where the function returned `Some`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::filter_map(|a| if a > 2 { Some(a * 2) } else { None }, x);
	/// assert_eq!(y, Some(10));
	/// ```
	fn filter_map<'a, Func, A: 'a, B: 'a>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Option<B> + 'a,
	{
		Self::compact(Self::map(func, fa))
	}

	/// Filters a data structure based on a predicate.
	///
	/// The default implementation uses [`filter_map`].
	///
	/// ### Type Signature
	///
	/// `forall a f. Filterable f => (a -> bool) -> f a -> f a`
	///
	/// ### Type Parameters
	///
	/// * `Func`: The type of the predicate function.
	/// * `A`: The type of the elements in the structure.
	///
	/// ### Parameters
	///
	/// * `func`: The predicate function.
	/// * `fa`: The data structure to filter.
	///
	/// ### Returns
	///
	/// A new data structure containing only the elements that satisfy the predicate.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::filter(|a| a > 2, x);
	/// assert_eq!(y, Some(5));
	/// ```
	fn filter<'a, Func, A: 'a + Clone>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Func: Fn(A) -> bool + 'a,
	{
		Self::filter_map(move |a| if func(a.clone()) { Some(a) } else { None }, fa)
	}
}

/// Partitions a data structure based on a function that returns a `Result`.
///
/// Free function version that dispatches to [the type class' associated function][`Filterable::partition_map`].
///
/// ### Type Signature
///
/// `forall a e o f. Filterable f => (a -> Result o e) -> f a -> (f o, f e)`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the filterable structure.
/// * `Func`: The type of the function to apply.
/// * `A`: The type of the elements in the input structure.
/// * `E`: The type of the error values.
/// * `O`: The type of the success values.
///
/// ### Parameters
///
/// * `func`: The function to apply to each element, returning a `Result`.
/// * `fa`: The data structure to partition.
///
/// ### Returns
///
/// A pair of data structures: the first containing the `Ok` values, and the second containing the `Err` values.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::filterable::partition_map;
/// use fp_library::brands::OptionBrand;
/// use fp_library::types::Pair;
///
/// let x = Some(5);
/// let Pair(oks, errs) = partition_map::<OptionBrand, _, _, _, _>(|a| if a > 2 { Ok(a) } else { Err(a) }, x);
/// assert_eq!(oks, Some(5));
/// assert_eq!(errs, None);
/// ```
pub fn partition_map<'a, Brand: Filterable, Func, A: 'a, E: 'a, O: 'a>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Pair<
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
>
where
	Func: Fn(A) -> Result<O, E> + 'a,
{
	Brand::partition_map(func, fa)
}

/// Partitions a data structure based on a predicate.
///
/// Free function version that dispatches to [the type class' associated function][`Filterable::partition`].
///
/// **Note**: The return order is `(satisfied, not_satisfied)`, matching Rust's `Iterator::partition`.
///
/// ### Type Signature
///
/// `forall a f. Filterable f => (a -> bool) -> f a -> (f a, f a)`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the filterable structure.
/// * `Func`: The type of the predicate function.
/// * `A`: The type of the elements in the structure.
///
/// ### Parameters
///
/// * `func`: The predicate function.
/// * `fa`: The data structure to partition.
///
/// ### Returns
///
/// A pair of data structures: the first containing elements that satisfy the predicate, and the second containing elements that do not.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::filterable::partition;
/// use fp_library::brands::OptionBrand;
/// use fp_library::types::Pair;
///
/// let x = Some(5);
/// let Pair(satisfied, not_satisfied) = partition::<OptionBrand, _, _>(|a| a > 2, x);
/// assert_eq!(satisfied, Some(5));
/// assert_eq!(not_satisfied, None);
/// ```
pub fn partition<'a, Brand: Filterable, Func, A: 'a + Clone>(
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

/// Maps a function over a data structure and filters out `None` results.
///
/// Free function version that dispatches to [the type class' associated function][`Filterable::filter_map`].
///
/// ### Type Signature
///
/// `forall a b f. Filterable f => (a -> Option b) -> f a -> f b`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the filterable structure.
/// * `Func`: The type of the function to apply.
/// * `A`: The type of the elements in the input structure.
/// * `B`: The type of the elements in the output structure.
///
/// ### Parameters
///
/// * `func`: The function to apply to each element, returning an `Option`.
/// * `fa`: The data structure to filter and map.
///
/// ### Returns
///
/// A new data structure containing only the values where the function returned `Some`.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::filterable::filter_map;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = filter_map::<OptionBrand, _, _, _>(|a| if a > 2 { Some(a * 2) } else { None }, x);
/// assert_eq!(y, Some(10));
/// ```
pub fn filter_map<'a, Brand: Filterable, Func, A: 'a, B: 'a>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	Func: Fn(A) -> Option<B> + 'a,
{
	Brand::filter_map(func, fa)
}

/// Filters a data structure based on a predicate.
///
/// Free function version that dispatches to [the type class' associated function][`Filterable::filter`].
///
/// ### Type Signature
///
/// `forall a f. Filterable f => (a -> bool) -> f a -> f a`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the filterable structure.
/// * `Func`: The type of the predicate function.
/// * `A`: The type of the elements in the structure.
///
/// ### Parameters
///
/// * `func`: The predicate function.
/// * `fa`: The data structure to filter.
///
/// ### Returns
///
/// A new data structure containing only the elements that satisfy the predicate.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::filterable::filter;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = filter::<OptionBrand, _, _>(|a| a > 2, x);
/// assert_eq!(y, Some(5));
/// ```
pub fn filter<'a, Brand: Filterable, Func, A: 'a + Clone>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
where
	Func: Fn(A) -> bool + 'a,
{
	Brand::filter(func, fa)
}
