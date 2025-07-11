use crate::hkt::Kind2;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tuple<A, B>(pub A, pub B);

pub struct TupleBrand;

impl<A, B> Kind2<A, B> for TupleBrand {
	type Output = Tuple<A, B>;
}
