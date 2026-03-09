//! Applicative functors, allowing for values and functions to be wrapped and applied within a context.
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
//! // Applicative combines Pointed (pure) and Semiapplicative (apply)
//! let f = pure::<OptionBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
//! let x = pure::<OptionBrand, _>(5);
//! let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
//! assert_eq!(y, Some(10));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A type class for applicative functors, allowing for values to be wrapped in a context and for functions within a context to be applied to values within a context.
	///
	/// `class (Pointed f, Semiapplicative f) => Applicative f`
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// // Applicative combines Pointed (pure) and Semiapplicative (apply)
	/// let f = pure::<OptionBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let x = pure::<OptionBrand, _>(5);
	/// let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
	/// assert_eq!(y, Some(10));
	/// ```
	pub trait Applicative: Pointed + Semiapplicative + ApplyFirst + ApplySecond {}

	/// Blanket implementation of [`Applicative`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> Applicative for Brand where Brand: Pointed + Semiapplicative + ApplyFirst + ApplySecond {}
}

pub use inner::*;
