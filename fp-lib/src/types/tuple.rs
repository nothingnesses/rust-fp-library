use crate::{
	hkt::{Apply, Apply2, Kind2},
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
