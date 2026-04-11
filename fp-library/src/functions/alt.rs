#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			dispatch::alt::AltDispatch,
			kinds::*,
		},
		fp_macros::*,
	};

	// -- alt --

	/// Combines two values in a context, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa1`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`alt_explicit`](crate::functions::alt_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the functor.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The first container (owned or borrowed).",
		"The second container (same ownership as the first)."
	)]
	///
	#[document_returns("A new container from the combination of both inputs.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(alt(None, Some(5)), Some(5));
	///
	/// let x = vec![1, 2];
	/// let y = vec![3, 4];
	/// assert_eq!(alt(&x, &y), vec![1, 2, 3, 4]);
	/// ```
	pub fn alt<'a, FA, A: 'a + Clone, Marker>(
		fa1: FA,
		fa2: FA,
	) -> <<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
	where
		FA: InferableBrand_cdc7cd43dac7585f
			+ AltDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, Marker>, {
		fa1.dispatch(fa2)
	}
}

pub use inner::*;
