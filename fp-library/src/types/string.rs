//! Implementations for [`String`].

use crate::classes::{ClonableFn, Monoid, Semigroup, clonable_fn::ApplyFn};

impl<'b> Semigroup<'b> for String {
	/// # Examples
	///
	/// ```rust
	/// use fp_library::{brands::RcFnBrand, functions::append};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     append::<RcFnBrand, String>("Hello, ".to_string())("World!".to_string()),
	///     "Hello, World!"
	/// );
	/// ```
	fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
		a: Self
	) -> ApplyFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'b: 'a,
	{
		ClonableFnBrand::new(move |b: Self| a.to_owned() + &b)
	}
}

impl<'a> Monoid<'a> for String {
	/// # Examples
	///
	/// ```rust
	/// use fp_library::functions::empty;
	///
	/// assert_eq!(
	///     empty::<String>(),
	///     ""
	/// );
	/// ```
	fn empty() -> Self {
		Self::default()
	}
}
