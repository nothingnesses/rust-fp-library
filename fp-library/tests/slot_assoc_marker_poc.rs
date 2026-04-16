// Fourth Slot POC: tests whether a unified `map` can handle Ref +
// multi-brand by making `Marker` an ASSOCIATED TYPE of the dispatch
// trait rather than a free trait parameter.
//
// Prior POCs established:
//   - Slot with Brand as a trait PARAMETER: coherence OK, but Val/Ref
//     cross-competition defeats Ref + multi-brand inference.
//   - SelectBrand with Brand as an ASSOCIATED-TYPE PROJECTION:
//     inference OK (resolves Ref + multi-brand) but coherence rejects
//     the two multi-brand impls.
//   - Pinning Marker to Ref (hard-coded) resolves the Ref + multi-brand
//     case but loses the unified signature.
//
// Hypothesis: the Val/Ref cross-competition in Slot+FunctorDispatch
// arises because Marker is a free trait parameter. Making Marker an
// associated type should force Rust to distinguish Val and Ref impls
// purely by their Self-type patterns (by-value vs by-reference),
// eliminating the ambiguity.
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
// `Apply!(Brand::Of<A>)` vs `&'b Apply!(Brand::Of<A>)` - structurally
// distinct (one is a reference, the other isn't). But both contain
// associated-type projections (`Brand::Of<...>`), and Rust's coherence
// checker is conservative with projection types: it cannot prove
// non-overlap syntactically when projections could in principle
// normalise to either shape.
//
// The production `FunctorDispatch` avoids this coherence issue
// specifically because Marker is a free trait parameter whose two
// impls supply distinct concrete values (`Val` vs `Ref`). That
// distinction lives in the trait-argument tuple, which coherence CAN
// reason about. Moving Marker to an associated type removes that
// structural distinction and trips coherence.
//
// Combined with the SelectBrand POC's finding (coherence also rejects
// an associated-type Brand), this confirms a general pattern: on
// stable Rust, whenever we try to remove a trait parameter (Brand or
// Marker) by projecting it as an associated type, coherence rejects
// the resulting impls for multi-brand types. The parameter-as-
// disambiguator role is load-bearing.
//
// The implication for the unified `map` goal: there is no way on
// stable Rust to have:
//
//     (1) coherence-safe multi-brand impls,
//     (2) inference-based Brand resolution for Ref + multi-brand,
//         and
//     (3) a single unified Val+Ref entry point
//
// simultaneously. Each of SelectBrand and AssocMarkerDispatch
// sacrifices (1); Slot sacrifices (2) (for the Ref + multi-brand
// case specifically); the only stable-Rust solution that preserves
// (1) and (2) sacrifices (3) by splitting Val and Ref into separate
// inference entry points.
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
// Slot trait (trait-parameter Brand, for coherence).
// -------------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub trait Slot_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
}

impl<'a, A: 'a> Slot_cdc7cd43dac7585f<'a, OptionBrand, A> for Option<A> {}
impl<'a, A: 'a> Slot_cdc7cd43dac7585f<'a, VecBrand, A> for Vec<A> {}

impl<'a, A: 'a, E: 'static> Slot_cdc7cd43dac7585f<'a, ResultErrAppliedBrand<E>, A>
	for Result<A, E>
{
}

impl<'a, T: 'static, A: 'a> Slot_cdc7cd43dac7585f<'a, ResultOkAppliedBrand<T>, A> for Result<T, A> {}

impl<'a, T: ?Sized, Brand, A: 'a> Slot_cdc7cd43dac7585f<'a, Brand, A> for &T
where
	T: Slot_cdc7cd43dac7585f<'a, Brand, A>,
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
	FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
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
