use {
	crate::{
		classes::default_brand::DefaultBrand,
		dispatch::apply_first::ApplyFirstDispatch,
		kinds::*,
	},
	fp_macros::*,
};

// -- apply_first --

/// Sequences two applicative actions, keeping the result of the first,
/// inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`apply_first_explicit`](crate::functions::apply_first_explicit) with a turbofish.
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
	"The first container (its values are preserved).",
	"The second container (its values are discarded)."
)]
///
#[document_returns("A container preserving the values from the first input.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// assert_eq!(apply_first(Some(5), Some(10)), Some(5));
///
/// let a = Some(5);
/// let b = Some(10);
/// assert_eq!(apply_first(&a, &b), Some(5));
/// ```
pub fn apply_first<'a, FA, A: 'a, B: 'a, Marker>(
	fa: FA,
	fb: <FA as ApplyFirstDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>>::FB,
) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
	FA: DefaultBrand + ApplyFirstDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>, {
	fa.dispatch(fb)
}
