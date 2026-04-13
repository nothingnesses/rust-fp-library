#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			dispatch::traversable::TraverseDispatch,
			kinds::*,
		},
		fp_macros::*,
	};

	// -- traverse --

	/// Traverses a structure, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ta`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). `FnBrand` and `F` (the applicative brand) must
	/// still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::traverse`](crate::functions::explicit::traverse) with a turbofish.
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
		"The function to apply to each element, returning a value in an applicative context.",
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
	/// let y = traverse::<RcFnBrand, _, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Some(5));
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	pub fn traverse<'a, FnBrand, FA, A: 'a, B: 'a, F: Kind_cdc7cd43dac7585f, Marker>(
		func: impl TraverseDispatch<
			'a,
			FnBrand,
			<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
			A,
			B,
			F,
			FA,
			Marker,
		>,
		ta: FA,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		FA: InferableBrand_cdc7cd43dac7585f, {
		func.dispatch(ta)
	}
}

pub use inner::*;
