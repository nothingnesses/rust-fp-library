//! A type class for types that can be constructed lazily.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::*, functions::*, types::*};
//!
//! let eval: Thunk<i32> = defer::<Thunk<i32>, RcFnBrand>(
//!     cloneable_fn_new::<RcFnBrand, _, _>(|_| Thunk::new(|| 42))
//! );
//! assert_eq!(eval.run(), 42);
//! ```

use super::CloneableFn;

/// A type class for types that can be constructed lazily.
pub trait Defer<'a> {
	/// Creates a value from a computation that produces the value.
	///
	/// This function takes a thunk (wrapped in a cloneable function) and creates a deferred value that will be computed using the thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. Defer d => (() -> d a) -> d a`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the cloneable function wrapper.
	///
	/// ### Parameters
	///
	/// * `f`: A thunk (wrapped in a cloneable function) that produces the value.
	///
	/// ### Returns
	///
	/// The deferred value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let eval: Thunk<i32> = defer::<Thunk<i32>, RcFnBrand>(
	///     cloneable_fn_new::<RcFnBrand, _, _>(|_| Thunk::new(|| 42))
	/// );
	/// assert_eq!(eval.run(), 42);
	/// ```
	fn defer<FnBrand: 'a + CloneableFn>(f: <FnBrand as CloneableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized;
}

/// Creates a value from a computation that produces the value.
///
/// Free function version that dispatches to [the type class' associated function][`Defer::defer`].
///
/// ### Type Signature
///
/// `forall a. Defer d => (() -> d a) -> d a`
///
/// ### Type Parameters
///
/// * `D`: The type of the deferred value.
/// * `FnBrand`: The brand of the cloneable function wrapper.
///
/// ### Parameters
///
/// * `f`: A thunk (wrapped in a cloneable function) that produces the value.
///
/// ### Returns
///
/// The deferred value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::*, functions::*, types::*};
///
/// let eval: Thunk<i32> = defer::<Thunk<i32>, RcFnBrand>(
///     cloneable_fn_new::<RcFnBrand, _, _>(|_| Thunk::new(|| 42))
/// );
/// assert_eq!(eval.run(), 42);
/// ```
pub fn defer<'a, D, FnBrand>(f: <FnBrand as CloneableFn>::Of<'a, (), D>) -> D
where
	D: Defer<'a>,
	FnBrand: 'a + CloneableFn,
{
	D::defer::<FnBrand>(f)
}
