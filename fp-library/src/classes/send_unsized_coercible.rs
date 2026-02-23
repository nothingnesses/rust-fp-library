//! Pointer brands that can perform unsized coercion to thread-safe `dyn Fn` trait objects.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = coerce_send_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
//! assert_eq!(f(1), 2);
//! ```

use {
	super::{
		SendRefCountedPointer,
		UnsizedCoercible,
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
};

/// Extension trait for pointer brands that can coerce to thread-safe `dyn Fn + Send + Sync`.
pub trait SendUnsizedCoercible: UnsizedCoercible + SendRefCountedPointer + 'static {
	/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
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
	/// The closure wrapped in the pointer type as a thread-safe trait object.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = coerce_send_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	fn coerce_send_fn<'a, A: 'a, B: 'a>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::SendOf<'a, dyn 'a + Fn(A) -> B + Send + Sync>;
}

/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
///
/// Free function version that dispatches to [the type class' associated function][`SendUnsizedCoercible::coerce_send_fn`].
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
/// The closure wrapped in the pointer type as a thread-safe trait object.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::send_unsized_coercible::*,
/// 	functions::*,
/// };
///
/// let f = coerce_send_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
/// assert_eq!(f(1), 2);
/// ```
pub fn coerce_send_fn<'a, Brand: SendUnsizedCoercible, A: 'a, B: 'a, Func>(
	func: Func
) -> Brand::SendOf<'a, dyn 'a + Fn(A) -> B + Send + Sync>
where
	Func: 'a + Fn(A) -> B + Send + Sync, {
	Brand::coerce_send_fn::<A, B>(func)
}
