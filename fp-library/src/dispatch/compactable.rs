//! Dispatch for compactable operations:
//! [`Compactable`](crate::classes::Compactable) and
//! [`RefCompactable`](crate::classes::RefCompactable).
//!
//! Provides the following dispatch traits and unified free functions:
//!
//! - [`CompactDispatch`] + [`compact`]
//! - [`SeparateDispatch`] + [`separate`]
//!
//! Each routes to the appropriate trait method based on whether the container
//! is owned or borrowed.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // compact
//! let y = compact_explicit::<VecBrand, _, _, _>(vec![Some(1), None, Some(3)]);
//! assert_eq!(y, vec![1, 3]);
//!
//! // separate
//! let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(vec![Ok(1), Err(2), Ok(3)]);
//! assert_eq!(oks, vec![1, 3]);
//! assert_eq!(errs, vec![2]);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			brands::OptionBrand,
			classes::{
				Compactable,
				RefCompactable,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- CompactDispatch --

	/// Trait that routes a compact operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the container type; callers never specify it directly.
	/// Owned containers resolve to [`Val`], borrowed containers resolve to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the compactable.",
		"The type of the value(s) inside the `Option` wrappers.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The container implementing this dispatch.")]
	pub trait CompactDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, Marker> {
		/// Perform the dispatched compact operation.
		#[document_signature]
		///
		#[document_returns(
			"A new container with `None` values removed and `Some` values unwrapped."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = compact_explicit::<VecBrand, _, _, _>(vec![Some(1), None, Some(3)]);
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		fn dispatch(self) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	/// Routes owned containers to [`Compactable::compact`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the compactable.",
		"The type of the value(s) inside the `Option` wrappers."
	)]
	#[document_parameters("The owned container of `Option` values.")]
	impl<'a, Brand, A> CompactDispatch<'a, Brand, A, Val> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Brand: Compactable,
		A: 'a,
	{
		#[document_signature]
		///
		#[document_returns(
			"A new container with `None` values removed and `Some` values unwrapped."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = compact_explicit::<VecBrand, _, _, _>(vec![Some(1), None, Some(3)]);
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		fn dispatch(self) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::compact(self)
		}
	}

	/// Routes borrowed containers to [`RefCompactable::ref_compact`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the compactable.",
		"The type of the value(s) inside the `Option` wrappers."
	)]
	#[document_parameters("The borrowed container of `Option` values.")]
	impl<'a, Brand, A> CompactDispatch<'a, Brand, A, Ref> for &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Brand: RefCompactable,
		A: 'a + Clone,
	{
		#[document_signature]
		///
		#[document_returns(
			"A new container with `None` values removed and `Some` values unwrapped."
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
		fn dispatch(self) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::ref_compact(self)
		}
	}

	/// Removes `None` values from a container of `Option`s, unwrapping the `Some` values.
	///
	/// Dispatches to either [`Compactable::compact`] or [`RefCompactable::ref_compact`]
	/// based on whether the container is owned or borrowed.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the compactable.",
		"The type of the value(s) inside the `Option` wrappers.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters("The container of `Option` values (owned or borrowed).")]
	///
	#[document_returns("A new container with `None` values removed and `Some` values unwrapped.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Owned
	/// let y = compact_explicit::<VecBrand, _, _, _>(vec![Some(1), None, Some(3)]);
	/// assert_eq!(y, vec![1, 3]);
	///
	/// // By-ref
	/// let v = vec![Some(1), None, Some(3)];
	/// let y = compact_explicit::<VecBrand, _, _, _>(&v);
	/// assert_eq!(y, vec![1, 3]);
	/// ```
	#[allow_named_generics]
	pub fn compact<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, FA, Marker>(
		fa: FA
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		FA: CompactDispatch<'a, Brand, A, Marker>, {
		fa.dispatch()
	}

	// -- SeparateDispatch --

	/// Trait that routes a separate operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the container type; callers never specify it directly.
	/// Owned containers resolve to [`Val`], borrowed containers resolve to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the compactable.",
		"The error type inside the `Result` wrappers.",
		"The success type inside the `Result` wrappers.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The container implementing this dispatch.")]
	pub trait SeparateDispatch<'a, Brand: Kind_cdc7cd43dac7585f, E: 'a, O: 'a, Marker> {
		/// Perform the dispatched separate operation.
		#[document_signature]
		///
		#[document_returns("A tuple of two containers: `Err` values and `Ok` values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(vec![Ok(1), Err(2), Ok(3)]);
		/// assert_eq!(oks, vec![1, 3]);
		/// assert_eq!(errs, vec![2]);
		/// ```
		fn dispatch(
			self
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		);
	}

	/// Routes owned containers to [`Compactable::separate`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the compactable.",
		"The error type inside the `Result` wrappers.",
		"The success type inside the `Result` wrappers."
	)]
	#[document_parameters("The owned container of `Result` values.")]
	impl<'a, Brand, E, O> SeparateDispatch<'a, Brand, E, O, Val> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	where
		Brand: Compactable,
		E: 'a,
		O: 'a,
	{
		#[document_signature]
		///
		#[document_returns("A tuple of two containers: `Err` values and `Ok` values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(vec![Ok(1), Err(2), Ok(3)]);
		/// assert_eq!(oks, vec![1, 3]);
		/// assert_eq!(errs, vec![2]);
		/// ```
		fn dispatch(
			self
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Brand::separate(self)
		}
	}

	/// Routes borrowed containers to [`RefCompactable::ref_separate`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the compactable.",
		"The error type inside the `Result` wrappers.",
		"The success type inside the `Result` wrappers."
	)]
	#[document_parameters("The borrowed container of `Result` values.")]
	impl<'a, Brand, E, O> SeparateDispatch<'a, Brand, E, O, Ref> for &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	where
		Brand: RefCompactable,
		E: 'a + Clone,
		O: 'a + Clone,
	{
		#[document_signature]
		///
		#[document_returns("A tuple of two containers: `Err` values and `Ok` values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v: Vec<Result<i32, i32>> = vec![Ok(1), Err(2), Ok(3)];
		/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(&v);
		/// assert_eq!(oks, vec![1, 3]);
		/// assert_eq!(errs, vec![2]);
		/// ```
		fn dispatch(
			self
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Brand::ref_separate(self)
		}
	}

	/// Separates a container of `Result` values into two containers.
	///
	/// Dispatches to either [`Compactable::separate`] or [`RefCompactable::ref_separate`]
	/// based on whether the container is owned or borrowed.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the compactable.",
		"The error type inside the `Result` wrappers.",
		"The success type inside the `Result` wrappers.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters("The container of `Result` values (owned or borrowed).")]
	///
	#[document_returns("A tuple of two containers: `Err` values and `Ok` values.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Owned
	/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(vec![Ok(1), Err(2), Ok(3)]);
	/// assert_eq!(oks, vec![1, 3]);
	/// assert_eq!(errs, vec![2]);
	///
	/// // By-ref
	/// let v: Vec<Result<i32, i32>> = vec![Ok(1), Err(2), Ok(3)];
	/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(&v);
	/// assert_eq!(oks, vec![1, 3]);
	/// assert_eq!(errs, vec![2]);
	/// ```
	#[allow_named_generics]
	pub fn separate<'a, Brand: Kind_cdc7cd43dac7585f, E: 'a, O: 'a, FA, Marker>(
		fa: FA
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	)
	where
		FA: SeparateDispatch<'a, Brand, E, O, Marker>, {
		fa.dispatch()
	}
}

pub use inner::*;
