use crate::hkt::{Apply1L1T, Kind1L1T};

/// A type class for types that can be constructed from a single value.
///
/// `Pointed` represents a context that can be initialized with a value.
pub trait Pointed: Kind1L1T {
	/// The value wrapped in the context.
	///
	/// # Type Signature
	///
	/// `forall a. Pointed f => a -> f a`
	///
	/// # Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// # Returns
	///
	/// A new context containing the value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::pointed::Pointed;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = OptionBrand::pure(5);
	/// assert_eq!(x, Some(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A>;
}

/// The value wrapped in the context.
///
/// Free function version that dispatches to [the type class' associated function][`Pointed::pure`].
///
/// # Type Signature
///
/// `forall a. Pointed f => a -> f a`
///
/// # Parameters
///
/// * `a`: The value to wrap.
///
/// # Returns
///
/// A new context containing the value.
///
/// # Examples
///
/// ```
/// use fp_library::classes::pointed::pure;
/// use fp_library::brands::OptionBrand;
///
/// let x = pure::<OptionBrand, _>(5);
/// assert_eq!(x, Some(5));
/// ```
pub fn pure<'a, Brand: Pointed, A: 'a>(a: A) -> Apply1L1T<'a, Brand, A> {
	Brand::pure(a)
}
