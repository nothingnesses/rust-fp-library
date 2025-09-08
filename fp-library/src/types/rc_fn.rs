//! Implementations for [reference-counted][std::rc::Rc] [closures][Fn]
//! (`Rc<dyn Fn(A) -> B>`).

use crate::{
	classes::{
		Category, ClonableFn, Function, Monoid, Semigroup, Semigroupoid,
		clonable_fn::ApplyClonableFn, function::ApplyFunction,
	},
	functions::{compose, identity},
	hkt::{Apply1L2T, Kind1L2T},
};
use std::rc::Rc;

/// A brand type for [reference-counted][std::rc::Rc] [closures][Fn]
/// (`Rc<dyn Fn(A) -> B>`).
///
/// This struct implements [`ClonableFn`] to provide a way to construct and
/// type-check [`Rc`]-wrapped closures in a generic context. The lifetime `'a`
/// ensures the closure doesn't outlive referenced data, while `A` and `B`
/// represent input and output types.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFnBrand;

impl Kind1L2T for RcFnBrand {
	type Output<'a, A, B> = Rc<dyn 'a + Fn(A) -> B>;
}

impl Function for RcFnBrand {
	type Output<'a, A: 'a, B: 'a> = Apply1L2T<'a, Self, A, B>;

	fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> ApplyFunction<'a, Self, A, B> {
		Rc::new(f)
	}
}

impl ClonableFn for RcFnBrand {
	type Output<'a, A: 'a, B: 'a> = Apply1L2T<'a, Self, A, B>;

	fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> ApplyClonableFn<'a, Self, A, B> {
		Rc::new(f)
	}
}

impl Semigroupoid for RcFnBrand {
	fn compose<'a, ClonableFnBrand: 'a + ClonableFn, B, C, D>(
		f: Apply1L2T<'a, Self, C, D>
	) -> ApplyClonableFn<'a, ClonableFnBrand, Apply1L2T<'a, Self, B, C>, Apply1L2T<'a, Self, B, D>>
	{
		<ClonableFnBrand as ClonableFn>::new::<'a, _, _>(move |g: Apply1L2T<'a, Self, B, C>| {
			<Self as ClonableFn>::new::<'a, _, _>({
				let f = f.clone();
				move |a| compose::<'a, Self, _, _, _>(f.clone())(g.clone())(a)
			})
		})
	}
}

impl Category for RcFnBrand {
	fn identity<'a, A: 'a>() -> Apply1L2T<'a, Self, A, A> {
		<Self as ClonableFn>::new::<'a, _, _>(identity)
	}
}

impl<'b, A: 'b + Clone, B: Semigroup<'b> + 'b> Semigroup<'b> for Rc<dyn 'b + Fn(A) -> B> {
	fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
		a: Self
	) -> ApplyClonableFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'b: 'a,
	{
		<ClonableFnBrand as ClonableFn>::new(move |b: Self| {
			<RcFnBrand as ClonableFn>::new({
				let a = a.clone();
				move |c: A| B::append::<ClonableFnBrand>(a(c.clone()))(b(c))
			})
		})
	}
}

impl<'b, A: 'b + Clone, B: Monoid<'b> + 'b> Monoid<'b> for Rc<dyn 'b + Fn(A) -> B> {
	fn empty() -> Self {
		<RcFnBrand as ClonableFn>::new(move |_| B::empty())
	}
}
