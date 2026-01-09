//! Implementations for [`Lazy`], the type of lazily-computed, memoized values.

use crate::{
	classes::{
		ClonableFn, Defer, Monoid, Once, Semigroup, clonable_fn::ApplyClonableFn, once::ApplyOnce,
	},
	hkt::Kind0L1T,
};
use core::fmt;
use std::{
	fmt::{Debug, Formatter},
	hash::{Hash, Hasher},
};

/// Represents a lazily-computed, memoized value.
pub struct Lazy<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A: 'a>(
	pub ApplyOnce<OnceBrand, A>,
	pub ApplyClonableFn<'a, ClonableFnBrand, (), A>,
);

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A> Lazy<'a, OnceBrand, ClonableFnBrand, A> {
	pub fn new(a: ApplyClonableFn<'a, ClonableFnBrand, (), A>) -> Self {
		Self(OnceBrand::new(), a)
	}

	pub fn force(a: Self) -> A
	where
		A: Clone,
	{
		<OnceBrand as Once>::get_or_init(&a.0, move || (a.1)(())).clone()
	}
}

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A: 'a + Clone> Clone
	for Lazy<'a, OnceBrand, ClonableFnBrand, A>
where
	ApplyOnce<OnceBrand, A>: Clone,
{
	fn clone(&self) -> Self {
		Self(self.0.clone(), self.1.clone())
	}
}

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A: Debug> Debug
	for Lazy<'a, OnceBrand, ClonableFnBrand, A>
where
	ApplyOnce<OnceBrand, A>: Debug,
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("Lazy").field(&self.0).field(&self.1).finish()
	}
}

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A: Eq> Eq
	for Lazy<'a, OnceBrand, ClonableFnBrand, A>
where
	ApplyOnce<OnceBrand, A>: Eq,
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: Eq,
{
}

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A: Hash> Hash
	for Lazy<'a, OnceBrand, ClonableFnBrand, A>
where
	ApplyOnce<OnceBrand, A>: Hash,
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: Hash,
{
	fn hash<H: Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
		self.1.hash(state);
	}
}

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A: Ord> Ord
	for Lazy<'a, OnceBrand, ClonableFnBrand, A>
where
	ApplyOnce<OnceBrand, A>: Ord,
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A: PartialEq> PartialEq
	for Lazy<'a, OnceBrand, ClonableFnBrand, A>
where
	ApplyOnce<OnceBrand, A>: PartialEq,
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0 && self.1 == other.1
	}
}

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A: PartialOrd> PartialOrd
	for Lazy<'a, OnceBrand, ClonableFnBrand, A>
where
	ApplyOnce<OnceBrand, A>: PartialOrd,
	ApplyClonableFn<'a, ClonableFnBrand, (), A>: PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'b, OnceBrand: 'b + Once, CFB: 'b + ClonableFn, A: Semigroup<'b> + Clone> Semigroup<'b>
	for Lazy<'b, OnceBrand, CFB, A>
where
	ApplyOnce<OnceBrand, A>: Clone,
{
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

impl<'b, OnceBrand: 'b + Once, CFB: 'b + ClonableFn, A: Monoid<'b> + Clone> Monoid<'b>
	for Lazy<'b, OnceBrand, CFB, A>
where
	ApplyOnce<OnceBrand, A>: Clone,
{
	fn empty() -> Self {
		Self::new(<CFB as ClonableFn>::new(move |_| <A as Monoid<'b>>::empty()))
	}
}

impl<'a, OnceBrand: Once, CFB: ClonableFn, A: Clone> Defer<'a> for Lazy<'a, OnceBrand, CFB, A> {
	fn defer<ClonableFnBrand: 'a + ClonableFn>(
		f: ApplyClonableFn<'a, ClonableFnBrand, (), Self>
	) -> Self
	where
		Self: Sized,
	{
		Self::new(<CFB as ClonableFn>::new(move |_| Lazy::<'a, OnceBrand, CFB, A>::force(f(()))))
	}
}

pub struct LazyBrand<OnceBrand: Once, ClonableFnBrand: ClonableFn>(OnceBrand, ClonableFnBrand);

// impl<OnceBrand: Once, ClonableFnBrand: ClonableFn> Kind0L1T
// 	for LazyBrand<OnceBrand, ClonableFnBrand>
// where
// 	A: 'static,
// {
// 	type Output<A> = Lazy<'static, OnceBrand, ClonableFnBrand, A>;
// }
