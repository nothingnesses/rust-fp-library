// Brand-as-associated-type-projection POC (negative result).
//
// -- Background --
//
// The library's production `map` uses `<FA as InferableBrand>::Brand`
// (an associated-type projection) to commit the Brand from the
// container alone, before FunctorDispatch's Val/Ref selection begins.
// This only works for single-brand types because InferableBrand
// requires a unique Brand per container type. Multi-brand types have
// no InferableBrand impl.
//
// -- Hypothesis --
//
// Replace InferableBrand's single-valued `type Brand` with a trait
// `SelectBrand<'a, A>` keyed on BOTH (FA, A) - where A comes from the
// closure's Fn signature. Multi-brand types get one impl per brand,
// each with A in a different position. Brand is an associated type, so
// once the closure pins A, Rust projects the unique Brand before
// FunctorDispatch is considered.
//
// ========================================================================
// FINDING: HYPOTHESIS REJECTED.
// ========================================================================
//
// Coherence (E0119) rejects the two Result impls of `SelectBrand<'a, A>`:
//
//     impl<'a, A, E> SelectBrand<'a, A> for Result<A, E>   // trait arg: A
//     impl<'a, T, A> SelectBrand<'a, A> for Result<T, A>   // trait arg: A
//
// Both impls have trait-argument pattern `SelectBrand<'_, A>` for the
// same Self-type shape `Result<_, _>`. Coherence considers these
// overlapping because the trait arguments are structurally identical
// (both have a single type variable `A`); the distinguishing
// information (which Result slot `A` names) lives only in the
// Self-type, and coherence checks impl heads globally without caring
// about that positional detail.
//
// Why `InferableBrand<Brand, A>` doesn't hit this: InferableBrand has Brand as a trait
// PARAMETER. The two InferableBrand impls have:
//
//     impl InferableBrand<ResultErrAppliedBrand<E>, A> for Result<A, E>
//     impl InferableBrand<ResultOkAppliedBrand<T>, A> for Result<T, A>
//
// Coherence sees different Brand values (`ResultErrAppliedBrand<_>` vs
// `ResultOkAppliedBrand<_>`) as structurally distinct trait-argument
// patterns and accepts the impls.
//
// General pattern: on stable Rust, an associated-type projection
// requires a UNIQUE impl per (Self, trait-args) combination. Multi-brand
// types need multiple impls with the same (Self-shape, trait-arg-shape),
// which is only possible if a trait-parameter (like Brand) structurally
// distinguishes them. An associated type cannot play this role.
//
// NOTE: the inference concern this POC attempted to address (eagerly
// committing Brand before FunctorDispatch) was later resolved by a
// different approach (`slot_marker_via_slot_poc.rs`): lifting Marker
// into InferableBrand as an associated type, where InferableBrand's Brand parameter
// provides the structural distinction coherence needs. The Marker
// projection commits from FA's reference-ness before Brand resolves.
//
// Kept as a documentation artifact. The second multi-brand impl is
// commented out to keep the file in a buildable state.

#![allow(unused_imports, reason = "Kind is used inside Apply! macro expansion")]

use {
	fp_library::{
		brands::{
			LazyBrand,
			OptionBrand,
			ResultErrAppliedBrand,
			ResultOkAppliedBrand,
			VecBrand,
		},
		dispatch::functor::FunctorDispatch,
		kinds::Kind_cdc7cd43dac7585f,
		types::{
			Lazy,
			RcLazy,
			lazy::LazyConfig,
		},
	},
	fp_macros::{
		Apply,
		Kind,
	},
};

// -------------------------------------------------------------------------
// SelectBrand trait: associated-type projection from (FA, A) -> Brand.
// -------------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub trait SelectBrand_cdc7cd43dac7585f<'a, A: 'a> {
	type Brand: Kind_cdc7cd43dac7585f;
}

// Single-brand: Brand is the canonical brand for the container.
impl<'a, A: 'a> SelectBrand_cdc7cd43dac7585f<'a, A> for Option<A> {
	type Brand = OptionBrand;
}

impl<'a, A: 'a> SelectBrand_cdc7cd43dac7585f<'a, A> for Vec<A> {
	type Brand = VecBrand;
}

impl<'a, A: 'a, Config: LazyConfig> SelectBrand_cdc7cd43dac7585f<'a, A> for Lazy<'a, A, Config> {
	type Brand = LazyBrand<Config>;
}

// Multi-brand: one impl per brand, keyed on which Result slot is free.
// Only the first impl is uncommented - adding the second triggers E0119
// (conflicting implementations) on stable rustc.
impl<'a, A: 'a, E: 'static> SelectBrand_cdc7cd43dac7585f<'a, A> for Result<A, E> {
	type Brand = ResultErrAppliedBrand<E>;
}

// impl<'a, T: 'static, A: 'a> SelectBrand_cdc7cd43dac7585f<'a, A> for Result<T, A> {
//     type Brand = ResultOkAppliedBrand<T>;
// }
//
// Uncommenting the above produces:
//
//   error[E0119]: conflicting implementations of trait
//     `SelectBrand_cdc7cd43dac7585f<'_, _>` for type `Result<_, _>`
//
// Coherence treats the two impls as overlapping because their trait-
// argument patterns are structurally identical (`SelectBrand<'_, A>`
// in both). The only thing distinguishing them is which position of
// the Result Self-type `A` names - information that coherence checks
// do not take into account.

// &T blanket: references inherit the wrapped type's projection.
impl<'a, T: ?Sized, A: 'a> SelectBrand_cdc7cd43dac7585f<'a, A> for &T
where
	T: SelectBrand_cdc7cd43dac7585f<'a, A>,
{
	type Brand = <T as SelectBrand_cdc7cd43dac7585f<'a, A>>::Brand;
}

// -------------------------------------------------------------------------
// Unified map function: Brand projected from SelectBrand, not passed as a
// trait parameter.
// -------------------------------------------------------------------------

pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
	f: impl FunctorDispatch<'a, <FA as SelectBrand_cdc7cd43dac7585f<'a, A>>::Brand, A, B, FA, Marker>,
	fa: FA,
) -> Apply!(<<FA as SelectBrand_cdc7cd43dac7585f<'a, A>>::Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	FA: SelectBrand_cdc7cd43dac7585f<'a, A>, {
	f.dispatch(fa)
}

// -------------------------------------------------------------------------
// Tests: only the single-brand and Result-with-ResultErrAppliedBrand
// direction work here, because only one of the two multi-brand impls is
// present. This demonstrates that the associated-type projection
// mechanism itself works - Rust commits Brand through the projection and
// FunctorDispatch selects cleanly for both Val and Ref - but it cannot
// support multi-brand without coherence-safe disambiguation.
// -------------------------------------------------------------------------

#[test]
fn val_option() {
	let r: Option<i32> = map(|x: i32| x * 2, Some(5));
	assert_eq!(r, Some(10));
}

#[test]
fn val_vec() {
	let r: Vec<i32> = map(|x: i32| x + 1, vec![1, 2, 3]);
	assert_eq!(r, vec![2, 3, 4]);
}

#[test]
fn val_result_ok_side_only() {
	// Works because only the ResultErrAppliedBrand impl exists.
	let r = map(|x: i32| x + 1, Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

#[test]
fn ref_option() {
	let opt = Some(5);
	let r: Option<i32> = map(|x: &i32| *x * 2, &opt);
	assert_eq!(r, Some(10));
	assert_eq!(opt, Some(5));
}

#[test]
fn ref_lazy() {
	let lazy = RcLazy::pure(10);
	let r = map(|x: &i32| *x * 3, &lazy);
	assert_eq!(*r.evaluate(), 30);
}

#[test]
fn ref_result_ok_side_only() {
	// Ref + multi-brand works here, because the associated-type
	// projection commits Brand cleanly BEFORE FunctorDispatch considers
	// Val vs Ref - sidestepping the cross-competition issue seen with
	// InferableBrand's trait-parameter Brand. The broken piece is coherence on
	// the impls themselves, not the inference machinery.
	let ok: Result<i32, String> = Ok(5);
	let r: Result<i32, String> = map(|x: &i32| *x + 1, &ok);
	assert_eq!(r, Ok(6));
}
