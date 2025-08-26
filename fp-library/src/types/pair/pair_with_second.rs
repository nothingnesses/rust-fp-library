//! Implementations for the partially-applied form of [`Pair`] with [the second value][Pair#structfield.1] filled in.

use crate::{
	functions::map,
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{
		Applicative, ClonableFn, Foldable, Functor, Monoid, Pure, Traversable, clonable_fn::ApplyFn,
	},
	types::Pair,
};

/// [Brand][crate::brands] for the partially-applied form of [`Pair`] with [the second value][Pair#structfield.1] filled in.
pub struct PairWithSecondBrand<Second>(Second);

impl<Second> Kind0L1T for PairWithSecondBrand<Second> {
	type Output<First> = Pair<First, Second>;
}

impl<Second: Monoid + Clone> Pure for PairWithSecondBrand<Second> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithSecondBrand, RcFnBrand}, functions::pure, types::Pair};
	///
	/// assert_eq!(
	///     pure::<RcFnBrand, PairWithSecondBrand<String>, _>(()),
	///     Pair((), "".to_string())
	/// );
	/// ```
	fn pure<ClonableFnBrand: ClonableFn, A: Clone>(a: A) -> Apply0L1T<Self, A> {
		Pair::new::<ClonableFnBrand>(a)(Second::empty())
	}
}

impl<Second> Functor for PairWithSecondBrand<Second> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithSecondBrand, RcFnBrand}, functions::{identity, map}, types::Pair};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     map::<RcFnBrand, PairWithSecondBrand<_>, _, _>(Rc::new(|x: bool| !x))(Pair(true, ())),
	///     Pair(false, ())
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| Pair(f(fa.0), fa.1))
	}
}

impl<Second> Foldable for PairWithSecondBrand<Second> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithSecondBrand, RcFnBrand}, functions::fold_right, types::Pair};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_right::<RcFnBrand, PairWithSecondBrand<_>, _, _>(Rc::new(|a| Rc::new(move |b| a + b)))(1)(Pair(1, ())),
	///     2
	/// );
	/// ```
	fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, ApplyFn<'a, ClonableFnBrand, B, B>>
	) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>> {
		ClonableFnBrand::new(move |b: B| {
			ClonableFnBrand::new({
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
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithSecondBrand, OptionBrand, RcFnBrand}, functions::traverse, types::Pair};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     traverse::<RcFnBrand, PairWithSecondBrand<_>, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(Pair(3, ())),
	///     Some(Pair(6, ()))
	/// );
	/// ```
	fn traverse<
		'a,
		ClonableFnBrand: 'a + ClonableFn,
		F: Applicative,
		A: 'a + Clone,
		B: 'a + Clone,
	>(
		f: ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<F, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<F, Apply0L1T<Self, B>>>
	where
		Apply0L1T<F, B>: Clone,
		Apply0L1T<F, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>>>: Clone,
		Apply0L1T<Self, B>: 'a,
		Apply0L1T<Self, Apply0L1T<F, B>>: 'a,
	{
		ClonableFnBrand::new(move |ta: Apply0L1T<Self, _>| {
			let (f, Pair(first, second)) = (f.clone(), ta);
			map::<ClonableFnBrand, F, B, Apply0L1T<Self, B>>(ClonableFnBrand::new(move |first| {
				Pair::new::<ClonableFnBrand>(first)(second.to_owned())
			}))(f(first))
		})
	}
}
