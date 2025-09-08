use crate::{
	classes::{Semigroup, semigroup::Semigroup1L0T},
	hkt::Apply1L0T,
};

/// A type class for monoids.
///
/// `Monoid` extends [`Semigroup`] with an identity element. A monoid is a set
/// equipped with an associative binary operation and an identity element.
///
/// In functional programming, monoids are useful for combining values in
/// a consistent way, especially when accumulating results or folding
/// collections.
///
/// # Laws
///
/// `Monoid` instances must satisfy the following laws:
/// * Left identity: `append(empty(), x) = x`.
/// * Right identity: `append(x, empty()) = x`.
/// * Associativity: `append(append(x, y), z) = append(x, append(y, z))`.
pub trait Monoid<'a>: Semigroup<'a> {
	/// Returns the identity element for the monoid.
	///
	/// # Type Signature
	///
	/// `Monoid a => () -> a`
	///
	/// # Returns
	///
	/// The identity element which, when combined with any other element
	/// using the semigroup operation, leaves the other element unchanged.
	fn empty() -> Self;
}

/// A higher-kinded Monoid, abstracting over the lifetime parameter.
pub trait Monoid1L0T: Semigroup1L0T
where
	for<'a> Apply1L0T<'a, Self>: Monoid<'a>,
{
}

/// Returns the identity element for the monoid.
///
/// Free function version that dispatches to [the type class' associated function][`Monoid::empty`].
///
/// # Type Signature
///
/// `Monoid a => () -> a`
///
/// # Returns
///
/// The identity element which, when combined with any other element
/// using the semigroup operation, leaves the other element unchanged.
///
/// # Examples
///
/// ```
/// use fp_library::functions::empty;
///
/// assert_eq!(empty::<String>(), "".to_string());
///
pub fn empty<'a, Brand: Monoid1L0T>() -> Apply1L0T<'a, Brand>
where
	for<'b> Apply1L0T<'b, Brand>: Monoid<'b>,
{
	<Apply1L0T<'a, Brand> as Monoid<'a>>::empty()
}
