//! Owned heap-allocated pointer abstraction using [`Box`].
//!
//! Provides trait implementations for using `Box` in the library's pointer abstraction.
//! `BoxBrand` implements [`Pointer`](crate::classes::Pointer),
//! [`ToDynFn`](crate::classes::ToDynFn), and
//! [`ToDynFnOnce`](crate::classes::ToDynFnOnce) but not
//! [`RefCountedPointer`](crate::classes::RefCountedPointer)
//! (since `Box<dyn Fn>` is not `Clone`).
//!
//! `BoxBrand` is the only brand that implements
//! [`ToDynFnOnce`](crate::classes::ToDynFnOnce); `Rc<dyn FnOnce>` and
//! `Arc<dyn FnOnce>` cannot, because [`FnOnce::call_once`] consumes
//! `self` (the trait object), which cannot be moved out of a shared
//! pointer without invalidating other clones.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! };
//!
//! let ptr = <BoxBrand as Pointer>::new(42);
//! assert_eq!(*ptr, 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::BoxBrand,
			classes::{
				Pointer,
				ToDynFn,
				ToDynFnOnce,
			},
		},
		fp_macros::*,
	};

	impl Pointer for BoxBrand {
		type Of<'a, T: ?Sized + 'a> = Box<T>;

		/// Wraps a sized value in a `Box`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("The value wrapped in a `Box`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let ptr = <BoxBrand as Pointer>::new(42);
		/// assert_eq!(*ptr, 42);
		/// ```
		fn new<'a, T: 'a>(value: T) -> Box<T> {
			Box::new(value)
		}
	}

	impl ToDynFn for BoxBrand {
		/// Coerces a sized closure to a `dyn Fn` wrapped in a `Box`.
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
		#[document_returns("The closure wrapped in a `Box` as a trait object.")]
		///
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
		fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> Box<dyn 'a + Fn(A) -> B> {
			Box::new(f)
		}

		/// Coerces a sized by-reference closure to a `dyn Fn(&A) -> B` wrapped in a `Box`.
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
		#[document_returns("The closure wrapped in a `Box` as a by-reference trait object.")]
		///
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
		fn ref_new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(&A) -> B) -> Box<dyn 'a + Fn(&A) -> B> {
			Box::new(f)
		}
	}

	impl ToDynFnOnce for BoxBrand {
		/// Coerces a sized closure to a `dyn FnOnce` wrapped in a `Box`.
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
		#[document_returns("The closure wrapped in a `Box` as a single-shot trait object.")]
		///
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
		fn new<'a, A: 'a, B: 'a>(f: impl 'a + FnOnce(A) -> B) -> Box<dyn 'a + FnOnce(A) -> B> {
			Box::new(f)
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		brands::BoxBrand,
		classes::{
			Pointer,
			ToDynFn,
			ToDynFnOnce,
			to_dyn_fn::{
				to_dyn_fn,
				to_ref_dyn_fn,
			},
			to_dyn_fn_once::to_dyn_fn_once,
		},
	};

	#[test]
	fn test_box_new() {
		let ptr = <BoxBrand as Pointer>::new(42);
		assert_eq!(*ptr, 42);
	}

	#[test]
	fn test_box_to_dyn_fn() {
		let f = <BoxBrand as ToDynFn>::new(|x: i32| x + 1);
		assert_eq!(f(1), 2);
	}

	#[test]
	fn test_box_to_dyn_fn_ref() {
		let f = <BoxBrand as ToDynFn>::ref_new(|x: &i32| *x + 1);
		assert_eq!(f(&1), 2);
	}

	#[test]
	fn test_box_to_dyn_fn_free_fn() {
		let f = to_dyn_fn::<BoxBrand, _, _>(|x: i32| x + 1);
		assert_eq!(f(1), 2);
	}

	#[test]
	fn test_box_to_ref_dyn_fn_free_fn() {
		let f = to_ref_dyn_fn::<BoxBrand, _, _>(|x: &i32| *x + 1);
		assert_eq!(f(&1), 2);
	}

	#[test]
	fn test_box_not_clone() {
		// Box<dyn Fn> is not Clone, confirming BoxBrand cannot implement RefCountedPointer.
		let f = <BoxBrand as ToDynFn>::new(|x: i32| x + 1);
		assert_eq!(f(1), 2);
		// f.clone() would not compile
	}

	#[test]
	fn test_box_to_dyn_fn_once() {
		let f = <BoxBrand as ToDynFnOnce>::new(|x: i32| x + 1);
		assert_eq!(f(1), 2);
	}

	#[test]
	fn test_box_to_dyn_fn_once_free_fn() {
		let f = to_dyn_fn_once::<BoxBrand, _, _>(|x: i32| x + 1);
		assert_eq!(f(1), 2);
	}

	#[test]
	fn test_box_to_dyn_fn_once_consumes_capture() {
		// Capture a non-Clone owned value; FnOnce semantics let us move it
		// out of the closure on the single call.
		let s = String::from("hello");
		let f = <BoxBrand as ToDynFnOnce>::new(move |suffix: &str| s + suffix);
		assert_eq!(f(" world"), "hello world");
	}
}
