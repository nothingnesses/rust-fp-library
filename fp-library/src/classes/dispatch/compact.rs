//! Dispatch for [`Compactable::compact`](crate::classes::Compactable::compact) and
//! [`RefCompactable::ref_compact`](crate::classes::RefCompactable::ref_compact).
//!
//! Provides the [`CompactDispatch`] trait and a unified [`compact`] free function
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
//! // Owned: dispatches to Compactable::compact
//! let y = compact_explicit::<VecBrand, _, _, _>(vec![Some(1), None, Some(3)]);
//! assert_eq!(y, vec![1, 3]);
//!
//! // By-ref: dispatches to RefCompactable::ref_compact
//! let v = vec![Some(1), None, Some(3)];
//! let y = compact_explicit::<VecBrand, _, _, _>(&v);
//! assert_eq!(y, vec![1, 3]);
//! ```

pub(crate) mod inner {
	use {
		crate::{
			brands::OptionBrand,
			classes::{
				Compactable,
				RefCompactable,
				dispatch::{
					Ref,
					Val,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a compact operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the container type; callers never specify it directly.
	/// Owned containers resolve to [`Val`], borrowed containers resolve to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the compactable.",
		"The type of the value(s) inside the `Option` wrappers.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
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
		fn dispatch_compact(self)
		-> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	// -- Val: owned container -> Compactable::compact --

	/// Routes owned containers to [`Compactable::compact`].
	impl<'a, Brand, A> CompactDispatch<'a, Brand, A, Val> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Brand: Compactable,
		A: 'a,
	{
		fn dispatch_compact(
			self
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::compact(self)
		}
	}

	// -- Ref: borrowed container -> RefCompactable::ref_compact --

	/// Routes borrowed containers to [`RefCompactable::ref_compact`].
	impl<'a, Brand, A> CompactDispatch<'a, Brand, A, Ref> for &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Brand: RefCompactable,
		A: 'a + Clone,
	{
		fn dispatch_compact(
			self
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::ref_compact(self)
		}
	}

	// -- Unified free function --

	/// Removes `None` values from a container of `Option`s, unwrapping the `Some` values.
	///
	/// Dispatches to either [`Compactable::compact`] or [`RefCompactable::ref_compact`]
	/// based on whether the container is owned or borrowed.
	///
	/// The `Marker` type parameter is inferred automatically by the
	/// compiler from the container argument. Callers write
	/// `compact_explicit::<Brand, _>(...)` and never need to specify `Marker` explicitly.
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
	/// // Owned: dispatches to Compactable::compact
	/// let y = compact_explicit::<VecBrand, _, _, _>(vec![Some(1), None, Some(3)]);
	/// assert_eq!(y, vec![1, 3]);
	///
	/// // By-ref: dispatches to RefCompactable::ref_compact
	/// let v = vec![Some(1), None, Some(3)];
	/// let y = compact_explicit::<VecBrand, _, _, _>(&v);
	/// assert_eq!(y, vec![1, 3]);
	/// ```
	pub fn compact<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, FA, Marker>(
		fa: FA
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		FA: CompactDispatch<'a, Brand, A, Marker>, {
		fa.dispatch_compact()
	}
}

pub use inner::*;
