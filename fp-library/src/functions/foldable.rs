use {
	crate::{
		classes::Monoid,
		dispatch::foldable::{
			FoldLeftDispatch,
			FoldMapDispatch,
			FoldRightDispatch,
		},
		kinds::*,
	},
	fp_macros::*,
};

// -- fold_right --

/// Folds a structure from the right, inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). `FnBrand` must still be specified explicitly.
/// Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`fold_right_explicit`](crate::functions::fold_right_explicit) with a turbofish.
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
	"The folding function.",
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
/// let result = fold_right::<RcFnBrand, _, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
/// assert_eq!(result, 6);
/// ```
pub fn fold_right<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Marker>(
	func: impl FoldRightDispatch<
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
	FA: InferableBrand_cdc7cd43dac7585f, {
	func.dispatch(initial, fa)
}

// -- fold_left --

/// Folds a structure from the left, inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). `FnBrand` must still be specified explicitly.
/// Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`fold_left_explicit`](crate::functions::fold_left_explicit) with a turbofish.
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
	"The folding function.",
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
/// let result = fold_left::<RcFnBrand, _, _, _, _>(|b, a| b + a, 0, vec![1, 2, 3]);
/// assert_eq!(result, 6);
/// ```
pub fn fold_left<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Marker>(
	func: impl FoldLeftDispatch<
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
	FA: InferableBrand_cdc7cd43dac7585f, {
	func.dispatch(initial, fa)
}

// -- fold_map --

/// Maps values to a monoid and combines them, inferring the brand from the container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa`
/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). `FnBrand` must still be specified explicitly.
/// Both owned and borrowed containers are supported.
///
/// For types with multiple brands, use
/// [`fold_map_explicit`](crate::functions::fold_map_explicit) with a turbofish.
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
	"The mapping function.",
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
/// let result = fold_map::<RcFnBrand, _, _, _, _>(|a: i32| a.to_string(), vec![1, 2, 3]);
/// assert_eq!(result, "123");
/// ```
pub fn fold_map<'a, FnBrand, FA, A: 'a, M: Monoid + 'a, Marker>(
	func: impl FoldMapDispatch<
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
	FA: InferableBrand_cdc7cd43dac7585f, {
	func.dispatch(fa)
}
