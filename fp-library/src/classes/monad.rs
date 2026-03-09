//! Monads, allowing for sequencing computations where the structure depends on previous results.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Monad combines Pointed (pure) and Semimonad (bind)
//! let x = pure::<OptionBrand, _>(5);
//! let y = bind::<OptionBrand, _, _, _>(x, |i| pure::<OptionBrand, _>(i * 2));
//! assert_eq!(y, Some(10));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
	};

	/// A type class for monads, allowing for sequencing computations where the structure of the computation depends on the result of the previous computation.
	///
	/// `class (Applicative m, Semimonad m) => Monad m`
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Monad combines Pointed (pure) and Semimonad (bind)
	/// let x = pure::<OptionBrand, _>(5);
	/// let y = bind::<OptionBrand, _, _, _>(x, |i| pure::<OptionBrand, _>(i * 2));
	/// assert_eq!(y, Some(10));
	/// ```
	pub trait Monad: Applicative + Semimonad {}

	/// Blanket implementation of [`Monad`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> Monad for Brand where Brand: Applicative + Semimonad {}
}

pub use inner::*;
