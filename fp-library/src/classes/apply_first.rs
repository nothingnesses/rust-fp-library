use super::lift::Lift;
use crate::{Apply, kinds::*};

/// A type class for types that support combining two contexts, keeping the first value.
///
/// `ApplyFirst` provides the ability to sequence two computations but discard
/// the result of the second computation, keeping only the result of the first.
pub trait ApplyFirst: Lift {
	/// Combines two contexts, keeping the value from the first context.
	///
	/// # Type Signature
	///
	/// `forall a b. ApplyFirst f => (f a, f b) -> f a`
	///
	/// # Parameters
	///
	/// * `fa`: The first context.
	/// * `fb`: The second context.
	///
	/// # Returns
	///
	/// The first context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::apply_first::ApplyFirst;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = Some(10);
	/// let z = OptionBrand::apply_first(x, y);
	/// assert_eq!(z, Some(5));
	/// ```
	fn apply_first<'a, A: 'a + Clone, B: 'a + Clone>(
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
		signature: ('a, A: 'a) -> 'a,
	) {
		Self::lift2(|a, _| a, fa, fb)
	}
}

/// Combines two contexts, keeping the value from the first context.
///
/// Free function version that dispatches to [the type class' associated function][`ApplyFirst::apply_first`].
///
/// # Type Signature
///
/// `forall a b. ApplyFirst f => (f a, f b) -> f a`
///
/// # Parameters
///
/// * `fa`: The first context.
/// * `fb`: The second context.
///
/// # Returns
///
/// The first context.
///
/// # Examples
///
/// ```
/// use fp_library::classes::apply_first::apply_first;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = Some(10);
/// let z = apply_first::<OptionBrand, _, _>(x, y);
/// assert_eq!(z, Some(5));
/// ```
pub fn apply_first<'a, Brand: ApplyFirst, A: 'a + Clone, B: 'a + Clone>(
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
	signature: ('a, A: 'a) -> 'a,
) {
	Brand::apply_first(fa, fb)
}
