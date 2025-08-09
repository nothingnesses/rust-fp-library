//! Implementations for [`String`].

use std::sync::Arc;

use crate::{
	aliases::ClonableFn,
	hkt::{Apply, Brand0, Kind0},
	impl_brand,
	typeclasses::{Monoid, Semigroup},
};

impl_brand!(StringBrand, String, Kind0, Brand0, ());

impl<'a> Semigroup<'a> for StringBrand {
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
	fn append(a: Apply<Self, ()>) -> ClonableFn<'a, Apply<Self, ()>, Apply<Self, ()>> {
		Arc::new(move |b| a.to_owned() + &b)
	}
}

impl<'a> Monoid<'a> for StringBrand {
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
