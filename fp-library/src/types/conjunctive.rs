//! A newtype wrapper whose [`Semigroup`] and [`Monoid`] instances use
//! conjunction from [`HeytingAlgebra`].
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	functions::*,
//! 	types::Conjunctive,
//! };
//!
//! assert_eq!(append(Conjunctive(true), Conjunctive(false)), Conjunctive(false));
//! assert_eq!(empty::<Conjunctive<bool>>(), Conjunctive(true));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A newtype wrapper whose [`Semigroup`] instance uses
	/// [`HeytingAlgebra::conjoin`].
	///
	/// This provides a [`Monoid`] based on logical conjunction (AND),
	/// with [`HeytingAlgebra::true_value`] as the identity element.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::Conjunctive,
	/// };
	///
	/// assert_eq!(append(Conjunctive(true), Conjunctive(true)), Conjunctive(true));
	/// assert_eq!(append(Conjunctive(true), Conjunctive(false)), Conjunctive(false));
	/// assert_eq!(empty::<Conjunctive<bool>>(), Conjunctive(true));
	/// ```
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Conjunctive<A>(
		/// The wrapped value.
		pub A,
	);

	#[document_type_parameters("The Heyting algebra type.")]
	impl<A: HeytingAlgebra> Semigroup for Conjunctive<A> {
		/// Combines two values using [`HeytingAlgebra::conjoin`].
		#[document_signature]
		///
		#[document_parameters("The first conjunctive value.", "The second conjunctive value.")]
		///
		#[document_returns("The conjunction wrapped in `Conjunctive`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Conjunctive,
		/// };
		///
		/// assert_eq!(append(Conjunctive(true), Conjunctive(true)), Conjunctive(true));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			Conjunctive(A::conjoin(a.0, b.0))
		}
	}

	#[document_type_parameters("The Heyting algebra type.")]
	impl<A: HeytingAlgebra> Monoid for Conjunctive<A> {
		/// Returns `Conjunctive(true_value())`.
		#[document_signature]
		///
		#[document_returns("The top element wrapped in `Conjunctive`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Conjunctive,
		/// };
		///
		/// assert_eq!(empty::<Conjunctive<bool>>(), Conjunctive(true));
		/// ```
		fn empty() -> Self {
			Conjunctive(A::true_value())
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
		a: bool,
		b: bool,
		c: bool,
	) -> bool {
		let x = Conjunctive(a);
		let y = Conjunctive(b);
		let z = Conjunctive(c);
		append(x, append(y, z)) == append(append(x, y), z)
	}

	#[quickcheck]
	fn monoid_left_identity(a: bool) -> bool {
		let x = Conjunctive(a);
		append(empty::<Conjunctive<bool>>(), x) == x
	}

	#[quickcheck]
	fn monoid_right_identity(a: bool) -> bool {
		let x = Conjunctive(a);
		append(x, empty::<Conjunctive<bool>>()) == x
	}
}
