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
//! let eval: Thunk<i32> = defer(|| Thunk::pure(42));
//! assert_eq!(eval.evaluate(), 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use fp_macros::*;
	/// A type class for types that can be constructed lazily.
	///
	/// # Warning
	///
	/// Some implementations may evaluate the thunk eagerly when the produced type requires
	/// `Send`. For example, `ArcLazy`'s `Deferrable` implementation evaluates the outer thunk
	/// immediately because `ArcLazy::new` requires a `Send` closure, but the `Deferrable`
	/// trait does not impose that bound. If you need guaranteed deferred evaluation with
	/// thread-safe types, prefer [`SendDeferrable`](crate::classes::SendDeferrable) instead.
	#[document_type_parameters("The lifetime of the computation.")]
	pub trait Deferrable<'a> {
		/// Creates a value from a computation that produces the value.
		///
		/// This function takes a thunk and creates a deferred value that will be computed using the thunk.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the value.")]
		///
		#[document_returns("The deferred value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let eval: Thunk<i32> = defer(|| Thunk::pure(42));
		/// assert_eq!(eval.evaluate(), 42);
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized;
	}

	/// Creates a value from a computation that produces the value.
	///
	/// Free function version that dispatches to [the type class' associated function][`Deferrable::defer`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation",
		"The type of the deferred value."
	)]
	///
	#[document_parameters("A thunk that produces the value.")]
	///
	#[document_returns("The deferred value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let eval: Thunk<i32> = defer(|| Thunk::pure(42));
	/// assert_eq!(eval.evaluate(), 42);
	/// ```
	pub fn defer<'a, D: Deferrable<'a>>(f: impl FnOnce() -> D + 'a) -> D {
		D::defer(f)
	}
}

pub use inner::*;
