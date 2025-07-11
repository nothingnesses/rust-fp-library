use crate::{
	brands::Brand,
	functions::map,
	hkt::{Apply, Kind, Kind2},
	typeclasses::{Bind, Functor, Sequence},
};

pub struct ResultWithErrBrand<E>(E);

impl<A, E> Kind<A> for ResultWithErrBrand<E> {
	type Output = Result<A, E>;
}

impl<A, E> Brand<Result<A, E>, A> for ResultWithErrBrand<E> {
	fn inject(a: &Result<A, E>) -> &Apply<Self, A> {
		a
	}
	fn project(a: &Apply<Self, A>) -> &Result<A, E> {
		a
	}
}

impl<E> Bind for ResultWithErrBrand<E>
where
	E: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithErrBrand, functions::bind};
	///
	/// let zero = Ok(0);
	/// let add_one = |a: &_| Ok(a + 1);
	/// assert_eq!(bind::<ResultWithErrBrand<()>, _, _, _>(&zero)(&add_one), Ok(1));
	/// ```
	fn bind<F, A, B>(ma: &Apply<Self, A>) -> impl Fn(&F) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B>,
		F: Fn(&A) -> Apply<Self, B>,
		Apply<Self, B>: Clone,
	{
		|f| match ResultWithErrBrand::project(ma) {
			Ok(a) => f(a),
			Err(e) => {
				(*<ResultWithErrBrand<E> as Brand<Result<B, _>, _>>::inject(&Err((*e).to_owned())))
					.to_owned()
			}
		}
	}
}

impl<E> Functor for ResultWithErrBrand<E>
where
	E: Clone,
{
	fn map<F, A, B>(f: &F) -> impl Fn(&Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(&A) -> B,
		A: Clone,
	{
		|fa| match fa {
			Ok(a) => Ok(f(a)),
			Err(e) => Err(e.to_owned()),
		}
	}
}

impl<E> Sequence for ResultWithErrBrand<E>
where
	E: Clone,
{
	fn sequence<F, A, B>(ff: &Apply<Self, F>) -> impl Fn(&Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(&A) -> B,
		A: Clone,
	{
		move |fa| match (ff, fa) {
			(Ok(f), _) => map::<ResultWithErrBrand<_>, _, _, _>(f)(fa),
			(Err(e), _) => Err(e.to_owned()),
		}
	}
}

pub struct ResultBrand;

impl<A, B> Kind2<A, B> for ResultBrand {
	type Output = Result<B, A>;
}
