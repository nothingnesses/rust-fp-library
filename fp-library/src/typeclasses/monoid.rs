use crate::{hkt::Apply, typeclasses::Semigroup};

/// A typeclass for monoids.
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
/// Monoid instances must satisfy the following laws:
/// * Left identity: `append(empty(), x) = x`.
/// * Right identity: `append(x, empty()) = x`.
/// * Associativity: `append(append(x, y), z) = append(x, append(y, z))`.
///
/// # Examples
///
/// Common monoids include:
/// * Strings with concatenation and empty string.
/// * Numbers with addition and zero.
/// * Numbers with multiplication and one.
/// * Lists with concatenation and empty list.
pub trait Monoid<'a>: Semigroup<'a> {
	/// Returns the identity element for the monoid.
	///
	/// # Type Signature
	///
	/// `forall a. Monoid a => () -> a`
	///
	/// # Returns
	///
	/// The identity element which, when combined with any other element
	/// using the semigroup operation, leaves the other element unchanged.
	fn empty() -> Apply<Self, ()>;
}

/// Returns the identity element for the monoid.
///
/// Free function version that dispatches to [the typeclass' associated function][`Monoid::empty`].
///
/// # Type Signature
///
/// `forall a. Monoid a => () -> a`
///
/// # Returns
///
/// The identity element which, when combined with any other element
/// using the semigroup operation, leaves the other element unchanged.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::StringBrand, functions::empty};
///
/// assert_eq!(empty::<StringBrand>(), "".to_string());
/// ```
pub fn empty<Brand>() -> Apply<Brand, ()>
where
	for<'a> Brand: Monoid<'a>,
{
	Brand::empty()
}
