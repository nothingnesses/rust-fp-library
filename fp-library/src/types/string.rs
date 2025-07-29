//! Implementations for [`String`].

use crate::{
	hkt::{Apply, Brand0, Kind0},
	impl_brand,
	typeclasses::{Monoid, Semigroup},
};

impl_brand!(StringBrand, String, Kind0, Brand0, ());

impl Semigroup for StringBrand {
	/// # Examples
	///
	/// ```rust
	/// use fp_library::{brands::StringBrand, functions::append};
	///
	/// assert_eq!(
	///     append::<StringBrand>("Hello, ".to_string())("World!".to_string()),
	///     "Hello, World!"
	/// );
	/// ```
	fn append(a: Apply<Self, ()>) -> impl Fn(Apply<Self, ()>) -> Apply<Self, ()> {
		move |b| a.to_owned() + &b
	}
}

impl Monoid for StringBrand {
	/// # Examples
	///
	/// ```rust
	/// use fp_library::{brands::StringBrand, functions::empty};
	///
	/// assert_eq!(
	///     empty::<StringBrand>(),
	///     ""
	/// );
	/// ```
	fn empty() -> Apply<Self, ()> {
		Apply::<Self, ()>::default()
	}
}
