// Feasibility POC for "approach 5: closure-directed brand inference" from
// docs/plans/brand-inference/analysis/multi-brand-evaluation.md.
//
// Hypothesis: when a container type (e.g. `Result<A, E>`) has multiple valid
// arity-1 brands, the closure's input type can disambiguate which brand to
// use. A `Slot<Brand, A>` trait with one impl per candidate brand lets trait
// selection unify on the unique `(Brand, A)` pair that matches the concrete
// container type.
//
// Expected positive results: non-diagonal cases (container type parameters
// differ in type, or the closure input matches only one slot) resolve
// unambiguously on stable rustc without turbofish or unstable features.
//
// Expected failure: the diagonal `Result<T, T>` case with a closure
// consuming `T` produces two equally-valid `Slot` impls and inference fails.
// The failing case is kept commented out with the expected diagnostic below.
//
// This POC intentionally does not use the library's HKT machinery; it
// reproduces the minimum pattern needed to exercise Rust's trait selection.

use std::marker::PhantomData;

// -- Standalone brands (no HKT machinery) --

struct OptionBrand;
struct ResultOkBrand<E>(#[allow(dead_code)] PhantomData<E>);
struct ResultErrBrand<T>(#[allow(dead_code)] PhantomData<T>);

// -- The Slot disambiguation trait --
//
// `Slot<Brand, A> for Container` declares: "Container is the `Of<A>` projection
// of Brand, with Brand's partially-applied parameters derived from Container's
// remaining type parameters."

trait Slot<Brand, A> {
	type Out<B>;
	fn slot_map<B>(
		fa: Self,
		f: impl Fn(A) -> B,
	) -> Self::Out<B>;
}

// -- Single-brand sanity check: Option --

impl<A> Slot<OptionBrand, A> for Option<A> {
	type Out<B> = Option<B>;

	fn slot_map<B>(
		fa: Self,
		f: impl Fn(A) -> B,
	) -> Option<B> {
		fa.map(f)
	}
}

// -- Multi-brand case: Result with both slots available --

impl<A, E> Slot<ResultOkBrand<E>, A> for Result<A, E> {
	type Out<B> = Result<B, E>;

	fn slot_map<B>(
		fa: Self,
		f: impl Fn(A) -> B,
	) -> Result<B, E> {
		fa.map(f)
	}
}

impl<T, A> Slot<ResultErrBrand<T>, A> for Result<T, A> {
	type Out<B> = Result<T, B>;

	fn slot_map<B>(
		fa: Self,
		f: impl Fn(A) -> B,
	) -> Result<T, B> {
		fa.map_err(f)
	}
}

// -- The polymorphic map function --
//
// `Brand` appears only in the trait bound and the return type; trait selection
// determines it from the `(FA, A)` pair.

fn map<FA, A, B, Brand>(
	f: impl Fn(A) -> B,
	fa: FA,
) -> <FA as Slot<Brand, A>>::Out<B>
where
	FA: Slot<Brand, A>, {
	FA::slot_map(fa, f)
}

// -- Tests: single-brand sanity check --

#[test]
fn option_map_infers_brand() {
	let r: Option<i32> = map(|x: i32| x * 2, Some(5));
	assert_eq!(r, Some(10));
}

#[test]
fn option_map_different_output_type() {
	let r: Option<String> = map(|x: i32| x.to_string(), Some(5));
	assert_eq!(r, Some("5".to_string()));
}

// -- Tests: Result with distinct type parameters --
//
// Closure input type uniquely identifies the slot.

#[test]
fn result_ok_mapping_via_closure_input_i32() {
	// Closure takes i32, which unifies only with the Ok slot of
	// Result<i32, String>. Brand resolves to ResultOkBrand<String>.
	let r = map(|x: i32| x + 1, Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

#[test]
fn result_err_mapping_via_closure_input_string() {
	// Closure takes String, which unifies only with the Err slot of
	// Result<i32, String>. Brand resolves to ResultErrBrand<i32>.
	let r: Result<i32, usize> = map(|e: String| e.len(), Err::<i32, String>("hi".into()));
	assert_eq!(r, Err(2));
}

#[test]
fn result_ok_different_output_type() {
	// Output type B differs from input A. Verifies B also flows through
	// Slot::Out<B> correctly.
	let r = map(|x: i32| x.to_string(), Ok::<i32, String>(5));
	assert_eq!(r, Ok("5".to_string()));
}

#[test]
fn result_err_with_err_value_present() {
	// Mapping over Err when the concrete value is an Err variant.
	let r: Result<i32, usize> = map(|e: String| e.len(), Err::<i32, String>("fail".into()));
	assert_eq!(r, Err(4));
}

#[test]
fn result_ok_with_err_value_present() {
	// Mapping over Ok when the concrete value is an Err variant. The
	// transformation is still selected by the closure input type, not the
	// variant. Uses `Err::<i32, String>(...)` so the two slots have distinct
	// types; a `Result<T, T>` value would be diagonal-ambiguous regardless of
	// which variant is present.
	let r: Result<String, String> = map(|x: i32| x.to_string(), Err::<i32, String>("fail".into()));
	assert_eq!(r, Err("fail".to_string()));
}

// -- Diagonal failure case --
//
// When both type parameters of Result are equal and the closure input matches
// that common type, both Slot impls apply and trait selection is ambiguous.
// Uncomment to observe the compile error:
//
// #[test]
// fn result_diagonal_case_is_ambiguous() {
//     let _ = map(|x: i32| x + 1, Ok::<i32, i32>(5));
//
//     // Verified 2026-04-16 on rustc in-tree; compile error:
//     //   error[E0283]: type annotations needed
//     //     cannot infer type of the type parameter `Brand` declared on the
//     //     function `map`
//     //   note: multiple `impl`s satisfying `Result<i32, i32>: Slot<_, i32>` found
//     //     impl<A, E> Slot<ResultOkBrand<E>, A> for Result<A, E>
//     //     impl<T, A> Slot<ResultErrBrand<T>, A> for Result<T, A>
// }
//
// Same pattern fails for any `(T, T)`-shaped multi-brand container: `Pair<T, T>`,
// `(T, T)`, `ControlFlow<T, T>`, `TryThunk<T, T>` with a closure consuming T.
