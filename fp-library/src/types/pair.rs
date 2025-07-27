//! Implementations for [`Pair`], a type that wraps two values.

use crate::{
	brands::Brand2,
	hkt::{Apply, Kind2},
	impl_brand,
};

/// Wraps two values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<A, B>(pub A, pub B);

pub mod pair_with_first;
pub mod pair_with_second;

pub use pair_with_first::*;
pub use pair_with_second::*;

impl_brand!(
	/// [Brand][crate::brands] for [`Pair`].
	PairBrand,
	Pair,
	Kind2,
	Brand2,
	(A, B)
);
