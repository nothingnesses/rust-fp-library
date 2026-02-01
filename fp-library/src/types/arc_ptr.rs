use crate::{
	brands::ArcBrand,
	classes::{
		pointer::Pointer, ref_counted_pointer::RefCountedPointer,
		send_ref_counted_pointer::SendRefCountedPointer,
		send_unsized_coercible::SendUnsizedCoercible, unsized_coercible::UnsizedCoercible,
	},
};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;
use std::sync::Arc;

impl Pointer for ArcBrand {
	type Of<T: ?Sized> = Arc<T>;

	/// Wraps a sized value in an `Arc`.
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
	/// The value wrapped in an `Arc`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let ptr = pointer_new::<ArcBrand, _>(42);
	/// assert_eq!(*ptr, 42);
	/// ```
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
	/// The value wrapped in an `Arc`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let ptr = ref_counted_pointer_new::<ArcBrand, _>(42);
	/// assert_eq!(*ptr, 42);
	/// ```
	fn cloneable_new<T>(value: T) -> Arc<T> {
		Arc::new(value)
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
	/// let ptr = ref_counted_pointer_new::<ArcBrand, _>(42);
	/// assert_eq!(try_unwrap::<ArcBrand, _>(ptr), Ok(42));
	/// ```
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
	#[hm_signature(Send)]
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
	/// The value wrapped in an `Arc`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let ptr = send_ref_counted_pointer_new::<ArcBrand, _>(42);
	/// assert_eq!(*ptr, 42);
	/// ```
	fn send_new<T: Send + Sync>(value: T) -> Arc<T> {
		Arc::new(value)
	}
}

impl UnsizedCoercible for ArcBrand {
	/// Coerces a sized closure to a `dyn Fn` wrapped in an `Arc`.
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
	/// The closure wrapped in an `Arc` as a trait object.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = coerce_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Arc<dyn 'a + Fn(A) -> B> {
		Arc::new(f)
	}
}

impl SendUnsizedCoercible for ArcBrand {
	/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync` wrapped in an `Arc`.
	///
	/// ### Type Signature
	///
	#[hm_signature(Send)]
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
	/// The closure wrapped in an `Arc` as a thread-safe trait object.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = coerce_send_fn::<ArcBrand, _, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	fn coerce_send_fn<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Arc<dyn 'a + Fn(A) -> B + Send + Sync> {
		Arc::new(f)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::classes::{
		pointer::new, ref_counted_pointer::cloneable_new, send_ref_counted_pointer::send_new,
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
