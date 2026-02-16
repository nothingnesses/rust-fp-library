//! Data structures that can be traversed and filtered simultaneously in an applicative context.
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
//! let y = wither::<OptionBrand, OptionBrand, _, _, _>(
//! 	|a| Some(if a > 2 { Some(a * 2) } else { None }),
//! 	x,
//! );
//! assert_eq!(y, Some(Some(10)));
//! ```

use {
	crate::{
		Apply,
		classes::{Applicative, Filterable, Traversable},
		kinds::*,
	},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
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
	#[document_signature]
	///
	#[document_type_parameters(
		"The applicative context.",
		"The type of the elements in the input structure.",
		"The type of the error values.",
		"The type of the success values.",
		"The type of the function to apply."
	)]
	///
	#[document_parameters(
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
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = wilt::<OptionBrand, OptionBrand, _, _, _, _>(
	/// 	|a| Some(if a > 2 { Ok(a) } else { Err(a) }),
	/// 	x,
	/// );
	/// assert_eq!(y, Some((None, Some(5))));
	/// ```
	fn wilt<M: Applicative, A: Clone, E: Clone, O: Clone, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
	) -> Apply!(<M as Kind!( type Of<T>; )>::Of<(
		Apply!(<Self as Kind!( type Of<T>; )>::Of<E>),
		Apply!(<Self as Kind!( type Of<T>; )>::Of<O>),
	)>)
	where
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<T>; )>::Of<Result<O, E>>),
		Apply!(<Self as Kind!( type Of<T>; )>::Of<Result<O, E>>): Clone,
		Apply!(<M as Kind!( type Of<T>; )>::Of<Result<O, E>>): Clone,
	{
		M::map(
			|res| Self::separate::<E, O>(res),
			Self::traverse::<A, Result<O, E>, M, Func>(func, ta),
		)
	}

	/// Maps a function over a data structure and filters out [`None`] results in an applicative context.
	///
	/// The default implementation uses [`traverse`](crate::functions::traverse) and [`compact`](crate::functions::compact).
	#[document_signature]
	///
	#[document_type_parameters(
		"The applicative context.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The type of the function to apply."
	)]
	///
	#[document_parameters(
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
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = wither::<OptionBrand, OptionBrand, _, _, _>(
	/// 	|a| Some(if a > 2 { Some(a * 2) } else { None }),
	/// 	x,
	/// );
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	fn wither<M: Applicative, A: Clone, B: Clone, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
	) -> Apply!(<M as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)>)
	where
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<T>; )>::Of<Option<B>>),
		Apply!(<Self as Kind!( type Of<T>; )>::Of<Option<B>>): Clone,
		Apply!(<M as Kind!( type Of<T>; )>::Of<Option<B>>): Clone,
	{
		M::map(|opt| Self::compact(opt), Self::traverse::<A, Option<B>, M, Func>(func, ta))
	}
}

/// Partitions a data structure based on a function that returns a [`Result`] in an applicative context.
///
/// Free function version that dispatches to [the type class' associated function][`Witherable::wilt`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the witherable structure.",
	"The applicative context.",
	"The type of the elements in the input structure.",
	"The type of the error values.",
	"The type of the success values.",
	"The type of the function to apply."
)]
///
#[document_parameters(
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
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// 	types::*,
/// };
///
/// let x = Some(5);
/// let y = wilt::<OptionBrand, OptionBrand, _, _, _, _>(
/// 	|a| Some(if a > 2 { Ok(a) } else { Err(a) }),
/// 	x,
/// );
/// assert_eq!(y, Some((None, Some(5))));
/// ```
pub fn wilt<F: Witherable, M: Applicative, A: Clone, E: Clone, O: Clone, Func>(
	func: Func,
	ta: Apply!(<F as Kind!( type Of<T>; )>::Of<A>),
) -> Apply!(<M as Kind!( type Of<T>; )>::Of<(
	Apply!(<F as Kind!( type Of<T>; )>::Of<E>),
	Apply!(<F as Kind!( type Of<T>; )>::Of<O>),
)>)
where
	Func: Fn(A) -> Apply!(<M as Kind!( type Of<T>; )>::Of<Result<O, E>>),
	Apply!(<F as Kind!( type Of<T>; )>::Of<Result<O, E>>): Clone,
	Apply!(<M as Kind!( type Of<T>; )>::Of<Result<O, E>>): Clone,
{
	F::wilt::<M, A, E, O, Func>(func, ta)
}

/// Maps a function over a data structure and filters out [`None`] results in an applicative context.
///
/// Free function version that dispatches to [the type class' associated function][`Witherable::wither`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the witherable structure.",
	"The applicative context.",
	"The type of the elements in the input structure.",
	"The type of the elements in the output structure.",
	"The type of the function to apply."
)]
///
#[document_parameters(
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
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Some(5);
/// let y = wither::<OptionBrand, OptionBrand, _, _, _>(
/// 	|a| Some(if a > 2 { Some(a * 2) } else { None }),
/// 	x,
/// );
/// assert_eq!(y, Some(Some(10)));
/// ```
pub fn wither<F: Witherable, M: Applicative, A: Clone, B: Clone, Func>(
	func: Func,
	ta: Apply!(<F as Kind!( type Of<T>; )>::Of<A>),
) -> Apply!(<M as Kind!( type Of<T>; )>::Of<Apply!(<F as Kind!( type Of<T>; )>::Of<B>)>)
where
	Func: Fn(A) -> Apply!(<M as Kind!( type Of<T>; )>::Of<Option<B>>),
	Apply!(<F as Kind!( type Of<T>; )>::Of<Option<B>>): Clone,
	Apply!(<M as Kind!( type Of<T>; )>::Of<Option<B>>): Clone,
{
	F::wither::<M, A, B, Func>(func, ta)
}
