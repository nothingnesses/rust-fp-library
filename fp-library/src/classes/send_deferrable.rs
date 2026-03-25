//! Deferred lazy evaluation using thread-safe thunks.
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
//! let memo: ArcLazy<i32> = send_defer(|| ArcLazy::new(|| 42));
//! assert_eq!(*memo.evaluate(), 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use fp_macros::*;
	/// A trait for deferred lazy evaluation with thread-safe thunks.
	///
	/// This is similar to [`Deferrable`](crate::classes::Deferrable), but the thunk must be
	/// `Send`. Unlike [`SendCloneableFn`](crate::classes::SendCloneableFn), which wraps
	/// multi-use `Fn` closures that are `Send + Sync`, this trait accepts a `FnOnce` closure
	/// that only needs to be `Send` (not `Sync`), since deferred computations are executed
	/// at most once.
	#[document_type_parameters("The lifetime of the computation.")]
	pub trait SendDeferrable<'a> {
		/// Creates a deferred value from a thread-safe thunk.
		#[document_signature]
		///
		#[document_parameters("The function that produces the value.")]
		///
		#[document_returns("A deferred value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let memo: ArcLazy<i32> = send_defer(|| ArcLazy::new(|| 42));
		/// assert_eq!(*memo.evaluate(), 42);
		/// ```
		fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self
		where
			Self: Sized;
	}

	/// Creates a deferred value from a thread-safe thunk.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendDeferrable::send_defer`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation",
		"The type of the deferred value."
	)]
	///
	#[document_parameters("The function that produces the value.")]
	///
	#[document_returns("A deferred value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let memo: ArcLazy<i32> = send_defer(|| ArcLazy::new(|| 42));
	/// assert_eq!(*memo.evaluate(), 42);
	/// ```
	pub fn send_defer<'a, D: SendDeferrable<'a>>(f: impl FnOnce() -> D + Send + 'a) -> D {
		D::send_defer(f)
	}
}

pub use inner::*;
