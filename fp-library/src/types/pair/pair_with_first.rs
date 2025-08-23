//! Implementations for the partially-applied form of [`Pair`] with [the first value][Pair#structfield.0] filled in.

use crate::{
	aliases::ArcFn,
	functions::map,
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{Applicative, ClonableFn, Foldable, Functor, Traversable, clonable_fn::ApplyFn},
	types::Pair,
};
use std::sync::Arc;

/// [Brand][crate::brands] for the partially-applied form of [`Pair`] with [the first value][Pair#structfield.0] filled in.
pub struct PairWithFirstBrand<First>(First);

impl<First> Kind0L1T for PairWithFirstBrand<First> {
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
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| Pair(fa.0, f(fa.1)))
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
	) -> ArcFn<'a, B, ArcFn<'a, Apply0L1T<Self, A>, B>> {
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

impl<'a, First> Traversable<'a> for PairWithFirstBrand<First>
where
	First: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithFirstBrand, OptionBrand}, functions::traverse, types::Pair};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     traverse::<PairWithFirstBrand<_>, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(Pair((), 3)),
	///     Some(Pair((), 6))
	/// );
	/// ```
	fn traverse<F: Applicative, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, Apply0L1T<F, B>>
	) -> ArcFn<'a, Apply0L1T<Self, A>, Apply0L1T<F, Apply0L1T<Self, B>>>
	where
		Apply0L1T<F, B>: 'a + Clone,
		Apply0L1T<F, ArcFn<'a, Apply0L1T<Self, B>, Apply0L1T<Self, B>>>: Clone,
	{
		Arc::new(move |ta| {
			let (f, Pair(first, second)) = (f.clone(), ta);
			map::<F, B, Apply0L1T<Self, B>>(Arc::new(move |second| {
				Pair::new(first.to_owned())(second)
			}))(f(second))
		})
	}
}
