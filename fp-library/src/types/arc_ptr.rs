//! ArcBrand pointer implementation.
//!
//! This module provides implementations of the pointer traits for [`ArcBrand`],
//! enabling the use of `Arc` as a thread-safe reference-counted pointer in the library's
//! abstraction hierarchy.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::pointer::*, functions::*};
//!
//! let ptr = send_ref_counted_pointer_new::<ArcBrand, _>(42);
//! assert_eq!(*ptr, 42);
//! ```

use crate::{
	brands::ArcBrand,
	classes::{
		pointer::Pointer,
		ref_counted_pointer::RefCountedPointer,
		send_ref_counted_pointer::SendRefCountedPointer,
		send_unsized_coercible::SendUnsizedCoercible,
		thunk_wrapper::ThunkWrapper,
		unsized_coercible::UnsizedCoercible,
	},
};
use std::sync::{Arc, Mutex};

impl Pointer for ArcBrand {
	type Of<T: ?Sized> = Arc<T>;

	/// Wraps a sized value in an `Arc`.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Arc a`
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
	/// The value wrapped in an `Arc`.
	fn new<T>(value: T) -> Arc<T> {
		Arc::new(value)
	}
}

impl RefCountedPointer for ArcBrand {
	type CloneableOf<T: ?Sized> = Arc<T>;

	/// Wraps a sized value in an `Arc`.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Arc a`
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
	/// The value wrapped in an `Arc`.
	fn cloneable_new<T>(value: T) -> Arc<T> {
		Arc::new(value)
	}

	/// Attempts to unwrap the inner value if this is the sole reference.
	///
	/// ### Type Signature
	///
	/// `forall a. Arc a -> Result a (Arc a)`
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
	fn try_unwrap<T>(ptr: Arc<T>) -> Result<T, Arc<T>> {
		Arc::try_unwrap(ptr)
	}
}

impl SendRefCountedPointer for ArcBrand {
	type SendOf<T: ?Sized + Send + Sync> = Arc<T>;

	/// Wraps a sized value in an `Arc`.
	///
	/// ### Type Signature
	///
	/// `forall a. (Send a, Sync a) => a -> Arc a`
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
	/// The value wrapped in an `Arc`.
	fn send_new<T: Send + Sync>(value: T) -> Arc<T> {
		Arc::new(value)
	}
}

impl UnsizedCoercible for ArcBrand {
	/// Coerces a sized closure to a `dyn Fn` wrapped in an `Arc`.
	///
	/// ### Type Signature
	///
	/// `forall a b. (a -> b) -> Arc (dyn Fn a -> b)`
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
	/// The closure wrapped in an `Arc` as a trait object.
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Arc<dyn 'a + Fn(A) -> B> {
		Arc::new(f)
	}
}

impl SendUnsizedCoercible for ArcBrand {
	/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync` wrapped in an `Arc`.
	///
	/// ### Type Signature
	///
	/// `forall a b. (Send (a -> b), Sync (a -> b)) => (a -> b) -> Arc (dyn Fn a -> b + Send + Sync)`
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
	/// The closure wrapped in an `Arc` as a thread-safe trait object.
	fn coerce_fn_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Arc<dyn 'a + Fn(A) -> B + Send + Sync> {
		Arc::new(f)
	}
}

impl ThunkWrapper for ArcBrand {
	type Cell<T> = Mutex<Option<T>>;

	/// Creates a new cell containing the value.
	///
	/// ### Type Signature
	///
	/// `forall a. Option a -> Mutex (Option a)`
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
	fn new<T>(value: Option<T>) -> Self::Cell<T> {
		Mutex::new(value)
	}

	/// Takes the value out of the cell.
	///
	/// ### Type Signature
	///
	/// `forall a. Mutex (Option a) -> Option a`
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
		cell.lock().unwrap().take()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::classes::{
		pointer::new,
		ref_counted_pointer::cloneable_new,
		send_ref_counted_pointer::send_new,
	};

	/// Tests that `pointer_new` correctly creates an `Arc` wrapping the value.
	#[test]
	fn test_arc_new() {
		let ptr = new::<ArcBrand, _>(42);
		assert_eq!(*ptr, 42);
	}

	/// Tests that `ref_counted_pointer_new` correctly creates an `Arc` wrapping the value.
	#[test]
	fn test_arc_cloneable_new() {
		let ptr = cloneable_new::<ArcBrand, _>(42);
		assert_eq!(*ptr, 42);
	}

	/// Tests that `send_ref_counted_pointer_new` correctly creates an `Arc` wrapping the value.
	#[test]
	fn test_arc_send_new() {
		let ptr = send_new::<ArcBrand, _>(42);
		assert_eq!(*ptr, 42);
	}

	/// Tests that cloning the pointer works as expected (shared ownership).
	#[test]
	fn test_arc_clone() {
		let ptr = cloneable_new::<ArcBrand, _>(42);
		let clone = ptr.clone();
		assert_eq!(*clone, 42);
	}

	/// Tests `try_unwrap` behavior:
	/// - Returns `Ok(value)` when there is only one reference.
	/// - Returns `Err(ptr)` when there are multiple references.
	#[test]
	fn test_arc_try_unwrap() {
		let ptr = cloneable_new::<ArcBrand, _>(42);
		assert_eq!(ArcBrand::try_unwrap(ptr), Ok(42));

		let ptr = cloneable_new::<ArcBrand, _>(42);
		let _clone = ptr.clone();
		assert!(ArcBrand::try_unwrap(ptr).is_err());
	}
}
