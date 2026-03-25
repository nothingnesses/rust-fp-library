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
	use {
		crate::classes::Deferrable,
		fp_macros::*,
	};
	/// A trait for deferred lazy evaluation with thread-safe thunks.
	///
	/// This extends [`Deferrable`] with the additional requirement that the thunk
	/// must be `Send + Sync`, following the same supertrait pattern used by
	/// [`SendCloneableFn: CloneableFn`](crate::classes::SendCloneableFn).
	///
	/// Every `SendDeferrable` type is also `Deferrable`, so generic code written
	/// against `Deferrable` accepts both single-threaded and thread-safe types.
	///
	/// Unlike [`SendCloneableFn`](crate::classes::SendCloneableFn), which wraps multi-use
	/// `Fn` closures that are `Send + Sync`, this trait accepts a `FnOnce` closure that
	/// only needs to be `Send` (not `Sync`), since deferred computations are executed
	/// at most once.
	///
	/// ### Laws
	///
	/// `SendDeferrable` instances must satisfy the following law:
	/// * Transparency: `send_defer(|| x)` is observationally equivalent to `x` when evaluated.
	///
	/// ### Why there is no generic `fix`
	///
	/// As with [`Deferrable`](crate::classes::Deferrable), lazy self-reference requires
	/// shared ownership and interior mutability, which are properties specific to
	/// [`Lazy`](crate::types::Lazy). The concrete function
	/// [`arc_lazy_fix`](crate::types::lazy::arc_lazy_fix) provides this capability for
	/// `ArcLazy` specifically.
	#[document_type_parameters("The lifetime of the computation.")]
	#[document_examples]
	///
	/// Transparency law for [`ArcLazy`](crate::types::ArcLazy):
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // Transparency: send_defer(|| x) is equivalent to x when evaluated.
	/// let x = ArcLazy::pure(42);
	/// let deferred: ArcLazy<i32> = send_defer(|| ArcLazy::pure(42));
	/// assert_eq!(*deferred.evaluate(), *x.evaluate());
	/// ```
	pub trait SendDeferrable<'a>: Deferrable<'a> {
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
		"The lifetime of the computation.",
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
