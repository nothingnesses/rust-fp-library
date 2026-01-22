//! Thread-safe reference-counted pointer trait.

use super::ref_counted_pointer::RefCountedPointer;
use std::ops::Deref;

/// Extension trait for thread-safe reference-counted pointers.
///
/// This follows the same pattern as `SendClonableFn` extends `ClonableFn`,
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
	/// * `T`: The type of the value to wrap.
	///
	/// ### Parameters
	///
	/// * `value`: The value to wrap.
	///
	/// ### Returns
	///
	/// The value wrapped in the thread-safe pointer type.
	fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>
	where
		Self::SendOf<T>: Sized;
}

/// Wraps a sized value in a thread-safe pointer.
///
/// ### Type Signature
///
/// `forall p a. (SendRefCountedPointer p, Send a) => a -> SendRefCountedPointer a`
///
/// ### Type Parameters
///
/// * `P`: The pointer brand.
/// * `T`: The type of the value to wrap.
///
/// ### Parameters
///
/// * `value`: The value to wrap.
///
/// ### Returns
///
/// The value wrapped in the thread-safe pointer type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::send_ref_counted_pointer::*, functions::*};
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
