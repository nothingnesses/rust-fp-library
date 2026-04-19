//! Thread-safe reference-counted pointers that carry `Send + Sync` bounds.
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
		fp_macros::*,
		std::ops::Deref,
	};

	/// Thread-safe counterpart to
	/// [`RefCountedPointer`](crate::classes::RefCountedPointer).
	///
	/// This is an independent trait (not a supertrait of `RefCountedPointer`),
	/// matching the pattern used by `SendCloneFn` (independent of `CloneFn`).
	/// Both traits have their own associated type with different bounds:
	/// `RefCountedPointer::Of` requires `Clone + Deref`, while
	/// `SendRefCountedPointer::Of` requires `Clone + Send + Sync + Deref`.
	pub trait SendRefCountedPointer {
		/// The thread-safe pointer type constructor.
		///
		/// For `ArcBrand`, this is `Arc<T>` where `T: Send + Sync`.
		type Of<'a, T: ?Sized + Send + Sync + 'a>: Clone + Send + Sync + Deref<Target = T> + 'a;

		/// Wraps a sized value in a thread-safe pointer.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("The value wrapped in the thread-safe pointer type.")]
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
		fn new<'a, T: Send + Sync + 'a>(value: T) -> Self::Of<'a, T>
		where
			Self::Of<'a, T>: Sized;
	}

	/// Wraps a sized value in a thread-safe pointer.
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
	#[document_returns("The value wrapped in the thread-safe pointer type.")]
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
	pub fn new<'a, P: SendRefCountedPointer, T: Send + Sync + 'a>(value: T) -> P::Of<'a, T>
	where
		P::Of<'a, T>: Sized, {
		P::new(value)
	}
}

pub use inner::*;
