//! ApplySecond type class.
//!
//! This module defines the [`ApplySecond`] trait, which provides the ability to sequence two computations
//! but discard the result of the first computation, keeping only the result of the second.

use super::lift::Lift;
use crate::{Apply, kinds::*};

/// A type class for types that support combining two contexts, keeping the second value.
///
/// `ApplySecond` provides the ability to sequence two computations but discard
/// the result of the first computation, keeping only the result of the second.
pub trait ApplySecond: Lift {
	/// Combines two contexts, keeping the value from the second context.
	///
	/// This function sequences two computations and discards the result of the first computation, keeping only the result of the second.
	///
	/// ### Type Signature
	///
	/// `forall a b. ApplySecond f => (f a, f b) -> f b`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value in the first context.
	/// * `B`: The type of the value in the second context.
	///
	/// ### Parameters
	///
	/// * `fa`: The first context.
	/// * `fb`: The second context.
	///
	/// ### Returns
	///
	/// The second context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::apply_second::ApplySecond;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = Some(10);
	/// let z = OptionBrand::apply_second(x, y);
	/// assert_eq!(z, Some(10));
	/// ```
	fn apply_second<'a, A: 'a + Clone, B: 'a + Clone>(
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
		signature: ('a, B: 'a) -> 'a,
	) {
		Self::lift2(|_, b| b, fa, fb)
	}
}

/// Combines two contexts, keeping the value from the second context.
///
/// Free function version that dispatches to [the type class' associated function][`ApplySecond::apply_second`].
///
/// ### Type Signature
///
/// `forall a b. ApplySecond f => (f a, f b) -> f b`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the context.
/// * `A`: The type of the value in the first context.
/// * `B`: The type of the value in the second context.
///
/// ### Parameters
///
/// * `fa`: The first context.
/// * `fb`: The second context.
///
/// ### Returns
///
/// The second context.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::apply_second::apply_second;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = Some(10);
/// let z = apply_second::<OptionBrand, _, _>(x, y);
/// assert_eq!(z, Some(10));
/// ```
pub fn apply_second<'a, Brand: ApplySecond, A: 'a + Clone, B: 'a + Clone>(
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
	signature: ('a, B: 'a) -> 'a,
) {
	Brand::apply_second(fa, fb)
}
