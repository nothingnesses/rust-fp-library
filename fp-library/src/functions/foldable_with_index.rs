use {
	crate::{
		classes::{
			Monoid,
			WithIndex,
		},
		dispatch::foldable_with_index::{
			FoldLeftWithIndexDispatch,
			FoldMapWithIndexDispatch,
			FoldRightWithIndexDispatch,
		},
		kinds::*,
	},
	fp_macros::*,
};

// -- fold_map_with_index --

/// Maps values with their index to a monoid and combines them, inferring the brand
/// from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). `FnBrand` must still be specified explicitly.
/// Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`fold_map_with_index_explicit`](crate::functions::fold_map_with_index_explicit()) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the cloneable function to use (must be specified explicitly).",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the elements.",
	"The monoid type.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The mapping function that receives an index and element.",
	"The structure to fold (owned for Val, borrowed for Ref)."
)]
///
#[document_returns("The combined monoid value.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let result = fold_map_with_index::<RcFnBrand, _, _, _, _>(
/// 	|i, x: i32| format!("{i}:{x}"),
/// 	vec![10, 20, 30],
/// );
/// assert_eq!(result, "0:101:202:30");
/// ```
pub fn fold_map_with_index<'a, FnBrand, FA, A: 'a, M: Monoid + 'a, Marker>(
	func: impl FoldMapWithIndexDispatch<
		'a,
		FnBrand,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
		A,
		M,
		FA,
		Marker,
	>,
	fa: FA,
) -> M
where
	FA: InferableBrand_cdc7cd43dac7585f,
	<FA as InferableBrand_cdc7cd43dac7585f>::Brand: WithIndex, {
	func.dispatch(fa)
}

// -- fold_right_with_index --

/// Folds a structure from the right with index, inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). `FnBrand` must still be specified explicitly.
/// Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`fold_right_with_index_explicit`](crate::functions::fold_right_with_index_explicit()) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the cloneable function to use (must be specified explicitly).",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the elements.",
	"The type of the accumulator.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The folding function that receives an index, element, and accumulator.",
	"The initial accumulator value.",
	"The structure to fold (owned for Val, borrowed for Ref)."
)]
///
#[document_returns("The final accumulator value.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let result = fold_right_with_index::<RcFnBrand, _, _, _, _>(
/// 	|i, x: i32, acc: String| format!("{acc}{i}:{x},"),
/// 	String::new(),
/// 	vec![10, 20, 30],
/// );
/// assert_eq!(result, "2:30,1:20,0:10,");
/// ```
pub fn fold_right_with_index<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Marker>(
	func: impl FoldRightWithIndexDispatch<
		'a,
		FnBrand,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
		A,
		B,
		FA,
		Marker,
	>,
	initial: B,
	fa: FA,
) -> B
where
	FA: InferableBrand_cdc7cd43dac7585f,
	<FA as InferableBrand_cdc7cd43dac7585f>::Brand: WithIndex, {
	func.dispatch(initial, fa)
}

// -- fold_left_with_index --

/// Folds a structure from the left with index, inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). `FnBrand` must still be specified explicitly.
/// Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`fold_left_with_index_explicit`](crate::functions::fold_left_with_index_explicit()) with a turbofish.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the cloneable function to use (must be specified explicitly).",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the elements.",
	"The type of the accumulator.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"The folding function that receives an index, accumulator, and element.",
	"The initial accumulator value.",
	"The structure to fold (owned for Val, borrowed for Ref)."
)]
///
#[document_returns("The final accumulator value.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let result = fold_left_with_index::<RcFnBrand, _, _, _, _>(
/// 	|i, acc: String, x: i32| format!("{acc}{i}:{x},"),
/// 	String::new(),
/// 	vec![10, 20, 30],
/// );
/// assert_eq!(result, "0:10,1:20,2:30,");
/// ```
pub fn fold_left_with_index<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Marker>(
	func: impl FoldLeftWithIndexDispatch<
		'a,
		FnBrand,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
		A,
		B,
		FA,
		Marker,
	>,
	initial: B,
	fa: FA,
) -> B
where
	FA: InferableBrand_cdc7cd43dac7585f,
	<FA as InferableBrand_cdc7cd43dac7585f>::Brand: WithIndex, {
	func.dispatch(initial, fa)
}
