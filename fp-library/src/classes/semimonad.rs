//! Sequencing of computations where the structure depends on previous results, without an identity element.
//!
//! `Semimonad` is the dual of [`Extend`](crate::classes::Extend): where `extend` feeds an
//! entire context into a function, `bind` extracts a value and feeds it into a function that
//! produces a new context. Similarly, [`join`] is the dual of
//! [`duplicate`](crate::functions::duplicate).
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
//! let y = bind_explicit::<OptionBrand, _, _, _, _>(x, |i| Some(i * 2));
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
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait Semimonad {
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
		/// let y = bind_explicit::<OptionBrand, _, _, _, _>(x, |i| Some(i * 2));
		/// assert_eq!(y, Some(10));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Collapses two nested layers of a semimonad into one.
	///
	/// Equivalent to `bind(mma, identity)`. Removes one level of monadic wrapping
	/// from a doubly-wrapped value.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The brand of the semimonad.",
		"The type of the value inside the nested semimonad."
	)]
	///
	#[document_parameters("The doubly-wrapped semimonadic value.")]
	///
	#[document_returns("The singly-wrapped semimonadic value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(Some(5));
	/// let y = join_explicit::<OptionBrand, _, _>(x);
	/// assert_eq!(y, Some(5));
	///
	/// let z: Option<Option<i32>> = Some(None);
	/// assert_eq!(join_explicit::<OptionBrand, _, _>(z), None);
	/// ```
	pub fn join<'a, Brand: Semimonad, A: 'a>(
		mma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::bind(mma, |ma| ma)
	}
}

pub use inner::*;
