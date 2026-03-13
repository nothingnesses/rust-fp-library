//! Types that can be mapped over, allowing functions to be applied to values within a context.
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
//! let y = map::<OptionBrand, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for types that can be mapped over.
	///
	/// A `Functor` represents a context or container that allows functions to be applied
	/// to values within that context without altering the structure of the context itself.
	///
	/// ### Hierarchy Unification
	///
	/// This trait now inherits from [`Kind_cdc7cd43dac7585f`], ensuring that all functor
	/// contexts satisfy the strict lifetime requirements where the type argument must
	/// outlive the context's application lifetime.
	///
	/// By explicitly requiring that the type parameter outlives the application lifetime `'a`,
	/// we provide the compiler with the necessary guarantees to handle trait objects
	/// (like `dyn Fn`) commonly used in functor implementations. This resolves potential
	/// E0310 errors where the compiler cannot otherwise prove that captured variables in
	/// closures satisfy the required lifetime bounds.
	///
	/// ### Laws
	///
	/// `Functor` instances must satisfy the following laws:
	/// * Identity: `map(identity, fa) = fa`.
	/// * Composition: `map(compose(f, g), fa) = map(f, map(g, fa))`.
	#[document_examples]
	///
	/// Functor laws for [`Option`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Identity: map(identity, fa) = fa
	/// assert_eq!(map::<OptionBrand, _, _>(identity, Some(5)), Some(5));
	/// assert_eq!(map::<OptionBrand, _, _>(identity, None::<i32>), None);
	///
	/// // Composition: map(compose(f, g), fa) = map(f, map(g, fa))
	/// let f = |x: i32| x + 1;
	/// let g = |x: i32| x * 2;
	/// assert_eq!(
	/// 	map::<OptionBrand, _, _>(compose(f, g), Some(5)),
	/// 	map::<OptionBrand, _, _>(f, map::<OptionBrand, _, _>(g, Some(5))),
	/// );
	/// ```
	///
	/// Functor laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Identity: map(identity, fa) = fa
	/// assert_eq!(map::<VecBrand, _, _>(identity, vec![1, 2, 3]), vec![1, 2, 3]);
	///
	/// // Composition: map(compose(f, g), fa) = map(f, map(g, fa))
	/// let f = |x: i32| x + 1;
	/// let g = |x: i32| x * 2;
	/// assert_eq!(
	/// 	map::<VecBrand, _, _>(compose(f, g), vec![1, 2, 3]),
	/// 	map::<VecBrand, _, _>(f, map::<VecBrand, _, _>(g, vec![1, 2, 3])),
	/// );
	/// ```
	pub trait Functor: Kind_cdc7cd43dac7585f {
		/// Maps a function over the values in the functor context.
		///
		/// This method applies a function to the value(s) inside the functor context, producing a new functor context with the transformed value(s).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the value(s) inside the functor.",
			"The functor instance containing the value(s)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = map::<OptionBrand, _, _>(|i| i * 2, x);
		/// assert_eq!(y, Some(10));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Maps a function over the values in the functor context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Functor::map`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s) inside the functor.",
		"The functor instance containing the value(s)."
	)]
	///
	#[document_returns("A new functor instance containing the result(s) of applying the function.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = map::<OptionBrand, _, _>(|i| i * 2, x);
	/// assert_eq!(y, Some(10));
	/// ```
	pub fn map<'a, Brand: Functor, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::map(f, fa)
	}
}

pub use inner::*;
