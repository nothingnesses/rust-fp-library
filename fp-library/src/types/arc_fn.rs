use crate::{hkt::Kind1L2T, typeclasses::ClonableFn};
use std::sync::Arc;

pub struct ArcFn;

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
