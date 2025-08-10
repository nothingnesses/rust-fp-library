use crate::{
	aliases::ClonableFn,
	hkt::{Apply, Kind},
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
pub trait Functor {
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
	fn map<'a, A, B>(f: ClonableFn<'a, A, B>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>;
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
pub fn map<'a, Brand, A, B>(
	f: ClonableFn<'a, A, B>
) -> impl Fn(Apply<Brand, (A,)>) -> Apply<Brand, (B,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + Functor + ?Sized,
{
	Brand::map(f)
}
