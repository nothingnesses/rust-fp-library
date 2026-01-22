//! Pointer abstraction traits.
//!
//! This module defines a hierarchy of traits for abstracting over different types of pointers,
//! specifically focusing on reference-counted pointers (`Rc`, `Arc`) and their capabilities.
//!
//! The hierarchy is as follows:
//! * [`Pointer`]: Base trait for any heap-allocated pointer.
//! * [`RefCountedPointer`]: Extension for pointers that allow shared ownership (cloning).
//! * [`SendRefCountedPointer`]: Extension for thread-safe reference-counted pointers.
//!
//! Additionally, [`UnsizedCoercible`] and [`SendUnsizedCoercible`] are provided to support
//! coercing sized closures into trait objects (`dyn Fn`).

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
	/// `forall a. a -> Pointer a`
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
	/// The value wrapped in the pointer type.
	fn new<T>(value: T) -> Self::Of<T>
	where
		Self::Of<T>: Sized;
}

/// Extension trait for reference-counted pointers with shared ownership.
///
/// Adds `CloneableOf` associated type which is Clone + Deref. This follows
/// the pattern of `SendClonableFn` adding `SendOf` to `ClonableFn`.
pub trait RefCountedPointer: Pointer {
	/// The clonable pointer type constructor.
	///
	/// For Rc/Arc, this is the same as `Of<T>`.
	type CloneableOf<T: ?Sized>: Clone + Deref<Target = T>;

	/// Wraps a sized value in a clonable pointer.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> RefCountedPointer a`
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
	/// The value wrapped in the clonable pointer type.
	fn cloneable_new<T>(value: T) -> Self::CloneableOf<T>
	where
		Self::CloneableOf<T>: Sized;

	/// Attempts to unwrap the inner value if this is the sole reference.
	///
	/// ### Type Signature
	///
	/// `forall a. RefCountedPointer a -> Result a (RefCountedPointer a)`
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
	fn try_unwrap<T>(ptr: Self::CloneableOf<T>) -> Result<T, Self::CloneableOf<T>>;
}

/// Extension trait for thread-safe reference-counted pointers.
///
/// This follows the same pattern as `SendClonableFn` extends `ClonableFn`,
/// adding a `SendOf` associated type with explicit `Send + Sync` bounds.
pub trait SendRefCountedPointer: RefCountedPointer {
	/// The thread-safe pointer type constructor.
	///
	/// For `ArcBrand`, this is `Arc<T>` where `T: Send + Sync`.
	type SendOf<T: ?Sized + Send + Sync>: Clone + Send + Sync + Deref<Target = T>;

	/// Wraps a sized value in a thread-safe pointer.
	///
	/// ### Type Signature
	///
	/// `forall a. Send a => a -> SendRefCountedPointer a`
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
	/// The value wrapped in the thread-safe pointer type.
	fn send_new<T: Send + Sync>(value: T) -> Self::SendOf<T>
	where
		Self::SendOf<T>: Sized;
}

/// Trait for pointer brands that can perform unsized coercion to `dyn Fn`.
pub trait UnsizedCoercible: RefCountedPointer + 'static {
	/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
	///
	/// ### Type Signature
	///
	/// `forall a b. (a -> b) -> UnsizedCoercible (a -> b)`
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
	/// The closure wrapped in the pointer type as a trait object.
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::CloneableOf<dyn 'a + Fn(A) -> B>;
}

/// Extension trait for pointer brands that can coerce to thread-safe `dyn Fn + Send + Sync`.
pub trait SendUnsizedCoercible: UnsizedCoercible + SendRefCountedPointer + 'static {
	/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
	///
	/// ### Type Signature
	///
	/// `forall a b. (Send (a -> b)) => (a -> b) -> SendUnsizedCoercible (a -> b)`
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
	/// The closure wrapped in the pointer type as a thread-safe trait object.
	fn coerce_fn_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Self::SendOf<dyn 'a + Fn(A) -> B + Send + Sync>;
}

/// Wraps a sized value in the pointer.
///
/// ### Type Signature
///
/// `forall p a. Pointer p => a -> Pointer a`
///
/// ### Type Parameters
///
/// * `P`: The pointer brand.
/// * `T`: The type of the value to wrap.
///
/// ### Parameters
///
/// * `value`: The value to wrap.
///
/// ### Returns
///
/// The value wrapped in the pointer type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::pointer::*, functions::*};
///
/// let ptr = pointer_new::<RcBrand, _>(42);
/// assert_eq!(*ptr, 42);
/// ```
pub fn pointer_new<P: Pointer, T>(value: T) -> P::Of<T>
where
	P::Of<T>: Sized,
{
	P::new(value)
}

/// Wraps a sized value in a clonable pointer.
///
/// ### Type Signature
///
/// `forall p a. RefCountedPointer p => a -> RefCountedPointer a`
///
/// ### Type Parameters
///
/// * `P`: The pointer brand.
/// * `T`: The type of the value to wrap.
///
/// ### Parameters
///
/// * `value`: The value to wrap.
///
/// ### Returns
///
/// The value wrapped in the clonable pointer type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::pointer::*, functions::*};
///
/// let ptr = ref_counted_new::<RcBrand, _>(42);
/// let clone = ptr.clone();
/// assert_eq!(*clone, 42);
/// ```
pub fn ref_counted_new<P: RefCountedPointer, T>(value: T) -> P::CloneableOf<T>
where
	P::CloneableOf<T>: Sized,
{
	P::cloneable_new(value)
}

/// Wraps a sized value in a thread-safe pointer.
///
/// ### Type Signature
///
/// `forall p a. (SendRefCountedPointer p, Send a) => a -> SendRefCountedPointer a`
///
/// ### Type Parameters
///
/// * `P`: The pointer brand.
/// * `T`: The type of the value to wrap.
///
/// ### Parameters
///
/// * `value`: The value to wrap.
///
/// ### Returns
///
/// The value wrapped in the thread-safe pointer type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::pointer::*, functions::*};
///
/// let ptr = send_ref_counted_new::<ArcBrand, _>(42);
/// assert_eq!(*ptr, 42);
/// ```
pub fn send_ref_counted_new<P: SendRefCountedPointer, T: Send + Sync>(value: T) -> P::SendOf<T>
where
	P::SendOf<T>: Sized,
{
	P::send_new(value)
}
