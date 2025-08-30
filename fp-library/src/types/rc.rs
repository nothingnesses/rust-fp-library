use crate::{classes::SmartPointer, hkt::Kind0L1T};
use std::rc::Rc;

/// A brand type for reference-counted closures (`Rc<dyn Fn(A) -> B>`).
///
/// This struct implements [`ClonableFn`] to provide a way to construct and
/// type-check [`Rc`]-wrapped closures in a generic context. The lifetime `'a`
/// ensures the closure doesn't outlive referenced data, while `A` and `B`
/// represent input and output types.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcBrand;

impl Kind0L1T for RcBrand {
	type Output<A> = Rc<A>;
}

impl SmartPointer for RcBrand {
	type Output<A> = <Self as Kind0L1T>::Output<A>;

	fn new<A>(a: A) -> <Self as SmartPointer>::Output<A> {
		Rc::new(a)
	}

	fn inner<A: Clone>(a: <Self as SmartPointer>::Output<A>) -> A {
		(*a).clone()
	}
}
