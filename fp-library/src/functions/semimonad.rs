use {
	crate::{
		classes::default_brand::DefaultBrand,
		dispatch::semimonad::{
			BindDispatch,
			JoinDispatch,
		},
		kinds::*,
	},
	fp_macros::*,
};

// -- bind --

/// Sequences a monadic computation, inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `ma`
/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`bind_explicit`](crate::functions::bind_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the value inside the monad.",
	"The type of the result.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The monadic value (owned for Val, borrowed for Ref).",
	"The function to apply to the value."
)]
///
#[document_returns("The result of sequencing the computation.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// let result = bind(Some(5), |x: i32| Some(x * 2));
/// assert_eq!(result, Some(10));
/// ```
pub fn bind<'a, FA, A: 'a, B: 'a, Marker>(
	ma: FA,
	f: impl BindDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	FA: DefaultBrand, {
	f.dispatch(ma)
}

// -- bind_flipped --

/// Sequences a monadic computation (flipped argument order), inferring the brand
/// from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `ma`
/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`bind_flipped_explicit`](crate::functions::bind_flipped_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The input element type.",
	"The output element type.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The function to apply to each element.",
	"The monadic value (owned for Val, borrowed for Ref)."
)]
///
#[document_returns("The result of binding the function over the value.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// let result = bind_flipped(|x: i32| Some(x * 2), Some(5));
/// assert_eq!(result, Some(10));
/// ```
pub fn bind_flipped<'a, FA, A: 'a, B: 'a, Marker>(
	f: impl BindDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
	ma: FA,
) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	FA: DefaultBrand, {
	f.dispatch(ma)
}

// -- join --

/// Removes one layer of monadic nesting, inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `mma`
/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`join_explicit`](crate::functions::join_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the value(s) inside the inner layer.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters("The nested monadic value (owned or borrowed).")]
///
#[document_returns("A container with one layer of nesting removed.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// assert_eq!(join(Some(Some(5))), Some(5));
///
/// let x = Some(Some(5));
/// assert_eq!(join(&x), Some(5));
/// ```
pub fn join<'a, FA, A: 'a, Marker>(
	mma: FA
) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
	FA: DefaultBrand + JoinDispatch<'a, <FA as DefaultBrand>::Brand, A, Marker>, {
	mma.dispatch()
}
