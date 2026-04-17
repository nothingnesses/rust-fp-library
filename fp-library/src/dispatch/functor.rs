//! Dispatch for [`Functor::map`](crate::classes::Functor::map) and
//! [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map).
//!
//! Provides the [`FunctorDispatch`] trait and a unified [`explicit::map`] free
//! function that routes to the appropriate trait method based on the closure's
//! argument type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! 	types::*,
//! };
//!
//! // Owned: dispatches to Functor::map
//! let y = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
//! assert_eq!(y, Some(10));
//!
//! // By-ref: dispatches to RefFunctor::ref_map
//! let lazy = RcLazy::pure(10);
//! let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, &lazy);
//! assert_eq!(*mapped.evaluate(), 20);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Functor,
				RefFunctor,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a map operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly. The `FA` type parameter is inferred from the container
	/// argument: owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FunctorDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker> {
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
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: FA,
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
	impl<'a, Brand, A, B, F>
		FunctorDispatch<
			'a,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
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
		/// 	functions::explicit::*,
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
	///
	/// The container must be passed by reference (`&fa`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, B, F>
		FunctorDispatch<
			'a,
			Brand,
			A,
			B,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFunctor,
		A: 'a,
		B: 'a,
		F: Fn(&A) -> B + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("A reference to the functor instance.")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::pure(10);
		/// let result = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, &lazy);
		/// assert_eq!(*result.evaluate(), 20);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_map(self, fa)
		}
	}

	// -- Inference wrapper --

	/// Maps a function over a functor, inferring the brand from the container type.
	///
	/// This is the primary API for mapping. The `Brand` type parameter is
	/// inferred from the concrete type of `fa` via the `Slot` trait. Both
	/// owned and borrowed containers are supported:
	///
	/// - Owned: `map(|x: i32| x + 1, Some(5))` infers `OptionBrand`.
	/// - Borrowed: `map(|x: &i32| *x + 1, &Some(5))` infers `OptionBrand`
	///   via the blanket `impl Slot for &T`.
	///
	/// For multi-brand types (e.g., `Result`), the closure's input type
	/// disambiguates which brand applies:
	///
	/// - `map(|x: i32| x + 1, Ok::<i32, String>(5))` infers
	///   `ResultErrAppliedBrand<String>` (maps over Ok).
	/// - `map(|e: String| e.len(), Err::<i32, String>("hi".into()))` infers
	///   `ResultOkAppliedBrand<i32>` (maps over Err).
	///
	/// For diagonal cases where the closure cannot disambiguate (e.g.,
	/// `Result<T, T>`), use [`explicit::map`](crate::functions::explicit::map)
	/// with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The brand, inferred via Slot from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s).",
		"The functor instance (owned or borrowed)."
	)]
	///
	#[document_returns("A new functor instance containing the result(s) of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// // Brand inferred from Option<i32>
	/// assert_eq!(map(|x: i32| x * 2, Some(5)), Some(10));
	///
	/// // Brand inferred from &Vec<i32> via blanket impl
	/// let v = vec![1, 2, 3];
	/// assert_eq!(map(|x: &i32| *x + 10, &v), vec![11, 12, 13]);
	/// ```
	pub fn map<'a, FA, A: 'a, B: 'a, Brand>(
		f: impl FunctorDispatch<
			'a,
			Brand,
			A,
			B,
			FA,
			<FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	// -- Explicit dispatch free function --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Maps a function over the values in a functor context.
		///
		/// Dispatches to either [`Functor::map`] or [`RefFunctor::ref_map`]
		/// based on the closure's argument type:
		///
		/// - If the closure takes owned values (`Fn(A) -> B`) and the container is
		///   owned, dispatches to [`Functor::map`].
		/// - If the closure takes references (`Fn(&A) -> B`) and the container is
		///   borrowed (`&fa`), dispatches to [`RefFunctor::ref_map`].
		///
		/// The `FA` type parameter is inferred automatically by the compiler from
		/// the container argument. The `Marker` is projected from the Slot trait.
		/// Callers write `map::<Brand, _, _, _>(...)` and never need to specify
		/// `FA` explicitly.
		///
		/// The dispatch is resolved at compile time with no runtime cost.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the functor.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function.",
			"The container type (owned or borrowed), inferred from the argument."
		)]
		///
		#[document_parameters(
			"The function to apply to the value(s) inside the functor.",
			"The functor instance (owned for Val, borrowed for Ref)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// // Owned: dispatches to Functor::map
		/// let y = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		/// assert_eq!(y, Some(10));
		///
		/// // By-ref: dispatches to RefFunctor::ref_map
		/// let lazy = RcLazy::pure(10);
		/// let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, &lazy);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		pub fn map<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA>(
			f: impl FunctorDispatch<
				'a,
				Brand,
				A,
				B,
				FA,
				<FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
			>,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
			f.dispatch(fa)
		}
	}
}

pub use inner::*;
