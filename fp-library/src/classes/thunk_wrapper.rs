//! A trait for pointers that can wrap a thunk with interior mutability.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let cell = thunk_wrapper_new::<RcBrand, _>(Some(42));
//! assert_eq!(thunk_wrapper_take::<RcBrand, _>(&cell), Some(42));
//! ```

/// Trait for pointers that can wrap a thunk with interior mutability.
///
/// This is used by `Lazy` to store the thunk and clear it after execution.
pub trait ThunkWrapper {
	/// The cell type used to store the thunk.
	type Cell<T>;

	/// Creates a new cell containing the value.
	///
	/// ### Type Signature
	///
	/// `forall a. Option a -> Cell a`
	///
	/// ### Type Parameters
	///
	/// * `T`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `value`: The value to wrap.
	///
	/// ### Returns
	///
	/// A new cell containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let cell = thunk_wrapper_new::<RcBrand, _>(Some(42));
	/// assert_eq!(thunk_wrapper_take::<RcBrand, _>(&cell), Some(42));
	/// ```
	fn new<T>(value: Option<T>) -> Self::Cell<T>;

	/// Takes the value out of the cell.
	///
	/// ### Type Signature
	///
	/// `forall a. Cell a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `T`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `cell`: The cell to take the value from.
	///
	/// ### Returns
	///
	/// The value if it was present, or `None`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let cell = thunk_wrapper_new::<RcBrand, _>(Some(42));
	/// assert_eq!(thunk_wrapper_take::<RcBrand, _>(&cell), Some(42));
	/// assert_eq!(thunk_wrapper_take::<RcBrand, _>(&cell), None);
	/// ```
	fn take<T>(cell: &Self::Cell<T>) -> Option<T>;
}

/// Creates a new cell containing the value.
///
/// Free function version that dispatches to [the type class' associated function][`ThunkWrapper::new`].
///
/// ### Type Signature
///
/// `forall a. Option a -> Cell a`
///
/// ### Type Parameters
///
/// * `Brand`: The pointer brand.
/// * `T`: The type of the value.
///
/// ### Parameters
///
/// * `value`: The value to wrap.
///
/// ### Returns
///
/// A new cell containing the value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let cell = thunk_wrapper_new::<RcBrand, _>(Some(42));
/// assert_eq!(thunk_wrapper_take::<RcBrand, _>(&cell), Some(42));
/// ```
pub fn new<Brand: ThunkWrapper, T>(value: Option<T>) -> Brand::Cell<T> {
	Brand::new(value)
}

/// Takes the value out of the cell.
///
/// Free function version that dispatches to [the type class' associated function][`ThunkWrapper::take`].
///
/// ### Type Signature
///
/// `forall a. Cell a -> Option a`
///
/// ### Type Parameters
///
/// * `Brand`: The pointer brand.
/// * `T`: The type of the value.
///
/// ### Parameters
///
/// * `cell`: The cell to take the value from.
///
/// ### Returns
///
/// The value if it was present, or `None`.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let cell = thunk_wrapper_new::<RcBrand, _>(Some(42));
/// assert_eq!(thunk_wrapper_take::<RcBrand, _>(&cell), Some(42));
/// assert_eq!(thunk_wrapper_take::<RcBrand, _>(&cell), None);
/// ```
pub fn take<Brand: ThunkWrapper, T>(cell: &Brand::Cell<T>) -> Option<T> {
	Brand::take(cell)
}
