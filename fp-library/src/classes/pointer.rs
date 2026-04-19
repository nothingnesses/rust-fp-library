//! Independent traits for abstracting over different types of pointers and their capabilities.
//!
//! The pointer traits are independent (no supertrait relationships between them):
//! * [`Pointer`]: Base trait for any heap-allocated pointer (`Box`, `Rc`, `Arc`).
//! * [`RefCountedPointer`][super::ref_counted_pointer::RefCountedPointer]: Clonable reference-counted pointer (`Rc`, `Arc`).
//! * [`SendRefCountedPointer`][super::send_ref_counted_pointer::SendRefCountedPointer]: Thread-safe reference-counted pointer (`Arc`).
//!
//! Coercion traits wrap closures into `dyn Fn` trait objects:
//! * [`ToDynFn`][super::to_dyn_fn::ToDynFn]: Coerce to `dyn Fn` behind any pointer (extends `Pointer`).
//! * [`ToDynCloneFn`][super::to_dyn_clone_fn::ToDynCloneFn]: Coerce to `dyn Fn` behind a clonable pointer (extends `RefCountedPointer`).
//! * [`ToDynSendFn`][super::to_dyn_send_fn::ToDynSendFn]: Coerce to `dyn Fn + Send + Sync` behind a thread-safe pointer (extends `SendRefCountedPointer`).
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
		fp_macros::*,
		std::ops::Deref,
	};

	/// Base type class for heap-allocated pointers.
	///
	/// This is the minimal abstraction: any type that can wrap a value and
	/// dereference to it. Does NOT require Clone - that's added by subtraits.
	///
	/// By explicitly requiring that the type parameter `T` outlives the application lifetime `'a`,
	/// we provide the compiler with the necessary guarantees to handle trait objects
	/// (like `dyn Fn`) commonly used in pointer implementations. This resolves potential
	/// E0310 errors where the compiler cannot otherwise prove that captured variables in
	/// closures satisfy the required lifetime bounds.
	pub trait Pointer {
		/// The pointer type constructor.
		///
		/// For `RcBrand`, this is `Rc<T>`. For `ArcBrand`, this is `Arc<T>`.
		type Of<'a, T: ?Sized + 'a>: Deref<Target = T> + 'a;

		/// Wraps a sized value in the pointer.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("The value wrapped in the pointer type.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let ptr = <RcBrand as Pointer>::new(42);
		/// assert_eq!(*ptr, 42);
		/// ```
		fn new<'a, T: 'a>(value: T) -> Self::Of<'a, T>
		where
			Self::Of<'a, T>: Sized;
	}

	/// Wraps a sized value in the pointer.
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
	#[document_returns("The value wrapped in the pointer type.")]
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
	pub fn new<'a, P: Pointer, T: 'a>(value: T) -> P::Of<'a, T>
	where
		P::Of<'a, T>: Sized, {
		P::new(value)
	}
}

pub use inner::*;
