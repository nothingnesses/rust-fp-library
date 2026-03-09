//! Thread-safe reference-counted pointer abstraction using [`Arc`](std::sync::Arc).
//!
//! Provides trait implementations for using `Arc` in the library's pointer abstraction hierarchy.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let ptr = send_ref_counted_pointer_new::<ArcBrand, _>(42);
//! assert_eq!(*ptr, 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::ArcBrand,
			classes::{
				Pointer,
				RefCountedPointer,
				SendRefCountedPointer,
				SendUnsizedCoercible,
				UnsizedCoercible,
			},
		},
		fp_macros::*,
		std::sync::{
			Arc,
			Mutex,
		},
	};

	impl Pointer for ArcBrand {
		type Of<'a, T: ?Sized + 'a> = Arc<T>;

		/// Wraps a sized value in an `Arc`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("The value wrapped in an `Arc`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let ptr = pointer_new::<ArcBrand, _>(42);
		/// assert_eq!(*ptr, 42);
		/// ```
		fn new<'a, T: 'a>(value: T) -> Arc<T> {
			Arc::new(value)
		}
	}

	impl RefCountedPointer for ArcBrand {
		type CloneableOf<'a, T: ?Sized + 'a> = Arc<T>;
		type TakeCellOf<'a, T: 'a> = Arc<Mutex<Option<T>>>;

		/// Wraps a sized value in an `Arc`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("The value wrapped in an `Arc`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let ptr = ref_counted_pointer_new::<ArcBrand, _>(42);
		/// assert_eq!(*ptr, 42);
		/// ```
		fn cloneable_new<'a, T: 'a>(value: T) -> Arc<T> {
			Arc::new(value)
		}

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
		#[document_returns("`Ok(value)` if this is the sole reference, otherwise `Err(ptr)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let ptr = ref_counted_pointer_new::<ArcBrand, _>(42);
		/// assert_eq!(try_unwrap::<ArcBrand, _>(ptr), Ok(42));
		/// ```
		fn try_unwrap<'a, T: 'a>(ptr: Arc<T>) -> Result<T, Arc<T>> {
			Arc::try_unwrap(ptr)
		}

		/// Creates a new take-cell containing the given value.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to store.")]
		///
		#[document_parameters("The value to store in the cell.")]
		///
		#[document_returns("A new `Arc<Mutex<Option<T>>>` containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let cell = ArcBrand::take_cell_new(42);
		/// assert_eq!(ArcBrand::take_cell_take(&cell), Some(42));
		/// ```
		fn take_cell_new<'a, T: 'a>(value: T) -> Arc<Mutex<Option<T>>> {
			Arc::new(Mutex::new(Some(value)))
		}

		/// Takes the value out of the cell, leaving `None` behind.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the stored value.")]
		///
		#[document_parameters("The cell to take the value from.")]
		///
		#[document_returns("`Some(value)` if the cell still contains a value, `None` otherwise.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let cell = ArcBrand::take_cell_new(42);
		/// assert_eq!(ArcBrand::take_cell_take(&cell), Some(42));
		/// assert_eq!(ArcBrand::take_cell_take(&cell), None);
		/// ```
		fn take_cell_take<'a, T: 'a>(cell: &Arc<Mutex<Option<T>>>) -> Option<T> {
			cell.lock().unwrap().take()
		}
	}

	impl SendRefCountedPointer for ArcBrand {
		type SendOf<'a, T: ?Sized + Send + Sync + 'a> = Arc<T>;

		/// Wraps a sized value in an `Arc`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("The value wrapped in an `Arc`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let ptr = send_ref_counted_pointer_new::<ArcBrand, _>(42);
		/// assert_eq!(*ptr, 42);
		/// ```
		fn send_new<'a, T: Send + Sync + 'a>(value: T) -> Arc<T> {
			Arc::new(value)
		}
	}

	impl UnsizedCoercible for ArcBrand {
		/// Coerces a sized closure to a `dyn Fn` wrapped in an `Arc`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the closure.",
			"The input type of the function.",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to coerce.")]
		///
		#[document_returns("The closure wrapped in an `Arc` as a trait object.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = coerce_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
		/// assert_eq!(f(1), 2);
		/// ```
		fn coerce_fn<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> Arc<dyn 'a + Fn(A) -> B> {
			Arc::new(f)
		}
	}

	impl SendUnsizedCoercible for ArcBrand {
		/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync` wrapped in an `Arc`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the closure.",
			"The input type of the function.",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to coerce.")]
		///
		#[document_returns("The closure wrapped in an `Arc` as a thread-safe trait object.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = coerce_send_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
		/// assert_eq!(f(1), 2);
		/// ```
		fn coerce_send_fn<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(A) -> B + Send + Sync
		) -> Arc<dyn 'a + Fn(A) -> B + Send + Sync> {
			Arc::new(f)
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		brands::ArcBrand,
		classes::{
			RefCountedPointer,
			pointer::new,
			ref_counted_pointer::cloneable_new,
			send_ref_counted_pointer::send_new,
		},
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
