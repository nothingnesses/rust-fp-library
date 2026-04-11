use {
	crate::{
		classes::{
			WithIndex,
			default_brand::DefaultBrand,
		},
		dispatch::traversable_with_index::TraverseWithIndexDispatch,
		kinds::*,
	},
	fp_macros::*,
};

// -- traverse_with_index --

/// Traverses a structure with an indexed effectful function, inferring the brand
/// from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `ta`
/// via [`DefaultBrand`]. `FnBrand` and `F` (the applicative brand) must
/// still be specified explicitly.
/// Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`traverse_with_index_explicit`](crate::functions::traverse_with_index_explicit()) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the cloneable function to use (must be specified explicitly).",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the elements in the input structure.",
	"The type of the elements in the output structure.",
	"The applicative functor brand (must be specified explicitly).",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The indexed function to apply to each element, returning a value in an applicative context.",
	"The traversable structure (owned for Val, borrowed for Ref)."
)]
///
#[document_returns("The structure wrapped in the applicative context.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let y = traverse_with_index::<RcFnBrand, _, _, _, OptionBrand, _>(
/// 	|_i, x: i32| Some(x * 2),
/// 	vec![1, 2, 3],
/// );
/// assert_eq!(y, Some(vec![2, 4, 6]));
/// ```
pub fn traverse_with_index<'a, FnBrand, FA, A: 'a, B: 'a, F: Kind_cdc7cd43dac7585f, Marker>(
	func: impl TraverseWithIndexDispatch<'a, FnBrand, <FA as DefaultBrand>::Brand, A, B, F, FA, Marker>,
	ta: FA,
) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
where
	FA: DefaultBrand,
	<FA as DefaultBrand>::Brand: WithIndex, {
	func.dispatch(ta)
}
