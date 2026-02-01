//! Data structures that can be compacted by filtering out [`None`] or separated by splitting [`Result`] values.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(Some(5));
//! let y = compact::<OptionBrand, _>(x);
//! assert_eq!(y, Some(5));
//! ```

use crate::{Apply, brands::OptionBrand, kinds::*, types::Pair};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A type class for data structures that can be compacted and separated.
///
/// `Compactable` allows for:
/// *   `compact`: Filtering out [`None`] values and unwrapping [`Some`] values from a structure of [`Option`]s.
/// *   `separate`: Splitting a structure of [`Result`]s into a pair of structures, one containing the [`Err`] values and the other containing the [`Ok`] values.
pub trait Compactable: Kind_cdc7cd43dac7585f {
	/// Compacts a data structure of [`Option`]s, discarding [`None`] values and keeping [`Some`] values.
	///
	/// ### Type Signature
	///
	#[hm_signature(Compactable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The type of the elements in the [`Option`]."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The data structure containing [`Option`] values.")]
	///
	/// ### Returns
	///
	/// A new data structure containing only the values from the [`Some`] variants.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(Some(5));
	/// let y = compact::<OptionBrand, _>(x);
	/// assert_eq!(y, Some(5));
	///
	/// let z = Some(None::<i32>);
	/// let w = compact::<OptionBrand, _>(z);
	/// assert_eq!(w, None);
	/// ```
	fn compact<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);

	/// Separates a data structure of [`Result`]s into two data structures: one containing the [`Ok`] values and one containing the [`Err`] values.
	///
	/// ### Type Signature
	///
	#[hm_signature(Compactable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the elements.",
		"The type of the success values.",
		"The type of the error values."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The data structure containing [`Result`] values.")]
	///
	/// ### Returns
	///
	/// A pair of data structures: the first containing the [`Ok`] values, and the second containing the [`Err`] values.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let x: Option<Result<i32, &str>> = Some(Ok(5));
	/// let Pair(oks, errs) = separate::<OptionBrand, _, _>(x);
	/// assert_eq!(oks, Some(5));
	/// assert_eq!(errs, None);
	///
	/// let y: Option<Result<i32, &str>> = Some(Err("error"));
	/// let Pair(oks2, errs2) = separate::<OptionBrand, _, _>(y);
	/// assert_eq!(oks2, None);
	/// assert_eq!(errs2, Some("error"));
	/// ```
	fn separate<'a, O: 'a, E: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	>;
}

/// Compacts a data structure of [`Option`]s, discarding [`None`] values and keeping [`Some`] values.
///
/// Free function version that dispatches to [the type class' associated function][`Compactable::compact`].
///
/// ### Type Signature
///
#[hm_signature(Compactable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the compactable structure.",
	"The type of the elements in the [`Option`]."
)]
///
/// ### Parameters
///
#[doc_params("The data structure containing [`Option`] values.")]
///
/// ### Returns
///
/// A new data structure containing only the values from the [`Some`] variants.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(Some(5));
/// let y = compact::<OptionBrand, _>(x);
/// assert_eq!(y, Some(5));
/// ```
pub fn compact<'a, Brand: Compactable, A: 'a>(
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
	Brand::compact(fa)
}

/// Separates a data structure of [`Result`]s into two data structures: one containing the [`Ok`] values and one containing the [`Err`] values.
///
/// Free function version that dispatches to [the type class' associated function][`Compactable::separate`].
///
/// ### Type Signature
///
#[hm_signature(Compactable)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the elements.",
	"The brand of the compactable structure.",
	"The type of the success values.",
	"The type of the error values."
)]
///
/// ### Parameters
///
#[doc_params("The data structure containing [`Result`] values.")]
///
/// ### Returns
///
/// A pair of data structures: the first containing the [`Ok`] values, and the second containing the [`Err`] values.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let x: Option<Result<i32, &str>> = Some(Ok(5));
/// let Pair(oks, errs) = separate::<OptionBrand, _, _>(x);
/// assert_eq!(oks, Some(5));
/// assert_eq!(errs, None);
/// ```
pub fn separate<'a, Brand: Compactable, O: 'a, E: 'a>(
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
) -> Pair<
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
> {
	Brand::separate::<O, E>(fa)
}
