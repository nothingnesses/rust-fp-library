//! Helper functions for working with optics.

use {
	super::{
		base::Optic,
		iso::IsoPrime,
		lens::LensPrime,
		prism::PrismPrime,
	},
	crate::{
		Apply,
		classes::{
			Profunctor,
			UnsizedCoercible,
		},
		kinds::*,
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
};

/// View the focus of a lens-like optic.
///
/// This is a convenience function that works with any lens-based optic.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
///
#[document_parameters("The lens optic.", "The structure to view.")]
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::RcBrand,
/// 	types::optics::*,
/// };
///
/// let l: LensPrime<RcBrand, (i32, String), i32> =
/// 	LensPrime::new(|(x, _)| x, |(_, x)| (x, "".to_string()));
/// assert_eq!(optics_view(&l, (42, "hello".to_string())), 42);
/// ```
pub fn optics_view<'a, P, S, A>(
	optic: &LensPrime<'a, P, S, A>,
	s: S,
) -> A
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	optic.view(s)
}

/// Set the focus of a lens-like optic.
///
/// This is a convenience function that works with any lens-based optic.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
///
#[document_parameters("The lens optic.", "The structure to update.", "The new focus value.")]
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::RcBrand,
/// 	types::optics::*,
/// };
///
/// let l: LensPrime<RcBrand, (i32, String), i32> =
/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
/// assert_eq!(optics_set(&l, (42, "hello".to_string()), 99), (99, "hello".to_string()));
/// ```
pub fn optics_set<'a, P, S, A>(
	optic: &LensPrime<'a, P, S, A>,
	s: S,
	a: A,
) -> S
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	optic.set(s, a)
}

/// Modify the focus of a lens-like optic using a function.
///
/// This is a convenience function that works with any lens-based optic.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus.",
	"The type of the modification function."
)]
///
#[document_parameters(
	"The lens optic.",
	"The structure to update.",
	"The function to apply to the focus."
)]
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::RcBrand,
/// 	types::optics::*,
/// };
///
/// let l: LensPrime<RcBrand, (i32, String), i32> =
/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
/// assert_eq!(optics_over(&l, (42, "hello".to_string()), |x| x * 2), (84, "hello".to_string()));
/// ```
pub fn optics_over<'a, P, S, A, F>(
	optic: &LensPrime<'a, P, S, A>,
	s: S,
	f: F,
) -> S
where
	P: UnsizedCoercible,
	S: 'a + Clone,
	A: 'a,
	F: Fn(A) -> A, {
	optic.over(s, f)
}

/// Preview the focus of a prism-like optic.
///
/// This is a convenience function that works with any prism-based optic.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
///
#[document_parameters("The prism optic.", "The structure to preview.")]
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::RcBrand,
/// 	types::optics::*,
/// };
///
/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
/// assert_eq!(optics_preview(&ok_prism, Ok(42)), Some(42));
/// assert_eq!(optics_preview(&ok_prism, Err("error".to_string())), None);
/// ```
pub fn optics_preview<'a, P, S, A>(
	optic: &PrismPrime<'a, P, S, A>,
	s: S,
) -> Option<A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	optic.preview(s)
}

/// Review a focus value into a structure using a prism-like optic.
///
/// This is a convenience function that works with any prism-based optic.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
///
#[document_parameters("The prism optic.", "The focus value.")]
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::RcBrand,
/// 	types::optics::*,
/// };
///
/// let ok_prism: PrismPrime<RcBrand, Result<i32, String>, i32> =
/// 	PrismPrime::new(|r: Result<i32, String>| r.ok(), |x| Ok(x));
/// assert_eq!(optics_review(&ok_prism, 42), Ok(42));
/// ```
pub fn optics_review<'a, P, S, A>(
	optic: &PrismPrime<'a, P, S, A>,
	a: A,
) -> S
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	optic.review(a)
}

/// Apply an isomorphism in the forward direction.
///
/// This is a convenience function that converts from structure to focus.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
///
#[document_parameters("The iso optic.", "The structure to convert.")]
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::RcBrand,
/// 	types::optics::*,
/// };
///
/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
/// assert_eq!(optics_from(&iso, (42,)), 42);
/// ```
pub fn optics_from<'a, P, S, A>(
	optic: &IsoPrime<'a, P, S, A>,
	s: S,
) -> A
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	optic.from(s)
}

/// Apply an isomorphism in the backward direction.
///
/// This is a convenience function that converts from focus to structure.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
///
#[document_parameters("The iso optic.", "The focus value to convert.")]
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::RcBrand,
/// 	types::optics::*,
/// };
///
/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
/// assert_eq!(optics_to(&iso, 42), (42,));
/// ```
pub fn optics_to<'a, P, S, A>(
	optic: &IsoPrime<'a, P, S, A>,
	a: A,
) -> S
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	optic.to(a)
}

/// Evaluate an optic with a profunctor.
///
/// This is the most general function for working with optics, allowing you to
/// evaluate any optic with any compatible profunctor.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The profunctor type.",
	"The optic type.",
	"The type of the structure.",
	"The target type after update.",
	"The type of the focus.",
	"The target focus type after update."
)]
///
#[document_parameters("The optic.", "The profunctor value.")]
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// 	types::optics::*,
/// };
///
/// let l: LensPrime<RcBrand, (i32, String), i32> =
/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
///
/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
/// let modifier = optics_eval::<RcFnBrand, _, _, _, _, _>(&l, f);
/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
/// ```
pub fn optics_eval<'a, P, O, S: 'a, T: 'a, A: 'a, B: 'a>(
	optic: &O,
	pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>)
where
	P: Profunctor,
	O: Optic<'a, P, S, T, A, B>, {
	optic.evaluate(pab)
}
