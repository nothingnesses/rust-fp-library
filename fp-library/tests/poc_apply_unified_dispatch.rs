// Unified Val/Ref apply dispatch POC.
//
// -- Background --
//
// The library has two separate type classes for applicative function
// application:
//
// - Semiapplicative::apply (Val): takes owned containers. Wrapped
//   functions have type <FnBrand as CloneFn<Val>>::Of<A, B>, which
//   normalizes to e.g. Rc<dyn Fn(A) -> B>. Requires A: Clone.
//
// - RefSemiapplicative::ref_apply (Ref): takes borrowed containers.
//   Wrapped functions have type <FnBrand as CloneFn<Ref>>::Of<A, B>,
//   which normalizes to e.g. Rc<dyn Fn(&A) -> B>. No A: Clone needed.
//
// Other dispatch modules (FunctorDispatch, BindDispatch, etc.) unify
// Val/Ref via a Marker type parameter on a dispatch trait:
//
// - Val impl: Self = Brand::Of<A>, closure is Fn(A) -> B
// - Ref impl: Self = &Brand::Of<A>, closure is Fn(&A) -> B
// - Marker is projected from <FA as InferableBrand<Brand, A>>::Marker
//
// The current dispatch/semiapplicative.rs handles Val only. The
// challenge for unification is that apply's FnBrand bound depends on
// the Marker: Val needs FnBrand: CloneFn<Val>, Ref needs
// FnBrand: CloneFn<Ref>.
//
// Additionally, FnBrandSlot (which maps concrete wrapper types like
// Rc<dyn Fn(A)->B> back to RcFnBrand) currently only handles Val
// wrapper types. For Ref, the wrapper Rc<dyn Fn(&A)->B> must also
// be mapped back to RcFnBrand, but a naive impl overlaps with the
// Val impl (Rc<dyn Fn(&A)->B> matches the Val pattern with A=&A').
//
// This POC validates three approaches for unifying Val and Ref apply
// into a single inference wrapper function.
//
// -- Approach A: Dual CloneFn bounds --
//
// The inference wrapper requires FnBrand: CloneFn<Val> + CloneFn<Ref>.
// This is always satisfied (RcFnBrand and ArcFnBrand implement both).
// The dispatch trait routes to the correct method. FnBrandSlot uses a
// broad pointer-only matching (Rc<T: ?Sized> -> RcFnBrand) that avoids
// the Val/Ref overlap.
//
// -- Approach B: CloneFn<Marker> --
//
// The inference wrapper bounds FnBrand: CloneFn<Marker> where Marker
// is projected from the InferableBrand trait. Since Val and Ref ARE the
// ClosureMode types (they implement ClosureMode directly), this bound
// selects the correct CloneFn mode automatically. FnBrandSlot gains a
// Mode parameter to disambiguate Val and Ref wrapper types.
//
// -- Approach C: CloneFn on dispatch trait only --
//
// The inference wrapper has no CloneFn bound on FnBrand at all. The
// CloneFn requirements live entirely in the dispatch trait impls.
// FnBrandSlot uses broad pointer-only matching. The question is
// whether Rust's solver can verify the impl-level CloneFn bounds
// from the available context.
//
// -- Hypothesis --
//
// All three approaches should compile and work for both Val and Ref
// dispatch with single-brand types (Option, Vec). Multi-brand types
// (Result) are expected to need explicit Brand turbofish due to the
// InferableBrand-based inference limitation (no closure to anchor the element
// type). Approach B is expected to be the cleanest since the CloneFn
// bound precisely matches the dispatch mode.
//
// -- Finding --
//
// ALL THREE APPROACHES WORK for single-brand types (Option). Both Val
// and Ref dispatch compile, run correctly, and infer Brand + FnBrand
// from the container types alone (no turbofish needed).
//
// MULTI-BRAND TYPES (Result) require explicit Brand turbofish, same as
// the existing explicit::apply / ref_apply. This is an inherent
// limitation of InferableBrand-based inference: Result has two InferableBrand impls and
// apply has no user closure to anchor the element type (unlike map/bind
// where the closure pins A and disambiguates). For multi-brand apply,
// users must use explicit::apply or ref_apply with a Brand turbofish.
//
// APPROACH COMPARISON:
//
// Approach A (Dual CloneFn bounds):
//   FnBrand: CloneFn<Val> + CloneFn<Ref>
//   + Simplest FnBrandSlot (broad Rc<T> matching, no Mode parameter).
//   + Works for all existing FnBrand types (RcFnBrand, ArcFnBrand).
//   - Over-constraining: requires both modes even when only one is used.
//   - Would reject a hypothetical FnBrand implementing only one mode.
//
// Approach B (CloneFn<Marker>):
//   FnBrand: CloneFn<Marker> where Marker from InferableBrand
//   + Precise bound: only requires the mode that matches the dispatch.
//   + FnBrandSlot with Mode parameter provides clean disambiguation.
//   + Marker: ClosureMode bound is always satisfied (Val/Ref are the
//     ClosureMode types).
//   - Requires Mode-parameterized FnBrandSlot (4 impls instead of 2).
//   - Slightly more complex inference wrapper signature.
//   RECOMMENDED: cleanest semantics, most extensible.
//
// Approach C (CloneFn on dispatch trait only):
//   Inference wrapper has NO CloneFn bound on FnBrand.
//   + Minimal bounds on the inference wrapper.
//   + Works because Rust verifies impl-level bounds lazily.
//   + Uses broad FnBrandSlot (same as approach A).
//   - CloneFn requirement is invisible at the call site; errors from
//     unresolved FnBrand may be confusing.
//   - Less self-documenting: the wrapper signature doesn't show that
//     FnBrand must be a function brand.
//
// REFERENCE BRIDGE:
//
// The Ref dispatch impls use an Into bound on references:
//   &'b Brand::Of<W>: Into<&'b Brand::Of<CloneFn<Ref>::Of<A,B>>>
// This is satisfied by the identity From impl when W = CloneFn<Ref>::Of.
// It requires 'a: 'b (the data must outlive the borrow), which is
// always true at concrete call sites but must be stated explicitly in
// the generic impl.
//
// FnBrandSlot DISAMBIGUATION:
//
// The existing production FnBrandSlot (Val only) has impls for
// Rc<dyn Fn(A)->B>. A naive Ref impl for Rc<dyn Fn(&A)->B> overlaps
// because Rc<dyn Fn(&X)->Y> matches the Val pattern with A=&X.
//
// Two solutions validated:
// 1. Mode-parameterized FnBrandSlot (approach B): separate Val and Ref
//    impls keyed on Mode. Clean, no overlap.
// 2. Broad pointer matching (approaches A, C): match on Rc<T: ?Sized>
//    regardless of closure signature. No overlap between Rc/Arc.
//    Less specific but sufficient when combined with bridge bounds.
//
// A: Clone CONDITIONALITY:
//
// The A: Clone bound (required by Semiapplicative::apply for Val) is
// placed on the dispatch trait's Val impl only, not on the inference
// wrapper. Rust's solver checks impl bounds lazily: when matching
// the Val impl, it verifies A: Clone; when matching the Ref impl,
// it does not require Clone. Non-Clone types work correctly for Ref
// dispatch without any special handling.

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
			ArcFnBrand,
			OptionBrand,
			RcFnBrand,
			ResultErrAppliedBrand,
			VecBrand,
		},
		classes::{
			CloneFn,
			RefSemiapplicative,
			Semiapplicative,
		},
		dispatch::{
			ClosureMode,
			Ref,
			Val,
		},
		functions::lift_fn_new,
		kinds::{
			InferableBrand_cdc7cd43dac7585f,
			Kind_cdc7cd43dac7585f,
		},
	},
	std::rc::Rc,
};

// =========================================================================
// FnBrandSlot with Mode parameter (for Approach B)
// =========================================================================
//
// The existing production FnBrandSlot maps concrete wrapper types to
// their FnBrand, but only handles Val wrappers. For Ref wrappers
// (Rc<dyn Fn(&A)->B>), a naive FnBrandSlot<FnBrand, A, B> impl
// overlaps with the Val impl because Rc<dyn Fn(&A)->B> also matches
// the Val pattern with A' = &A.
//
// Adding a Mode parameter disambiguates:
// - FnBrandSlot<RcFnBrand, A, B, Val> for Rc<dyn Fn(A)->B>
// - FnBrandSlot<RcFnBrand, A, B, Ref> for Rc<dyn Fn(&A)->B>
//
// These don't overlap because Mode differs, and the Self types are
// genuinely different (Fn(A)->B vs Fn(&A)->B are different trait
// objects even though Fn(&A)->B could match Fn(A)->B with A=&A').
//
// The Mode parameter is linked to the InferableBrand Marker, so the solver
// uses the known Marker to select the correct FnBrandInferableBrand impl.

trait FnBrandSlotModed<FnBrand, A, B, Mode = Val> {}

impl<'a, A: 'a, B: 'a> FnBrandSlotModed<RcFnBrand, A, B, Val> for Rc<dyn 'a + Fn(A) -> B> {}

impl<'a, A: 'a, B: 'a> FnBrandSlotModed<ArcFnBrand, A, B, Val>
	for std::sync::Arc<dyn 'a + Fn(A) -> B + Send + Sync>
{
}

impl<'a, A: 'a, B: 'a> FnBrandSlotModed<RcFnBrand, A, B, Ref> for Rc<dyn 'a + Fn(&A) -> B> {}

impl<'a, A: 'a, B: 'a> FnBrandSlotModed<ArcFnBrand, A, B, Ref>
	for std::sync::Arc<dyn 'a + Fn(&A) -> B + Send + Sync>
{
}

// =========================================================================
// FnBrandSlot with broad pointer matching (for Approaches A and C)
// =========================================================================
//
// Instead of encoding the closure signature, match only on the pointer
// type: any Rc<T> maps to RcFnBrand, any Arc<T> maps to ArcFnBrand.
// This avoids the Val/Ref overlap entirely because Rc and Arc are
// distinct types.
//
// Trade-off: less specific matching (an Rc<i32> would also match), but
// the bridge bounds in the dispatch trait ensure type safety. Only
// valid function wrappers satisfy the full bound set.

trait FnBrandSlotBroad<FnBrand> {}

impl<T: ?Sized> FnBrandSlotBroad<RcFnBrand> for Rc<T> {}
impl<T: ?Sized> FnBrandSlotBroad<ArcFnBrand> for std::sync::Arc<T> {}

// =========================================================================
// Approach A: Dual CloneFn bounds
// =========================================================================
//
// The inference wrapper requires FnBrand: CloneFn<Val> + CloneFn<Ref>.
// This is over-constraining (a Val-only call only needs CloneFn<Val>),
// but since RcFnBrand and ArcFnBrand implement both modes, the bound
// is always satisfiable for valid FnBrands.
//
// The dispatch trait routes based on container ownership:
// - Val: Self = Brand::Of<W> (owned), calls Semiapplicative::apply
// - Ref: Self = &Brand::Of<W> (borrowed), calls RefSemiapplicative::ref_apply

mod approach_a {
	use super::*;

	pub trait ApplyDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		W: 'a,
		FA,
		Marker,
	> {
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
	}

	// Val impl: Self = Brand::Of<W>
	impl<'a, FnBrand, Brand, A, B, W>
		ApplyDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			W,
			Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
			Val,
		> for Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>)
	where
		FnBrand: CloneFn + 'a,
		Brand: Semiapplicative,
		A: Clone + 'a,
		B: 'a,
		W: Clone + 'a,
		Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>): Into<
			Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
		>,
	{
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>) {
			Brand::apply::<FnBrand, A, B>(self.into(), fa)
		}
	}

	// Ref impl: Self = &Brand::Of<W>
	impl<'a, 'b, FnBrand, Brand, A, B, W>
		ApplyDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			W,
			&'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
			Ref,
		> for &'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>)
	where
		'a: 'b,
		FnBrand: CloneFn<Ref> + 'a,
		Brand: RefSemiapplicative,
		A: 'a,
		B: 'a,
		W: 'a,
		// Bridge: &Brand::Of<W> -> &Brand::Of<CloneFn<Ref>::Of<A,B>>
		// Satisfied by identity From when W = CloneFn<Ref>::Of<A,B>
		&'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>): Into<
			&'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
		>,
	{
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>) {
			Brand::ref_apply::<FnBrand, A, B>(self.into(), fa)
		}
	}

	// -- Inference wrapper --

	pub fn apply<'a, FnBrand, Brand, A, B, W, FF, FA>(
		ff: FF,
		fa: FA,
	) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FnBrand: CloneFn + CloneFn<Ref> + 'a,
		A: 'a,
		B: 'a,
		W: 'a + FnBrandSlotBroad<FnBrand>,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>,
		FF: InferableBrand_cdc7cd43dac7585f<'a, Brand, W>
			+ ApplyDispatch<
				'a,
				FnBrand,
				Brand,
				A,
				B,
				W,
				FA,
				<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
			>, {
		ff.dispatch(fa)
	}
}

// =========================================================================
// Approach B: CloneFn<Marker>
// =========================================================================
//
// The inference wrapper bounds FnBrand: CloneFn<Marker> where Marker
// is projected from the InferableBrand trait. Val and Ref implement ClosureMode,
// so CloneFn<Val> and CloneFn<Ref> are selected automatically.
//
// FnBrandSlot uses the Mode parameter to disambiguate Val and Ref
// wrapper types. The Mode parameter is linked to the Marker.

mod approach_b {
	use super::*;

	pub trait ApplyDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		W: 'a,
		FA,
		Marker,
	> {
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
	}

	// Val impl
	impl<'a, FnBrand, Brand, A, B, W>
		ApplyDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			W,
			Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
			Val,
		> for Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>)
	where
		FnBrand: CloneFn + 'a,
		Brand: Semiapplicative,
		A: Clone + 'a,
		B: 'a,
		W: Clone + 'a,
		Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>): Into<
			Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
		>,
	{
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>) {
			Brand::apply::<FnBrand, A, B>(self.into(), fa)
		}
	}

	// Ref impl
	impl<'a, 'b, FnBrand, Brand, A, B, W>
		ApplyDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			W,
			&'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
			Ref,
		> for &'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>)
	where
		'a: 'b,
		FnBrand: CloneFn<Ref> + 'a,
		Brand: RefSemiapplicative,
		A: 'a,
		B: 'a,
		W: 'a,
		&'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>): Into<
			&'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
		>,
	{
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>) {
			Brand::ref_apply::<FnBrand, A, B>(self.into(), fa)
		}
	}

	// -- Inference wrapper --

	pub fn apply<'a, FnBrand, Brand, A, B, W, FF, FA>(
		ff: FF,
		fa: FA,
	) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		W: 'a,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>,
		<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker: ClosureMode,
		FnBrand: CloneFn<<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker> + 'a,
		W: FnBrandSlotModed<
				FnBrand,
				A,
				B,
				<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
			>,
		FF: InferableBrand_cdc7cd43dac7585f<'a, Brand, W>
			+ ApplyDispatch<
				'a,
				FnBrand,
				Brand,
				A,
				B,
				W,
				FA,
				<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
			>, {
		ff.dispatch(fa)
	}
}

// =========================================================================
// Approach C: CloneFn on dispatch trait only
// =========================================================================
//
// The inference wrapper has NO CloneFn bound on FnBrand. The CloneFn
// requirements live entirely in the ApplyDispatch impls. The solver
// must verify impl-level bounds from the known concrete FnBrand type
// (RcFnBrand or ArcFnBrand).
//
// FnBrandSlot uses broad pointer matching (no Mode needed since the
// wrapper doesn't bind CloneFn).

mod approach_c {
	use super::*;

	pub trait ApplyDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		W: 'a,
		FA,
		Marker,
	> {
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
	}

	// Val impl
	impl<'a, FnBrand, Brand, A, B, W>
		ApplyDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			W,
			Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
			Val,
		> for Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>)
	where
		FnBrand: CloneFn + 'a,
		Brand: Semiapplicative,
		A: Clone + 'a,
		B: 'a,
		W: Clone + 'a,
		Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>): Into<
			Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
		>,
	{
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>) {
			Brand::apply::<FnBrand, A, B>(self.into(), fa)
		}
	}

	// Ref impl
	impl<'a, 'b, FnBrand, Brand, A, B, W>
		ApplyDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			W,
			&'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
			Ref,
		> for &'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>)
	where
		'a: 'b,
		FnBrand: CloneFn<Ref> + 'a,
		Brand: RefSemiapplicative,
		A: 'a,
		B: 'a,
		W: 'a,
		&'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, W>): Into<
			&'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
		>,
	{
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>) {
			Brand::ref_apply::<FnBrand, A, B>(self.into(), fa)
		}
	}

	// -- Inference wrapper (no CloneFn bound on FnBrand) --

	pub fn apply<'a, FnBrand, Brand, A, B, W, FF, FA>(
		ff: FF,
		fa: FA,
	) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FnBrand: 'a,
		A: 'a,
		B: 'a,
		W: 'a + FnBrandSlotBroad<FnBrand>,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>,
		FF: InferableBrand_cdc7cd43dac7585f<'a, Brand, W>
			+ ApplyDispatch<
				'a,
				FnBrand,
				Brand,
				A,
				B,
				W,
				FA,
				<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
			>, {
		ff.dispatch(fa)
	}
}

// =========================================================================
// Approach A tests
// =========================================================================

#[test]
fn approach_a_val_option() {
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	let x = Some(5i32);
	let result: Option<i32> = approach_a::apply(f, x);
	assert_eq!(result, Some(10));
}

#[test]
fn approach_a_val_option_none() {
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	let x: Option<i32> = None;
	let result: Option<i32> = approach_a::apply(f, x);
	assert_eq!(result, None);
}

#[test]
fn approach_a_ref_option() {
	let f: Option<Rc<dyn Fn(&i32) -> i32>> = Some(Rc::new(|x: &i32| *x * 2));
	let x = Some(5i32);
	let result: Option<i32> = approach_a::apply(&f, &x);
	assert_eq!(result, Some(10));
}

// Multi-brand types (Result) cannot use the unified inference wrapper
// because Result has two InferableBrand impls and the solver cannot intersect
// them without a closure to anchor the element type (unlike map/bind
// where the closure pins A). For multi-brand apply, users fall back
// to the library's explicit::apply / ref_apply with Brand turbofish.
//
// Cross-validation: verify the Val and Ref type class methods work
// correctly for Result (confirming the dispatch trait can be wired to
// the right method; only the inference wrapper's InferableBrand resolution
// fails for multi-brand).
#[test]
fn approach_a_val_result_crosscheck() {
	let f: Result<_, String> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	let x: Result<i32, String> = Ok(5);
	let result =
		fp_library::functions::explicit::apply::<RcFnBrand, ResultErrAppliedBrand<String>, _, _>(
			f, x,
		);
	assert_eq!(result, Ok(6));
}

#[test]
fn approach_a_ref_result_crosscheck() {
	let f: Result<Rc<dyn Fn(&i32) -> i32>, String> = Ok(Rc::new(|x: &i32| *x + 1));
	let x: Result<i32, String> = Ok(5);
	let result =
		fp_library::functions::ref_apply::<RcFnBrand, ResultErrAppliedBrand<String>, _, _>(&f, &x);
	assert_eq!(result, Ok(6));
}

// =========================================================================
// Approach B tests
// =========================================================================

#[test]
fn approach_b_val_option() {
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	let x = Some(5i32);
	let result: Option<i32> = approach_b::apply(f, x);
	assert_eq!(result, Some(10));
}

#[test]
fn approach_b_val_option_none() {
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	let x: Option<i32> = None;
	let result: Option<i32> = approach_b::apply(f, x);
	assert_eq!(result, None);
}

#[test]
fn approach_b_ref_option() {
	let f: Option<Rc<dyn Fn(&i32) -> i32>> = Some(Rc::new(|x: &i32| *x * 2));
	let x = Some(5i32);
	let result: Option<i32> = approach_b::apply(&f, &x);
	assert_eq!(result, Some(10));
}

// Multi-brand Result: same limitation as approach A (see comment above).
#[test]
fn approach_b_val_result_crosscheck() {
	let f: Result<_, String> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	let x: Result<i32, String> = Ok(5);
	let result =
		fp_library::functions::explicit::apply::<RcFnBrand, ResultErrAppliedBrand<String>, _, _>(
			f, x,
		);
	assert_eq!(result, Ok(6));
}

#[test]
fn approach_b_ref_result_crosscheck() {
	let f: Result<Rc<dyn Fn(&i32) -> i32>, String> = Ok(Rc::new(|x: &i32| *x + 1));
	let x: Result<i32, String> = Ok(5);
	let result =
		fp_library::functions::ref_apply::<RcFnBrand, ResultErrAppliedBrand<String>, _, _>(&f, &x);
	assert_eq!(result, Ok(6));
}

// =========================================================================
// Approach C tests
// =========================================================================

#[test]
fn approach_c_val_option() {
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	let x = Some(5i32);
	let result: Option<i32> = approach_c::apply(f, x);
	assert_eq!(result, Some(10));
}

#[test]
fn approach_c_val_option_none() {
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	let x: Option<i32> = None;
	let result: Option<i32> = approach_c::apply(f, x);
	assert_eq!(result, None);
}

#[test]
fn approach_c_ref_option() {
	let f: Option<Rc<dyn Fn(&i32) -> i32>> = Some(Rc::new(|x: &i32| *x * 2));
	let x = Some(5i32);
	let result: Option<i32> = approach_c::apply(&f, &x);
	assert_eq!(result, Some(10));
}

// Multi-brand Result: same limitation as approaches A and B.
#[test]
fn approach_c_val_result_crosscheck() {
	let f: Result<_, String> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	let x: Result<i32, String> = Ok(5);
	let result =
		fp_library::functions::explicit::apply::<RcFnBrand, ResultErrAppliedBrand<String>, _, _>(
			f, x,
		);
	assert_eq!(result, Ok(6));
}

#[test]
fn approach_c_ref_result_crosscheck() {
	let f: Result<Rc<dyn Fn(&i32) -> i32>, String> = Ok(Rc::new(|x: &i32| *x + 1));
	let x: Result<i32, String> = Ok(5);
	let result =
		fp_library::functions::ref_apply::<RcFnBrand, ResultErrAppliedBrand<String>, _, _>(&f, &x);
	assert_eq!(result, Ok(6));
}

// =========================================================================
// Additional edge-case tests (using recommended approach B)
// =========================================================================

#[test]
fn approach_b_val_option_type_change() {
	// i32 -> String: verifies B is inferred correctly when A != B.
	let f = Some(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x.to_string()));
	let x = Some(42i32);
	let result: Option<String> = approach_b::apply(f, x);
	assert_eq!(result, Some("42".to_string()));
}

#[test]
fn approach_b_ref_option_type_change() {
	let f: Option<Rc<dyn Fn(&i32) -> String>> = Some(Rc::new(|x: &i32| x.to_string()));
	let x = Some(42i32);
	let result: Option<String> = approach_b::apply(&f, &x);
	assert_eq!(result, Some("42".to_string()));
}

#[test]
fn approach_b_val_vec() {
	// Vec: Cartesian product apply (each function applied to each value).
	let f = vec![
		lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 10),
		lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 100),
	];
	let x = vec![1, 2];
	let result: Vec<i32> = approach_b::apply(f, x);
	assert_eq!(result, vec![10, 20, 101, 102]);
}

#[test]
fn approach_b_ref_vec() {
	let f: Vec<Rc<dyn Fn(&i32) -> i32>> =
		vec![Rc::new(|x: &i32| *x * 10), Rc::new(|x: &i32| *x + 100)];
	let x = vec![1, 2];
	let result: Vec<i32> = approach_b::apply(&f, &x);
	assert_eq!(result, vec![10, 20, 101, 102]);
}

#[test]
fn approach_b_val_option_ff_none() {
	// Short-circuit when ff is None.
	let f: Option<_> = None::<std::rc::Rc<dyn Fn(i32) -> i32>>;
	let x = Some(5i32);
	let result: Option<i32> = approach_b::apply(f, x);
	assert_eq!(result, None);
}
