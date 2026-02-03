//! Data structures that can be traversed, accumulating results in an applicative context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{functions::*, brands::*};
//!
//! let x = Some(5);
//! let y = traverse::<OptionBrand, _, _, OptionBrand, _>(|a| Some(a * 2), x);
//! assert_eq!(y, Some(Some(10)));
//! ```

use super::{Applicative, Foldable, Functor};
use crate::{Apply, functions::identity, kinds::*};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

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
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
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
	/// use fp_library::{functions::*, brands::*};
	///
	/// let x = Some(5);
	/// let y = traverse::<OptionBrand, _, _, OptionBrand, _>(|a| Some(a * 2), x);
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		Self::sequence::<B, F>(Self::map::<
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
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The traversable structure containing values in an applicative context.")]
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
	/// let y = sequence::<OptionBrand, _, OptionBrand>(x);
	/// assert_eq!(y, Some(Some(5)));
	/// ```
	fn sequence<'a, A: 'a + Clone, F: Applicative>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		Self::traverse::<Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>), A, F, _>(
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
#[hm_signature]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the traversable structure.",
	"The type of the elements in the traversable structure.",
	"The type of the elements in the resulting traversable structure.",
	"The applicative context.",
	"The type of the function to apply."
)]
///
/// ### Parameters
///
#[doc_params(
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
/// use fp_library::{functions::*, brands::*};
///
/// let x = Some(5);
/// let y = traverse::<OptionBrand, _, _, OptionBrand, _>(|a| Some(a * 2), x);
/// assert_eq!(y, Some(Some(10)));
/// ```
pub fn traverse<'a, Brand: Traversable, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
	func: Func,
	ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
where
	Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
{
	Brand::traverse::<A, B, F, Func>(func, ta)
}

/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::sequence`].
///
/// ### Type Signature
///
#[hm_signature]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the traversable structure.",
	"The type of the elements in the traversable structure.",
	"The applicative context."
)]
///
/// ### Parameters
///
#[doc_params("The traversable structure containing values in an applicative context.")]
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
/// let y = sequence::<OptionBrand, _, OptionBrand>(x);
/// assert_eq!(y, Some(Some(5)));
/// ```
pub fn sequence<'a, Brand: Traversable, A: 'a + Clone, F: Applicative>(
	ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
where
	Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
{
	Brand::sequence::<A, F>(ta)
}
