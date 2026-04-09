//! Lifting of functions to operate on values within a context.
//!
//! Provides [`lift2`](crate::functions::lift2) through [`lift5`](crate::functions::lift5) for lifting multi-argument functions
//! into a context. Higher-arity lifts are built from [`lift2`](crate::functions::lift2) using tuple
//! intermediaries.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(1);
//! let y = Some(2);
//! let z = lift2::<OptionBrand, _, _, _, _, _, _>(|a, b| a + b, x, y);
//! assert_eq!(z, Some(3));
//!
//! let w = lift3::<OptionBrand, _, _, _, _, _, _, _, _>(
//! 	|a, b, c| a + b + c,
//! 	Some(1),
//! 	Some(2),
//! 	Some(3),
//! );
//! assert_eq!(w, Some(6));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for lifting binary functions into a context.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait Lift {
		/// Lifts a binary function into the context.
		///
		/// This method lifts a binary function to operate on values within the context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first context.",
			"The second context."
		)]
		///
		#[document_returns("A new context containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(1);
		/// let y = Some(2);
		/// let z = lift2::<OptionBrand, _, _, _, _, _, _>(|a, b| a + b, x, y);
		/// assert_eq!(z, Some(3));
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a;
	}
}

pub use inner::*;
