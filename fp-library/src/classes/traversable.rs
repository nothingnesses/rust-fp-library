//! Data structures that can be traversed, accumulating results in an applicative context.
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
//! let y = traverse::<OptionBrand, _, _, OptionBrand, _>(|a| Some(a * 2), x);
//! assert_eq!(y, Some(Some(10)));
//! ```

use {
	super::{Applicative, Foldable, Functor},
	crate::{Apply, functions::identity, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for traversable functors.
///
/// `Traversable` functors can be traversed, which accumulates results and effects in some [`Applicative`] context.
pub trait Traversable: Functor + Foldable {
	/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
	///
	/// The default implementation is defined in terms of [`sequence`] and [`map`](crate::functions::map).
	///
	/// **Note**: This default implementation may be less efficient than a direct implementation because it performs two passes:
	/// first mapping the function to create an intermediate structure of computations, and then sequencing that structure.
	/// A direct implementation can often perform the traversal in a single pass without allocating an intermediate container.
	/// Types should provide their own implementation if possible.
	#[document_signature]
	///
	#[document_type_parameters(
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context.",
		"The type of the function to apply."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning a value in an applicative context.",
		"The traversable structure."
	)]
	///
	/// ### Returns
	///
	/// The traversable structure wrapped in the applicative context.
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
	/// let y = traverse::<OptionBrand, _, _, OptionBrand, _>(|a| Some(a * 2), x);
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	fn traverse<A: Clone, B: Clone, F: Applicative, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
	) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)>)
	where
		A: 'static,
		B: 'static,
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<T>; )>::Of<B>) + 'static,
		Apply!(<Self as Kind!( type Of<T>; )>::Of<B>): Clone,
		Apply!(<F as Kind!( type Of<T>; )>::Of<B>): Clone,
	{
		Self::sequence::<B, F>(Self::map::<A, Apply!(<F as Kind!( type Of<T>; )>::Of<B>), Func>(func, ta))
	}

	/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
	///
	/// The default implementation is defined in terms of [`traverse`] and [`identity`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]
	///
	#[document_parameters("The traversable structure containing values in an applicative context.")]
	///
	/// ### Returns
	///
	/// The traversable structure wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(Some(5));
	/// let y = sequence::<OptionBrand, _, OptionBrand>(x);
	/// assert_eq!(y, Some(Some(5)));
	/// ```
	fn sequence<A: Clone, F: Applicative>(
		ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<Apply!(<F as Kind!( type Of<T>; )>::Of<A>)>)
	) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<A>)>)
	where
		A: 'static,
		Apply!(<F as Kind!( type Of<T>; )>::Of<A>): Clone + 'static,
		Apply!(<Self as Kind!( type Of<T>; )>::Of<A>): Clone + 'static,
	{
		Self::traverse::<Apply!(<F as Kind!( type Of<T>; )>::Of<A>), A, F, _>(
			identity, ta,
		)
	}
}

/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::traverse`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the traversable structure.",
	"The type of the elements in the traversable structure.",
	"The type of the elements in the resulting traversable structure.",
	"The applicative context.",
	"The type of the function to apply."
)]
///
#[document_parameters(
	"The function to apply to each element, returning a value in an applicative context.",
	"The traversable structure."
)]
///
/// ### Returns
///
/// The traversable structure wrapped in the applicative context.
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
/// let y = traverse::<OptionBrand, _, _, OptionBrand, _>(|a| Some(a * 2), x);
/// assert_eq!(y, Some(Some(10)));
/// ```
pub fn traverse<Brand: Traversable, A: Clone, B: Clone, F: Applicative, Func>(
	func: Func,
	ta: Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>),
) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Brand as Kind!( type Of<T>; )>::Of<B>)>)
where
	A: 'static,
	B: 'static,
	Func: Fn(A) -> Apply!(<F as Kind!( type Of<T>; )>::Of<B>) + 'static,
	Apply!(<Brand as Kind!( type Of<T>; )>::Of<B>): Clone + 'static,
	Apply!(<F as Kind!( type Of<T>; )>::Of<B>): Clone + 'static,
{
	Brand::traverse::<A, B, F, Func>(func, ta)
}

/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::sequence`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the traversable structure.",
	"The type of the elements in the traversable structure.",
	"The applicative context."
)]
///
#[document_parameters("The traversable structure containing values in an applicative context.")]
///
/// ### Returns
///
/// The traversable structure wrapped in the applicative context.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Some(Some(5));
/// let y = sequence::<OptionBrand, _, OptionBrand>(x);
/// assert_eq!(y, Some(Some(5)));
/// ```
pub fn sequence<Brand: Traversable, A: Clone, F: Applicative>(
	ta: Apply!(<Brand as Kind!( type Of<T>; )>::Of<Apply!(<F as Kind!( type Of<T>; )>::Of<A>)>)
) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>)>)
where
	A: 'static,
	Apply!(<F as Kind!( type Of<T>; )>::Of<A>): Clone + 'static,
	Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>): Clone + 'static,
{
	Brand::sequence::<A, F>(ta)
}
