//! Functor type class.
//!
//! This module defines the [`Functor`] trait, which represents types that can be mapped over.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(5);
//! let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```

use crate::{Apply, kinds::*};

/// A type class for types that can be mapped over.
///
/// A `Functor` represents a context or container that allows functions to be applied
/// to values within that context without altering the structure of the context itself.
///
/// ### Laws
///
/// `Functor` instances must satisfy the following laws:
/// * Identity: `map(identity, fa) = fa`.
/// * Composition: `map(compose(f, g), fa) = map(f, map(g, fa))`.
pub trait Functor: Kind_cdc7cd43dac7585f {
	/// Maps a function over the values in the functor context.
	///
	/// This method applies a function to the value(s) inside the functor context, producing a new functor context with the transformed value(s).
	///
	/// ### Type Signature
	///
	/// `forall f b a. Functor f => (a -> b, f a) -> f b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result(s) of applying the function.
	/// * `A`: The type of the value(s) inside the functor.
	/// * `F`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the value(s) inside the functor.
	/// * `fa`: The functor instance containing the value(s).
	///
	/// ### Returns
	///
	/// A new functor instance containing the result(s) of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(5);
	/// let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
	/// assert_eq!(y, Some(10));
	/// ```
	fn map<'a, B: 'a, A: 'a, F>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> B + 'a;
}

/// Maps a function over the values in the functor context.
///
/// Free function version that dispatches to [the type class' associated function][`Functor::map`].
///
/// ### Type Signature
///
/// `forall f b a. Functor f => (a -> b, f a) -> f b`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the functor.
/// * `B`: The type of the result(s) of applying the function.
/// * `A`: The type of the value(s) inside the functor.
/// * `F`: The type of the function to apply.
///
/// ### Parameters
///
/// * `f`: The function to apply to the value(s) inside the functor.
/// * `fa`: The functor instance containing the value(s).
///
/// ### Returns
///
/// A new functor instance containing the result(s) of applying the function.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(5);
/// let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
/// assert_eq!(y, Some(10));
/// ```
pub fn map<'a, Brand: Functor, B: 'a, A: 'a, F>(
	f: F,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	F: Fn(A) -> B + 'a,
{
	Brand::map::<B, A, F>(f, fa)
}
