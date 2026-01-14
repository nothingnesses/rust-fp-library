use super::function::Function;
use crate::Apply;
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
	type Of<'a, A, B>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

	/// Creates a new clonable function wrapper.
	///
	/// # Type Signature
	///
	/// `forall a b. ClonableFn f => (a -> b) -> f a b`
	///
	/// # Parameters
	///
	/// * `f`: The closure to wrap.
	///
	/// # Returns
	///
	/// The wrapped clonable function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::RcFnBrand;
	///
	/// let f = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Apply!(Self, ClonableFn, ('a), (A, B));
}

/// Creates a new clonable function wrapper.
///
/// Free function version that dispatches to [the type class' associated function][`ClonableFn::new`].
///
/// # Type Signature
///
/// `forall a b. ClonableFn f => (a -> b) -> f a b`
///
/// # Parameters
///
/// * `f`: The closure to wrap.
///
/// # Returns
///
/// The wrapped clonable function.
///
/// # Examples
///
/// ```
/// use fp_library::classes::clonable_fn::new;
/// use fp_library::brands::RcFnBrand;
///
/// let f = new::<RcFnBrand, _, _>(|x: i32| x * 2);
/// assert_eq!(f(5), 10);
/// ```
pub fn new<'a, F, A, B>(f: impl 'a + Fn(A) -> B) -> Apply!(F, ClonableFn, ('a), (A, B))
where
	F: ClonableFn,
{
	<F as ClonableFn>::new(f)
}
