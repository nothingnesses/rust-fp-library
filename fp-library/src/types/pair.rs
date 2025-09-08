//! Implementations for [`Pair`], a type that wraps two values.

pub mod pair_with_first;
pub mod pair_with_second;

use crate::{
	classes::{ClonableFn, clonable_fn::ApplyClonableFn},
	hkt::Kind0L2T,
};
pub use pair_with_first::*;
pub use pair_with_second::*;

/// Wraps two values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<First, Second>(pub First, pub Second);

pub struct PairBrand;

impl Kind0L2T for PairBrand {
	type Output<A, B> = Pair<A, B>;
}

impl<'a, First, Second> Pair<First, Second>
where
	First: 'a + Clone,
{
	pub fn new<ClonableFnBrand: 'a + ClonableFn>(
		first: First
	) -> ApplyClonableFn<'a, ClonableFnBrand, Second, Self> {
		<ClonableFnBrand as ClonableFn>::new(move |second| Pair(first.to_owned(), second))
	}
}
