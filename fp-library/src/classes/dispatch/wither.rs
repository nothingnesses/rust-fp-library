//! Dispatch for [`Witherable::wither`](crate::classes::Witherable::wither) and
//! [`RefWitherable::ref_wither`](crate::classes::RefWitherable::ref_wither).
//!
//! Provides the [`WitherDispatch`] trait and a unified [`wither`] free function
//! that routes to the appropriate trait method based on the closure's argument
//! type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Owned: dispatches to Witherable::wither
//! let y = wither_explicit::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _>(
//! 	|a: i32| Some(if a > 2 { Some(a * 2) } else { None }),
//! 	Some(5),
//! );
//! assert_eq!(y, Some(Some(10)));
//!
//! // By-ref: dispatches to RefWitherable::ref_wither
//! let v = vec![1, 2, 3, 4, 5];
//! let result: Option<Vec<i32>> = wither_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>(
//! 	|x: &i32| if *x > 3 { Some(Some(*x)) } else { Some(None) },
//! 	&v,
//! );
//! assert_eq!(result, Some(vec![4, 5]));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Applicative,
				LiftFn,
				RefWitherable,
				Witherable,
				dispatch::{
					Ref,
					Val,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a wither operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly. The `FA` type parameter is inferred from the container
	/// argument: owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the witherable structure.",
		"The applicative functor brand for the computation.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait WitherDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		M: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched wither operation.
		#[document_signature]
		///
		#[document_parameters("The structure to filter.")]
		///
		#[document_returns("The filtered result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = wither_explicit::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _>(
		/// 	|a: i32| Some(if a > 2 { Some(a * 2) } else { None }),
		/// 	Some(5),
		/// );
		/// assert_eq!(result, Some(Some(10)));
		/// ```
		fn dispatch(
			self,
			ta: FA,
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		>);
	}

	// -- Val: Fn(A) -> M<Option<B>> -> Witherable::wither --

	/// Routes `Fn(A) -> M::Of<Option<B>>` closures to [`Witherable::wither`].
	///
	/// The `FnBrand` parameter is unused by the Val path but is accepted for
	/// uniformity with the Ref path.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand (unused by Val path).",
		"The brand of the witherable structure.",
		"The applicative functor brand.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, FnBrand, Brand, M, A, B, Func>
		WitherDispatch<
			'a,
			FnBrand,
			Brand,
			M,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for Func
	where
		Brand: Witherable,
		A: 'a + Clone,
		B: 'a + Clone,
		M: Applicative,
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters("The structure to filter.")]
		///
		#[document_returns("The filtered result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = wither_explicit::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _>(
		/// 	|a: i32| Some(if a > 2 { Some(a * 2) } else { None }),
		/// 	Some(5),
		/// );
		/// assert_eq!(result, Some(Some(10)));
		/// ```
		fn dispatch(
			self,
			ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		>) {
			Brand::wither::<M, A, B>(self, ta)
		}
	}

	// -- Ref: Fn(&A) -> M<Option<B>> -> RefWitherable::ref_wither --

	/// Routes `Fn(&A) -> M::Of<Option<B>>` closures to [`RefWitherable::ref_wither`].
	///
	/// The `FnBrand` parameter is passed through to the underlying
	/// [`ref_wither`](RefWitherable::ref_wither) call, allowing callers
	/// to choose between [`RcFnBrand`](crate::brands::RcFnBrand) and
	/// [`ArcFnBrand`](crate::brands::ArcFnBrand).
	///
	/// The container must be passed by reference (`&ta`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The cloneable function brand.",
		"The brand of the witherable structure.",
		"The applicative functor brand.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, FnBrand, Brand, M, A, B, Func>
		WitherDispatch<
			'a,
			FnBrand,
			Brand,
			M,
			A,
			B,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for Func
	where
		Brand: RefWitherable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a + Clone,
		M: Applicative,
		Func: Fn(&A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters("A reference to the structure to filter.")]
		///
		#[document_returns("The filtered result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result: Option<Vec<i32>> = wither_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>(
		/// 	|x: &i32| if *x > 3 { Some(Some(*x)) } else { Some(None) },
		/// 	&v,
		/// );
		/// assert_eq!(result, Some(vec![4, 5]));
		/// ```
		fn dispatch(
			self,
			ta: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		>) {
			Brand::ref_wither::<FnBrand, M, A, B>(self, ta)
		}
	}

	// -- Unified free function --

	/// Maps a function over a data structure and filters out None results in an applicative context.
	///
	/// Dispatches to either [`Witherable::wither`] or
	/// [`RefWitherable::ref_wither`] based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(A) -> M::Of<Option<B>>`) and
	///   the container is owned, dispatches to [`Witherable::wither`]. The
	///   `FnBrand` parameter is unused but must be specified for uniformity.
	/// - If the closure takes references (`Fn(&A) -> M::Of<Option<B>>`) and
	///   the container is borrowed (`&ta`), dispatches to
	///   [`RefWitherable::ref_wither`]. The `FnBrand` parameter is passed
	///   through as the function brand.
	///
	/// The `Marker` and `FA` type parameters are inferred automatically by
	/// the compiler from the closure's argument type and the container
	/// argument. Callers write
	/// `wither_explicit::<FnBrand, Brand, M, _, _, _, _>(...)` and never need to
	/// specify `Marker` or `FA` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the witherable structure.",
		"The applicative functor brand for the computation.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning an Option in an applicative context.",
		"The witherable structure (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The filtered structure wrapped in the applicative context.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Owned: dispatches to Witherable::wither
	/// let y = wither_explicit::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _>(
	/// 	|a: i32| Some(if a > 2 { Some(a * 2) } else { None }),
	/// 	Some(5),
	/// );
	/// assert_eq!(y, Some(Some(10)));
	///
	/// // By-ref: dispatches to RefWitherable::ref_wither
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result: Option<Vec<i32>> = wither_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>(
	/// 	|x: &i32| if *x > 3 { Some(Some(*x)) } else { Some(None) },
	/// 	&v,
	/// );
	/// assert_eq!(result, Some(vec![4, 5]));
	/// ```
	pub fn wither<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		M: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		FA,
		Marker,
	>(
		func: impl WitherDispatch<'a, FnBrand, Brand, M, A, B, FA, Marker>,
		ta: FA,
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>) {
		func.dispatch(ta)
	}
}

pub use inner::*;
