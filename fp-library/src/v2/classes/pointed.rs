use crate::hkt::{Apply1L1T, Kind1L1T};

/// A type class for types that can be constructed from a single value.
///
/// `Pointed` represents a context that can be initialized with a value.
pub trait Pointed: Kind1L1T {
    /// The value wrapped in the context.
    fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A>;
}

/// The value wrapped in the context.
///
/// Free function version that dispatches to [the type class' associated function][`Pointed::pure`].
pub fn pure<'a, Brand: Pointed, A: 'a>(a: A) -> Apply1L1T<'a, Brand, A> {
    Brand::pure(a)
}
