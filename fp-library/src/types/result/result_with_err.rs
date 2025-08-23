//! Implementations for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.

use crate::{
	aliases::ArcFn,
	functions::{map, pure},
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{
		Applicative, Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, ClonableFn, Foldable,
		Functor, Pure, Traversable, clonable_fn::ApplyFn,
	},
};
use std::sync::Arc;

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.
pub struct ResultWithErrBrand<E>(E);

impl<E> Kind0L1T for ResultWithErrBrand<E> {
	type Output<A> = Result<A, E>;
}

impl<E> Pure for ResultWithErrBrand<E> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::pure};
	///
	/// assert_eq!(
	///     pure::<ResultWithErrBrand<()>, _>(()),
	///     Ok(())
	/// );
	fn pure<A>(a: A) -> Apply0L1T<Self, A> {
		Ok(a)
	}
}

impl<E> Functor for ResultWithErrBrand<E> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{identity, map}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     map::<ResultWithErrBrand<_>, _, _>(Arc::new(identity::<()>))(Err(true)),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     map::<ResultWithErrBrand<bool>, _, _>(Arc::new(identity))(Ok(())),
	///     Ok(())
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| fa.map(&*f))
	}
}

impl<E: Clone> TypeclassApply for ResultWithErrBrand<E>
where
	for<'a> E: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{apply, identity}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     apply::<ResultWithErrBrand<_>, (), ()>(Err(true))(Err(true)),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithErrBrand<_>, (), ()>(Err(true))(Ok(())),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithErrBrand<_>, (), ()>(Ok(Arc::new(identity::<()>)))(Err(true)),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithErrBrand<bool>, (), ()>(Ok(Arc::new(identity)))(Ok(())),
	///     Ok(())
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa| match (ff.to_owned(), &fa) {
			(Ok(f), _) => map::<ResultWithErrBrand<_>, ClonableFnBrand, _, _>(f)(fa),
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
	/// use fp_library::{brands::ResultWithErrBrand, functions::{apply_first, identity}};
	///
	/// assert_eq!(
	///     apply_first::<ResultWithErrBrand<_>, bool, bool>(Err(()))(Err(())),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_first::<ResultWithErrBrand<_>, bool, _>(Err(()))(Ok(false)),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_first::<ResultWithErrBrand<_>, _, bool>(Ok(true))(Err(())),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_first::<ResultWithErrBrand<()>, _, _>(Ok(true))(Ok(false)),
	///     Ok(true)
	/// );
	/// ```
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		ClonableFnBrand::new(move |fb: Apply0L1T<Self, _>| match (fa.to_owned(), fb) {
			(Ok(a), Ok(_a)) => Ok(a),
			(Err(e), _) | (_, Err(e)) => Err(e),
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
	/// use fp_library::{brands::ResultWithErrBrand, functions::{apply_second, identity}};
	///
	/// assert_eq!(
	///     apply_second::<ResultWithErrBrand<_>, bool, bool>(Err(()))(Err(())),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_second::<ResultWithErrBrand<_>, bool, _>(Err(()))(Ok(false)),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_second::<ResultWithErrBrand<_>, _, bool>(Ok(true))(Err(())),
	///     Err(())
	/// );
	/// assert_eq!(
	///     apply_second::<ResultWithErrBrand<()>, _, _>(Ok(true))(Ok(false)),
	///     Ok(false)
	/// );
	/// ```
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fb| match (fa.to_owned(), fb) {
			(Ok(_a), Ok(a)) => Ok(a),
			(Err(e), _) | (_, Err(e)) => Err(e),
		})
	}
}

impl<E: Clone> Bind for ResultWithErrBrand<E>
where
	for<'a> E: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{bind, pure}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     bind::<ResultWithErrBrand<_>, _, _>(Err(()))(Arc::new(pure::<ResultWithErrBrand<_>, ()>)),
	///     Err(())
	/// );
	/// assert_eq!(
	///     bind::<ResultWithErrBrand<()>, _, _>(Ok(()))(Arc::new(pure::<ResultWithErrBrand<_>, _>)),
	///     Ok(())
	/// );
	/// ```
	fn bind<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B>(
		ma: Apply0L1T<Self, A>
	) -> ApplyFn<
		'a,
		ClonableFnBrand,
		ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<Self, B>>,
		Apply0L1T<Self, B>,
	> {
		ClonableFnBrand::new(move |f: ApplyFn<'a, ClonableFnBrand, _, _>| {
			ma.to_owned().and_then(|a| -> Result<B, _> { f(a) })
		})
	}
}

impl<E> Foldable for ResultWithErrBrand<E> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::fold_right};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_right::<ResultWithErrBrand<i32>, _, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(Ok(1)),
	///     2
	/// );
	/// assert_eq!(
	///     fold_right::<ResultWithErrBrand<_>, i32, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(Err(())),
	///     1
	/// );
	/// ```
	fn fold_right<'a, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, ArcFn<'a, B, B>>
	) -> ArcFn<'a, B, ArcFn<'a, Apply0L1T<Self, A>, B>> {
		Arc::new(move |b| {
			Arc::new({
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
/// use fp_library::{brands::{ResultWithErrBrand, OptionBrand}, functions::traverse};
/// use std::sync::Arc;
///
/// assert_eq!(
///     traverse::<ResultWithErrBrand<String>, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(Ok(3)),
///     Some(Ok(6))
/// );
/// assert_eq!(
///     traverse::<ResultWithErrBrand<String>, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(Err("error".to_string())),
///     Some(Err("error".to_string()))
/// );
/// ```
impl<'a, E> Traversable<'a> for ResultWithErrBrand<E> {
	fn traverse<F: Applicative, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, Apply0L1T<F, B>>
	) -> ArcFn<'a, Apply0L1T<Self, A>, Apply0L1T<F, Apply0L1T<Self, B>>>
	where
		Apply0L1T<F, B>: 'a + Clone,
		Apply0L1T<F, ArcFn<'a, Apply0L1T<Self, B>, Apply0L1T<Self, B>>>: Clone,
	{
		Arc::new(move |ta| match (f.clone(), ta) {
			(_, Err(e)) => pure::<F, _>(Err(e)),
			(f, Ok(a)) => map::<F, B, _>(Arc::new(pure::<Self, _>))(f(a)),
		})
	}
}
