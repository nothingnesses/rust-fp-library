//! Data structures that can be compacted by filtering out [`None`] or separated by splitting [`Result`] values.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! let x = Some(Some(5));
//! let y = compact::<OptionBrand, _, _, _>(x);
//! assert_eq!(y, Some(5));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for data structures that can be compacted and separated.
	///
	/// `Compactable` allows for:
	/// *   `compact`: Filtering out [`None`] values and unwrapping [`Some`] values from a structure of [`Option`]s.
	/// *   `separate`: Splitting a structure of [`Result`]s into a pair of structures, one containing the [`Err`] values and the other containing the [`Ok`] values.
	///
	/// ### Laws
	///
	/// To be `Compactable` alone, no laws must be satisfied other than the type signature.
	///
	/// If the data type is also a [`Functor`](crate::classes::Functor):
	/// * Identity: `compact(map(Some, fa)) = fa`.
	///
	/// If the data type is also [`Plus`](crate::classes::Plus):
	/// * Annihilation (empty): `compact(empty) = empty`.
	/// * Annihilation (map): `compact(map(|_| None, xs)) = empty`.
	#[document_examples]
	///
	/// Compactable laws for [`Option`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::{
	/// 		explicit::{
	/// 			compact,
	/// 			map,
	/// 		},
	/// 		*,
	/// 	},
	/// };
	///
	/// // Functor Identity: compact(map(Some, fa)) = fa
	/// assert_eq!(
	/// 	compact::<OptionBrand, _, _, _>(map::<OptionBrand, _, _, _, _>(Some, Some(5))),
	/// 	Some(5),
	/// );
	/// assert_eq!(
	/// 	compact::<OptionBrand, _, _, _>(map::<OptionBrand, _, _, _, _>(Some, None::<i32>)),
	/// 	None,
	/// );
	///
	/// // Plus Annihilation (empty): compact(empty) = empty
	/// assert_eq!(
	/// 	compact::<OptionBrand, _, _, _>(plus_empty::<OptionBrand, Option<i32>>()),
	/// 	plus_empty::<OptionBrand, i32>(),
	/// );
	///
	/// // Plus Annihilation (map): compact(map(|_| None, xs)) = empty
	/// assert_eq!(
	/// 	compact::<OptionBrand, _, _, _>(map::<OptionBrand, _, _, _, _>(
	/// 		|_: i32| None::<i32>,
	/// 		Some(5)
	/// 	)),
	/// 	plus_empty::<OptionBrand, i32>(),
	/// );
	/// ```
	///
	/// Compactable laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::{
	/// 		explicit::{
	/// 			compact,
	/// 			map,
	/// 		},
	/// 		*,
	/// 	},
	/// };
	///
	/// // Functor Identity: compact(map(Some, fa)) = fa
	/// assert_eq!(
	/// 	compact::<VecBrand, _, _, _>(map::<VecBrand, _, _, _, _>(Some, vec![1, 2, 3])),
	/// 	vec![1, 2, 3],
	/// );
	///
	/// // Plus Annihilation (empty): compact(empty) = empty
	/// assert_eq!(
	/// 	compact::<VecBrand, _, _, _>(plus_empty::<VecBrand, Option<i32>>()),
	/// 	plus_empty::<VecBrand, i32>(),
	/// );
	///
	/// // Plus Annihilation (map): compact(map(|_| None, xs)) = empty
	/// assert_eq!(
	/// 	compact::<VecBrand, _, _, _>(map::<VecBrand, _, _, _, _>(
	/// 		|_: i32| None::<i32>,
	/// 		vec![1, 2, 3]
	/// 	)),
	/// 	plus_empty::<VecBrand, i32>(),
	/// );
	/// ```
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait Compactable {
		/// Compacts a data structure of [`Option`]s, discarding [`None`] values and keeping [`Some`] values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the [`Option`]."
		)]
		///
		#[document_parameters("The data structure containing [`Option`] values.")]
		///
		#[document_returns(
			"A new data structure containing only the values from the [`Some`] variants."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let x = Some(Some(5));
		/// let y = compact::<OptionBrand, _, _, _>(x);
		/// assert_eq!(y, Some(5));
		///
		/// let z = Some(None::<i32>);
		/// let w = compact::<OptionBrand, _, _, _>(z);
		/// assert_eq!(w, None);
		/// ```
		fn compact<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);

		/// Separates a data structure of [`Result`]s into two data structures: one containing the [`Err`] values and one containing the [`Ok`] values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the error values.",
			"The type of the success values."
		)]
		///
		#[document_parameters("The data structure containing [`Result`] values.")]
		///
		#[document_returns(
			"A pair of data structures: the first containing the [`Err`] values, and the second containing the [`Ok`] values."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let x: Option<Result<i32, &str>> = Some(Ok(5));
		/// let (errs, oks) = separate::<OptionBrand, _, _, _, _>(x);
		/// assert_eq!(oks, Some(5));
		/// assert_eq!(errs, None);
		///
		/// let y: Option<Result<i32, &str>> = Some(Err("error"));
		/// let (errs2, oks2) = separate::<OptionBrand, _, _, _, _>(y);
		/// assert_eq!(oks2, None);
		/// assert_eq!(errs2, Some("error"));
		/// ```
		fn separate<'a, E: 'a, O: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		);
	}

	/// Compacts a data structure of [`Option`]s, discarding [`None`] values and keeping [`Some`] values.
	///
	/// Free function version that dispatches to [the type class' associated function][`Compactable::compact`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the compactable structure.",
		"The type of the elements in the [`Option`]."
	)]
	///
	#[document_parameters("The data structure containing [`Option`] values.")]
	///
	#[document_returns(
		"A new data structure containing only the values from the [`Some`] variants."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::explicit::*,
	/// };
	///
	/// let x = Some(Some(5));
	/// let y = compact::<OptionBrand, _, _, _>(x);
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

	/// Separates a data structure of [`Result`]s into two data structures: one containing the [`Err`] values and one containing the [`Ok`] values.
	///
	/// Free function version that dispatches to [the type class' associated function][`Compactable::separate`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the compactable structure.",
		"The type of the error values.",
		"The type of the success values."
	)]
	///
	#[document_parameters("The data structure containing [`Result`] values.")]
	///
	#[document_returns(
		"A pair of data structures: the first containing the [`Err`] values, and the second containing the [`Ok`] values."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::explicit::*,
	/// };
	///
	/// let x: Option<Result<i32, &str>> = Some(Ok(5));
	/// let (errs, oks) = separate::<OptionBrand, _, _, _, _>(x);
	/// assert_eq!(oks, Some(5));
	/// assert_eq!(errs, None);
	/// ```
	pub fn separate<'a, Brand: Compactable, E: 'a, O: 'a>(
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	) {
		Brand::separate::<E, O>(fa)
	}
}

pub use inner::*;
