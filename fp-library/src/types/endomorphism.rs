//! Implementations for [`Endomorphism`], a wrapper for endomorphisms (morphisms from an object to the same object) that enables monoidal operations.

use core::fmt;
use std::{
	fmt::{Debug, Formatter},
	hash::Hash,
	marker::PhantomData,
};

use crate::{
	classes::{
		Category, ClonableFn, Monoid, Semigroup, clonable_fn::ApplyFn, monoid::Monoid1L0T,
		semigroup::Semigroup1L0T,
	},
	hkt::{Apply1L2T, Kind1L0T},
};

/// A wrapper for endomorphisms (morphisms from an object to the same object) that enables monoidal operations.
///
/// `Endomorphism c a` represents a morphism `c a a` where `c` is a `Category`.
/// For the category of functions, this represents functions of type `a -> a`.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation [append][Semigroup::append] is [morphism composition][crate::classes::Semigroupoid::compose].
/// * The identity element [empty][Monoid::empty] is the [identity morphism][Category::identity].
///
/// The wrapped morphism can be accessed directly via the [`.0` field][Endomorphism#structfield.0].
pub struct Endomorphism<'a, CategoryBrand: Category, A: 'a>(pub Apply1L2T<'a, CategoryBrand, A, A>);

impl<'a, CategoryBrand: Category, A> Endomorphism<'a, CategoryBrand, A> {
	pub fn new(a: Apply1L2T<'a, CategoryBrand, A, A>) -> Self {
		Self(a)
	}
}

impl<'a, CategoryBrand: Category, A> Clone for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
	fn clone(&self) -> Self {
		Endomorphism(self.0.clone())
	}
}

impl<'a, CategoryBrand: Category, A> Debug for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("Endomorphism").field(&self.0).finish()
	}
}

impl<'a, CategoryBrand: 'a + Category, A> Eq for Endomorphism<'a, CategoryBrand, A> where
	Apply1L2T<'a, CategoryBrand, A, A>: Eq
{
}

impl<'a, CategoryBrand: Category, A> Hash for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
	}
}

impl<'a, CategoryBrand: 'a + Category, A> Ord for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, CategoryBrand: Category, A> PartialEq for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0
	}
}

impl<'a, CategoryBrand: Category, A> PartialOrd for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'b, CategoryBrand: 'b + Category, A> Semigroup<'b> for Endomorphism<'b, CategoryBrand, A>
where
	Apply1L2T<'b, CategoryBrand, A, A>: Clone,
{
	fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
		a: Self
	) -> ApplyFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'b: 'a,
	{
		ClonableFnBrand::new(move |b: Self| {
			Endomorphism(CategoryBrand::compose::<'b, ClonableFnBrand, _, _, _>(a.0.clone())(b.0))
		})
	}
}

impl<'a, CategoryBrand: 'a + Category, A> Monoid<'a> for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
	fn empty() -> Self {
		Endomorphism(CategoryBrand::identity::<'a, _>())
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndomorphismBrand<CategoryBrand: Category, A>(PhantomData<(CategoryBrand, A)>);

impl<CategoryBrand: Category, A: 'static> Kind1L0T for EndomorphismBrand<CategoryBrand, A> {
	type Output<'a> = Endomorphism<'a, CategoryBrand, A>;
}

impl<CategoryBrand: 'static + Category, A: 'static> Semigroup1L0T
	for EndomorphismBrand<CategoryBrand, A>
where
	for<'a> Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
}

impl<CategoryBrand: 'static + Category, A: 'static> Monoid1L0T
	for EndomorphismBrand<CategoryBrand, A>
where
	for<'a> Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
}
