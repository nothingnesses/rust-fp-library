// Marker-as-dispatch-trait-associated-type POC (negative result).
//
// -- Background --
//
// The library's `FunctorDispatch<..., Marker>` trait uses `Marker` as
// a free trait parameter with two impls: one for Val (owned container,
// `Fn(A) -> B`) and one for Ref (borrowed container, `Fn(&A) -> B`).
// When combined with a InferableBrand trait that has Brand as a separate free
// trait parameter, the solver sees both Val and Ref impls as candidates
// alongside multiple Brand candidates. This "cross-competition" blocks
// Ref + multi-brand inference.
//
// -- Hypothesis --
//
// Make Marker an ASSOCIATED TYPE of the dispatch trait (rather than a
// free parameter) so that Val and Ref impls become distinguishable
// purely by their Self-type patterns: the Val impl's Self-type pattern
// is `Apply!(Brand::Of<A>)` (not a reference); the Ref impl's is
// `&'b Apply!(Brand::Of<A>)` (a reference). The solver should
// eliminate the irrelevant impl as soon as FA's reference-ness is
// known, without competing Marker candidates.
//
// ========================================================================
// FINDING: HYPOTHESIS REJECTED.
// ========================================================================
//
// Coherence (E0119) rejects the two AssocMarkerDispatch impls:
//
//   error[E0119]: conflicting implementations of trait
//     `AssocMarkerDispatch<'_, _, _, _, <_ as Kind>::Of<'_, _>>`
//
// The Val and Ref impls have Self-type patterns
// `Apply!(Brand::Of<A>)` vs `&'b Apply!(Brand::Of<A>)` which are
// structurally distinct (one is a reference, the other isn't). But
// both contain associated-type projections (`Brand::Of<...>`), and
// Rust's coherence checker is conservative with projection types: it
// cannot prove non-overlap when projections could in principle
// normalise to either shape.
//
// The production `FunctorDispatch` avoids this because Marker is a
// free trait parameter whose two impls supply distinct concrete values
// (`Val` vs `Ref`). That distinction lives in the trait-argument
// tuple, which coherence CAN reason about. Moving Marker to an
// associated type removes that structural distinction and trips
// coherence.
//
// General pattern: on stable Rust, when a multi-valued disambiguator
// (Brand or Marker) is moved from a free trait parameter into an
// associated-type projection on a trait whose impls would then overlap
// in trait-argument shape, coherence rejects the result.
//
// NOTE: a subsequent POC (`slot_marker_via_slot_poc.rs`) found a
// different approach that works: attach Marker as an associated type
// of the SLOT trait (not the dispatch trait), where InferableBrand already has
// distinct Brand parameter values between impls. InferableBrand's `&T` blanket
// gives `type Marker = Ref` uniformly; direct impls give
// `type Marker = Val`. The Marker projection commits from FA's
// reference-ness before Brand resolution begins, eliminating
// cross-competition.
//
// Kept as a documentation artifact. The Ref impl below is commented
// out to keep the file in a buildable state while still preserving
// the structural demonstration of where coherence rejects the
// combination.

#![allow(unused_imports, reason = "Kind is used inside Apply! macro expansion")]

use {
	fp_library::{
		brands::{
			OptionBrand,
			ResultErrAppliedBrand,
			ResultOkAppliedBrand,
			VecBrand,
		},
		classes::{
			Functor,
			RefFunctor,
		},
		kinds::Kind_cdc7cd43dac7585f,
	},
	fp_macros::{
		Apply,
		Kind,
	},
};

// -------------------------------------------------------------------------
// InferableBrand trait (trait-parameter Brand, for coherence).
// -------------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub trait InferableBrand_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
}

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

impl<'a, T: ?Sized, Brand, A: 'a> InferableBrand_cdc7cd43dac7585f<'a, Brand, A> for &T
where
	T: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>,
	Brand: Kind_cdc7cd43dac7585f,
{
}

// -------------------------------------------------------------------------
// Custom dispatch trait with Marker as an ASSOCIATED TYPE.
//
// Compared to the production FunctorDispatch (where Marker is a trait
// parameter), this shape forces the Val/Ref distinction to be encoded
// in the Self-type pattern alone. A closure with a matching Fn(A)
// signature can only satisfy the Val impl; a closure with Fn(&A) can
// only satisfy the Ref impl. The solver doesn't have a free Marker
// dimension to search over.
// -------------------------------------------------------------------------

pub struct ValMark;
pub struct RefMark;

pub trait AssocMarkerDispatch<'a, Brand, A: 'a, B: 'a, FA>
where
	Brand: Kind_cdc7cd43dac7585f, {
	type Marker;
	fn dispatch(
		self,
		fa: FA,
	) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>);
}

// Val impl: closure takes owned A, container is owned.
impl<'a, Brand, A, B, F>
	AssocMarkerDispatch<
		'a,
		Brand,
		A,
		B,
		Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
	> for F
where
	Brand: Functor,
	A: 'a,
	B: 'a,
	F: Fn(A) -> B + 'a,
{
	type Marker = ValMark;

	fn dispatch(
		self,
		fa: Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>) {
		Brand::map(self, fa)
	}
}

// Ref impl: closure takes &A, container is &T.
//
// Uncommenting this impl alongside the Val impl above produces E0119
// on stable rustc. See top-of-file explanation.
//
// impl<'a, 'b, Brand, A, B, F>
//     AssocMarkerDispatch<
//         'a, Brand, A, B,
//         &'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
//     > for F
// where
//     Brand: RefFunctor,
//     A: 'a, B: 'a,
//     F: Fn(&A) -> B + 'a,
// {
//     type Marker = RefMark;
//     fn dispatch(
//         self,
//         fa: &'b Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, A>),
//     ) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>) {
//         Brand::ref_map(self, fa)
//     }
// }

// -------------------------------------------------------------------------
// Unified map function: Marker is no longer a free type parameter here.
// -------------------------------------------------------------------------

pub fn map<'a, FA, A: 'a, B: 'a, Brand>(
	f: impl AssocMarkerDispatch<'a, Brand, A, B, FA>,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
	f.dispatch(fa)
}

// -------------------------------------------------------------------------
// Val tests.
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
fn val_result_ok() {
	let r = map(|x: i32| x + 1, Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

#[test]
fn val_result_err() {
	let r: Result<i32, usize> = map(|e: String| e.len(), Err::<i32, String>("hi".into()));
	assert_eq!(r, Err(2));
}

// -------------------------------------------------------------------------
// Ref tests would go here but require the Ref AssocMarkerDispatch impl
// above to be uncommented. See top-of-file explanation: uncommenting
// both impls produces E0119 on stable rustc, so the Val-only surface
// is the most that compiles. The tests for Ref + multi-brand would
// have been structured as below:
//
//     #[test]
//     fn ref_result_ok_mapping() {
//         let ok: Result<i32, String> = Ok(5);
//         let r: Result<i32, String> = map(|x: &i32| *x + 1, &ok);
//         assert_eq!(r, Ok(6));
//     }
//
// Whether this would have *inferred* correctly is moot - coherence
// rejects the impl combination before inference gets a chance.
// -------------------------------------------------------------------------
