//! [`Semigroup`](crate::classes::Semigroup) and [`Monoid`](crate::classes::Monoid) instances for the standard library [`String`] type.
//!
//! Provides string concatenation as a monoidal operation with the empty string as the identity element.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				Monoid,
				Semigroup,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
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
		#[document_signature]
		///
		#[document_parameters("The first string.", "The second string.")]
		///
		#[document_returns("The combined string.")]
		///
		#[document_examples]
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
		#[document_signature]
		///
		#[document_returns("The identity element.")]
		///
		#[document_examples]
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
}

#[cfg(test)]
mod tests {
	use {
		crate::classes::{
			monoid::Monoid,
			semigroup::append,
		},
		quickcheck_macros::quickcheck,
	};

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
