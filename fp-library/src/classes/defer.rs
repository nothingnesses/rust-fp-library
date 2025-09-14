use crate::{
	classes::{ClonableFn, clonable_fn::ApplyClonableFn},
	hkt::Kind0L1T,
};

pub trait Defer<'a> {
	fn defer<ClonableFnBrand: 'a + ClonableFn>(
		f: ApplyClonableFn<'a, ClonableFnBrand, (), Self>
	) -> Self
	where
		Self: Sized;
}
