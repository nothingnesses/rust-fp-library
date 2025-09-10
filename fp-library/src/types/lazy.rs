//! Implementations for [`Lazy`], the type of lazily-computed, memoized values.

use crate::classes::{ClonableFn, Defer, Semigroup, clonable_fn::ApplyClonableFn};
use core::fmt;
use std::{
	fmt::{Debug, Formatter},
	hash::Hash,
};

/// Represents a lazily-computed, memoized value.
pub struct Lazy<'a, ClonableFnBrand: ClonableFn, A: 'a>(
	pub Option<A>,
	pub ApplyClonableFn<'a, ClonableFnBrand, (), A>,
);

impl<'a, ClonableFnBrand: ClonableFn, A> Lazy<'a, ClonableFnBrand, A> {
	pub fn new(a: ApplyClonableFn<'a, ClonableFnBrand, (), A>) -> Self {
		Self(None, a)
	}

	pub fn evaluate(a: Self) -> Self {
		match a {
			Self(Some(_), _) => a,
			Self(_, f) => Self(Some(f(())), f.clone()),
		}
	}

	pub fn force(a: Self) -> A {
		Self::evaluate(a).0.unwrap()
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

impl<'b, CFB: 'b + ClonableFn, A: Semigroup<'b> + Clone> Semigroup<'b> for Lazy<'b, CFB, A> {
	fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
		a: Self
	) -> ApplyClonableFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'b: 'a,
	{
		<ClonableFnBrand as ClonableFn>::new(move |b: Self| {
			Self::new(<CFB as ClonableFn>::new({
				let a = a.clone();
				move |_: ()| {
					A::append::<ClonableFnBrand>(Lazy::force(a.clone()))(Lazy::force(b.clone()))
				}
			}))
		})
	}
}

impl<'a, CFB: ClonableFn, A> Defer<'a> for Lazy<'a, CFB, A> {
	fn defer<ClonableFnBrand: 'a + ClonableFn>(
		f: ApplyClonableFn<'a, ClonableFnBrand, (), Self>
	) -> Self
	where
		Self: Sized,
	{
		Self::new(<CFB as ClonableFn>::new(move |_| Lazy::<'a, CFB, A>::force(f(()))))
	}
}
