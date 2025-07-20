//! Implementations for `Tuple`, a type that wraps two values.

use crate::hkt::Kind2;

/// Wraps two values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tuple<A, B>(pub A, pub B);

/// Brand for [`Tuple`](../tuple/struct.Tuple.html).
pub struct TupleBrand;

impl<A, B> Kind2<A, B> for TupleBrand {
	type Output = Tuple<A, B>;
}
