use crate::{hkt::Kind1L2T, make_type_apply};
use std::ops::Deref;

/// Abstracts over smart pointers for clonable closures.
///
/// This trait is implemented by zero-sized "Brand" types (like `ArcBrand` and `RcBrand`)
/// to provide a way to construct and type-check function wrappers (`Arc<dyn Fn...>`
/// or `Rc<dyn Fn...>`) in a generic context.
pub trait ClonableFn: Kind1L2T + Clone {
	type Output<'a, A: 'a, B: 'a>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

	fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> <Self as ClonableFn>::Output<'a, A, B>;
}

make_type_apply!(ApplyFn, ClonableFn, ('a), (A, B), "' -> * -> *");
