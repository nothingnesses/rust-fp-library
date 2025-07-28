//! Implementations for [`String`] using the HKT system.

use crate::{
	hkt::{Apply, Brand, Brand0, Kind, Kind0},
	impl_brand,
	typeclasses::semigroup_hkt::Semigroup,
};

// Create the brand using the new macro support for arity 0
impl_brand!(StringBrandHKT, String, Kind0, Brand0, ());

impl Semigroup for StringBrandHKT {
	/// # Examples
	///
	/// ```
	/// use fp_library::types::StringBrandHKT;
	/// use fp_library::typeclasses::semigroup_hkt::append;
	/// use fp_library::hkt::Brand;
	///
	/// let s1 = <StringBrandHKT as Brand<String, ()>>::inject("Hello, ".to_string());
	/// let s2 = <StringBrandHKT as Brand<String, ()>>::inject("World!".to_string());
	/// let result = append::<StringBrandHKT>(s1, s2);
	/// let string_result = <StringBrandHKT as Brand<String, ()>>::project(result);
	/// assert_eq!(string_result, "Hello, World!");
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
