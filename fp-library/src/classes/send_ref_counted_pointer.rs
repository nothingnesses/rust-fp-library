//! A trait for thread-safe reference-counted pointers.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let ptr = send_ref_counted_pointer_new::<ArcBrand, _>(42);
//! assert_eq!(*ptr, 42);
//! ```

use super::RefCountedPointer;
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;
use std::ops::Deref;

/// Extension trait for thread-safe reference-counted pointers.
///
/// This follows the same pattern as `SendCloneableFn` extends `CloneableFn`,
/// adding a `SendOf` associated type with explicit `Send + Sync` bounds.
pub trait SendRefCountedPointer: RefCountedPointer {
	/// The thread-safe pointer type constructor.
	///
	/// For `ArcBrand`, this is `Arc<T>` where `T: Send + Sync`.
	type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;

	/// Wraps a sized value in a thread-safe pointer.
	///
	/// ### Type Signature
	///
	/// `forall a. Send a => a -> SendRefCountedPointer a`
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the value to wrap.")]
	///
	/// ### Parameters
	///
	#[doc_params("The value to wrap.")]
	///
	/// ### Returns
	///
	/// The value wrapped in the thread-safe pointer type.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let ptr = send_ref_counted_pointer_new::<ArcBrand, _>(42);
	/// assert_eq!(*ptr, 42);
	/// ```
	fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>
	where
		Self::SendOf<T>: Sized;
}

/// Wraps a sized value in a thread-safe pointer.
///
/// ### Type Signature
///
#[hm_signature(SendRefCountedPointer)]
///
/// ### Type Parameters
///
#[doc_type_params("The pointer brand.", "The type of the value to wrap.")]
///
/// ### Parameters
///
#[doc_params("The value to wrap.")]
///
/// ### Returns
///
/// The value wrapped in the thread-safe pointer type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let ptr = send_ref_counted_pointer_new::<ArcBrand, _>(42);
/// assert_eq!(*ptr, 42);
/// ```
pub fn send_new<P: SendRefCountedPointer, T: Send + Sync>(value: T) -> P::SendOf<T>
where
	P::SendOf<T>: Sized,
{
	P::send_new(value)
}
