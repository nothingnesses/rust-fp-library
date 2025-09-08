use crate::{classes::Semigroupoid, hkt::Apply1L2T};

/// A type class for categories.
///
/// `Category` extends [`Semigroupoid`] with an identity element.
///
/// # Laws
///
/// `Category` instances must satisfy the identity law:
/// * Identity: `compose(identity)(p) = compose(p)(identity)`.
pub trait Category: Semigroupoid {
	/// Returns the identity morphism.
	///
	/// # Type Signature
	///
	/// `forall a. Category c => () -> c a a`
	///
	/// # Returns
	///
	/// The identity morphism.
	fn identity<'a, A: 'a>() -> Apply1L2T<'a, Self, A, A>;
}

/// Returns the identity morphism.
///
/// Free function version that dispatches to [the type class' associated function][`Category::identity`].
///
/// # Type Signature
///
/// `forall a. Category c => () -> c a a`
///
/// # Returns
///
/// The identity morphism.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::RcFnBrand, functions::category_identity};
///
/// assert_eq!(category_identity::<RcFnBrand, _>()(()), ());
/// ```
pub fn category_identity<'a, Brand: Category, A: 'a>() -> Apply1L2T<'a, Brand, A, A> {
	Brand::identity::<'a, _>()
}
