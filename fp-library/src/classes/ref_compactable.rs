//! By-reference compacting and separating of structures.
//!
//! Similar to [`Compactable`](crate::classes::Compactable), but operates on borrowed containers. Elements are cloned
//! out of the borrowed structure, so the element types must implement [`Clone`].
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let v = vec![Some(1), None, Some(3)];
//! let result = compact_explicit::<VecBrand, _, _, _>(&v);
//! assert_eq!(result, vec![1, 3]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for data structures that can be compacted and separated by reference.
	///
	/// Like [`Compactable`](crate::classes::Compactable), but takes the container by reference
	/// (`&F<Option<A>>`) instead of by value. Because elements must be extracted from a
	/// borrowed container, the element types require [`Clone`].
	///
	/// ### Laws
	///
	/// The same laws as [`Compactable`](crate::classes::Compactable) apply;
	/// `ref_compact` and `ref_separate` must agree with their by-value counterparts
	/// when applied to cloned inputs.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![Some(1), None, Some(3)];
	/// let result = compact_explicit::<VecBrand, _, _, _>(&v);
	/// assert_eq!(result, vec![1, 3]);
	///
	/// let v2: Vec<Result<i32, &str>> = vec![Ok(1), Err("bad"), Ok(3)];
	/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(&v2);
	/// assert_eq!(oks, vec![1, 3]);
	/// assert_eq!(errs, vec!["bad"]);
	/// ```
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefCompactable {
		/// Compacts a borrowed data structure of [`Option`]s, discarding [`None`] values and cloning [`Some`] values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the [`Option`]. Must be [`Clone`] because elements are extracted from a borrowed container."
		)]
		///
		#[document_parameters("A reference to the data structure containing [`Option`] values.")]
		///
		#[document_returns(
			"A new data structure containing only the cloned values from the [`Some`] variants."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![Some(1), None, Some(3)];
		/// let result = compact_explicit::<VecBrand, _, _, _>(&v);
		/// assert_eq!(result, vec![1, 3]);
		///
		/// let v2 = vec![None::<i32>, None, None];
		/// let result2 = compact_explicit::<VecBrand, _, _, _>(&v2);
		/// assert_eq!(result2, Vec::<i32>::new());
		/// ```
		fn ref_compact<'a, A: 'a + Clone>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<A>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);

		/// Separates a borrowed data structure of [`Result`]s into two data structures: one containing the cloned [`Err`] values and one containing the cloned [`Ok`] values.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the error values. Must be [`Clone`] because elements are extracted from a borrowed container.",
			"The type of the success values. Must be [`Clone`] because elements are extracted from a borrowed container."
		)]
		///
		#[document_parameters("A reference to the data structure containing [`Result`] values.")]
		///
		#[document_returns(
			"A pair of data structures: the first containing the cloned [`Err`] values, and the second containing the cloned [`Ok`] values."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v: Vec<Result<i32, &str>> = vec![Ok(1), Err("bad"), Ok(3)];
		/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(&v);
		/// assert_eq!(oks, vec![1, 3]);
		/// assert_eq!(errs, vec!["bad"]);
		/// ```
		fn ref_separate<'a, E: 'a + Clone, O: 'a + Clone>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		);
	}

	/// Compacts a borrowed data structure of [`Option`]s, discarding [`None`] values and cloning [`Some`] values.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefCompactable::ref_compact`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the compactable structure.",
		"The type of the elements in the [`Option`]. Must be [`Clone`] because elements are extracted from a borrowed container."
	)]
	///
	#[document_parameters("A reference to the data structure containing [`Option`] values.")]
	///
	#[document_returns(
		"A new data structure containing only the cloned values from the [`Some`] variants."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![Some(1), None, Some(3)];
	/// let result = compact_explicit::<VecBrand, _, _, _>(&v);
	/// assert_eq!(result, vec![1, 3]);
	/// ```
	pub fn ref_compact<'a, Brand: RefCompactable, A: 'a + Clone>(
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<A>>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::ref_compact(fa)
	}

	/// Separates a borrowed data structure of [`Result`]s into two data structures: one containing the cloned [`Err`] values and one containing the cloned [`Ok`] values.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefCompactable::ref_separate`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the compactable structure.",
		"The type of the error values. Must be [`Clone`] because elements are extracted from a borrowed container.",
		"The type of the success values. Must be [`Clone`] because elements are extracted from a borrowed container."
	)]
	///
	#[document_parameters("A reference to the data structure containing [`Result`] values.")]
	///
	#[document_returns(
		"A pair of data structures: the first containing the cloned [`Err`] values, and the second containing the cloned [`Ok`] values."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v: Vec<Result<i32, &str>> = vec![Ok(1), Err("bad"), Ok(3)];
	/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(&v);
	/// assert_eq!(oks, vec![1, 3]);
	/// assert_eq!(errs, vec!["bad"]);
	/// ```
	pub fn ref_separate<'a, Brand: RefCompactable, E: 'a + Clone, O: 'a + Clone>(
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	) {
		Brand::ref_separate::<E, O>(fa)
	}
}

pub use inner::*;
