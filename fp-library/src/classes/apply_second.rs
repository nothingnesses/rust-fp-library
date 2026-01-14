use super::lift::Lift;
use crate::{Apply, hkt::Kind_L1_T1_B0l0_Ol0};

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
		fa: Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (A)),
		fb: Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (B)),
	) -> Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (B)) {
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
	fa: Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (A)),
	fb: Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (B)),
) -> Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (B)) {
	Brand::apply_second(fa, fb)
}
