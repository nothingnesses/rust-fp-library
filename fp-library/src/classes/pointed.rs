//! Contexts that can be initialized with a value via the [`pure`] operation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{functions::*, brands::*};
//!
//! let x = pure::<OptionBrand, _>(5);
//! assert_eq!(x, Some(5));
//! ```

use crate::{Apply, kinds::*};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A type class for contexts that can be initialized with a value.
pub trait Pointed: Kind_cdc7cd43dac7585f {
	/// The value wrapped in the context.
	///
	/// This method wraps a value in a context.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the value.", "The type of the value to wrap.")]
	///
	/// ### Parameters
	///
	#[doc_params("The value to wrap.")]
	///
	/// ### Returns
	///
	/// A new context containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{functions::*, brands::*};
	///
	/// let x = pure::<OptionBrand, _>(5);
	/// assert_eq!(x, Some(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
}

/// The value wrapped in the context.
///
/// Free function version that dispatches to [the type class' associated function][`Pointed::pure`].
///
/// ### Type Signature
///
#[hm_signature]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the value.",
	"The brand of the context.",
	"The type of the value to wrap."
)]
///
/// ### Parameters
///
#[doc_params("The value to wrap.")]
///
/// ### Returns
///
/// A new context containing the value.
///
/// ### Examples
///
/// ```
/// use fp_library::{functions::*, brands::*};
///
/// let x = pure::<OptionBrand, _>(5);
/// assert_eq!(x, Some(5));
/// ```
pub fn pure<'a, Brand: Pointed, A: 'a>(
	a: A
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
	Brand::pure(a)
}
