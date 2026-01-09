/// A type class for types that support an associative binary operation.
///
/// `Semigroup` instances must satisfy the associative law:
/// * Associativity: `append(a, append(b, c)) = append(append(a, b), c)`.
pub trait Semigroup {
    /// The result of combining the two values using the semigroup operation.
    fn append(a: Self, b: Self) -> Self;
}

/// The result of combining the two values using the semigroup operation.
///
/// Free function version that dispatches to [the type class' associated function][`Semigroup::append`].
pub fn append<S: Semigroup>(a: S, b: S) -> S {
    S::append(a, b)
}
