use crate::hkt::Apply1L2T;
use super::semigroupoid::Semigroupoid;

/// A type class for categories.
///
/// `Category` extends [`Semigroupoid`] with an identity element.
///
/// # Laws
///
/// `Category` instances must satisfy the identity law:
/// * Identity: `compose(identity, p) = compose(p, identity)`.
pub trait Category: Semigroupoid {
    /// Returns the identity morphism.
    fn identity<'a, A>() -> Apply1L2T<'a, Self, A, A>;
}

/// Returns the identity morphism.
///
/// Free function version that dispatches to [the type class' associated function][`Category::identity`].
pub fn identity<'a, Brand: Category, A>() -> Apply1L2T<'a, Brand, A, A> {
    Brand::identity()
}
