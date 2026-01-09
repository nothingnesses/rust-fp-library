use crate::hkt::{Apply0L1T, Kind0L1T};
use super::monoid::Monoid;

/// A type class for structures that can be folded to a single value.
///
/// A `Foldable` represents a structure that can be folded over to combine its elements
/// into a single result.
pub trait Foldable: Kind0L1T {
    /// Folds the structure by applying a function from right to left.
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(A, B) -> B;

    /// Folds the structure by applying a function from left to right.
    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Self, A>) -> B
    where
        F: Fn(B, A) -> B;

    /// Maps values to a monoid and combines them.
    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply0L1T<Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M;
}

/// Folds the structure by applying a function from right to left.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_right`].
pub fn fold_right<'a, Brand: Foldable, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Brand, A>) -> B
where
    F: Fn(A, B) -> B
{
    Brand::fold_right(f, init, fa)
}

/// Folds the structure by applying a function from left to right.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_left`].
pub fn fold_left<'a, Brand: Foldable, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply0L1T<Brand, A>) -> B
where
    F: Fn(B, A) -> B
{
    Brand::fold_left(f, init, fa)
}

/// Maps values to a monoid and combines them.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_map`].
pub fn fold_map<'a, Brand: Foldable, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply0L1T<Brand, A>) -> M
where
    M: Monoid,
    F: Fn(A) -> M
{
    Brand::fold_map(f, fa)
}
