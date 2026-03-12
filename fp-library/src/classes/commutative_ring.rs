//! Types where multiplication is commutative.
//!
//! ### Examples
//!
//! ```
//! use fp_library::classes::Semiring;
//!
//! // CommutativeRing guarantees: multiply(a, b) = multiply(b, a)
//! assert_eq!(i32::multiply(3, 7), i32::multiply(7, 3));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A marker trait for [`Ring`] types where multiplication is commutative.
	///
	/// ### Laws
	///
	/// * Commutativity: `multiply(a, b) = multiply(b, a)`
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::Semiring;
	///
	/// assert_eq!(i32::multiply(3, 7), i32::multiply(7, 3));
	/// ```
	pub trait CommutativeRing: Ring {}

	macro_rules! impl_commutative_ring {
		($($t:ty),+) => {
			$(impl CommutativeRing for $t {})+
		};
	}

	impl_commutative_ring!(i8, i16, i32, i64, i128, isize, f32, f64);
}

pub use inner::*;
