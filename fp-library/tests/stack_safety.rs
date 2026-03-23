//! Stack safety tests for `Trampoline`, `Thunk`, and `TryTrampoline`.
//!
//! This module contains tests to verify that trampolined and tail-recursive
//! computations are stack-safe for deep recursion, deep bind chains, and
//! deep defer chains.

use fp_library::types::{
	Step,
	Trampoline,
	TryTrampoline,
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
					Trampoline::pure(Step::Done(0))
				} else {
					Trampoline::pure(Step::Loop(n - 1))
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
		|n: u64| pure::<ThunkBrand, _>(if n == 0 { Step::Done(0u64) } else { Step::Loop(n - 1) }),
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
				TryTrampoline::ok(Step::Done(acc))
			} else {
				TryTrampoline::ok(Step::Loop((remaining - 1, acc.wrapping_add(remaining))))
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
