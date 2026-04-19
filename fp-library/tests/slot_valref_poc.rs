// InferableBrand + production FunctorDispatch composition POC.
//
// -- Background --
//
// `FunctorDispatch<'a, Brand, A, B, FA, Marker>` is the library's
// dispatch trait routing `Fn(A) -> B` closures to `Functor::map`
// (Val marker) and `Fn(&A) -> B` closures to `RefFunctor::ref_map`
// (Ref marker). Today it is bound by `FA: InferableBrand` so that
// Brand is committed from a concrete container with a unique brand.
// Multi-brand containers (e.g., `Result<A, E>`) have no
// `InferableBrand` impl and fall through to an explicit-brand
// turbofish path.
//
// A separate POC (`slot_production_poc.rs`) validated that a
// `InferableBrand<Brand, A> for FA` trait with per-brand impls lets closure
// input types disambiguate Brand, but it used a bespoke
// `MapDispatch` shim rather than the production FunctorDispatch. This
// file composes InferableBrand directly with the production `FunctorDispatch`
// to check whether InferableBrand's brand-inference axis and FunctorDispatch's
// Val/Ref Marker axis can both be inferred simultaneously.
//
// -- Findings --
//
//   Val + single-brand:        WORKS.
//   Val + multi-brand:         WORKS with closure annotation.
//   Ref + single-brand:        WORKS.
//   Ref + multi-brand (unified
//      Val+Ref signature):     DOES NOT COMPILE (E0283). The solver
//                              treats both Val and Ref FunctorDispatch
//                              impls as candidates until a Marker is
//                              committed, and cannot commit Brand
//                              without that.
//   Ref + multi-brand (Marker
//      pinned to Ref):         WORKS. A Ref-only variant
//                              (`map_via_slot_ref_only` below) that
//                              hard-codes Marker = Ref compiles for
//                              multi-brand types. The failure is a
//                              Val/Ref cross-competition issue, not a
//                              Brand-resolution issue.
//   Explicit-brand variant:    WORKS for every case including Ref +
//                              multi-brand, because the turbofish pins
//                              Brand directly (see `map_explicit`
//                              below).
//
// NOTE: the Ref + multi-brand gap was later resolved by a different
// approach (`slot_marker_via_slot_poc.rs`): lifting Marker into InferableBrand
// as an associated type so it commits from FA's reference-ness before
// Brand resolution begins.

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
		dispatch::{
			Ref,
			functor::FunctorDispatch,
		},
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
// InferableBrand trait (marker-only variant).
// -------------------------------------------------------------------------
//
// Finding from an earlier iteration: a InferableBrand trait with its own `Out<B>`
// GAT does not unify with FunctorDispatch's `Apply!(Brand::Of<'a, B>)`
// return type, even when structurally equal. Rust treats the two
// associated-type projections as distinct, producing E0308 in the
// map_via_slot body.
//
// This variant keeps InferableBrand as a pure marker asserting
// `Brand::Of<'a, A> = Self`, with no associated types. The
// map_via_slot return type is then expressed directly as
// `Apply!(Brand::Of<'a, B>)` matching FunctorDispatch's dispatcher
// return type. This avoids the projection mismatch.

#[allow(non_camel_case_types)]
pub trait InferableBrand_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
}

// Direct marker impls per brand.

impl<'a, A: 'a> InferableBrand_cdc7cd43dac7585f<'a, OptionBrand, A> for Option<A> {}

impl<'a, A: 'a> InferableBrand_cdc7cd43dac7585f<'a, VecBrand, A> for Vec<A> {}

impl<'a, A: 'a, E: 'static> InferableBrand_cdc7cd43dac7585f<'a, ResultErrAppliedBrand<E>, A>
	for Result<A, E>
{
}

impl<'a, T: 'static, A: 'a> InferableBrand_cdc7cd43dac7585f<'a, ResultOkAppliedBrand<T>, A>
	for Result<T, A>
{
}

impl<'a, A: 'a, Config: LazyConfig> InferableBrand_cdc7cd43dac7585f<'a, LazyBrand<Config>, A>
	for Lazy<'a, A, Config>
{
}

// Reference blanket: &T inherits T's InferableBrand impls.

impl<'a, T: ?Sized, Brand, A: 'a> InferableBrand_cdc7cd43dac7585f<'a, Brand, A> for &T
where
	T: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>,
	Brand: Kind_cdc7cd43dac7585f,
{
}

// -------------------------------------------------------------------------
// map_via_slot: InferableBrand-bound dispatch using the production FunctorDispatch
// trait and Val/Ref marker.
// -------------------------------------------------------------------------
//
// This is the core signature under test. Brand is resolved via InferableBrand
// (keyed on FA + A); Marker is resolved via FunctorDispatch (keyed on
// the closure's input type). Both must be inferable from the call site.

pub fn map_via_slot<'a, FA, A: 'a, B: 'a, Brand, Marker>(
	f: impl FunctorDispatch<'a, Brand, A, B, FA, Marker>,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
	f.dispatch(fa)
}

// -------------------------------------------------------------------------
// Val tests: single-brand.
// -------------------------------------------------------------------------

#[test]
fn val_option_single_brand() {
	let r: Option<i32> = map_via_slot(|x: i32| x * 2, Some(5));
	assert_eq!(r, Some(10));
}

#[test]
fn val_vec_single_brand() {
	let r: Vec<i32> = map_via_slot(|x: i32| x + 1, vec![1, 2, 3]);
	assert_eq!(r, vec![2, 3, 4]);
}

#[test]
fn val_option_different_output() {
	let r: Option<String> = map_via_slot(|x: i32| x.to_string(), Some(5));
	assert_eq!(r, Some("5".to_string()));
}

// -------------------------------------------------------------------------
// Ref tests: single-brand with Val/Ref marker routing.
// -------------------------------------------------------------------------

#[test]
fn ref_option_single_brand() {
	let opt = Some(5);
	let r: Option<i32> = map_via_slot(|x: &i32| *x * 2, &opt);
	assert_eq!(r, Some(10));
	assert_eq!(opt, Some(5));
}

#[test]
fn ref_vec_single_brand() {
	let v = vec![1, 2, 3];
	let r: Vec<i32> = map_via_slot(|x: &i32| *x + 10, &v);
	assert_eq!(r, vec![11, 12, 13]);
	assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn ref_lazy_single_brand() {
	let lazy = RcLazy::pure(10);
	let r = map_via_slot(|x: &i32| *x * 3, &lazy);
	assert_eq!(*r.evaluate(), 30);
}

// -------------------------------------------------------------------------
// Val tests: multi-brand.
// -------------------------------------------------------------------------

#[test]
fn val_result_ok_mapping() {
	// Closure input i32 pins Brand = ResultErrAppliedBrand<String>.
	let r = map_via_slot(|x: i32| x + 1, Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

#[test]
fn val_result_err_mapping() {
	// Closure input String pins Brand = ResultOkAppliedBrand<i32>.
	let r: Result<i32, usize> = map_via_slot(|e: String| e.len(), Err::<i32, String>("hi".into()));
	assert_eq!(r, Err(2));
}

#[test]
fn val_result_ok_with_err_value() {
	// Concrete value is Err; closure picks Ok-mapping brand; Err passes
	// through.
	let r: Result<String, String> =
		map_via_slot(|x: i32| x.to_string(), Err::<i32, String>("fail".into()));
	assert_eq!(r, Err("fail".to_string()));
}

// -------------------------------------------------------------------------
// Ref tests: multi-brand. THE critical composition test.
// -------------------------------------------------------------------------
//
// FINDING: these tests DO NOT COMPILE under the simple composition
// `FunctorDispatch<'a, Brand, A, B, FA, Marker>` + `FA: InferableBrand<'a, Brand, A>`
// signature used in `map_via_slot`.
//
// Error: E0283 "cannot infer type for type parameter `Brand`" with notes
// naming multiple FunctorDispatch impls for
// `{closure}: FunctorDispatch<'_, _, _, _, &Result<i32, String>, _>` and
// multiple InferableBrand impls for `Result<i32, String>: InferableBrand<'_, _, _>`.
//
// What's happening: the Ref FunctorDispatch impl's Self-type is
// `&'b Apply!(Brand::Of<'a, A>)`. Rust unifies this with &Result<i32, String>
// by solving `Apply!(Brand::Of<'a, A>) = Result<i32, String>`, which has
// two solutions (ResultErrAppliedBrand<String>+A=i32 and
// ResultOkAppliedBrand<i32>+A=String). The closure's `F: Fn(&A) -> B`
// where-clause would uniquely pin A=i32, but Rust's trait solver treats
// the impl match and the where-clause as separate phases: it can't pick
// an impl without committing Brand/A first, and can't commit Brand/A
// without checking F's signature. This manifests as ambiguity.
//
// The same structure in the Val case (`Fn(A) -> B`, container by value)
// works because... the POC confirms val_result_ok_mapping passes, so
// something about by-value dispatch permits inference that by-reference
// dispatch does not. Needs investigation during phase 1 or a targeted
// follow-up POC that explicitly bounds on Fn(&A) -> B outside the
// FunctorDispatch trait.
//
// Kept commented so the POC file still compiles. Uncomment to reproduce.
//
// #[test]
// fn ref_result_ok_mapping() {
//     let ok: Result<i32, String> = Ok(5);
//     let r: Result<i32, String> = map_via_slot(|x: &i32| *x + 1, &ok);
//     assert_eq!(r, Ok(6));
// }
//
// #[test]
// fn ref_result_err_mapping() {
//     let err: Result<i32, String> = Err("hi".into());
//     let r: Result<i32, usize> = map_via_slot(|e: &String| e.len(), &err);
//     assert_eq!(r, Err(2));
// }
//
// #[test]
// fn ref_result_ok_with_err_value() {
//     let err: Result<i32, String> = Err("fail".into());
//     let r: Result<String, String> = map_via_slot(|x: &i32| x.to_string(), &err);
//     assert_eq!(r, Err("fail".to_string()));
// }

// -------------------------------------------------------------------------
// Mixed cases: the same container used with Val then Ref.
// -------------------------------------------------------------------------

#[test]
fn val_then_ref_same_container() {
	let opt = Some(5);
	let r_ref: Option<i32> = map_via_slot(|x: &i32| *x * 2, &opt);
	let r_val: Option<i32> = map_via_slot(|x: i32| x + 1, opt);
	assert_eq!(r_ref, Some(10));
	assert_eq!(r_val, Some(6));
}

// Same failure mode as ref_result_* above. Kept commented.
//
// #[test]
// fn multi_brand_ref_then_val() {
//     let ok: Result<i32, String> = Ok(5);
//     let r_ref: Result<i32, String> = map_via_slot(|x: &i32| *x + 100, &ok);
//     let r_val: Result<i32, String> = map_via_slot(|x: i32| x * 10, ok);
//     assert_eq!(r_ref, Ok(105));
//     assert_eq!(r_val, Ok(50));
// }

// -------------------------------------------------------------------------
// Probe: does pinning Marker disambiguate Ref + multi-brand?
// -------------------------------------------------------------------------
//
// Hypothesis being tested: the Ref + multi-brand failure might be caused
// by the solver considering both the Val and Ref FunctorDispatch impls
// as candidates, rather than by genuine Brand-only ambiguity within
// the Ref impl alone.
//
// Test: write a Ref-only variant of `map_via_slot` that pins the Marker
// type parameter to `Ref`. If the Ref + multi-brand failure was caused
// by Val/Ref cross-competition, this variant should succeed. If it
// still fails, the problem is Brand disambiguation within the Ref
// FunctorDispatch impl alone.

pub fn map_via_slot_ref_only<'a, FA, A: 'a, B: 'a, Brand>(
	f: impl FunctorDispatch<'a, Brand, A, B, FA, Ref>,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
	f.dispatch(fa)
}

// Sanity: single-brand via the ref-only variant still works.
#[test]
fn probe_ref_only_single_brand() {
	let opt = Some(5);
	let r = map_via_slot_ref_only(|x: &i32| *x * 2, &opt);
	assert_eq!(r, Some(10));
}

// Core probe: if the Val/Ref cross-competition hypothesis is correct,
// this should compile and pass.
#[test]
fn probe_ref_only_multi_brand() {
	let ok: Result<i32, String> = Ok(5);
	let r: Result<i32, String> = map_via_slot_ref_only(|x: &i32| *x + 1, &ok);
	assert_eq!(r, Ok(6));
}

// -------------------------------------------------------------------------
// Explicit-brand variant: map function bounded on InferableBrand with Brand
// pinned via turbofish.
// -------------------------------------------------------------------------
//
// With Brand fixed at the call site, InferableBrand's only job is to assert
// `Brand::Of<'a, A> = Self`. The closure's input type fixes A (as in
// any `Fn(A) -> B` bound), and FA is inferred from the container
// argument. This factoring works in every case the
// inference-based `map_via_slot` above fails in (notably Ref +
// multi-brand), because the turbofish removes the brand-selection
// ambiguity up front.

pub fn map_explicit<'a, Brand, A: 'a, B: 'a, FA, Marker>(
	f: impl FunctorDispatch<'a, Brand, A, B, FA, Marker>,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
	f.dispatch(fa)
}

#[test]
fn map_explicit_val_single_brand() {
	let r = map_explicit::<OptionBrand, _, _, _, _>(|x: i32| x * 2, Some(5));
	assert_eq!(r, Some(10));
}

#[test]
fn map_explicit_val_multi_brand() {
	// With Brand pinned via turbofish, the diagonal-type ambiguity that
	// defeats inference-based map disappears. Users trade annotation on
	// the closure for annotation on the function.
	let r = map_explicit::<ResultErrAppliedBrand<String>, _, _, _, _>(
		|x: i32| x + 1,
		Ok::<i32, String>(5),
	);
	assert_eq!(r, Ok(6));
}

#[test]
fn map_explicit_ref_single_brand() {
	let opt = Some(5);
	let r = map_explicit::<OptionBrand, _, _, _, _>(|x: &i32| *x * 3, &opt);
	assert_eq!(r, Some(15));
}

#[test]
fn map_explicit_ref_multi_brand() {
	// The Ref + multi-brand combination that fails for map_via_slot
	// (because Brand cannot be inferred from a multi-brand container
	// behind a reference) DOES work here because the turbofish pins
	// Brand directly.
	let ok: Result<i32, String> = Ok(5);
	let r = map_explicit::<ResultErrAppliedBrand<String>, _, _, _, _>(|x: &i32| *x + 1, &ok);
	assert_eq!(r, Ok(6));
}

// Diagonal case: same as today's explicit::map, since Brand is pinned.
#[test]
fn map_explicit_diagonal() {
	let r =
		map_explicit::<ResultErrAppliedBrand<i32>, _, _, _, _>(|x: i32| x + 1, Ok::<i32, i32>(5));
	assert_eq!(r, Ok(6));
}

// -------------------------------------------------------------------------
// Sanity check: the diagonal failure still fails with the full production
// dispatch signature.
// -------------------------------------------------------------------------
//
// Kept commented. Uncomment to reproduce E0283 under the InferableBrand +
// FunctorDispatch + Marker composition.
//
// #[test]
// fn diagonal_still_ambiguous() {
//     let _ = map_via_slot(|x: i32| x + 1, Ok::<i32, i32>(5));
// }
