use {
	crate::{
		dispatch::lift::{
			Lift2Dispatch,
			Lift3Dispatch,
			Lift4Dispatch,
			Lift5Dispatch,
		},
		kinds::*,
	},
	fp_macros::*,
};

// -- lift2 --

/// Lifts a binary function into a functor context, inferring the brand
/// from the first container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). The dispatch trait constrains `fb` to the same brand.
///
/// For types with multiple brands, use
/// [`lift2_explicit`](crate::functions::lift2_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The first container type. Brand is inferred from this.",
	"The second container type.",
	"The type of the first value.",
	"The type of the second value.",
	"The type of the result.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The function to lift.",
	"The first context (owned or borrowed).",
	"The second context (owned or borrowed)."
)]
///
#[document_returns("A new context containing the result of applying the function.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// let z = lift2(|a, b| a + b, Some(1), Some(2));
/// assert_eq!(z, Some(3));
/// ```
pub fn lift2<'a, FA, FB, A: 'a, B: 'a, C: 'a, Marker>(
	f: impl Lift2Dispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, B, C, FA, FB, Marker>,
	fa: FA,
	fb: FB,
) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
where
	FA: InferableBrand_cdc7cd43dac7585f, {
	f.dispatch(fa, fb)
}

// -- lift3 --

/// Lifts a ternary function into a functor context, inferring the brand
/// from the first container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). The dispatch trait constrains all other containers
/// to the same brand.
///
/// For types with multiple brands, use
/// [`lift3_explicit`](crate::functions::lift3_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The first container type. Brand is inferred from this.",
	"The second container type.",
	"The third container type.",
	"The type of the first value.",
	"The type of the second value.",
	"The type of the third value.",
	"The type of the result.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The function to lift.",
	"First context (owned or borrowed).",
	"Second context (owned or borrowed).",
	"Third context (owned or borrowed)."
)]
///
#[document_returns("A new context containing the result.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// let r = lift3(|a, b, c| a + b + c, Some(1), Some(2), Some(3));
/// assert_eq!(r, Some(6));
/// ```
pub fn lift3<'a, FA, FB, FC, A: 'a, B: 'a, C: 'a, D: 'a, Marker>(
	f: impl Lift3Dispatch<
		'a,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
		A,
		B,
		C,
		D,
		FA,
		FB,
		FC,
		Marker,
	>,
	fa: FA,
	fb: FB,
	fc: FC,
) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>)
where
	FA: InferableBrand_cdc7cd43dac7585f, {
	f.dispatch(fa, fb, fc)
}

// -- lift4 --

/// Lifts a quaternary function into a functor context, inferring the brand
/// from the first container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). The dispatch trait constrains all other containers
/// to the same brand.
///
/// For types with multiple brands, use
/// [`lift4_explicit`](crate::functions::lift4_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The first container type. Brand is inferred from this.",
	"The second container type.",
	"The third container type.",
	"The fourth container type.",
	"The type of the first value.",
	"The type of the second value.",
	"The type of the third value.",
	"The type of the fourth value.",
	"The type of the result.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The function to lift.",
	"First context (owned or borrowed).",
	"Second context (owned or borrowed).",
	"Third context (owned or borrowed).",
	"Fourth context (owned or borrowed)."
)]
///
#[document_returns("A new context containing the result.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// let r = lift4(|a, b, c, d| a + b + c + d, Some(1), Some(2), Some(3), Some(4));
/// assert_eq!(r, Some(10));
/// ```
pub fn lift4<'a, FA, FB, FC, FD, A: 'a, B: 'a, C: 'a, D: 'a, E: 'a, Marker>(
	f: impl Lift4Dispatch<
		'a,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
		A,
		B,
		C,
		D,
		E,
		FA,
		FB,
		FC,
		FD,
		Marker,
	>,
	fa: FA,
	fb: FB,
	fc: FC,
	fd: FD,
) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>)
where
	FA: InferableBrand_cdc7cd43dac7585f, {
	f.dispatch(fa, fb, fc, fd)
}

// -- lift5 --

/// Lifts a quinary function into a functor context, inferring the brand
/// from the first container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). The dispatch trait constrains all other containers
/// to the same brand.
///
/// For types with multiple brands, use
/// [`lift5_explicit`](crate::functions::lift5_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The first container type. Brand is inferred from this.",
	"The second container type.",
	"The third container type.",
	"The fourth container type.",
	"The fifth container type.",
	"The type of the first value.",
	"The type of the second value.",
	"The type of the third value.",
	"The type of the fourth value.",
	"The type of the fifth value.",
	"The type of the result.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The function to lift.",
	"1st context (owned or borrowed).",
	"2nd context (owned or borrowed).",
	"3rd context (owned or borrowed).",
	"4th context (owned or borrowed).",
	"5th context (owned or borrowed)."
)]
///
#[document_returns("A new context containing the result.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// let r = lift5(|a, b, c, d, e| a + b + c + d + e, Some(1), Some(2), Some(3), Some(4), Some(5));
/// assert_eq!(r, Some(15));
/// ```
pub fn lift5<'a, FA, FB, FC, FD, FE, A: 'a, B: 'a, C: 'a, D: 'a, E: 'a, G: 'a, Marker>(
	f: impl Lift5Dispatch<
		'a,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
		A,
		B,
		C,
		D,
		E,
		G,
		FA,
		FB,
		FC,
		FD,
		FE,
		Marker,
	>,
	fa: FA,
	fb: FB,
	fc: FC,
	fd: FD,
	fe: FE,
) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>)
where
	FA: InferableBrand_cdc7cd43dac7585f, {
	f.dispatch(fa, fb, fc, fd, fe)
}
