use crate::typeclasses::{ClonableFn, Monoid, Semigroup, clonable_fn::ApplyFn};

impl<A> Semigroup for Vec<A>
where
	A: Clone,
{
	/// # Examples
	///
	/// ```rust
	/// use fp_library::{brands::RcFnBrand, functions::append};
	///
	/// assert_eq!(
	///     append::<RcFnBrand, Vec<_>>(vec![true])(vec![false]),
	///     vec![true, false]
	/// );
	/// ```
	fn append<'a, ClonableFnBrand: 'a + ClonableFn>(
		a: Self
	) -> ApplyFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
	{
		ClonableFnBrand::new(move |b: Self| [a.to_owned(), b.to_owned()].concat())
	}
}

impl<A> Monoid for Vec<A>
where
	A: Clone,
{
	/// # Examples
	///
	/// ```rust
	/// use fp_library::functions::empty;
	///
	/// assert_eq!(
	///     empty::<Vec<()>>(),
	///     []
	/// );
	/// ```
	fn empty() -> Self {
		Self::default()
	}
}
