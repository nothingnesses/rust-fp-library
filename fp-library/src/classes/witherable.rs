//! Data structures that can be traversed and filtered simultaneously in an applicative context.
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
};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

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
	#[hm_signature(Witherable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The applicative context.",
		"The type of the elements in the input structure.",
		"The type of the error values.",
		"The type of the success values.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The function to apply to each element, returning a [`Result`] in an applicative context.",
		"The data structure to partition."
	)]
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
	/// assert_eq!(y, Some((None, Some(5))));
	/// ```
	fn wilt<'a, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		(
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		),
	>)
	where
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
	{
		M::map(
			|res| Self::separate::<E, O>(res),
			Self::traverse::<A, Result<O, E>, M, Func>(func, ta),
		)
	}

	/// Maps a function over a data structure and filters out [`None`] results in an applicative context.
	///
	/// The default implementation uses [`traverse`](crate::functions::traverse) and [`compact`](crate::functions::compact).
	///
	/// ### Type Signature
	///
	#[hm_signature(Witherable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The applicative context.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The function to apply to each element, returning an [`Option`] in an applicative context.",
		"The data structure to filter and map."
	)]
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
#[hm_signature(Witherable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the witherable structure.",
	"The applicative context.",
	"The type of the elements in the input structure.",
	"The type of the error values.",
	"The type of the success values.",
	"The type of the function to apply."
)]
///
/// ### Parameters
///
#[doc_params(
	"The function to apply to each element, returning a [`Result`] in an applicative context.",
	"The data structure to partition."
)]
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
/// assert_eq!(y, Some((None, Some(5))));
/// ```
pub fn wilt<'a, F: Witherable, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone, Func>(
	func: Func,
	ta: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
	'a,
	(
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	),
>)
where
	Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
	Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
{
	F::wilt::<M, A, E, O, Func>(func, ta)
}

/// Maps a function over a data structure and filters out [`None`] results in an applicative context.
///
/// Free function version that dispatches to [the type class' associated function][`Witherable::wither`].
///
/// ### Type Signature
///
#[hm_signature(Witherable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the witherable structure.",
	"The applicative context.",
	"The type of the elements in the input structure.",
	"The type of the elements in the output structure.",
	"The type of the function to apply."
)]
///
/// ### Parameters
///
#[doc_params(
	"The function to apply to each element, returning an [`Option`] in an applicative context.",
	"The data structure to filter and map."
)]
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
pub fn wither<'a, F: Witherable, M: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
	func: Func,
	ta: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
	'a,
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
>)
where
	Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
	Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
{
	F::wither::<M, A, B, Func>(func, ta)
}
