// Marker-via-InferableBrand POC (adopted design).
//
// -- Background --
//
// The library's `FunctorDispatch<..., Marker>` trait has two impls:
// Val (owned container, `Fn(A) -> B`) and Ref (borrowed container,
// `Fn(&A) -> B`). When combined with a InferableBrand trait that has Brand as
// a free trait parameter, the solver sees both Val and Ref impls as
// candidates. This "cross-competition" blocks Ref + multi-brand
// inference because Brand cannot commit while Marker is also free.
//
// Two alternative approaches to resolve this were tried and failed:
//   - SelectBrand (`slot_select_brand_poc.rs`): project Brand as an
//     associated type keyed on (FA, A). Coherence rejects multi-brand
//     impls because their trait-argument patterns are structurally
//     identical.
//   - AssocMarkerDispatch (`slot_assoc_marker_poc.rs`): make Marker an
//     associated type of the dispatch trait. Coherence rejects the
//     Val/Ref impls because both Self-type patterns contain
//     associated-type projections the checker can't prove non-overlap
//     for.
//
// -- Hypothesis --
//
// Attach Marker as an associated type of SLOT (not of the dispatch
// trait). The `&T` blanket sets `type Marker = Ref`; direct impls
// for owned types set `type Marker = Val`. The inference wrapper
// projects `<FA as InferableBrand<...>>::Marker` so that Marker commits from
// FA's reference-ness alone, without needing (Brand, A) to be
// resolved. Once Marker commits, FunctorDispatch picks the unique
// matching Val or Ref impl; the closure's Fn signature pins A; and
// InferableBrand's (Brand, A) ambiguity resolves from there.
//
// Coherence is not an issue because InferableBrand already has distinct
// trait-argument patterns (the Brand parameter differs between
// multi-brand impls). The associated-type Marker rides alongside
// Brand without being the sole disambiguator.
//
// -- Finding --
//
// CONFIRMED. All 9 tests pass on stable rustc, covering the full
// Val/Ref x single/multi-brand matrix including Ref + multi-brand
// (the critical case that failed in earlier approaches). This is
// the adopted design for the unified inference wrapper.

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
			Val,
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
// InferableBrand with associated Marker.
// -------------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub trait SlotM_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
	type Marker;
}

// Owned types: direct impls with Marker = Val.
impl<'a, A: 'a> SlotM_cdc7cd43dac7585f<'a, OptionBrand, A> for Option<A> {
	type Marker = Val;
}

impl<'a, A: 'a> SlotM_cdc7cd43dac7585f<'a, VecBrand, A> for Vec<A> {
	type Marker = Val;
}

impl<'a, A: 'a, Config: LazyConfig> SlotM_cdc7cd43dac7585f<'a, LazyBrand<Config>, A>
	for Lazy<'a, A, Config>
{
	type Marker = Val;
}

impl<'a, A: 'a, E: 'static> SlotM_cdc7cd43dac7585f<'a, ResultErrAppliedBrand<E>, A>
	for Result<A, E>
{
	type Marker = Val;
}

impl<'a, T: 'static, A: 'a> SlotM_cdc7cd43dac7585f<'a, ResultOkAppliedBrand<T>, A>
	for Result<T, A>
{
	type Marker = Val;
}

// References: blanket with Marker = Ref.
impl<'a, T: ?Sized, Brand, A: 'a> SlotM_cdc7cd43dac7585f<'a, Brand, A> for &T
where
	T: SlotM_cdc7cd43dac7585f<'a, Brand, A>,
	Brand: Kind_cdc7cd43dac7585f,
{
	type Marker = Ref;
}

// -------------------------------------------------------------------------
// Unified map function: Marker projected from InferableBrand.
// -------------------------------------------------------------------------

pub fn map<'a, FA, A: 'a, B: 'a, Brand>(
	f: impl FunctorDispatch<'a, Brand, A, B, FA, <FA as SlotM_cdc7cd43dac7585f<'a, Brand, A>>::Marker>,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: SlotM_cdc7cd43dac7585f<'a, Brand, A>, {
	f.dispatch(fa)
}

// -------------------------------------------------------------------------
// Tests.
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
fn val_result_ok_mapping() {
	let r = map(|x: i32| x + 1, Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

#[test]
fn val_result_err_mapping() {
	let r: Result<i32, usize> = map(|e: String| e.len(), Err::<i32, String>("hi".into()));
	assert_eq!(r, Err(2));
}

#[test]
fn ref_option() {
	let opt = Some(5);
	let r: Option<i32> = map(|x: &i32| *x * 2, &opt);
	assert_eq!(r, Some(10));
}

#[test]
fn ref_vec() {
	let v = vec![1, 2, 3];
	let r: Vec<i32> = map(|x: &i32| *x + 10, &v);
	assert_eq!(r, vec![11, 12, 13]);
}

#[test]
fn ref_lazy() {
	let lazy = RcLazy::pure(10);
	let r = map(|x: &i32| *x * 3, &lazy);
	assert_eq!(*r.evaluate(), 30);
}

#[test]
fn ref_result_ok_mapping() {
	// THE critical case. If this compiles and runs, the unified
	// signature handles Ref + multi-brand.
	let ok: Result<i32, String> = Ok(5);
	let r: Result<i32, String> = map(|x: &i32| *x + 1, &ok);
	assert_eq!(r, Ok(6));
}

#[test]
fn ref_result_err_mapping() {
	let err: Result<i32, String> = Err("hi".into());
	let r: Result<i32, usize> = map(|e: &String| e.len(), &err);
	assert_eq!(r, Err(2));
}
