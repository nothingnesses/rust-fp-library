//! Choosing between values in a context by reference, associatively.
//!
//! `RefAlt` is to type constructors with by-reference access as
//! [`Alt`](crate::classes::Alt) is to type constructors with by-value access.
//! Both containers are borrowed, and `A: Clone` is required so elements can
//! be cloned out of the references when constructing the result.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! };
//!
//! let x: Option<i32> = None;
//! let y = Some(5);
//! let z = ref_alt::<OptionBrand, _>(&x, &y);
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

	/// A type class for associative choice on type constructors, operating by reference.
	///
	/// `RefAlt` is the by-reference counterpart of [`Alt`]. Where `Alt` consumes
	/// its arguments, `RefAlt` borrows both containers and clones elements as
	/// needed to produce the result. This is useful for memoized or shared types
	/// that only expose `&A` access.
	///
	/// ### Laws
	///
	/// `RefAlt` instances must satisfy the following law:
	/// * Associativity: `ref_alt(&ref_alt(&x, &y), &z) = ref_alt(&x, &ref_alt(&y, &z))`.
	#[document_examples]
	///
	/// RefAlt associativity for [`Option`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// // Associativity: ref_alt(&ref_alt(&x, &y), &z) = ref_alt(&x, &ref_alt(&y, &z))
	/// let x: Option<i32> = None;
	/// let y = Some(1);
	/// let z = Some(2);
	/// assert_eq!(
	/// 	ref_alt::<OptionBrand, _>(&ref_alt::<OptionBrand, _>(&x, &y), &z),
	/// 	ref_alt::<OptionBrand, _>(&x, &ref_alt::<OptionBrand, _>(&y, &z)),
	/// );
	/// ```
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefAlt: RefFunctor {
		/// Chooses between two values in a context, operating by reference.
		///
		/// Both containers are borrowed. Elements are cloned as needed to
		/// construct the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value inside the context."
		)]
		///
		#[document_parameters("The first value.", "The second value.")]
		///
		#[document_returns("The chosen/combined value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Option<i32> = None;
		/// let y = Some(5);
		/// let z = ref_alt::<OptionBrand, _>(&x, &y);
		/// assert_eq!(z, Some(5));
		/// ```
		fn ref_alt<'a, A: 'a + Clone>(
			fa1: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fa2: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	/// Chooses between two values in a context, operating by reference.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefAlt::ref_alt`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the value inside the context."
	)]
	///
	#[document_parameters("The first value.", "The second value.")]
	///
	#[document_returns("The chosen/combined value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Option<i32> = None;
	/// let y = Some(5);
	/// let z = ref_alt::<OptionBrand, _>(&x, &y);
	/// assert_eq!(z, Some(5));
	/// ```
	pub fn ref_alt<'a, Brand: RefAlt, A: 'a + Clone>(
		fa1: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fa2: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::ref_alt(fa1, fa2)
	}
}

pub use inner::*;
