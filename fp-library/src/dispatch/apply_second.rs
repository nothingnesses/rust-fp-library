//! Dispatch for [`ApplySecond::apply_second`](crate::classes::ApplySecond::apply_second) and
//! [`RefApplySecond::ref_apply_second`](crate::classes::RefApplySecond::ref_apply_second).
//!
//! Provides the [`ApplySecondDispatch`] trait and a unified
//! [`explicit::apply_second`] free function that routes to the appropriate trait
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
//! // Owned: dispatches to ApplySecond::apply_second
//! let y = apply_second::<OptionBrand, _, _, _, _>(Some(5), Some(10));
//! assert_eq!(y, Some(10));
//!
//! // By-ref: dispatches to RefApplySecond::ref_apply_second
//! let a = Some(5);
//! let b = Some(10);
//! let y = apply_second::<OptionBrand, _, _, _, _>(&a, &b);
//! assert_eq!(y, Some(10));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				ApplySecond,
				RefApplySecond,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes an apply_second operation to the appropriate type class method.
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
	pub trait ApplySecondDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// The type of the second container argument.
		type FB;

		/// Perform the dispatched apply_second operation.
		#[document_signature]
		///
		#[document_parameters("The second container (its result is kept).")]
		///
		#[document_returns("A container preserving the values from the second input.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = apply_second::<OptionBrand, _, _, _, _>(Some(5), Some(10));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fb: Self::FB,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val: owned containers -> ApplySecond::apply_second --

	/// Routes owned containers to [`ApplySecond::apply_second`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the applicative.",
		"The type of the value(s) inside the first container.",
		"The type of the value(s) inside the second container."
	)]
	#[document_parameters("The owned first container.")]
	impl<'a, Brand, A, B> ApplySecondDispatch<'a, Brand, A, B, Val> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Brand: ApplySecond,
		A: 'a + Clone,
		B: 'a + Clone,
	{
		type FB = Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);

		#[document_signature]
		///
		#[document_parameters("The second container (its result is kept).")]
		///
		#[document_returns("A container preserving the values from the second input.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = apply_second::<OptionBrand, _, _, _, _>(Some(5), Some(10));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fb: Self::FB,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::apply_second::<A, B>(self, fb)
		}
	}

	// -- Ref: borrowed containers -> RefApplySecond::ref_apply_second --

	/// Routes borrowed containers to [`RefApplySecond::ref_apply_second`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the applicative.",
		"The type of the value(s) inside the first container.",
		"The type of the value(s) inside the second container."
	)]
	#[document_parameters("The borrowed first container.")]
	impl<'a, 'b, Brand, A, B> ApplySecondDispatch<'a, Brand, A, B, Ref> for &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		'a: 'b,
		Brand: RefApplySecond,
		A: 'a,
		B: 'a + Clone,
	{
		type FB = &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);

		#[document_signature]
		///
		#[document_parameters("The second borrowed container (its result is kept).")]
		///
		#[document_returns("A container preserving the values from the second input.")]
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
		/// let result = apply_second::<OptionBrand, _, _, _, _>(&a, &b);
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fb: Self::FB,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_apply_second(self, fb)
		}
	}

	// -- Inference wrapper --

	/// Sequences two applicative actions, keeping the result of the second,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::apply_second`](crate::functions::explicit::apply_second) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The first container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the first container.",
		"The type of the value(s) inside the second container.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The first container (its values are discarded).",
		"The second container (its values are preserved)."
	)]
	///
	#[document_returns("A container preserving the values from the second input.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(apply_second(Some(5), Some(10)), Some(10));
	///
	/// let a = Some(5);
	/// let b = Some(10);
	/// assert_eq!(apply_second(&a, &b), Some(10));
	/// ```
	pub fn apply_second<'a, FA, A: 'a, B: 'a, Marker>(
		fa: FA,
		fb: <FA as ApplySecondDispatch<
			'a,
			<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
			A,
			B,
			Marker,
		>>::FB,
	) -> <<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
	where
		FA: InferableBrand_cdc7cd43dac7585f
			+ ApplySecondDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, B, Marker>, {
		fa.dispatch(fb)
	}

	// -- Explicit dispatch free function --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Sequences two applicative actions, keeping the result of the second.
		///
		/// Dispatches to either [`ApplySecond::apply_second`] or
		/// [`RefApplySecond::ref_apply_second`] based on whether the containers
		/// are owned or borrowed.
		///
		/// The `Marker` type parameter is inferred automatically by the
		/// compiler from the container arguments. Callers write
		/// `apply_second::<Brand, _, _>(...)` and never need to specify
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
			"The first container (its values are discarded).",
			"The second container (its values are preserved)."
		)]
		///
		#[document_returns("A container preserving the values from the second input.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// // Owned: dispatches to ApplySecond::apply_second
		/// let y = apply_second::<OptionBrand, _, _, _, _>(Some(5), Some(10));
		/// assert_eq!(y, Some(10));
		///
		/// // By-ref: dispatches to RefApplySecond::ref_apply_second
		/// let a = Some(5);
		/// let b = Some(10);
		/// let y = apply_second::<OptionBrand, _, _, _, _>(&a, &b);
		/// assert_eq!(y, Some(10));
		/// ```
		pub fn apply_second<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker>(
			fa: FA,
			fb: <FA as ApplySecondDispatch<'a, Brand, A, B, Marker>>::FB,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			FA: ApplySecondDispatch<'a, Brand, A, B, Marker>, {
			fa.dispatch(fb)
		}
	}
}

pub use inner::*;
