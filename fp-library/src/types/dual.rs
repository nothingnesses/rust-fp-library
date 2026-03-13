//! A newtype wrapper that reverses the order of a [`Semigroup`](crate::classes::Semigroup)'s operation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	functions::*,
//! 	types::Dual,
//! };
//!
//! let x = Dual("hello ".to_string());
//! let y = Dual("world".to_string());
//! // Dual reverses append order: append(Dual(a), Dual(b)) = Dual(append(b, a))
//! assert_eq!(append(x, y), Dual("worldhello ".to_string()));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A newtype wrapper that reverses the order of a [`Semigroup`]'s operation.
	///
	/// `append(Dual(a), Dual(b))` is equivalent to `Dual(append(b, a))`.
	///
	/// If the inner type is a [`Monoid`], `Dual` is also a [`Monoid`] with the
	/// same identity element.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::Dual,
	/// };
	///
	/// let x = Dual("ab".to_string());
	/// let y = Dual("cd".to_string());
	/// assert_eq!(append(x, y), Dual("cdab".to_string()));
	/// ```
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Dual<A>(
		/// The wrapped value.
		pub A,
	);

	#[document_type_parameters("The semigroup type.")]
	impl<A: Semigroup> Semigroup for Dual<A> {
		/// Combines two values in reverse order.
		///
		/// `append(Dual(a), Dual(b))` computes `Dual(append(b, a))`.
		#[document_signature]
		///
		#[document_parameters("The first dual value.", "The second dual value.")]
		///
		#[document_returns("The reversed combination wrapped in `Dual`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Dual,
		/// };
		///
		/// assert_eq!(append(Dual("ab".to_string()), Dual("cd".to_string())), Dual("cdab".to_string()));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			Dual(A::append(b.0, a.0))
		}
	}

	#[document_type_parameters("The monoid type.")]
	impl<A: Monoid> Monoid for Dual<A> {
		/// Returns `Dual(empty())`.
		#[document_signature]
		///
		#[document_returns("The identity element wrapped in `Dual`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Dual,
		/// };
		///
		/// assert_eq!(empty::<Dual<String>>(), Dual(String::new()));
		/// ```
		fn empty() -> Self {
			Dual(A::empty())
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
		a: String,
		b: String,
		c: String,
	) -> bool {
		let x = Dual(a);
		let y = Dual(b);
		let z = Dual(c);
		append(x.clone(), append(y.clone(), z.clone())) == append(append(x, y), z)
	}

	#[quickcheck]
	fn monoid_left_identity(a: String) -> bool {
		let x = Dual(a);
		append(empty::<Dual<String>>(), x.clone()) == x
	}

	#[quickcheck]
	fn monoid_right_identity(a: String) -> bool {
		let x = Dual(a);
		append(x.clone(), empty::<Dual<String>>()) == x
	}
}
