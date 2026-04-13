//! Dispatch for [`ApplyFirst::apply_first`](crate::classes::ApplyFirst::apply_first) and
//! [`RefApplyFirst::ref_apply_first`](crate::classes::RefApplyFirst::ref_apply_first).
//!
//! Provides the [`ApplyFirstDispatch`] trait and a unified
//! [`explicit::apply_first`] free function that routes to the appropriate trait
//! method based on whether the containers are owned or borrowed.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! // Owned: dispatches to ApplyFirst::apply_first
//! let y = apply_first::<OptionBrand, _, _, _, _>(Some(5), Some(10));
//! assert_eq!(y, Some(5));
//!
//! // By-ref: dispatches to RefApplyFirst::ref_apply_first
//! let a = Some(5);
//! let b = Some(10);
//! let y = apply_first::<OptionBrand, _, _, _, _>(&a, &b);
//! assert_eq!(y, Some(5));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				ApplyFirst,
				RefApplyFirst,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes an apply_first operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the container type; callers never specify it directly.
	/// Owned containers resolve to [`Val`], borrowed containers resolve to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the applicative.",
		"The type of the value(s) inside the first container.",
		"The type of the value(s) inside the second container.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The first container implementing this dispatch.")]
	pub trait ApplyFirstDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// The type of the second container argument.
		type FB;

		/// Perform the dispatched apply_first operation.
		#[document_signature]
		///
		#[document_parameters("The second container (its result is discarded).")]
		///
		#[document_returns("A container preserving the values from the first input.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = apply_first::<OptionBrand, _, _, _, _>(Some(5), Some(10));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(
			self,
			fb: Self::FB,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	// -- Val: owned containers -> ApplyFirst::apply_first --

	/// Routes owned containers to [`ApplyFirst::apply_first`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the applicative.",
		"The type of the value(s) inside the first container.",
		"The type of the value(s) inside the second container."
	)]
	#[document_parameters("The owned first container.")]
	impl<'a, Brand, A, B> ApplyFirstDispatch<'a, Brand, A, B, Val> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Brand: ApplyFirst,
		A: 'a + Clone,
		B: 'a + Clone,
	{
		type FB = Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);

		#[document_signature]
		///
		#[document_parameters("The second container (its result is discarded).")]
		///
		#[document_returns("A container preserving the values from the first input.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = apply_first::<OptionBrand, _, _, _, _>(Some(5), Some(10));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(
			self,
			fb: Self::FB,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::apply_first(self, fb)
		}
	}

	// -- Ref: borrowed containers -> RefApplyFirst::ref_apply_first --

	/// Routes borrowed containers to [`RefApplyFirst::ref_apply_first`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the applicative.",
		"The type of the value(s) inside the first container.",
		"The type of the value(s) inside the second container."
	)]
	#[document_parameters("The borrowed first container.")]
	impl<'a, 'b, Brand, A, B> ApplyFirstDispatch<'a, Brand, A, B, Ref> for &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		'a: 'b,
		Brand: RefApplyFirst,
		A: 'a + Clone,
		B: 'a,
	{
		type FB = &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);

		#[document_signature]
		///
		#[document_parameters("The second borrowed container (its result is discarded).")]
		///
		#[document_returns("A container preserving the values from the first input.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let a = Some(5);
		/// let b = Some(10);
		/// let result = apply_first::<OptionBrand, _, _, _, _>(&a, &b);
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(
			self,
			fb: Self::FB,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::ref_apply_first(self, fb)
		}
	}

	// -- Explicit dispatch free function --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Sequences two applicative actions, keeping the result of the first.
		///
		/// Dispatches to either [`ApplyFirst::apply_first`] or
		/// [`RefApplyFirst::ref_apply_first`] based on whether the containers
		/// are owned or borrowed.
		///
		/// The `Marker` type parameter is inferred automatically by the
		/// compiler from the container arguments. Callers write
		/// `apply_first::<Brand, _, _>(...)` and never need to specify
		/// `Marker` explicitly.
		///
		/// The dispatch is resolved at compile time with no runtime cost.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the applicative.",
			"The type of the value(s) inside the first container.",
			"The type of the value(s) inside the second container.",
			"The first container type (owned or borrowed), inferred from the argument.",
			"Dispatch marker type, inferred automatically."
		)]
		///
		#[document_parameters(
			"The first container (its values are preserved).",
			"The second container (its values are discarded)."
		)]
		///
		#[document_returns("A container preserving the values from the first input.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// // Owned: dispatches to ApplyFirst::apply_first
		/// let y = apply_first::<OptionBrand, _, _, _, _>(Some(5), Some(10));
		/// assert_eq!(y, Some(5));
		///
		/// // By-ref: dispatches to RefApplyFirst::ref_apply_first
		/// let a = Some(5);
		/// let b = Some(10);
		/// let y = apply_first::<OptionBrand, _, _, _, _>(&a, &b);
		/// assert_eq!(y, Some(5));
		/// ```
		pub fn apply_first<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker>(
			fa: FA,
			fb: <FA as ApplyFirstDispatch<'a, Brand, A, B, Marker>>::FB,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			FA: ApplyFirstDispatch<'a, Brand, A, B, Marker>, {
			fa.dispatch(fb)
		}
	}
}

pub use inner::*;
