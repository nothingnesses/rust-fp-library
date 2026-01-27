//! Stack safety tests for `Task`.
//!
//! This module contains tests to verify that `Task` is stack-safe for deep recursion,
//! deep bind chains, and deep defer chains.

use fp_library::types::{Step, Task};

/// Tests deep recursion using `tail_rec_m`.
///
/// Verifies that `tail_rec_m` can handle 1,000,000 iterations without stack overflow.
#[test]
fn test_deep_recursion() {
	fn count_down(n: u64) -> Task<u64> {
		Task::tail_rec_m(
			|n| {
				if n == 0 { Task::pure(Step::Done(0)) } else { Task::pure(Step::Loop(n - 1)) }
			},
			n,
		)
	}

	// 1,000,000 iterations
	assert_eq!(count_down(1_000_000).run(), 0);
}

/// Tests deep bind chains using.
///
/// Verifies that a chain of 100,000 `bind` calls does not cause stack overflow.
#[test]
fn test_deep_bind_chain() {
	let mut task = Task::pure(0);
	for _ in 0..100_000 {
		task = task.bind(|x| Task::pure(x + 1));
	}
	assert_eq!(task.run(), 100_000);
}

/// Tests deep defer chains.
///
/// Verifies that a chain of 100,000 `defer` calls does not cause stack overflow.
#[test]
fn test_deep_defer_chain() {
	fn recursive_defer(n: u64) -> Task<u64> {
		if n == 0 {
			Task::pure(0)
		} else {
			Task::defer(move || recursive_defer(n - 1).map(|x| x + 1))
		}
	}

	// 100,000 iterations
	assert_eq!(recursive_defer(100_000).run(), 100_000);
}
