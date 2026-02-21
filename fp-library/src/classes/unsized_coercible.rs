//! Pointer brands that can perform unsized coercion to `dyn Fn` trait objects.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = coerce_fn::<RcBrand, _, _, _>(|x: i32| x + 1);
//! assert_eq!(f(1), 2);
//! ```

use {
	super::RefCountedPointer,
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// Trait for pointer brands that can perform unsized coercion to `dyn Fn`.
pub trait UnsizedCoercible: RefCountedPointer + 'static {
	/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the closure.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	#[document_parameters("The closure to coerce.")]
	///
	/// ### Returns
	///
	/// The closure wrapped in the pointer type as a trait object.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = coerce_fn::<RcBrand, _, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	fn coerce_fn<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> Self::CloneableOf<'a, dyn 'a + Fn(A) -> B>;
}

/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
///
/// Free function version that dispatches to [the type class' associated function][`UnsizedCoercible::coerce_fn`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the closure.",
	"The brand of the pointer.",
	"The input type of the function.",
	"The output type of the function.",
	"The type of the closure function."
)]
///
#[document_parameters("The closure to coerce.")]
///
/// ### Returns
///
/// The closure wrapped in the pointer type as a trait object.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::unsized_coercible::*,
/// 	functions::*,
/// };
///
/// let f = coerce_fn::<RcBrand, _, _, _>(|x: i32| x + 1);
/// assert_eq!(f(1), 2);
/// ```
pub fn coerce_fn<'a, Brand: UnsizedCoercible, A: 'a, B: 'a, Func>(
	func: Func
) -> Brand::CloneableOf<'a, dyn 'a + Fn(A) -> B>
where
	Func: 'a + Fn(A) -> B,
{
	Brand::coerce_fn::<A, B>(func)
}
