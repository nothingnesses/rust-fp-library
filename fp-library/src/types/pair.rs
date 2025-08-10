//! Implementations for [`Pair`], a type that wraps two values.

pub mod pair_with_first;
pub mod pair_with_second;

use crate::{
	hkt::{Apply, Brand2, Kind2},
	impl_brand,
};
pub use pair_with_first::*;
pub use pair_with_second::*;

/// Wraps two values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<First, Second>(pub First, pub Second);

impl_brand!(PairBrand, Pair, Kind2, Brand2, (A, B));

impl<First, Second> Pair<First, Second>
where
	First: Clone,
{
	pub fn new(first: First) -> impl Fn(Second) -> Self {
		move |second| Pair(first.to_owned(), second)
	}
}
