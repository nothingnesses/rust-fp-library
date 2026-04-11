use {
	crate::{
		classes::Monoid,
		dispatch::bifoldable::{
			BiFoldLeftDispatch,
			BiFoldMapDispatch,
			BiFoldRightDispatch,
		},
		kinds::*,
	},
	fp_macros::*,
};

// -- bi_fold_left --

/// Left-folds over a bifoldable structure, inferring the brand from the
/// container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa` via
/// [`InferableBrand`](crate::kinds::InferableBrand_266801a817966495). `FnBrand`
/// must still be specified explicitly.
///
/// For types that need an explicit brand, use
/// [`bi_fold_left_explicit`](crate::functions::bi_fold_left_explicit).
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The cloneable function brand.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the first element.",
	"The type of the second element.",
	"The accumulator type.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"A tuple of (first fold function, second fold function).",
	"The initial accumulator value.",
	"The bifoldable value (owned for Val, borrowed for Ref)."
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
/// let x: Result<i32, i32> = Ok(5);
/// let y = bi_fold_left::<RcFnBrand, _, _, _, _, _>((|acc, e| acc - e, |acc, s| acc + s), 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn bi_fold_left<'a, FnBrand, FA, A: 'a, B: 'a, C: 'a, Marker>(
	fg: impl BiFoldLeftDispatch<
		'a,
		FnBrand,
		<FA as InferableBrand_266801a817966495>::Brand,
		A,
		B,
		C,
		FA,
		Marker,
	>,
	z: C,
	fa: FA,
) -> C
where
	FA: InferableBrand_266801a817966495, {
	fg.dispatch(z, fa)
}

// -- bi_fold_right --

/// Right-folds over a bifoldable structure, inferring the brand from the
/// container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa` via
/// [`InferableBrand`](crate::kinds::InferableBrand_266801a817966495). `FnBrand`
/// must still be specified explicitly.
///
/// For types that need an explicit brand, use
/// [`bi_fold_right_explicit`](crate::functions::bi_fold_right_explicit).
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The cloneable function brand.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the first element.",
	"The type of the second element.",
	"The accumulator type.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"A tuple of (first fold function, second fold function).",
	"The initial accumulator value.",
	"The bifoldable value (owned for Val, borrowed for Ref)."
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
/// let x: Result<i32, i32> = Err(3);
/// let y = bi_fold_right::<RcFnBrand, _, _, _, _, _>((|e, acc| acc - e, |s, acc| acc + s), 10, x);
/// assert_eq!(y, 7);
/// ```
pub fn bi_fold_right<'a, FnBrand, FA, A: 'a, B: 'a, C: 'a, Marker>(
	fg: impl BiFoldRightDispatch<
		'a,
		FnBrand,
		<FA as InferableBrand_266801a817966495>::Brand,
		A,
		B,
		C,
		FA,
		Marker,
	>,
	z: C,
	fa: FA,
) -> C
where
	FA: InferableBrand_266801a817966495, {
	fg.dispatch(z, fa)
}

// -- bi_fold_map --

/// Folds a bifoldable structure into a monoid, inferring the brand from the
/// container type.
///
/// The `Brand` type parameter is inferred from the concrete type of `fa` via
/// [`InferableBrand`](crate::kinds::InferableBrand_266801a817966495). `FnBrand`
/// must still be specified explicitly.
///
/// For types that need an explicit brand, use
/// [`bi_fold_map_explicit`](crate::functions::bi_fold_map_explicit).
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The cloneable function brand.",
	"The container type (owned or borrowed). Brand is inferred from this.",
	"The type of the first element.",
	"The type of the second element.",
	"The monoid type to fold into.",
	"Dispatch marker type, inferred automatically."
)]
///
#[document_parameters(
	"A tuple of (first mapping function, second mapping function).",
	"The bifoldable value (owned for Val, borrowed for Ref)."
)]
///
#[document_returns("The monoidal result of folding.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x: Result<i32, i32> = Ok(5);
/// let y = bi_fold_map::<RcFnBrand, _, _, _, _, _>(
/// 	(|e: &i32| e.to_string(), |s: &i32| s.to_string()),
/// 	&x,
/// );
/// assert_eq!(y, "5".to_string());
/// ```
pub fn bi_fold_map<'a, FnBrand, FA, A: 'a, B: 'a, M: Monoid + 'a, Marker>(
	fg: impl BiFoldMapDispatch<
		'a,
		FnBrand,
		<FA as InferableBrand_266801a817966495>::Brand,
		A,
		B,
		M,
		FA,
		Marker,
	>,
	fa: FA,
) -> M
where
	FA: InferableBrand_266801a817966495, {
	fg.dispatch(fa)
}
