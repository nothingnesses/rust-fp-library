//! Clonable function wrappers.
//!
//! This module defines the [`ClonableFn`] trait, which provides an abstraction for clonable wrappers over closures.
//! This allows for generic handling of clonable functions (like `Rc<dyn Fn>`) in higher-kinded contexts.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let f = clonable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
//! assert_eq!(f(5), 10);
//! ```

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
	type Of<'a, A, B>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

	/// Creates a new clonable function wrapper.
	///
	/// This function wraps the provided closure `f` into a clonable function.
	///
	/// ### Type Signature
	///
	/// `forall a b. ClonableFn f => (a -> b) -> f a b`
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
	/// The wrapped clonable function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = clonable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> <Self as ClonableFn>::Of<'a, A, B>;
}

/// Creates a new clonable function wrapper.
///
/// Free function version that dispatches to [the type class' associated function][`ClonableFn::new`].
///
/// ### Type Signature
///
/// `forall a b. ClonableFn f => (a -> b) -> f a b`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the clonable function wrapper.
/// * `A`: The input type of the function.
/// * `B`: The output type of the function.
///
/// ### Parameters
///
/// * `f`: The closure to wrap.
///
/// ### Returns
///
/// The wrapped clonable function.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let f = clonable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
/// assert_eq!(f(5), 10);
/// ```
pub fn new<'a, Brand, A, B>(f: impl 'a + Fn(A) -> B) -> <Brand as ClonableFn>::Of<'a, A, B>
where
	Brand: ClonableFn,
{
	<Brand as ClonableFn>::new(f)
}
