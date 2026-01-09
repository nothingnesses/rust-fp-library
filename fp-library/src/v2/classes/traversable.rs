use crate::hkt::Apply0L1T;
use super::{applicative::Applicative, foldable::Foldable, functor::Functor};

/// A type class for traversable functors.
///
/// `Traversable` functors can be traversed, which accumulates results and effects in some [`Applicative`] context.
pub trait Traversable: Functor + Foldable {
    /// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
    fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func: 'a>(
        f: Func,
        ta: Apply0L1T<Self, A>
    ) -> Apply0L1T<F, Apply0L1T<Self, B>>
    where
        Func: Fn(A) -> Apply0L1T<F, B>,
        Apply0L1T<Self, B>: Clone;

    /// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply0L1T<Self, Apply0L1T<F, A>>
    ) -> Apply0L1T<F, Apply0L1T<Self, A>>
    where
        Apply0L1T<F, A>: Clone,
        Apply0L1T<Self, A>: Clone;
}

/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::traverse`].
pub fn traverse<'a, Brand: Traversable, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func: 'a>(
    f: Func,
    ta: Apply0L1T<Brand, A>
) -> Apply0L1T<F, Apply0L1T<Brand, B>>
where
    Func: Fn(A) -> Apply0L1T<F, B>,
    Apply0L1T<Brand, B>: Clone
{
    Brand::traverse::<F, A, B, Func>(f, ta)
}

/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::sequence`].
pub fn sequence<'a, Brand: Traversable, F: Applicative, A: 'a + Clone>(
    ta: Apply0L1T<Brand, Apply0L1T<F, A>>
) -> Apply0L1T<F, Apply0L1T<Brand, A>>
where
    Apply0L1T<F, A>: Clone,
    Apply0L1T<Brand, A>: Clone
{
    Brand::sequence::<F, A>(ta)
}
