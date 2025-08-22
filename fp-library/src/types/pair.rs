//! Implementations for [`Pair`], a type that wraps two values.

pub mod pair_with_first;
pub mod pair_with_second;

use std::sync::Arc;

use crate::{aliases::ArcFn, hkt::Kind2};
pub use pair_with_first::*;
pub use pair_with_second::*;

/// Wraps two values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<First, Second>(pub First, pub Second);

pub struct PairBrand;

impl Kind2 for PairBrand {
	type Output<A, B> = Pair<A, B>;
}

impl<'a, First, Second> Pair<First, Second>
where
	First: 'a + Clone,
{
	pub fn new(first: First) -> ArcFn<'a, Second, Self> {
		Arc::new(move |second| Pair(first.to_owned(), second))
	}
}
