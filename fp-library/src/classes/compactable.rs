//! Compactable type class.
//!
//! This module defines the [`Compactable`] trait, which represents data structures that can be compacted (filtering out `None` values) and separated (splitting `Result` values).

use crate::{Apply, brands::OptionBrand, kinds::*, types::Pair};

/// A type class for data structures that can be compacted and separated.
///
/// `Compactable` allows for:
/// *   `compact`: Filtering out `None` values and unwrapping `Some` values from a structure of `Option`s.
/// *   `separate`: Splitting a structure of `Result`s into a pair of structures, one containing the `Err` values and the other containing the `Ok` values.
pub trait Compactable: Kind_cdc7cd43dac7585f {
	/// Compacts a data structure of `Option`s, discarding `None` values and keeping `Some` values.
	///
	/// ### Type Signature
	///
	/// `forall a f. Compactable f => f (Option a) -> f a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the elements in the `Option`.
	///
	/// ### Parameters
	///
	/// * `fa`: The data structure containing `Option` values.
	///
	/// ### Returns
	///
	/// A new data structure containing only the values from the `Some` variants.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::compactable::Compactable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(Some(5));
	/// let y = OptionBrand::compact(x);
	/// assert_eq!(y, Some(5));
	///
	/// let z = Some(None::<i32>);
	/// let w = OptionBrand::compact(z);
	/// assert_eq!(w, None);
	/// ```
	fn compact<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);

	/// Separates a data structure of `Result`s into two data structures: one containing the `Ok` values and one containing the `Err` values.
	///
	/// ### Type Signature
	///
	/// `forall e a f. Compactable f => f (Result a e) -> (f a, f e)`
	///
	/// ### Type Parameters
	///
	/// * `E`: The type of the error values.
	/// * `O`: The type of the success values.
	///
	/// ### Parameters
	///
	/// * `fa`: The data structure containing `Result` values.
	///
	/// ### Returns
	///
	/// A pair of data structures: the first containing the `Ok` values, and the second containing the `Err` values.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::compactable::Compactable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::Pair;
	///
	/// let x: Option<Result<i32, &str>> = Some(Ok(5));
	/// let Pair(oks, errs) = OptionBrand::separate(x);
	/// assert_eq!(oks, Some(5));
	/// assert_eq!(errs, None);
	///
	/// let y: Option<Result<i32, &str>> = Some(Err("error"));
	/// let Pair(oks2, errs2) = OptionBrand::separate(y);
	/// assert_eq!(oks2, None);
	/// assert_eq!(errs2, Some("error"));
	/// ```
	fn separate<'a, E: 'a, O: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	>;
}

/// Compacts a data structure of `Option`s, discarding `None` values and keeping `Some` values.
///
/// Free function version that dispatches to [the type class' associated function][`Compactable::compact`].
///
/// ### Type Signature
///
/// `forall a f. Compactable f => f (Option a) -> f a`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the compactable structure.
/// * `A`: The type of the elements in the `Option`.
///
/// ### Parameters
///
/// * `fa`: The data structure containing `Option` values.
///
/// ### Returns
///
/// A new data structure containing only the values from the `Some` variants.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::compactable::compact;
/// use fp_library::brands::OptionBrand;
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

/// Separates a data structure of `Result`s into two data structures: one containing the `Ok` values and one containing the `Err` values.
///
/// Free function version that dispatches to [the type class' associated function][`Compactable::separate`].
///
/// ### Type Signature
///
/// `forall e a f. Compactable f => f (Result a e) -> (f a, f e)`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the compactable structure.
/// * `E`: The type of the error values.
/// * `O`: The type of the success values.
///
/// ### Parameters
///
/// * `fa`: The data structure containing `Result` values.
///
/// ### Returns
///
/// A pair of data structures: the first containing the `Ok` values, and the second containing the `Err` values.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::compactable::separate;
/// use fp_library::brands::OptionBrand;
/// use fp_library::types::Pair;
///
/// let x: Option<Result<i32, &str>> = Some(Ok(5));
/// let Pair(oks, errs) = separate::<OptionBrand, _, _>(x);
/// assert_eq!(oks, Some(5));
/// assert_eq!(errs, None);
/// ```
pub fn separate<'a, Brand: Compactable, E: 'a, O: 'a>(
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
) -> Pair<
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
> {
	Brand::separate(fa)
}
