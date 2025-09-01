use crate::{
	classes::{Category, ClonableFn, Semigroup, Semigroupoid, clonable_fn::ApplyFn},
	functions::{append, compose, identity},
	hkt::{Apply1L2T, Kind0L1T, Kind1L2T},
};
use core::fmt;
use std::{
	fmt::{Debug, Formatter},
	marker::PhantomData,
	rc::Rc,
};

// todo:
// * implement instance (Monoid b) => Monoid (a -> b)
// * https://github.com/purescript/purescript-prelude/blob/master/src/Data/Semigroup.purs#L48-L49
// * https://github.com/purescript/purescript-prelude/blob/master/src/Data/Monoid.purs#L55-L56

pub struct RcFn<'a, A, B>(pub Rc<dyn 'a + Fn(A) -> B>);

impl<'a, A, B> Clone for RcFn<'a, A, B> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<'a, A, B> Debug for RcFn<'a, A, B> {
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("RcFn").field(&"{closure}").finish()
	}
}

impl<'a, A, B> RcFn<'a, A, B> {
	fn new(f: impl 'a + Fn(A) -> B) -> Self {
		Self(Rc::new(f))
	}
}

/// A brand type for reference-counted closures (`Rc<dyn Fn(A) -> B>`).
///
/// This struct implements [`ClonableFn`] to provide a way to construct and
/// type-check [`Rc`]-wrapped closures in a generic context. The lifetime `'a`
/// ensures the closure doesn't outlive referenced data, while `A` and `B`
/// represent input and output types.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFnBrand;

impl Kind1L2T for RcFnBrand {
	type Output<'a, A, B> = RcFn<'a, A, B>;
}

impl Semigroupoid for RcFnBrand {
	fn compose<'a, ClonableFnBrand: 'a + ClonableFn, B, C, D>(
		f: Apply1L2T<'a, Self, C, D>
	) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Self, B, C>, Apply1L2T<'a, Self, B, D>> {
		ClonableFnBrand::new::<'a, _, _>(move |g: Apply1L2T<'a, Self, B, C>| {
			<Self as Kind1L2T>::Output::new({
				let f = f.clone();
				move |a| f.0(g.0(a)) // refactor using compose
			})
		})
	}
}

impl Category for RcFnBrand {
	fn identity<'a, T: 'a>() -> Apply1L2T<'a, Self, T, T> {
		<Self as Kind1L2T>::Output::new(identity)
	}
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFnWithOutputBrand<'a, Output: 'a>(PhantomData<&'a Output>);

impl<'a, Output> Kind0L1T for RcFnWithOutputBrand<'a, Output> {
	type Output<A> = RcFn<'a, A, Output>;
}

impl<'b, Output: Semigroup<'b>> Semigroup<'b> for RcFnWithOutputBrand<'b, Output> {
	fn append<'a, ClonableFnBrand: 'a + 'a + ClonableFn>(
		a: Self
	) -> ApplyFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'a: 'a,
	{
		ClonableFnBrand::new(move |b| ClonableFnBrand::new(move |c| append(a(c))(b(c))))
	}
}
