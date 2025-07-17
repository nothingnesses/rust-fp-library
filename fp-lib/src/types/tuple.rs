use crate::{
	hkt::{Apply, Apply2, Kind, Kind2},
	typeclasses::{Functor, Pure, Sequence},
	types::SoloBrand,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tuple<A, B>(A, B);

pub struct TupleBrand;

impl<A, B> Kind2<A, B> for TupleBrand {
	type Output = Tuple<A, B>;
}

impl TupleBrand {
	pub fn from_solos<A: Clone, B>(
		a: Apply<SoloBrand, A>
	) -> impl Fn(Apply<SoloBrand, B>) -> Apply2<Self, A, B> {
		move |b| Tuple(a.0.clone(), b.0)
	}
}

// impl Functor for TupleBrand {
// 	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
// 	where
// 		F: Fn(A) -> B + Copy,
// 	{
// 		move |fa| Tuple(f(fa.0))
// 	}
// }

// impl Pure for TupleBrand {
// 	fn pure<A>(a: A) -> Apply<Self, A>
// 	{
// 		Tuple(a)
// 	}
// }

// impl Sequence for TupleBrand {
// 	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
// 	where
// 		F: Fn(A) -> B + Copy,
// 	{
// 		move |fa| Tuple(ff.0(fa.0))
// 	}
// }
