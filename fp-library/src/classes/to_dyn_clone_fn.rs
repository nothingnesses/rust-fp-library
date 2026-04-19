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
//! let f = to_dyn_clone_fn::<RcBrand, _, _>(|x: i32| x + 1);
//! assert_eq!(f(1), 2);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// Trait for pointer brands that can perform unsized coercion to `dyn Fn`.
	pub trait ToDynCloneFn: RefCountedPointer + 'static {
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
		/// let f = to_dyn_clone_fn::<RcBrand, _, _>(|x: i32| x + 1);
		/// assert_eq!(f(1), 2);
		/// ```
		fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, dyn 'a + Fn(A) -> B>;

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
		/// let f = <RcBrand as ToDynCloneFn>::ref_new(|x: &i32| *x + 1);
		/// assert_eq!(f(&1), 2);
		/// ```
		fn ref_new<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(&A) -> B
		) -> Self::Of<'a, dyn 'a + Fn(&A) -> B>;
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
	/// let f = to_ref_dyn_clone_fn::<RcBrand, _, _>(|x: &i32| *x + 1);
	/// assert_eq!(f(&1), 2);
	/// ```
	pub fn ref_new<'a, Brand: ToDynCloneFn, A: 'a, B: 'a>(
		func: impl 'a + Fn(&A) -> B
	) -> Brand::Of<'a, dyn 'a + Fn(&A) -> B> {
		<Brand as ToDynCloneFn>::ref_new::<A, B>(func)
	}

	/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
	///
	/// Free function version that dispatches to [the type class' associated function][`ToDynCloneFn::new`].
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
	/// 	classes::to_dyn_clone_fn::*,
	/// 	functions::*,
	/// };
	///
	/// let f = to_dyn_clone_fn::<RcBrand, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	pub fn new<'a, Brand: ToDynCloneFn, A: 'a, B: 'a>(
		func: impl 'a + Fn(A) -> B
	) -> Brand::Of<'a, dyn 'a + Fn(A) -> B> {
		<Brand as ToDynCloneFn>::new::<A, B>(func)
	}
}

pub use inner::*;
