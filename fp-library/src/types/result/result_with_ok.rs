//! Implementations for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.

use crate::{
	functions::map,
	hkt::{Apply, Brand, Brand1, Kind, Kind1},
	typeclasses::{Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Functor, Pure},
};

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.
pub struct ResultWithOkBrand<T>(T);

impl<A, T> Kind1<A> for ResultWithOkBrand<T> {
	type Output = Result<T, A>;
}

impl<A, T> Brand1<Result<T, A>, A> for ResultWithOkBrand<T> {
	fn inject(a: Result<T, A>) -> Apply<Self, (A,)> {
		a
	}
	fn project(a: Apply<Self, (A,)>) -> Result<T, A> {
		a
	}
}

impl<T> Pure for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::pure};
	///
	/// assert_eq!(pure::<ResultWithOkBrand<()>, _>(()), Err(()));
	fn pure<A>(a: A) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)>,
	{
		<Self as Brand<_, _>>::inject(Err(a))
	}
}

impl<T> Functor for ResultWithOkBrand<T>
where
	T: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{identity, map}};
	///
	/// assert_eq!(map::<ResultWithOkBrand<_>, _, _, _>(identity::<()>)(Ok(true)), Ok(true));
	/// assert_eq!(map::<ResultWithOkBrand<bool>, _, _, _>(identity)(Err(())), Err(()));
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
	{
		move |fa| {
			<Self as Brand<_, _>>::inject(match <Self as Brand<_, _>>::project(fa) {
				Ok(a) => Ok(a),
				Err(e) => Err(f(e)),
			})
		}
	}
}

impl<T> TypeclassApply for ResultWithOkBrand<T>
where
	T: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{apply, identity}};
	///
	/// assert_eq!(apply::<ResultWithOkBrand<_>, fn(()) -> (), _, _>(Ok(true))(Ok(true)), Ok(true));
	/// assert_eq!(apply::<ResultWithOkBrand<_>, fn(()) -> (), _, _>(Ok(true))(Err(())), Ok(true));
	/// assert_eq!(apply::<ResultWithOkBrand<_>, _, _, _>(Err(identity::<()>))(Ok(true)), Ok(true));
	/// assert_eq!(apply::<ResultWithOkBrand<bool>, _, _, _>(Err(identity))(Err(())), Err(()));
	/// ```
	fn apply<F, A, B>(ff: Apply<Self, (F,)>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
		Apply<Self, (F,)>: Clone,
	{
		move |fa| match (<Self as Brand<_, _>>::project(ff.to_owned()), &fa) {
			(Ok(e), _) => <Self as Brand<_, _>>::inject(Ok::<_, B>(e)),
			(Err(f), _) => map::<ResultWithOkBrand<_>, F, _, _>(f)(fa),
		}
	}
}

impl<T> ApplyFirst for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{apply_first, identity}};
	///
	/// assert_eq!(apply_first::<ResultWithOkBrand<_>, bool, bool>(Ok(()))(Ok(())), Ok(()));
	/// assert_eq!(apply_first::<ResultWithOkBrand<_>, bool, _>(Ok(()))(Err(false)), Ok(()));
	/// assert_eq!(apply_first::<ResultWithOkBrand<_>, _, bool>(Err(true))(Ok(())), Ok(()));
	/// assert_eq!(apply_first::<ResultWithOkBrand<()>, _, _>(Err(true))(Err(false)), Err(true));
	/// ```
	fn apply_first<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
	{
		move |fb| {
			<Self as Brand<_, (A,)>>::inject(
				match (
					<Self as Brand<_, _>>::project(fa.to_owned()),
					<Self as Brand<_, (B,)>>::project(fb),
				) {
					(Err(a), Err(_a)) => Err(a),
					(Ok(e), _) | (_, Ok(e)) => Ok(e),
				},
			)
		}
	}
}

impl<T> ApplySecond for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{apply_second, identity}};
	///
	/// assert_eq!(apply_second::<ResultWithOkBrand<_>, bool, bool>(Ok(()))(Ok(())), Ok(()));
	/// assert_eq!(apply_second::<ResultWithOkBrand<_>, bool, _>(Ok(()))(Err(false)), Ok(()));
	/// assert_eq!(apply_second::<ResultWithOkBrand<_>, _, bool>(Err(true))(Ok(())), Ok(()));
	/// assert_eq!(apply_second::<ResultWithOkBrand<()>, _, _>(Err(true))(Err(false)), Err(false));
	/// ```
	fn apply_second<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
	{
		move |fb| {
			<Self as Brand<_, (B,)>>::inject(
				match (
					<Self as Brand<_, (A,)>>::project(fa.to_owned()),
					<Self as Brand<_, _>>::project(fb),
				) {
					(Err(_a), Err(a)) => Err(a),
					(Ok(e), _) | (_, Ok(e)) => Ok(e),
				},
			)
		}
	}
}

impl<T> Bind for ResultWithOkBrand<T>
where
	T: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{bind, pure}};
	///
	/// assert_eq!(bind::<ResultWithOkBrand<_>, _, _, _>(Ok(()))(pure::<ResultWithOkBrand<_>, ()>), Ok(()));
	/// assert_eq!(bind::<ResultWithOkBrand<()>, _, _, _>(Err(()))(pure::<ResultWithOkBrand<_>, _>), Err(()));
	/// ```
	fn bind<F, A, B>(ma: Apply<Self, (A,)>) -> impl Fn(F) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Sized,
		F: Fn(A) -> Apply<Self, (B,)>,
		Apply<Self, (A,)>: Clone,
	{
		move |f| {
			<Self as Brand<_, _>>::inject(
				<Self as Brand<_, _>>::project(ma.to_owned())
					.or_else(|a| -> Result<_, B> { <Self as Brand<_, _>>::project(f(a)) }),
			)
		}
	}
}
