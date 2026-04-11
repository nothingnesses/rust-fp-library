use {
	crate::{
		dispatch::functor::FunctorDispatch,
		kinds::*,
	},
	fp_macros::*,
};

// -- map --

/// Maps a function over a functor, inferring the brand from the container type.
///
/// This is the primary API for mapping. The `Brand` type parameter is
/// inferred from the concrete type of `fa` via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both
/// owned and borrowed containers are supported:
///
/// - Owned: `map(|x: i32| x + 1, Some(5))` infers `OptionBrand`.
/// - Borrowed: `map(|x: &i32| *x + 1, &Some(5))` infers `OptionBrand`
///   via the blanket `impl InferableBrand for &T`.
///
/// For types with multiple brands (e.g., `Result`), use
/// [`map_explicit`](crate::functions::map_explicit) with a turbofish.
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
	"The function to apply to the value(s).",
	"The functor instance (owned or borrowed)."
)]
///
#[document_returns("A new functor instance containing the result(s) of applying the function.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// // Brand inferred from Option<i32>
/// assert_eq!(map(|x: i32| x * 2, Some(5)), Some(10));
///
/// // Brand inferred from &Vec<i32> via blanket impl
/// let v = vec![1, 2, 3];
/// assert_eq!(map(|x: &i32| *x + 10, &v), vec![11, 12, 13]);
/// ```
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
	f: impl FunctorDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, B, FA, Marker>,
	fa: FA,
) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	FA: InferableBrand_cdc7cd43dac7585f, {
	f.dispatch(fa)
}
