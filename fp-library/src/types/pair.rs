//! Implementations for `Pair`, a type that wraps two values.

use crate::hkt::Kind2;

/// Wraps two values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<A, B>(pub A, pub B);

/// Brand for [`Pair`](../pair/struct.Pair.html).
pub struct PairBrand;

impl<A, B> Kind2<A, B> for PairBrand {
	type Output = Pair<A, B>;
}
