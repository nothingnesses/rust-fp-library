//! A newtype wrapper whose [`Semigroup`](crate::classes::Semigroup) instance always keeps the last
//! (rightmost) value.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	functions::*,
//! 	types::Last,
//! };
//!
//! assert_eq!(append(Last(1), Last(2)), Last(2));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A newtype wrapper whose [`Semigroup`] instance always keeps the last value.
	///
	/// `append(Last(a), Last(b))` returns `Last(b)`, discarding `a`.
	///
	/// There is no [`Monoid`] instance because there is no identity element.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::Last,
	/// };
	///
	/// assert_eq!(append(Last("hello"), Last("world")), Last("world"));
	/// ```
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Last<A>(
		/// The wrapped value.
		pub A,
	);

	#[document_type_parameters("The wrapped type.")]
	impl<A> Semigroup for Last<A> {
		/// Returns the last (rightmost) value, discarding the first.
		#[document_signature]
		///
		#[document_parameters("The first value (discarded).", "The second value (kept).")]
		///
		#[document_returns("The second value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::Last,
		/// };
		///
		/// assert_eq!(append(Last(1), Last(2)), Last(2));
		/// ```
		fn append(
			_a: Self,
			b: Self,
		) -> Self {
			b
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
		let x = Last(a);
		let y = Last(b);
		let z = Last(c);
		append(x, append(y, z)) == append(append(x, y), z)
	}
}
