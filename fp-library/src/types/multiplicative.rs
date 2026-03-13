//! A newtype wrapper whose [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances use
//! multiplication from [`Semiring`](crate::classes::Semiring).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	functions::*,
//! 	types::Multiplicative,
//! };
//!
//! let x = Multiplicative(3i32);
//! let y = Multiplicative(4i32);
//! assert_eq!(append(x, y), Multiplicative(12));
//! assert_eq!(empty::<Multiplicative<i32>>(), Multiplicative(1));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A newtype wrapper whose [`Semigroup`] instance uses [`Semiring::multiply`].
	///
	/// This provides a canonical [`Monoid`] for numeric types based on multiplication,
	/// with [`Semiring::one`] as the identity element.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::Multiplicative,
	/// };
	///
	/// assert_eq!(append(Multiplicative(2i32), Multiplicative(3)), Multiplicative(6));
	/// assert_eq!(empty::<Multiplicative<i32>>(), Multiplicative(1));
	/// ```
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Multiplicative<A>(
		/// The wrapped value.
		pub A,
	);

	#[document_type_parameters("The semiring type.")]
	impl<A: Semiring> Semigroup for Multiplicative<A> {
		/// Combines two values using [`Semiring::multiply`].
		#[document_signature]
		///
		#[document_parameters(
			"The first multiplicative value.",
			"The second multiplicative value."
		)]
		///
		#[document_returns("The product wrapped in `Multiplicative`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Multiplicative,
		/// };
		///
		/// assert_eq!(append(Multiplicative(5i32), Multiplicative(4)), Multiplicative(20));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			Multiplicative(A::multiply(a.0, b.0))
		}
	}

	#[document_type_parameters("The semiring type.")]
	impl<A: Semiring> Monoid for Multiplicative<A> {
		/// Returns `Multiplicative(one())`.
		#[document_signature]
		///
		#[document_returns("The multiplicative identity wrapped in `Multiplicative`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Multiplicative,
		/// };
		///
		/// assert_eq!(empty::<Multiplicative<i32>>(), Multiplicative(1));
		/// ```
		fn empty() -> Self {
			Multiplicative(A::one())
		}
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::functions::*,
		quickcheck_macros::quickcheck,
	};

	#[quickcheck]
	fn semigroup_associativity(
		a: i32,
		b: i32,
		c: i32,
	) -> bool {
		let x = Multiplicative(a);
		let y = Multiplicative(b);
		let z = Multiplicative(c);
		append(x, append(y, z)) == append(append(x, y), z)
	}

	#[quickcheck]
	fn monoid_left_identity(a: i32) -> bool {
		let x = Multiplicative(a);
		append(empty::<Multiplicative<i32>>(), x) == x
	}

	#[quickcheck]
	fn monoid_right_identity(a: i32) -> bool {
		let x = Multiplicative(a);
		append(x, empty::<Multiplicative<i32>>()) == x
	}
}
