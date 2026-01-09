use crate::hkt::{Apply0L1T, Kind0L1T};

/// A type class for types that can be constructed from a single value.
///
/// `Pointed` represents a context that can be initialized with a value.
pub trait Pointed: Kind0L1T {
    /// The value wrapped in the context.
    fn pure<A>(a: A) -> Apply0L1T<Self, A>;
}

/// The value wrapped in the context.
///
/// Free function version that dispatches to [the type class' associated function][`Pointed::pure`].
pub fn pure<Brand: Pointed, A>(a: A) -> Apply0L1T<Brand, A> {
    Brand::pure(a)
}
