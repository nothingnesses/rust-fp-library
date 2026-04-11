//! Sequencing of two computations while keeping the result of the first.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(5);
//! let y = Some(10);
//! let z = apply_first_explicit::<OptionBrand, _, _, _, _>(x, y);
//! assert_eq!(z, Some(5));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for types that support combining two contexts, keeping the first value.
	///
	/// `ApplyFirst` provides the ability to sequence two computations but discard
	/// the result of the second computation, keeping only the result of the first.
	pub trait ApplyFirst: Lift {
		/// Combines two contexts, keeping the value from the first context.
		///
		/// This function sequences two computations and discards the result of the second computation, keeping only the result of the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value in the first context.",
			"The type of the value in the second context."
		)]
		///
		#[document_parameters("The first context.", "The second context.")]
		///
		#[document_returns("The first context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = Some(10);
		/// let z = apply_first_explicit::<OptionBrand, _, _, _, _>(x, y);
		/// assert_eq!(z, Some(5));
		/// ```
		fn apply_first<'a, A: 'a + Clone, B: 'a + Clone>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Self::lift2(|a, _| a, fa, fb)
		}
	}

	/// Combines two contexts, keeping the value from the first context.
	///
	/// Free function version that dispatches to [the type class' associated function][`ApplyFirst::apply_first`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the value in the first context.",
		"The type of the value in the second context."
	)]
	///
	#[document_parameters("The first context.", "The second context.")]
	///
	#[document_returns("The first context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = Some(10);
	/// let z = apply_first_explicit::<OptionBrand, _, _, _, _>(x, y);
	/// assert_eq!(z, Some(5));
	/// ```
	pub fn apply_first<'a, Brand: ApplyFirst, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::apply_first(fa, fb)
	}
}

pub use inner::*;
