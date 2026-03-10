//! Lifting of binary functions to operate on values within a context.
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
//! let z = lift2::<OptionBrand, _, _, _>(|a, b| a + b, x, y);
//! assert_eq!(z, Some(3));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for lifting binary functions into a context.
	pub trait Lift: Kind_cdc7cd43dac7585f {
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
		/// let z = lift2::<OptionBrand, _, _, _>(|a, b| a + b, x, y);
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

	/// Lifts a binary function into the context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Lift::lift2`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
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
	/// let z = lift2::<OptionBrand, _, _, _>(|a, b| a + b, x, y);
	/// assert_eq!(z, Some(3));
	/// ```
	pub fn lift2<'a, Brand: Lift, A, B, C>(
		func: impl Fn(A, B) -> C + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a, {
		Brand::lift2(func, fa, fb)
	}
}

pub use inner::*;
