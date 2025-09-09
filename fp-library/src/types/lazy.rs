//! Implementations for [`Lazy`], the type of lazily-computed, memoized values.

use core::fmt;
use std::{
	fmt::{Debug, Formatter},
	hash::Hash,
};

use crate::classes::{ClonableFn, Defer, clonable_fn::ApplyClonableFn};

/// Represents a lazily-computed, memoized value.
pub struct Lazy<'a, ClonableFnBrand: ClonableFn, A: 'a>(
	pub Option<A>,
	pub ApplyClonableFn<'a, ClonableFnBrand, (), A>,
);

impl<'a, ClonableFnBrand: ClonableFn, A> Lazy<'a, ClonableFnBrand, A> {
	pub fn new(a: impl 'a + Fn(()) -> A) -> Self {
		Self(None, <ClonableFnBrand as ClonableFn>::new(a))
	}

	pub fn force(&mut self) -> A
	where
		A: Clone,
	{
		match self {
			Self(Some(a), _) => a.clone(),
			Self(_, f) => {
				let a = f(());
				self.0 = Some(a.clone());
				a
			}
		}
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A: 'a + Clone> Clone for Lazy<'a, ClonableFnBrand, A> {
	fn clone(&self) -> Self {
		Self(self.0.clone(), self.1.clone())
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A: Debug> Debug for Lazy<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("Lazy").field(&self.0).field(&self.1).finish()
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A: Eq> Eq for Lazy<'a, ClonableFnBrand, A> where
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: Eq
{
}

impl<'a, ClonableFnBrand: ClonableFn, A: Hash> Hash for Lazy<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
		self.1.hash(state);
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A: Ord> Ord for Lazy<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A: PartialEq> PartialEq for Lazy<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0 && self.1 == other.1
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A: PartialOrd> PartialOrd for Lazy<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A: Clone> Defer<'a> for Lazy<'a, ClonableFnBrand, A> {
	fn defer(f: impl 'a + Fn(()) -> Self) -> Self {
		Self::defer(<ClonableFnBrand as ClonableFn>::new(|_| f(()).force()))
	}
}
