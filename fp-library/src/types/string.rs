//! Implementations for [`String`].

use crate::{
	hkt::{Apply, Brand, Brand0, Kind, Kind0},
	impl_brand,
	typeclasses::Semigroup,
};

impl_brand!(StringBrand, String, Kind0, Brand0, ());

impl Semigroup for StringBrand {
	/// # Examples
	///
	/// ```rust
	/// use fp_library::{brands::StringBrand, functions::append};
	///
	/// let s1 = "Hello, ".to_string();
	/// let s2 = "World!".to_string();
	/// assert_eq!(
	///     append::<StringBrand>(s1, s2),
	///     "Hello, World!"
	/// );
	/// ```
	fn append(
		a: Apply<Self, ()>,
		b: Apply<Self, ()>,
	) -> Apply<Self, ()>
	where
		Self: Kind<()>,
	{
		let s1 = <Self as Brand<String, ()>>::project(a);
		let s2 = <Self as Brand<String, ()>>::project(b);
		<Self as Brand<String, ()>>::inject(s1 + &s2)
	}
}
