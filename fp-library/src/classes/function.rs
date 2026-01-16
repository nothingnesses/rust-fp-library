//! Function wrappers.
//!
//! This module defines the [`Function`] trait, which provides an abstraction for wrappers over closures.
//! This allows for generic handling of functions in higher-kinded contexts.

use super::category::Category;
use crate::Apply;
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
	type Of<'a, A, B>: Deref<Target = dyn 'a + Fn(A) -> B>;

	/// Creates a new function wrapper.
	///
	/// This function wraps the provided closure `f` into a function wrapper.
	///
	/// ### Type Signature
	///
	/// `forall a b. Function f => (a -> b) -> f a b`
	///
	/// ### Type Parameters
	///
	/// * `A`: The input type of the function.
	/// * `B`: The output type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to wrap.
	///
	/// ### Returns
	///
	/// The wrapped function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::function::Function;
	/// use fp_library::brands::RcFnBrand;
	///
	/// let f = <RcFnBrand as Function>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(
		f: impl 'a + Fn(A) -> B
	) -> Apply!(brand: Self, kind: Function, lifetimes: ('a), types: (A, B));
}

/// Creates a new function wrapper.
///
/// Free function version that dispatches to [the type class' associated function][`Function::new`].
///
/// ### Type Signature
///
/// `forall a b. Function f => (a -> b) -> f a b`
///
/// ### Type Parameters
///
/// * `F`: The brand of the function wrapper.
/// * `A`: The input type of the function.
/// * `B`: The output type of the function.
///
/// ### Parameters
///
/// * `f`: The closure to wrap.
///
/// ### Returns
///
/// The wrapped function.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::function::new;
/// use fp_library::brands::RcFnBrand;
///
/// let f = new::<RcFnBrand, _, _>(|x: i32| x * 2);
/// assert_eq!(f(5), 10);
/// ```
pub fn new<'a, F, A, B>(
	f: impl 'a + Fn(A) -> B
) -> Apply!(brand: F, kind: Function, lifetimes: ('a), types: (A, B))
where
	F: Function,
{
	F::new(f)
}
