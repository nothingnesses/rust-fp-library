//! A hierarchy of traits for abstracting over different types of pointers, specifically focusing on reference-counted pointers ([`Rc`](`std::rc::Rc`), [`Arc`](std::sync::Arc)) and their capabilities.
//!
//! The hierarchy is as follows:
//! * [`Pointer`]: Base trait for any heap-allocated pointer.
//! * [`RefCountedPointer`][super::ref_counted_pointer::RefCountedPointer]: Extension for pointers that allow shared ownership (cloning).
//! * [`SendRefCountedPointer`][super::send_ref_counted_pointer::SendRefCountedPointer]: Extension for thread-safe reference-counted pointers.
//!
//! Additionally, [`UnsizedCoercible`][super::unsized_coercible::UnsizedCoercible] and [`SendUnsizedCoercible`][super::send_unsized_coercible::SendUnsizedCoercible] are provided to support
//! coercing sized closures into trait objects (`dyn Fn`).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let ptr = pointer_new::<RcBrand, _>(42);
//! assert_eq!(*ptr, 42);
//! ```

use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;
use std::ops::Deref;

/// Base type class for heap-allocated pointers.
///
/// This is the minimal abstraction: any type that can wrap a value and
/// dereference to it. Does NOT require Clone — that's added by subtraits.
pub trait Pointer {
	/// The pointer type constructor.
	///
	/// For `RcBrand`, this is `Rc<T>`. For `BoxBrand`, this would be `Box<T>`.
	type Of<T: ?Sized>: Deref<Target = T>;

	/// Wraps a sized value in the pointer.
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
	/// The value wrapped in the pointer type.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*};
	///
	/// let ptr = <RcBrand as Pointer>::new(42);
	/// assert_eq!(*ptr, 42);
	/// ```
	fn new<T>(value: T) -> Self::Of<T>
	where
		Self::Of<T>: Sized;
}

/// Wraps a sized value in the pointer.
///
/// ### Type Signature
///
#[hm_signature(Pointer)]
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
/// The value wrapped in the pointer type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let ptr = pointer_new::<RcBrand, _>(42);
/// assert_eq!(*ptr, 42);
/// ```
pub fn new<P: Pointer, T>(value: T) -> P::Of<T>
where
	P::Of<T>: Sized,
{
	P::new(value)
}
