//! A trait for wrappers over closures, allowing for generic handling of functions in higher-kinded contexts.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let f = fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
//! assert_eq!(f(5), 10);
//! ```

use fp_macros::doc_type_params;
use super::category::Category;
use fp_macros::hm_signature;
use std::ops::Deref;

/// A trait for wrappers over closures, allowing for generic handling of functions in higher-kinded contexts.
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
	#[doc_type_params(
		"Undocumented",
		"The input type of the function.",
		"The output type of the function."
	)]	///
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
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> <Self as Function>::Of<'a, A, B>;
}

/// Creates a new function wrapper.
///
/// Free function version that dispatches to [the type class' associated function][`Function::new`].
///
/// ### Type Signature
///
#[hm_signature(Function)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"Undocumented",
	"The brand of the function wrapper.",
	"The input type of the function.",
	"The output type of the function."
)]///
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
/// use fp_library::{brands::*, functions::*};
///
/// let f = fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
/// assert_eq!(f(5), 10);
/// ```
pub fn new<'a, Brand, A, B>(f: impl 'a + Fn(A) -> B) -> <Brand as Function>::Of<'a, A, B>
where
	Brand: Function,
{
	Brand::new(f)
}
