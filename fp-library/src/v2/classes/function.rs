use crate::make_type_apply;
use super::category::Category;
use std::ops::Deref;

/// Abstraction for wrappers over closures.
///
/// This trait is implemented by "Brand" types (like [`ArcFnBrand`][crate::brands::ArcFnBrand]
/// and [`RcFnBrand`][crate::brands::RcFnBrand]) to provide a way to construct
/// and type-check wrappers over closures (`Arc<dyn Fn...>`, `Rc<dyn Fn...>`,
/// etc.) in a generic context, allowing library users to choose between
/// implementations at function call sites.
///
/// The lifetime `'a` ensures the function doesn't outlive referenced data,
/// while generic types `A` and `B` represent the input and output types, respectively.
pub trait Function: Category {
	type Output<'a, A, B>: Deref<Target = dyn 'a + Fn(A) -> B>;

	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ApplyFunction<'a, Self, A, B>;
}

make_type_apply!(ApplyFunction, Function, ('a), (A, B), "' -> * -> *");
