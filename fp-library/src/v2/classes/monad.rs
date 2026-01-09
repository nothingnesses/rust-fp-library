use super::{applicative::Applicative, semimonad::Semimonad};

/// A type class for monads.
///
/// `Monad` extends [`Applicative`] and [`Semimonad`].
/// It allows for sequencing computations where the structure of the computation depends on the result of the previous computation.
pub trait Monad: Applicative + Semimonad {}
