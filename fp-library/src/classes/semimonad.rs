//! Sequencing of computations where the structure depends on previous results, without an identity element.
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
//! let y = bind::<OptionBrand, _, _>(x, |i| Some(i * 2));
//! assert_eq!(y, Some(10));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// Sequences two computations, allowing the second to depend on the value computed by the first.
	///
	/// If `x` has type `m a` and `f` has type `a -> m b`, then `bind(x, f)` has type `m b`,
	/// representing the result of executing `x` to get a value of type `a` and then
	/// passing it to `f` to get a computation of type `m b`.
	pub trait Semimonad: Kind_cdc7cd43dac7585f {
		/// Sequences two computations, allowing the second to depend on the value computed by the first.
		///
		/// This method chains two computations, where the second computation depends on the result of the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computations.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first computation.",
			"The function to apply to the result of the first computation."
		)]
		///
		#[document_returns("The result of the second computation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = bind::<OptionBrand, _, _>(x, |i| Some(i * 2));
		/// assert_eq!(y, Some(10));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Sequences two computations, allowing the second to depend on the value computed by the first.
	///
	/// Free function version that dispatches to [the type class' associated function][`Semimonad::bind`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computations.",
		"The brand of the semimonad.",
		"The type of the result of the first computation.",
		"The type of the result of the second computation."
	)]
	///
	#[document_parameters(
		"The first computation.",
		"The function to apply to the result of the first computation."
	)]
	///
	#[document_returns("The result of the second computation.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = bind::<OptionBrand, _, _>(x, |i| Some(i * 2));
	/// assert_eq!(y, Some(10));
	/// ```
	pub fn bind<'a, Brand: Semimonad, A: 'a, B: 'a>(
		ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: impl Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::bind(ma, f)
	}
}

pub use inner::*;
