#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			dispatch::witherable::{
				WiltDispatch,
				WitherDispatch,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- wilt --

	/// Partitions a structure based on a function returning a Result in an applicative context,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ta`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). `FnBrand` and `M` (the applicative brand) must
	/// still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`wilt_explicit`](crate::functions::wilt_explicit()) with a turbofish.
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
		"Dispatch marker type, inferred automatically."
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
	pub fn wilt<'a, FnBrand, FA, M: Kind_cdc7cd43dac7585f, A: 'a, E: 'a, O: 'a, Marker>(
		func: impl WiltDispatch<
			'a,
			FnBrand,
			<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
			M,
			A,
			E,
			O,
			FA,
			Marker,
		>,
		ta: FA,
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		(
			Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		),
	>)
	where
		FA: InferableBrand_cdc7cd43dac7585f, {
		func.dispatch(ta)
	}

	// -- wither --

	/// Maps a function over a data structure and filters out None results in an applicative context,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ta`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). `FnBrand` and `M` (the applicative brand) must
	/// still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`wither_explicit`](crate::functions::wither_explicit()) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The applicative functor brand (must be specified explicitly).",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"Dispatch marker type, inferred automatically."
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
	pub fn wither<'a, FnBrand, FA, M: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker>(
		func: impl WitherDispatch<
			'a,
			FnBrand,
			<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
			M,
			A,
			B,
			FA,
			Marker,
		>,
		ta: FA,
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
	where
		FA: InferableBrand_cdc7cd43dac7585f, {
		func.dispatch(ta)
	}
}

pub use inner::*;
