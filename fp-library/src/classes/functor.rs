//! Functor type class.
//!
//! This module defines the [`Functor`] trait, which represents types that can be mapped over.

use crate::{Apply, kinds::*};

/// A type class for types that can be mapped over.
///
/// A `Functor` represents a context or container that allows functions to be applied
/// to values within that context without altering the structure of the context itself.
///
/// # Laws
///
/// `Functor` instances must satisfy the following laws:
/// * Identity: `map(identity, fa) = fa`.
/// * Composition: `map(compose(f, g), fa) = map(f, map(g, fa))`.
pub trait Functor: Kind_c3c3610c70409ee6 {
	/// Maps a function over the values in the functor context.
	///
	/// This method applies a function to the value(s) inside the functor context, producing a new functor context with the transformed value(s).
	///
	/// ### Type Signature
	///
	/// `forall a b. Functor f => (a -> b, f a) -> f b`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the function to apply.
	/// * `A`: The type of the value(s) inside the functor.
	/// * `B`: The type of the result(s) of applying the function.
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
	/// use fp_library::classes::functor::Functor;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::map(|i| i * 2, x);
	/// assert_eq!(y, Some(10));
	/// ```
	fn map<'a, F, A: 'a, B: 'a>(
		f: F,
		fa: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
		),
	) -> Apply!(
		brand: Self,
		signature: ('a, B: 'a) -> 'a,
	)
	where
		F: Fn(A) -> B + 'a;
}

/// Maps a function over the values in the functor context.
///
/// Free function version that dispatches to [the type class' associated function][`Functor::map`].
///
/// ### Type Signature
///
/// `forall a b. Functor f => (a -> b, f a) -> f b`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the functor.
/// * `F`: The type of the function to apply.
/// * `A`: The type of the value(s) inside the functor.
/// * `B`: The type of the result(s) of applying the function.
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
/// use fp_library::classes::functor::map;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
/// assert_eq!(y, Some(10));
/// ```
pub fn map<'a, Brand: Functor, F, A: 'a, B: 'a>(
	f: F,
	fa: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
	),
) -> Apply!(
	brand: Brand,
	signature: ('a, B: 'a) -> 'a,
)
where
	F: Fn(A) -> B + 'a,
{
	Brand::map(f, fa)
}
