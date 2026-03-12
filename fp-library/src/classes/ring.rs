//! Types that extend [`Semiring`](crate::classes::Semiring) with subtraction.
//!
//! ### Examples
//!
//! ```
//! use fp_library::classes::Ring;
//!
//! assert_eq!(i32::subtract(5, 3), 2);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A type class for types that extend [`Semiring`] with subtraction.
	///
	/// ### Laws
	///
	/// * Additive inverse: `subtract(a, a) = zero`
	/// * Compatibility: `subtract(a, b) = add(a, negate(b))`
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::{
	/// 	Ring,
	/// 	Semiring,
	/// };
	///
	/// // Additive inverse: subtract(a, a) = zero
	/// assert_eq!(i32::subtract(5, 5), i32::zero());
	/// ```
	pub trait Ring: Semiring {
		/// Subtracts the second value from the first.
		#[document_signature]
		///
		#[document_parameters("The value to subtract from.", "The value to subtract.")]
		///
		#[document_returns("The difference.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::Ring;
		///
		/// assert_eq!(i32::subtract(5, 3), 2);
		/// ```
		fn subtract(
			a: Self,
			b: Self,
		) -> Self;
	}

	/// Subtracts the second value from the first.
	///
	/// Free function version that dispatches to [`Ring::subtract`].
	#[document_signature]
	///
	#[document_type_parameters("The ring type.")]
	///
	#[document_parameters("The value to subtract from.", "The value to subtract.")]
	///
	#[document_returns("The difference.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::ring::subtract;
	///
	/// assert_eq!(subtract(5i32, 3), 2);
	/// ```
	pub fn subtract<R: Ring>(
		a: R,
		b: R,
	) -> R {
		R::subtract(a, b)
	}

	/// Negates a value (additive inverse).
	///
	/// Equivalent to `subtract(zero, a)`.
	#[document_signature]
	///
	#[document_type_parameters("The ring type.")]
	///
	#[document_parameters("The value to negate.")]
	///
	#[document_returns("The negated value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::ring::negate;
	///
	/// assert_eq!(negate(5i32), -5);
	/// ```
	pub fn negate<R: Ring>(a: R) -> R {
		R::subtract(R::zero(), a)
	}

	macro_rules! impl_ring_int {
		($($t:ty),+) => {
			$(
				impl Ring for $t {
					/// Subtracts using wrapping subtraction.
					#[document_signature]
					///
					#[document_parameters("The value to subtract from.", "The value to subtract.")]
					///
					#[document_returns("The difference (wrapping on overflow).")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Ring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::subtract(5 as ", stringify!($t), ", 3 as ", stringify!($t), "), 2 as ", stringify!($t), ");")]
					/// ```
					fn subtract(a: Self, b: Self) -> Self { a.wrapping_sub(b) }
				}
			)+
		};
	}

	impl_ring_int!(i8, i16, i32, i64, i128, isize);

	macro_rules! impl_ring_float {
		($($t:ty),+) => {
			$(
				impl Ring for $t {
					/// Subtracts using the `-` operator.
					#[document_signature]
					///
					#[document_parameters("The value to subtract from.", "The value to subtract.")]
					///
					#[document_returns("The difference.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Ring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::subtract(5.0 as ", stringify!($t), ", 3.0 as ", stringify!($t), "), 2.0 as ", stringify!($t), ");")]
					/// ```
					fn subtract(a: Self, b: Self) -> Self { a - b }
				}
			)+
		};
	}

	impl_ring_float!(f32, f64);
}

pub use inner::*;
