//! A type class for data structures that can be traversed, accumulating results in an applicative context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{functions::*, brands::*};
//!
//! let x = Some(5);
//! let y = traverse::<OptionBrand, OptionBrand, _, _, _>(|a| Some(a * 2), x);
//! assert_eq!(y, Some(Some(10)));
//! ```

use super::{Applicative, Foldable, Functor};
use crate::{Apply, functions::identity, kinds::*};

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
	///
	/// ### Type Signature
	///
	/// `forall t f b a. (Traversable t, Applicative f) => (a -> f b, t a) -> f (t b)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `B`: The type of the elements in the resulting traversable structure.
	/// * `A`: The type of the elements in the traversable structure.
	/// * `Func`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a value in an applicative context.
	/// * `ta`: The traversable structure.
	///
	/// ### Returns
	///
	/// The traversable structure wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{functions::*, brands::*};
	///
	/// let x = Some(5);
	/// let y = traverse::<OptionBrand, OptionBrand, _, _, _>(|a| Some(a * 2), x);
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	fn traverse<'a, F: Applicative, B: 'a + Clone, A: 'a + Clone, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		Self::sequence::<F, B>(Self::map::<
			A,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			Func,
		>(func, ta))
	}

	/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
	///
	/// The default implementation is defined in terms of [`traverse`] and [`identity`].
	///
	/// ### Type Signature
	///
	/// `forall t f a. (Traversable t, Applicative f) => (t (f a)) -> f (t a)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `A`: The type of the elements in the traversable structure.
	///
	/// ### Parameters
	///
	/// * `ta`: The traversable structure containing values in an applicative context.
	///
	/// ### Returns
	///
	/// The traversable structure wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{functions::*, brands::*};
	///
	/// let x = Some(Some(5));
	/// let y = sequence::<OptionBrand, OptionBrand, _>(x);
	/// assert_eq!(y, Some(Some(5)));
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		Self::traverse::<F, A, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>), _>(
			identity, ta,
		)
	}
}

/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::traverse`].
///
/// ### Type Signature
///
/// `forall t f b a. (Traversable t, Applicative f) => (a -> f b, t a) -> f (t b)`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the traversable structure.
/// * `F`: The applicative context.
/// * `B`: The type of the elements in the resulting traversable structure.
/// * `A`: The type of the elements in the traversable structure.
/// * `Func`: The type of the function to apply.
///
/// ### Parameters
///
/// * `func`: The function to apply to each element, returning a value in an applicative context.
/// * `ta`: The traversable structure.
///
/// ### Returns
///
/// The traversable structure wrapped in the applicative context.
///
/// ### Examples
///
/// ```
/// use fp_library::{functions::*, brands::*};
///
/// let x = Some(5);
/// let y = traverse::<OptionBrand, OptionBrand, _, _, _>(|a| Some(a * 2), x);
/// assert_eq!(y, Some(Some(10)));
/// ```
pub fn traverse<'a, Brand: Traversable, F: Applicative, B: 'a + Clone, A: 'a + Clone, Func>(
	func: Func,
	ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
where
	Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
{
	Brand::traverse::<F, B, A, Func>(func, ta)
}

/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::sequence`].
///
/// ### Type Signature
///
/// `forall t f a. (Traversable t, Applicative f) => (t (f a)) -> f (t a)`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the traversable structure.
/// * `F`: The applicative context.
/// * `A`: The type of the elements in the traversable structure.
///
/// ### Parameters
///
/// * `ta`: The traversable structure containing values in an applicative context.
///
/// ### Returns
///
/// The traversable structure wrapped in the applicative context.
///
/// ### Examples
///
/// ```
/// use fp_library::{functions::*, brands::*};
///
/// let x = Some(Some(5));
/// let y = sequence::<OptionBrand, OptionBrand, _>(x);
/// assert_eq!(y, Some(Some(5)));
/// ```
pub fn sequence<'a, Brand: Traversable, F: Applicative, A: 'a + Clone>(
	ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
where
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
{
	Brand::sequence::<F, A>(ta)
}
