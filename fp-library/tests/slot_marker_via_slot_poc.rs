// Fifth Slot POC: tests whether lifting `Marker` into Slot as an
// associated-type projection (keyed on FA's reference-ness) can break
// the Val/Ref cross-competition that blocks Ref + multi-brand in the
// unified signature.
//
// Earlier findings:
//   - Slot with Brand as a trait PARAMETER (POC 1, POC 2): coherence
//     OK, but unified Val/Ref signature fails for Ref + multi-brand
//     because FunctorDispatch's free-parameter `Marker` leaves Val and
//     Ref impls as parallel candidates until Brand commits, and Brand
//     cannot commit because A is still unresolved.
//   - SelectBrand with Brand as an associated type (POC 3): coherence
//     rejects the two multi-brand impls.
//   - AssocMarkerDispatch with Marker as an associated type of the
//     dispatch trait (POC 4): coherence rejects Val and Ref impls.
//   - Pinning Marker to Ref (probe in POC 2): works but costs the
//     unified signature.
//
// Hypothesis tested here: Slot gets an associated `type Marker`. The
// blanket for references sets `type Marker = Ref`; the direct impls
// for owned types set `type Marker = Val`. Marker is then projected
// through `<FA as Slot<...>>::Marker` in the map signature, so it
// commits as soon as FA is known (from its reference-ness alone),
// without needing (Brand, A) to be resolved first. Val vs Ref
// competition in FunctorDispatch is thereby eliminated because Marker
// is no longer a free trait parameter at the call site.
//
// If this works, Ref + multi-brand is unblocked for the unified
// signature and all four cases (Val/Ref x single/multi-brand) work
// through a single `map(f, fa)` entry point.

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
// Slot with associated Marker.
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
// Unified map function: Marker projected from Slot.
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
