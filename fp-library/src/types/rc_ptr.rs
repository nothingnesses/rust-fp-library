use crate::{
	brands::RcBrand,
	classes::{
		pointer::Pointer, ref_counted_pointer::RefCountedPointer,
		unsized_coercible::UnsizedCoercible,
	},
};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;
use std::rc::Rc;

impl Pointer for RcBrand {
	type Of<T: ?Sized> = Rc<T>;

	/// Wraps a sized value in an `Rc`.
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
	/// The value wrapped in an `Rc`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let ptr = pointer_new::<RcBrand, _>(42);
	/// assert_eq!(*ptr, 42);
	/// ```
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
	/// The value wrapped in an `Rc`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let ptr = ref_counted_pointer_new::<RcBrand, _>(42);
	/// assert_eq!(*ptr, 42);
	/// ```
	fn cloneable_new<T>(value: T) -> Rc<T> {
		Rc::new(value)
	}

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
	/// ```
	fn try_unwrap<T>(ptr: Rc<T>) -> Result<T, Rc<T>> {
		Rc::try_unwrap(ptr)
	}
}

impl UnsizedCoercible for RcBrand {
	/// Coerces a sized closure to a `dyn Fn` wrapped in an `Rc`.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the closure.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The closure to coerce.")]
	///
	/// ### Returns
	///
	/// The closure wrapped in an `Rc` as a trait object.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = coerce_fn::<RcBrand, _, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Rc<dyn 'a + Fn(A) -> B> {
		Rc::new(f)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::classes::{pointer::new, ref_counted_pointer::cloneable_new};

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
