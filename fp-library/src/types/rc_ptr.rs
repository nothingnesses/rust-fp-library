//! RcBrand pointer implementation.
//!
//! This module provides implementations of the pointer traits for [`RcBrand`],
//! enabling the use of `Rc` as a reference-counted pointer in the library's
//! abstraction hierarchy.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::pointer::*, functions::*};
//!
//! let ptr = pointer_new::<RcBrand, _>(42);
//! assert_eq!(*ptr, 42);
//! ```

use crate::{
	brands::RcBrand,
	classes::pointer::{Pointer, RefCountedPointer, ThunkWrapper, UnsizedCoercible},
};
use std::{cell::RefCell, rc::Rc};

impl Pointer for RcBrand {
	type Of<T: ?Sized> = Rc<T>;

	/// Wraps a sized value in an `Rc`.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Rc a`
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
	/// The value wrapped in an `Rc`.
	fn new<T>(value: T) -> Rc<T> {
		Rc::new(value)
	}
}

impl RefCountedPointer for RcBrand {
	type CloneableOf<T: ?Sized> = Rc<T>;

	/// Wraps a sized value in an `Rc`.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Rc a`
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
	/// The value wrapped in an `Rc`.
	fn cloneable_new<T>(value: T) -> Rc<T> {
		Rc::new(value)
	}

	/// Attempts to unwrap the inner value if this is the sole reference.
	///
	/// ### Type Signature
	///
	/// `forall a. Rc a -> Result a (Rc a)`
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
	fn try_unwrap<T>(ptr: Rc<T>) -> Result<T, Rc<T>> {
		Rc::try_unwrap(ptr)
	}
}

impl UnsizedCoercible for RcBrand {
	/// Coerces a sized closure to a `dyn Fn` wrapped in an `Rc`.
	///
	/// ### Type Signature
	///
	/// `forall a b. (a -> b) -> Rc (dyn Fn a -> b)`
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
	/// The closure wrapped in an `Rc` as a trait object.
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Rc<dyn 'a + Fn(A) -> B> {
		Rc::new(f)
	}
}

impl ThunkWrapper for RcBrand {
	type Cell<T> = RefCell<Option<T>>;

	/// Creates a new cell containing the value.
	///
	/// ### Type Signature
	///
	/// `forall a. Option a -> RefCell (Option a)`
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
	fn new_cell<T>(value: Option<T>) -> Self::Cell<T> {
		RefCell::new(value)
	}

	/// Takes the value out of the cell.
	///
	/// ### Type Signature
	///
	/// `forall a. RefCell (Option a) -> Option a`
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
	fn take<T>(cell: &Self::Cell<T>) -> Option<T> {
		cell.borrow_mut().take()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::classes::pointer::*;

	/// Tests that `pointer_new` correctly creates an `Rc` wrapping the value.
	#[test]
	fn test_rc_new() {
		let ptr = pointer_new::<RcBrand, _>(42);
		assert_eq!(*ptr, 42);
	}

	/// Tests that `ref_counted_new` correctly creates an `Rc` wrapping the value.
	#[test]
	fn test_rc_cloneable_new() {
		let ptr = ref_counted_new::<RcBrand, _>(42);
		assert_eq!(*ptr, 42);
	}

	/// Tests that cloning the pointer works as expected (shared ownership).
	#[test]
	fn test_rc_clone() {
		let ptr = ref_counted_new::<RcBrand, _>(42);
		let clone = ptr.clone();
		assert_eq!(*clone, 42);
	}

	/// Tests `try_unwrap` behavior:
	/// - Returns `Ok(value)` when there is only one reference.
	/// - Returns `Err(ptr)` when there are multiple references.
	#[test]
	fn test_rc_try_unwrap() {
		let ptr = ref_counted_new::<RcBrand, _>(42);
		assert_eq!(RcBrand::try_unwrap(ptr), Ok(42));

		let ptr = ref_counted_new::<RcBrand, _>(42);
		let _clone = ptr.clone();
		assert!(RcBrand::try_unwrap(ptr).is_err());
	}
}
