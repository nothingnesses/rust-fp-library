//! The identity element for [`Alt`](crate::classes::Alt), forming a monoid on type constructors.
//!
//! `Plus` is to type constructors as [`Monoid`](crate::classes::Monoid) is to concrete types.
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
//! let x = alt::<OptionBrand, _>(plus_empty::<OptionBrand, i32>(), Some(5));
//! assert_eq!(x, Some(5));
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

	/// A type class extending [`Alt`] with an identity element.
	///
	/// `Plus` is similar to [`Monoid`], except that it applies to types of
	/// kind `* -> *` (like `Option` or `Vec`) rather than concrete types.
	///
	/// ### Laws
	///
	/// `Plus` instances must satisfy the following laws:
	/// * Left identity: `alt(empty, x) = x`.
	/// * Right identity: `alt(x, empty) = x`.
	/// * Annihilation: `map(f, empty) = empty`.
	#[document_examples]
	///
	/// Plus laws for [`Option`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// // Left identity: alt(empty, x) = x
	/// let x = Some(5);
	/// assert_eq!(alt::<OptionBrand, _>(plus_empty::<OptionBrand, i32>(), x), x,);
	///
	/// // Right identity: alt(x, empty) = x
	/// assert_eq!(alt::<OptionBrand, _>(x, plus_empty::<OptionBrand, i32>()), x,);
	///
	/// // Annihilation: map(f, empty) = empty
	/// let f = |i: i32| i * 2;
	/// assert_eq!(
	/// 	map_explicit::<OptionBrand, _, _, _, _>(f, plus_empty::<OptionBrand, i32>()),
	/// 	plus_empty::<OptionBrand, i32>(),
	/// );
	/// ```
	pub trait Plus: Alt {
		/// Returns the identity element for [`alt`](Alt::alt).
		///
		/// This is the empty/failure value for the type constructor.
		/// For `Option`, this is `None`. For `Vec`, this is `vec![]`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the context."
		)]
		///
		#[document_returns("The identity element.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x: Option<i32> = plus_empty::<OptionBrand, i32>();
		/// assert_eq!(x, None);
		/// ```
		fn empty<'a, A: 'a>() -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	/// Returns the identity element for [`alt`](crate::classes::Alt::alt).
	///
	/// Free function version that dispatches to [the type class' associated function][`Plus::empty`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the value.",
		"The brand of the context.",
		"The type of the value inside the context."
	)]
	///
	#[document_returns("The identity element.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Option<i32> = plus_empty::<OptionBrand, i32>();
	/// assert_eq!(x, None);
	/// ```
	pub fn empty<'a, Brand: Plus, A: 'a>()
	-> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::empty()
	}
}

pub use inner::*;
