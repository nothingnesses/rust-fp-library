//! Implementations for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.

use crate::{
	functions::map,
	hkt::{Apply, Brand, Brand1, Kind, Kind1},
	typeclasses::{Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Functor, Pure},
};

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Err`] constructor filled in.
pub struct ResultWithErrBrand<E>(E);

impl<A, E> Kind1<A> for ResultWithErrBrand<E> {
	type Output = Result<A, E>;
}

impl<A, E> Brand1<Result<A, E>, A> for ResultWithErrBrand<E> {
	fn inject(a: Result<A, E>) -> Apply<Self, (A,)> {
		a
	}
	fn project(a: Apply<Self, (A,)>) -> Result<A, E> {
		a
	}
}

impl<E> Pure for ResultWithErrBrand<E> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::pure};
	///
	/// assert_eq!(pure::<ResultWithErrBrand<()>, _>(()), Ok(()));
	fn pure<A>(a: A) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)>,
	{
		<Self as Brand<_, _>>::inject(Ok(a))
	}
}

impl<E> Functor for ResultWithErrBrand<E>
where
	E: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{identity, map}};
	///
	/// assert_eq!(map::<ResultWithErrBrand<_>, _, _, _>(identity::<()>)(Err(true)), Err(true));
	/// assert_eq!(map::<ResultWithErrBrand<bool>, _, _, _>(identity)(Ok(())), Ok(()));
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
	{
		move |fa| <Self as Brand<_, _>>::inject(<Self as Brand<_, _>>::project(fa).map(&f))
	}
}

impl<E> TypeclassApply for ResultWithErrBrand<E>
where
	E: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{apply, identity}};
	///
	/// assert_eq!(apply::<ResultWithErrBrand<_>, fn(()) -> (), _, _>(Err(true))(Err(true)), Err(true));
	/// assert_eq!(apply::<ResultWithErrBrand<_>, fn(()) -> (), _, _>(Err(true))(Ok(())), Err(true));
	/// assert_eq!(apply::<ResultWithErrBrand<_>, _, _, _>(Ok(identity::<()>))(Err(true)), Err(true));
	/// assert_eq!(apply::<ResultWithErrBrand<bool>, _, _, _>(Ok(identity))(Ok(())), Ok(()));
	/// ```
	fn apply<F, A, B>(ff: Apply<Self, (F,)>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
		Apply<Self, (F,)>: Clone,
	{
		move |fa| match (<Self as Brand<_, _>>::project(ff.to_owned()), &fa) {
			(Ok(f), _) => map::<ResultWithErrBrand<_>, F, _, _>(f)(fa),
			(Err(e), _) => <Self as Brand<_, _>>::inject(Err::<B, _>(e)),
		}
	}
}

impl<E> ApplyFirst for ResultWithErrBrand<E> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{apply_first, identity}};
	///
	/// assert_eq!(apply_first::<ResultWithErrBrand<_>, bool, bool>(Err(()))(Err(())), Err(()));
	/// assert_eq!(apply_first::<ResultWithErrBrand<_>, bool, _>(Err(()))(Ok(false)), Err(()));
	/// assert_eq!(apply_first::<ResultWithErrBrand<_>, _, bool>(Ok(true))(Err(())), Err(()));
	/// assert_eq!(apply_first::<ResultWithErrBrand<()>, _, _>(Ok(true))(Ok(false)), Ok(true));
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
					(Ok(a), Ok(_a)) => Ok(a),
					(Err(e), _) | (_, Err(e)) => Err(e),
				},
			)
		}
	}
}

impl<E> ApplySecond for ResultWithErrBrand<E> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{apply_second, identity}};
	///
	/// assert_eq!(apply_second::<ResultWithErrBrand<_>, bool, bool>(Err(()))(Err(())), Err(()));
	/// assert_eq!(apply_second::<ResultWithErrBrand<_>, bool, _>(Err(()))(Ok(false)), Err(()));
	/// assert_eq!(apply_second::<ResultWithErrBrand<_>, _, bool>(Ok(true))(Err(())), Err(()));
	/// assert_eq!(apply_second::<ResultWithErrBrand<()>, _, _>(Ok(true))(Ok(false)), Ok(false));
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
					(Ok(_a), Ok(a)) => Ok(a),
					(Err(e), _) | (_, Err(e)) => Err(e),
				},
			)
		}
	}
}

impl<E> Bind for ResultWithErrBrand<E>
where
	E: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{bind, pure}};
	///
	/// assert_eq!(bind::<ResultWithErrBrand<_>, _, _, _>(Err(()))(pure::<ResultWithErrBrand<_>, ()>), Err(()));
	/// assert_eq!(bind::<ResultWithErrBrand<()>, _, _, _>(Ok(()))(pure::<ResultWithErrBrand<_>, _>), Ok(()));
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
					.and_then(|a| -> Result<B, _> { <Self as Brand<_, _>>::project(f(a)) }),
			)
		}
	}
}
