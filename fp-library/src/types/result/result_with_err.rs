//! Partially applied form of `Result` with the `Err` constructor filled in.

use crate::{
	brands::Brand,
	functions::map,
	hkt::{Apply, Kind},
	typeclasses::{Bind, Functor, Pure, Sequence},
};

pub struct ResultWithErrBrand<E>(E);

impl<A, E> Kind<A> for ResultWithErrBrand<E> {
	type Output = Result<A, E>;
}

impl<A, E> Brand<Result<A, E>, A> for ResultWithErrBrand<E> {
	fn inject(a: Result<A, E>) -> Apply<Self, A> {
		a
	}
	fn project(a: Apply<Self, A>) -> Result<A, E> {
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
	fn pure<A>(a: A) -> Apply<Self, A>
	where
		Self: Kind<A>,
	{
		Self::inject(Ok(a))
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
	/// assert_eq!(map::<ResultWithErrBrand<()>, _, _, _>(identity)(Ok(())), Ok(()));
	/// assert_eq!(map::<ResultWithErrBrand<_>, _, _, _>(identity::<()>)(Err(())), Err(()));
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B>,
		F: Fn(A) -> B,
	{
		move |fa| ResultWithErrBrand::inject(ResultWithErrBrand::project(fa).map(&f))
	}
}

impl<E> Sequence for ResultWithErrBrand<E>
where
	E: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::{identity, sequence}};
	///
	/// assert_eq!(sequence::<ResultWithErrBrand<()>, _, _, _>(Ok(identity))(Ok(())), Ok(()));
	/// assert_eq!(sequence::<ResultWithErrBrand<_>, _, _, _>(Ok(identity::<()>))(Err(())), Err(()));
	/// assert_eq!(sequence::<ResultWithErrBrand<_>, fn(()) -> (), _, _>(Err(()))(Ok(())), Err(()));
	/// assert_eq!(sequence::<ResultWithErrBrand<_>, fn(()) -> (), _, _>(Err(()))(Err(())), Err(()));
	/// ```
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<F> + Kind<A> + Kind<B>,
		F: Fn(A) -> B,
		Apply<Self, F>: Clone,
	{
		move |fa| match (ResultWithErrBrand::project(ff.to_owned()), &fa) {
			(Ok(f), _) => map::<ResultWithErrBrand<_>, F, _, _>(f)(fa),
			(Err(e), _) => ResultWithErrBrand::inject(Err::<B, _>(e)),
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
	/// assert_eq!(bind::<ResultWithErrBrand<()>, _, _, _>(Ok(()))(pure::<ResultWithErrBrand<_>, _>), Ok(()));
	/// assert_eq!(bind::<ResultWithErrBrand<_>, _, _, _>(Err(()))(pure::<ResultWithErrBrand<_>, ()>), Err(()));
	/// ```
	fn bind<F, A, B>(ma: Apply<Self, A>) -> impl Fn(F) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B> + Sized,
		F: Fn(A) -> Apply<Self, B>,
		Apply<Self, A>: Clone,
	{
		move |f| {
			ResultWithErrBrand::inject(
				ResultWithErrBrand::project(ma.to_owned())
					.and_then(|a| -> Result<B, _> { ResultWithErrBrand::project(f(a)) }),
			)
		}
	}
}
