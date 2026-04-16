// Third Slot POC: tests whether a unified `map` signature can handle
// Ref + multi-brand by using an associated-type projection for Brand
// instead of a free trait parameter.
//
// Earlier POCs established that:
//   - Slot with Brand as a trait PARAMETER creates Val/Ref cross-
//     competition in Rust's solver, so Ref + multi-brand fails in a
//     unified signature.
//   - Splitting Val and Ref into separate inference functions works but
//     gives up the single `map` entry point.
//
// Hypothesis tested: replace the trait-parameter Brand with an
// ASSOCIATED-TYPE projection keyed on (FA, A). Today's production `map`
// works because `<FA as InferableBrand>::Brand` is such a projection -
// Rust commits Brand immediately once FA is known, before FunctorDispatch
// selection begins. Extending this to key on (FA, A) where A comes from
// the closure's Fn signature might let multi-brand types share the
// unified signature.
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
// (both have a single type variable `A`); the distinguishing information
// - which Result slot `A` names - lives only in the Self-type, and
// coherence checks impl heads globally without caring about that
// positional detail.
//
// Why Slot didn't hit this: Slot has Brand as a trait PARAMETER. The two
// Slot impls are:
//
//     impl Slot<ResultErrAppliedBrand<E>, A> for Result<A, E>
//     impl Slot<ResultOkAppliedBrand<T>, A> for Result<T, A>
//
// Coherence sees different Brand values (`ResultErrAppliedBrand<_>` vs
// `ResultOkAppliedBrand<_>`) as structurally distinct trait-argument
// patterns and accepts the impls. The cost is that Brand becomes a
// separate inference dimension Rust has to solve alongside Marker,
// which is what causes the Val/Ref cross-competition failure.
//
// The associated-type projection idea cannot sidestep this trade-off on
// stable Rust without specialization (or a disambiguating zero-size
// position token threaded through the trait, which just moves the
// ambiguity to another layer). The unified `map` signature remains
// infeasible for Ref + multi-brand via any stable-Rust-only mechanism
// explored so far.
//
// Kept as a documentation artifact and regression signpost. The impls
// below do not compile together; the second multi-brand impl is
// commented out to keep the file in a buildable state while still
// preserving the structural demonstration.

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
	// Slot's trait-parameter Brand. The broken piece is coherence on
	// the impls themselves, not the inference machinery.
	let ok: Result<i32, String> = Ok(5);
	let r: Result<i32, String> = map(|x: &i32| *x + 1, &ok);
	assert_eq!(r, Ok(6));
}
