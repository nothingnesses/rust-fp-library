use crate::hkt::{Apply_L1_T1_B0l0_Ol0, Kind_L1_T1_B0l0_Ol0};

/// A type class for types that can be lifted.
///
/// `Lift` allows binary functions to be lifted into the context.
pub trait Lift: Kind_L1_T1_B0l0_Ol0 {
	/// Lifts a binary function into the context.
	///
	/// # Type Signature
	///
	/// `forall a b c. Lift f => ((a, b) -> c, f a, f b) -> f c`
	///
	/// # Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first context.
	/// * `fb`: The second context.
	///
	/// # Returns
	///
	/// A new context containing the result of applying the function.
	///
	/// # Examples
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
	fn lift2<'a, A, B, C, F>(
		f: F,
		fa: Apply_L1_T1_B0l0_Ol0<'a, Self, A>,
		fb: Apply_L1_T1_B0l0_Ol0<'a, Self, B>,
	) -> Apply_L1_T1_B0l0_Ol0<'a, Self, C>
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
/// # Type Signature
///
/// `forall a b c. Lift f => ((a, b) -> c, f a, f b) -> f c`
///
/// # Parameters
///
/// * `f`: The binary function to apply.
/// * `fa`: The first context.
/// * `fb`: The second context.
///
/// # Returns
///
/// A new context containing the result of applying the function.
///
/// # Examples
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
pub fn lift2<'a, Brand: Lift, A, B, C, F>(
	f: F,
	fa: Apply_L1_T1_B0l0_Ol0<'a, Brand, A>,
	fb: Apply_L1_T1_B0l0_Ol0<'a, Brand, B>,
) -> Apply_L1_T1_B0l0_Ol0<'a, Brand, C>
where
	F: Fn(A, B) -> C + 'a,
	A: Clone + 'a,
	B: Clone + 'a,
	C: 'a,
{
	Brand::lift2(f, fa, fb)
}
