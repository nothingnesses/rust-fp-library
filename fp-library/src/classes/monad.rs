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
//! let y = bind::<OptionBrand, _, _>(x, |i| pure::<OptionBrand, _>(i * 2));
//! assert_eq!(y, Some(10));
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
	/// let y = bind::<OptionBrand, _, _>(x, |i| pure::<OptionBrand, _>(i * 2));
	/// assert_eq!(y, Some(10));
	/// ```
	pub trait Monad: Applicative + Semimonad {}

	/// Blanket implementation of [`Monad`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> Monad for Brand where Brand: Applicative + Semimonad {}

	/// Executes a monadic action conditionally.
	///
	/// Evaluates the monadic boolean condition, then returns one of the two branches
	/// depending on the result. Both branches are provided as monadic values.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computations.",
		"The brand of the monad.",
		"The type of the value produced by each branch."
	)]
	///
	#[document_parameters(
		"A monadic computation that produces a boolean.",
		"The computation to execute if the condition is `true`.",
		"The computation to execute if the condition is `false`."
	)]
	///
	#[document_returns("The result of the selected branch.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result = if_m::<OptionBrand, _>(Some(true), Some(1), Some(0));
	/// assert_eq!(result, Some(1));
	///
	/// let result = if_m::<OptionBrand, _>(Some(false), Some(1), Some(0));
	/// assert_eq!(result, Some(0));
	///
	/// let result = if_m::<OptionBrand, i32>(None, Some(1), Some(0));
	/// assert_eq!(result, None);
	/// ```
	pub fn if_m<'a, Brand: Monad, A: 'a>(
		cond: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>),
		then_branch: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		else_branch: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
		Brand::bind(cond, move |c| if c { then_branch.clone() } else { else_branch.clone() })
	}
}

pub use inner::*;
