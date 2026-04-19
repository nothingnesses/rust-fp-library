//! Owned heap-allocated pointer abstraction using [`Box`].
//!
//! Provides trait implementations for using `Box` in the library's pointer abstraction.
//! `BoxBrand` implements [`Pointer`](crate::classes::Pointer) and
//! [`ToDynFn`](crate::classes::ToDynFn) but not
//! [`RefCountedPointer`](crate::classes::RefCountedPointer)
//! (since `Box<dyn Fn>` is not `Clone`).
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
}

#[cfg(test)]
mod tests {
	use crate::{
		brands::BoxBrand,
		classes::{
			Pointer,
			ToDynFn,
			to_dyn_fn::{
				to_dyn_fn,
				to_ref_dyn_fn,
			},
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
}
