use std::sync::Arc;

use crate::{
	aliases::ClonableFn,
	hkt::{Apply, Brand0, Kind0},
	typeclasses::{Monoid, Semigroup},
};

/// [Brand][crate::brands] for the concrete form of [`Vec`], `Vec<A>`.
pub struct ConcreteVecBrand<A>(A);

impl<A> Kind0 for ConcreteVecBrand<A> {
	type Output = Vec<A>;
}

impl<A> Brand0<Vec<A>> for ConcreteVecBrand<A> {
	fn inject(a: Vec<A>) -> Apply<Self, ()> {
		a
	}
	fn project(a: Apply<Self, ()>) -> Vec<A> {
		a
	}
}

impl<'a, A> Semigroup<'a> for ConcreteVecBrand<A>
where
	A: 'a + Clone,
{
	/// # Examples
	///
	/// ```rust
	/// use fp_library::{brands::ConcreteVecBrand, functions::append};
	///
	/// assert_eq!(
	///     append::<ConcreteVecBrand<_>>(vec![true])(vec![false]),
	///     vec![true, false]
	/// );
	/// ```
	fn append(a: Apply<Self, ()>) -> ClonableFn<'a, Apply<Self, ()>, Apply<Self, ()>> {
		Arc::new(move |b: Apply<Self, ()>| [a.to_owned(), b.to_owned()].concat())
	}
}

impl<'a, A> Monoid<'a> for ConcreteVecBrand<A>
where
	A: 'a + Clone,
{
	/// # Examples
	///
	/// ```rust
	/// use fp_library::{brands::ConcreteVecBrand, functions::empty};
	///
	/// assert_eq!(
	///     empty::<ConcreteVecBrand<()>>(),
	///     []
	/// );
	/// ```
	fn empty() -> Apply<Self, ()> {
		Apply::<Self, ()>::default()
	}
}
