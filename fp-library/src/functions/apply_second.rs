use {
	crate::{
		dispatch::apply_second::ApplySecondDispatch,
		kinds::*,
	},
	fp_macros::*,
};

// -- apply_second --

/// Sequences two applicative actions, keeping the result of the second,
/// inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`apply_second_explicit`](crate::functions::apply_second_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The first container type (owned or borrowed). Brand is inferred from this.",
	"The type of the value(s) inside the first container.",
	"The type of the value(s) inside the second container.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The first container (its values are discarded).",
	"The second container (its values are preserved)."
)]
///
#[document_returns("A container preserving the values from the second input.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// assert_eq!(apply_second(Some(5), Some(10)), Some(10));
///
/// let a = Some(5);
/// let b = Some(10);
/// assert_eq!(apply_second(&a, &b), Some(10));
/// ```
pub fn apply_second<'a, FA, A: 'a, B: 'a, Marker>(
	fa: FA,
	fb: <FA as ApplySecondDispatch<
		'a,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
		A,
		B,
		Marker,
	>>::FB,
) -> <<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
where
	FA: InferableBrand_cdc7cd43dac7585f
		+ ApplySecondDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, B, Marker>, {
	fa.dispatch(fb)
}
