//! Types that can be constructed lazily from a computation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*, types::*};
//!
//! let eval: Thunk<i32> = defer(|| Thunk::new(|| 42));
//! assert_eq!(eval.evaluate(), 42);
//! ```

use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A type class for types that can be constructed lazily.
pub trait Deferrable<'a> {
	/// Creates a value from a computation that produces the value.
	///
	/// This function takes a thunk and creates a deferred value that will be computed using the thunk.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the thunk.")]
	///
	/// ### Parameters
	///
	#[doc_params("A thunk that produces the value.")]
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
	/// let eval: Thunk<i32> = defer(|| Thunk::new(|| 42));
	/// assert_eq!(eval.evaluate(), 42);
	/// ```
	fn defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Self + 'a,
		Self: Sized;
}

/// Creates a value from a computation that produces the value.
///
/// Free function version that dispatches to [the type class' associated function][`Deferrable::defer`].
///
/// ### Type Signature
///
#[hm_signature]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the computation",
	"The type of the deferred value.",
	"The type of the thunk."
)]
///
/// ### Parameters
///
#[doc_params("A thunk that produces the value.")]
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
/// let eval: Thunk<i32> = defer(|| Thunk::new(|| 42));
/// assert_eq!(eval.evaluate(), 42);
/// ```
pub fn defer<'a, D, F>(f: F) -> D
where
	D: Deferrable<'a>,
	F: FnOnce() -> D + 'a,
{
	D::defer(f)
}
