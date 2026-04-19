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
//! let f = to_dyn_send_fn::<ArcBrand, _, _>(|x: i32| x + 1);
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
	/// This is an independent trait (not a supertrait of `ToDynCloneFn`),
	/// matching the pattern used by `SendCloneFn` (independent of `CloneFn`).
	/// It extends `SendRefCountedPointer` because its methods return
	/// `SendRefCountedPointer::SendOf` types.
	pub trait ToDynSendFn: SendRefCountedPointer + 'static {
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
		/// let f = to_dyn_send_fn::<ArcBrand, _, _>(|x: i32| x + 1);
		/// assert_eq!(f(1), 2);
		/// ```
		fn new<'a, A: 'a, B: 'a>(
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
		/// let f = <ArcBrand as ToDynSendFn>::ref_new(|x: &i32| *x + 1);
		/// assert_eq!(f(&1), 2);
		/// ```
		fn ref_new<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(&A) -> B + Send + Sync
		) -> Self::SendOf<'a, dyn 'a + Fn(&A) -> B + Send + Sync>;
	}

	/// Coerces a sized Send+Sync by-reference closure to a `dyn Fn(&A) -> B + Send + Sync`.
	///
	/// Free function version that dispatches to [`ToDynSendFn::ref_new`].
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
	/// let f = to_ref_dyn_send_fn::<ArcBrand, _, _>(|x: &i32| *x + 1);
	/// assert_eq!(f(&1), 2);
	/// ```
	pub fn ref_new<'a, Brand: ToDynSendFn, A: 'a, B: 'a>(
		func: impl 'a + Fn(&A) -> B + Send + Sync
	) -> Brand::SendOf<'a, dyn 'a + Fn(&A) -> B + Send + Sync> {
		<Brand as ToDynSendFn>::ref_new::<A, B>(func)
	}

	/// Coerces a sized Send+Sync closure to a `dyn Fn + Send + Sync`.
	///
	/// Free function version that dispatches to [the type class' associated function][`ToDynSendFn::new`].
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
	/// 	classes::to_dyn_send_fn::*,
	/// 	functions::*,
	/// };
	///
	/// let f = to_dyn_send_fn::<ArcBrand, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	pub fn new<'a, Brand: ToDynSendFn, A: 'a, B: 'a>(
		func: impl 'a + Fn(A) -> B + Send + Sync
	) -> Brand::SendOf<'a, dyn 'a + Fn(A) -> B + Send + Sync> {
		<Brand as ToDynSendFn>::new::<A, B>(func)
	}
}

pub use inner::*;
