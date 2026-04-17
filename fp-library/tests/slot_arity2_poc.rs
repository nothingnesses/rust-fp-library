// Arity-2 InferableBrand for Bifunctor POC.
//
// -- Background --
//
// The InferableBrand trait carries Brand as a trait parameter (for coherence)
// and Marker as an associated type projected from FA's reference-ness
// (blanket for `&T` -> Ref; direct impls for owned types -> Val).
// This lets a unified `map(f, fa)` signature commit Marker early from
// FA alone, resolve Val/Ref dispatch, then pin Brand from the closure
// input.
//
// This POC validates the pattern at arity 2, the Kind signature used
// by `Bifunctor::bimap` and `RefBifunctor::ref_bimap`. For arity 2:
//   - `Slot2<'a, Brand, A, B>` mirrors the arity-2 Kind signature.
//   - `BimapDispatch<..., Marker>` routes to `Bifunctor::bimap` (Val)
//     or `RefBifunctor::ref_bimap` (Ref) based on the projected
//     `<FA as Slot2<...>>::Marker`.
//
// Arity-2 brands like `ResultBrand` are currently single-brand at
// arity 2, so multi-brand disambiguation is not exercised here; the
// test confirms the structural generality of the trait family
// across arities.
//
// -- Finding --
//
// CONFIRMED. All 6 tests pass on stable rustc (Val and Ref for
// Result, including type-changing transformations). The InferableBrand pattern
// generalises mechanically from arity 1 to arity 2.

#![allow(unused_imports, reason = "Kind is used inside Apply! macro expansion")]

use {
	fp_library::{
		brands::ResultBrand,
		dispatch::{
			Ref,
			Val,
			bifunctor::BimapDispatch,
		},
		kinds::Kind_266801a817966495,
	},
	fp_macros::{
		Apply,
		Kind,
	},
};

// -------------------------------------------------------------------------
// Slot2 trait: arity-2 analogue of POC 5's SlotM.
// -------------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub trait Slot2_266801a817966495<'a, Brand, A: 'a, B: 'a>
where
	Brand: Kind_266801a817966495, {
	type Marker;
}

// Direct impls: owned types set Marker = Val.
//
// Note: ResultBrand::Of<'a, A, B> = Result<B, A>. The first brand
// parameter (A) corresponds to Rust's Err side; the second (B) to the
// Ok side. The Self-type must match that shape.
impl<'a, Err: 'a, Ok: 'a> Slot2_266801a817966495<'a, ResultBrand, Err, Ok> for Result<Ok, Err> {
	type Marker = Val;
}

// Blanket for references: Marker = Ref uniformly.
impl<'a, T: ?Sized, Brand, A: 'a, B: 'a> Slot2_266801a817966495<'a, Brand, A, B> for &T
where
	T: Slot2_266801a817966495<'a, Brand, A, B>,
	Brand: Kind_266801a817966495,
{
	type Marker = Ref;
}

// -------------------------------------------------------------------------
// Unified bimap: Marker projected from Slot2.
// -------------------------------------------------------------------------

pub fn bimap<'a, FA, A: 'a, B: 'a, C: 'a, D: 'a, Brand>(
	fg: impl BimapDispatch<
		'a,
		Brand,
		A,
		B,
		C,
		D,
		FA,
		<FA as Slot2_266801a817966495<'a, Brand, A, C>>::Marker,
	>,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, A: 'a, B: 'a>: 'a;)>::Of<'a, B, D>)
where
	Brand: Kind_266801a817966495,
	FA: Slot2_266801a817966495<'a, Brand, A, C>, {
	fg.dispatch(fa)
}

// -------------------------------------------------------------------------
// Tests.
// -------------------------------------------------------------------------

#[test]
fn val_result_ok() {
	let x: Result<i32, i32> = Ok(5);
	let y: Result<i32, i32> = bimap((|e: i32| e + 1, |s: i32| s * 2), x);
	assert_eq!(y, Ok(10));
}

#[test]
fn val_result_err() {
	let x: Result<i32, i32> = Err(3);
	let y: Result<i32, i32> = bimap((|e: i32| e + 1, |s: i32| s * 2), x);
	assert_eq!(y, Err(4));
}

#[test]
fn val_result_type_change() {
	// Result<i32, String> has Ok(i32), Err(String). For bimap, closures
	// are (err_fn, ok_fn): err_fn: String -> usize, ok_fn: i32 -> String.
	// Output: Result<String, usize>.
	let x: Result<i32, String> = Ok(42);
	let y: Result<String, usize> = bimap((|e: String| e.len(), |s: i32| s.to_string()), x);
	assert_eq!(y, Ok("42".to_string()));
}

#[test]
fn ref_result_ok() {
	let x: Result<i32, i32> = Ok(5);
	let y: Result<i32, i32> = bimap((|e: &i32| *e + 1, |s: &i32| *s * 2), &x);
	assert_eq!(y, Ok(10));
}

#[test]
fn ref_result_err() {
	let x: Result<i32, i32> = Err(3);
	let y: Result<i32, i32> = bimap((|e: &i32| *e + 1, |s: &i32| *s * 2), &x);
	assert_eq!(y, Err(4));
}

#[test]
fn ref_result_type_change() {
	let x: Result<i32, String> = Ok(42);
	let y: Result<String, usize> = bimap((|e: &String| e.len(), |s: &i32| s.to_string()), &x);
	assert_eq!(y, Ok("42".to_string()));
}
