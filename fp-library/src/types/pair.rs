//! Implementations for [`Pair`], a type that wraps two values.

pub mod pair_with_first;
pub mod pair_with_second;

use crate::hkt::Kind2;
pub use pair_with_first::*;
pub use pair_with_second::*;

/// Wraps two values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<First, Second>(pub First, pub Second);

pub struct PairBrand;

impl Kind2 for PairBrand {
	type Output<A, B> = Pair<A, B>;
}

impl<First, Second> Pair<First, Second>
where
	First: Clone,
{
	pub fn new(first: First) -> impl Fn(Second) -> Self {
		move |second| Pair(first.to_owned(), second)
	}
}
