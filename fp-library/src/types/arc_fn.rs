use crate::{
	classes::{Category, ClonableFn, Semigroupoid, clonable_fn::ApplyFn},
	functions::identity,
	hkt::{Apply1L2T, Kind1L2T},
};
use std::sync::Arc;

/// A brand type for atomically reference-counted closures (`Arc<dyn Fn(A) -> B>`).
///
/// This struct implements [`ClonableFn`] to provide a way to construct and
/// type-check [`Arc`]-wrapped closures in a generic context. The lifetime `'a`
/// ensures the closure doesn't outlive referenced data, while `A` and `B`
/// represent input and output types.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArcFnBrand;

impl Kind1L2T for ArcFnBrand {
	type Output<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;
}

impl ClonableFn for ArcFnBrand {
	type Output<'a, A: 'a, B: 'a> = <Self as Kind1L2T>::Output<'a, A, B>;

	fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> <Self as ClonableFn>::Output<'a, A, B> {
		Arc::new(f)
	}
}

impl Semigroupoid for ArcFnBrand {
	fn compose<'a, ClonableFnBrand: 'a + ClonableFn, B, C, D>(
		f: Apply1L2T<'a, Self, C, D>
	) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Self, B, C>, Apply1L2T<'a, Self, B, D>> {
		ClonableFnBrand::new::<'a, _, _>(move |g: Apply1L2T<'a, Self, B, C>| {
			Self::new::<'a, _, _>({
				let f = f.clone();
				move |a| f(g(a))
			})
		})
	}
}

impl Category for ArcFnBrand {
	fn identity<'a, T: 'a>() -> Apply1L2T<'a, Self, T, T> {
		Self::new::<'a, _, _>(identity)
	}
}
