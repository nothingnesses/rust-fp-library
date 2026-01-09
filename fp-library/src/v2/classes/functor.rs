use crate::hkt::{Apply1L1T, Kind1L1T};

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
pub trait Functor: Kind1L1T {
    /// Maps a function over the values in the functor context.
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> B;
}

/// Maps a function over the values in the functor context.
///
/// Free function version that dispatches to [the type class' associated function][`Functor::map`].
pub fn map<'a, Brand: Functor, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Brand, A>) -> Apply1L1T<'a, Brand, B>
where
    F: Fn(A) -> B
{
    Brand::map(f, fa)
}
