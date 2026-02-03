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

use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A trait for deferred lazy evaluation with thread-safe thunks.
///
/// This is similar to [`Deferrable`](crate::classes::Deferrable), but the thunk must be `Send + Sync`.
pub trait SendDeferrable<'a> {
	/// Creates a deferred value from a thread-safe thunk.
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
	#[doc_params("The function that produces the value.")]
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
#[doc_params("The function that produces the value.")]
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
