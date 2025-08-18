use crate::{
	aliases::ArcFn,
	hkt::{
		// Apply,
		// Kind,
		Apply1,
		Kind1,
	},
};

/// A typeclass for types that can be mapped over.
///
/// A `Functor` represents a context or container that allows functions to be applied
/// to values within that context without altering the structure of the context itself.
///
/// # Laws
///
/// Functors must satisfy the following laws:
/// * Identity: `map(identity) = identity`.
/// * Composition: `map(f . g) = map(f) . map(g)`.
pub trait Functor: Kind1 {
	/// Maps a function over the values in the functor context.
	///
	/// # Type Signature
	///
	/// `forall f a b. Functor f => (a -> b) -> f a -> f b`
	///
	/// # Parameters
	///
	/// * `f`: A function to apply to the values within the functor context.
	/// * `fa`: A functor containing values of type `A`.
	///
	/// # Returns
	///
	/// A functor containing values of type `B`.
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>>;
}

/// Maps a function over the values in the functor context.
///
/// Free function version that dispatches to [the typeclass' associated function][`Functor::map`].
///
/// # Type Signature
///
/// `forall f a b. Functor f => (a -> b) -> f a -> f b`
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
/// use fp_library::{brands::OptionBrand, functions::map};
/// use std::sync::Arc;
///
/// assert_eq!(map::<OptionBrand, _, _>(Arc::new(|x: i32| x * 2))(Some(5)), Some(10));
/// ```
pub fn map<'a, Brand: Functor + ?Sized, A: 'a, B: 'a>(
	f: ArcFn<'a, A, B>
) -> ArcFn<'a, Apply1<Brand, A>, Apply1<Brand, B>> {
	Brand::map(f)
}
