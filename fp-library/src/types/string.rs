//! Implementations for [`String`].

use std::sync::Arc;

use crate::{
	aliases::ArcFn,
	hkt::{Apply0L0T, Kind0L0T},
	typeclasses::{Monoid, Semigroup},
};

pub struct StringBrand;

impl Kind0L0T for StringBrand {
	type Output = String;
}

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
	fn append(a: Apply0L0T<Self>) -> ArcFn<'a, Apply0L0T<Self>, Apply0L0T<Self>> {
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
	fn empty() -> Apply0L0T<Self> {
		Apply0L0T::<Self>::default()
	}
}
