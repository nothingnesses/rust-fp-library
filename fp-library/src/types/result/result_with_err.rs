//! Implementations for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.

use crate::{
	classes::{
		Applicative, ApplyFirst, ApplySecond, ClonableFn, Foldable, Functor, Pointed,
		Semiapplicative, Semimonad, Traversable, clonable_fn::ApplyClonableFn,
	},
	functions::{map, pure},
	hkt::{Apply0L1T, Kind0L1T},
};

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.
pub struct ResultWithErrBrand<E>(E);

impl<E> Kind0L1T for ResultWithErrBrand<E> {
	type Output<A> = Result<A, E>;
}

impl<E> Functor for ResultWithErrBrand<E> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithErrBrand, RcFnBrand}, functions::{identity, map}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     map::<RcFnBrand, ResultWithErrBrand<_>, _, _>(Rc::new(identity::<()>))(Err(true)),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     map::<RcFnBrand, ResultWithErrBrand<bool>, _, _>(Rc::new(identity))(Ok(())),
	///     Ok(())
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyClonableFn<'a, ClonableFnBrand, A, B>
	) -> ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		<ClonableFnBrand as ClonableFn>::new(move |fa: Apply0L1T<Self, _>| fa.map(&*f))
	}
}

impl<E: Clone> Semiapplicative for ResultWithErrBrand<E>
where
	for<'a> E: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithErrBrand, RcFnBrand}, functions::{apply, identity}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply::<RcFnBrand, ResultWithErrBrand<_>, (), ()>(Err(true))(Err(true)),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, ResultWithErrBrand<_>, (), ()>(Err(true))(Ok(())),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, ResultWithErrBrand<_>, (), ()>(Ok(Rc::new(identity::<()>)))(Err(true)),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, ResultWithErrBrand<bool>, (), ()>(Ok(Rc::new(identity)))(Ok(())),
	///     Ok(())
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyClonableFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		<ClonableFnBrand as ClonableFn>::new(move |fa| match (ff.to_owned(), &fa) {
			(Ok(f), _) => map::<ClonableFnBrand, ResultWithErrBrand<_>, _, _>(f)(fa),
			(Err(e), _) => Err::<B, _>(e),
		})
	}
}

impl<E: Clone> ApplyFirst for ResultWithErrBrand<E>
where
	for<'a> E: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithErrBrand, RcFnBrand}, functions::{apply_first, identity}};
	///
	/// assert_eq!(
	///     apply_first::<RcFnBrand, ResultWithErrBrand<_>, bool, bool>(Err(()))(Err(())),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, ResultWithErrBrand<_>, bool, _>(Err(()))(Ok(false)),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, ResultWithErrBrand<_>, _, bool>(Ok(true))(Err(())),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, ResultWithErrBrand<()>, _, _>(Ok(true))(Ok(false)),
	///     Ok(true)
	/// );
	/// ```
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		<ClonableFnBrand as ClonableFn>::new(move |fb: Apply0L1T<Self, _>| {
			match (fa.to_owned(), fb) {
				(Ok(a), Ok(_a)) => Ok(a),
				(Err(e), _) | (_, Err(e)) => Err(e),
			}
		})
	}
}

impl<E: Clone> ApplySecond for ResultWithErrBrand<E>
where
	for<'a> E: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithErrBrand, RcFnBrand}, functions::{apply_second, identity}};
	///
	/// assert_eq!(
	///     apply_second::<RcFnBrand, ResultWithErrBrand<_>, bool, bool>(Err(()))(Err(())),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, ResultWithErrBrand<_>, bool, _>(Err(()))(Ok(false)),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, ResultWithErrBrand<_>, _, bool>(Ok(true))(Err(())),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, ResultWithErrBrand<()>, _, _>(Ok(true))(Ok(false)),
	///     Ok(false)
	/// );
	/// ```
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		<ClonableFnBrand as ClonableFn>::new(move |fb| match (fa.to_owned(), fb) {
			(Ok(_a), Ok(a)) => Ok(a),
			(Err(e), _) | (_, Err(e)) => Err(e),
		})
	}
}

impl<E> Pointed for ResultWithErrBrand<E> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{RcFnBrand, ResultWithErrBrand}, functions::pure};
	///
	/// assert_eq!(
	///     pure::<RcFnBrand, ResultWithErrBrand<()>, _>(()),
	///     Ok(())
	/// );
	fn pure<ClonableFnBrand: ClonableFn, A: Clone>(a: A) -> Apply0L1T<Self, A> {
		Ok(a)
	}
}

impl<E: Clone> Semimonad for ResultWithErrBrand<E>
where
	for<'a> E: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithErrBrand, RcFnBrand}, functions::{bind, pure}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     bind::<RcFnBrand, ResultWithErrBrand<_>, _, _>(Err(()))(Rc::new(pure::<RcFnBrand, ResultWithErrBrand<_>, ()>)),
	///     Err(())
	/// );
	/// assert_eq!(
	///     bind::<RcFnBrand, ResultWithErrBrand<()>, _, _>(Ok(()))(Rc::new(pure::<RcFnBrand, ResultWithErrBrand<_>, _>)),
	///     Ok(())
	/// );
	/// ```
	fn bind<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		ma: Apply0L1T<Self, A>
	) -> ApplyClonableFn<
		'a,
		ClonableFnBrand,
		ApplyClonableFn<'a, ClonableFnBrand, A, Apply0L1T<Self, B>>,
		Apply0L1T<Self, B>,
	> {
		<ClonableFnBrand as ClonableFn>::new(
			move |f: ApplyClonableFn<'a, ClonableFnBrand, _, _>| {
				ma.to_owned().and_then(|a| -> Result<B, _> { f(a) })
			},
		)
	}
}

impl<E> Foldable for ResultWithErrBrand<E> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithErrBrand, RcFnBrand}, functions::fold_right};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_right::<RcFnBrand, ResultWithErrBrand<i32>, _, _>(Rc::new(|a| Rc::new(move |b| a + b)))(1)(Ok(1)),
	///     2
	/// );
	/// assert_eq!(
	///     fold_right::<RcFnBrand, ResultWithErrBrand<_>, i32, _>(Rc::new(|a| Rc::new(move |b| a + b)))(1)(Err(())),
	///     1
	/// );
	/// ```
	fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: Clone, B: Clone>(
		f: ApplyClonableFn<'a, ClonableFnBrand, A, ApplyClonableFn<'a, ClonableFnBrand, B, B>>
	) -> ApplyClonableFn<
		'a,
		ClonableFnBrand,
		B,
		ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>,
	> {
		<ClonableFnBrand as ClonableFn>::new(move |b: B| {
			<ClonableFnBrand as ClonableFn>::new({
				let f = f.clone();
				move |fa| match (f.clone(), b.to_owned(), fa) {
					(_, b, Err(_)) => b,
					(f, b, Ok(a)) => f(a)(b),
				}
			})
		})
	}
}

/// # Examples
///
/// ```
/// use fp_library::{brands::{ResultWithErrBrand, RcFnBrand, OptionBrand}, functions::traverse};
/// use std::rc::Rc;
///
/// assert_eq!(
///     traverse::<RcFnBrand, ResultWithErrBrand<String>, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(Ok(3)),
///     Some(Ok(6))
/// );
/// assert_eq!(
///     traverse::<RcFnBrand, ResultWithErrBrand<String>, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(Err("error".to_string())),
///     Some(Err("error".to_string()))
/// );
/// ```
impl<E: Clone> Traversable for ResultWithErrBrand<E> {
	fn traverse<'a, ClonableFnBrand: 'a + ClonableFn, F: Applicative, A: Clone, B: 'a + Clone>(
		f: ApplyClonableFn<'a, ClonableFnBrand, A, Apply0L1T<F, B>>
	) -> ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<F, Apply0L1T<Self, B>>>
	where
		Apply0L1T<F, B>: Clone,
		Apply0L1T<F, ApplyClonableFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>>>:
			Clone,
		Apply0L1T<Self, B>: 'a,
		Apply0L1T<Self, Apply0L1T<F, B>>: 'a,
	{
		<ClonableFnBrand as ClonableFn>::new(move |ta: Apply0L1T<Self, _>| match (f.clone(), ta) {
			(_, Err(e)) => pure::<ClonableFnBrand, F, _>(Err(e)),
			(f, Ok(a)) => map::<ClonableFnBrand, F, B, _>(<ClonableFnBrand as ClonableFn>::new(
				pure::<ClonableFnBrand, Self, _>,
			))(f(a)),
		})
	}
}
