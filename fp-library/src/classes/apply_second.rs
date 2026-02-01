//! A type class for sequencing two computations and keeping the result of the second.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(5);
//! let y = Some(10);
//! let z = apply_second::<OptionBrand, _, _>(x, y);
//! assert_eq!(z, Some(10));
//! ```

use super::lift::Lift;
use crate::{Apply, kinds::*};
use fp_macros::hm_signature;

/// A type class for types that support combining two contexts, keeping the second value.
///
/// `ApplySecond` provides the ability to sequence two computations but discard
/// the result of the first computation, keeping only the result of the second.
pub trait ApplySecond: Lift {
	/// Combines two contexts, keeping the value from the second context.
	///
	/// This function sequences two computations and discards the result of the first computation, keeping only the result of the second.
	///
	/// ### Type Signature
	///
	#[hm_signature(ApplySecond)]
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value in the first context.
	/// * `B`: The type of the value in the second context.
	///
	/// ### Parameters
	///
	/// * `fa`: The first context.
	/// * `fb`: The second context.
	///
	/// ### Returns
	///
	/// The second context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(5);
	/// let y = Some(10);
	/// let z = apply_second::<OptionBrand, _, _>(x, y);
	/// assert_eq!(z, Some(10));
	/// ```
	fn apply_second<'a, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Self::lift2::<A, B, B, _>(|_, b| b, fa, fb)
	}
}

/// Combines two contexts, keeping the value from the second context.
///
/// Free function version that dispatches to [the type class' associated function][`ApplySecond::apply_second`].
///
/// ### Type Signature
///
#[hm_signature(ApplySecond)]
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the context.
/// * `A`: The type of the value in the first context.
/// * `B`: The type of the value in the second context.
///
/// ### Parameters
///
/// * `fa`: The first context.
/// * `fb`: The second context.
///
/// ### Returns
///
/// The second context.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(5);
/// let y = Some(10);
/// let z = apply_second::<OptionBrand, _, _>(x, y);
/// assert_eq!(z, Some(10));
/// ```
pub fn apply_second<'a, Brand: ApplySecond, A: 'a + Clone, B: 'a + Clone>(
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
	Brand::apply_second::<A, B>(fa, fb)
}
