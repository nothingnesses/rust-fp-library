//! Choosing between values in a context, associatively.
//!
//! `Alt` is to type constructors as [`Semigroup`](crate::classes::Semigroup) is to concrete types.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::explicit::*,
//! };
//!
//! let x: Option<i32> = None;
//! let y = Some(5);
//! let z = alt::<OptionBrand, _, _, _>(x, y);
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

	/// A type class for associative choice on type constructors.
	///
	/// `Alt` is similar to [`Semigroup`], except that it applies to types of
	/// kind `* -> *` (like `Option` or `Vec`) rather than concrete types
	/// (like `String` or `i32`).
	///
	/// A common use case is to select the first "valid" item, or, if all items
	/// are "invalid", fall back to the last item.
	///
	/// ### Laws
	///
	/// `Alt` instances must satisfy the following laws:
	/// * Associativity: `alt(alt(x, y), z) = alt(x, alt(y, z))`.
	/// * Distributivity: `map(f, alt(x, y)) = alt(map(f, x), map(f, y))`.
	#[document_examples]
	///
	/// Alt laws for [`Option`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::explicit::*,
	/// };
	///
	/// // Associativity: alt(alt(x, y), z) = alt(x, alt(y, z))
	/// let x: Option<i32> = None;
	/// let y = Some(1);
	/// let z = Some(2);
	/// assert_eq!(
	/// 	alt::<OptionBrand, _, _, _>(alt::<OptionBrand, _, _, _>(x, y), z),
	/// 	alt::<OptionBrand, _, _, _>(x, alt::<OptionBrand, _, _, _>(y, z)),
	/// );
	///
	/// // Distributivity: map(f, alt(x, y)) = alt(map(f, x), map(f, y))
	/// let f = |i: i32| i * 2;
	/// let x = Some(3);
	/// let y: Option<i32> = None;
	/// assert_eq!(
	/// 	map::<OptionBrand, _, _, _, _>(f, alt::<OptionBrand, _, _, _>(x, y)),
	/// 	alt::<OptionBrand, _, _, _>(
	/// 		map::<OptionBrand, _, _, _, _>(f, x),
	/// 		map::<OptionBrand, _, _, _, _>(f, y)
	/// 	),
	/// );
	/// ```
	pub trait Alt: Functor {
		/// Chooses between two values in a context.
		///
		/// This method provides an associative binary operation on type constructors.
		/// For `Option`, this returns the first `Some` value. For `Vec`, this
		/// concatenates the two vectors.
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let x: Option<i32> = None;
		/// let y = Some(5);
		/// let z = alt::<OptionBrand, _, _, _>(x, y);
		/// assert_eq!(z, Some(5));
		/// ```
		fn alt<'a, A: 'a>(
			fa1: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fa2: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	/// Chooses between two values in a context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Alt::alt`].
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
	/// 	functions::explicit::*,
	/// };
	///
	/// let x: Option<i32> = None;
	/// let y = Some(5);
	/// let z = alt::<OptionBrand, _, _, _>(x, y);
	/// assert_eq!(z, Some(5));
	/// ```
	pub fn alt<'a, Brand: Alt, A: 'a>(
		fa1: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fa2: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::alt(fa1, fa2)
	}
}

pub use inner::*;
