//! Implementations for [`String`].

use crate::{
	classes::{
		ClonableFn, Monoid, Semigroup, clonable_fn::ApplyClonableFn, monoid::Monoid1L0T,
		semigroup::Semigroup1L0T,
	},
	hkt::Kind1L0T,
};

#[cfg(not(feature = "v2"))]
impl Kind1L0T for String {
    type Output<'a> = String;
}

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
	) -> ApplyClonableFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'b: 'a,
	{
		<ClonableFnBrand as ClonableFn>::new(move |b: Self| a.to_owned() + &b)
	}
}

impl Semigroup1L0T for String {}

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

impl Monoid1L0T for String {}
