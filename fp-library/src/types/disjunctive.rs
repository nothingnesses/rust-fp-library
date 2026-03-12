//! A newtype wrapper whose [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances use
//! disjunction from [`HeytingAlgebra`](crate::classes::HeytingAlgebra).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	functions::*,
//! 	types::Disjunctive,
//! };
//!
//! assert_eq!(append(Disjunctive(false), Disjunctive(true)), Disjunctive(true));
//! assert_eq!(empty::<Disjunctive<bool>>(), Disjunctive(false));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A newtype wrapper whose [`Semigroup`] instance uses
	/// [`HeytingAlgebra::disjoin`].
	///
	/// This provides a [`Monoid`] based on logical disjunction (OR),
	/// with [`HeytingAlgebra::false_value`] as the identity element.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::Disjunctive,
	/// };
	///
	/// assert_eq!(append(Disjunctive(false), Disjunctive(false)), Disjunctive(false));
	/// assert_eq!(append(Disjunctive(false), Disjunctive(true)), Disjunctive(true));
	/// assert_eq!(empty::<Disjunctive<bool>>(), Disjunctive(false));
	/// ```
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Disjunctive<A>(
		/// The wrapped value.
		pub A,
	);

	#[document_type_parameters("The Heyting algebra type.")]
	impl<A: HeytingAlgebra> Semigroup for Disjunctive<A> {
		/// Combines two values using [`HeytingAlgebra::disjoin`].
		#[document_signature]
		///
		#[document_parameters("The first disjunctive value.", "The second disjunctive value.")]
		///
		#[document_returns("The disjunction wrapped in `Disjunctive`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Disjunctive,
		/// };
		///
		/// assert_eq!(append(Disjunctive(false), Disjunctive(true)), Disjunctive(true));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			Disjunctive(A::disjoin(a.0, b.0))
		}
	}

	#[document_type_parameters("The Heyting algebra type.")]
	impl<A: HeytingAlgebra> Monoid for Disjunctive<A> {
		/// Returns `Disjunctive(false_value())`.
		#[document_signature]
		///
		#[document_returns("The bottom element wrapped in `Disjunctive`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Disjunctive,
		/// };
		///
		/// assert_eq!(empty::<Disjunctive<bool>>(), Disjunctive(false));
		/// ```
		fn empty() -> Self {
			Disjunctive(A::false_value())
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
		let x = Disjunctive(a);
		let y = Disjunctive(b);
		let z = Disjunctive(c);
		append(x, append(y, z)) == append(append(x, y), z)
	}

	#[quickcheck]
	fn monoid_left_identity(a: bool) -> bool {
		let x = Disjunctive(a);
		append(empty::<Disjunctive<bool>>(), x) == x
	}

	#[quickcheck]
	fn monoid_right_identity(a: bool) -> bool {
		let x = Disjunctive(a);
		append(x, empty::<Disjunctive<bool>>()) == x
	}
}
