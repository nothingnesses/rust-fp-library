use crate::{
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::ClonableFn,
};

/// A typeclass for types that can lift values into a context.
///
/// `Pure` provides the ability to lift a value into a context without
/// adding any additional structure or effects.
pub trait Pure: Kind0L1T {
	/// Lifts a value into the context.
	///
	/// # Type Signature
	///
	/// `forall f a. Pure f => a -> f a`
	///
	/// # Parameters
	///
	/// * `a`: A value to be lifted into the context.
	///
	/// # Returns
	///
	/// The value wrapped in the context.
	fn pure<ClonableFnBrand: ClonableFn, A: Clone>(a: A) -> Apply0L1T<Self, A>;
}

/// Lifts a value into the context.
///
/// Free function version that dispatches to [the typeclass' associated function][`Pure::pure`].
///
/// # Type Signature
///
/// `forall f a. Pure f => a -> f a`
///
/// # Parameters
///
/// * `a`: A value to be lifted into the context.
///
/// # Returns
///
/// The value wrapped in the context.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::pure};
///
/// assert_eq!(pure::<RcFnBrand, OptionBrand, _>(5), Some(5));
/// ```
pub fn pure<ClonableFnBrand: ClonableFn, Brand: Pure, A: Clone>(a: A) -> Apply0L1T<Brand, A> {
	Brand::pure::<ClonableFnBrand, _>(a)
}
