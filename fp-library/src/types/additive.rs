//! A newtype wrapper whose [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances use addition
//! from [`Semiring`](crate::classes::Semiring).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	functions::*,
//! 	types::Additive,
//! };
//!
//! let x = Additive(3i32);
//! let y = Additive(4i32);
//! assert_eq!(append(x, y), Additive(7));
//! assert_eq!(empty::<Additive<i32>>(), Additive(0));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A newtype wrapper whose [`Semigroup`] instance uses [`Semiring::add`].
	///
	/// This provides a canonical [`Monoid`] for numeric types based on addition,
	/// with [`Semiring::zero`] as the identity element.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::Additive,
	/// };
	///
	/// assert_eq!(append(Additive(2i32), Additive(3)), Additive(5));
	/// assert_eq!(empty::<Additive<i32>>(), Additive(0));
	/// ```
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Additive<A>(
		/// The wrapped value.
		pub A,
	);

	#[document_type_parameters("The semiring type.")]
	impl<A: Semiring> Semigroup for Additive<A> {
		/// Combines two values using [`Semiring::add`].
		#[document_signature]
		///
		#[document_parameters("The first additive value.", "The second additive value.")]
		///
		#[document_returns("The sum wrapped in `Additive`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Additive,
		/// };
		///
		/// assert_eq!(append(Additive(10i32), Additive(20)), Additive(30));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			Additive(A::add(a.0, b.0))
		}
	}

	#[document_type_parameters("The semiring type.")]
	impl<A: Semiring> Monoid for Additive<A> {
		/// Returns `Additive(zero())`.
		#[document_signature]
		///
		#[document_returns("The additive identity wrapped in `Additive`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Additive,
		/// };
		///
		/// assert_eq!(empty::<Additive<i32>>(), Additive(0));
		/// ```
		fn empty() -> Self {
			Additive(A::zero())
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
		let x = Additive(a);
		let y = Additive(b);
		let z = Additive(c);
		append(x, append(y, z)) == append(append(x, y), z)
	}

	#[quickcheck]
	fn monoid_left_identity(a: i32) -> bool {
		let x = Additive(a);
		append(empty::<Additive<i32>>(), x) == x
	}

	#[quickcheck]
	fn monoid_right_identity(a: i32) -> bool {
		let x = Additive(a);
		append(x, empty::<Additive<i32>>()) == x
	}
}
