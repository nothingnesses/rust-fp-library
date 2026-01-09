use super::{pointed::Pointed, semiapplicative::Semiapplicative};

/// A type class for applicative functors.
///
/// `Applicative` extends [`Pointed`] and [`Semiapplicative`].
/// It allows for values to be wrapped in a context and for functions within a context to be applied to values within a context.
pub trait Applicative: Pointed + Semiapplicative {}
