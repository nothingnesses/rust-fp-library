//! Pointer brands that can perform unsized coercion to `dyn Fn` trait objects.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = coerce_fn::<RcBrand, _, _>(|x: i32| x + 1);
//! assert_eq!(f(1), 2);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// Trait for pointer brands that can perform unsized coercion to `dyn Fn`.
	pub trait UnsizedCoercible: RefCountedPointer + 'static {
		/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
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
		#[document_returns("The closure wrapped in the pointer type as a trait object.")]
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
		fn coerce_fn<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(A) -> B
		) -> Self::CloneableOf<'a, dyn 'a + Fn(A) -> B>;

		/// Coerces a sized by-reference closure to a `dyn Fn(&A) -> B` wrapped in this pointer type.
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
			"The closure wrapped in the pointer type as a by-reference trait object."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let f = RcBrand::coerce_ref_fn(|x: &i32| *x + 1);
		/// assert_eq!(f(&1), 2);
		/// ```
		fn coerce_ref_fn<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(&A) -> B
		) -> Self::CloneableOf<'a, dyn 'a + Fn(&A) -> B>;
	}

	/// Coerces a sized by-reference closure to a `dyn Fn(&A) -> B` wrapped in this pointer type.
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
	#[document_returns("The closure wrapped in the pointer type as a by-reference trait object.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = coerce_ref_fn::<RcBrand, _, _>(|x: &i32| *x + 1);
	/// assert_eq!(f(&1), 2);
	/// ```
	pub fn coerce_ref_fn<'a, Brand: UnsizedCoercible, A: 'a, B: 'a>(
		func: impl 'a + Fn(&A) -> B
	) -> Brand::CloneableOf<'a, dyn 'a + Fn(&A) -> B> {
		Brand::coerce_ref_fn::<A, B>(func)
	}

	/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
	///
	/// Free function version that dispatches to [the type class' associated function][`UnsizedCoercible::coerce_fn`].
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
	#[document_returns("The closure wrapped in the pointer type as a trait object.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::unsized_coercible::*,
	/// 	functions::*,
	/// };
	///
	/// let f = coerce_fn::<RcBrand, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	pub fn coerce_fn<'a, Brand: UnsizedCoercible, A: 'a, B: 'a>(
		func: impl 'a + Fn(A) -> B
	) -> Brand::CloneableOf<'a, dyn 'a + Fn(A) -> B> {
		Brand::coerce_fn::<A, B>(func)
	}
}

pub use inner::*;
