//! Reference-counted pointers with shared ownership and unwrapping capabilities.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
//! let clone = ptr.clone();
//! assert_eq!(*clone, 42);
//! ```

use {
	super::Pointer,
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
	std::ops::Deref,
};

/// Extension trait for reference-counted pointers with shared ownership.
///
/// Adds `CloneableOf` associated type which is Clone + Deref. This follows
/// the pattern of `SendCloneableFn` adding `SendOf` to `CloneableFn`.
pub trait RefCountedPointer: Pointer {
	/// The cloneable pointer type constructor.
	///
	/// For Rc/Arc, this is the same as `Of<'a, T>`.
	type CloneableOf<'a, T: ?Sized + 'a>: Clone + Deref<Target = T> + 'a;

	/// Wraps a sized value in a cloneable pointer.
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
	///
	#[document_parameters("The value to wrap.")]
	///
	/// ### Returns
	///
	/// The value wrapped in the cloneable pointer type.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
	/// assert_eq!(*ptr, 42);
	/// ```
	fn cloneable_new<'a, T: 'a>(value: T) -> Self::CloneableOf<'a, T>
	where
		Self::CloneableOf<'a, T>: Sized;

	/// Attempts to unwrap the inner value if this is the sole reference.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the wrapped value.",
		"The type of the wrapped value."
	)]
	///
	#[document_parameters("The pointer to attempt to unwrap.")]
	///
	/// ### Returns
	///
	/// `Ok(value)` if this is the sole reference, otherwise `Err(ptr)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
	/// assert_eq!(try_unwrap::<RcBrand, _>(ptr), Ok(42));
	///
	/// let ptr1 = ref_counted_pointer_new::<RcBrand, _>(42);
	/// let ptr2 = ptr1.clone();
	/// assert!(try_unwrap::<RcBrand, _>(ptr1).is_err());
	/// ```
	fn try_unwrap<'a, T: 'a>(ptr: Self::CloneableOf<'a, T>) -> Result<T, Self::CloneableOf<'a, T>>;
}

/// Attempts to unwrap the inner value if this is the sole reference.
///
/// Free function version that dispatches to [the type class' associated function][`RefCountedPointer::try_unwrap`].
#[document_signature]
///
#[document_type_parameters(
	"The pointer brand.",
	"The lifetime of the wrapped value.",
	"The type of the wrapped value."
)]
///
#[document_parameters("The pointer to attempt to unwrap.")]
///
/// ### Returns
///
/// `Ok(value)` if this is the sole reference, otherwise `Err(ptr)`.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
/// assert_eq!(try_unwrap::<RcBrand, _>(ptr), Ok(42));
///
/// let ptr1 = ref_counted_pointer_new::<RcBrand, _>(42);
/// let ptr2 = ptr1.clone();
/// assert!(try_unwrap::<RcBrand, _>(ptr1).is_err());
/// ```
pub fn try_unwrap<'a, P: RefCountedPointer, T: 'a>(
	ptr: P::CloneableOf<'a, T>
) -> Result<T, P::CloneableOf<'a, T>> {
	P::try_unwrap(ptr)
}

/// Wraps a sized value in a cloneable pointer.
#[document_signature]
///
#[document_type_parameters(
	"The pointer brand.",
	"The lifetime of the value.",
	"The type of the value to wrap."
)]
///
#[document_parameters("The value to wrap.")]
///
/// ### Returns
///
/// The value wrapped in the cloneable pointer type.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::*,
/// 	functions::*,
/// };
///
/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
/// let clone = ptr.clone();
/// assert_eq!(*clone, 42);
/// ```
pub fn cloneable_new<'a, P: RefCountedPointer, T: 'a>(value: T) -> P::CloneableOf<'a, T>
where
	P::CloneableOf<'a, T>: Sized, {
	P::cloneable_new(value)
}
