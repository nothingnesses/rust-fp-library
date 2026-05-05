//! Coercion of sized closures into `dyn FnOnce` trait objects behind a
//! [`Pointer`](crate::classes::Pointer).
//!
//! Unlike [`ToDynFn`](crate::classes::ToDynFn),
//! [`ToDynCloneFn`](crate::classes::ToDynCloneFn), and
//! [`ToDynSendFn`](crate::classes::ToDynSendFn) , which produce multi-shot
//! `dyn Fn` closures, this trait produces single-shot `dyn FnOnce` closures.
//! Implementable only by [`BoxBrand`](crate::brands::BoxBrand);
//! `Rc<dyn FnOnce>` and `Arc<dyn FnOnce>` are operationally broken because
//! [`FnOnce::call_once`] consumes `self` (the trait object), which cannot
//! be moved out of a shared pointer without invalidating other clones.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! };
//!
//! let f = <BoxBrand as ToDynFnOnce>::new(|x: i32| x + 1);
//! assert_eq!(f(1), 2);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::Pointer,
		fp_macros::*,
	};

	/// Coerces sized closures into `dyn FnOnce` trait objects behind a
	/// [`Pointer`](crate::classes::Pointer).
	///
	/// This trait extends [`Pointer`](crate::classes::Pointer) to add the
	/// ability to wrap concrete closure types into type-erased
	/// `dyn FnOnce` trait objects stored in the pointer. For example,
	/// `BoxBrand` coerces `impl FnOnce(A) -> B` into
	/// `Box<dyn FnOnce(A) -> B>`.
	///
	/// Single-shot semantics by construction: the returned closure can be
	/// called exactly once. For multi-shot closures, see
	/// [`ToDynFn`](crate::classes::ToDynFn). For clonable variants, see
	/// [`ToDynCloneFn`](crate::classes::ToDynCloneFn). For thread-safe
	/// variants, see [`ToDynSendFn`](crate::classes::ToDynSendFn).
	///
	/// Only [`BoxBrand`](crate::brands::BoxBrand) implements this trait.
	/// `Rc<dyn FnOnce>` and `Arc<dyn FnOnce>` cannot implement it because
	/// [`FnOnce::call_once`] consumes `self` (the trait object), which
	/// cannot be moved out of a shared pointer without invalidating other
	/// clones.
	pub trait ToDynFnOnce: Pointer + 'static {
		/// Coerces a sized closure to a `dyn FnOnce` wrapped in this pointer type.
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
			"The closure wrapped in the pointer type as a single-shot trait object."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let f = <BoxBrand as ToDynFnOnce>::new(|x: i32| x + 1);
		/// assert_eq!(f(1), 2);
		/// ```
		fn new<'a, A: 'a, B: 'a>(
			f: impl 'a + FnOnce(A) -> B
		) -> <Self as Pointer>::Of<'a, dyn 'a + FnOnce(A) -> B>;
	}

	/// Coerces a sized closure to a `dyn FnOnce` wrapped in a pointer.
	///
	/// Free function version that dispatches to [`ToDynFnOnce::new`].
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
	#[document_returns("The closure wrapped in the pointer type as a single-shot trait object.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::to_dyn_fn_once::*,
	/// };
	///
	/// let f = to_dyn_fn_once::<BoxBrand, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	pub fn to_dyn_fn_once<'a, Brand: ToDynFnOnce, A: 'a, B: 'a>(
		f: impl 'a + FnOnce(A) -> B
	) -> <Brand as Pointer>::Of<'a, dyn 'a + FnOnce(A) -> B> {
		<Brand as ToDynFnOnce>::new(f)
	}
}

pub use inner::*;
