//! Thread-safe by-ref value injection with [`send_ref_pure`].
//!
//! Like [`RefPointed::ref_pure`](crate::classes::RefPointed::ref_pure), but
//! requires `A: Send + Sync + Clone` so the result can cross thread boundaries.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let value = 42;
//! let lazy = send_ref_pure::<LazyBrand<ArcLazyConfig>, _>(&value);
//! assert_eq!(*lazy.evaluate(), 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for injecting a value into a context from a reference,
	/// with `Send + Sync` bounds.
	///
	/// This is the thread-safe counterpart of [`RefPointed`](crate::classes::RefPointed).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefPointed {
		/// Wraps a cloned value in a new thread-safe memoized context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value. Must be `Clone + Send + Sync`."
		)]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("A new thread-safe memoized value containing a clone of the input.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let value = 42;
		/// let lazy = LazyBrand::<ArcLazyConfig>::send_ref_pure(&value);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn send_ref_pure<'a, A: Clone + Send + Sync + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	/// Wraps a cloned value in a new thread-safe memoized context.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefPointed::send_ref_pure`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the value.",
		"The brand of the context.",
		"The type of the value. Must be `Clone + Send + Sync`."
	)]
	///
	#[document_parameters("A reference to the value to wrap.")]
	///
	#[document_returns("A new thread-safe memoized value containing a clone of the input.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let value = 42;
	/// let lazy = send_ref_pure::<LazyBrand<ArcLazyConfig>, _>(&value);
	/// assert_eq!(*lazy.evaluate(), 42);
	/// ```
	pub fn send_ref_pure<'a, Brand: SendRefPointed, A: Clone + Send + Sync + 'a>(
		a: &A
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::send_ref_pure(a)
	}
}

pub use inner::*;
