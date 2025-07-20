//! Implementations for the partially-applied form of `Result` with the `Ok` constructor filled in.

use crate::{
	brands::Brand,
	functions::map,
	hkt::{Apply, Kind},
	typeclasses::{Bind, Functor, Pure, Sequence},
};

/// Brand for the partially-applied form of `Result` with the `Ok` constructor filled in.
pub struct ResultWithOkBrand<T>(T);

impl<A, T> Kind<A> for ResultWithOkBrand<T> {
	type Output = Result<T, A>;
}

impl<A, T> Brand<Result<T, A>, A> for ResultWithOkBrand<T> {
	fn inject(a: Result<T, A>) -> Apply<Self, A> {
		a
	}
	fn project(a: Apply<Self, A>) -> Result<T, A> {
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
	fn pure<A>(a: A) -> Apply<Self, A>
	where
		Self: Kind<A>,
	{
		Self::inject(Err(a))
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
	/// assert_eq!(map::<ResultWithOkBrand<()>, _, _, _>(identity)(Err(())), Err(()));
	/// assert_eq!(map::<ResultWithOkBrand<_>, _, _, _>(identity::<()>)(Ok(())), Ok(()));
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B>,
		F: Fn(A) -> B,
	{
		move |fa| {
			ResultWithOkBrand::inject(match ResultWithOkBrand::project(fa) {
				Ok(a) => Ok(a),
				Err(e) => Err(f(e)),
			})
		}
	}
}

impl<T> Sequence for ResultWithOkBrand<T>
where
	T: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{identity, sequence}};
	///
	/// assert_eq!(sequence::<ResultWithOkBrand<()>, _, _, _>(Err(identity))(Err(())), Err(()));
	/// assert_eq!(sequence::<ResultWithOkBrand<_>, _, _, _>(Err(identity::<()>))(Ok(())), Ok(()));
	/// assert_eq!(sequence::<ResultWithOkBrand<_>, fn(()) -> (), _, _>(Ok(()))(Err(())), Ok(()));
	/// assert_eq!(sequence::<ResultWithOkBrand<_>, fn(()) -> (), _, _>(Ok(()))(Ok(())), Ok(()));
	/// ```
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<F> + Kind<A> + Kind<B>,
		F: Fn(A) -> B,
		Apply<Self, F>: Clone,
	{
		move |fa| match (ResultWithOkBrand::project(ff.to_owned()), &fa) {
			(Ok(e), _) => ResultWithOkBrand::inject(Ok::<_, B>(e)),
			(Err(f), _) => map::<ResultWithOkBrand<_>, F, _, _>(f)(fa),
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
	/// assert_eq!(bind::<ResultWithOkBrand<()>, _, _, _>(Err(()))(pure::<ResultWithOkBrand<_>, _>), Err(()));
	/// assert_eq!(bind::<ResultWithOkBrand<_>, _, _, _>(Ok(()))(pure::<ResultWithOkBrand<_>, ()>), Ok(()));
	/// ```
	fn bind<F, A, B>(ma: Apply<Self, A>) -> impl Fn(F) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B> + Sized,
		F: Fn(A) -> Apply<Self, B>,
		Apply<Self, A>: Clone,
	{
		move |f| {
			ResultWithOkBrand::inject(
				ResultWithOkBrand::project(ma.to_owned())
					.or_else(|a| -> Result<_, B> { ResultWithOkBrand::project(f(a)) }),
			)
		}
	}
}
