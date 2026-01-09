use crate::hkt::{Apply0L1T, Kind0L1T};

/// A type class for types that can be lifted.
///
/// `Lift` allows binary functions to be lifted into the context.
pub trait Lift: Kind0L1T {
    /// Lifts a binary function into the context.
    fn lift2<'a, A: 'a, B: 'a, C: 'a, F: 'a>(
        f: F,
        fa: Apply0L1T<Self, A>,
        fb: Apply0L1T<Self, B>
    ) -> Apply0L1T<Self, C>
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
    fa: Apply0L1T<Brand, A>,
    fb: Apply0L1T<Brand, B>
) -> Apply0L1T<Brand, C>
where
    F: Fn(A, B) -> C,
    A: Clone,
    B: Clone
{
    Brand::lift2(f, fa, fb)
}
