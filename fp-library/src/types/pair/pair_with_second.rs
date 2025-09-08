//! Implementations for the partially-applied form of [`Pair`] with [the second value][Pair#structfield.1] filled in.

use crate::{
	classes::{
		Applicative, Apply, ApplyFirst, ApplySecond, Bind, ClonableFn, Foldable, Functor, Monoid,
		Pointed, Semigroup, Traversable, clonable_fn::ApplyFn, monoid::Monoid1L0T,
		semigroup::Semigroup1L0T,
	},
	functions::{append, apply, constant, identity, map},
	hkt::{Apply0L1T, Apply1L0T, Kind0L1T},
	types::Pair,
};

/// [Brand][crate::brands] for the partially-applied form of [`Pair`] with [the second value][Pair#structfield.1] filled in.
pub struct PairWithSecondBrand<Second>(Second);

impl<Second> Kind0L1T for PairWithSecondBrand<Second> {
	type Output<First> = Pair<First, Second>;
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

impl<Second: Clone> Apply for PairWithSecondBrand<Second>
where
	for<'a> Apply1L0T<'a, Second>: Semigroup<'a>,
	for<'a> Second: Semigroup1L0T<Output<'a> = Second>,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::{PairWithSecondBrand, RcFnBrand},
	///     functions::{apply, identity},
	///     types::Pair
	/// };
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply::<RcFnBrand, PairWithSecondBrand<String>, _, _>(
	///         Pair(Rc::new(identity), "Hello, ".to_string())
	///     )(
	///         Pair(true, "World!".to_string())
	///     ),
	///     Pair(true, "Hello, World!".to_string())
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| {
			Pair(ff.0(fa.0), append::<ClonableFnBrand, Second>(ff.1.to_owned())(fa.1))
		})
	}
}

impl<Second: Clone> ApplyFirst for PairWithSecondBrand<Second>
where
	for<'a> Apply1L0T<'a, Second>: Semigroup<'a>,
	for<'a> Second: Semigroup1L0T<Output<'a> = Second>,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::{PairWithSecondBrand, RcFnBrand},
	///     functions::{apply_first, identity},
	///     types::Pair
	/// };
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_first::<RcFnBrand, PairWithSecondBrand<String>, _, _>(
	///         Pair(false, "Hello, ".to_string())
	///     )(
	///         Pair(true, "World!".to_string())
	///     ),
	///     Pair(false, "Hello, World!".to_string())
	/// );
	/// ```
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		ClonableFnBrand::new(move |fb| {
			apply::<ClonableFnBrand, Self, _, _>(map::<ClonableFnBrand, Self, _, _>(
				ClonableFnBrand::new(constant::<ClonableFnBrand, _, _>),
			)(fa.to_owned()))(fb)
		})
	}
}

impl<Second: Clone> ApplySecond for PairWithSecondBrand<Second>
where
	for<'a> Apply1L0T<'a, Second>: Semigroup<'a>,
	for<'a> Second: Semigroup1L0T<Output<'a> = Second>,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::{PairWithSecondBrand, RcFnBrand},
	///     functions::apply_second,
	///     types::Pair
	/// };
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_second::<RcFnBrand, PairWithSecondBrand<String>, _, _>(
	///         Pair(false, "Hello, ".to_string())
	///     )(
	///         Pair(true, "World!".to_string())
	///     ),
	///     Pair(true, "Hello, World!".to_string())
	/// );
	/// ```
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fb| {
			(apply::<ClonableFnBrand, Self, _, _>((map::<ClonableFnBrand, Self, _, _>(
				constant::<ClonableFnBrand, _, _>(ClonableFnBrand::new(identity)),
			))(fa.to_owned())))(fb)
		})
	}
}

impl<Second: Clone> Pointed for PairWithSecondBrand<Second>
where
	for<'a> Apply1L0T<'a, Second>: Monoid<'a>,
	for<'a> Second: Monoid1L0T<Output<'a> = Second>,
{
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

impl<Second: Clone> Bind for PairWithSecondBrand<Second>
where
	for<'a> Apply1L0T<'a, Second>: Semigroup<'a>,
	for<'a> Second: Semigroup1L0T<Output<'a> = Second>,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithSecondBrand, RcFnBrand}, functions::bind, types::Pair};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     bind::<RcFnBrand, PairWithSecondBrand<String>, _, _>(
	///         Pair(true, "Hello, ".to_string())
	///     )(
	///         Rc::new(|b: bool| Pair(b, "World!".to_string()))
	///     ),
	///     Pair(true, "Hello, World!".to_string())
	/// );
	/// ```
	fn bind<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		ma: Apply0L1T<Self, A>
	) -> ApplyFn<
		'a,
		ClonableFnBrand,
		ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<Self, B>>,
		Apply0L1T<Self, B>,
	> {
		ClonableFnBrand::new(move |f: ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<Self, B>>| {
			let Pair(ma_first, ma_second) = &ma;
			let Pair(f_ma_first_first, f_ma_first_second) = f(ma_first.to_owned());
			Pair::new::<ClonableFnBrand>(f_ma_first_first)(append::<ClonableFnBrand, Second>(
				ma_second.to_owned(),
			)(f_ma_first_second))
		})
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
