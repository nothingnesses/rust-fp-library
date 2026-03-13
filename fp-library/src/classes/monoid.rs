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
	#[document_examples]
	///
	/// Monoid laws for [`String`]:
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let a = "hello".to_string();
	///
	/// // Left Identity: append(empty(), a) = a
	/// assert_eq!(append(empty::<String>(), a.clone()), a);
	///
	/// // Right Identity: append(a, empty()) = a
	/// assert_eq!(append(a.clone(), empty::<String>()), a);
	/// ```
	///
	/// Monoid laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let a = vec![1, 2, 3];
	///
	/// // Left Identity: append(empty(), a) = a
	/// assert_eq!(append(empty::<Vec<i32>>(), a.clone()), a);
	///
	/// // Right Identity: append(a, empty()) = a
	/// assert_eq!(append(a.clone(), empty::<Vec<i32>>()), a);
	/// ```
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

	/// Appends a value to itself a given number of times.
	///
	/// Uses binary exponentiation for O(log n) appends.
	#[document_signature]
	///
	#[document_type_parameters("The monoid type.")]
	///
	#[document_parameters("The value to exponentiate.", "The number of times to append.")]
	///
	#[document_returns("The value appended to itself `n` times, or `empty()` if `n` is 0.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(power("ab".to_string(), 3), "ababab");
	/// assert_eq!(power("x".to_string(), 0), "");
	/// assert_eq!(power(vec![1, 2], 2), vec![1, 2, 1, 2]);
	/// ```
	pub fn power<M: Monoid + Clone>(
		a: M,
		n: usize,
	) -> M {
		if n == 0 {
			M::empty()
		} else if n == 1 {
			a
		} else if n.is_multiple_of(2) {
			let half = power(a, n / 2);
			M::append(half.clone(), half)
		} else {
			let rest = power(a.clone(), n - 1);
			M::append(rest, a)
		}
	}
}

pub use inner::*;
