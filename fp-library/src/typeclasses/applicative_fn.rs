//! Trait for applicative functors that can handle function types

use crate::{
    aliases::ClonableFn,
    hkt::Apply,
    typeclasses::{Applicative, Pure},
};

/// A trait for applicative functors that can handle function types
pub trait ApplicativeFn<'a, A, B>: Applicative {
    /// Lifts a function into the applicative context
    fn lift_fn(f: ClonableFn<'a, A, B>) -> Apply<Self, (ClonableFn<'a, A, B>,)>;
}

/// Blanket implementation for applicative functors that implement the necessary kind
impl<'a, A, B, F> ApplicativeFn<'a, A, B> for F
where
    F: Applicative + Kind<(ClonableFn<'a, A, B>,)>,
{
    fn lift_fn(f: ClonableFn<'a, A, B>) -> Apply<Self, (ClonableFn<'a, A, B>,)> {
        pure(f)
    }
}