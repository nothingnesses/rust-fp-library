use crate::{hkt::{Apply, Kind}, typeclasses::{Functor, Pure, Sequence}};


#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Solo<A>(pub A);

pub struct SoloBrand;

impl<A> Kind<A> for SoloBrand {
	type Output = Solo<A>;
}

impl Functor for SoloBrand {
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
	{
		move |fa| Solo(f(fa.0))
	}
}

impl Pure for SoloBrand {
	fn pure<A>(a: A) -> Apply<Self, A>
	{
		Solo(a)
	}
}

impl Sequence for SoloBrand {
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
	{
		move |fa| Solo(ff.0(fa.0))
	}
}
