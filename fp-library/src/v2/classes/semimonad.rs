use crate::hkt::{Apply0L1T, Kind0L1T};

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// If `x` has type `m a` and `f` has type `a -> m b`, then `bind(x, f)` has type `m b`,
/// representing the result of executing `x` to get a value of type `a` and then
/// passing it to `f` to get a computation of type `m b`.
pub trait Semimonad: Kind0L1T {
    /// Sequences two computations, allowing the second to depend on the value computed by the first.
    fn bind<'a, A: 'a, B: 'a, F: 'a>(
        ma: Apply0L1T<Self, A>,
        f: F
    ) -> Apply0L1T<Self, B>
    where
        F: Fn(A) -> Apply0L1T<Self, B>;
}

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// Free function version that dispatches to [the type class' associated function][`Semimonad::bind`].
pub fn bind<'a, Brand: Semimonad, A: 'a, B: 'a, F: 'a>(
    ma: Apply0L1T<Brand, A>,
    f: F
) -> Apply0L1T<Brand, B>
where
    F: Fn(A) -> Apply0L1T<Brand, B>
{
    Brand::bind(ma, f)
}
