use crate::{
	aliases::ArcFn,
	hkt::{Apply0, Kind0},
	typeclasses::{Monoid, Semigroup},
};
use std::sync::Arc;

/// [Brand][crate::brands] for the concrete form of [`Vec`], `Vec<A>`.
pub struct ConcreteVecBrand<A>(A);

impl<A> Kind0 for ConcreteVecBrand<A> {
	type Output = Vec<A>;
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
	fn append(a: Apply0<Self>) -> ArcFn<'a, Apply0<Self>, Apply0<Self>> {
		Arc::new(move |b| [a.to_owned(), b.to_owned()].concat())
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
	fn empty() -> Apply0<Self> {
		Apply0::<Self>::default()
	}
}
