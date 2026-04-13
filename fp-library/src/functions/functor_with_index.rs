#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::WithIndex,
			dispatch::functor_with_index::MapWithIndexDispatch,
			kinds::*,
		},
		fp_macros::*,
	};

	// -- map_with_index --

	/// Maps a function with index over a functor, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::map_with_index`](crate::functions::explicit::map_with_index) with a turbofish.
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
		f: impl MapWithIndexDispatch<
			'a,
			<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
			A,
			B,
			FA,
			Marker,
		>,
		fa: FA,
	) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FA: InferableBrand_cdc7cd43dac7585f,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand: WithIndex, {
		f.dispatch(fa)
	}
}

pub use inner::*;
