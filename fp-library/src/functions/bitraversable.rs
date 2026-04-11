#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			dispatch::bitraversable::BiTraverseDispatch,
			kinds::*,
		},
		fp_macros::*,
	};

	// -- bi_traverse --

	/// Traverses a bifoldable structure with an applicative effect, inferring
	/// the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa` via
	/// [`InferableBrand`](crate::kinds::InferableBrand_266801a817966495). `FnBrand`
	/// and `F` (the applicative brand) must still be specified explicitly.
	///
	/// For types that need an explicit brand, use
	/// [`bi_traverse_explicit`](crate::functions::bi_traverse_explicit).
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the first element.",
		"The type of the second element.",
		"The type of the first result.",
		"The type of the second result.",
		"The applicative effect brand.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"A tuple of (first traversal function, second traversal function).",
		"The bitraversable value (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The applicative effect containing the traversed bifunctor.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = bi_traverse::<RcFnBrand, _, _, _, _, _, OptionBrand, _>(
	/// 	(|e: i32| Some(e + 1), |s: i32| Some(s * 2)),
	/// 	x,
	/// );
	/// assert_eq!(y, Some(Ok(10)));
	/// ```
	pub fn bi_traverse<
		'a,
		FnBrand,
		FA,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		F: Kind_cdc7cd43dac7585f,
		Marker,
	>(
		fg: impl BiTraverseDispatch<
			'a,
			FnBrand,
			<FA as InferableBrand_266801a817966495>::Brand,
			A,
			B,
			C,
			D,
			F,
			FA,
			Marker,
		>,
		fa: FA,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a, B: 'a>: 'a;)>::Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
	where
		FA: InferableBrand_266801a817966495, {
		fg.dispatch(fa)
	}
}

pub use inner::*;
