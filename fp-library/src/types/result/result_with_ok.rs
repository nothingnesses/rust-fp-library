//! Implementations for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.

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

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.
pub struct ResultWithOkBrand<T>(T);

impl<T> Kind1 for ResultWithOkBrand<T> {
	type Output<A> = Result<T, A>;
}

impl<T> Pure for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::pure};
	///
	/// assert_eq!(
	///     pure::<ResultWithOkBrand<()>, _>(()),
	///     Err(())
	/// );
	fn pure<A>(a: A) -> Apply1<Self, A> {
		Err(a)
	}
}

impl<T> Functor for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{identity, map}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     map::<ResultWithOkBrand<_>, _, _>(Arc::new(identity::<()>))(Ok(true)),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     map::<ResultWithOkBrand<bool>, _, _>(Arc::new(identity))(Err(())),
	///     Err(())
	/// );
	/// ```
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>> {
		Arc::new(move |fa| match fa {
			Ok(a) => Ok(a),
			Err(e) => Err(f(e)),
		})
	}
}

impl<T> TypeclassApply for ResultWithOkBrand<T>
where
	T: Clone,
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{apply, identity}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     apply::<ResultWithOkBrand<_>, (), ()>(Ok(true))(Ok(true)),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithOkBrand<_>, (), ()>(Ok(true))(Err(())),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithOkBrand<_>, (), ()>(Err(Arc::new(identity::<()>)))(Ok(true)),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithOkBrand<bool>, (), ()>(Err(Arc::new(identity)))(Err(())),
	///     Err(())
	/// );
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a>(
		ff: Apply1<Self, ArcFn<'a, A, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>>
	where
		Apply1<Self, ArcFn<'a, A, B>>: Clone,
	{
		Arc::new(move |fa| match (ff.to_owned(), &fa) {
			(Ok(e), _) => Ok::<_, B>(e),
			(Err(f), _) => map::<ResultWithOkBrand<_>, _, _>(f)(fa),
		})
	}
}

impl<T> ApplyFirst for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{apply_first, identity}};
	///
	/// assert_eq!(
	///     apply_first::<ResultWithOkBrand<_>, bool, bool>(Ok(()))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_first::<ResultWithOkBrand<_>, bool, _>(Ok(()))(Err(false)),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_first::<ResultWithOkBrand<_>, _, bool>(Err(true))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_first::<ResultWithOkBrand<()>, _, _>(Err(true))(Err(false)),
	///     Err(true)
	/// );
	/// ```
	fn apply_first<'a, A: 'a + Clone, B>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, A>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(move |fb| match (fa.to_owned(), fb) {
			(Err(a), Err(_a)) => Err(a),
			(Ok(e), _) | (_, Ok(e)) => Ok(e),
		})
	}
}

impl<T> ApplySecond for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{apply_second, identity}};
	///
	/// assert_eq!(
	///     apply_second::<ResultWithOkBrand<_>, bool, bool>(Ok(()))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_second::<ResultWithOkBrand<_>, bool, _>(Ok(()))(Err(false)),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_second::<ResultWithOkBrand<_>, _, bool>(Err(true))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_second::<ResultWithOkBrand<()>, _, _>(Err(true))(Err(false)),
	///     Err(false)
	/// );
	/// ```
	fn apply_second<'a, A: 'a, B: 'a + Clone>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(move |fb| match (fa.to_owned(), fb) {
			(Err(_a), Err(a)) => Err(a),
			(Ok(e), _) | (_, Ok(e)) => Ok(e),
		})
	}
}

impl<T> Bind for ResultWithOkBrand<T>
where
	T: Clone,
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{bind, pure}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     bind::<ResultWithOkBrand<_>, _, _>(Ok(()))(Arc::new(pure::<ResultWithOkBrand<_>, ()>)),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     bind::<ResultWithOkBrand<()>, _, _>(Err(()))(Arc::new(pure::<ResultWithOkBrand<_>, _>)),
	///     Err(())
	/// );
	/// ```
	fn bind<'a, A: 'a + Clone, B>(
		ma: Apply1<Self, A>
	) -> ArcFn<'a, ArcFn<'a, A, Apply1<Self, B>>, Apply1<Self, B>> {
		Arc::new(move |f| ma.to_owned().or_else(|a| -> Result<_, B> { f(a) }))
	}
}

impl<T> Foldable for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::fold_right};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_right::<ResultWithOkBrand<i32>, _, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(Err(1)),
	///     2
	/// );
	/// assert_eq!(
	///     fold_right::<ResultWithOkBrand<_>, i32, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(Ok(())),
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
					(_, b, Ok(_)) => b,
					(f, b, Err(a)) => f(a)(b),
				}
			})
		})
	}
}

impl<'a, T> Traversable<'a> for ResultWithOkBrand<T> {
	fn traverse<F: Applicative, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, Apply1<F, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<F, Apply1<Self, B>>>
	where
		Apply1<F, B>: 'a + Clone,
		Apply1<F, ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>>: Clone,
	{
		Arc::new(move |ta| match (f.clone(), ta) {
			(_, Ok(e)) => pure::<F, _>(Ok(e)),
			(f, Err(a)) => map::<F, B, _>(Arc::new(pure::<Self, _>))(f(a)),
		})
	}
}
