//! Types that can be mapped over two type arguments simultaneously.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Result::<i32, i32>::Ok(5);
//! let y = bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x);
//! assert_eq!(y, Ok(10));
//! ```

use {
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for types that can be mapped over two type arguments.
///
/// A `Bifunctor` represents a context or container with two type parameters,
/// allowing functions to be applied to values of both types.
///
/// ### Laws
///
/// `Bifunctor` instances must satisfy the following laws:
/// * Identity: `bimap(identity, identity, p) = p`.
/// * Composition: `bimap(compose(f, g), compose(h, i), p) = bimap(f, h, bimap(g, i, p))`.
pub trait Bifunctor: Kind_5b1bcedfd80bdc16 {
	/// Maps functions over the values in the bifunctor context.
	///
	/// This method applies two functions to the values inside the bifunctor context, producing a new bifunctor context with the transformed values.
	#[document_signature]
	///
	#[document_type_parameters(
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value.",
		"The type of the second result.",
		"The type of the first function.",
		"The type of the second function."
	)]
	///
	#[document_parameters(
		"The function to apply to the first value.",
		"The function to apply to the second value.",
		"The bifunctor instance."
	)]
	///
	/// ### Returns
	///
	/// A new bifunctor instance containing the results of applying the functions.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Result::<i32, i32>::Ok(5);
	/// let y = bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x);
	/// assert_eq!(y, Ok(10));
	/// ```
	fn bimap<A, B, C, D, F, G>(
		f: F,
		g: G,
		p: Apply!(<Self as Kind!( type Of<A, B>; )>::Of<A, C>),
	) -> Apply!(<Self as Kind!( type Of<A, B>; )>::Of<B, D>)
	where
		F: Fn(A) -> B,
		G: Fn(C) -> D;
}

/// Maps functions over the values in the bifunctor context.
///
/// Free function version that dispatches to [the type class' associated function][`Bifunctor::bimap`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the bifunctor.",
	"The type of the first value.",
	"The type of the first result.",
	"The type of the second value.",
	"The type of the second result.",
	"The type of the first function.",
	"The type of the second function."
)]
///
#[document_parameters(
	"The function to apply to the first value.",
	"The function to apply to the second value.",
	"The bifunctor instance."
)]
///
/// ### Returns
///
/// A new bifunctor instance containing the results of applying the functions.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Result::<i32, i32>::Ok(5);
/// let y = bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x);
/// assert_eq!(y, Ok(10));
/// ```
pub fn bimap<Brand, A, B, C, D, F, G>(
	f: F,
	g: G,
	p: Apply!(<Brand as Kind!( type Of<A, B>; )>::Of<A, C>),
) -> Apply!(<Brand as Kind!( type Of<A, B>; )>::Of<B, D>)
where
	Brand: Bifunctor,
	F: Fn(A) -> B,
	G: Fn(C) -> D,
{
	Brand::bimap(f, g, p)
}

/// Maps a function over the first type argument.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Result::<i32, i32>::Err(5);
/// let y = map_first::<ResultBrand, _, _, _, _, _>(|e| e + 1, x);
/// assert_eq!(y, Err(6));
/// ```
pub fn map_first<Brand, A, B, C, F>(
	f: F,
	p: Apply!(<Brand as Kind!( type Of<A, B>; )>::Of<A, C>),
) -> Apply!(<Brand as Kind!( type Of<A, B>; )>::Of<B, C>)
where
	Brand: Bifunctor,
	F: Fn(A) -> B,
{
	Brand::bimap(f, |c| c, p)
}

/// Maps a function over the second type argument.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Result::<i32, i32>::Ok(5);
/// let y = map_second::<ResultBrand, _, _, _, _, _>(|s| s * 2, x);
/// assert_eq!(y, Ok(10));
/// ```
pub fn map_second<Brand, A, B, C, G>(
	g: G,
	p: Apply!(<Brand as Kind!( type Of<A, B>; )>::Of<A, B>),
) -> Apply!(<Brand as Kind!( type Of<A, B>; )>::Of<A, C>)
where
	Brand: Bifunctor,
	G: Fn(B) -> C,
{
	Brand::bimap(|a| a, g, p)
}
