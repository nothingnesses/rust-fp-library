//! Implementations for the partially-applied form of [`Pair`] with [the first value][Pair#structfield.0] filled in.

use crate::{
	brands::{Brand, Brand1},
	hkt::{Apply, Kind, Kind1},
	typeclasses::Functor,
	types::Pair,
};

/// [Brand][crate::brands] for the partially-applied form of [`Pair`] with [the first value][Pair#structfield.0] filled in.
pub struct PairWithFirstBrand<First>(First);

impl<First, Second> Kind1<Second> for PairWithFirstBrand<First> {
	type Output = Pair<First, Second>;
}

impl<First, Second> Brand1<Pair<First, Second>, Second> for PairWithFirstBrand<First> {
	fn inject(a: Pair<First, Second>) -> Apply<Self, (Second,)> {
		a
	}
	fn project(a: Apply<Self, (Second,)>) -> Pair<First, Second> {
		a
	}
}

impl<First> Functor for PairWithFirstBrand<First>
where
	First: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::PairWithFirstBrand, functions::{identity, map}, types::Pair};
	///
	/// assert_eq!(map::<PairWithFirstBrand<_>, _, _, _>(|x: bool| !x)(Pair((), true)), Pair((), false));
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
	{
		move |fa| {
			let fa = <Self as Brand<_, (A,)>>::project(fa);
			<Self as Brand<_, _>>::inject(Pair(fa.0, f(fa.1)))
		}
	}
}
