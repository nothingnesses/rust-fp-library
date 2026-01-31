//! A type class for data structures that can be traversed and filtered in an applicative context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{functions::*, brands::*};
//!
//! let x = Some(5);
//! let y = wither::<OptionBrand, OptionBrand, _, _, _>(|a| Some(if a > 2 { Some(a * 2) } else { None }), x);
//! assert_eq!(y, Some(Some(10)));
//! ```

use crate::{
	Apply,
	classes::{Applicative, Filterable, Traversable},
	kinds::*,
	types::Pair,
};

/// A type class for data structures that can be traversed and filtered.
///
/// `Witherable` extends [`Filterable`] and [`Traversable`], adding methods for:
/// *   `wither`: Effectful `filter_map`.
/// *   `wilt`: Effectful `partition_map`.
///
/// ### Minimal Implementation
///
/// A minimal implementation of `Witherable` requires no specific method implementations, as all methods have default implementations based on [`Traversable`] and [`Compactable`](crate::classes::compactable::Compactable).
///
/// However, it is recommended to implement [`Witherable::wilt`] and [`Witherable::wither`] to avoid the intermediate structure created by the default implementations (which use [`traverse`](crate::functions::traverse) followed by [`separate`](crate::functions::separate) or [`compact`](crate::functions::compact)).
pub trait Witherable: Filterable + Traversable {
	/// Partitions a data structure based on a function that returns a [`Result`] in an applicative context.
	///
	/// The default implementation uses [`traverse`](crate::functions::traverse) and [`separate`](crate::functions::separate).
	///
	/// ### Type Signature
	///
	/// `forall f m o e a. (Witherable f, Applicative m) => (a -> m (Result o e), f a) -> m (Pair (f o) (f e))`
	///
	/// ### Type Parameters
	///
	/// * `M`: The applicative context.
	/// * `O`: The type of the success values.
	/// * `E`: The type of the error values.
	/// * `A`: The type of the elements in the input structure.
	/// * `Func`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a [`Result`] in an applicative context.
	/// * `ta`: The data structure to partition.
	///
	/// ### Returns
	///
	/// The partitioned data structure wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{functions::*, brands::*, types::*};
	///
	/// let x = Some(5);
	/// let y = wilt::<OptionBrand, OptionBrand, _, _, _, _>(|a| Some(if a > 2 { Ok(a) } else { Err(a) }), x);
	/// assert_eq!(y, Some(Pair(Some(5), None)));
	/// ```
	fn wilt<'a, M: Applicative, A: 'a + Clone, O: 'a + Clone, E: 'a + Clone, Func>(
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
		M::map(|res| Self::separate(res), Self::traverse::<A, Result<O, E>, M, Func>(func, ta))
	}

	/// Maps a function over a data structure and filters out [`None`] results in an applicative context.
	///
	/// The default implementation uses [`traverse`](crate::functions::traverse) and [`compact`](crate::functions::compact).
	///
	/// ### Type Signature
	///
	/// `forall f m b a. (Witherable f, Applicative m) => (a -> m (Option b), f a) -> m (f b)`
	///
	/// ### Type Parameters
	///
	/// * `M`: The applicative context.
	/// * `B`: The type of the elements in the output structure.
	/// * `A`: The type of the elements in the input structure.
	/// * `Func`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning an [`Option`] in an applicative context.
	/// * `ta`: The data structure to filter and map.
	///
	/// ### Returns
	///
	/// The filtered and mapped data structure wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{functions::*, brands::*};
	///
	/// let x = Some(5);
	/// let y = wither::<OptionBrand, OptionBrand, _, _, _>(|a| Some(if a > 2 { Some(a * 2) } else { None }), x);
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	fn wither<'a, M: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
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
		M::map(|opt| Self::compact(opt), Self::traverse::<A, Option<B>, M, Func>(func, ta))
	}
}

/// Partitions a data structure based on a function that returns a [`Result`] in an applicative context.
///
/// Free function version that dispatches to [the type class' associated function][`Witherable::wilt`].
///
/// ### Type Signature
///
/// `forall f m o e a. (Witherable f, Applicative m) => (a -> m (Result o e), f a) -> m (Pair (f o) (f e))`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the witherable structure.
/// * `M`: The applicative context.
/// * `O`: The type of the success values.
/// * `E`: The type of the error values.
/// * `A`: The type of the elements in the input structure.
/// * `Func`: The type of the function to apply.
///
/// ### Parameters
///
/// * `func`: The function to apply to each element, returning a [`Result`] in an applicative context.
/// * `ta`: The data structure to partition.
///
/// ### Returns
///
/// The partitioned data structure wrapped in the applicative context.
///
/// ### Examples
///
/// ```
/// use fp_library::{functions::*, brands::*, types::*};
///
/// let x = Some(5);
/// let y = wilt::<OptionBrand, OptionBrand, _, _, _, _>(|a| Some(if a > 2 { Ok(a) } else { Err(a) }), x);
/// assert_eq!(y, Some(Pair(Some(5), None)));
/// ```
pub fn wilt<
	'a,
	Brand: Witherable,
	M: Applicative,
	A: 'a + Clone,
	O: 'a + Clone,
	E: 'a + Clone,
	Func,
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
	Brand::wilt::<M, A, O, E, Func>(func, ta)
}

/// Maps a function over a data structure and filters out [`None`] results in an applicative context.
///
/// Free function version that dispatches to [the type class' associated function][`Witherable::wither`].
///
/// ### Type Signature
///
/// `forall f m b a. (Witherable f, Applicative m) => (a -> m (Option b), f a) -> m (f b)`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the witherable structure.
/// * `M`: The applicative context.
/// * `B`: The type of the elements in the output structure.
/// * `A`: The type of the elements in the input structure.
/// * `Func`: The type of the function to apply.
///
/// ### Parameters
///
/// * `func`: The function to apply to each element, returning an [`Option`] in an applicative context.
/// * `ta`: The data structure to filter and map.
///
/// ### Returns
///
/// The filtered and mapped data structure wrapped in the applicative context.
///
/// ### Examples
///
/// ```
/// use fp_library::{functions::*, brands::*};
///
/// let x = Some(5);
/// let y = wither::<OptionBrand, OptionBrand, _, _, _>(|a| Some(if a > 2 { Some(a * 2) } else { None }), x);
/// assert_eq!(y, Some(Some(10)));
/// ```
pub fn wither<'a, Brand: Witherable, M: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
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
	Brand::wither::<M, A, B, Func>(func, ta)
}
