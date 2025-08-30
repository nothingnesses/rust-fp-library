use crate::{hkt::Kind0L1T, make_type_apply};
use std::ops::Deref;

/// Abstraction for clonable wrappers over closures.
///
/// This trait is implemented by "Brand" types (like [`ArcFnBrand`][crate::brands::ArcFnBrand]
/// and [`RcFnBrand`][crate::brands::RcFnBrand]) to provide a way to construct
/// and type-check clonable wrappers over closures (`Arc<dyn Fn...>` or
/// `Rc<dyn Fn...>`) in a generic context, allowing library users to choose
/// between implementations at function call sites.
///
/// The lifetime `'a` ensures the function doesn't outlive referenced data,
/// while generic types `A` and `B` represent the input and output types, respectively.
pub trait SmartPointer: Kind0L1T {
	type Output<A>: Deref<Target = A> + Clone;

	fn new<A>(a: A) -> <Self as SmartPointer>::Output<A>;

	fn inner<A: Clone>(a: <Self as SmartPointer>::Output<A>) -> A;
}

make_type_apply!(SmartPointerInner, SmartPointer, (), (A), "' -> * -> *");