//! Dispatch for [`Bifunctor::map_first`](crate::classes::Bifunctor::map_first) and
//! [`RefBifunctor::ref_map_first`](crate::classes::RefBifunctor::ref_map_first).
//!
//! Provides the [`MapFirstDispatch`] trait and a unified [`explicit::map_first`]
//! free function that routes to the appropriate trait method based on the
//! closure's argument type.
//!
//! Corresponds to `lmap` in both Haskell and PureScript.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! // Owned: dispatches to Bifunctor::map_first
//! let x = Result::<i32, i32>::Err(5);
//! let y = map_first::<ResultBrand, _, _, _, _, _>(|e| e * 2, x);
//! assert_eq!(y, Err(10));
//!
//! // By-ref: dispatches to RefBifunctor::ref_map_first
//! let x = Result::<i32, i32>::Err(5);
//! let y = map_first::<ResultBrand, _, _, _, _, _>(|e: &i32| *e * 2, &x);
//! assert_eq!(y, Err(10));
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

	/// Trait that routes a map_first operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is inferred from the closure's argument type:
	/// `Fn(A) -> B` resolves to [`Val`](crate::dispatch::Val),
	/// `Fn(&A) -> B` resolves to [`Ref`](crate::dispatch::Ref).
	/// The `FA` type parameter is inferred from the container argument: owned
	/// for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bifunctor.",
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait MapFirstDispatch<'a, Brand: Kind_266801a817966495, A: 'a, B: 'a, C: 'a, FA, Marker> {
		/// Perform the dispatched map_first operation.
		#[document_signature]
		#[document_parameters("The bifunctor value.")]
		#[document_returns("A new bifunctor with the first value transformed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result = map_first::<ResultBrand, _, _, _, _, _>(|e| e + 1, Err::<i32, i32>(5));
		/// assert_eq!(result, Err(6));
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, C>);
	}

	// -- Val: Fn(A) -> B -> Bifunctor::map_first --

	/// Routes `Fn(A) -> B` closures to [`Bifunctor::map_first`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bifunctor.",
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, C, F>
		MapFirstDispatch<
			'a,
			Brand,
			A,
			B,
			C,
			Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
			Val,
		> for F
	where
		Brand: Bifunctor,
		A: 'a,
		B: 'a,
		C: 'a,
		F: Fn(A) -> B + 'a,
	{
		#[document_signature]
		#[document_parameters("The bifunctor value.")]
		#[document_returns("A new bifunctor with the first value transformed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result = map_first::<ResultBrand, _, _, _, _, _>(|e| e + 1, Err::<i32, i32>(5));
		/// assert_eq!(result, Err(6));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, C>) {
			Brand::map_first(self, fa)
		}
	}

	// -- Ref: Fn(&A) -> B -> RefBifunctor::ref_map_first --

	/// Routes `Fn(&A) -> B` closures to [`RefBifunctor::ref_map_first`].
	///
	/// The container must be passed by reference (`&p`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the bifunctor.",
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value (must be Clone).",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, B, C, F>
		MapFirstDispatch<
			'a,
			Brand,
			A,
			B,
			C,
			&'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
			Ref,
		> for F
	where
		Brand: RefBifunctor,
		A: 'a,
		B: 'a,
		C: Clone + 'a,
		F: Fn(&A) -> B + 'a,
	{
		#[document_signature]
		#[document_parameters("A reference to the bifunctor value.")]
		#[document_returns("A new bifunctor with the first value transformed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let x = Err::<i32, i32>(5);
		/// let result = map_first::<ResultBrand, _, _, _, _, _>(|e: &i32| *e + 1, &x);
		/// assert_eq!(result, Err(6));
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, C>) {
			Brand::ref_map_first(self, fa)
		}
	}

	// -- Inference wrapper --

	/// Maps a function over the first type argument of a bifunctor, inferring the
	/// brand from the container type.
	///
	/// Corresponds to `lmap` in both Haskell and PureScript.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `p` via
	/// the `InferableBrand` trait. Both owned and borrowed containers are
	/// supported.
	///
	/// For types that need an explicit brand, use
	/// [`explicit::map_first`](crate::functions::explicit::map_first) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value.",
		"The brand, inferred via InferableBrand from FA and the element types."
	)]
	///
	#[document_parameters(
		"The function to apply to the first value.",
		"The bifunctor value (owned or borrowed)."
	)]
	///
	#[document_returns("A new bifunctor with the first value transformed.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// // Brand inferred from Result<i32, String>
	/// let x: Result<String, i32> = Err(5);
	/// let y = map_first(|e| e * 2, x);
	/// assert_eq!(y, Err(10));
	/// ```
	pub fn map_first<'a, FA, A: 'a, B: 'a, C: 'a, Brand>(
		f: impl MapFirstDispatch<
			'a,
			Brand,
			A,
			B,
			C,
			FA,
			<FA as InferableBrand_266801a817966495<'a, Brand, A, C>>::Marker,
		>,
		p: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, C>)
	where
		Brand: Kind_266801a817966495,
		FA: InferableBrand_266801a817966495<'a, Brand, A, C>, {
		f.dispatch(p)
	}

	// -- Explicit dispatch free function --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Maps a function over the first type argument of a bifunctor.
		///
		/// Corresponds to `lmap` in both Haskell and PureScript.
		///
		/// Dispatches to either [`Bifunctor::map_first`] or
		/// [`RefBifunctor::ref_map_first`] based on the closure's argument type.
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by the
		/// compiler. Callers write `map_first::<Brand, _, _, _, _>(...)` and never
		/// need to specify `Marker` or `FA` explicitly.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the bifunctor.",
			"The type of the first value.",
			"The type of the first result.",
			"The type of the second value.",
			"The container type (owned or borrowed), inferred from the argument.",
			"Dispatch marker type, inferred automatically."
		)]
		///
		#[document_parameters(
			"The function to apply to the first value.",
			"The bifunctor value (owned for Val, borrowed for Ref)."
		)]
		///
		#[document_returns("A new bifunctor with the first value transformed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// // Owned
		/// let x = Result::<i32, i32>::Err(5);
		/// let y = map_first::<ResultBrand, _, _, _, _, _>(|e| e * 2, x);
		/// assert_eq!(y, Err(10));
		///
		/// // By-ref
		/// let x = Result::<i32, i32>::Err(5);
		/// let y = map_first::<ResultBrand, _, _, _, _, _>(|e: &i32| *e * 2, &x);
		/// assert_eq!(y, Err(10));
		/// ```
		pub fn map_first<'a, Brand: Kind_266801a817966495, A: 'a, B: 'a, C: 'a, FA, Marker>(
			f: impl MapFirstDispatch<'a, Brand, A, B, C, FA, Marker>,
			p: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, C>) {
			f.dispatch(p)
		}
	}
}

pub use inner::*;
