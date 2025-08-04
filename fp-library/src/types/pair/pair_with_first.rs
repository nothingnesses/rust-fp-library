//! Implementations for the partially-applied form of [`Pair`] with [the first value][Pair#structfield.0] filled in.

use crate::{
	aliases::ClonableFn,
	hkt::{Apply, Brand, Brand1, Kind, Kind1},
	typeclasses::{Foldable, Functor},
	types::Pair,
};
use std::sync::Arc;

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
	/// assert_eq!(
	///     map::<PairWithFirstBrand<_>, _, _, _>(|x: bool| !x)(Pair((), true)),
	///     Pair((), false)
	/// );
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

impl<First> Foldable for PairWithFirstBrand<First> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::PairWithFirstBrand, functions::fold_right, types::Pair};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_right::<PairWithFirstBrand<_>, _, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(Pair((), 1)),
	///     2
	/// );
	/// ```
	fn fold_right<'a, A, B>(
		f: ClonableFn<'a, A, ClonableFn<'a, B, B>>
	) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Self, (A,)>, B>>
	where
		Self: 'a + Kind<(A,)>,
		A: 'a + Clone,
		B: 'a + Clone,
		Apply<Self, (A,)>: 'a,
	{
		Arc::new(move |b| {
			Arc::new({
				let f = f.clone();
				move |fa| {
					let (f, b, Pair(_, a)) =
						(f.clone(), b.to_owned(), <Self as Brand<_, (A,)>>::project(fa));
					f(a)(b)
				}
			})
		})
	}
}
