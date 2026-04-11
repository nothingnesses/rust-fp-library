//! Dispatch for [`Compactable::separate`](crate::classes::Compactable::separate) and
//! [`RefCompactable::ref_separate`](crate::classes::RefCompactable::ref_separate).
//!
//! Provides the [`SeparateDispatch`] trait and a unified [`separate`] free function
//! that routes to the appropriate trait method based on whether the container
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
//! // Owned: dispatches to Compactable::separate
//! let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(vec![Ok(1), Err(2), Ok(3)]);
//! assert_eq!(oks, vec![1, 3]);
//! assert_eq!(errs, vec![2]);
//!
//! // By-ref: dispatches to RefCompactable::ref_separate
//! let v: Vec<Result<i32, i32>> = vec![Ok(1), Err(2), Ok(3)];
//! let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(&v);
//! assert_eq!(oks, vec![1, 3]);
//! assert_eq!(errs, vec![2]);
//! ```

pub(crate) mod inner {
	use {
		crate::{
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
		fn dispatch_separate(
			self
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		);
	}

	// -- Val: owned container -> Compactable::separate --

	/// Routes owned containers to [`Compactable::separate`].
	impl<'a, Brand, E, O> SeparateDispatch<'a, Brand, E, O, Val> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	where
		Brand: Compactable,
		E: 'a,
		O: 'a,
	{
		fn dispatch_separate(
			self
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Brand::separate(self)
		}
	}

	// -- Ref: borrowed container -> RefCompactable::ref_separate --

	/// Routes borrowed containers to [`RefCompactable::ref_separate`].
	impl<'a, Brand, E, O> SeparateDispatch<'a, Brand, E, O, Ref> for &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	where
		Brand: RefCompactable,
		E: 'a + Clone,
		O: 'a + Clone,
	{
		fn dispatch_separate(
			self
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Brand::ref_separate(self)
		}
	}

	// -- Unified free function --

	/// Separates a container of `Result` values into two containers.
	///
	/// Dispatches to either [`Compactable::separate`] or [`RefCompactable::ref_separate`]
	/// based on whether the container is owned or borrowed.
	///
	/// The `Marker` type parameter is inferred automatically by the
	/// compiler from the container argument. Callers write
	/// `separate_explicit::<Brand, _, _>(...)` and never need to specify `Marker` explicitly.
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
	/// // Owned: dispatches to Compactable::separate
	/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(vec![Ok(1), Err(2), Ok(3)]);
	/// assert_eq!(oks, vec![1, 3]);
	/// assert_eq!(errs, vec![2]);
	///
	/// // By-ref: dispatches to RefCompactable::ref_separate
	/// let v: Vec<Result<i32, i32>> = vec![Ok(1), Err(2), Ok(3)];
	/// let (errs, oks) = separate_explicit::<VecBrand, _, _, _, _>(&v);
	/// assert_eq!(oks, vec![1, 3]);
	/// assert_eq!(errs, vec![2]);
	/// ```
	pub fn separate<'a, Brand: Kind_cdc7cd43dac7585f, E: 'a, O: 'a, FA, Marker>(
		fa: FA
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	)
	where
		FA: SeparateDispatch<'a, Brand, E, O, Marker>, {
		fa.dispatch_separate()
	}
}

pub use inner::*;
