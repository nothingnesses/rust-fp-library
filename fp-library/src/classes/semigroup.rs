//! Types that support an associative binary operation.
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

#[fp_macros::document_module]
mod inner {
	use fp_macros::*;
	/// A type class for types that support an associative binary operation.
	///
	/// `Semigroup` instances must satisfy the associative law:
	/// * Associativity: `append(a, append(b, c)) = append(append(a, b), c)`.
	#[document_examples]
	///
	/// Associativity for [`String`]:
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let a = "hello".to_string();
	/// let b = " ".to_string();
	/// let c = "world".to_string();
	///
	/// // Associativity: append(a, append(b, c)) = append(append(a, b), c)
	/// assert_eq!(append(a.clone(), append(b.clone(), c.clone())), append(append(a, b), c),);
	/// ```
	///
	/// Associativity for [`Vec`]:
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let a = vec![1, 2];
	/// let b = vec![3];
	/// let c = vec![4, 5];
	///
	/// assert_eq!(append(a.clone(), append(b.clone(), c.clone())), append(append(a, b), c),);
	/// ```
	pub trait Semigroup {
		/// The result of combining the two values using the semigroup operation.
		///
		/// This method combines two values of the same type into a single value of that type.
		#[document_signature]
		///
		#[document_parameters("The first value.", "The second value.")]
		///
		#[document_returns("The combined value.")]
		#[document_examples]
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
	#[document_signature]
	///
	#[document_type_parameters("The type of the semigroup.")]
	///
	#[document_parameters("The first value.", "The second value.")]
	///
	#[document_returns("The combined value.")]
	#[document_examples]
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
}

pub use inner::*;
