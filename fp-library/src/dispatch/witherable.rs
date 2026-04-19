//! Dispatch for witherable operations:
//! [`Witherable`](crate::classes::Witherable) and
//! [`RefWitherable`](crate::classes::RefWitherable).
//!
//! Provides the following dispatch traits and unified free functions:
//!
//! - [`WiltDispatch`] + [`explicit::wilt`]
//! - [`WitherDispatch`] + [`explicit::wither`]
//!
//! Each routes to the appropriate trait method based on the closure's argument
//! type.
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
//! // wilt
//! let y = wilt::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _, _>(
//! 	|a: i32| Some(if a > 2 { Ok(a) } else { Err(a) }),
//! 	Some(5),
//! );
//! assert_eq!(y, Some((None, Some(5))));
//!
//! // wither
//! let y = wither::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _>(
//! 	|a: i32| Some(if a > 2 { Some(a * 2) } else { None }),
//! 	Some(5),
//! );
//! assert_eq!(y, Some(Some(10)));
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

	// -- WiltDispatch --

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
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// let result = wilt::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _, _>(
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

	/// Routes `Fn(A) -> M::Of<Result<O, E>>` closures to [`Witherable::wilt`].
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
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// let result = wilt::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _, _>(
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

	/// Routes `Fn(&A) -> M::Of<Result<O, E>>` closures to [`RefWitherable::ref_wilt`].
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
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result: Option<(Vec<i32>, Vec<i32>)> =
		/// 	wilt::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _, _>(
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

	// -- WitherDispatch --

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
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = wither::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _>(
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

	/// Routes `Fn(A) -> M::Of<Option<B>>` closures to [`Witherable::wither`].
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = wither::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _>(
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

	/// Routes `Fn(&A) -> M::Of<Option<B>>` closures to [`RefWitherable::ref_wither`].
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result: Option<Vec<i32>> = wither::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>(
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

	// -- Inference wrappers --

	/// Partitions a structure based on a function returning a Result in an applicative context,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ta`
	/// via the `InferableBrand` trait. `FnBrand` and `M` (the applicative brand) must
	/// still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::wilt`](crate::functions::explicit::wilt) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The applicative functor brand (must be specified explicitly).",
		"The type of the elements in the input structure.",
		"The error type.",
		"The success type.",
		"The brand, inferred via InferableBrand from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning a Result in an applicative context.",
		"The witherable structure (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The partitioned structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let y = wilt::<RcFnBrand, _, OptionBrand, _, _, _, _>(
	/// 	|a: i32| Some(if a > 2 { Ok(a) } else { Err(a) }),
	/// 	Some(5),
	/// );
	/// assert_eq!(y, Some((None, Some(5))));
	/// ```
	pub fn wilt<'a, FnBrand, FA, M: Kind_cdc7cd43dac7585f, A: 'a, E: 'a, O: 'a, Brand>(
		func: impl WiltDispatch<
			'a,
			FnBrand,
			Brand,
			M,
			A,
			E,
			O,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		ta: FA,
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		(
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		),
	>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		func.dispatch(ta)
	}

	/// Maps a function over a data structure and filters out None results in an applicative context,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ta`
	/// via the `InferableBrand` trait. `FnBrand` and `M` (the applicative brand) must
	/// still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::wither`](crate::functions::explicit::wither) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The applicative functor brand (must be specified explicitly).",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The brand, inferred via InferableBrand from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning an Option in an applicative context.",
		"The witherable structure (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The filtered structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let y = wither::<RcFnBrand, _, OptionBrand, _, _, _>(
	/// 	|a: i32| Some(if a > 2 { Some(a * 2) } else { None }),
	/// 	Some(5),
	/// );
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	pub fn wither<'a, FnBrand, FA, M: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Brand>(
		func: impl WitherDispatch<
			'a,
			FnBrand,
			Brand,
			M,
			A,
			B,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		ta: FA,
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		func.dispatch(ta)
	}

	// -- Explicit dispatch free functions --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Partitions a structure based on a function returning a Result in an applicative context.
		///
		/// Dispatches to either [`Witherable::wilt`] or
		/// [`RefWitherable::ref_wilt`] based on the closure's argument type.
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
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// let y = wilt::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _, _>(
		/// 	|a: i32| Some(if a > 2 { Ok(a) } else { Err(a) }),
		/// 	Some(5),
		/// );
		/// assert_eq!(y, Some((None, Some(5))));
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

		/// Maps a function over a data structure and filters out None results in an applicative context.
		///
		/// Dispatches to either [`Witherable::wither`] or
		/// [`RefWitherable::ref_wither`] based on the closure's argument type.
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let y = wither::<RcFnBrand, OptionBrand, OptionBrand, _, _, _, _>(
		/// 	|a: i32| Some(if a > 2 { Some(a * 2) } else { None }),
		/// 	Some(5),
		/// );
		/// assert_eq!(y, Some(Some(10)));
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
}

pub use inner::*;
