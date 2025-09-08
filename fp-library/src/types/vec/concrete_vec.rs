use crate::{
	classes::{
		ClonableFn, Monoid, Semigroup, clonable_fn::ApplyClonableFn, monoid::Monoid1L0T,
		semigroup::Semigroup1L0T,
	},
	hkt::Kind1L0T,
};

impl<A> Kind1L0T for Vec<A> {
	type Output<'a> = Vec<A>;
}

impl<'b, A> Semigroup<'b> for Vec<A>
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
	) -> ApplyClonableFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
	{
		<ClonableFnBrand as ClonableFn>::new(move |b: Self| [a.to_owned(), b.to_owned()].concat())
	}
}

impl<A: Clone> Semigroup1L0T for Vec<A> {}

impl<'a, A> Monoid<'a> for Vec<A>
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

impl<A: Clone> Monoid1L0T for Vec<A> {}
