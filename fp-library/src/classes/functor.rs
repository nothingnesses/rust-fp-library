use crate::hkt::{Apply_L1_T1_B0l0_Ol0, Kind_L1_T1_B0l0_Ol0};

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
pub trait Functor: Kind_L1_T1_B0l0_Ol0 {
	/// Maps a function over the values in the functor context.
	///
	/// # Type Signature
	///
	/// `forall a b. Functor f => (a -> b, f a) -> f b`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply to the value(s) inside the functor.
	/// * `fa`: The functor instance containing the value(s).
	///
	/// # Returns
	///
	/// A new functor instance containing the result(s) of applying the function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::functor::Functor;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::map(|i| i * 2, x);
	/// assert_eq!(y, Some(10));
	/// ```
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply_L1_T1_B0l0_Ol0<'a, Self, A>,
	) -> Apply_L1_T1_B0l0_Ol0<'a, Self, B>
	where
		F: Fn(A) -> B + 'a;
}

/// Maps a function over the values in the functor context.
///
/// Free function version that dispatches to [the type class' associated function][`Functor::map`].
///
/// # Type Signature
///
/// `forall a b. Functor f => (a -> b, f a) -> f b`
///
/// # Parameters
///
/// * `f`: The function to apply to the value(s) inside the functor.
/// * `fa`: The functor instance containing the value(s).
///
/// # Returns
///
/// A new functor instance containing the result(s) of applying the function.
///
/// # Examples
///
/// ```
/// use fp_library::classes::functor::map;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
/// assert_eq!(y, Some(10));
/// ```
pub fn map<'a, Brand: Functor, A: 'a, B: 'a, F>(
	f: F,
	fa: Apply_L1_T1_B0l0_Ol0<'a, Brand, A>,
) -> Apply_L1_T1_B0l0_Ol0<'a, Brand, B>
where
	F: Fn(A) -> B + 'a,
{
	Brand::map(f, fa)
}
