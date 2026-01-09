use crate::hkt::{Apply1L1T, Kind1L1T};

/// A type class for types that can be lifted.
///
/// `Lift` allows binary functions to be lifted into the context.
pub trait Lift: Kind1L1T {
    /// Lifts a binary function into the context.
    fn lift2<'a, A: 'a, B: 'a, C: 'a, F: 'a>(
        f: F,
        fa: Apply1L1T<'a, Self, A>,
        fb: Apply1L1T<'a, Self, B>
    ) -> Apply1L1T<'a, Self, C>
    where
        F: Fn(A, B) -> C,
        A: Clone,
        B: Clone;
}

/// Lifts a binary function into the context.
///
/// Free function version that dispatches to [the type class' associated function][`Lift::lift2`].
pub fn lift2<'a, Brand: Lift, A: 'a, B: 'a, C: 'a, F: 'a>(
    f: F,
    fa: Apply1L1T<'a, Brand, A>,
    fb: Apply1L1T<'a, Brand, B>
) -> Apply1L1T<'a, Brand, C>
where
    F: Fn(A, B) -> C,
    A: Clone,
    B: Clone
{
    Brand::lift2(f, fa, fb)
}
