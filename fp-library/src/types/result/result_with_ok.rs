//! Implementations for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.

use crate::{
	classes::{
		Applicative, Apply, ApplyFirst, ApplySecond, Bind, ClonableFn, Foldable, Functor, Pointed,
		Traversable, clonable_fn::ApplyFn,
	},
	functions::{map, pure},
	hkt::{Apply0L1T, Kind0L1T},
};

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.
pub struct ResultWithOkBrand<T>(T);

impl<T> Kind0L1T for ResultWithOkBrand<T> {
	type Output<A> = Result<T, A>;
}

impl<T> Functor for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithOkBrand, RcFnBrand}, functions::{identity, map}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     map::<RcFnBrand, ResultWithOkBrand<_>, _, _>(Rc::new(identity::<()>))(Ok(true)),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     map::<RcFnBrand, ResultWithOkBrand<bool>, _, _>(Rc::new(identity))(Err(())),
	///     Err(())
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa| match fa {
			Ok(a) => Ok(a),
			Err(e) => Err(f(e)),
		})
	}
}

impl<T: Clone> Apply for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithOkBrand, RcFnBrand}, functions::{apply, identity}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply::<RcFnBrand, ResultWithOkBrand<_>, (), ()>(Ok(true))(Ok(true)),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, ResultWithOkBrand<_>, (), ()>(Ok(true))(Err(())),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, ResultWithOkBrand<_>, (), ()>(Err(Rc::new(identity::<()>)))(Ok(true)),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, ResultWithOkBrand<bool>, (), ()>(Err(Rc::new(identity)))(Err(())),
	///     Err(())
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa| match (ff.to_owned(), &fa) {
			(Ok(e), _) => Ok::<_, B>(e),
			(Err(f), _) => map::<ClonableFnBrand, ResultWithOkBrand<_>, _, _>(f)(fa),
		})
	}
}

impl<T: Clone> ApplyFirst for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithOkBrand, RcFnBrand}, functions::{apply_first, identity}};
	///
	/// assert_eq!(
	///     apply_first::<RcFnBrand, ResultWithOkBrand<_>, bool, bool>(Ok(()))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, ResultWithOkBrand<_>, bool, _>(Ok(()))(Err(false)),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, ResultWithOkBrand<_>, _, bool>(Err(true))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, ResultWithOkBrand<()>, _, _>(Err(true))(Err(false)),
	///     Err(true)
	/// );
	/// ```
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		ClonableFnBrand::new(move |fb: Apply0L1T<Self, _>| match (fa.to_owned(), fb) {
			(Err(a), Err(_a)) => Err(a),
			(Ok(e), _) | (_, Ok(e)) => Ok(e),
		})
	}
}

impl<T: Clone> ApplySecond for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithOkBrand, RcFnBrand}, functions::{apply_second, identity}};
	///
	/// assert_eq!(
	///     apply_second::<RcFnBrand, ResultWithOkBrand<_>, bool, bool>(Ok(()))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, ResultWithOkBrand<_>, bool, _>(Ok(()))(Err(false)),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, ResultWithOkBrand<_>, _, bool>(Err(true))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, ResultWithOkBrand<()>, _, _>(Err(true))(Err(false)),
	///     Err(false)
	/// );
	/// ```
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fb| match (fa.to_owned(), fb) {
			(Err(_a), Err(a)) => Err(a),
			(Ok(e), _) | (_, Ok(e)) => Ok(e),
		})
	}
}

impl<T> Pointed for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{RcFnBrand, ResultWithOkBrand}, functions::pure};
	///
	/// assert_eq!(
	///     pure::<RcFnBrand, ResultWithOkBrand<()>, _>(()),
	///     Err(())
	/// );
	fn pure<ClonableFnBrand: ClonableFn, A: Clone>(a: A) -> Apply0L1T<Self, A> {
		Err(a)
	}
}

impl<T: Clone> Bind for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithOkBrand, RcFnBrand}, functions::{bind, pure}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     bind::<RcFnBrand, ResultWithOkBrand<_>, _, _>(Ok(()))(Rc::new(pure::<RcFnBrand, ResultWithOkBrand<_>, ()>)),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     bind::<RcFnBrand, ResultWithOkBrand<()>, _, _>(Err(()))(Rc::new(pure::<RcFnBrand, ResultWithOkBrand<_>, _>)),
	///     Err(())
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
		ClonableFnBrand::new(move |f: ApplyFn<'a, ClonableFnBrand, _, _>| {
			ma.to_owned().or_else(|a| -> Result<_, B> { f(a) })
		})
	}
}

impl<T> Foldable for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithOkBrand, RcFnBrand}, functions::fold_right};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_right::<RcFnBrand, ResultWithOkBrand<i32>, _, _>(Rc::new(|a| Rc::new(move |b| a + b)))(1)(Err(1)),
	///     2
	/// );
	/// assert_eq!(
	///     fold_right::<RcFnBrand, ResultWithOkBrand<_>, i32, _>(Rc::new(|a| Rc::new(move |b| a + b)))(1)(Ok(())),
	///     1
	/// );
	/// ```
	fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: Clone, B: Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, ApplyFn<'a, ClonableFnBrand, B, B>>
	) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>> {
		ClonableFnBrand::new(move |b: B| {
			ClonableFnBrand::new({
				let f = f.clone();
				move |fa| match (f.clone(), b.to_owned(), fa) {
					(_, b, Ok(_)) => b,
					(f, b, Err(a)) => f(a)(b),
				}
			})
		})
	}
}

impl<T: Clone> Traversable for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithOkBrand, RcFnBrand, OptionBrand}, functions::traverse};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     traverse::<RcFnBrand, ResultWithOkBrand<String>, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(Ok(String::from("success"))),
	///     Some(Ok(String::from("success")))
	/// );
	/// assert_eq!(
	///     traverse::<RcFnBrand, ResultWithOkBrand<String>, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(Err(5)),
	///     Some(Err(10))
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
		ClonableFnBrand::new(move |ta: Apply0L1T<Self, _>| match (f.clone(), ta) {
			(_, Ok(e)) => pure::<ClonableFnBrand, F, _>(Ok(e)),
			(f, Err(a)) => map::<ClonableFnBrand, F, B, _>(ClonableFnBrand::new(
				pure::<ClonableFnBrand, Self, _>,
			))(f(a)),
		})
	}
}
