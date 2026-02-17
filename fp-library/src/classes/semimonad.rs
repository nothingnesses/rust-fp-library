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
//! let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
//! assert_eq!(y, Some(10));
//! ```

use {
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// If `x` has type `m a` and `f` has type `a -> m b`, then `bind(x, f)` has type `m b`,
/// representing the result of executing `x` to get a value of type `a` and then
/// passing it to `f` to get a computation of type `m b`.
pub trait Semimonad: Kind_ad6c20556a82a1f0 {
	/// Sequences two computations, allowing the second to depend on the value computed by the first.
	///
	/// This method chains two computations, where the second computation depends on the result of the first.
	#[document_signature]
	///
	#[document_type_parameters(
		"The type of the result of the first computation.",
		"The type of the result of the second computation.",
		"The type of the function to apply."
	)]
	///
	#[document_parameters(
		"The first computation.",
		"The function to apply to the result of the first computation."
	)]
	///
	/// ### Returns
	///
	/// The result of the second computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
	/// assert_eq!(y, Some(10));
	/// ```
	fn bind<A, B, Func>(
		ma: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
	where
		A: 'static,
		B: 'static,
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>) + 'static;
}

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// Free function version that dispatches to [the type class' associated function][`Semimonad::bind`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the semimonad.",
	"The type of the result of the first computation.",
	"The type of the result of the second computation.",
	"The type of the function to apply."
)]
///
#[document_parameters(
	"The first computation.",
	"The function to apply to the result of the first computation."
)]
///
/// ### Returns
///
/// The result of the second computation.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Some(5);
/// let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
/// assert_eq!(y, Some(10));
/// ```
pub fn bind<Brand: Semimonad, A, B, Func>(
	ma: Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>),
	f: Func,
) -> Apply!(<Brand as Kind!( type Of<T>; )>::Of<B>)
where
	A: 'static,
	B: 'static,
	Func: Fn(A) -> Apply!(<Brand as Kind!( type Of<T>; )>::Of<B>) + 'static,
{
	Brand::bind::<A, B, Func>(ma, f)
}
