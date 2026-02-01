//! A type class for semigroupoids, representing a set of objects and composable relationships between them.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::*, functions::*};
//!
//! let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
//! let g = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
//! let h = semigroupoid_compose::<RcFnBrand, _, _, _>(f, g);
//! assert_eq!(h(5), 12); // (5 + 1) * 2
//! ```

use crate::{Apply, kinds::*};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A type class for semigroupoids.
///
/// A `Semigroupoid` is a set of objects and composable relationships
/// (morphisms) between them.
///
/// ### Laws
///
/// Semigroupoid instances must satisfy the associative law:
/// * Associativity: `compose(p, compose(q, r)) = compose(compose(p, q), r)`.
pub trait Semigroupoid: Kind_140eb1e35dc7afb3 {
	/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
	///
	/// This method composes two morphisms `f` and `g` to produce a new morphism that represents the application of `g` followed by `f`.
	///
	/// ### Type Signature
	///
	/// `forall b d c. Semigroupoid f => (f c d, f b c) -> f b d`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The source type of the first morphism.",
		"The target type of the first morphism and the source type of the second morphism.",
		"The target type of the second morphism."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The second morphism to apply (from C to D).",
		"The first morphism to apply (from B to C)."
	)]
	///
	/// ### Returns
	///
	/// The composed morphism (from B to D).
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*};
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let g = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	/// let h = semigroupoid_compose::<RcFnBrand, _, _, _>(f, g);
	/// assert_eq!(h(5), 12); // (5 + 1) * 2
	/// ```
	fn compose<'a, B: 'a, C: 'a, D: 'a>(
		f: Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, C, D>),
		g: Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, B, C>),
	) -> Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, B, D>);
}

/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
///
/// Free function version that dispatches to [the type class' associated function][`Semigroupoid::compose`].
///
/// ### Type Signature
///
#[hm_signature(Semigroupoid)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"Undocumented",
	"The brand of the semigroupoid.",
	"The source type of the first morphism.",
	"The target type of the first morphism and the source type of the second morphism.",
	"The target type of the second morphism."
)]
///
/// ### Parameters
///
#[doc_params(
	"The second morphism to apply (from C to D).",
	"The first morphism to apply (from B to C)."
)]
///
/// ### Returns
///
/// The composed morphism (from B to D).
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::*, functions::*};
///
/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
/// let g = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
/// let h = semigroupoid_compose::<RcFnBrand, _, _, _>(f, g);
/// assert_eq!(h(5), 12); // (5 + 1) * 2
/// ```
pub fn compose<'a, Brand: Semigroupoid, B: 'a, C: 'a, D: 'a>(
	f: Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, C, D>),
	g: Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, B, C>),
) -> Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, B, D>) {
	Brand::compose::<B, C, D>(f, g)
}
