//! Contexts that can be initialized with a value via the [`pure`] operation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = pure::<OptionBrand, _>(5);
//! assert_eq!(x, Some(5));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for contexts that can be initialized with a value.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait Pointed {
		/// The value wrapped in the context.
		///
		/// This method wraps a value in a context.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new context containing the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = pure::<OptionBrand, _>(5);
		/// assert_eq!(x, Some(5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	/// The value wrapped in the context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Pointed::pure`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the value.",
		"The brand of the context.",
		"The type of the value to wrap."
	)]
	///
	#[document_parameters("The value to wrap.")]
	///
	#[document_returns("A new context containing the value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = pure::<OptionBrand, _>(5);
	/// assert_eq!(x, Some(5));
	/// ```
	pub fn pure<'a, Brand: Pointed, A: 'a>(
		a: A
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::pure(a)
	}
}

pub use inner::*;
