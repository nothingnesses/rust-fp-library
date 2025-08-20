//! Implementations for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.

use crate::{
	aliases::ArcFn,
	functions::{map, pure},
	hkt::{Apply1, Kind1},
	typeclasses::{
		Applicative, Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Foldable, Functor,
		Pure, Traversable,
	},
};
use std::sync::Arc;

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.
pub struct ResultWithErrBrand<E>(E);

impl<E> Kind1 for ResultWithErrBrand<E> {
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
	fn pure<A>(a: A) -> Apply1<Self, A> {
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
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>> {
		Arc::new(move |fa| fa.map(&*f))
	}
}

impl<E> TypeclassApply for ResultWithErrBrand<E>
where
	E: Clone,
	for<'a> E: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{apply, identity}};
	///
	/// assert_eq!(
	///     apply::<ResultWithErrBrand<_>, fn(()) -> (), _, _>(Err(true))(Err(true)),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithErrBrand<_>, fn(()) -> (), _, _>(Err(true))(Ok(())),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithErrBrand<_>, _, _, _>(Ok(identity::<()>))(Err(true)),
	///     Err(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithErrBrand<bool>, _, _, _>(Ok(identity))(Ok(())),
	///     Ok(())
	/// );
	/// ```
	fn apply<'a, F: 'a + Fn(A) -> B, A: 'a + Clone, B: 'a>(
		ff: Apply1<Self, F>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>>
	where
		Apply1<Self, F>: Clone,
	{
		Arc::new(move |fa| match (ff.to_owned(), &fa) {
			(Ok(f), _) => map::<ResultWithErrBrand<_>, _, _>(Arc::new(f))(fa),
			(Err(e), _) => Err::<B, _>(e),
		})
	}
}

impl<E> ApplyFirst for ResultWithErrBrand<E>
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
	fn apply_first<'a, A: 'a + Clone, B>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, A>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(move |fb| match (fa.to_owned(), fb) {
			(Ok(a), Ok(_a)) => Ok(a),
			(Err(e), _) | (_, Err(e)) => Err(e),
		})
	}
}

impl<E> ApplySecond for ResultWithErrBrand<E>
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
	fn apply_second<'a, A: 'a, B: 'a + Clone>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(move |fb| match (fa.to_owned(), fb) {
			(Ok(_a), Ok(a)) => Ok(a),
			(Err(e), _) | (_, Err(e)) => Err(e),
		})
	}
}

impl<E> Bind for ResultWithErrBrand<E>
where
	E: Clone,
	for<'a> E: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{bind, pure}};
	///
	/// assert_eq!(
	///     bind::<ResultWithErrBrand<_>, _, _, _>(Err(()))(pure::<ResultWithErrBrand<_>, ()>),
	///     Err(())
	/// );
	/// assert_eq!(
	///     bind::<ResultWithErrBrand<()>, _, _, _>(Ok(()))(pure::<ResultWithErrBrand<_>, _>),
	///     Ok(())
	/// );
	/// ```
	fn bind<'a, F: Fn(A) -> Apply1<Self, B>, A: 'a + Clone, B>(
		ma: Apply1<Self, A>
	) -> ArcFn<'a, F, Apply1<Self, B>> {
		Arc::new(move |f| ma.to_owned().and_then(|a| -> Result<B, _> { f(a) }))
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
	) -> ArcFn<'a, B, ArcFn<'a, Apply1<Self, A>, B>> {
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

impl<E> Traversable for ResultWithErrBrand<E> {
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: Clone>(
		f: ArcFn<'a, A, Apply1<F, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<F, Apply1<Self, B>>>
	where
		Apply1<F, B>: 'a + Clone,
	{
		Arc::new(move |ta| match (f.clone(), ta) {
			(_, Err(e)) => pure::<F, _>(Err(e)),
			(f, Ok(a)) => map::<F, B, _>(Arc::new(pure::<Self, _>))(f(a)),
		})
	}
}
