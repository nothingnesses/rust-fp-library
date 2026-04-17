// Slot-based `apply` (Semiapplicative) POC.
//
// -- Background --
//
// The library's `apply` takes two containers sharing an outer Brand:
//   - `ff`: a container holding a wrapped function (e.g., `Option<Rc<dyn Fn(i32) -> String>>`).
//   - `fa`: a container holding a value (e.g., `Option<i32>`).
// Today, the outer Brand must be pinned via turbofish:
//
//     apply::<RcFnBrand, OptionBrand, _, _>(ff, fa)
//
// The Slot-based inference mechanism validated in earlier POCs (for
// `map` and `bind`) uses a Slot<Brand, A> trait where closure input
// drives A, disambiguating the Brand. For `apply`, there is no
// direct closure. Instead, the function INSIDE `ff` carries A and B
// in its payload type: `<FnBrand as CloneFn>::Of<'a, A, B>`.
//
// -- How this differs from `map` and `bind` --
//
// 1. Two containers must agree on Brand. FF keys Slot on the
//    payload type `<FnBrand as CloneFn>::Of<'a, A, B>`; FA keys
//    Slot on A. Rust must find a Brand satisfying both bounds
//    simultaneously.
// 2. The library has no unified Val/Ref `ApplyDispatch` trait;
//    `apply` and `ref_apply` are separate functions. So the Val/Ref
//    cross-competition that motivated the Marker-via-Slot design
//    does not arise here.
// 3. The function payload is a branded wrapper (e.g., `Rc<dyn Fn>`)
//    accessed through `CloneFn::Of`, adding a level of indirection
//    Rust's solver must see through.
//
// -- Hypothesis --
//
// Slot-based inference of Brand from two simultaneous Slot bounds
// works for both single-brand and multi-brand Val `apply`. Rust's
// solver intersects the two bounds to commit a unique Brand.
//
// -- Finding --
//
// HYPOTHESIS CONFIRMED. All tests pass on stable rustc for both
// Val (`apply`) and Ref (`ref_apply`), including multi-brand Result
// with type-changing transformations and short-circuit on either
// side. The two-bound Brand resolution is the first case in this
// POC series where one bound alone is ambiguous for multi-brand
// types but the pair is uniquely solvable.

#![allow(unused_imports, reason = "Kind is used inside Apply! macro expansion")]
#![expect(
	clippy::type_complexity,
	reason = "Complex Apply! projections are inherent to HKT dispatch POCs"
)]

use {
	fp_library::{
		Apply,
		Kind,
		brands::{
			OptionBrand,
			RcFnBrand,
			ResultErrAppliedBrand,
			ResultOkAppliedBrand,
			VecBrand,
		},
		classes::{
			CloneFn,
			RefSemiapplicative,
			Semiapplicative,
		},
		dispatch::{
			Ref,
			Val,
		},
		functions::lift_fn_new,
		kinds::Kind_cdc7cd43dac7585f,
	},
	std::rc::Rc,
};

// -------------------------------------------------------------------------
// SlotApp: arity-1 Slot with Brand trait-param and Marker assoc-type.
// Direct impls set Marker = Val; &T blanket sets Marker = Ref.
// -------------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub trait SlotApp_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
	type Marker;
}

impl<'a, A: 'a> SlotApp_cdc7cd43dac7585f<'a, OptionBrand, A> for Option<A> {
	type Marker = Val;
}

impl<'a, A: 'a> SlotApp_cdc7cd43dac7585f<'a, VecBrand, A> for Vec<A> {
	type Marker = Val;
}

impl<'a, A: 'a, E: 'static> SlotApp_cdc7cd43dac7585f<'a, ResultErrAppliedBrand<E>, A>
	for Result<A, E>
{
	type Marker = Val;
}

impl<'a, T: 'static, A: 'a> SlotApp_cdc7cd43dac7585f<'a, ResultOkAppliedBrand<T>, A>
	for Result<T, A>
{
	type Marker = Val;
}

impl<'a, T: ?Sized, Brand, A: 'a> SlotApp_cdc7cd43dac7585f<'a, Brand, A> for &T
where
	T: SlotApp_cdc7cd43dac7585f<'a, Brand, A>,
	Brand: Kind_cdc7cd43dac7585f,
{
	type Marker = Ref;
}

// -------------------------------------------------------------------------
// Slot-based apply signature.
//
// The bounds say:
//   - FF is the Brand's Of applied to a CloneFn wrapper of (A, B).
//   - FA is the Brand's Of applied to A.
// Both share the same Brand type parameter; the solver must commit Brand
// consistently from both bounds.
//
// Because the library's `Semiapplicative::apply` already takes
// `Apply!(Brand::Of<...>)` shaped inputs directly, the implementation can
// just call `Brand::apply` once Brand is resolved. No custom dispatch
// trait is required in this POC.
// -------------------------------------------------------------------------

pub fn apply_via_slot<'a, FnBrand, Brand, A, B, FF, FA>(
	ff: FF,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	FnBrand: CloneFn + 'a,
	Brand: Semiapplicative,
	A: Clone + 'a,
	B: 'a,
	FF: SlotApp_cdc7cd43dac7585f<'a, Brand, <FnBrand as CloneFn>::Of<'a, A, B>>
		+ Into<
			Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
		>,
	FA: SlotApp_cdc7cd43dac7585f<'a, Brand, A>
		+ Into<Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>)>, {
	Brand::apply::<FnBrand, A, B>(ff.into(), fa.into())
}

// -------------------------------------------------------------------------
// Tests.
// -------------------------------------------------------------------------

#[test]
fn val_option_single_brand_full_turbofish() {
	// Force every type parameter explicitly to test whether the Slot
	// mechanism works at all for apply, independent of inference.
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	let x = Some(5i32);
	let y: Option<i32> = apply_via_slot::<RcFnBrand, OptionBrand, i32, i32, _, _>(f, x);
	assert_eq!(y, Some(10));
}

#[test]
fn val_option_single_brand() {
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	let x = Some(5i32);
	let y: Option<i32> = apply_via_slot::<RcFnBrand, _, _, _, _, _>(f, x);
	assert_eq!(y, Some(10));
}

#[test]
fn val_option_none_passthrough() {
	let f: Option<_> = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	let x: Option<i32> = None;
	let y: Option<i32> = apply_via_slot::<RcFnBrand, _, _, _, _, _>(f, x);
	assert_eq!(y, None);
}

#[test]
fn val_result_multi_brand_ok_mapping() {
	// ResultErrAppliedBrand<E> is Semiapplicative; the Ok-side function
	// gets applied to the Ok-side value when both are Ok.
	let f: Result<_, String> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	let x: Result<i32, String> = Ok(5);
	let y: Result<i32, String> = apply_via_slot::<RcFnBrand, _, _, _, _, _>(f, x);
	assert_eq!(y, Ok(6));
}

#[test]
fn val_result_multi_brand_ff_err() {
	let f: Result<_, String> = Err("bad fn".to_string());
	let x: Result<i32, String> = Ok(5);
	let y: Result<i32, String> = apply_via_slot::<RcFnBrand, _, _, _, _, _>(f, x);
	assert_eq!(y, Err("bad fn".to_string()));
}

#[test]
fn val_result_multi_brand_fa_err() {
	let f: Result<_, String> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	let x: Result<i32, String> = Err("bad val".to_string());
	let y: Result<i32, String> = apply_via_slot::<RcFnBrand, _, _, _, _, _>(f, x);
	assert_eq!(y, Err("bad val".to_string()));
}

#[test]
fn val_result_multi_brand_type_change() {
	// i32 -> String across the apply.
	let f: Result<_, String> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x.to_string()));
	let x: Result<i32, String> = Ok(42);
	let y: Result<String, String> = apply_via_slot::<RcFnBrand, _, _, _, _, _>(f, x);
	assert_eq!(y, Ok("42".to_string()));
}

// -------------------------------------------------------------------------
// Ref apply: inference validation via Slot.
//
// `ref_apply` takes both containers by reference. The `&T` blanket on
// SlotApp gives Marker = Ref. Brand is inferred from the inner types
// via the blanket delegation.
//
// The production implementation would use a dispatch trait (analogous to
// FunctorDispatch) to route `(&FF, &FA)` to `Brand::ref_apply`. For
// this POC we validate inference in two steps:
//   (1) A `ref_apply_infer_brand` function that takes generic `(&TFF, &TFA)`
//       with Slot bounds - if it compiles, Brand resolves.
//   (2) Call the library's `ref_apply` directly with the inferred Brand
//       to verify correctness.
//
// `RefSemiapplicative::ref_apply` uses `CloneFn<Ref>` for the function
// wrappers (the wrapped function takes `&A`, not owned A).
// -------------------------------------------------------------------------

/// Inference-only helper: validates that Slot resolves Brand from two
/// reference containers. Returns `PhantomData<Brand>` so callers can
/// assert which Brand was inferred via a type annotation.
pub fn ref_apply_infer_brand<'a, FnBrand, Brand, A, B, TFF, TFA>(
	_ff: &TFF,
	_fa: &TFA,
) -> std::marker::PhantomData<Brand>
where
	FnBrand: CloneFn<Ref> + 'a,
	Brand: RefSemiapplicative,
	A: 'a,
	B: 'a,
	TFF: SlotApp_cdc7cd43dac7585f<'a, Brand, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>,
	TFA: SlotApp_cdc7cd43dac7585f<'a, Brand, A>, {
	std::marker::PhantomData
}

// -------------------------------------------------------------------------
// Ref apply tests.
//
// Each test validates:
//   - Slot inference commits the correct Brand (via PhantomData type
//     annotation on ref_apply_infer_brand).
//   - The library's ref_apply produces the correct result when called
//     with that Brand.
// -------------------------------------------------------------------------

#[test]
fn ref_option_single_brand() {
	let f: Option<Rc<dyn Fn(&i32) -> i32>> = Some(Rc::new(|x: &i32| *x * 2));
	let x = Some(5i32);
	// Step 1: Slot inference commits Brand = OptionBrand.
	let _: std::marker::PhantomData<OptionBrand> =
		ref_apply_infer_brand::<RcFnBrand, _, _, _, _, _>(&f, &x);
	// Step 2: call ref_apply with the inferred Brand.
	let y: Option<i32> = fp_library::functions::ref_apply::<RcFnBrand, OptionBrand, _, _>(&f, &x);
	assert_eq!(y, Some(10));
}

#[test]
fn ref_result_multi_brand_ok_mapping() {
	let f: Result<Rc<dyn Fn(&i32) -> i32>, String> = Ok(Rc::new(|x: &i32| *x + 1));
	let x: Result<i32, String> = Ok(5);
	// Brand infers to ResultErrAppliedBrand<String> from both containers.
	let _: std::marker::PhantomData<ResultErrAppliedBrand<String>> =
		ref_apply_infer_brand::<RcFnBrand, _, _, _, _, _>(&f, &x);
	let y: Result<i32, String> =
		fp_library::functions::ref_apply::<RcFnBrand, ResultErrAppliedBrand<String>, _, _>(&f, &x);
	assert_eq!(y, Ok(6));
}

#[test]
fn ref_result_multi_brand_ff_err() {
	let f: Result<Rc<dyn Fn(&i32) -> i32>, String> = Err("bad fn".to_string());
	let x: Result<i32, String> = Ok(5);
	let _: std::marker::PhantomData<ResultErrAppliedBrand<String>> =
		ref_apply_infer_brand::<RcFnBrand, _, _, _, _, _>(&f, &x);
	let y: Result<i32, String> =
		fp_library::functions::ref_apply::<RcFnBrand, ResultErrAppliedBrand<String>, _, _>(&f, &x);
	assert_eq!(y, Err("bad fn".to_string()));
}

#[test]
fn ref_result_multi_brand_type_change() {
	let f: Result<Rc<dyn Fn(&i32) -> String>, String> = Ok(Rc::new(|x: &i32| x.to_string()));
	let x: Result<i32, String> = Ok(42);
	let _: std::marker::PhantomData<ResultErrAppliedBrand<String>> =
		ref_apply_infer_brand::<RcFnBrand, _, _, _, _, _>(&f, &x);
	let y: Result<String, String> =
		fp_library::functions::ref_apply::<RcFnBrand, ResultErrAppliedBrand<String>, _, _>(&f, &x);
	assert_eq!(y, Ok("42".to_string()));
}
