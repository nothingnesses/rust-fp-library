//! A type class for types that can be constructed lazily.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*, types::*};
//!
//! let eval: Thunk<i32> = defer::<Thunk<i32>, RcFnBrand>(
//!     cloneable_fn_new::<RcFnBrand, _, _>(|_| Thunk::new(|| 42))
//! );
//! assert_eq!(eval.evaluate(), 42);
//! ```

use super::CloneableFn;
use fp_macros::doc_params;
use fp_macros::doc_type_params;

/// A type class for types that can be constructed lazily.
pub trait Deferrable<'a> {
	/// Creates a value from a computation that produces the value.
	///
	/// This function takes a thunk (wrapped in a cloneable function) and creates a deferred value that will be computed using the thunk.
	///
	/// ### Type Signature
	///
	/// `forall. Deferrable d => (() -> d) -> d`
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The brand of the cloneable function wrapper.")]
	///
	/// ### Parameters
	///
	#[doc_params("A thunk (wrapped in a cloneable function) that produces the value.")]
	///
	/// ### Returns
	///
	/// The deferred value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let eval: Thunk<i32> = defer::<Thunk<i32>, RcFnBrand>(
	///     cloneable_fn_new::<RcFnBrand, _, _>(|_| Thunk::new(|| 42))
	/// );
	/// assert_eq!(eval.evaluate(), 42);
	/// ```
	fn defer<FnBrand: 'a + CloneableFn>(f: <FnBrand as CloneableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized;
}

/// Creates a value from a computation that produces the value.
///
/// Free function version that dispatches to [the type class' associated function][`Deferrable::defer`].
///
/// ### Type Signature
///
/// `forall. Deferrable d => (() -> d) -> d`
///
/// ### Type Parameters
//
/// * `FnBrand`: The brand of the cloneable function wrapper./
/// * `D`: The type of the deferred value.
///
/// ### Parameters
///
#[doc_params("A thunk (wrapped in a cloneable function) that produces the value.")]
///
/// ### Returns
///
/// The deferred value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let eval: Thunk<i32> = defer::<Thunk<i32>, RcFnBrand>(
///     cloneable_fn_new::<RcFnBrand, _, _>(|_| Thunk::new(|| 42))
/// );
/// assert_eq!(eval.evaluate(), 42);
/// ```
pub fn defer<'a, D, FnBrand>(f: <FnBrand as CloneableFn>::Of<'a, (), D>) -> D
where
	D: Deferrable<'a>,
	FnBrand: 'a + CloneableFn,
{
	D::defer::<FnBrand>(f)
}
