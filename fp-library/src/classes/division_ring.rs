//! Types that support multiplicative inverses.
//!
//! ### Examples
//!
//! ```
//! use fp_library::classes::DivisionRing;
//!
//! assert_eq!(f64::reciprocate(2.0), 0.5);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A type class for [`Ring`] types that support multiplicative inverses.
	///
	/// ### Laws
	///
	/// * Non-zero ring: `one != zero`
	/// * Multiplicative inverse: For all non-zero `a`,
	///   `multiply(reciprocate(a), a) = multiply(a, reciprocate(a)) = one`
	///
	/// The behaviour of `reciprocate(zero)` is undefined.
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::{
	/// 	DivisionRing,
	/// 	Semiring,
	/// };
	///
	/// let a = 4.0f64;
	/// assert_eq!(f64::multiply(f64::reciprocate(a), a), f64::one());
	/// ```
	pub trait DivisionRing: Ring {
		/// Computes the multiplicative inverse.
		#[document_signature]
		///
		#[document_parameters("The value to invert.")]
		///
		#[document_returns("The multiplicative inverse.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::DivisionRing;
		///
		/// assert_eq!(f64::reciprocate(4.0), 0.25);
		/// ```
		fn reciprocate(a: Self) -> Self;
	}

	/// Computes the multiplicative inverse.
	///
	/// Free function version that dispatches to [`DivisionRing::reciprocate`].
	#[document_signature]
	///
	#[document_type_parameters("The division ring type.")]
	///
	#[document_parameters("The value to invert.")]
	///
	#[document_returns("The multiplicative inverse.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::division_ring::reciprocate;
	///
	/// assert_eq!(reciprocate(2.0f64), 0.5);
	/// ```
	pub fn reciprocate<D: DivisionRing>(a: D) -> D {
		D::reciprocate(a)
	}

	/// Divides from the left: `multiply(reciprocate(b), a)`.
	#[document_signature]
	///
	#[document_type_parameters("The division ring type.")]
	///
	#[document_parameters("The dividend.", "The divisor.")]
	///
	#[document_returns("The result of left division.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::division_ring::divide_left;
	///
	/// assert_eq!(divide_left(6.0f64, 2.0), 3.0);
	/// ```
	pub fn divide_left<D: DivisionRing>(
		a: D,
		b: D,
	) -> D {
		D::multiply(D::reciprocate(b), a)
	}

	/// Divides from the right: `multiply(a, reciprocate(b))`.
	#[document_signature]
	///
	#[document_type_parameters("The division ring type.")]
	///
	#[document_parameters("The dividend.", "The divisor.")]
	///
	#[document_returns("The result of right division.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::division_ring::divide_right;
	///
	/// assert_eq!(divide_right(6.0f64, 2.0), 3.0);
	/// ```
	pub fn divide_right<D: DivisionRing>(
		a: D,
		b: D,
	) -> D {
		D::multiply(a, D::reciprocate(b))
	}

	impl DivisionRing for f32 {
		/// Computes the multiplicative inverse using `1.0 / a`.
		#[document_signature]
		///
		#[document_parameters("The value to invert.")]
		///
		#[document_returns("The multiplicative inverse.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::DivisionRing;
		///
		/// assert_eq!(f32::reciprocate(4.0), 0.25);
		/// ```
		fn reciprocate(a: Self) -> Self {
			1.0 / a
		}
	}

	impl DivisionRing for f64 {
		/// Computes the multiplicative inverse using `1.0 / a`.
		#[document_signature]
		///
		#[document_parameters("The value to invert.")]
		///
		#[document_returns("The multiplicative inverse.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::DivisionRing;
		///
		/// assert_eq!(f64::reciprocate(4.0), 0.25);
		/// ```
		fn reciprocate(a: Self) -> Self {
			1.0 / a
		}
	}
}

pub use inner::*;
