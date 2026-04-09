//! Dispatch for [`Functor::map`](crate::classes::Functor::map) and
//! [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map).
//!
//! Provides the [`FunctorDispatch`] trait and a unified [`map`] free function
//! that routes to the appropriate trait method based on the closure's argument
//! type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! // Owned: dispatches to Functor::map
//! let y = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
//! assert_eq!(y, Some(10));
//!
//! // By-ref: dispatches to RefFunctor::ref_map
//! let lazy = RcLazy::pure(10);
//! let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
//! assert_eq!(*mapped.evaluate(), 20);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Functor,
				RefFunctor,
				dispatch::{
					Ref,
					Val,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a map operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FunctorDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// Perform the dispatched map operation.
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val: Fn(A) -> B -> Functor::map --

	/// Routes `Fn(A) -> B` closures to [`Functor::map`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, F> FunctorDispatch<'a, Brand, A, B, Val> for F
	where
		Brand: Functor,
		A: 'a,
		B: 'a,
		F: Fn(A) -> B + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::map(self, fa)
		}
	}

	// -- Ref: Fn(&A) -> B -> RefFunctor::ref_map --

	/// Routes `Fn(&A) -> B` closures to [`RefFunctor::ref_map`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, Brand, A, B, F> FunctorDispatch<'a, Brand, A, B, Ref> for F
	where
		Brand: RefFunctor,
		A: 'a,
		B: 'a,
		F: Fn(&A) -> B + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::pure(10);
		/// let result = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
		/// assert_eq!(*result.evaluate(), 20);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_map(self, &fa)
		}
	}

	// -- Unified free function --

	/// Maps a function over the values in a functor context.
	///
	/// Dispatches to either [`Functor::map`] or [`RefFunctor::ref_map`]
	/// based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(A) -> B`), dispatches to
	///   [`Functor::map`].
	/// - If the closure takes references (`Fn(&A) -> B`), dispatches to
	///   [`RefFunctor::ref_map`].
	///
	/// The `Marker` type parameter is inferred automatically by the compiler
	/// from the closure's argument type. Callers write `map::<Brand, _, _, _>(...)`
	/// and never need to specify `Marker` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s) inside the functor.",
		"The functor instance containing the value(s)."
	)]
	///
	#[document_returns("A new functor instance containing the result(s) of applying the function.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // Owned: dispatches to Functor::map
	/// let y = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
	/// assert_eq!(y, Some(10));
	///
	/// // By-ref: dispatches to RefFunctor::ref_map
	/// let lazy = RcLazy::pure(10);
	/// let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
	/// assert_eq!(*mapped.evaluate(), 20);
	/// ```
	pub fn map<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker>(
		f: impl FunctorDispatch<'a, Brand, A, B, Marker>,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		f.dispatch(fa)
	}
}

pub use inner::*;
