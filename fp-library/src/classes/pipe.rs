//! Left-to-right function application via method syntax.
//!
//! Provides the [`Pipe`] trait for pipeline-style composition, similar to
//! PureScript's `#` operator or Haskell's `&` operator.
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
//! let result = Some(5).pipe(|x| map::<OptionBrand, _, _>(|n| n + 1, x));
//!
//! assert_eq!(result, Some(6));
//! ```

#[fp_macros::document_module]
mod inner {
	use fp_macros::*;

	/// A trait for left-to-right function application via method syntax.
	///
	/// `Pipe` provides the `.pipe()` method on all sized types via a blanket
	/// implementation, enabling pipeline-style composition similar to
	/// PureScript's `#` operator or Haskell's `&` operator.
	///
	/// This is particularly useful for composing operations on types where
	/// inherent methods are not available (e.g., stdlib types like `Option`
	/// and `Vec`).
	#[document_parameters("The value to pipe.")]
	pub trait Pipe: Sized {
		/// Pipes `self` into a function, enabling left-to-right composition.
		///
		/// Applies `f` to `self` and returns the result. This is the method
		/// syntax version of [`pipe`].
		#[document_signature]
		///
		#[document_type_parameters("The return type of the function.")]
		///
		#[document_parameters("The function to apply to the value.")]
		///
		#[document_returns("The result of applying `f` to `self`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let result = Some(5)
		/// 	.pipe(|x| map::<OptionBrand, _, _>(|n| n + 1, x))
		/// 	.pipe(|x| bind::<OptionBrand, _, _>(x, |n| if n > 3 { Some(n) } else { None }));
		///
		/// assert_eq!(result, Some(6));
		/// ```
		fn pipe<B>(
			self,
			f: impl FnOnce(Self) -> B,
		) -> B {
			f(self)
		}
	}

	#[document_type_parameters("The type that implements Pipe.")]
	impl<T> Pipe for T {}

	/// Pipes a value into a function, enabling left-to-right composition.
	///
	/// Free function version of [`Pipe::pipe`]. Applies `f` to `a` and
	/// returns the result. This is equivalent to PureScript's `applyFlipped`
	/// or Haskell's `(&)`.
	#[document_signature]
	///
	#[document_type_parameters("The type of the input value.", "The return type of the function.")]
	///
	#[document_parameters("The value to pipe.", "The function to apply to the value.")]
	///
	#[document_returns("The result of applying `f` to `a`.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let result = pipe(5, |x| x + 1);
	/// assert_eq!(result, 6);
	/// ```
	pub fn pipe<A, B>(
		a: A,
		f: impl FnOnce(A) -> B,
	) -> B {
		f(a)
	}
}

pub use inner::*;
