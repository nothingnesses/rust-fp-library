//! A type class for types that can be mapped over two type arguments.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Result::<i32, i32>::Ok(5);
//! let y = bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x);
//! assert_eq!(y, Ok(10));
//! ```

use fp_macros::doc_type_params;
use crate::{Apply, kinds::*};

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
pub trait Bifunctor: Kind_266801a817966495 {
	/// Maps functions over the values in the bifunctor context.
	///
	/// ### Type Signature
	///
	/// `forall p a b c d. Bifunctor p => (a -> b, c -> d, p a c) -> p b d`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value.",
		"The type of the second result.",
		"The type of the first function.",
		"The type of the second function."
	)]	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the first value.
	/// * `g`: The function to apply to the second value.
	/// * `p`: The bifunctor instance.
	///
	/// ### Returns
	///
	/// A new bifunctor instance containing the results of applying the functions.
	fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, F, G>(
		f: F,
		g: G,
		p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
	) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
	where
		F: Fn(A) -> B + 'a,
		G: Fn(C) -> D + 'a;
}

/// Maps functions over the values in the bifunctor context.
///
/// Free function version that dispatches to [the type class' associated function][`Bifunctor::bimap`].
pub fn bimap<'a, Brand: Bifunctor, A: 'a, B: 'a, C: 'a, D: 'a, F, G>(
	f: F,
	g: G,
	p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
where
	F: Fn(A) -> B + 'a,
	G: Fn(C) -> D + 'a,
{
	Brand::bimap::<A, B, C, D, F, G>(f, g, p)
}
