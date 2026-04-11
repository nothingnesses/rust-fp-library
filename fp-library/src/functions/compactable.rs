use {
	crate::{
		classes::default_brand::DefaultBrand,
		dispatch::compactable::{
			CompactDispatch,
			SeparateDispatch,
		},
		kinds::*,
	},
	fp_macros::*,
};

// -- compact --

/// Removes `None` values from a container of `Option`s, inferring the brand
/// from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`compact_explicit`](crate::functions::compact_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the value(s) inside the `Option` wrappers.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters("The container of `Option` values (owned or borrowed).")]
///
#[document_returns("A new container with `None` values removed and `Some` values unwrapped.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// assert_eq!(compact(vec![Some(1), None, Some(3)]), vec![1, 3]);
///
/// let v = vec![Some(1), None, Some(3)];
/// assert_eq!(compact(&v), vec![1, 3]);
/// ```
pub fn compact<'a, FA, A: 'a, Marker>(
	fa: FA
) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
	FA: DefaultBrand + CompactDispatch<'a, <FA as DefaultBrand>::Brand, A, Marker>, {
	fa.dispatch_compact()
}

// -- separate --

/// Separates a container of `Result` values into two containers, inferring
/// the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`separate_explicit`](crate::functions::separate_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The error type inside the `Result` wrappers.",
	"The success type inside the `Result` wrappers.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters("The container of `Result` values (owned or borrowed).")]
///
#[document_returns("A tuple of two containers: `Err` values and `Ok` values.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// let (errs, oks) = separate(vec![Ok(1), Err(2), Ok(3)]);
/// assert_eq!(oks, vec![1, 3]);
/// assert_eq!(errs, vec![2]);
/// ```
pub fn separate<'a, FA, E: 'a, O: 'a, Marker>(
	fa: FA
) -> (
	<<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, E>,
	<<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, O>,
)
where
	FA: DefaultBrand + SeparateDispatch<'a, <FA as DefaultBrand>::Brand, E, O, Marker>, {
	fa.dispatch_separate()
}
