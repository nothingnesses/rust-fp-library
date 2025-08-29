use crate::{
	classes::{ClonableFn, clonable_fn::ApplyFn},
	hkt::{Apply0L1T, Kind0L1T},
};

/// A type class for types that can be mapped over.
///
/// A `Functor` represents a context or container that allows functions to be applied
/// to values within that context without altering the structure of the context itself.
///
/// # Laws
///
/// Functors must satisfy the following laws:
/// * Identity: `map(identity) = identity`.
/// * Composition: `map(f . g) = map(f) . map(g)`.
pub trait Functor: Kind0L1T {
	/// Maps a function over the values in the functor context.
	///
	/// # Type Signature
	///
	/// `forall a b. Functor f => (a -> b) -> f a -> f b`
	///
	/// # Parameters
	///
	/// * `f`: A function to apply to the values within the functor context.
	/// * `fa`: A functor containing values of type `A`.
	///
	/// # Returns
	///
	/// A functor containing values of type `B`.
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>>;
}

/// Maps a function over the values in the functor context.
///
/// Free function version that dispatches to [the type class' associated function][`Functor::map`].
///
/// # Type Signature
///
/// `forall a b. Functor f => (a -> b) -> f a -> f b`
///
/// # Parameters
///
/// * `f`: A function to apply to the values within the functor context.
/// * `fa`: A functor containing values of type `A`.
///
/// # Returns
///
/// A functor containing values of type `B`.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::map};
/// use std::rc::Rc;
///
/// assert_eq!(map::<RcFnBrand, OptionBrand, _, _>(Rc::new(|x: i32| x * 2))(Some(5)), Some(10));
/// ```
pub fn map<'a, ClonableFnBrand: 'a + ClonableFn, Brand: Functor + ?Sized, A: 'a, B: 'a>(
	f: ApplyFn<'a, ClonableFnBrand, A, B>
) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, A>, Apply0L1T<Brand, B>> {
	Brand::map::<ClonableFnBrand, _, _>(f)
}
