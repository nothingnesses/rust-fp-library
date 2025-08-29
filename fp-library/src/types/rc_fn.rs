use crate::{
	classes::{ClonableFn, Semigroupoid, clonable_fn::ApplyFn},
	hkt::{Apply1L2T, Kind1L2T},
};
use std::rc::Rc;

/// A brand type for reference-counted closures (`Rc<dyn Fn(A) -> B>`).
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

impl ClonableFn for RcFnBrand {
	type Output<'a, A: 'a, B: 'a> = <Self as Kind1L2T>::Output<'a, A, B>;

	fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> <Self as ClonableFn>::Output<'a, A, B> {
		Rc::new(f)
	}
}

impl Semigroupoid for RcFnBrand {
	fn compose<'a, ClonableFnBrand: 'a + ClonableFn, B, C, D>(
		f: Apply1L2T<'a, Self, C, D>
	) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Self, B, C>, Apply1L2T<'a, Self, B, D>> {
		ClonableFnBrand::new::<'a, _, _>(move |g: Apply1L2T<'a, Self, B, C>| {
			RcFnBrand::new::<'a, _, _>({
				let f = f.clone();
				move |a| f(g(a))
			})
		})
	}
}
