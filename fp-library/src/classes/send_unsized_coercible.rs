//! Thread-safe unsized coercible pointer trait.

use super::{send_ref_counted_pointer::SendRefCountedPointer, unsized_coercible::UnsizedCoercible};

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
	/// * `A`: The input type of the function.
	/// * `B`: The output type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to coerce.
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
	/// let f = ArcBrand::coerce_fn_send(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	fn coerce_fn_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::SendOf<dyn 'a + Fn(A) -> B + Send + Sync>;
}
