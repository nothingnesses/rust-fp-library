use {
	crate::{
		dispatch::bifunctor::BimapDispatch,
		kinds::*,
	},
	fp_macros::*,
};

// -- bimap --

/// Maps two functions over the values in a bifunctor context, inferring the
/// brand from the container type.
///
/// This is the primary API for bimapping. The `Brand` type parameter is
/// inferred from the concrete type of `p` via
/// [`InferableBrand`](crate::kinds::InferableBrand_266801a817966495). Both
/// owned and borrowed containers are supported.
///
/// For types that need an explicit brand, use
/// [`bimap_explicit`](crate::functions::bimap_explicit) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the first value.",
	"The type of the first result.",
	"The type of the second value.",
	"The type of the second result.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"A tuple of (first function, second function).",
	"The bifunctor value (owned for Val, borrowed for Ref)."
)]
///
#[document_returns("A new bifunctor instance containing the results of applying the functions.")]
#[document_examples]
///
/// ```
/// use fp_library::functions::*;
///
/// // Brand inferred from Result<i32, i32>
/// let x = Result::<i32, i32>::Ok(5);
/// let y = bimap((|e| e + 1, |s| s * 2), x);
/// assert_eq!(y, Ok(10));
///
/// // Brand inferred from &Result<i32, i32> via blanket impl
/// let x = Result::<i32, i32>::Ok(5);
/// let y = bimap((|e: &i32| *e + 1, |s: &i32| *s * 2), &x);
/// assert_eq!(y, Ok(10));
/// ```
pub fn bimap<'a, FA, A: 'a, B: 'a, C: 'a, D: 'a, Marker>(
	fg: impl BimapDispatch<'a, <FA as InferableBrand_266801a817966495>::Brand, A, B, C, D, FA, Marker>,
	p: FA,
) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a, B: 'a>: 'a;)>::Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
where
	FA: InferableBrand_266801a817966495, {
	fg.dispatch(p)
}
