use crate::{
	aliases::ArcFn,
	hkt::{Apply0L0T, Kind0L0T},
	typeclasses::{Monoid, Semigroup},
};
use std::sync::Arc;

/// [Brand][crate::brands] for the concrete form of [`Vec`], `Vec<A>`.
pub struct ConcreteVecBrand<A>(A);

impl<A> Kind0L0T for ConcreteVecBrand<A> {
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
	fn append(a: Apply0L0T<Self>) -> ArcFn<'a, Apply0L0T<Self>, Apply0L0T<Self>> {
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
	fn empty() -> Apply0L0T<Self> {
		Apply0L0T::<Self>::default()
	}
}
