//! A trait for cloneable wrappers over closures, allowing for generic handling of cloneable functions in higher-kinded contexts.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
//! assert_eq!(f(5), 10);
//! ```

use super::function::Function;
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;
use std::ops::Deref;

/// A trait for cloneable wrappers over closures, allowing for generic handling of cloneable functions in higher-kinded contexts.
///
/// This trait is implemented by "Brand" types (like [`ArcFnBrand`][crate::brands::ArcFnBrand]
/// and [`RcFnBrand`][crate::brands::RcFnBrand]) to provide a way to construct
/// and type-check cloneable wrappers over closures (`Arc<dyn Fn...>` or
/// `Rc<dyn Fn...>`) in a generic context, allowing library users to choose
/// between implementations at function call sites.
///
/// The lifetime `'a` ensures the function doesn't outlive referenced data,
/// while generic types `A` and `B` represent the input and output types, respectively.
pub trait CloneableFn: Function {
	type Of<'a, A, B>: Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

	/// Creates a new cloneable function wrapper.
	///
	/// This function wraps the provided closure `f` into a cloneable function.
	///
	/// ### Type Signature
	///
	/// `forall a b. CloneableFn f => (a -> b) -> f a b`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the function and its captured data.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The closure to wrap.", "The input value to the function.")]
	/// ### Returns
	///
	/// The wrapped cloneable function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> <Self as CloneableFn>::Of<'a, A, B>;
}

/// Creates a new cloneable function wrapper.
///
/// Free function version that dispatches to [the type class' associated function][`CloneableFn::new`].
///
/// ### Type Signature
///
#[hm_signature(CloneableFn)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the function and its captured data.",
	"The brand of the cloneable function wrapper.",
	"The input type of the function.",
	"The output type of the function."
)]
///
/// ### Parameters
///
#[doc_params("The closure to wrap.", "The input value to the function.")]
/// ### Returns
///
/// The wrapped cloneable function.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
/// assert_eq!(f(5), 10);
/// ```
pub fn new<'a, Brand, A, B>(f: impl 'a + Fn(A) -> B) -> <Brand as CloneableFn>::Of<'a, A, B>
where
	Brand: CloneableFn,
{
	<Brand as CloneableFn>::new(f)
}
