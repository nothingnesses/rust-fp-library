// Sixth Slot POC: validates the Marker-via-Slot pattern at arity 2 for
// `Bifunctor::bimap` and `RefBifunctor::ref_bimap`.
//
// POC 5 established that a unified `map(f, fa)` signature can handle
// all four Val/Ref x single/multi-brand cases at arity 1 (with
// lifetime) when:
//   - Slot carries Brand as a trait parameter (for coherence), and
//   - Slot carries Marker as an associated-type projection keyed on
//     FA's reference-ness (for early Marker commitment in dispatch).
//
// This POC verifies the pattern scales to arity 2 - the Kind signature
// used by Bifunctor, Bifoldable, and Bitraversable. Arity-2 brands like
// `ResultBrand` have only a single arity-2 brand (not multi-brand), so
// this POC is primarily about confirming:
//   (1) Slot2 compiles with Brand as trait parameter and Marker as
//       associated type at arity 2;
//   (2) The &T blanket projects Marker = Ref uniformly at arity 2;
//   (3) Direct impls project Marker = Val uniformly at arity 2;
//   (4) A unified `bimap(fg, fa)` signature dispatches Val and Ref
//       correctly through Slot2's Marker projection + production
//       `BimapDispatch`.
//
// If this works, Path 3 (eliminate InferableBrand in favour of Slot)
// generalises mechanically to every Kind arity used by the library.

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
