//! A type class for types that have an identity element and an associative binary operation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::functions::*;
//!
//! let x: String = empty();
//! assert_eq!(x, "".to_string());
//! ```

use super::semigroup::Semigroup;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A type class for types that have an identity element and an associative binary operation.
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
	/// `forall m. Monoid m => () -> m`
	///
	/// ### Returns
	///
	/// The identity element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let x: String = empty();
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
#[hm_signature(Monoid)]
///
/// ### Type Parameters
///
#[doc_type_params("The type of the monoid.")]
///
/// ### Returns
///
/// The identity element.
///
/// ### Examples
///
/// ```
/// use fp_library::functions::*;
///
/// let x: String = empty();
/// assert_eq!(x, "".to_string());
/// ```
pub fn empty<M: Monoid>() -> M {
	M::empty()
}
