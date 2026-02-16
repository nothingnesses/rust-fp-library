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

use {
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for contexts that can be initialized with a value.
pub trait Pointed: Kind_ad6c20556a82a1f0 {
	/// The value wrapped in the context.
	///
	/// This method wraps a value in a context.
	#[document_signature]
	///
	#[document_type_parameters("The type of the value to wrap.")]
	///
	#[document_parameters("The value to wrap.")]
	///
	/// ### Returns
	///
	/// A new context containing the value.
	///
	/// ### Examples
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
	fn pure<A>(a: A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<A>);
}

/// The value wrapped in the context.
///
/// Free function version that dispatches to [the type class' associated function][`Pointed::pure`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the context.",
	"The type of the value to wrap."
)]
///
#[document_parameters("The value to wrap.")]
///
/// ### Returns
///
/// A new context containing the value.
///
/// ### Examples
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
pub fn pure<Brand: Pointed, A>(
	a: A
) -> Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>) {
	Brand::pure(a)
}
