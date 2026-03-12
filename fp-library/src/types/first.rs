//! A newtype wrapper whose [`Semigroup`] instance always keeps the first
//! (leftmost) value.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	functions::*,
//! 	types::First,
//! };
//!
//! assert_eq!(append(First(1), First(2)), First(1));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A newtype wrapper whose [`Semigroup`] instance always keeps the first value.
	///
	/// `append(First(a), First(b))` returns `First(a)`, discarding `b`.
	///
	/// There is no [`Monoid`] instance because there is no identity element.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::First,
	/// };
	///
	/// assert_eq!(append(First("hello"), First("world")), First("hello"));
	/// ```
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct First<A>(
		/// The wrapped value.
		pub A,
	);

	#[document_type_parameters("The wrapped type.")]
	impl<A> Semigroup for First<A> {
		/// Returns the first (leftmost) value, discarding the second.
		#[document_signature]
		///
		#[document_parameters("The first value (kept).", "The second value (discarded).")]
		///
		#[document_returns("The first value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	functions::*,
		/// 	types::First,
		/// };
		///
		/// assert_eq!(append(First(1), First(2)), First(1));
		/// ```
		fn append(
			a: Self,
			_b: Self,
		) -> Self {
			a
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
		let x = First(a);
		let y = First(b);
		let z = First(c);
		append(x, append(y, z)) == append(append(x, y), z)
	}
}
