//! Dispatch for [`Semiapplicative::apply`](crate::classes::Semiapplicative::apply).
//!
//! Provides the [`ApplyDispatch`] trait and an inference wrapper [`apply`] that
//! uses dual Slot bounds to infer Brand from both the function container and
//! value container simultaneously.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! };
//!
//! let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
//! let x = Some(5);
//! let y = explicit::apply::<RcFnBrand, OptionBrand, _, _>(f, x);
//! assert_eq!(y, Some(10));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				CloneFn,
				Semiapplicative,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes an apply operation to [`Semiapplicative::apply`].
	///
	/// The function container `FF` is `Self`; the value container `FA` is
	/// taken as a parameter. Brand must be consistent across both containers.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The function-wrapping brand (e.g., RcFnBrand).",
		"The brand of the applicative.",
		"The type of the value(s) inside the value container.",
		"The result type after applying the function.",
		"The value container type."
	)]
	#[document_parameters("The function container implementing this dispatch.")]
	pub trait ApplyDispatch<
		'a,
		FnBrand: 'a + CloneFn,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		FA,
	> {
		/// Perform the dispatched apply operation.
		#[document_signature]
		///
		#[document_parameters("The value container to apply the function(s) to.")]
		///
		#[document_returns("A new container with the function(s) applied to the value(s).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	dispatch::semiapplicative::explicit::apply,
		/// 	functions::*,
		/// };
		///
		/// let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let x = Some(5);
		/// let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
		/// assert_eq!(y, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val: owned containers -> Semiapplicative::apply --

	/// Routes owned containers to [`Semiapplicative::apply`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The function-wrapping brand.",
		"The brand of the applicative.",
		"The type of the value(s) inside the value container.",
		"The result type after applying the function."
	)]
	#[document_parameters("The owned function container.")]
	impl<'a, FnBrand, Brand, A, B>
		ApplyDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>)
	where
		FnBrand: CloneFn + 'a,
		Brand: Semiapplicative,
		A: Clone + 'a,
		B: 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The value container to apply the function(s) to.")]
		///
		#[document_returns("A new container with the function(s) applied to the value(s).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	dispatch::semiapplicative::explicit::apply,
		/// 	functions::*,
		/// };
		///
		/// let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let x = Some(5);
		/// let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
		/// assert_eq!(y, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::apply::<FnBrand, A, B>(self, fa)
		}
	}

	// -- Inference wrapper --

	/// Applies a container of functions to a container of values, inferring
	/// Brand from both containers via dual Slot bounds.
	///
	/// Brand is resolved by intersecting the Slot bounds on FF (the function
	/// container, keyed on the function payload type) and FA (the value
	/// container, keyed on A). For single-brand types, a single Slot impl
	/// suffices. For multi-brand types, the two bounds together disambiguate
	/// Brand.
	///
	/// For types with multiple brands where the dual bounds are insufficient,
	/// use [`explicit::apply`](crate::dispatch::semiapplicative::explicit::apply) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The function-wrapping brand (e.g., RcFnBrand).",
		"The function container type. Brand is inferred from this.",
		"The value container type. Brand is also inferred from this.",
		"The type of the value(s) inside the value container.",
		"The result type after applying the function.",
		"The brand, inferred via Slot from FF and FA."
	)]
	///
	#[document_parameters(
		"The container of function(s) to apply.",
		"The container of value(s) to apply the function(s) to."
	)]
	///
	#[document_returns("A new container with the function(s) applied to the value(s).")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	dispatch::semiapplicative::apply,
	/// 	functions::*,
	/// };
	///
	/// let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let x = Some(5);
	/// let y: Option<i32> = apply::<RcFnBrand, _, _, _, _, _>(f, x);
	/// assert_eq!(y, Some(10));
	/// ```
	#[allow_named_generics]
	pub fn apply<'a, FnBrand, FF, FA, A, B, Brand>(
		ff: FF,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FnBrand: CloneFn + 'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: Clone + 'a,
		B: 'a,
		FF: Slot_cdc7cd43dac7585f<'a, Brand, <FnBrand as CloneFn>::Of<'a, A, B>>
			+ ApplyDispatch<'a, FnBrand, Brand, A, B, FA>,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
		ff.dispatch(fa)
	}

	// -- Explicit dispatch --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrapper.
	pub mod explicit {
		use super::*;

		/// Applies a container of functions to a container of values.
		///
		/// Both `FnBrand` and `Brand` must be specified via turbofish.
		/// The remaining type parameters are inferred.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The function-wrapping brand (e.g., RcFnBrand).",
			"The brand of the applicative.",
			"The type of the value(s) inside the value container.",
			"The result type after applying the function."
		)]
		///
		#[document_parameters(
			"The container of function(s) to apply.",
			"The container of value(s) to apply the function(s) to."
		)]
		///
		#[document_returns("A new container with the function(s) applied to the value(s).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	dispatch::semiapplicative::explicit::apply,
		/// 	functions::*,
		/// };
		///
		/// let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let x = Some(5);
		/// let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
		/// assert_eq!(y, Some(10));
		/// ```
		pub fn apply<'a, FnBrand: 'a + CloneFn, Brand: Semiapplicative, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::apply::<FnBrand, A, B>(ff, fa)
		}
	}
}

pub use inner::*;
