//! Coercion of sized closures into `dyn Fn` trait objects behind a
//! [`Pointer`](crate::classes::Pointer).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! };
//!
//! let f = <BoxBrand as ToDynFn>::new(|x: i32| x + 1);
//! assert_eq!(f(1), 2);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::Pointer,
		fp_macros::*,
	};

	/// Coerces sized closures into `dyn Fn` trait objects behind a
	/// [`Pointer`](crate::classes::Pointer).
	///
	/// This trait extends [`Pointer`](crate::classes::Pointer) to add the
	/// ability to wrap concrete closure types into type-erased `dyn Fn` trait
	/// objects stored in the pointer. For example, `BoxBrand` coerces
	/// `impl Fn(A) -> B` into `Box<dyn Fn(A) -> B>`.
	///
	/// For clonable variants, see
	/// [`UnsizedCoercible`](crate::classes::UnsizedCoercible) (which extends
	/// [`RefCountedPointer`](crate::classes::RefCountedPointer)). For
	/// thread-safe variants, see
	/// [`SendUnsizedCoercible`](crate::classes::SendUnsizedCoercible)
	/// (which extends
	/// [`SendRefCountedPointer`](crate::classes::SendRefCountedPointer)).
	pub trait ToDynFn: Pointer + 'static {
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
		/// 	classes::*,
		/// };
		///
		/// let f = <BoxBrand as ToDynFn>::new(|x: i32| x + 1);
		/// assert_eq!(f(1), 2);
		/// ```
		fn new<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(A) -> B
		) -> <Self as Pointer>::Of<'a, dyn 'a + Fn(A) -> B>;

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
		/// let f = <BoxBrand as ToDynFn>::ref_new(|x: &i32| *x + 1);
		/// assert_eq!(f(&1), 2);
		/// ```
		fn ref_new<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(&A) -> B
		) -> <Self as Pointer>::Of<'a, dyn 'a + Fn(&A) -> B>;
	}

	/// Coerces a sized closure to a `dyn Fn` wrapped in a pointer.
	///
	/// Free function version that dispatches to [`ToDynFn::new`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the closure.",
		"The pointer brand.",
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
	/// 	classes::to_dyn_fn::*,
	/// };
	///
	/// let f = to_dyn_fn::<BoxBrand, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	pub fn to_dyn_fn<'a, Brand: ToDynFn, A: 'a, B: 'a>(
		f: impl 'a + Fn(A) -> B
	) -> <Brand as Pointer>::Of<'a, dyn 'a + Fn(A) -> B> {
		<Brand as ToDynFn>::new(f)
	}

	/// Coerces a sized by-reference closure to a `dyn Fn(&A) -> B` wrapped in a pointer.
	///
	/// Free function version that dispatches to [`ToDynFn::ref_new`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the closure.",
		"The pointer brand.",
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
	/// 	classes::to_dyn_fn::*,
	/// };
	///
	/// let f = to_ref_dyn_fn::<BoxBrand, _, _>(|x: &i32| *x + 1);
	/// assert_eq!(f(&1), 2);
	/// ```
	pub fn to_ref_dyn_fn<'a, Brand: ToDynFn, A: 'a, B: 'a>(
		f: impl 'a + Fn(&A) -> B
	) -> <Brand as Pointer>::Of<'a, dyn 'a + Fn(&A) -> B> {
		<Brand as ToDynFn>::ref_new(f)
	}
}

pub use inner::*;
