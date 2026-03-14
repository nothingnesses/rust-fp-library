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

	/// Sequences two computations with flipped arguments, allowing the function to be provided first.
	///
	/// This is [`bind`] with its arguments reversed. Useful for pipelines where the function
	/// is known before the computation it should be applied to.
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
		"The function to apply to the result of the computation.",
		"The computation."
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
	/// let f = |i: i32| Some(i * 2);
	/// let x = Some(5);
	/// let y = bind_flipped::<OptionBrand, _, _>(f, x);
	/// assert_eq!(y, Some(10));
	/// ```
	pub fn bind_flipped<'a, Brand: Semimonad, A: 'a, B: 'a>(
		f: impl Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::bind(ma, f)
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
	/// let y = join::<OptionBrand, _>(x);
	/// assert_eq!(y, Some(5));
	///
	/// let z: Option<Option<i32>> = Some(None);
	/// assert_eq!(join::<OptionBrand, _>(z), None);
	/// ```
	pub fn join<'a, Brand: Semimonad, A: 'a>(
		mma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::bind(mma, |ma| ma)
	}

	/// Forwards Kleisli composition.
	///
	/// Composes two monadic functions left-to-right: first applies `f`, then passes the
	/// result to `g` via [`bind`]. Equivalent to Haskell's `>=>` operator.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computations.",
		"The brand of the semimonad.",
		"The input type of the first function.",
		"The output type of the first function and input type of the second.",
		"The output type of the second function."
	)]
	///
	#[document_parameters(
		"The first monadic function.",
		"The second monadic function.",
		"The input value."
	)]
	///
	#[document_returns(
		"The result of composing both monadic functions and applying them to the input."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let parse = |s: &str| s.parse::<i32>().ok();
	/// let double = |n: i32| Some(n * 2);
	/// let parse_and_double = |s| compose_kleisli::<OptionBrand, _, _, _>(parse, double, s);
	///
	/// assert_eq!(parse_and_double("5"), Some(10));
	/// assert_eq!(parse_and_double("abc"), None);
	/// ```
	pub fn compose_kleisli<'a, Brand: Semimonad, A: 'a, B: 'a, C: 'a>(
		f: impl Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		g: impl Fn(B) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		a: A,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
		Brand::bind(f(a), g)
	}

	/// Backwards Kleisli composition.
	///
	/// Composes two monadic functions right-to-left: first applies `g`, then passes the
	/// result to `f` via [`bind`]. Equivalent to Haskell's `<=<` operator.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computations.",
		"The brand of the semimonad.",
		"The input type of the second function.",
		"The output type of the second function and input type of the first.",
		"The output type of the first function."
	)]
	///
	#[document_parameters(
		"The second monadic function (applied after `g`).",
		"The first monadic function (applied first to the input).",
		"The input value."
	)]
	///
	#[document_returns(
		"The result of composing both monadic functions and applying them to the input."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let parse = |s: &str| s.parse::<i32>().ok();
	/// let double = |n: i32| Some(n * 2);
	/// let double_then_parse = |s| compose_kleisli_flipped::<OptionBrand, _, _, _>(double, parse, s);
	///
	/// assert_eq!(double_then_parse("5"), Some(10));
	/// assert_eq!(double_then_parse("abc"), None);
	/// ```
	pub fn compose_kleisli_flipped<'a, Brand: Semimonad, A: 'a, B: 'a, C: 'a>(
		f: impl Fn(B) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		g: impl Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		a: A,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
		Brand::bind(g(a), f)
	}
}

pub use inner::*;
