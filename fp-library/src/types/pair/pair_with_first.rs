//! Implementations for the partially-applied form of [`Pair`] with [the first value][Pair#structfield.0] filled in.

use crate::{
	aliases::ArcFn,
	functions::map,
	hkt::{Apply1, Kind1},
	typeclasses::{Applicative, Foldable, Functor, Traversable},
	types::Pair,
};
use std::sync::Arc;

/// [Brand][crate::brands] for the partially-applied form of [`Pair`] with [the first value][Pair#structfield.0] filled in.
pub struct PairWithFirstBrand<First>(First);

impl<First> Kind1 for PairWithFirstBrand<First> {
	type Output<Second> = Pair<First, Second>;
}

impl<First> Functor for PairWithFirstBrand<First> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::PairWithFirstBrand, functions::{identity, map}, types::Pair};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     map::<PairWithFirstBrand<_>, _, _>(Arc::new(|x: bool| !x))(Pair((), true)),
	///     Pair((), false)
	/// );
	/// ```
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>> {
		Arc::new(move |fa| Pair(fa.0, f(fa.1)))
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
	fn fold_right<'a, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, ArcFn<'a, B, B>>
	) -> ArcFn<'a, B, ArcFn<'a, Apply1<Self, A>, B>> {
		Arc::new(move |b| {
			Arc::new({
				let f = f.clone();
				move |fa| {
					let (f, b, Pair(_, a)) = (f.clone(), b.to_owned(), fa);
					f(a)(b)
				}
			})
		})
	}
}

impl<First> Traversable for PairWithFirstBrand<First>
where
	First: Clone,
{
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: Clone>(
		f: ArcFn<'a, A, Apply1<F, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<F, Apply1<Self, B>>>
	where
		Apply1<F, B>: 'a + Clone,
	{
		Arc::new(move |ta| {
			let (f, Pair(first, second)) = (f.clone(), ta);
			map::<F, B, Apply1<Self, B>>(Arc::new(move |second| {
				Pair::new(first.to_owned())(second)
			}))(f(second))
		})
	}
}
