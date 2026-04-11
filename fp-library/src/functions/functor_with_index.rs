use {
	crate::{
		classes::{
			WithIndex,
			default_brand::DefaultBrand,
		},
		dispatch::functor_with_index::MapWithIndexDispatch,
		kinds::*,
	},
	fp_macros::*,
};

// -- map_with_index --

/// Maps a function with index over a functor, inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`map_with_index_explicit`](crate::functions::map_with_index_explicit()) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the value(s) inside the functor.",
	"The type of the result(s) of applying the function.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The function to apply to each value and its index.",
	"The functor instance (owned or borrowed)."
)]
///
#[document_returns("A new functor instance containing the results.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// let y = map_with_index(|i, x: i32| x + i as i32, vec![10, 20, 30]);
/// assert_eq!(y, vec![10, 21, 32]);
/// ```
pub fn map_with_index<'a, FA, A: 'a, B: 'a, Marker>(
	f: impl MapWithIndexDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
	fa: FA,
) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	FA: DefaultBrand,
	<FA as DefaultBrand>::Brand: WithIndex, {
	f.dispatch(fa)
}
