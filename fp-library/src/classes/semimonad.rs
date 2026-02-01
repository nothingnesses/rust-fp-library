//! A type class for sequencing computations where the second computation depends on the result of the first.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{functions::*, brands::*};
//!
//! let x = Some(5);
//! let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
//! assert_eq!(y, Some(10));
//! ```

use fp_macros::doc_type_params;
use crate::{Apply, kinds::*};
use fp_macros::hm_signature;

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// If `x` has type `m a` and `f` has type `a -> m b`, then `bind(x, f)` has type `m b`,
/// representing the result of executing `x` to get a value of type `a` and then
/// passing it to `f` to get a computation of type `m b`.
pub trait Semimonad: Kind_cdc7cd43dac7585f {
	/// Sequences two computations, allowing the second to depend on the value computed by the first.
	///
	/// This method chains two computations, where the second computation depends on the result of the first.
	///
	/// ### Type Signature
	///
	/// `forall m b a. Semimonad m => (m a, a -> m b) -> m b`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the result of the first computation.",
		"The type of the result of the second computation.",
		("A", "The type of the result of the first computation.")
	)]	///
	/// ### Parameters
	///
	/// * `ma`: The first computation.
	/// * `f`: The function to apply to the result of the first computation.
	///
	/// ### Returns
	///
	/// The result of the second computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{functions::*, brands::*};
	///
	/// let x = Some(5);
	/// let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
	/// assert_eq!(y, Some(10));
	/// ```
	fn bind<'a, A: 'a, B: 'a, Func>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a;
}

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// Free function version that dispatches to [the type class' associated function][`Semimonad::bind`].
///
/// ### Type Signature
///
#[hm_signature(Semimonad)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"Undocumented",
	"The brand of the semimonad.",
	"The type of the result of the first computation.",
	"The type of the result of the second computation.",
	("A", "The type of the result of the first computation.")
)]///
/// ### Parameters
///
/// * `ma`: The first computation.
/// * `f`: The function to apply to the result of the first computation.
///
/// ### Returns
///
/// The result of the second computation.
///
/// ### Examples
///
/// ```
/// use fp_library::{functions::*, brands::*};
///
/// let x = Some(5);
/// let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
/// assert_eq!(y, Some(10));
/// ```
pub fn bind<'a, Brand: Semimonad, A: 'a, B: 'a, Func>(
	ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	f: Func,
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	Func: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
{
	Brand::bind::<A, B, Func>(ma, f)
}
