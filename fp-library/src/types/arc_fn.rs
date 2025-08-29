use crate::{classes::ClonableFn, hkt::Kind1L2T};
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
