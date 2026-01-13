//! Implementations for [`String`].

use crate::{
	classes::{monoid::Monoid, semigroup::Semigroup},
	hkt::Kind_L1_T0,
};

impl Kind_L1_T0 for String {
	type Output<'a> = String;
}

impl Semigroup for String {
	/// Appends one string to another.
	///
	/// # Type Signature
	///
	/// `forall. Semigroup String => (String, String) -> String`
	///
	/// # Parameters
	///
	/// * `a`: The first string.
	/// * `b`: The second string.
	///
	/// # Returns
	///
	/// The concatenated string.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semigroup::append;
	///
	/// assert_eq!(append("Hello, ".to_string(), "World!".to_string()), "Hello, World!".to_string());
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		a + &b
	}
}

impl Monoid for String {
	/// Returns an empty string.
	///
	/// # Type Signature
	///
	/// `forall. Monoid String => () -> String`
	///
	/// # Returns
	///
	/// An empty string.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::monoid::empty;
	///
	/// assert_eq!(empty::<String>(), "".to_string());
	/// ```
	fn empty() -> Self {
		String::new()
	}
}

#[cfg(test)]
mod tests {
	use crate::classes::{monoid::Monoid, semigroup::append};
	use quickcheck_macros::quickcheck;

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(
		a: String,
		b: String,
		c: String,
	) -> bool {
		append(a.clone(), append(b.clone(), c.clone())) == append(append(a, b), c)
	}

	// Monoid Laws

	/// Tests the left identity law for Monoid.
	#[quickcheck]
	fn monoid_left_identity(a: String) -> bool {
		append(String::empty(), a.clone()) == a
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(a: String) -> bool {
		append(a.clone(), String::empty()) == a
	}
}
