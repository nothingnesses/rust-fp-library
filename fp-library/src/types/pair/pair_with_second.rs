//! Implementations for the partially-applied form of [`Pair`] with [the second value][Pair#structfield.1] filled in.

use crate::{
	aliases::ArcFn,
	functions::map,
	hkt::{Apply1, Kind1},
	typeclasses::{Applicative, Foldable, Functor, Traversable},
	types::Pair,
};
use std::sync::Arc;

/// [Brand][crate::brands] for the partially-applied form of [`Pair`] with [the second value][Pair#structfield.1] filled in.
pub struct PairWithSecondBrand<Second>(Second);

impl<Second> Kind1 for PairWithSecondBrand<Second> {
	type Output<First> = Pair<First, Second>;
}

impl<Second> Functor for PairWithSecondBrand<Second> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::PairWithSecondBrand, functions::{identity, map}, types::Pair};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     map::<PairWithSecondBrand<_>, _, _>(Arc::new(|x: bool| !x))(Pair(true, ())),
	///     Pair(false, ())
	/// );
	/// ```
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>> {
		Arc::new(move |fa| Pair(f(fa.0), fa.1))
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
	fn fold_right<'a, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, ArcFn<'a, B, B>>
	) -> ArcFn<'a, B, ArcFn<'a, Apply1<Self, A>, B>> {
		Arc::new(move |b| {
			Arc::new({
				let f = f.clone();
				move |fa| {
					let (f, b, Pair(a, _)) = (f.clone(), b.to_owned(), fa);
					f(a)(b)
				}
			})
		})
	}
}

impl<Second> Traversable for PairWithSecondBrand<Second>
where
	Second: Clone,
{
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: Clone>(
		f: ArcFn<'a, A, Apply1<F, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<F, Apply1<Self, B>>>
	where
		Apply1<F, B>: 'a + Clone,
	{
		Arc::new(move |ta| {
			let (f, Pair(first, second)) = (f.clone(), ta);
			map::<F, B, Apply1<Self, B>>(Arc::new(move |first| Pair::new(first)(second.to_owned())))(
				f(first),
			)
		})
	}
}
