//! Implementations for the partially-applied form of [`Pair`] with [the first value][Pair#structfield.0] filled in.

use crate::{
	functions::{append, apply, constant, identity, map},
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{
		Applicative, Apply, ApplyFirst, ApplySecond, Bind, ClonableFn, Foldable, Functor, Monoid,
		Pure, Semigroup, Traversable, clonable_fn::ApplyFn,
	},
	types::Pair,
};

/// [Brand][crate::brands] for the partially-applied form of [`Pair`] with [the first value][Pair#structfield.0] filled in.
pub struct PairWithFirstBrand<First>(First);

impl<First> Kind0L1T for PairWithFirstBrand<First> {
	type Output<Second> = Pair<First, Second>;
}

impl<First> Functor for PairWithFirstBrand<First> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithFirstBrand, RcFnBrand}, functions::{identity, map}, types::Pair};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     map::<RcFnBrand, PairWithFirstBrand<_>, _, _>(Rc::new(|x: bool| !x))(Pair((), true)),
	///     Pair((), false)
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| Pair(fa.0, f(fa.1)))
	}
}

impl<First: Semigroup + Clone> Apply for PairWithFirstBrand<First> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::{PairWithFirstBrand, RcFnBrand},
	///     functions::{apply, identity},
	///     types::Pair
	/// };
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
	///         Pair("Hello, ".to_string(), Rc::new(identity))
	///     )(
	///         Pair("World!".to_string(), true)
	///     ),
	///     Pair("Hello, World!".to_string(), true)
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| {
			Pair(append::<ClonableFnBrand, First>(ff.0.to_owned())(fa.0), ff.1(fa.1))
		})
	}
}

impl<First: Clone + Semigroup> ApplyFirst for PairWithFirstBrand<First> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::{PairWithFirstBrand, RcFnBrand},
	///     functions::apply_first,
	///     types::Pair
	/// };
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_first::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
	///         Pair("Hello, ".to_string(), false)
	///     )(
	///         Pair("World!".to_string(), true)
	///     ),
	///     Pair("Hello, World!".to_string(), false)
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

impl<First: Clone + Semigroup> ApplySecond for PairWithFirstBrand<First> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::{PairWithFirstBrand, RcFnBrand},
	///     functions::apply_second,
	///     types::Pair
	/// };
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_second::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
	///         Pair("Hello, ".to_string(), false)
	///     )(
	///         Pair("World!".to_string(), true)
	///     ),
	///     Pair("Hello, World!".to_string(), true)
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

impl<First: Monoid + Clone> Pure for PairWithFirstBrand<First> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithFirstBrand, RcFnBrand}, functions::pure, types::Pair};
	///
	/// assert_eq!(
	///     pure::<RcFnBrand, PairWithFirstBrand<String>, _>(()),
	///     Pair("".to_string(), ())
	/// );
	/// ```
	fn pure<ClonableFnBrand: ClonableFn, A: Clone>(a: A) -> Apply0L1T<Self, A> {
		Pair::new::<ClonableFnBrand>(First::empty())(a)
	}
}

impl<First: Semigroup + Clone> Bind for PairWithFirstBrand<First> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithFirstBrand, RcFnBrand}, functions::bind, types::Pair};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     bind::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
	///         Pair("Hello, ".to_string(), true)
	///     )(
	///         Rc::new(|b: bool| Pair("World!".to_string(), b))
	///     ),
	///     Pair("Hello, World!".to_string(), true)
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
			let Pair(f_ma_second_first, f_ma_second_second) = f(ma_second.to_owned());
			Pair::new::<ClonableFnBrand>(append::<ClonableFnBrand, _>(ma_first.to_owned())(
				f_ma_second_first,
			))(f_ma_second_second)
		})
	}
}

impl<First> Foldable for PairWithFirstBrand<First> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithFirstBrand, RcFnBrand}, functions::fold_right, types::Pair};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_right::<RcFnBrand, PairWithFirstBrand<_>, _, _>(Rc::new(|a| Rc::new(move |b| a + b)))(1)(Pair((), 1)),
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
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{PairWithFirstBrand, OptionBrand, RcFnBrand}, functions::traverse, types::Pair};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     traverse::<RcFnBrand, PairWithFirstBrand<_>, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(Pair((), 3)),
	///     Some(Pair((), 6))
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
			map::<ClonableFnBrand, F, B, Apply0L1T<Self, B>>(ClonableFnBrand::new(move |second| {
				Pair::new::<ClonableFnBrand>(first.to_owned())(second)
			}))(f(second))
		})
	}
}
