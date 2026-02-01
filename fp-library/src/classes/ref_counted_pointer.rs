//! A trait for reference-counted pointers with shared ownership.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
//! let clone = ptr.clone();
//! assert_eq!(*clone, 42);
//! ```

use super::Pointer;
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;
use std::ops::Deref;

/// Extension trait for reference-counted pointers with shared ownership.
///
/// Adds `CloneableOf` associated type which is Clone + Deref. This follows
/// the pattern of `SendCloneableFn` adding `SendOf` to `CloneableFn`.
pub trait RefCountedPointer: Pointer {
	/// The cloneable pointer type constructor.
	///
	/// For Rc/Arc, this is the same as `Of<T>`.
	type CloneableOf<T: ?Sized>: Clone + Deref<Target = T>;

	/// Wraps a sized value in a cloneable pointer.
	///
	/// ### Type Signature
	///
	#[hm_signature]
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
	/// The value wrapped in the cloneable pointer type.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
	/// assert_eq!(*ptr, 42);
	/// ```
	fn cloneable_new<T>(value: T) -> Self::CloneableOf<T>
	where
		Self::CloneableOf<T>: Sized;

	/// Attempts to unwrap the inner value if this is the sole reference.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the wrapped value.")]
	///
	/// ### Parameters
	///
	#[doc_params("The pointer to attempt to unwrap.")]
	///
	/// ### Returns
	///
	/// `Ok(value)` if this is the sole reference, otherwise `Err(ptr)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
	/// assert_eq!(try_unwrap::<RcBrand, _>(ptr), Ok(42));
	///
	/// let ptr1 = ref_counted_pointer_new::<RcBrand, _>(42);
	/// let ptr2 = ptr1.clone();
	/// assert!(try_unwrap::<RcBrand, _>(ptr1).is_err());
	/// ```
	fn try_unwrap<T>(ptr: Self::CloneableOf<T>) -> Result<T, Self::CloneableOf<T>>;
}

/// Attempts to unwrap the inner value if this is the sole reference.
///
/// Free function version that dispatches to [the type class' associated function][`RefCountedPointer::try_unwrap`].
///
/// ### Type Signature
///
#[hm_signature]
///
/// ### Type Parameters
///
#[doc_type_params("The pointer brand.", "The type of the wrapped value.")]
///
/// ### Parameters
///
#[doc_params("The pointer to attempt to unwrap.")]
///
/// ### Returns
///
/// `Ok(value)` if this is the sole reference, otherwise `Err(ptr)`.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
/// assert_eq!(try_unwrap::<RcBrand, _>(ptr), Ok(42));
///
/// let ptr1 = ref_counted_pointer_new::<RcBrand, _>(42);
/// let ptr2 = ptr1.clone();
/// assert!(try_unwrap::<RcBrand, _>(ptr1).is_err());
/// ```
pub fn try_unwrap<P: RefCountedPointer, T>(ptr: P::CloneableOf<T>) -> Result<T, P::CloneableOf<T>> {
	P::try_unwrap(ptr)
}

/// Wraps a sized value in a cloneable pointer.
///
/// ### Type Signature
///
#[hm_signature(RefCountedPointer)]
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
/// The value wrapped in the cloneable pointer type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::*, functions::*};
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
