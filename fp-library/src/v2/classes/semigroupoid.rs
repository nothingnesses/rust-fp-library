use crate::hkt::{Apply1L2T, Kind1L2T};

/// A type class for semigroupoids.
///
/// A `Semigroupoid` is a set of objects and composable relationships
/// (morphisms) between them.
///
/// # Laws
///
/// Semigroupoid instances must satisfy the associative law:
/// * Associativity: `compose(p, compose(q, r)) = compose(compose(p, q), r)`.
pub trait Semigroupoid: Kind1L2T {
    /// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
    fn compose<'a, B: 'a, C: 'a, D: 'a>(
        f: Apply1L2T<'a, Self, C, D>,
        g: Apply1L2T<'a, Self, B, C>
    ) -> Apply1L2T<'a, Self, B, D>;
}

/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
///
/// Free function version that dispatches to [the type class' associated function][`Semigroupoid::compose`].
pub fn compose<'a, Brand: Semigroupoid, B: 'a, C: 'a, D: 'a>(
    f: Apply1L2T<'a, Brand, C, D>,
    g: Apply1L2T<'a, Brand, B, C>
) -> Apply1L2T<'a, Brand, B, D> {
    Brand::compose(f, g)
}
