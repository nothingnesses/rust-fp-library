//! Implementations for [`String`].

use crate::typeclasses::semigroup::Semigroup;

/// Brand for [`String`].
pub struct StringBrand;

impl Semigroup<String> for StringBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::StringBrand, functions::append};
	///
	/// let s1 = "Hello, ".to_string();
	/// let s2 = "World!".to_string();
	/// let result = append::<StringBrand, _>(s1, s2);
	/// assert_eq!(result, "Hello, World!");
	/// ```
	fn append(
		a: String,
		b: String,
	) -> String {
		a + &b
	}
}
