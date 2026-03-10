//! Types that have an identity element and an associative binary operation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::functions::*;
//!
//! let x: String = empty();
//! assert_eq!(x, "".to_string());
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

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
		#[document_signature]
		///
		#[document_returns("The identity element.")]
		#[document_examples]
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
	#[document_signature]
	///
	#[document_type_parameters("The type of the monoid.")]
	///
	#[document_returns("The identity element.")]
	#[document_examples]
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
}

pub use inner::*;
