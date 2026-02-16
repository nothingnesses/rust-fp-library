//! Lifting of binary functions to operate on values within a context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(1);
//! let y = Some(2);
//! let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, x, y);
//! assert_eq!(z, Some(3));
//! ```

use {
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for lifting binary functions into a context.
pub trait Lift: Kind_ad6c20556a82a1f0 {
	/// Lifts a binary function into the context.
	///
	/// This method lifts a binary function to operate on values within the context.
	#[document_signature]
	///
	#[document_type_parameters(
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result.",
		"The type of the binary function."
	)]
	///
	#[document_parameters(
		"The binary function to apply.",
		"The first context.",
		"The second context."
	)]
	///
	/// ### Returns
	///
	/// A new context containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(1);
	/// let y = Some(2);
	/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, x, y);
	/// assert_eq!(z, Some(3));
	/// ```
	fn lift2<A, B, C, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		fb: Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
	) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<C>)
	where
		Func: Fn(A, B) -> C,
		A: Clone,
		B: Clone;
}

/// Lifts a binary function into the context.
///
/// Free function version that dispatches to [the type class' associated function][`Lift::lift2`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the context.",
	"The type of the first value.",
	"The type of the second value.",
	"The type of the result.",
	"The type of the binary function."
)]
///
#[document_parameters("The binary function to apply.", "The first context.", "The second context.")]
///
/// ### Returns
///
/// A new context containing the result of applying the function.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Some(1);
/// let y = Some(2);
/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, x, y);
/// assert_eq!(z, Some(3));
/// ```
pub fn lift2<Brand: Lift, A, B, C, Func>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>),
	fb: Apply!(<Brand as Kind!( type Of<T>; )>::Of<B>),
) -> Apply!(<Brand as Kind!( type Of<T>; )>::Of<C>)
where
	Func: Fn(A, B) -> C,
	A: Clone,
	B: Clone,
{
	Brand::lift2::<A, B, C, Func>(func, fa, fb)
}
