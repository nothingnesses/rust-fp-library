use crate::hkt::{Apply0L1T, Kind0L1T};

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
pub trait Functor: Kind0L1T {
    /// Maps a function over the values in the functor context.
    fn map<'a, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply0L1T<Self, A>) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> B;
}

/// Maps a function over the values in the functor context.
///
/// Free function version that dispatches to [the type class' associated function][`Functor::map`].
pub fn map<'a, Brand: Functor, A: 'a, B: 'a, F: 'a>(f: F, fa: Apply0L1T<Brand, A>) -> Apply0L1T<Brand, B>
where
    F: Fn(A) -> B
{
    Brand::map(f, fa)
}
