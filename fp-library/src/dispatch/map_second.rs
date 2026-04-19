//! Dispatch for [`Bifunctor::map_second`](crate::classes::Bifunctor::map_second) and
//! [`RefBifunctor::ref_map_second`](crate::classes::RefBifunctor::ref_map_second).
//!
//! Provides the [`MapSecondDispatch`] trait and a unified [`explicit::map_second`]
//! free function that routes to the appropriate trait method based on the
//! closure's argument type.
//!
//! Corresponds to `rmap` in both Haskell and PureScript.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! // Owned: dispatches to Bifunctor::map_second
//! let x = Result::<i32, i32>::Ok(5);
//! let y = map_second::<ResultBrand, _, _, _, _, _>(|s| s * 2, x);
//! assert_eq!(y, Ok(10));
//!
//! // By-ref: dispatches to RefBifunctor::ref_map_second
//! let x = Result::<i32, i32>::Ok(5);
//! let y = map_second::<ResultBrand, _, _, _, _, _>(|s: &i32| *s * 2, &x);
//! assert_eq!(y, Ok(10));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Bifunctor,
				RefBifunctor,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a map_second operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is inferred from the closure's argument type:
	/// `Fn(B) -> C` resolves to [`Val`](crate::dispatch::Val),
	/// `Fn(&B) -> C` resolves to [`Ref`](crate::dispatch::Ref).
	/// The `FA` type parameter is inferred from the container argument: owned
	/// for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bifunctor.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the second result.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait MapSecondDispatch<'a, Brand: Kind_266801a817966495, A: 'a, B: 'a, C: 'a, FA, Marker> {
		/// Perform the dispatched map_second operation.
		#[document_signature]
		#[document_parameters("The bifunctor value.")]
		#[document_returns("A new bifunctor with the second value transformed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result = map_second::<ResultBrand, _, _, _, _, _>(|s| s * 2, Ok::<i32, i32>(5));
		/// assert_eq!(result, Ok(10));
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>);
	}

	// -- Val: Fn(B) -> C -> Bifunctor::map_second --

	/// Routes `Fn(B) -> C` closures to [`Bifunctor::map_second`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bifunctor.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the second result.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, F>
		MapSecondDispatch<
			'a,
			Brand,
			A,
			B,
			C,
			Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
			Val,
		> for F
	where
		Brand: Bifunctor,
		A: 'a,
		B: 'a,
		C: 'a,
		F: Fn(B) -> C + 'a,
	{
		#[document_signature]
		#[document_parameters("The bifunctor value.")]
		#[document_returns("A new bifunctor with the second value transformed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result = map_second::<ResultBrand, _, _, _, _, _>(|s| s * 2, Ok::<i32, i32>(5));
		/// assert_eq!(result, Ok(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>) {
			Brand::map_second(self, fa)
		}
	}

	// -- Ref: Fn(&B) -> C -> RefBifunctor::ref_map_second --

	/// Routes `Fn(&B) -> C` closures to [`RefBifunctor::ref_map_second`].
	///
	/// The container must be passed by reference (`&p`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the bifunctor.",
		"The type of the first value (must be Clone).",
		"The type of the second value.",
		"The type of the second result.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, B, C, F>
		MapSecondDispatch<
			'a,
			Brand,
			A,
			B,
			C,
			&'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
			Ref,
		> for F
	where
		Brand: RefBifunctor,
		A: Clone + 'a,
		B: 'a,
		C: 'a,
		F: Fn(&B) -> C + 'a,
	{
		#[document_signature]
		#[document_parameters("A reference to the bifunctor value.")]
		#[document_returns("A new bifunctor with the second value transformed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let x = Ok::<i32, i32>(5);
		/// let result = map_second::<ResultBrand, _, _, _, _, _>(|s: &i32| *s * 2, &x);
		/// assert_eq!(result, Ok(10));
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>) {
			Brand::ref_map_second(self, fa)
		}
	}

	// -- Inference wrapper --

	/// Maps a function over the second type argument of a bifunctor, inferring the
	/// brand from the container type.
	///
	/// Corresponds to `rmap` in both Haskell and PureScript.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `p` via
	/// the `InferableBrand` trait. Both owned and borrowed containers are
	/// supported.
	///
	/// For types that need an explicit brand, use
	/// [`explicit::map_second`](crate::functions::explicit::map_second) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the second result.",
		"The brand, inferred via InferableBrand from FA and the element types."
	)]
	///
	#[document_parameters(
		"The function to apply to the second value.",
		"The bifunctor value (owned or borrowed)."
	)]
	///
	#[document_returns("A new bifunctor with the second value transformed.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// // Brand inferred from Result<i32, String>
	/// let x: Result<i32, String> = Ok(5);
	/// let y = map_second(|s| s * 2, x);
	/// assert_eq!(y, Ok(10));
	/// ```
	pub fn map_second<'a, FA, A: 'a, B: 'a, C: 'a, Brand>(
		g: impl MapSecondDispatch<
			'a,
			Brand,
			A,
			B,
			C,
			FA,
			<FA as InferableBrand_266801a817966495<'a, Brand, A, B>>::Marker,
		>,
		p: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>)
	where
		Brand: Kind_266801a817966495,
		FA: InferableBrand_266801a817966495<'a, Brand, A, B>, {
		g.dispatch(p)
	}

	// -- Explicit dispatch free function --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Maps a function over the second type argument of a bifunctor.
		///
		/// Corresponds to `rmap` in both Haskell and PureScript.
		///
		/// Dispatches to either [`Bifunctor::map_second`] or
		/// [`RefBifunctor::ref_map_second`] based on the closure's argument type.
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by the
		/// compiler. Callers write `map_second::<Brand, _, _, _, _>(...)` and never
		/// need to specify `Marker` or `FA` explicitly.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the bifunctor.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the second result.",
			"The container type (owned or borrowed), inferred from the argument.",
			"Dispatch marker type, inferred automatically."
		)]
		///
		#[document_parameters(
			"The function to apply to the second value.",
			"The bifunctor value (owned for Val, borrowed for Ref)."
		)]
		///
		#[document_returns("A new bifunctor with the second value transformed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// // Owned
		/// let x = Result::<i32, i32>::Ok(5);
		/// let y = map_second::<ResultBrand, _, _, _, _, _>(|s| s * 2, x);
		/// assert_eq!(y, Ok(10));
		///
		/// // By-ref
		/// let x = Result::<i32, i32>::Ok(5);
		/// let y = map_second::<ResultBrand, _, _, _, _, _>(|s: &i32| *s * 2, &x);
		/// assert_eq!(y, Ok(10));
		/// ```
		pub fn map_second<'a, Brand: Kind_266801a817966495, A: 'a, B: 'a, C: 'a, FA, Marker>(
			g: impl MapSecondDispatch<'a, Brand, A, B, C, FA, Marker>,
			p: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>) {
			g.dispatch(p)
		}
	}
}

pub use inner::*;
