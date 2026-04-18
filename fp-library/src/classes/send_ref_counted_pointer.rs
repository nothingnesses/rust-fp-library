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
		crate::classes::*,
		fp_macros::*,
		std::ops::Deref,
	};

	/// Thread-safe counterpart to [`RefCountedPointer`].
	///
	/// Unlike `SendCloneFn` (which is independent of `CloneFn`), this trait
	/// is a supertrait of `RefCountedPointer`, adding a `SendOf` associated
	/// type with explicit `Send + Sync` bounds.
	pub trait SendRefCountedPointer: RefCountedPointer {
		/// The thread-safe pointer type constructor.
		///
		/// For `ArcBrand`, this is `Arc<T>` where `T: Send + Sync`.
		type SendOf<'a, T: ?Sized + Send + Sync + 'a>: Clone + Send + Sync + Deref<Target = T> + 'a;

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
		fn send_new<'a, T: Send + Sync + 'a>(value: T) -> Self::SendOf<'a, T>
		where
			Self::SendOf<'a, T>: Sized;
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
	pub fn send_new<'a, P: SendRefCountedPointer, T: Send + Sync + 'a>(
		value: T
	) -> P::SendOf<'a, T>
	where
		P::SendOf<'a, T>: Sized, {
		P::send_new(value)
	}
}

pub use inner::*;
