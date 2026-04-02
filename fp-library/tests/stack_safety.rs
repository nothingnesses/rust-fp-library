//! Stack safety tests for `Trampoline`, `Thunk`, `TryTrampoline`,
//! `RcCoyoneda`, and `ArcCoyoneda`.
//!
//! This module contains tests to verify that trampolined and tail-recursive
//! computations are stack-safe for deep recursion, deep bind chains, and
//! deep defer chains.

use {
	core::ops::ControlFlow,
	fp_library::{
		brands::ThunkBrand,
		types::{
			Free,
			Thunk,
			Trampoline,
			TryTrampoline,
		},
	},
};

/// Tests deep recursion using `tail_rec_m`.
///
/// Verifies that `tail_rec_m` can handle 1,000,000 iterations without stack overflow.
#[test]
fn test_deep_recursion() {
	fn count_down(n: u64) -> Trampoline<u64> {
		Trampoline::tail_rec_m(
			|n| {
				if n == 0 {
					Trampoline::pure(ControlFlow::Break(0))
				} else {
					Trampoline::pure(ControlFlow::Continue(n - 1))
				}
			},
			n,
		)
	}

	// 1,000,000 iterations
	assert_eq!(count_down(1_000_000).evaluate(), 0);
}

/// Tests deep bind chains using.
///
/// Verifies that a chain of 100,000 `bind` calls does not cause stack overflow.
#[test]
fn test_deep_bind_chain() {
	let mut task = Trampoline::pure(0);
	for _ in 0 .. 100_000 {
		task = task.bind(|x| Trampoline::pure(x + 1));
	}
	assert_eq!(task.evaluate(), 100_000);
}

/// Tests deep defer chains.
///
/// Verifies that a chain of 100,000 `defer` calls does not cause stack overflow.
#[test]
fn test_deep_defer_chain() {
	fn recursive_defer(n: u64) -> Trampoline<u64> {
		if n == 0 {
			Trampoline::pure(0)
		} else {
			Trampoline::defer(move || recursive_defer(n - 1).map(|x| x + 1))
		}
	}

	// 100,000 iterations
	assert_eq!(recursive_defer(100_000).evaluate(), 100_000);
}

/// Tests `tail_rec_m` on `ThunkBrand` at depth 1,000,000.
///
/// Verifies that the loop-based `MonadRec` implementation for `ThunkBrand`
/// is stack-safe for deep recursion.
#[test]
fn test_thunk_tail_rec_m_stack_safety() {
	use fp_library::{
		brands::ThunkBrand,
		functions::*,
	};

	let result = tail_rec_m::<ThunkBrand, _, _>(
		|n: u64| {
			pure::<ThunkBrand, _>(
				if n == 0 { ControlFlow::Break(0u64) } else { ControlFlow::Continue(n - 1) },
			)
		},
		1_000_000u64,
	);
	assert_eq!(result.evaluate(), 0);
}

/// Tests `TryTrampoline` stack safety at 100,000+ depth.
///
/// Verifies that `TryTrampoline::tail_rec_m` does not overflow the stack
/// with deep recursion, producing a successful result.
#[test]
fn test_try_trampoline_stack_safety() {
	let n = 200_000u64;
	let task: TryTrampoline<u64, String> = TryTrampoline::tail_rec_m(
		|(remaining, acc)| {
			if remaining == 0 {
				TryTrampoline::ok(ControlFlow::Break(acc))
			} else {
				TryTrampoline::ok(ControlFlow::Continue((
					remaining - 1,
					acc.wrapping_add(remaining),
				)))
			}
		},
		(n, 0u64),
	);

	assert_eq!(task.evaluate(), Ok(n.wrapping_mul(n.wrapping_add(1)) / 2));
}

/// Tests `TryTrampoline` deep bind chain stack safety.
///
/// Verifies that a chain of 100,000 `bind` calls on TryTrampoline
/// does not cause stack overflow.
#[test]
fn test_try_trampoline_deep_bind_chain() {
	let mut task: TryTrampoline<i64, String> = TryTrampoline::ok(0);
	for _ in 0 .. 100_000 {
		task = task.bind(|x| TryTrampoline::ok(x + 1));
	}
	assert_eq!(task.evaluate(), Ok(100_000));
}

/// Tests that dropping a deeply nested Wrap-only `Free` chain does not overflow the stack.
///
/// Constructs 100,000 nested `Free::wrap(Thunk::new(|| ...))` layers and drops them.
/// Without iterative drop, this would cause a stack overflow.
#[test]
fn test_deep_wrap_chain_drop() {
	let depth = 100_000;
	let mut free: Free<ThunkBrand, i32> = Free::pure(42);
	for _ in 0 .. depth {
		let inner = free;
		free = Free::<ThunkBrand, _>::wrap(Thunk::new(move || inner));
	}
	// Dropping `free` should not overflow the stack.
	drop(free);
}

/// Tests that evaluating a deeply nested Wrap-only `Free` chain works correctly.
///
/// This complements the drop test by verifying the value is preserved through
/// 100,000 nested Wrap layers.
#[test]
fn test_deep_wrap_chain_evaluate() {
	let depth = 100_000;
	let mut free: Free<ThunkBrand, i32> = Free::pure(42);
	for _ in 0 .. depth {
		let inner = free;
		free = Free::<ThunkBrand, _>::wrap(Thunk::new(move || inner));
	}
	assert_eq!(free.evaluate(), 42);
}

// -- RcCoyoneda / ArcCoyoneda stack safety --

/// Tests that `RcCoyoneda::collapse` resets the recursion depth.
///
/// Builds a chain of 500 maps using `OptionBrand` (lighter stack frames than
/// `VecBrand`), collapses, then adds 500 more. Without collapse, 1000 layers
/// would risk stack overflow in debug builds. With collapse resetting the depth
/// to 1, each segment of 500 is safe.
#[test]
fn test_rc_coyoneda_collapse_resets_depth() {
	use fp_library::{
		brands::OptionBrand,
		types::RcCoyoneda,
	};

	let mut coyo = RcCoyoneda::<OptionBrand, _>::lift(Some(0i32));
	for _ in 0 .. 500 {
		coyo = coyo.map(|x| x + 1);
	}
	coyo = coyo.collapse();
	for _ in 0 .. 500 {
		coyo = coyo.map(|x| x + 1);
	}
	assert_eq!(coyo.lower_ref(), Some(1000));
}

/// Tests that `ArcCoyoneda::collapse` resets the recursion depth.
#[test]
fn test_arc_coyoneda_collapse_resets_depth() {
	use fp_library::{
		brands::OptionBrand,
		types::ArcCoyoneda,
	};

	let mut coyo = ArcCoyoneda::<OptionBrand, _>::lift(Some(0i32));
	for _ in 0 .. 500 {
		coyo = coyo.map(|x| x + 1);
	}
	coyo = coyo.collapse();
	for _ in 0 .. 500 {
		coyo = coyo.map(|x| x + 1);
	}
	assert_eq!(coyo.lower_ref(), Some(1000));
}

/// Tests that `RcCoyoneda` with stacker handles deeper chains.
///
/// Uses `OptionBrand` (single-element functor) to minimize per-frame stack usage,
/// allowing the stacker to demonstrate its effect at higher depth.
#[cfg(feature = "stacker")]
#[test]
fn test_rc_coyoneda_deep_chain_with_stacker() {
	use fp_library::{
		brands::OptionBrand,
		types::RcCoyoneda,
	};

	let mut coyo = RcCoyoneda::<OptionBrand, _>::lift(Some(0i32));
	for _ in 0 .. 1_000 {
		coyo = coyo.map(|x| x + 1);
	}
	assert_eq!(coyo.lower_ref(), Some(1_000));
}

/// Tests that `ArcCoyoneda` with stacker handles deeper chains.
#[cfg(feature = "stacker")]
#[test]
fn test_arc_coyoneda_deep_chain_with_stacker() {
	use fp_library::{
		brands::OptionBrand,
		types::ArcCoyoneda,
	};

	let mut coyo = ArcCoyoneda::<OptionBrand, _>::lift(Some(0i32));
	for _ in 0 .. 1_000 {
		coyo = coyo.map(|x| x + 1);
	}
	assert_eq!(coyo.lower_ref(), Some(1_000));
}
