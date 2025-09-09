use crate::classes::{ClonableFn, clonable_fn::ApplyClonableFn};

pub trait Defer<'a> {
	fn defer<ClonableFnBrand: 'a + ClonableFn>(
		f: ApplyClonableFn<'a, ClonableFnBrand, (), Self>
	) -> Self
	where
		Self: Sized;
}
