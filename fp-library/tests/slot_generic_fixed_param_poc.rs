// Generic fixed-parameter POC.
//
// -- Background --
//
// The Slot-based brand inference mechanism works by letting the
// closure's input type pin A, which then selects a unique Slot impl
// and commits Brand. This is validated for concrete types (e.g.,
// `Result<i32, String>`) in `slot_marker_via_slot_poc.rs`.
//
// A subtler case arises when the multi-brand type has a GENERIC fixed
// parameter. For example:
//
//     fn process<E>(r: Result<i32, E>) { map(|x: i32| x + 1, r) }
//
// Here Result<i32, E> has two Slot impls:
//   - Slot<ResultErrAppliedBrand<E>, i32> for Result<i32, E>.
//     The closure pins A = i32; this impl matches directly.
//   - Slot<ResultOkAppliedBrand<i32>, E> for Result<i32, E>.
//     The closure pins A = i32, but this impl has A = E. It would
//     match only if E = i32, which Rust cannot rule out since E is
//     unconstrained.
//
// The question is whether Rust's trait solver commits to the first
// impl (since the closure concretely pins A = i32 and only the first
// impl matches without imposing additional constraints on E) or
// conservatively refuses due to the potential overlap on the diagonal
// E = i32.
//
// This POC tests several variants of the generic fixed-parameter
// pattern: generic error type, generic success type, both generic,
// Val and Ref dispatch, and with/without trait bounds on the generic
// parameter.
//
// -- Finding --
//
// CONFIRMED: all 9 cases pass on stable rustc 1.94.1. Rust's solver
// correctly commits to the Slot impl whose A matches the closure's
// concrete input type, even when the other impl cannot be statically
// ruled out (because the generic parameter could in principle equal
// the closure's input type). The solver does not require exhaustive
// proof of non-overlap; the concrete match is sufficient to commit.
//
// This holds for all tested variants: generic error, generic success,
// both generic, Val dispatch, Ref dispatch, and with additional trait
// bounds on the generic parameter.

#![allow(unused_imports, reason = "Kind is used inside Apply! macro expansion")]

use {
	fp_library::{
		brands::{
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
	},
	fp_macros::{
		Apply,
		Kind,
	},
};

// -------------------------------------------------------------------------
// Slot trait (same as POC 5, the adopted design).
// -------------------------------------------------------------------------

#[allow(non_camel_case_types)]
pub trait Slot_cdc7cd43dac7585f<'a, Brand, A: 'a>
where
	Brand: Kind_cdc7cd43dac7585f, {
	type Marker;
}

impl<'a, A: 'a> Slot_cdc7cd43dac7585f<'a, OptionBrand, A> for Option<A> {
	type Marker = Val;
}

impl<'a, A: 'a> Slot_cdc7cd43dac7585f<'a, VecBrand, A> for Vec<A> {
	type Marker = Val;
}

impl<'a, A: 'a, E: 'static> Slot_cdc7cd43dac7585f<'a, ResultErrAppliedBrand<E>, A>
	for Result<A, E>
{
	type Marker = Val;
}

impl<'a, T: 'static, A: 'a> Slot_cdc7cd43dac7585f<'a, ResultOkAppliedBrand<T>, A> for Result<T, A> {
	type Marker = Val;
}

impl<'a, T: ?Sized, Brand, A: 'a> Slot_cdc7cd43dac7585f<'a, Brand, A> for &T
where
	T: Slot_cdc7cd43dac7585f<'a, Brand, A>,
	Brand: Kind_cdc7cd43dac7585f,
{
	type Marker = Ref;
}

// -------------------------------------------------------------------------
// Unified map (same as POC 5).
// -------------------------------------------------------------------------

pub fn map<'a, FA, A: 'a, B: 'a, Brand>(
	f: impl FunctorDispatch<'a, Brand, A, B, FA, <FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker>,
	fa: FA,
) -> Apply!(<Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
where
	Brand: Kind_cdc7cd43dac7585f,
	FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
	f.dispatch(fa)
}

// =========================================================================
// Test cases: generic fixed parameter.
// =========================================================================

// -- Case 1: generic error type, concrete Ok type --
// fn process<E>(r: Result<i32, E>) with map(|x: i32| ..., r)
// Expected: ResultErrAppliedBrand<E> selected (A = i32 from closure).

fn generic_err_concrete_ok<E: 'static>(r: Result<i32, E>) -> Result<i32, E> {
	map(|x: i32| x + 1, r)
}

#[test]
fn generic_err_ok_value() {
	let r = generic_err_concrete_ok(Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

#[test]
fn generic_err_err_value() {
	let r = generic_err_concrete_ok(Err::<i32, String>("fail".into()));
	assert_eq!(r, Err("fail".to_string()));
}

// -- Case 2: generic Ok type, concrete error type --
// fn process<T>(r: Result<T, String>) with map(|e: String| ..., r)
// Expected: ResultOkAppliedBrand<T> selected (A = String from closure).

fn generic_ok_concrete_err<T: 'static>(r: Result<T, String>) -> Result<T, usize> {
	map(|e: String| e.len(), r)
}

#[test]
fn generic_ok_err_value() {
	let r = generic_ok_concrete_err(Err::<i32, String>("hello".into()));
	assert_eq!(r, Err(5));
}

#[test]
fn generic_ok_ok_passthrough() {
	let r = generic_ok_concrete_err(Ok::<i32, String>(42));
	assert_eq!(r, Ok(42));
}

// -- Case 3: Ref variant of case 1 --
// Same as case 1 but with a borrowed container.

fn generic_err_concrete_ok_ref<E>(r: &Result<i32, E>) -> Result<i32, E>
where
	E: 'static + Clone, {
	map(|x: &i32| *x + 1, r)
}

#[test]
fn generic_err_ref() {
	let r = generic_err_concrete_ok_ref(&Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

// -- Case 4: Ref variant of case 2 --

fn generic_ok_concrete_err_ref<T: 'static + Clone>(r: &Result<T, String>) -> Result<T, usize> {
	map(|e: &String| e.len(), r)
}

#[test]
fn generic_ok_ref() {
	let r = generic_ok_concrete_err_ref(&Err::<i32, String>("hello".into()));
	assert_eq!(r, Err(5));
}

// -- Case 5: both type params generic, closure annotates one --
// fn process<T, E>(r: Result<T, E>) with map(|x: T| ..., r)
// Expected: ResultErrAppliedBrand<E> selected (A = T from closure).

fn both_generic_map_ok<T: 'static + Clone, E: 'static>(r: Result<T, E>) -> Result<T, E> {
	map(|x: T| x.clone(), r)
}

#[test]
fn test_both_generic_map_ok() {
	let r = both_generic_map_ok(Ok::<i32, String>(10));
	assert_eq!(r, Ok(10));
}

// -- Case 6: both type params generic, closure annotates the other --

fn both_generic_map_err<T: 'static, E: 'static + Clone>(r: Result<T, E>) -> Result<T, E> {
	map(|e: E| e.clone(), r)
}

#[test]
fn test_both_generic_map_err() {
	let r = both_generic_map_err(Err::<i32, String>("hi".into()));
	assert_eq!(r, Err("hi".to_string()));
}

// -- Case 7: generic fixed param with a trait bound --
// Verifies bounds on E don't interfere with Slot resolution.

fn generic_err_with_bound<E: 'static + std::fmt::Debug>(r: Result<i32, E>) -> Result<String, E> {
	map(|x: i32| format!("{x}"), r)
}

#[test]
fn test_generic_err_with_bound() {
	let r = generic_err_with_bound(Ok::<i32, String>(42));
	assert_eq!(r, Ok("42".to_string()));
}
