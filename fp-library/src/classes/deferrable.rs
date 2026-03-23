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

#[fp_macros::document_module]
mod inner {
	use fp_macros::*;
	/// A type class for types that can be constructed lazily.
	///
	/// ### Laws
	///
	/// `Deferrable` instances must satisfy the following law:
	/// * Transparency: `defer(|| x)` is observationally equivalent to `x` when evaluated.
	///
	/// ### Why there is no generic `fix`
	///
	/// In PureScript, `fix :: Lazy l => (l -> l) -> l` enables lazy self-reference,
	/// which is essential for tying the knot in recursive values. In Rust, lazy
	/// self-reference requires shared ownership (`Rc`/`Arc`) and interior mutability,
	/// which are properties specific to [`Lazy`](crate::types::Lazy) rather than
	/// all `Deferrable` types. For example, [`Thunk`](crate::types::Thunk) is consumed
	/// on evaluation, so self-referential construction is not possible.
	///
	/// The concrete functions [`rc_lazy_fix`](crate::types::lazy::rc_lazy_fix) and
	/// [`arc_lazy_fix`](crate::types::lazy::arc_lazy_fix) provide this capability for
	/// `Lazy` specifically.
	#[document_type_parameters("The lifetime of the computation.")]
	#[document_examples]
	///
	/// Transparency law for [`Thunk`](crate::types::Thunk):
	///
	/// ```
	/// use fp_library::{
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // Transparency: defer(|| x) is equivalent to x when evaluated.
	/// let x = Thunk::pure(42);
	/// let deferred: Thunk<i32> = defer(|| Thunk::pure(42));
	/// assert_eq!(deferred.evaluate(), x.evaluate());
	/// ```
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
		/// let eval: Thunk<i32> = defer(|| Thunk::new(|| 42));
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
	/// let eval: Thunk<i32> = defer(|| Thunk::new(|| 42));
	/// assert_eq!(eval.evaluate(), 42);
	/// ```
	pub fn defer<'a, D: Deferrable<'a>>(f: impl FnOnce() -> D + 'a) -> D {
		D::defer(f)
	}
}

pub use inner::*;
