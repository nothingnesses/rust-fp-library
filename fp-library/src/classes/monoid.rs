//! Monoid type class.
//!
//! This module defines the [`Monoid`] trait, which extends [`Semigroup`] with an identity element.

use super::semigroup::Semigroup;

/// A type class for types that have an identity element and an associative binary operation.
///
/// `Monoid` extends [`Semigroup`] with an identity element.
///
/// ### Laws
///
/// `Monoid` instances must satisfy the identity laws:
/// * Left Identity: `append(empty(), a) = a`.
/// * Right Identity: `append(a, empty()) = a`.
pub trait Monoid: Semigroup {
	/// The identity element.
	///
	/// This method returns the identity element of the monoid.
	///
	/// ### Type Signature
	///
	/// `forall a. Monoid a => () -> a`
	///
	/// ### Returns
	///
	/// The identity element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::monoid::Monoid;
	/// use fp_library::types::string; // Import Monoid impl for String
	///
	/// let x = String::empty();
	/// assert_eq!(x, "".to_string());
	/// ```
	fn empty() -> Self;
}

/// The identity element.
///
/// Free function version that dispatches to [the type class' associated function][`Monoid::empty`].
///
/// ### Type Signature
///
/// `forall a. Monoid a => () -> a`
///
/// ### Type Parameters
///
/// * `M`: The type of the monoid.
///
/// ### Returns
///
/// The identity element.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::monoid::empty;
/// use fp_library::types::string; // Import Monoid impl for String
///
/// let x: String = empty();
/// assert_eq!(x, "".to_string());
/// ```
pub fn empty<M: Monoid>() -> M {
	M::empty()
}
