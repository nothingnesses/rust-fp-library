//! Lift type class.
//!
//! This module defines the [`Lift`] trait, which allows binary functions to be lifted into a context.

use crate::{Apply, kinds::*};

/// A type class for types that can be lifted.
///
/// `Lift` allows binary functions to be lifted into the context.
pub trait Lift: Kind_cdc7cd43dac7585f {
	/// Lifts a binary function into the context.
	///
	/// This method lifts a binary function to operate on values within the context.
	///
	/// ### Type Signature
	///
	/// `forall a b c. Lift f => ((a, b) -> c, f a, f b) -> f c`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the binary function.
	/// * `A`: The type of the first value.
	/// * `B`: The type of the second value.
	/// * `C`: The type of the result.
	///
	/// ### Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first context.
	/// * `fb`: The second context.
	///
	/// ### Returns
	///
	/// A new context containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::lift::Lift;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(1);
	/// let y = Some(2);
	/// let z = OptionBrand::lift2(|a, b| a + b, x, y);
	/// assert_eq!(z, Some(3));
	/// ```
	fn lift2<'a, F, A, B, C>(
		f: F,
		fa: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
		),
		fb: Apply!(
			brand: Self,
			signature: ('a, B: 'a) -> 'a,
		),
	) -> Apply!(
		brand: Self,
		signature: ('a, C: 'a) -> 'a,
	)
	where
		F: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a;
}

/// Lifts a binary function into the context.
///
/// Free function version that dispatches to [the type class' associated function][`Lift::lift2`].
///
/// ### Type Signature
///
/// `forall a b c. Lift f => ((a, b) -> c, f a, f b) -> f c`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the context.
/// * `F`: The type of the binary function.
/// * `A`: The type of the first value.
/// * `B`: The type of the second value.
/// * `C`: The type of the result.
///
/// ### Parameters
///
/// * `f`: The binary function to apply.
/// * `fa`: The first context.
/// * `fb`: The second context.
///
/// ### Returns
///
/// A new context containing the result of applying the function.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::lift::lift2;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(1);
/// let y = Some(2);
/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, x, y);
/// assert_eq!(z, Some(3));
/// ```
pub fn lift2<'a, Brand: Lift, F, A, B, C>(
	f: F,
	fa: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
	),
	fb: Apply!(
		brand: Brand,
		signature: ('a, B: 'a) -> 'a,
	),
) -> Apply!(
	brand: Brand,
	signature: ('a, C: 'a) -> 'a,
)
where
	F: Fn(A, B) -> C + 'a,
	A: Clone + 'a,
	B: Clone + 'a,
	C: 'a,
{
	Brand::lift2(f, fa, fb)
}
