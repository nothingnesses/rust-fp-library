//! Witherable type class.
//!
//! This module defines the [`Witherable`] trait, which represents data structures that can be traversed and filtered in an applicative context.

use crate::{
	Apply,
	classes::{applicative::Applicative, filterable::Filterable, traversable::Traversable},
	kinds::*,
	types::Pair,
};

/// A type class for data structures that can be traversed and filtered.
///
/// `Witherable` extends [`Filterable`] and [`Traversable`], adding methods for:
/// *   `wither`: Effectful `filter_map`.
/// *   `wilt`: Effectful `partition_map`.
pub trait Witherable: Filterable + Traversable {
	/// Partitions a data structure based on a function that returns a `Result` in an applicative context.
	///
	/// The default implementation uses [`Traversable::traverse`] and [`Compactable::separate`].
	///
	/// ### Type Signature
	///
	/// `forall a e o m f. (Witherable f, Applicative m) => (a -> m (Result o e)) -> f a -> m (f o, f e)`
	///
	/// ### Type Parameters
	///
	/// * `Func`: The type of the function to apply.
	/// * `M`: The applicative context.
	/// * `A`: The type of the elements in the input structure.
	/// * `E`: The type of the error values.
	/// * `O`: The type of the success values.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a `Result` in an applicative context.
	/// * `ta`: The data structure to partition.
	///
	/// ### Returns
	///
	/// The partitioned data structure wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::witherable::Witherable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::Pair;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::wilt::<_, OptionBrand, _, _, _>(|a| Some(if a > 2 { Ok(a) } else { Err(a) }), x);
	/// assert_eq!(y, Some(Pair(Some(5), None)));
	/// ```
	fn wilt<'a, Func, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Pair<
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		>,
	>)
	where
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
	{
		M::map(|res| Self::separate(res), Self::traverse::<M, Func, A, Result<O, E>>(func, ta))
	}

	/// Maps a function over a data structure and filters out `None` results in an applicative context.
	///
	/// The default implementation uses [`Traversable::traverse`] and [`Compactable::compact`].
	///
	/// ### Type Signature
	///
	/// `forall a b m f. (Witherable f, Applicative m) => (a -> m (Option b)) -> f a -> m (f b)`
	///
	/// ### Type Parameters
	///
	/// * `Func`: The type of the function to apply.
	/// * `M`: The applicative context.
	/// * `A`: The type of the elements in the input structure.
	/// * `B`: The type of the elements in the output structure.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning an `Option` in an applicative context.
	/// * `ta`: The data structure to filter and map.
	///
	/// ### Returns
	///
	/// The filtered and mapped data structure wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::witherable::Witherable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::wither::<_, OptionBrand, _, _>(|a| Some(if a > 2 { Some(a * 2) } else { None }), x);
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	fn wither<'a, Func, M: Applicative, A: 'a + Clone, B: 'a + Clone>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
	where
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
	{
		M::map(|opt| Self::compact(opt), Self::traverse::<M, Func, A, Option<B>>(func, ta))
	}
}

/// Partitions a data structure based on a function that returns a `Result` in an applicative context.
///
/// Free function version that dispatches to [the type class' associated function][`Witherable::wilt`].
///
/// ### Type Signature
///
/// `forall a e o m f. (Witherable f, Applicative m) => (a -> m (Result o e)) -> f a -> m (f o, f e)`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the witherable structure.
/// * `Func`: The type of the function to apply.
/// * `M`: The applicative context.
/// * `A`: The type of the elements in the input structure.
/// * `E`: The type of the error values.
/// * `O`: The type of the success values.
///
/// ### Parameters
///
/// * `func`: The function to apply to each element, returning a `Result` in an applicative context.
/// * `ta`: The data structure to partition.
///
/// ### Returns
///
/// The partitioned data structure wrapped in the applicative context.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::witherable::wilt;
/// use fp_library::brands::OptionBrand;
/// use fp_library::types::Pair;
///
/// let x = Some(5);
/// let y = wilt::<OptionBrand, _, OptionBrand, _, _, _>(|a| Some(if a > 2 { Ok(a) } else { Err(a) }), x);
/// assert_eq!(y, Some(Pair(Some(5), None)));
/// ```
pub fn wilt<
	'a,
	Brand: Witherable,
	Func,
	M: Applicative,
	A: 'a + Clone,
	E: 'a + Clone,
	O: 'a + Clone,
>(
	func: Func,
	ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
	'a,
	Pair<
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	>,
>)
where
	Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
	Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
{
	Brand::wilt::<_, M, _, _, _>(func, ta)
}

/// Maps a function over a data structure and filters out `None` results in an applicative context.
///
/// Free function version that dispatches to [the type class' associated function][`Witherable::wither`].
///
/// ### Type Signature
///
/// `forall a b m f. (Witherable f, Applicative m) => (a -> m (Option b)) -> f a -> m (f b)`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the witherable structure.
/// * `Func`: The type of the function to apply.
/// * `M`: The applicative context.
/// * `A`: The type of the elements in the input structure.
/// * `B`: The type of the elements in the output structure.
///
/// ### Parameters
///
/// * `func`: The function to apply to each element, returning an `Option` in an applicative context.
/// * `ta`: The data structure to filter and map.
///
/// ### Returns
///
/// The filtered and mapped data structure wrapped in the applicative context.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::witherable::wither;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = wither::<OptionBrand, _, OptionBrand, _, _>(|a| Some(if a > 2 { Some(a * 2) } else { None }), x);
/// assert_eq!(y, Some(Some(10)));
/// ```
pub fn wither<'a, Brand: Witherable, Func, M: Applicative, A: 'a + Clone, B: 'a + Clone>(
	func: Func,
	ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
	'a,
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
>)
where
	Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
	Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
{
	Brand::wither::<_, M, _, _>(func, ta)
}
