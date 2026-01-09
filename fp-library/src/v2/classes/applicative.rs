use super::{apply_first::ApplyFirst, apply_second::ApplySecond, pointed::Pointed, semiapplicative::Semiapplicative};

/// A type class for applicative functors.
///
/// `Applicative` extends [`Pointed`] and [`Semiapplicative`].
/// It allows for values to be wrapped in a context and for functions within a context to be applied to values within a context.
pub trait Applicative: Pointed + Semiapplicative + ApplyFirst + ApplySecond {}

impl<Brand> Applicative for Brand where Brand: Pointed + Semiapplicative + ApplyFirst + ApplySecond {}
