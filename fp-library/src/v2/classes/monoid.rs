use super::semigroup::Semigroup;

/// A type class for types that have an identity element and an associative binary operation.
///
/// `Monoid` extends [`Semigroup`] with an identity element.
///
/// # Laws
///
/// `Monoid` instances must satisfy the identity laws:
/// * Left Identity: `append(empty(), a) = a`.
/// * Right Identity: `append(a, empty()) = a`.
pub trait Monoid: Semigroup {
    /// The identity element.
    fn empty() -> Self;
}

/// The identity element.
///
/// Free function version that dispatches to [the type class' associated function][`Monoid::empty`].
pub fn empty<M: Monoid>() -> M {
    M::empty()
}
