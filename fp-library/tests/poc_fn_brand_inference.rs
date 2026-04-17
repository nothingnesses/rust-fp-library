// FnBrand inference via a Slot-like trait pattern.
//
// -- Background --
//
// The library's `apply` function takes a container of wrapped
// functions (`ff`) and a container of values (`fa`). Currently,
// `FnBrand` must be specified via turbofish because Rust's solver
// cannot reverse-project an associated type: given an
// `Rc<dyn Fn(A) -> B>`, it cannot determine that `FnBrand = RcFnBrand`.
//
// The Brand inference problem was solved with a Slot trait pattern
// where Brand is a trait parameter:
//
//     trait Slot<Brand, A> { type Marker; }
//     impl<A> Slot<OptionBrand, A> for Option<A> { type Marker = Val; }
//
// This file investigates whether the same pattern can provide the
// reverse mapping for FnBrand: given the concrete wrapper type
// (e.g., `Rc<dyn Fn(A) -> B>`), can the solver infer `FnBrand`
// (e.g., `RcFnBrand`) via a `FnBrandSlot` trait?
//
// -- Hypothesis --
//
// A `FnBrandSlot<FnBrand, A, B>` trait with impls for each concrete
// wrapper type can enable the solver to infer FnBrand from the
// concrete wrapped function type. Combined with the existing Brand
// Slot, this would let `apply` infer both Brand and FnBrand from
// the argument types.
//
// -- Finding --
//
// FnBrand inference IS feasible via a FnBrandSlot trait, but requires
// a specific approach to avoid a circular dependency in the solver.
//
// WHAT WORKS (all steps pass):
//   Step 1: Basic FnBrandSlot<FnBrand, A, B> trait with impls for
//     Rc<dyn Fn(A) -> B> and Arc<dyn Fn(A) -> B>. The solver infers
//     FnBrand from the concrete wrapper type when the bound is placed
//     directly on a function parameter's type.
//   Step 2: Works with lift_fn_new output (CloneFn::Of normalizes to
//     the concrete type, and the solver matches the impl).
//   Steps 3-5: Dual inference of Brand AND FnBrand works for Option,
//     Result (multi-brand), and both RcFnBrand and ArcFnBrand.
//
// WHAT DOES NOT WORK:
//   Putting FnBrandSlot as a bound on <FnBrand as CloneFn>::Of<'a, A, B>
//   directly. This creates a circular dependency: the solver needs
//   FnBrand to compute the associated type, but FnBrand is what it
//   is trying to infer. Errors: E0284 + E0283.
//
// THE WORKING APPROACH:
//   Introduce an explicit type parameter W for the wrapped function
//   type. BrandSlot on FF resolves Brand and W simultaneously (from
//   the concrete container type). FnBrandSlot on W then resolves
//   FnBrand. A bridge bound converts Brand's Of<W> to Of<CloneFn::Of>
//   for the actual Semiapplicative::apply call. This avoids the
//   circular dependency because W is resolved from FF's concrete type,
//   not through an associated type projection on the unknown FnBrand.
//
// CAVEATS:
//   - Requires one additional type parameter (W) on the apply function,
//     though it is inferred and never specified by callers.
//   - Requires a bridge bound (Into conversion) to equate W with
//     CloneFn::Of. In practice this is identity since W IS CloneFn::Of
//     after normalization, but the solver needs the explicit bound.
//   - FnBrandSlot impls must be generated for each concrete wrapper
//     type (Rc<dyn Fn>, Arc<dyn Fn>, etc.). This is a fixed set
//     determined by the FnBrand variants in the library.

#![allow(unused_imports, reason = "Kind is used inside Apply! macro expansion")]

use {
	fp_library::{
		Apply,
		Kind,
		brands::{
			ArcFnBrand,
			FnBrand,
			OptionBrand,
			RcFnBrand,
			ResultErrAppliedBrand,
		},
		classes::{
			CloneFn,
			Semiapplicative,
		},
		dispatch::Val,
		functions::lift_fn_new,
		kinds::Kind_cdc7cd43dac7585f,
	},
	std::{
		rc::Rc,
		sync::Arc,
	},
};

// =========================================================================
// Step 1: Define FnBrandSlot and test basic resolution
// =========================================================================
//
// Define a trait with FnBrand as a trait parameter. Add impls mapping
// each concrete wrapper type back to its FnBrand.
//
// Orphan rule: FnBrandSlot is local, so impls for foreign types
// (Rc, Arc) are allowed.

trait FnBrandSlot<FnBrand, A, B> {}

impl<'a, A: 'a, B: 'a> FnBrandSlot<RcFnBrand, A, B> for Rc<dyn 'a + Fn(A) -> B> {}
impl<'a, A: 'a, B: 'a> FnBrandSlot<ArcFnBrand, A, B> for Arc<dyn 'a + Fn(A) -> B> {}

/// Test function: takes a generic F with a FnBrandSlot bound.
/// If the solver can infer FnBrand from the concrete type, this
/// should compile without turbofish on FnBrand.
fn accepts_fn_brand_slot<FnBrand, A, B, F: FnBrandSlot<FnBrand, A, B>>(_f: &F) {}

#[test]
fn step1_basic_rc_resolution() {
	let f: Rc<dyn Fn(i32) -> i32> = Rc::new(|x: i32| x + 1);
	// If FnBrand is inferred as RcFnBrand from Rc<dyn Fn>, this compiles.
	accepts_fn_brand_slot(&f);
}

#[test]
fn step1_basic_arc_resolution() {
	let f: Arc<dyn Fn(i32) -> i32> = Arc::new(|x: i32| x + 1);
	// If FnBrand is inferred as ArcFnBrand from Arc<dyn Fn>, this compiles.
	accepts_fn_brand_slot(&f);
}

// =========================================================================
// Step 2: Test with lifetime-generic CloneFn::Of
// =========================================================================
//
// The actual wrapped type produced by lift_fn_new is:
//   <RcFnBrand as CloneFn>::Of<'a, A, B>
// which normalizes to:
//   Rc<dyn 'a + Fn(A) -> B>
//
// The solver must see through the associated type projection to match
// the FnBrandSlot impl.

#[test]
fn step2_lift_fn_new_rc() {
	let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	// f has type <RcFnBrand as CloneFn>::Of<'_, i32, i32> = Rc<dyn Fn(i32) -> i32>.
	// Does the solver match this against the FnBrandSlot impl?
	accepts_fn_brand_slot(&f);
}

#[test]
fn step2_lift_fn_new_arc() {
	let f = lift_fn_new::<ArcFnBrand, _, _>(|x: i32| x + 1);
	accepts_fn_brand_slot(&f);
}

// =========================================================================
// Step 3: Combined inference with Brand (Slot) and FnBrand (FnBrandSlot)
// =========================================================================
//
// A simplified apply-like function using BOTH Brand Slot and FnBrandSlot
// bounds. The solver must resolve Brand from the container type (via
// the existing Slot mechanism) AND FnBrand from the wrapped function
// type (via FnBrandSlot) simultaneously.

// Brand Slot (replicating the pattern from the codebase).
trait BrandSlot<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
	type Marker;
}

impl<'a, A: 'a> BrandSlot<'a, OptionBrand, A> for Option<A> {
	type Marker = Val;
}

impl<'a, A: 'a, E: 'static> BrandSlot<'a, ResultErrAppliedBrand<E>, A> for Result<A, E> {
	type Marker = Val;
}

// -- Step 3a: Direct bound on associated type (FAILS) --
//
// Approach: put FnBrandSlot bound on <FnBrand as CloneFn>::Of<'a, A, B>.
// This fails because the solver must know FnBrand to compute the LHS,
// but FnBrand is what we are trying to infer FROM this bound.
// Error: E0284 "cannot satisfy <_ as CloneFn>::Of<'_, i32, i32> == Rc<dyn Fn(i32) -> i32>"
// Error: E0283 "multiple impls satisfying _: FnBrandSlot<_, i32, i32>"
//
// fn apply_3a<'a, FnBrand, Brand, A, B, FF, FA>(ff: FF, fa: FA) -> ...
// where
//   <FnBrand as CloneFn>::Of<'a, A, B>: FnBrandSlot<FnBrand, A, B>,  // CIRCULAR
//   ...

// -- Step 3b: Extract element type W via BrandSlot, then put FnBrandSlot on W --
//
// Approach: add an explicit type parameter W for the wrapped function
// type. BrandSlot on FF resolves Brand and W simultaneously. Then
// FnBrandSlot on W resolves FnBrand. This avoids the circular
// dependency because W is resolved directly from FF's concrete type,
// not through an associated type projection.

fn apply_dual_infer<'a, FnBrand, Brand, A, B, W, FF, FA>(
	ff: FF,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	FnBrand: CloneFn + 'a,
	Brand: Semiapplicative,
	A: Clone + 'a,
	B: 'a,
	W: 'a + Clone + FnBrandSlot<FnBrand, A, B>,
	FF: BrandSlot<'a, Brand, W>
		+ Into<Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>)>,
	FA: BrandSlot<'a, Brand, A>
		+ Into<Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>)>,
	// Bridge: W must equal CloneFn::Of so Brand::apply accepts it.
	Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>): Into<
		Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
	>, {
	let ff_branded: Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>) = ff.into();
	let ff_cast: Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>) =
		ff_branded.into();
	Brand::apply::<FnBrand, A, B>(ff_cast, fa.into())
}

#[test]
fn step3_option_rc_dual_inference() {
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	let x = Some(5);
	// Both Brand (OptionBrand) and FnBrand (RcFnBrand) should be inferred.
	let result: Option<i32> = apply_dual_infer(f, x);
	assert_eq!(result, Some(6));
}

// =========================================================================
// Step 4: Multi-brand types (Result)
// =========================================================================

#[test]
fn step4_result_rc_dual_inference() {
	let f: Result<_, String> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	let x: Result<i32, String> = Ok(5);
	// Brand = ResultErrAppliedBrand<String>, FnBrand = RcFnBrand.
	let result: Result<i32, String> = apply_dual_infer(f, x);
	assert_eq!(result, Ok(6));
}

// =========================================================================
// Step 5: ArcFnBrand
// =========================================================================

#[test]
fn step5_option_arc_dual_inference() {
	let f = Some(lift_fn_new::<ArcFnBrand, _, _>(|x: i32| x + 1));
	let x = Some(5);
	// Brand = OptionBrand, FnBrand = ArcFnBrand.
	let result: Option<i32> = apply_dual_infer(f, x);
	assert_eq!(result, Some(6));
}

#[test]
fn step5_result_arc_dual_inference() {
	let f: Result<_, String> = Ok(lift_fn_new::<ArcFnBrand, _, _>(|x: i32| x + 1));
	let x: Result<i32, String> = Ok(5);
	let result: Result<i32, String> = apply_dual_infer(f, x);
	assert_eq!(result, Ok(6));
}
