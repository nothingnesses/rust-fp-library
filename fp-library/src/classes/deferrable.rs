//! Types that can be constructed lazily from a computation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let eval: Thunk<i32> = defer(|| Thunk::new(|| 42));
//! assert_eq!(eval.evaluate(), 42);
//! ```

use fp_macros::{document_parameters, document_signature, document_type_parameters};

/// A type class for types that can be constructed lazily.
pub trait Deferrable {
	/// Creates a value from a computation that produces the value.
	///
	/// This function takes a thunk and creates a deferred value that will be computed using the thunk.
	#[document_signature]
	///
	#[document_type_parameters("The type of the thunk.")]
	///
	#[document_parameters("A thunk that produces the value.")]
	///
	/// ### Returns
	///
	/// The deferred value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let eval: Thunk<i32> = defer(|| Thunk::new(|| 42));
	/// assert_eq!(eval.evaluate(), 42);
	/// ```
	fn defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Self,
		Self: Sized;
}

/// Creates a value from a computation that produces the value.
///
/// Free function version that dispatches to [the type class' associated function][`Deferrable::defer`].
#[document_signature]
///
#[document_type_parameters(
	"The type of the deferred value.",
	"The type of the thunk."
)]
///
#[document_parameters("A thunk that produces the value.")]
///
/// ### Returns
///
/// The deferred value.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// 	types::*,
/// };
///
/// let eval: Thunk<i32> = defer(|| Thunk::new(|| 42));
/// assert_eq!(eval.evaluate(), 42);
/// ```
pub fn defer<D, F>(f: F) -> D
where
	D: Deferrable,
	F: FnOnce() -> D,
{
	D::defer(f)
}
