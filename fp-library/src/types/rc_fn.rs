use crate::{hkt::Kind1L2T, typeclasses::ClonableFn};
use std::rc::Rc;

pub struct RcFn;

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
