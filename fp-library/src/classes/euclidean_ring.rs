//! Types that support Euclidean division with a degree function.
//!
//! ### Examples
//!
//! ```
//! use fp_library::classes::EuclideanRing;
//!
//! assert_eq!(i32::divide(7, 2), 3);
//! assert_eq!(i32::modulo(7, 2), 1);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A type class for [`CommutativeRing`] types that support Euclidean division.
	///
	/// ### Laws
	///
	/// * Integral domain: `one != zero`, and if `a` and `b` are both nonzero then `multiply(a, b)` is nonzero.
	/// * Euclidean function: For all `a` and nonzero `b`, let `q = divide(a, b)` and `r = modulo(a, b)`.
	///   Then `a = add(multiply(q, b), r)`, and either `r = zero` or `degree(r) < degree(b)`.
	/// * Nonnegativity: For all nonzero `a`, `degree(a) >= 0`.
	/// * Submultiplicativity: For all nonzero `a` and `b`, `degree(a) <= degree(multiply(a, b))`.
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::{
	/// 	EuclideanRing,
	/// 	Semiring,
	/// };
	///
	/// // Euclidean property: a = q*b + r
	/// let a = 7i32;
	/// let b = 3i32;
	/// let q = i32::divide(a, b);
	/// let r = i32::modulo(a, b);
	/// assert_eq!(a, i32::add(i32::multiply(q, b), r));
	/// ```
	pub trait EuclideanRing: CommutativeRing {
		/// Returns the degree of a value.
		///
		/// The degree function measures the "size" of elements for the Euclidean algorithm.
		#[document_signature]
		///
		#[document_parameters("The value to measure.")]
		///
		#[document_returns("The degree of the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::EuclideanRing;
		///
		/// assert_eq!(i32::degree(&7), 7);
		/// assert_eq!(i32::degree(&-3), 3);
		/// ```
		fn degree(a: &Self) -> usize;

		/// Performs Euclidean division.
		#[document_signature]
		///
		#[document_parameters("The dividend.", "The divisor.")]
		///
		#[document_returns("The quotient.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::EuclideanRing;
		///
		/// assert_eq!(i32::divide(7, 2), 3);
		/// assert_eq!(i32::divide(-7, 2), -4);
		/// ```
		fn divide(
			a: Self,
			b: Self,
		) -> Self;

		/// Computes the Euclidean remainder.
		#[document_signature]
		///
		#[document_parameters("The dividend.", "The divisor.")]
		///
		#[document_returns("The remainder (always non-negative for integers).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::EuclideanRing;
		///
		/// assert_eq!(i32::modulo(7, 2), 1);
		/// assert_eq!(i32::modulo(-7, 2), 1);
		/// ```
		fn modulo(
			a: Self,
			b: Self,
		) -> Self;
	}

	/// Returns the degree of a value.
	///
	/// Free function version that dispatches to [`EuclideanRing::degree`].
	#[document_signature]
	///
	#[document_parameters("The value to measure.")]
	///
	#[document_returns("The degree of the value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::euclidean_ring::degree;
	///
	/// assert_eq!(degree(&7i32), 7);
	/// ```
	pub fn degree(a: &impl EuclideanRing) -> usize {
		EuclideanRing::degree(a)
	}

	/// Performs Euclidean division.
	///
	/// Free function version that dispatches to [`EuclideanRing::divide`].
	#[document_signature]
	///
	#[document_type_parameters("The Euclidean ring type.")]
	///
	#[document_parameters("The dividend.", "The divisor.")]
	///
	#[document_returns("The quotient.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::euclidean_ring::divide;
	///
	/// assert_eq!(divide(7i32, 2), 3);
	/// ```
	pub fn divide<E: EuclideanRing>(
		a: E,
		b: E,
	) -> E {
		E::divide(a, b)
	}

	/// Computes the Euclidean remainder.
	///
	/// Free function version that dispatches to [`EuclideanRing::modulo`].
	#[document_signature]
	///
	#[document_type_parameters("The Euclidean ring type.")]
	///
	#[document_parameters("The dividend.", "The divisor.")]
	///
	#[document_returns("The remainder.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::euclidean_ring::modulo;
	///
	/// assert_eq!(modulo(7i32, 2), 1);
	/// ```
	pub fn modulo<E: EuclideanRing>(
		a: E,
		b: E,
	) -> E {
		E::modulo(a, b)
	}

	/// Computes the greatest common divisor of two values.
	#[document_signature]
	///
	#[document_type_parameters("The Euclidean ring type.")]
	///
	#[document_parameters("The first value.", "The second value.")]
	///
	#[document_returns("The greatest common divisor.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::euclidean_ring::greatest_common_divisor;
	///
	/// assert_eq!(greatest_common_divisor(12i32, 8), 4);
	/// ```
	pub fn greatest_common_divisor<E: EuclideanRing + PartialEq + Clone>(
		a: E,
		b: E,
	) -> E {
		if b == E::zero() {
			a
		} else {
			let r = E::modulo(a, b.clone());
			greatest_common_divisor(b, r)
		}
	}

	/// Computes the least common multiple of two values.
	#[document_signature]
	///
	#[document_type_parameters("The Euclidean ring type.")]
	///
	#[document_parameters("The first value.", "The second value.")]
	///
	#[document_returns("The least common multiple.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::euclidean_ring::least_common_multiple;
	///
	/// assert_eq!(least_common_multiple(4i32, 6), 12);
	/// ```
	pub fn least_common_multiple<E: EuclideanRing + PartialEq + Clone>(
		a: E,
		b: E,
	) -> E {
		if a == E::zero() || b == E::zero() {
			E::zero()
		} else {
			let g = greatest_common_divisor(a.clone(), b.clone());
			E::multiply(a, E::divide(b, g))
		}
	}

	macro_rules! impl_euclidean_ring_int {
		($($t:ty),+) => {
			$(
				impl EuclideanRing for $t {
					/// Returns the absolute value as the degree.
					#[document_signature]
					///
					#[document_parameters("The value to measure.")]
					///
					#[document_returns("The absolute value as a `usize`.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::EuclideanRing;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::degree(&(3 as ", stringify!($t), ")), 3);")]
					/// ```
					fn degree(a: &Self) -> usize {
						a.unsigned_abs() as usize
					}

					/// Performs Euclidean division using `div_euclid`.
					#[document_signature]
					///
					#[document_parameters("The dividend.", "The divisor.")]
					///
					#[document_returns("The quotient.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::EuclideanRing;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::divide(7 as ", stringify!($t), ", 2 as ", stringify!($t), "), 3 as ", stringify!($t), ");")]
					/// ```
					fn divide(a: Self, b: Self) -> Self {
						a.div_euclid(b)
					}

					/// Computes the Euclidean remainder using `rem_euclid`.
					#[document_signature]
					///
					#[document_parameters("The dividend.", "The divisor.")]
					///
					#[document_returns("The remainder (always non-negative).")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::EuclideanRing;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::modulo(7 as ", stringify!($t), ", 2 as ", stringify!($t), "), 1 as ", stringify!($t), ");")]
					/// ```
					fn modulo(a: Self, b: Self) -> Self {
						a.rem_euclid(b)
					}
				}
			)+
		};
	}

	impl_euclidean_ring_int!(i8, i16, i32, i64, i128);

	impl EuclideanRing for isize {
		/// Returns the absolute value as the degree.
		#[document_signature]
		///
		#[document_parameters("The value to measure.")]
		///
		#[document_returns("The absolute value as a `usize`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::EuclideanRing;
		///
		/// assert_eq!(isize::degree(&3), 3);
		/// ```
		fn degree(a: &Self) -> usize {
			a.unsigned_abs()
		}

		/// Performs Euclidean division using `div_euclid`.
		#[document_signature]
		///
		#[document_parameters("The dividend.", "The divisor.")]
		///
		#[document_returns("The quotient.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::EuclideanRing;
		///
		/// assert_eq!(isize::divide(7, 2), 3);
		/// ```
		fn divide(
			a: Self,
			b: Self,
		) -> Self {
			a.div_euclid(b)
		}

		/// Computes the Euclidean remainder using `rem_euclid`.
		#[document_signature]
		///
		#[document_parameters("The dividend.", "The divisor.")]
		///
		#[document_returns("The remainder (always non-negative).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::EuclideanRing;
		///
		/// assert_eq!(isize::modulo(7, 2), 1);
		/// ```
		fn modulo(
			a: Self,
			b: Self,
		) -> Self {
			a.rem_euclid(b)
		}
	}

	macro_rules! impl_euclidean_ring_float {
		($($t:ty),+) => {
			$(
				impl EuclideanRing for $t {
					/// Returns `1` as the degree for all values.
					///
					/// In a field, degree is constant since all non-zero elements are units.
					#[document_signature]
					///
					#[document_parameters("The value (unused).")]
					///
					#[document_returns("`1` for all values.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::EuclideanRing;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::degree(&(3.0 as ", stringify!($t), ")), 1);")]
					/// ```
					fn degree(_a: &Self) -> usize { 1 }

					/// Divides using the `/` operator.
					#[document_signature]
					///
					#[document_parameters("The dividend.", "The divisor.")]
					///
					#[document_returns("The quotient.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::EuclideanRing;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::divide(6.0 as ", stringify!($t), ", 2.0 as ", stringify!($t), "), 3.0 as ", stringify!($t), ");")]
					/// ```
					fn divide(a: Self, b: Self) -> Self { a / b }

					/// Returns `0.0` (floats form a field, so there is no remainder).
					#[document_signature]
					///
					#[document_parameters("The dividend (unused).", "The divisor (unused).")]
					///
					#[document_returns("`0.0`.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::EuclideanRing;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::modulo(7.0 as ", stringify!($t), ", 2.0 as ", stringify!($t), "), 0.0 as ", stringify!($t), ");")]
					/// ```
					fn modulo(_a: Self, _b: Self) -> Self { 0.0 }
				}
			)+
		};
	}

	impl_euclidean_ring_float!(f32, f64);
}

pub use inner::*;
