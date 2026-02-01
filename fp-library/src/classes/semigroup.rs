//! A type class for types that support an associative binary operation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::functions::*;
//!
//! let x = "Hello, ".to_string();
//! let y = "World!".to_string();
//! let z = append::<_>(x, y);
//! assert_eq!(z, "Hello, World!".to_string());
//! ```

use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;
/// A type class for types that support an associative binary operation.
///
/// `Semigroup` instances must satisfy the associative law:
/// * Associativity: `append(a, append(b, c)) = append(append(a, b), c)`.
pub trait Semigroup {
	/// The result of combining the two values using the semigroup operation.
	///
	/// This method combines two values of the same type into a single value of that type.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semigroup)]
	///
	/// ### Parameters
	///
	#[doc_params("The first value.", "The second value.")]
	///
	/// ### Returns
	///
	/// The combined value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let x = "Hello, ".to_string();
	/// let y = "World!".to_string();
	/// let z = append::<_>(x, y);
	/// assert_eq!(z, "Hello, World!".to_string());
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self;
}

/// The result of combining the two values using the semigroup operation.
///
/// Free function version that dispatches to [the type class' associated function][`Semigroup::append`].
///
/// ### Type Signature
///
#[hm_signature(Semigroup)]
///
/// ### Type Parameters
///
#[doc_type_params("The type of the semigroup.")]
///
/// ### Parameters
///
#[doc_params("The first value.", "The second value.")]
///
/// ### Returns
///
/// The combined value.
///
/// ### Examples
///
/// ```
/// use fp_library::functions::*;
///
/// let x = "Hello, ".to_string();
/// let y = "World!".to_string();
/// let z = append::<_>(x, y);
/// assert_eq!(z, "Hello, World!".to_string());
/// ```
pub fn append<S: Semigroup>(
	a: S,
	b: S,
) -> S {
	S::append(a, b)
}
