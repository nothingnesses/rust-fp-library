use super::lift::Lift;
use crate::hkt::Apply1L1T;

/// A type class for types that support combining two contexts, keeping the second value.
///
/// `ApplySecond` provides the ability to sequence two computations but discard
/// the result of the first computation, keeping only the result of the second.
pub trait ApplySecond: Lift {
	/// Combines two contexts, keeping the value from the second context.
	///
	/// # Type Signature
	///
	/// `forall a b. ApplySecond f => (f a, f b) -> f b`
	///
	/// # Parameters
	///
	/// * `fa`: The first context.
	/// * `fb`: The second context.
	///
	/// # Returns
	///
	/// The second context.
	///
	/// # Examples
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
		fa: Apply1L1T<'a, Self, A>,
		fb: Apply1L1T<'a, Self, B>,
	) -> Apply1L1T<'a, Self, B> {
		Self::lift2(|_, b| b, fa, fb)
	}
}

/// Combines two contexts, keeping the value from the second context.
///
/// Free function version that dispatches to [the type class' associated function][`ApplySecond::apply_second`].
///
/// # Type Signature
///
/// `forall a b. ApplySecond f => (f a, f b) -> f b`
///
/// # Parameters
///
/// * `fa`: The first context.
/// * `fb`: The second context.
///
/// # Returns
///
/// The second context.
///
/// # Examples
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
	fa: Apply1L1T<'a, Brand, A>,
	fb: Apply1L1T<'a, Brand, B>,
) -> Apply1L1T<'a, Brand, B> {
	Brand::apply_second(fa, fb)
}
