//! Data structures that can be compacted and separated in parallel.
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
//! let result: Vec<i32> = par_compact::<VecBrand, _>(v);
//! assert_eq!(result, vec![1, 3]);
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

	/// A type class for data structures that can be compacted and separated in parallel.
	///
	/// `ParCompactable` is the parallel counterpart to [`Compactable`](crate::classes::Compactable).
	/// Implementors define [`par_compact`][ParCompactable::par_compact] and
	/// [`par_separate`][ParCompactable::par_separate] directly: there is no intermediate `Vec`
	/// conversion imposed by the interface.
	///
	/// ### Thread Safety
	///
	/// All `par_*` functions require `Send` bounds on element types.
	/// These bounds apply even when the `rayon` feature is disabled, so that code compiles
	/// identically in both configurations.
	///
	/// **Note: The `rayon` feature must be enabled to use actual parallel execution. Without
	/// it, all `par_*` functions fall back to equivalent sequential operations.**
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![Some(1), None, Some(3)];
	/// let result: Vec<i32> = par_compact::<VecBrand, _>(v);
	/// assert_eq!(result, vec![1, 3]);
	///
	/// let v2: Vec<Result<i32, &str>> = vec![Ok(1), Err("e"), Ok(3)];
	/// let (errs, oks): (Vec<&str>, Vec<i32>) = par_separate::<VecBrand, _, _>(v2);
	/// assert_eq!(errs, vec!["e"]);
	/// assert_eq!(oks, vec![1, 3]);
	/// ```
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait ParCompactable {
		/// Compacts a data structure of [`Option`]s in parallel, discarding [`None`] values and
		/// keeping [`Some`] values.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential compaction.
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
		/// 	brands::VecBrand,
		/// 	classes::par_compactable::ParCompactable,
		/// };
		///
		/// let v = vec![Some(1), None, Some(3)];
		/// let result: Vec<i32> = VecBrand::par_compact(v);
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		fn par_compact<'a, A: 'a + Send>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);

		/// Separates a data structure of [`Result`]s into two data structures in parallel.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential separation.
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
		/// 	brands::VecBrand,
		/// 	classes::par_compactable::ParCompactable,
		/// };
		///
		/// let v: Vec<Result<i32, &str>> = vec![Ok(1), Err("e"), Ok(3)];
		/// let (errs, oks): (Vec<&str>, Vec<i32>) = VecBrand::par_separate(v);
		/// assert_eq!(errs, vec!["e"]);
		/// assert_eq!(oks, vec![1, 3]);
		/// ```
		fn par_separate<'a, E: 'a + Send, O: 'a + Send>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
		) -> (
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		);
	}

	/// Compacts a data structure of [`Option`]s in parallel, discarding [`None`] values and
	/// keeping [`Some`] values.
	///
	/// Free function version that dispatches to [`ParCompactable::par_compact`].
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
	/// 	functions::*,
	/// };
	///
	/// let v = vec![Some(1), None, Some(3)];
	/// let result: Vec<i32> = par_compact::<VecBrand, _>(v);
	/// assert_eq!(result, vec![1, 3]);
	/// ```
	pub fn par_compact<'a, Brand: ParCompactable, A: 'a + Send>(
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::par_compact(fa)
	}

	/// Separates a data structure of [`Result`]s into two data structures in parallel.
	///
	/// Free function version that dispatches to [`ParCompactable::par_separate`].
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
	/// 	functions::*,
	/// };
	///
	/// let v: Vec<Result<i32, &str>> = vec![Ok(1), Err("e"), Ok(3)];
	/// let (errs, oks): (Vec<&str>, Vec<i32>) = par_separate::<VecBrand, _, _>(v);
	/// assert_eq!(errs, vec!["e"]);
	/// assert_eq!(oks, vec![1, 3]);
	/// ```
	pub fn par_separate<'a, Brand: ParCompactable, E: 'a + Send, O: 'a + Send>(
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	) {
		Brand::par_separate::<E, O>(fa)
	}
}

pub use inner::*;
