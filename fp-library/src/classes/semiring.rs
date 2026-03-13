//! Types that form a semiring with addition and multiplication operations.
//!
//! ### Examples
//!
//! ```
//! use fp_library::classes::Semiring;
//!
//! assert_eq!(i32::add(2, 3), 5);
//! assert_eq!(i32::multiply(2, 3), 6);
//! assert_eq!(i32::zero(), 0);
//! assert_eq!(i32::one(), 1);
//! ```

#[fp_macros::document_module]
mod inner {
	use fp_macros::*;

	/// A type class for types that form a semiring.
	///
	/// A semiring provides two binary operations (addition and multiplication)
	/// with their respective identity elements (zero and one).
	///
	/// ### Laws
	///
	/// * Commutative monoid under addition:
	///   - `add(a, add(b, c)) = add(add(a, b), c)` (associativity)
	///   - `add(zero, a) = a` and `add(a, zero) = a` (identity)
	///   - `add(a, b) = add(b, a)` (commutativity)
	/// * Monoid under multiplication:
	///   - `multiply(a, multiply(b, c)) = multiply(multiply(a, b), c)` (associativity)
	///   - `multiply(one, a) = a` and `multiply(a, one) = a` (identity)
	/// * Left distributivity: `multiply(a, add(b, c)) = add(multiply(a, b), multiply(a, c))`
	/// * Right distributivity: `multiply(add(a, b), c) = add(multiply(a, c), multiply(b, c))`
	/// * Annihilation: `multiply(zero, a) = multiply(a, zero) = zero`
	///
	/// **Note:** Integer types do not strictly satisfy these laws due to overflow.
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::Semiring;
	///
	/// // Distributivity: multiply(a, add(b, c)) = add(multiply(a, b), multiply(a, c))
	/// let a = 2i32;
	/// let b = 3i32;
	/// let c = 4i32;
	/// assert_eq!(
	/// 	i32::multiply(a, i32::add(b, c)),
	/// 	i32::add(i32::multiply(a, b), i32::multiply(a, c)),
	/// );
	/// ```
	pub trait Semiring {
		/// Adds two values.
		#[document_signature]
		///
		#[document_parameters("The first value.", "The second value.")]
		///
		#[document_returns("The sum of the two values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::Semiring;
		///
		/// assert_eq!(i32::add(2, 3), 5);
		/// ```
		fn add(
			a: Self,
			b: Self,
		) -> Self;

		/// Returns the additive identity element.
		#[document_signature]
		///
		#[document_returns("The additive identity (zero).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::Semiring;
		///
		/// assert_eq!(i32::zero(), 0);
		/// ```
		fn zero() -> Self;

		/// Multiplies two values.
		#[document_signature]
		///
		#[document_parameters("The first value.", "The second value.")]
		///
		#[document_returns("The product of the two values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::Semiring;
		///
		/// assert_eq!(i32::multiply(2, 3), 6);
		/// ```
		fn multiply(
			a: Self,
			b: Self,
		) -> Self;

		/// Returns the multiplicative identity element.
		#[document_signature]
		///
		#[document_returns("The multiplicative identity (one).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::Semiring;
		///
		/// assert_eq!(i32::one(), 1);
		/// ```
		fn one() -> Self;
	}

	/// Adds two values.
	///
	/// Free function version that dispatches to [`Semiring::add`].
	#[document_signature]
	///
	#[document_type_parameters("The semiring type.")]
	///
	#[document_parameters("The first value.", "The second value.")]
	///
	#[document_returns("The sum of the two values.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::semiring::add;
	///
	/// assert_eq!(add(2i32, 3), 5);
	/// ```
	pub fn add<S: Semiring>(
		a: S,
		b: S,
	) -> S {
		S::add(a, b)
	}

	/// Returns the additive identity element.
	///
	/// Free function version that dispatches to [`Semiring::zero`].
	#[document_signature]
	///
	#[document_type_parameters("The semiring type.")]
	///
	#[document_returns("The additive identity (zero).")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::semiring::zero;
	///
	/// assert_eq!(zero::<i32>(), 0);
	/// ```
	pub fn zero<S: Semiring>() -> S {
		S::zero()
	}

	/// Multiplies two values.
	///
	/// Free function version that dispatches to [`Semiring::multiply`].
	#[document_signature]
	///
	#[document_type_parameters("The semiring type.")]
	///
	#[document_parameters("The first value.", "The second value.")]
	///
	#[document_returns("The product of the two values.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::semiring::multiply;
	///
	/// assert_eq!(multiply(2i32, 3), 6);
	/// ```
	pub fn multiply<S: Semiring>(
		a: S,
		b: S,
	) -> S {
		S::multiply(a, b)
	}

	/// Returns the multiplicative identity element.
	///
	/// Free function version that dispatches to [`Semiring::one`].
	#[document_signature]
	///
	#[document_type_parameters("The semiring type.")]
	///
	#[document_returns("The multiplicative identity (one).")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::classes::semiring::one;
	///
	/// assert_eq!(one::<i32>(), 1);
	/// ```
	pub fn one<S: Semiring>() -> S {
		S::one()
	}

	macro_rules! impl_semiring_int {
		($($t:ty),+) => {
			$(
				impl Semiring for $t {
					/// Adds two values using wrapping addition.
					#[document_signature]
					///
					#[document_parameters("The first value.", "The second value.")]
					///
					#[document_returns("The sum (wrapping on overflow).")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Semiring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::add(2 as ", stringify!($t), ", 3 as ", stringify!($t), "), 5 as ", stringify!($t), ");")]
					/// ```
					fn add(
						a: Self,
						b: Self,
					) -> Self {
						a.wrapping_add(b)
					}

					/// Returns the additive identity (`0`).
					#[document_signature]
					///
					#[document_returns("Zero.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Semiring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::zero(), 0 as ", stringify!($t), ");")]
					/// ```
					fn zero() -> Self {
						0
					}

					/// Multiplies two values using wrapping multiplication.
					#[document_signature]
					///
					#[document_parameters("The first value.", "The second value.")]
					///
					#[document_returns("The product (wrapping on overflow).")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Semiring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::multiply(2 as ", stringify!($t), ", 3 as ", stringify!($t), "), 6 as ", stringify!($t), ");")]
					/// ```
					fn multiply(
						a: Self,
						b: Self,
					) -> Self {
						a.wrapping_mul(b)
					}

					/// Returns the multiplicative identity (`1`).
					#[document_signature]
					///
					#[document_returns("One.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Semiring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::one(), 1 as ", stringify!($t), ");")]
					/// ```
					fn one() -> Self {
						1
					}
				}
			)+
		};
	}

	impl_semiring_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

	macro_rules! impl_semiring_float {
		($($t:ty),+) => {
			$(
				impl Semiring for $t {
					/// Adds two values using the `+` operator.
					#[document_signature]
					///
					#[document_parameters("The first value.", "The second value.")]
					///
					#[document_returns("The sum.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Semiring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::add(2.0 as ", stringify!($t), ", 3.0 as ", stringify!($t), "), 5.0 as ", stringify!($t), ");")]
					/// ```
					fn add(
						a: Self,
						b: Self,
					) -> Self {
						a + b
					}

					/// Returns the additive identity (`0.0`).
					#[document_signature]
					///
					#[document_returns("Zero.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Semiring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::zero(), 0.0 as ", stringify!($t), ");")]
					/// ```
					fn zero() -> Self {
						0.0
					}

					/// Multiplies two values using the `*` operator.
					#[document_signature]
					///
					#[document_parameters("The first value.", "The second value.")]
					///
					#[document_returns("The product.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Semiring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::multiply(2.0 as ", stringify!($t), ", 3.0 as ", stringify!($t), "), 6.0 as ", stringify!($t), ");")]
					/// ```
					fn multiply(
						a: Self,
						b: Self,
					) -> Self {
						a * b
					}

					/// Returns the multiplicative identity (`1.0`).
					#[document_signature]
					///
					#[document_returns("One.")]
					#[document_examples]
					///
					/// ```
					#[doc = concat!("use fp_library::classes::Semiring;")]
					///
					#[doc = concat!("assert_eq!(<", stringify!($t), ">::one(), 1.0 as ", stringify!($t), ");")]
					/// ```
					fn one() -> Self {
						1.0
					}
				}
			)+
		};
	}

	impl_semiring_float!(f32, f64);
}

pub use inner::*;
