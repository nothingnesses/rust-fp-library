//! Dispatch for [`Bifunctor::bimap`](crate::classes::Bifunctor::bimap) and
//! [`RefBifunctor::ref_bimap`](crate::classes::RefBifunctor::ref_bimap).
//!
//! Provides the [`BimapDispatch`] trait and a unified [`explicit::bimap`] free
//! function that routes to the appropriate trait method based on the closures'
//! argument types.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! // Owned: dispatches to Bifunctor::bimap
//! let x = Result::<i32, i32>::Ok(5);
//! let y = bimap::<ResultBrand, _, _, _, _, _, _>((|e| e + 1, |s| s * 2), x);
//! assert_eq!(y, Ok(10));
//!
//! // By-ref: dispatches to RefBifunctor::ref_bimap
//! let x = Result::<i32, i32>::Ok(5);
//! let y = bimap::<ResultBrand, _, _, _, _, _, _>((|e: &i32| *e + 1, |s: &i32| *s * 2), &x);
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

	/// Trait that routes a bimap operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is inferred from the closures' argument types:
	/// `(Fn(A) -> B, Fn(C) -> D)` resolves to [`Val`](crate::dispatch::Val),
	/// `(Fn(&A) -> B, Fn(&C) -> D)` resolves to [`Ref`](crate::dispatch::Ref).
	/// The `FA` type parameter is inferred from the container argument: owned
	/// for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the bifunctor.",
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value.",
		"The type of the second result.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure tuple implementing this dispatch.")]
	pub trait BimapDispatch<
		'a,
		Brand: Kind_266801a817966495,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched bimap operation.
		#[document_signature]
		#[document_parameters("The bifunctor value.")]
		#[document_returns("The result of bimapping.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result =
		/// 	bimap::<ResultBrand, _, _, _, _, _, _>((|e| e + 1, |s: i32| s * 2), Ok::<i32, i32>(5));
		/// assert_eq!(result, Ok(10));
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>);
	}

	/// Routes `(Fn(A) -> B, Fn(C) -> D)` closure tuples to [`Bifunctor::bimap`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The first input type.",
		"The first output type.",
		"The second input type.",
		"The second output type.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure tuple.")]
	impl<'a, Brand, A, B, C, D, F, G>
		BimapDispatch<
			'a,
			Brand,
			A,
			B,
			C,
			D,
			Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
			Val,
		> for (F, G)
	where
		Brand: Bifunctor,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		F: Fn(A) -> B + 'a,
		G: Fn(C) -> D + 'a,
	{
		#[document_signature]
		#[document_parameters("The bifunctor value.")]
		#[document_returns("The result of bimapping.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result =
		/// 	bimap::<ResultBrand, _, _, _, _, _, _>((|e| e + 1, |s: i32| s * 2), Ok::<i32, i32>(5));
		/// assert_eq!(result, Ok(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			Brand::bimap(self.0, self.1, fa)
		}
	}

	/// Routes `(Fn(&A) -> B, Fn(&C) -> D)` closure tuples to [`RefBifunctor::ref_bimap`].
	///
	/// The container must be passed by reference (`&p`).
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The brand.",
		"The first input type.",
		"The first output type.",
		"The second input type.",
		"The second output type.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure tuple.")]
	impl<'a, 'b, Brand, A, B, C, D, F, G>
		BimapDispatch<
			'a,
			Brand,
			A,
			B,
			C,
			D,
			&'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
			Ref,
		> for (F, G)
	where
		Brand: RefBifunctor,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		F: Fn(&A) -> B + 'a,
		G: Fn(&C) -> D + 'a,
	{
		#[document_signature]
		#[document_parameters("A reference to the bifunctor value.")]
		#[document_returns("The result of bimapping.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let x = Result::<i32, i32>::Ok(5);
		/// let result = bimap::<ResultBrand, _, _, _, _, _, _>((|e: &i32| *e + 1, |s: &i32| *s * 2), &x);
		/// assert_eq!(result, Ok(10));
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			Brand::ref_bimap(self.0, self.1, fa)
		}
	}

	// -- Inference wrapper --

	/// Maps two functions over the values in a bifunctor context, inferring the
	/// brand from the container type.
	///
	/// This is the primary API for bimapping. The `Brand` type parameter is
	/// inferred from the concrete type of `p` via the `Slot` trait. Both
	/// owned and borrowed containers are supported.
	///
	/// For types that need an explicit brand, use
	/// [`explicit::bimap`](crate::functions::explicit::bimap) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value.",
		"The type of the second result.",
		"The brand, inferred via Slot from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"A tuple of (first function, second function).",
		"The bifunctor value (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns(
		"A new bifunctor instance containing the results of applying the functions."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// // Brand inferred from Result<i32, i32>
	/// let x = Result::<i32, i32>::Ok(5);
	/// let y = bimap((|e| e + 1, |s| s * 2), x);
	/// assert_eq!(y, Ok(10));
	///
	/// // Brand inferred from &Result<i32, i32> via blanket impl
	/// let x = Result::<i32, i32>::Ok(5);
	/// let y = bimap((|e: &i32| *e + 1, |s: &i32| *s * 2), &x);
	/// assert_eq!(y, Ok(10));
	/// ```
	pub fn bimap<'a, FA, A: 'a, B: 'a, C: 'a, D: 'a, Brand>(
		fg: impl BimapDispatch<
			'a,
			Brand,
			A,
			B,
			C,
			D,
			FA,
			<FA as Slot_266801a817966495<'a, Brand, A, C>>::Marker,
		>,
		p: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
	where
		Brand: Kind_266801a817966495,
		FA: Slot_266801a817966495<'a, Brand, A, C>, {
		fg.dispatch(p)
	}

	// -- Explicit dispatch free function --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Maps two functions over the values in a bifunctor context.
		///
		/// Dispatches to either [`Bifunctor::bimap`] or [`RefBifunctor::ref_bimap`]
		/// based on the closures' argument types.
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by the
		/// compiler from the closures' argument types and the container argument.
		/// Callers write `bimap::<Brand, _, _, _, _, _, _>(...)` and never need to
		/// specify `Marker` or `FA` explicitly.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the bifunctor.",
			"The type of the first value.",
			"The type of the first result.",
			"The type of the second value.",
			"The type of the second result.",
			"The container type (owned or borrowed), inferred from the argument.",
			"Dispatch marker type, inferred automatically."
		)]
		///
		#[document_parameters(
			"A tuple of (first function, second function).",
			"The bifunctor value (owned for Val, borrowed for Ref)."
		)]
		///
		#[document_returns(
			"A new bifunctor instance containing the results of applying the functions."
		)]
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
		/// let y = bimap::<ResultBrand, _, _, _, _, _, _>((|e| e + 1, |s| s * 2), x);
		/// assert_eq!(y, Ok(10));
		///
		/// // By-ref
		/// let x = Result::<i32, i32>::Ok(5);
		/// let y = bimap::<ResultBrand, _, _, _, _, _, _>((|e: &i32| *e + 1, |s: &i32| *s * 2), &x);
		/// assert_eq!(y, Ok(10));
		/// ```
		pub fn bimap<'a, Brand: Kind_266801a817966495, A: 'a, B: 'a, C: 'a, D: 'a, FA, Marker>(
			fg: impl BimapDispatch<'a, Brand, A, B, C, D, FA, Marker>,
			p: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			fg.dispatch(p)
		}
	}
}

pub use inner::*;
