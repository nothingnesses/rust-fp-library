//! Reference-counted pointers with shared ownership, unwrapping, and take-cell capabilities.
//!
//! In addition to cloneable shared pointers, this module provides a
//! [`TakeCellOf`](RefCountedPointer::TakeCellOf) abstraction: a cloneable cell
//! that holds a value which can be taken exactly once. This pairs the
//! appropriate interior mutability primitive with each pointer type
//! (`RefCell` for `Rc`, `Mutex` for `Arc`), enabling move-out-of-closure
//! patterns while preserving thread safety when needed.
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
//!
//! let cell = take_cell_new::<RcBrand, _>(99);
//! let cell_clone = cell.clone();
//! assert_eq!(take_cell_take::<RcBrand, _>(&cell), Some(99));
//! assert_eq!(take_cell_take::<RcBrand, _>(&cell_clone), None);
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
/// Adds [`CloneableOf`](Self::CloneableOf) (a cloneable, dereferenceable pointer)
/// and [`TakeCellOf`](Self::TakeCellOf) (a cloneable cell supporting one-shot value
/// extraction). The latter pairs the pointer with an appropriate interior mutability
/// primitive (`RefCell` for `Rc`, `Mutex` for `Arc`).
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

	/// A cloneable cell that holds an optional value which can be taken exactly once.
	///
	/// For [`RcBrand`](crate::brands::RcBrand), this is `Rc<RefCell<Option<T>>>`.
	/// For [`ArcBrand`](crate::brands::ArcBrand), this is `Arc<Mutex<Option<T>>>`.
	type TakeCellOf<'a, T: 'a>: Clone + 'a;

	/// Creates a new take-cell containing the given value.
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the value.", "The type of the value to store.")]
	///
	#[document_parameters("The value to store in the cell.")]
	///
	/// ### Returns
	///
	/// A new take-cell containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let cell = take_cell_new::<RcBrand, _>(42);
	/// assert_eq!(take_cell_take::<RcBrand, _>(&cell), Some(42));
	/// ```
	fn take_cell_new<'a, T: 'a>(value: T) -> Self::TakeCellOf<'a, T>;

	/// Takes the value out of the cell, leaving `None` behind.
	///
	/// Returns `Some(value)` the first time, `None` on subsequent calls.
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the value.", "The type of the stored value.")]
	///
	#[document_parameters("The cell to take the value from.")]
	///
	/// ### Returns
	///
	/// `Some(value)` if the cell still contains a value, `None` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let cell = take_cell_new::<RcBrand, _>(42);
	/// assert_eq!(take_cell_take::<RcBrand, _>(&cell), Some(42));
	/// assert_eq!(take_cell_take::<RcBrand, _>(&cell), None);
	/// ```
	fn take_cell_take<'a, T: 'a>(cell: &Self::TakeCellOf<'a, T>) -> Option<T>;
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

/// Creates a new take-cell containing the given value.
///
/// Free function version that dispatches to [the type class' associated function][`RefCountedPointer::take_cell_new`].
#[document_signature]
///
#[document_type_parameters(
	"The pointer brand.",
	"The lifetime of the value.",
	"The type of the value to store."
)]
///
#[document_parameters("The value to store in the cell.")]
///
/// ### Returns
///
/// A new take-cell containing the value.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let cell = take_cell_new::<RcBrand, _>(42);
/// assert_eq!(take_cell_take::<RcBrand, _>(&cell), Some(42));
/// ```
pub fn take_cell_new<'a, P: RefCountedPointer, T: 'a>(value: T) -> P::TakeCellOf<'a, T> {
	P::take_cell_new(value)
}

/// Takes the value out of a take-cell, leaving `None` behind.
///
/// Free function version that dispatches to [the type class' associated function][`RefCountedPointer::take_cell_take`].
#[document_signature]
///
#[document_type_parameters(
	"The pointer brand.",
	"The lifetime of the value.",
	"The type of the stored value."
)]
///
#[document_parameters("The cell to take the value from.")]
///
/// ### Returns
///
/// `Some(value)` if the cell still contains a value, `None` otherwise.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let cell = take_cell_new::<RcBrand, _>(42);
/// assert_eq!(take_cell_take::<RcBrand, _>(&cell), Some(42));
/// assert_eq!(take_cell_take::<RcBrand, _>(&cell), None);
/// ```
pub fn take_cell_take<'a, P: RefCountedPointer, T: 'a>(cell: &P::TakeCellOf<'a, T>) -> Option<T> {
	P::take_cell_take(cell)
}
