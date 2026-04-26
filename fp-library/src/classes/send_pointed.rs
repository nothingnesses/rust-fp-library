//! Thread-safe by-value injection with [`send_pure`].
//!
//! Like [`Pointed::pure`](crate::classes::Pointed::pure), but requires
//! `A: Send + Sync` so the result can cross thread boundaries. By-value
//! parallel of [`SendRefPointed`](crate::classes::SendRefPointed), which
//! takes a reference and additionally requires `A: Clone`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = send_pure::<OptionBrand, _>(42);
//! assert_eq!(x, Some(42));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for injecting a value by-value into a thread-safe context.
	///
	/// This is the thread-safe by-value counterpart of
	/// [`Pointed`](crate::classes::Pointed). The `A: Send + Sync` bound on
	/// the consumed value ensures the resulting context can cross thread
	/// boundaries. Unlike
	/// [`SendRefPointed`](crate::classes::SendRefPointed), which lifts from
	/// a reference and clones, `SendPointed` consumes `A` directly and so
	/// does not require `Clone`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendPointed {
		/// Wraps a value by-value in a new thread-safe context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value. Must be `Send + Sync`."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new thread-safe context containing the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let x = OptionBrand::send_pure(42);
		/// assert_eq!(x, Some(42));
		/// ```
		fn send_pure<'a, A: Send + Sync + 'a>(
			a: A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	/// Wraps a value by-value in a new thread-safe context.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendPointed::send_pure`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the value.",
		"The brand of the context.",
		"The type of the value. Must be `Send + Sync`."
	)]
	///
	#[document_parameters("The value to wrap.")]
	///
	#[document_returns("A new thread-safe context containing the value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = send_pure::<OptionBrand, _>(42);
	/// assert_eq!(x, Some(42));
	/// ```
	pub fn send_pure<'a, Brand: SendPointed, A: Send + Sync + 'a>(
		a: A
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::send_pure(a)
	}
}

pub use inner::*;
