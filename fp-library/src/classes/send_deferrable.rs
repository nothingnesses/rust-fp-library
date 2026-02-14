//! Deferred lazy evaluation using thread-safe thunks.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*, types::*};
//!
//! let memo: ArcLazy<i32> = send_defer(|| ArcLazy::new(|| 42));
//! assert_eq!(*memo.evaluate(), 42);
//! ```

use fp_macros::document_parameters;
use fp_macros::document_signature;
use fp_macros::document_type_parameters;

/// A trait for deferred lazy evaluation with thread-safe thunks.
///
/// This is similar to [`Deferrable`](crate::classes::Deferrable), but the thunk must be `Send + Sync`.
pub trait SendDeferrable<'a> {
	/// Creates a deferred value from a thread-safe thunk.
	///
	#[document_signature]
	///
	#[document_type_parameters("The type of the thunk.")]
	///
	#[document_parameters("The function that produces the value.")]
	///
	/// ### Returns
	///
	/// A deferred value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let memo: ArcLazy<i32> = send_defer(|| ArcLazy::new(|| 42));
	/// assert_eq!(*memo.evaluate(), 42);
	/// ```
	fn send_defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Self + Send + Sync + 'a,
		Self: Sized;
}

/// Creates a deferred value from a thread-safe thunk.
///
/// Free function version that dispatches to [the type class' associated function][`SendDeferrable::send_defer`].
///
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the computation",
	"The type of the deferred value.",
	"The type of the thunk."
)]
///
#[document_parameters("The function that produces the value.")]
///
/// ### Returns
///
/// A deferred value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let memo: ArcLazy<i32> = send_defer(|| ArcLazy::new(|| 42));
/// assert_eq!(*memo.evaluate(), 42);
/// ```
pub fn send_defer<'a, D, F>(f: F) -> D
where
	D: SendDeferrable<'a>,
	F: FnOnce() -> D + Send + Sync + 'a,
{
	D::send_defer(f)
}
