//! Implementations for the partially-applied form of [`Pair`] with [the second value][Pair#structfield.1] filled in.

use crate::{
	aliases::ClonableFn,
	hkt::{Apply, Brand, Brand1, Kind, Kind1},
	typeclasses::{Foldable, Functor},
	types::Pair,
};
use std::sync::Arc;

/// [Brand][crate::brands] for the partially-applied form of [`Pair`] with [the second value][Pair#structfield.1] filled in.
pub struct PairWithSecondBrand<Second>(Second);

impl<First, Second> Kind1<First> for PairWithSecondBrand<Second> {
	type Output = Pair<First, Second>;
}

impl<First, Second> Brand1<Pair<First, Second>, First> for PairWithSecondBrand<Second> {
	fn inject(a: Pair<First, Second>) -> Apply<Self, (First,)> {
		a
	}
	fn project(a: Apply<Self, (First,)>) -> Pair<First, Second> {
		a
	}
}

impl<Second> Functor for PairWithSecondBrand<Second>
where
	Second: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::PairWithSecondBrand, functions::{identity, map}, types::Pair};
	///
	/// assert_eq!(
	///     map::<PairWithSecondBrand<_>, _, _, _>(|x: bool| !x)(Pair(true, ())),
	///     Pair(false, ())
	/// );
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
	{
		move |fa| {
			let fa = <Self as Brand<_, (A,)>>::project(fa);
			<Self as Brand<_, _>>::inject(Pair(f(fa.0), fa.1))
		}
	}
}

impl<Second> Foldable for PairWithSecondBrand<Second> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::PairWithSecondBrand, functions::fold_right, types::Pair};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_right::<PairWithSecondBrand<_>, _, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(Pair(1, ())),
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
					let (f, b, Pair(a, _)) =
						(f.clone(), b.to_owned(), <Self as Brand<_, (A,)>>::project(fa));
					f(a)(b)
				}
			})
		})
	}
}
