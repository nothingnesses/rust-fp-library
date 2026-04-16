// Second Slot POC: validates composition with production FunctorDispatch
// and Val/Ref markers.
//
// Targets Q4 from docs/plans/multi-brand-ergonomics/plan.md (workflow
// note 8). The first Slot POC (slot_production_poc.rs) validated that
// Slot resolution works at the type level, including through a &T
// blanket; it used a Clone-based MapDispatch shim rather than the
// library's production FunctorDispatch trait. This file validates that
// Slot's brand-dispatch axis composes with FunctorDispatch's Val/Ref
// Marker axis in a single function signature, with both inferred
// simultaneously from the closure's input type and the container.
//
// Expected outcome if composition works:
//   map_via_slot(|x: i32| ..., Some(5))                     -> Val,  OptionBrand
//   map_via_slot(|x: &i32| ..., &Some(5))                   -> Ref,  OptionBrand
//   map_via_slot(|x: i32| ..., Ok::<i32, String>(5))        -> Val,  ResultErrAppliedBrand<String>
//   map_via_slot(|x: &i32| ..., &Ok::<i32, String>(5))      -> Ref,  ResultErrAppliedBrand<String>
//   map_via_slot(|e: String| e.len(), Err::<i32, String>...)-> Val,  ResultOkAppliedBrand<i32>
//   map_via_slot(|e: &String| e.len(), &Err::<i32, String>.)-> Ref,  ResultOkAppliedBrand<i32>
//
// If Rust cannot simultaneously infer Brand (via Slot) and Marker (via
// FunctorDispatch) in one signature, the POC fails to compile and we
// learn that phase 1 needs a different factoring (for example, splitting
// Val and Ref into separate Slot-bounded functions).

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
// Slot trait (marker-only variant).
// -------------------------------------------------------------------------
//
// Finding from an earlier iteration: a Slot trait with its own `Out<B>`
// GAT does not unify with FunctorDispatch's `Apply!(Brand::Of<'a, B>)`
// return type, even when structurally equal. Rust treats the two
// associated-type projections as distinct, producing E0308 in the
// map_via_slot body.
//
// This variant keeps Slot as a pure marker asserting
// `Brand::Of<'a, A> = Self`, with no associated types. The
// map_via_slot return type is then expressed directly as
// `Apply!(Brand::Of<'a, B>)` matching FunctorDispatch's dispatcher
// return type. This avoids the projection mismatch.

#[allow(non_camel_case_types)]
pub trait Slot_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
}

// Direct marker impls per brand.

impl<'a, A: 'a> Slot_cdc7cd43dac7585f<'a, OptionBrand, A> for Option<A> {}

impl<'a, A: 'a> Slot_cdc7cd43dac7585f<'a, VecBrand, A> for Vec<A> {}

impl<'a, A: 'a, E: 'static> Slot_cdc7cd43dac7585f<'a, ResultErrAppliedBrand<E>, A>
	for Result<A, E>
{
}

impl<'a, T: 'static, A: 'a> Slot_cdc7cd43dac7585f<'a, ResultOkAppliedBrand<T>, A> for Result<T, A> {}

impl<'a, A: 'a, Config: LazyConfig> Slot_cdc7cd43dac7585f<'a, LazyBrand<Config>, A>
	for Lazy<'a, A, Config>
{
}

// Reference blanket: &T inherits T's Slot impls.

impl<'a, T: ?Sized, Brand, A: 'a> Slot_cdc7cd43dac7585f<'a, Brand, A> for &T
where
	T: Slot_cdc7cd43dac7585f<'a, Brand, A>,
	Brand: Kind_cdc7cd43dac7585f,
{
}

// -------------------------------------------------------------------------
// map_via_slot: Slot-bound dispatch using the production FunctorDispatch
// trait and Val/Ref marker.
// -------------------------------------------------------------------------
//
// This is the core signature under test. Brand is resolved via Slot
// (keyed on FA + A); Marker is resolved via FunctorDispatch (keyed on
// the closure's input type). Both must be inferable from the call site.

pub fn map_via_slot<'a, FA, A: 'a, B: 'a, Brand, Marker>(
	f: impl FunctorDispatch<'a, Brand, A, B, FA, Marker>,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
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
// `FunctorDispatch<'a, Brand, A, B, FA, Marker>` + `FA: Slot<'a, Brand, A>`
// signature used in `map_via_slot`.
//
// Error: E0283 "cannot infer type for type parameter `Brand`" with notes
// naming multiple FunctorDispatch impls for
// `{closure}: FunctorDispatch<'_, _, _, _, &Result<i32, String>, _>` and
// multiple Slot impls for `Result<i32, String>: Slot<'_, _, _>`.
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
// Q15 prototype: explicit::map-style function routing through Slot.
// -------------------------------------------------------------------------
//
// Signature shape under the Q15 option-b "rewrite explicit::map to bound
// on Slot" proposal. The caller pins Brand via turbofish; Slot + the
// container argument drive A and FA inference.

pub fn map_explicit<'a, Brand, A: 'a, B: 'a, FA, Marker>(
	f: impl FunctorDispatch<'a, Brand, A, B, FA, Marker>,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
	f.dispatch(fa)
}

#[test]
fn q15_explicit_with_slot_bound_val_single_brand() {
	let r = map_explicit::<OptionBrand, _, _, _, _>(|x: i32| x * 2, Some(5));
	assert_eq!(r, Some(10));
}

#[test]
fn q15_explicit_with_slot_bound_val_multi_brand() {
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
fn q15_explicit_with_slot_bound_ref_single_brand() {
	let opt = Some(5);
	let r = map_explicit::<OptionBrand, _, _, _, _>(|x: &i32| *x * 3, &opt);
	assert_eq!(r, Some(15));
}

#[test]
fn q15_explicit_with_slot_bound_ref_multi_brand() {
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
fn q15_explicit_with_slot_bound_diagonal() {
	let r =
		map_explicit::<ResultErrAppliedBrand<i32>, _, _, _, _>(|x: i32| x + 1, Ok::<i32, i32>(5));
	assert_eq!(r, Ok(6));
}

// -------------------------------------------------------------------------
// Sanity check: the diagonal failure still fails with the full production
// dispatch signature.
// -------------------------------------------------------------------------
//
// Kept commented. Uncomment to reproduce E0283 under the Slot +
// FunctorDispatch + Marker composition.
//
// #[test]
// fn diagonal_still_ambiguous() {
//     let _ = map_via_slot(|x: i32| x + 1, Ok::<i32, i32>(5));
// }
