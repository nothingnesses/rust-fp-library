//! Sequencing of two computations while keeping the result of the first.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(5);
//! let y = Some(10);
//! let z = apply_first::<OptionBrand, _, _>(x, y);
//! assert_eq!(z, Some(5));
//! ```

use {
	super::lift::Lift,
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for types that support combining two contexts, keeping the first value.
///
/// `ApplyFirst` provides the ability to sequence two computations but discard
/// the result of the second computation, keeping only the result of the first.
pub trait ApplyFirst: Lift {
	/// Combines two contexts, keeping the value from the first context.
	///
	/// This function sequences two computations and discards the result of the second computation, keeping only the result of the first.
	#[document_signature]
	///
	#[document_type_parameters(
		"The type of the value in the first context.",
		"The type of the value in the second context."
	)]
	///
	#[document_parameters("The first context.", "The second context.")]
	///
	/// ### Returns
	///
	/// The first context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = Some(10);
	/// let z = apply_first::<OptionBrand, _, _>(x, y);
	/// assert_eq!(z, Some(5));
	/// ```
	fn apply_first<A: Clone, B: Clone>(
		fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		fb: Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
	) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<A>) {
		Self::lift2(|a, _| a, fa, fb)
	}
}

/// Combines two contexts, keeping the value from the first context.
///
/// Free function version that dispatches to [the type class' associated function][`ApplyFirst::apply_first`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the context.",
	"The type of the value in the first context.",
	"The type of the value in the second context."
)]
///
#[document_parameters("The first context.", "The second context.")]
///
/// ### Returns
///
/// The first context.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Some(5);
/// let y = Some(10);
/// let z = apply_first::<OptionBrand, _, _>(x, y);
/// assert_eq!(z, Some(5));
/// ```
pub fn apply_first<Brand: ApplyFirst, A: Clone, B: Clone>(
	fa: Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>),
	fb: Apply!(<Brand as Kind!( type Of<T>; )>::Of<B>),
) -> Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>) {
	Brand::apply_first(fa, fb)
}
