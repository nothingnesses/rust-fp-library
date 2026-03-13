//! Types that form a field (commutative division ring with Euclidean structure).
//!
//! ### Examples
//!
//! ```
//! use fp_library::classes::{
//! 	DivisionRing,
//! 	EuclideanRing,
//! 	Semiring,
//! };
//!
//! let a = 6.0f64;
//! let b = 2.0f64;
//! assert_eq!(f64::divide(a, b), 3.0);
//! assert_eq!(f64::reciprocate(b), 0.5);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A marker trait for types that form a field.
	///
	/// A field is both an [`EuclideanRing`] and a [`DivisionRing`],
	/// combining commutative ring structure with multiplicative inverses.
	///
	/// ### Laws
	///
	/// All [`EuclideanRing`] and [`DivisionRing`] laws apply.
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::{
	/// 	DivisionRing,
	/// 	Semiring,
	/// };
	///
	/// // For fields, multiply(a, reciprocate(a)) = one
	/// let a = 3.0f64;
	/// assert_eq!(f64::multiply(a, f64::reciprocate(a)), f64::one());
	/// ```
	pub trait Field: EuclideanRing + DivisionRing {}

	impl Field for f32 {}
	impl Field for f64 {}
}

pub use inner::*;
