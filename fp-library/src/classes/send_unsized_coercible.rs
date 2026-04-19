//! Pointer brands that can perform unsized coercion to thread-safe `dyn Fn` trait objects.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = coerce_send_fn::<ArcBrand, _, _>(|x: i32| x + 1);
//! assert_eq!(f(1), 2);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// Trait for pointer brands that can coerce to thread-safe `dyn Fn + Send + Sync`.
	///
	/// This is an independent trait (not a supertrait of `UnsizedCoercible`),
	/// matching the pattern used by `SendCloneFn` (independent of `CloneFn`).
	/// It extends `SendRefCountedPointer` because its methods return
	/// `SendRefCountedPointer::SendOf` types.
	pub trait SendUnsizedCoercible: SendRefCountedPointer + 'static {
		/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
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
		#[document_returns(
			"The closure wrapped in the pointer type as a thread-safe trait object."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = coerce_send_fn::<ArcBrand, _, _>(|x: i32| x + 1);
		/// assert_eq!(f(1), 2);
		/// ```
		fn coerce_send_fn<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(A) -> B + Send + Sync
		) -> Self::SendOf<'a, dyn 'a + Fn(A) -> B + Send + Sync>;

		/// Coerces a sized Send+Sync by-reference closure to a `dyn Fn(&A) -> B + Send + Sync`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the closure.",
			"The input type (the closure receives `&A`).",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to coerce.")]
		///
		#[document_returns(
			"The closure wrapped in the pointer type as a thread-safe by-reference trait object."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let f = ArcBrand::coerce_send_ref_fn(|x: &i32| *x + 1);
		/// assert_eq!(f(&1), 2);
		/// ```
		fn coerce_send_ref_fn<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(&A) -> B + Send + Sync
		) -> Self::SendOf<'a, dyn 'a + Fn(&A) -> B + Send + Sync>;
	}

	/// Coerces a sized Send+Sync by-reference closure to a `dyn Fn(&A) -> B + Send + Sync`.
	///
	/// Free function version that dispatches to [`SendUnsizedCoercible::coerce_send_ref_fn`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the closure.",
		"The brand of the pointer.",
		"The input type (the closure receives `&A`).",
		"The output type of the function."
	)]
	///
	#[document_parameters("The closure to coerce.")]
	///
	#[document_returns(
		"The closure wrapped in the pointer type as a thread-safe by-reference trait object."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = coerce_send_ref_fn::<ArcBrand, _, _>(|x: &i32| *x + 1);
	/// assert_eq!(f(&1), 2);
	/// ```
	pub fn coerce_send_ref_fn<'a, Brand: SendUnsizedCoercible, A: 'a, B: 'a>(
		func: impl 'a + Fn(&A) -> B + Send + Sync
	) -> Brand::SendOf<'a, dyn 'a + Fn(&A) -> B + Send + Sync> {
		Brand::coerce_send_ref_fn::<A, B>(func)
	}

	/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendUnsizedCoercible::coerce_send_fn`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the closure.",
		"The brand of the pointer.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	#[document_parameters("The closure to coerce.")]
	///
	#[document_returns("The closure wrapped in the pointer type as a thread-safe trait object.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::send_unsized_coercible::*,
	/// 	functions::*,
	/// };
	///
	/// let f = coerce_send_fn::<ArcBrand, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	pub fn coerce_send_fn<'a, Brand: SendUnsizedCoercible, A: 'a, B: 'a>(
		func: impl 'a + Fn(A) -> B + Send + Sync
	) -> Brand::SendOf<'a, dyn 'a + Fn(A) -> B + Send + Sync> {
		Brand::coerce_send_fn::<A, B>(func)
	}
}

pub use inner::*;
