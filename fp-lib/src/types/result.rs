use crate::{
	functions::map,
	hkt::{Apply, Kind, Kind2},
	typeclasses::{Functor, Sequence},
};

pub struct ResultWithErrBrand<E>(E);

pub struct ResultBrand;

impl<A, B> Kind2<A, B> for ResultBrand {
	type Output = Result<B, A>;
}

impl<E, A> Kind<A> for ResultWithErrBrand<E> {
	type Output = Result<A, E>;
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
		move |fa| match fa {
			Err(e) => Err(e.to_owned()),
			Ok(a) => Ok(f(a)),
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
		move |fa| match (&ff, &fa) {
			(Err(e), _) => Err(e.to_owned()),
			(Ok(f), _) => map::<ResultWithErrBrand<_>, _, _, _>(f)(fa),
		}
	}
}
