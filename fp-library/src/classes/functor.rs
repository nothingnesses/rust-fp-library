//! Types that can be mapped over, allowing functions to be applied to values within a context.
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
//! let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```

use {
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for types that can be mapped over.
///
/// A `Functor` represents a context or container that allows functions to be applied
/// to values within that context without altering the structure of the context itself.
///
/// ### Laws
///
/// `Functor` instances must satisfy the following laws:
/// * Identity: `map(identity, fa) = fa`.
/// * Composition: `map(compose(f, g), fa) = map(f, map(g, fa))`.
pub trait Functor: Kind_ad6c20556a82a1f0 {
	/// Maps a function over the values in the functor context.
	///
	/// This method applies a function to the value(s) inside the functor context, producing a new functor context with the transformed value(s).
	#[document_signature]
	///
	#[document_type_parameters(
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The type of the function to apply."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s) inside the functor.",
		"The functor instance containing the value(s)."
	)]
	///
	/// ### Returns
	///
	/// A new functor instance containing the result(s) of applying the function.
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
	/// let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
	/// assert_eq!(y, Some(10));
	/// ```
	fn map<A, B, Func>(
		f: Func,
		fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
	) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
	where
		A: 'static,
		B: 'static,
		Func: Fn(A) -> B + 'static;
}

/// Maps a function over the values in the functor context.
///
/// Free function version that dispatches to [the type class' associated function][`Functor::map`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the functor.",
	"The type of the value(s) inside the functor.",
	"The type of the result(s) of applying the function.",
	"The type of the function to apply."
)]
///
#[document_parameters(
	"The function to apply to the value(s) inside the functor.",
	"The functor instance containing the value(s)."
)]
///
/// ### Returns
///
/// A new functor instance containing the result(s) of applying the function.
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
/// let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
/// assert_eq!(y, Some(10));
/// ```
pub fn map<Brand: Functor, A, B, Func>(
	f: Func,
	fa: Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>),
) -> Apply!(<Brand as Kind!( type Of<T>; )>::Of<B>)
where
	A: 'static,
	B: 'static,
	Func: Fn(A) -> B + 'static,
{
	Brand::map::<A, B, Func>(f, fa)
}
