//! A type class for sequencing two computations and keeping the result of the first.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(5);
//! let y = Some(10);
//! let z = apply_first::<OptionBrand, _, _>(x, y);
//! assert_eq!(z, Some(5));
//! ```

use super::lift::Lift;
use crate::{Apply, kinds::*};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A type class for types that support combining two contexts, keeping the first value.
///
/// `ApplyFirst` provides the ability to sequence two computations but discard
/// the result of the second computation, keeping only the result of the first.
pub trait ApplyFirst: Lift {
	/// Combines two contexts, keeping the value from the first context.
	///
	/// This function sequences two computations and discards the result of the second computation, keeping only the result of the first.
	///
	/// ### Type Signature
	///
	#[hm_signature(ApplyFirst)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the value in the first context.",
		"The type of the value in the second context."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The first context.", "The second context.")]
	///
	/// ### Returns
	///
	/// The first context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(5);
	/// let y = Some(10);
	/// let z = apply_first::<OptionBrand, _, _>(x, y);
	/// assert_eq!(z, Some(5));
	/// ```
	fn apply_first<'a, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Self::lift2(|a, _| a, fa, fb)
	}
}

/// Combines two contexts, keeping the value from the first context.
///
/// Free function version that dispatches to [the type class' associated function][`ApplyFirst::apply_first`].
///
/// ### Type Signature
///
#[hm_signature(ApplyFirst)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"Undocumented",
	"The brand of the context.",
	"The type of the value in the first context.",
	"The type of the value in the second context."
)]
///
/// ### Parameters
///
#[doc_params("The first context.", "The second context.")]
///
/// ### Returns
///
/// The first context.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(5);
/// let y = Some(10);
/// let z = apply_first::<OptionBrand, _, _>(x, y);
/// assert_eq!(z, Some(5));
/// ```
pub fn apply_first<'a, Brand: ApplyFirst, A: 'a + Clone, B: 'a + Clone>(
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
	Brand::apply_first(fa, fb)
}
