use crate::{
	classes::{monoid::Monoid, semigroup::Semigroup},
	impl_kind,
	kinds::*,
};

impl_kind! {
	for String {
		type Of<'a> = String;
	}
}

impl Semigroup for String {
	/// The result of combining two strings.
	///
	/// This method combines two strings into a single string.
	///
	/// ### Type Signature
	///
	/// `Semigroup String => (String, String) -> String`
	///
	/// ### Parameters
	///
	/// * `a`: The first string.
	/// * `b`: The second string.
	///
	/// ### Returns
	///
	/// The combined string.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let s1 = "Hello, ".to_string();
	/// let s2 = "World!".to_string();
	/// let result = append::<_>(s1, s2);
	/// assert_eq!(result, "Hello, World!");
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		a + &b
	}
}

impl Monoid for String {
	/// The identity element.
	///
	/// This method returns the identity element of the monoid.
	///
	/// ### Type Signature
	///
	/// `Monoid String => () -> String`
	///
	/// ### Returns
	///
	/// The identity element.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let empty_string = empty::<String>();
	/// assert_eq!(empty_string, "");
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
