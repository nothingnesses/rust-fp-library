//! Dispatch for [`Alt::alt`](crate::classes::Alt::alt) and
//! [`RefAlt::ref_alt`](crate::classes::RefAlt::ref_alt).
//!
//! Provides the [`AltDispatch`] trait and a unified [`explicit::alt`] free
//! function that routes to the appropriate trait method based on whether the
//! container is owned or borrowed.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! // Owned: dispatches to Alt::alt
//! let y = alt::<OptionBrand, _, _, _>(None, Some(5));
//! assert_eq!(y, Some(5));
//!
//! // By-ref: dispatches to RefAlt::ref_alt
//! let y = alt::<OptionBrand, _, _, _>(&None, &Some(5));
//! assert_eq!(y, Some(5));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Alt,
				RefAlt,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes an alt operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the container type; callers never specify it directly.
	/// Owned containers resolve to [`Val`], borrowed containers resolve to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The container implementing this dispatch.")]
	pub trait AltDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a + Clone, Marker> {
		/// Perform the dispatched alt operation.
		#[document_signature]
		///
		#[document_parameters("The other container to combine with.")]
		///
		#[document_returns("A new container from the combination of both inputs.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = alt::<OptionBrand, _, _, _>(None, Some(5));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(
			self,
			other: Self,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	// -- Val: owned container -> Alt::alt --

	/// Routes owned containers to [`Alt::alt`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor."
	)]
	#[document_parameters("The owned container.")]
	impl<'a, Brand, A> AltDispatch<'a, Brand, A, Val> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Brand: Alt,
		A: 'a + Clone,
	{
		#[document_signature]
		///
		#[document_parameters("The other container to combine with.")]
		///
		#[document_returns("A new container from the combination of both inputs.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = alt::<OptionBrand, _, _, _>(None, Some(5));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(
			self,
			other: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::alt(self, other)
		}
	}

	// -- Ref: borrowed container -> RefAlt::ref_alt --

	/// Routes borrowed containers to [`RefAlt::ref_alt`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor."
	)]
	#[document_parameters("The borrowed container.")]
	impl<'a, Brand, A> AltDispatch<'a, Brand, A, Ref> for &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Brand: RefAlt,
		A: 'a + Clone,
	{
		#[document_signature]
		///
		#[document_parameters("The other borrowed container to combine with.")]
		///
		#[document_returns("A new container from the combination of both inputs.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let x: Option<i32> = None;
		/// let y = Some(5);
		/// let result = alt::<OptionBrand, _, _, _>(&x, &y);
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(
			self,
			other: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::ref_alt(self, other)
		}
	}

	// -- Inference wrapper --

	/// Combines two values in a context, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa1`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::alt`](crate::functions::explicit::alt) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the functor.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The first container (owned or borrowed).",
		"The second container (same ownership as the first)."
	)]
	///
	#[document_returns("A new container from the combination of both inputs.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(alt(None, Some(5)), Some(5));
	///
	/// let x = vec![1, 2];
	/// let y = vec![3, 4];
	/// assert_eq!(alt(&x, &y), vec![1, 2, 3, 4]);
	/// ```
	pub fn alt<'a, FA, A: 'a + Clone, Marker>(
		fa1: FA,
		fa2: FA,
	) -> <<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
	where
		FA: InferableBrand_cdc7cd43dac7585f
			+ AltDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, Marker>, {
		fa1.dispatch(fa2)
	}

	// -- Explicit dispatch free function --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Combines two values in a context, choosing associatively.
		///
		/// Dispatches to either [`Alt::alt`] or [`RefAlt::ref_alt`]
		/// based on whether the containers are owned or borrowed.
		///
		/// The `Marker` type parameter is inferred automatically by the
		/// compiler from the container argument. Callers write
		/// `alt::<Brand, _>(...)` and never need to specify `Marker` explicitly.
		///
		/// The dispatch is resolved at compile time with no runtime cost.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the functor.",
			"The type of the value(s) inside the functor.",
			"The container type (owned or borrowed), inferred from the argument.",
			"Dispatch marker type, inferred automatically."
		)]
		///
		#[document_parameters("The first container.", "The second container.")]
		///
		#[document_returns("A new container from the combination of both inputs.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// // Owned: dispatches to Alt::alt
		/// let y = alt::<OptionBrand, _, _, _>(None, Some(5));
		/// assert_eq!(y, Some(5));
		///
		/// // By-ref: dispatches to RefAlt::ref_alt
		/// let x: Option<i32> = None;
		/// let y = Some(5);
		/// let z = alt::<OptionBrand, _, _, _>(&x, &y);
		/// assert_eq!(z, Some(5));
		/// ```
		pub fn alt<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a + Clone, FA, Marker>(
			fa1: FA,
			fa2: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			FA: AltDispatch<'a, Brand, A, Marker>, {
			fa1.dispatch(fa2)
		}
	}
}

pub use inner::*;
