//! Reference-counted pointer trait.

use super::pointer::Pointer;
use std::ops::Deref;

/// Extension trait for reference-counted pointers with shared ownership.
///
/// Adds `CloneableOf` associated type which is Clone + Deref. This follows
/// the pattern of `SendClonableFn` adding `SendOf` to `ClonableFn`.
pub trait RefCountedPointer: Pointer {
	/// The clonable pointer type constructor.
	///
	/// For Rc/Arc, this is the same as `Of<T>`.
	type CloneableOf<T: ?Sized>: Clone + Deref<Target = T>;

	/// Wraps a sized value in a clonable pointer.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> RefCountedPointer a`
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
	/// The value wrapped in the clonable pointer type.
	fn cloneable_new<T>(value: T) -> Self::CloneableOf<T>
	where
		Self::CloneableOf<T>: Sized;

	/// Attempts to unwrap the inner value if this is the sole reference.
	///
	/// ### Type Signature
	///
	/// `forall a. RefCountedPointer a -> Result a (RefCountedPointer a)`
	///
	/// ### Type Parameters
	///
	/// * `T`: The type of the wrapped value.
	///
	/// ### Parameters
	///
	/// * `ptr`: The pointer to attempt to unwrap.
	///
	/// ### Returns
	///
	/// `Ok(value)` if this is the sole reference, otherwise `Err(ptr)`.
	fn try_unwrap<T>(ptr: Self::CloneableOf<T>) -> Result<T, Self::CloneableOf<T>>;
}

/// Wraps a sized value in a clonable pointer.
///
/// ### Type Signature
///
/// `forall p a. RefCountedPointer p => a -> RefCountedPointer a`
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
/// The value wrapped in the clonable pointer type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::ref_counted_pointer::*, functions::*};
///
/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
/// let clone = ptr.clone();
/// assert_eq!(*clone, 42);
/// ```
pub fn cloneable_new<P: RefCountedPointer, T>(value: T) -> P::CloneableOf<T>
where
	P::CloneableOf<T>: Sized,
{
	P::cloneable_new(value)
}
