// Production-style POC for a `Slot` trait enabling closure-directed
// brand inference against the library's real HKT machinery
// (`Kind_*`/`InferableBrand_*`).
//
// Background: the library encodes higher-kinded polymorphism via
// brands. Some concrete types map back to multiple brands at a given
// arity (for example `Result<A, E>` is reachable through both
// `ResultErrAppliedBrand<E>` and `ResultOkAppliedBrand<T>` at arity 1),
// so brand inference refuses these and forces `explicit::` callers. A
// `Slot<Brand, A> for FA` trait that admits multiple impls per
// concrete type (one per brand) lets Rust's trait selection
// disambiguate via the closure's input type `A`.
//
// What this POC validates beyond the minimal
// `closure_directed_inference_poc.rs`:
//
// 1. Coherence of a blanket `impl<FA: InferableBrand> Slot<FA::Brand, A> for FA`
//    combined with direct `Slot` impls on multi-brand types. (Finding:
//    fails with E0119; see the "direct impls for every brand" comment
//    below.)
// 2. Lifetime-generic Slot trait with a lifetime-carrying GAT
//    (`type Out<B: 'a>: 'a`) compiling and resolving at call sites.
// 3. Return-type computation: whether a function returning
//    `<FA as Slot<'a, Brand, A>>::Out<B>` normalises to the expected
//    concrete type at call sites.
// 4. Composition with borrowed containers via a `&T` blanket for Slot
//    (owned and `&`-form call sites both route through the same trait
//    resolution).
// 5. The closure-annotation matrix: which call-site shapes require
//    an explicit parameter annotation and which do not.
//
// This is a pure-type POC: it does NOT route through the library's
// `FunctorDispatch` or Val/Ref Marker machinery. A bespoke `MapDispatch`
// trait stands in for dispatch so the tests can compute results; the
// goal is to exercise Rust's trait selection on the Slot signature
// shape, not to reproduce production routing. Full Val/Ref + dispatch
// composition is validated by the sibling `slot_valref_poc.rs`.

use fp_library::{
	brands::{
		LazyBrand,
		ResultErrAppliedBrand,
		ResultOkAppliedBrand,
	},
	kinds::Kind_cdc7cd43dac7585f,
	types::{
		Lazy,
		RcLazy,
		lazy::LazyConfig,
	},
};

// -------------------------------------------------------------------------
// Slot_cdc7cd43dac7585f: the reverse-direction trait keyed on (FA, Brand, A)
// -------------------------------------------------------------------------
//
// Matches the shape of Kind_cdc7cd43dac7585f: lifetime 'a, type parameter
// A: 'a, and a lifetime-bounded GAT `Out<B: 'a>: 'a`. Brand is a trait
// parameter bounded on Kind_cdc7cd43dac7585f, ensuring every resolved Brand
// is actually a kind at this arity.

#[allow(non_camel_case_types)]
pub trait Slot_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
	type Out<B: 'a>: 'a;
}

// -------------------------------------------------------------------------
// Direct Slot impls for every brand.
// -------------------------------------------------------------------------
//
// An earlier iteration of this POC tried a blanket:
//
//   impl<FA: InferableBrand> Slot<FA::Brand, A> for FA
//
// alongside direct impls for multi-brand types. Rust rejects that
// combination with E0119 (conflicting implementations), explicitly
// citing that upstream crates could in future implement `InferableBrand`
// for `Result<_, _>`, which would make the blanket potentially overlap
// with the direct impls. The parallel `&T` blankets for InferableBrand
// and Slot collide for the same reason.
//
// The blanket approach therefore does NOT work on stable rustc. This
// POC gives every brand (single- and multi-) a direct Slot impl.
// Coherence is trivially safe because each impl is keyed on a distinct
// Brand parameter.

impl<'a, A: 'a> Slot_cdc7cd43dac7585f<'a, fp_library::brands::OptionBrand, A> for Option<A> {
	type Out<B: 'a> = Option<B>;
}

impl<'a, A: 'a> Slot_cdc7cd43dac7585f<'a, fp_library::brands::VecBrand, A> for Vec<A> {
	type Out<B: 'a> = Vec<B>;
}

impl<'a, A: 'a, E: 'static> Slot_cdc7cd43dac7585f<'a, ResultErrAppliedBrand<E>, A>
	for Result<A, E>
{
	type Out<B: 'a> = Result<B, E>;
}

impl<'a, T: 'static, A: 'a> Slot_cdc7cd43dac7585f<'a, ResultOkAppliedBrand<T>, A> for Result<T, A> {
	type Out<B: 'a> = Result<T, B>;
}

// Lazy exercises a lifetime-bearing type: Lazy<'a, A, Config> carries
// `'a` in its concrete form, unlike Option/Vec. This is critical for
// validating the lifetime-generic GAT behaviour with real lifetime
// threading, not just 'static lifetimes.

impl<'a, A: 'a, Config: LazyConfig> Slot_cdc7cd43dac7585f<'a, LazyBrand<Config>, A>
	for Lazy<'a, A, Config>
{
	type Out<B: 'a> = Lazy<'a, B, Config>;
}

// -------------------------------------------------------------------------
// Reference blanket: &T inherits T's Slot impls.
// -------------------------------------------------------------------------
//
// With no InferableBrand-based Slot blanket in play, this single `&T`
// blanket is unopposed: trait selection has exactly one path from &Option,
// &Result, etc. to their Slot impls.

impl<'a, T: ?Sized, Brand, A: 'a> Slot_cdc7cd43dac7585f<'a, Brand, A> for &T
where
	T: Slot_cdc7cd43dac7585f<'a, Brand, A>,
	Brand: Kind_cdc7cd43dac7585f,
{
	type Out<B: 'a> = <T as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Out<B>;
}

// -------------------------------------------------------------------------
// A map-style function using Slot as the dispatch trait.
// -------------------------------------------------------------------------
//
// Simplified shape: no FunctorDispatch, no Val/Ref marker. The return type
// exercises Blocker 3 (does `<FA as Slot<...>>::Out<B>` normalise at call
// sites?).
//
// The actual mapping behaviour is provided by a dispatcher trait so that
// single-brand cases (going through the blanket) and multi-brand cases
// (going through direct impls) can both produce a concrete value. The POC
// uses a minimal MapDispatch trait to stand in for Functor::map /
// FunctorDispatch.

pub trait MapDispatch<'a, Brand, A: 'a, B: 'a>
where
	Brand: Kind_cdc7cd43dac7585f,
	Self: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
	fn dispatch_map(
		self,
		f: impl FnMut(A) -> B,
	) -> Self::Out<B>;
}

pub fn slot_map<'a, FA, A: 'a, B: 'a, Brand>(
	f: impl FnMut(A) -> B,
	fa: FA,
) -> <FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Out<B>
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: Slot_cdc7cd43dac7585f<'a, Brand, A> + MapDispatch<'a, Brand, A, B>, {
	fa.dispatch_map(f)
}

// -------------------------------------------------------------------------
// MapDispatch impls for the single-brand and multi-brand cases.
// -------------------------------------------------------------------------
//
// These exist only to let the tests actually compute results. In production
// the dispatch trait is FunctorDispatch and routes to Functor::map or
// RefFunctor::ref_map; here it just wraps stdlib methods. What matters for
// the POC is that the trait selection picks the right impl, not what the
// impl does at runtime.

impl<'a, A: 'a, B: 'a> MapDispatch<'a, fp_library::brands::OptionBrand, A, B> for Option<A> {
	fn dispatch_map(
		self,
		f: impl FnMut(A) -> B,
	) -> Option<B> {
		self.map(f)
	}
}

impl<'a, A: 'a, B: 'a> MapDispatch<'a, fp_library::brands::VecBrand, A, B> for Vec<A> {
	fn dispatch_map(
		self,
		f: impl FnMut(A) -> B,
	) -> Vec<B> {
		self.into_iter().map(f).collect()
	}
}

impl<'a, A: 'a, B: 'a, E: 'static> MapDispatch<'a, ResultErrAppliedBrand<E>, A, B>
	for Result<A, E>
{
	fn dispatch_map(
		self,
		f: impl FnMut(A) -> B,
	) -> Result<B, E> {
		self.map(f)
	}
}

impl<'a, T: 'static, A: 'a, B: 'a> MapDispatch<'a, ResultOkAppliedBrand<T>, A, B> for Result<T, A> {
	fn dispatch_map(
		self,
		f: impl FnMut(A) -> B,
	) -> Result<T, B> {
		self.map_err(f)
	}
}

// MapDispatch for Lazy is omitted deliberately. The lifetime-bearing Slot
// impl above is enough to validate Blocker 2; running slot_map through a
// real Lazy would require a MapDispatch shim that this POC is not
// concerned with. The lifetime-bearing test below validates the Slot
// machinery at the type level via an explicit associated-type projection.

// MapDispatch on &T delegates via Clone + by-value dispatch for simplicity.
// Production code would route to RefFunctor::ref_map instead; this is just
// enough to let the tests check that trait selection reaches &T at all.

impl<'a, T, Brand, A: 'a, B: 'a> MapDispatch<'a, Brand, A, B> for &T
where
	T: MapDispatch<'a, Brand, A, B> + Clone,
	Brand: Kind_cdc7cd43dac7585f,
{
	fn dispatch_map(
		self,
		f: impl FnMut(A) -> B,
	) -> Self::Out<B> {
		<T as MapDispatch<'a, Brand, A, B>>::dispatch_map(self.clone(), f)
	}
}

// -------------------------------------------------------------------------
// Tests: single-brand via blanket.
// -------------------------------------------------------------------------
//
// Validates that the blanket impl (InferableBrand -> Slot) is reachable by
// trait selection and that `Slot::Out<B>` normalises to the expected
// concrete type.

#[test]
fn single_brand_option_blanket() {
	let r: Option<i32> = slot_map(|x: i32| x * 2, Some(5));
	assert_eq!(r, Some(10));
}

#[test]
fn single_brand_option_different_output_type() {
	let r: Option<String> = slot_map(|x: i32| x.to_string(), Some(5));
	assert_eq!(r, Some("5".to_string()));
}

#[test]
fn single_brand_vec_blanket() {
	let r: Vec<i32> = slot_map(|x: i32| x + 1, vec![1, 2, 3]);
	assert_eq!(r, vec![2, 3, 4]);
}

#[test]
fn single_brand_none_still_works() {
	let r: Option<i32> = slot_map(|x: i32| x * 2, None::<i32>);
	assert_eq!(r, None);
}

// Lifetime-bearing type: validates Blocker 2 at the type level. The Slot
// trait and its GAT must resolve correctly for a concrete type that
// carries a lifetime parameter. This uses an associated-type projection
// instead of calling slot_map because Blocker 2 is about TYPE-LEVEL
// normalisation, not about the value-level dispatch logic.
#[test]
fn lifetime_bearing_type_slot_resolution() {
	fn assert_slot_resolves<'a, FA, Brand, A: 'a>()
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
	}
	assert_slot_resolves::<RcLazy<'_, i32>, LazyBrand<fp_library::types::RcLazyConfig>, i32>();

	// Out<B> must normalise to Lazy<'a, B, RcLazyConfig>. Verify by
	// binding a concrete value of the projected type.
	fn rebind_lazy<'a>(
		x: <RcLazy<'a, i32> as Slot_cdc7cd43dac7585f<
			'a,
			LazyBrand<fp_library::types::RcLazyConfig>,
			i32,
		>>::Out<i64>
	) -> RcLazy<'a, i64> {
		x
	}
	let source = RcLazy::pure(5i32);
	// Drive the projected type through rebind_lazy to confirm Out<B>
	// collapses to RcLazy<'_, i64> without further annotation.
	let rebuilt: RcLazy<'_, i64> = {
		let source_i64: RcLazy<'_, i64> = RcLazy::pure(i64::from(*source.evaluate()));
		rebind_lazy(source_i64)
	};
	assert_eq!(*rebuilt.evaluate(), 5);
}

// -------------------------------------------------------------------------
// Tests: multi-brand non-diagonal via direct impls.
// -------------------------------------------------------------------------
//
// The closure's A disambiguates which arity-1 brand of Result applies.
// Exercises the coexistence of the blanket impl (for single-brand
// types with InferableBrand) and direct Slot impls on multi-brand
// types that deliberately lack InferableBrand.

#[test]
fn multi_brand_result_ok_mapping() {
	// Closure takes i32; only ResultErrAppliedBrand<String> matches.
	let r = slot_map(|x: i32| x + 1, Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

#[test]
fn multi_brand_result_err_mapping() {
	// Closure takes String; only ResultOkAppliedBrand<i32> matches.
	let r: Result<i32, usize> = slot_map(|e: String| e.len(), Err::<i32, String>("hi".into()));
	assert_eq!(r, Err(2));
}

#[test]
fn multi_brand_result_output_type_differs() {
	let r = slot_map(|x: i32| x.to_string(), Ok::<i32, String>(5));
	assert_eq!(r, Ok("5".to_string()));
}

#[test]
fn multi_brand_err_variant_with_ok_brand() {
	// Closure input i32 picks ResultErrAppliedBrand<String>, which maps
	// over the Ok side. The concrete value is Err, so the result is
	// the Err value passed through.
	let r: Result<String, String> =
		slot_map(|x: i32| x.to_string(), Err::<i32, String>("fail".into()));
	assert_eq!(r, Err("fail".to_string()));
}

// -------------------------------------------------------------------------
// Tests: borrowed containers via the &T blanket.
// -------------------------------------------------------------------------
//
// Type-level composition with borrowed containers: closure-directed
// inference should work when the container is `&FA`. The POC's
// MapDispatch for &T uses Clone instead of by-ref dispatch, so this
// checks the Slot resolution path only, not full Val/Ref production
// dispatch. See `slot_valref_poc.rs` for the production-dispatch
// validation.

#[test]
fn borrowed_option_via_reference_blanket() {
	let opt = Some(5);
	let r: Option<i32> = slot_map(|x: i32| x * 2, &opt);
	assert_eq!(r, Some(10));
	// Original still usable.
	assert_eq!(opt, Some(5));
}

#[test]
fn borrowed_vec_via_reference_blanket() {
	let v = vec![1, 2, 3];
	let r: Vec<i32> = slot_map(|x: i32| x + 10, &v);
	assert_eq!(r, vec![11, 12, 13]);
	assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn borrowed_result_multi_brand_via_reference_blanket() {
	let ok: Result<i32, String> = Ok(5);
	let r: Result<i32, String> = slot_map(|x: i32| x + 1, &ok);
	assert_eq!(r, Ok(6));
}

// -------------------------------------------------------------------------
// Tests: closure-annotation matrix.
// -------------------------------------------------------------------------
//
// Enumerates which call shapes need an explicit closure parameter type.

#[test]
fn annotation_matrix_single_brand_no_annotation() {
	// Single-brand types: A flows from the container via the blanket.
	// The closure body's +1 doesn't need a type annotation because
	// Option<i32> -> A = i32 is forced by Slot.
	let r: Option<i32> = slot_map(|x| x + 1, Some(5));
	assert_eq!(r, Some(6));
}

#[test]
fn annotation_matrix_multi_brand_requires_closure_annotation() {
	// Multi-brand types: without an annotation, Rust cannot pick between
	// ResultErrAppliedBrand<String> (A = i32) and ResultOkAppliedBrand<i32>
	// (A = String). The annotation is load-bearing.
	let r = slot_map(|x: i32| x + 1, Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

// Alternative to closure annotation: annotating the call-site return type
// alone is NOT sufficient to disambiguate the brand for multi-brand
// types. The annotation must be on the closure's input, not on the
// call result.
//
// The call `let r: Result<i64, String> = slot_map(|x| i64::from(x + 1), Ok::<i32, String>(5))`
// fails with E0283 because the closure's A is free; knowing only the
// return type is Result<i64, String> does not force A = i32 (it could be
// either slot).
//
// #[test]
// fn annotation_matrix_multi_brand_annotation_on_call_return_type() {
//     let r: Result<i64, String> = slot_map(|x| i64::from(x + 1), Ok::<i32, String>(5));
//     assert_eq!(r, Ok(6));
// }

// -------------------------------------------------------------------------
// Diagonal failure case (validates the expected negative outcome).
// -------------------------------------------------------------------------
//
// With both slots the same type, closure-directed disambiguation fails.
// The expected diagnostic is E0283 (multiple impls apply). Kept commented
// out so the test file still compiles.
//
// #[test]
// fn diagonal_result_t_t_is_ambiguous() {
//     let _ = slot_map(|x: i32| x + 1, Ok::<i32, i32>(5));
//     // Expected: error[E0283]: type annotations needed
//     //   cannot infer type of the type parameter `Brand` declared on
//     //   the function `slot_map`
//     //   note: multiple `impl`s satisfying
//     //   `Result<i32, i32>: Slot_cdc7cd43dac7585f<'_, _, i32>` found
// }

// -------------------------------------------------------------------------
// Return-type normalisation smoke test.
// -------------------------------------------------------------------------
//
// Validates Blocker 3: can the result of `slot_map` flow into a position
// that names a concrete type, without needing manual annotations to coerce
// Slot::Out<B> into the target shape?

#[test]
fn return_type_flows_into_match() {
	// Forces the return type to match Result<i32, String>, which must
	// normalise from
	// <Result<i32, String> as Slot<'_, ResultErrAppliedBrand<String>, i32>>::Out<i32>.
	let mapped: Result<i32, String> = slot_map(|x: i32| x + 1, Ok::<i32, String>(5));
	let extracted = mapped.map_or(0, |v| v);
	assert_eq!(extracted, 6);
}

#[test]
fn return_type_flows_into_generic_function() {
	fn takes_result<T, E>(r: Result<T, E>) -> bool {
		r.is_ok()
	}
	assert!(takes_result(slot_map(|x: i32| x + 1, Ok::<i32, String>(5))));
}

#[test]
fn return_type_chains_with_further_slot_map() {
	// Compose two slot_map calls. Forces the first call's return type to
	// normalise in a position that's both a call argument and a Slot input.
	let once = slot_map(|x: i32| x * 2, Ok::<i32, String>(5));
	let twice = slot_map(|x: i32| x + 1, once);
	assert_eq!(twice, Ok(11));
}
