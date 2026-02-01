//! A trait for pointer brands that can coerce to thread-safe `dyn Fn + Send + Sync`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let f = coerce_send_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
//! assert_eq!(f(1), 2);
//! ```

use super::{SendRefCountedPointer, UnsizedCoercible};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// Extension trait for pointer brands that can coerce to thread-safe `dyn Fn + Send + Sync`.
pub trait SendUnsizedCoercible: UnsizedCoercible + SendRefCountedPointer + 'static {
	/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
	///
	/// ### Type Signature
	///
	/// `forall a b. (Send (a -> b)) => (a -> b) -> SendUnsizedCoercible (a -> b)`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The closure to coerce.")]
	///
	/// ### Returns
	///
	/// The closure wrapped in the pointer type as a thread-safe trait object.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = coerce_send_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	fn coerce_send_fn<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::SendOf<dyn 'a + Fn(A) -> B + Send + Sync>;
}

/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
///
/// Free function version that dispatches to [the type class' associated function][`SendUnsizedCoercible::coerce_send_fn`].
///
/// ### Type Signature
///
#[hm_signature(Send)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"Undocumented",
	"The brand of the pointer.",
	"The input type of the function.",
	"The output type of the function.",
	("F", "The type of the closure to coerce.")
)]
///
/// ### Parameters
///
#[doc_params("The closure to coerce.")]
///
/// ### Returns
///
/// The closure wrapped in the pointer type as a thread-safe trait object.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::send_unsized_coercible::*, functions::*};
///
/// let f = coerce_send_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
/// assert_eq!(f(1), 2);
/// ```
pub fn coerce_send_fn<'a, Brand: SendUnsizedCoercible, A, B, Func>(
	func: Func
) -> Brand::SendOf<dyn 'a + Fn(A) -> B + Send + Sync>
where
	Func: 'a + Fn(A) -> B + Send + Sync,
{
	Brand::coerce_send_fn::<A, B>(func)
}
