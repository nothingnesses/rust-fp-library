use crate::make_type_apply;
use super::function::Function;
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
pub trait ClonableFn: Function {
	type Output<'a, A, B>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ApplyClonableFn<'a, Self, A, B>;
}

make_type_apply!(ApplyClonableFn, ClonableFn, ('a), (A, B), "' -> * -> *");
