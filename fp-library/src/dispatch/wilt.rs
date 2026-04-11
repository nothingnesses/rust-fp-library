//! Dispatch for [`Witherable::wilt`](crate::classes::Witherable::wilt) and
//! [`RefWitherable::ref_wilt`](crate::classes::RefWitherable::ref_wilt).
//!
//! Provides the [`WiltDispatch`] trait and a unified [`wilt`] free function
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
//! // Owned: dispatches to Witherable::wilt
//! let y = wilt_explicit::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _, _>(
//! 	|a: i32| Some(if a > 2 { Ok(a) } else { Err(a) }),
//! 	Some(5),
//! );
//! assert_eq!(y, Some((None, Some(5))));
//!
//! // By-ref: dispatches to RefWitherable::ref_wilt
//! let v = vec![1, 2, 3, 4, 5];
//! let result: Option<(Vec<i32>, Vec<i32>)> =
//! 	wilt_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _, _>(
//! 		|x: &i32| Some(if *x > 3 { Ok(*x) } else { Err(*x) }),
//! 		&v,
//! 	);
//! assert_eq!(result, Some((vec![1, 2, 3], vec![4, 5])));
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
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a wilt operation to the appropriate type class method.
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
		"The error type.",
		"The success type.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait WiltDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		M: Kind_cdc7cd43dac7585f,
		A: 'a,
		E: 'a,
		O: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched wilt operation.
		#[document_signature]
		///
		#[document_parameters("The structure to partition.")]
		///
		#[document_returns("The partitioned result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = wilt_explicit::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _, _>(
		/// 	|a: i32| Some(if a > 2 { Ok(a) } else { Err(a) }),
		/// 	Some(5),
		/// );
		/// assert_eq!(result, Some((None, Some(5))));
		/// ```
		fn dispatch(
			self,
			ta: FA,
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			(
				Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
				Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
			),
		>);
	}

	// -- Val: Fn(A) -> M<Result<O, E>> -> Witherable::wilt --

	/// Routes `Fn(A) -> M::Of<Result<O, E>>` closures to [`Witherable::wilt`].
	///
	/// The `FnBrand` parameter is unused by the Val path but is accepted for
	/// uniformity with the Ref path.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand (unused by Val path).",
		"The brand of the witherable structure.",
		"The applicative functor brand.",
		"The type of the elements in the input structure.",
		"The error type.",
		"The success type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, FnBrand, Brand, M, A, E, O, Func>
		WiltDispatch<
			'a,
			FnBrand,
			Brand,
			M,
			A,
			E,
			O,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for Func
	where
		Brand: Witherable,
		A: 'a + Clone,
		E: 'a + Clone,
		O: 'a + Clone,
		M: Applicative,
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters("The structure to partition.")]
		///
		#[document_returns("The partitioned result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = wilt_explicit::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _, _>(
		/// 	|a: i32| Some(if a > 2 { Ok(a) } else { Err(a) }),
		/// 	Some(5),
		/// );
		/// assert_eq!(result, Some((None, Some(5))));
		/// ```
		fn dispatch(
			self,
			ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			(
				Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
				Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
			),
		>) {
			Brand::wilt::<M, A, E, O>(self, ta)
		}
	}

	// -- Ref: Fn(&A) -> M<Result<O, E>> -> RefWitherable::ref_wilt --

	/// Routes `Fn(&A) -> M::Of<Result<O, E>>` closures to [`RefWitherable::ref_wilt`].
	///
	/// The `FnBrand` parameter is passed through to the underlying
	/// [`ref_wilt`](RefWitherable::ref_wilt) call, allowing callers
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
		"The error type.",
		"The success type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, FnBrand, Brand, M, A, E, O, Func>
		WiltDispatch<
			'a,
			FnBrand,
			Brand,
			M,
			A,
			E,
			O,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for Func
	where
		Brand: RefWitherable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		E: 'a + Clone,
		O: 'a + Clone,
		M: Applicative,
		Func: Fn(&A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters("A reference to the structure to partition.")]
		///
		#[document_returns("The partitioned result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result: Option<(Vec<i32>, Vec<i32>)> =
		/// 	wilt_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _, _>(
		/// 		|x: &i32| Some(if *x > 3 { Ok(*x) } else { Err(*x) }),
		/// 		&v,
		/// 	);
		/// assert_eq!(result, Some((vec![1, 2, 3], vec![4, 5])));
		/// ```
		fn dispatch(
			self,
			ta: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			(
				Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
				Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
			),
		>) {
			Brand::ref_wilt::<FnBrand, M, A, E, O>(self, ta)
		}
	}

	// -- Unified free function --

	/// Partitions a structure based on a function returning a Result in an applicative context.
	///
	/// Dispatches to either [`Witherable::wilt`] or
	/// [`RefWitherable::ref_wilt`] based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(A) -> M::Of<Result<O, E>>`) and
	///   the container is owned, dispatches to [`Witherable::wilt`]. The
	///   `FnBrand` parameter is unused but must be specified for uniformity.
	/// - If the closure takes references (`Fn(&A) -> M::Of<Result<O, E>>`) and
	///   the container is borrowed (`&ta`), dispatches to
	///   [`RefWitherable::ref_wilt`]. The `FnBrand` parameter is passed
	///   through as the function brand.
	///
	/// The `Marker` and `FA` type parameters are inferred automatically by
	/// the compiler from the closure's argument type and the container
	/// argument. Callers write
	/// `wilt_explicit::<FnBrand, Brand, M, _, _, _, _, _>(...)` and never need to
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
		"The error type.",
		"The success type.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning a Result in an applicative context.",
		"The witherable structure (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The partitioned structure wrapped in the applicative context.")]
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
	/// // Owned: dispatches to Witherable::wilt
	/// let y = wilt_explicit::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _, _>(
	/// 	|a: i32| Some(if a > 2 { Ok(a) } else { Err(a) }),
	/// 	Some(5),
	/// );
	/// assert_eq!(y, Some((None, Some(5))));
	///
	/// // By-ref: dispatches to RefWitherable::ref_wilt
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result: Option<(Vec<i32>, Vec<i32>)> =
	/// 	wilt_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _, _>(
	/// 		|x: &i32| Some(if *x > 3 { Ok(*x) } else { Err(*x) }),
	/// 		&v,
	/// 	);
	/// assert_eq!(result, Some((vec![1, 2, 3], vec![4, 5])));
	/// ```
	pub fn wilt<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		M: Kind_cdc7cd43dac7585f,
		A: 'a,
		E: 'a,
		O: 'a,
		FA,
		Marker,
	>(
		func: impl WiltDispatch<'a, FnBrand, Brand, M, A, E, O, FA, Marker>,
		ta: FA,
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		(
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		),
	>) {
		func.dispatch(ta)
	}
}

pub use inner::*;
