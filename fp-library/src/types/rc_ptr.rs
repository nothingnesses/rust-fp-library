//! Single-threaded reference-counted pointer abstraction using [`Rc`](std::rc::Rc).
//!
//! Provides trait implementations for using `Rc` in the library's pointer abstraction hierarchy. Not thread-safe; use [`ArcBrand`](crate::brands::ArcBrand) for multi-threaded contexts.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let ptr = pointer_new::<RcBrand, _>(42);
//! assert_eq!(*ptr, 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::RcBrand,
			classes::{
				Pointer,
				RefCountedPointer,
				UnsizedCoercible,
			},
		},
		fp_macros::*,
		std::{
			cell::RefCell,
			rc::Rc,
		},
	};

	impl Pointer for RcBrand {
		type Of<'a, T: ?Sized + 'a> = Rc<T>;

		/// Wraps a sized value in an `Rc`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("The value wrapped in an `Rc`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let ptr = pointer_new::<RcBrand, _>(42);
		/// assert_eq!(*ptr, 42);
		/// ```
		fn new<'a, T: 'a>(value: T) -> Rc<T> {
			Rc::new(value)
		}
	}

	impl RefCountedPointer for RcBrand {
		type CloneableOf<'a, T: ?Sized + 'a> = Rc<T>;
		type TakeCellOf<'a, T: 'a> = Rc<RefCell<Option<T>>>;

		/// Wraps a sized value in an `Rc`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("The value wrapped in an `Rc`.")]
		///
		#[document_examples]
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
		fn cloneable_new<'a, T: 'a>(value: T) -> Rc<T> {
			Rc::new(value)
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
		/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
		/// assert_eq!(try_unwrap::<RcBrand, _>(ptr), Ok(42));
		/// ```
		fn try_unwrap<'a, T: 'a>(ptr: Rc<T>) -> Result<T, Rc<T>> {
			Rc::try_unwrap(ptr)
		}

		/// Creates a new take-cell containing the given value.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to store.")]
		///
		#[document_parameters("The value to store in the cell.")]
		///
		#[document_returns("A new `Rc<RefCell<Option<T>>>` containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let cell = RcBrand::take_cell_new(42);
		/// assert_eq!(RcBrand::take_cell_take(&cell), Some(42));
		/// ```
		fn take_cell_new<'a, T: 'a>(value: T) -> Rc<RefCell<Option<T>>> {
			Rc::new(RefCell::new(Some(value)))
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
		/// let cell = RcBrand::take_cell_new(42);
		/// assert_eq!(RcBrand::take_cell_take(&cell), Some(42));
		/// assert_eq!(RcBrand::take_cell_take(&cell), None);
		/// ```
		fn take_cell_take<'a, T: 'a>(cell: &Rc<RefCell<Option<T>>>) -> Option<T> {
			cell.borrow_mut().take()
		}
	}

	impl UnsizedCoercible for RcBrand {
		/// Coerces a sized closure to a `dyn Fn` wrapped in an `Rc`.
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
		#[document_returns("The closure wrapped in an `Rc` as a trait object.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = coerce_fn::<RcBrand, _, _>(|x: i32| x + 1);
		/// assert_eq!(f(1), 2);
		/// ```
		fn coerce_fn<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> Rc<dyn 'a + Fn(A) -> B> {
			Rc::new(f)
		}

		/// Coerces a sized by-reference closure to a `dyn Fn(&A) -> B` wrapped in an `Rc`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the closure.",
			"The input type (received by reference).",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to coerce.")]
		///
		#[document_returns("The closure wrapped in an `Rc` as a trait object.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::unsized_coercible::*,
		/// };
		///
		/// let f = coerce_ref_fn::<RcBrand, _, _>(|x: &i32| *x + 1);
		/// assert_eq!(f(&1), 2);
		/// ```
		fn coerce_ref_fn<'a, A: 'a, B: 'a>(f: impl 'a + Fn(&A) -> B) -> Rc<dyn 'a + Fn(&A) -> B> {
			Rc::new(f)
		}
	}
}

#[cfg(test)]
mod tests {

	use crate::{
		brands::RcBrand,
		classes::{
			RefCountedPointer,
			pointer::new,
			ref_counted_pointer::cloneable_new,
		},
	};

	/// Tests that `pointer_new` correctly creates an `Rc` wrapping the value.
	#[test]
	fn test_rc_new() {
		let ptr = new::<RcBrand, _>(42);
		assert_eq!(*ptr, 42);
	}

	/// Tests that `ref_counted_pointer_new` correctly creates an `Rc` wrapping the value.
	#[test]
	fn test_rc_cloneable_new() {
		let ptr = cloneable_new::<RcBrand, _>(42);
		assert_eq!(*ptr, 42);
	}

	/// Tests that cloning the pointer works as expected (shared ownership).
	#[test]
	fn test_rc_clone() {
		let ptr = cloneable_new::<RcBrand, _>(42);
		let clone = ptr.clone();
		assert_eq!(*clone, 42);
	}

	/// Tests `try_unwrap` behavior:
	/// - Returns `Ok(value)` when there is only one reference.
	/// - Returns `Err(ptr)` when there are multiple references.
	#[test]
	fn test_rc_try_unwrap() {
		let ptr = cloneable_new::<RcBrand, _>(42);
		assert_eq!(RcBrand::try_unwrap(ptr), Ok(42));

		let ptr = cloneable_new::<RcBrand, _>(42);
		let _clone = ptr.clone();
		assert!(RcBrand::try_unwrap(ptr).is_err());
	}
}
