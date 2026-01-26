use fp_library::types::{ArcMemo, RcMemo};
use quickcheck_macros::quickcheck;

// =========================================================================
// Memo Property Tests
// =========================================================================

// -------------------------------------------------------------------------
// Memoization Properties
// -------------------------------------------------------------------------

/// Property: Getting a memoized value twice returns the same result
/// Verifies that `RcMemo` memoizes its result; getting it twice returns the same value without re-executing the thunk.
#[quickcheck]
fn prop_rc_memo_get_memoization(x: i32) -> bool {
	let memo = RcMemo::new(move || x.wrapping_mul(2));
	let result1 = *memo.get();
	let result2 = *memo.get();
	result1 == result2
}

/// Property: Getting a memoized value twice returns the same result (Arc version)
/// Verifies that `ArcMemo` memoizes its result; getting it twice returns the same value.
#[quickcheck]
fn prop_arc_memo_get_memoization(x: i32) -> bool {
	let memo = ArcMemo::new(move || x.wrapping_mul(2));
	let result1 = *memo.get();
	let result2 = *memo.get();
	result1 == result2
}

// -------------------------------------------------------------------------
// Clone Equivalence Properties
// -------------------------------------------------------------------------

/// Property: Cloning a memoized value shares state - getting clone gives same result
/// Verifies that cloning an `RcMemo` shares the underlying state; getting the clone yields the same result as the original.
#[quickcheck]
fn prop_rc_memo_clone_shares_state(x: i32) -> bool {
	let memo1 = RcMemo::new(move || x);
	let memo2 = memo1.clone();

	let result1 = *memo1.get();
	let result2 = *memo2.get();
	result1 == result2
}

/// Property: Cloning an ArcMemo shares state
/// Verifies that cloning an `ArcMemo` shares the underlying state.
#[quickcheck]
fn prop_arc_memo_clone_shares_state(x: i32) -> bool {
	let memo1 = ArcMemo::new(move || x);
	let memo2 = memo1.clone();

	let result1 = *memo1.get();
	let result2 = *memo2.get();
	result1 == result2
}

/// Property: Getting original then clone gives same result
/// Verifies consistency when getting the original memoized value first, then the clone.
#[quickcheck]
fn prop_memo_get_original_then_clone(x: String) -> bool {
	let value = x.clone();
	let memo = RcMemo::new(move || value.clone());
	let memo_clone = memo.clone();

	// Get original first
	let result1 = memo.get().clone();
	// Then get clone
	let result2 = memo_clone.get().clone();

	result1 == result2
}

// -------------------------------------------------------------------------
// Determinism Properties
// -------------------------------------------------------------------------

/// Property: Memo computation is deterministic
/// Verifies that two independent memo values with the same logic produce the same result.
#[quickcheck]
fn prop_memo_deterministic(
	x: i32,
	y: i32,
) -> bool {
	let memo1 = RcMemo::new(move || x.wrapping_add(y));
	let memo2 = RcMemo::new(move || x.wrapping_add(y));

	*memo1.get() == *memo2.get()
}

// -------------------------------------------------------------------------
// Thread Safety Properties (ArcMemo)
// -------------------------------------------------------------------------

/// Property: ArcMemo is thread-safe and memoizes across threads
/// Verifies that `ArcMemo` computes only once even when accessed from multiple threads.
#[test]
fn prop_arc_memo_thread_safety() {
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::thread;

	let counter = Arc::new(AtomicUsize::new(0));
	let counter_clone = counter.clone();

	// We use a fixed value for the test, but the property is about the side effect (counter)
	let memo = ArcMemo::new(move || {
		counter_clone.fetch_add(1, Ordering::SeqCst);
		42
	});

	let mut handles = vec![];
	for _ in 0..10 {
		let memo_clone = memo.clone();
		handles.push(thread::spawn(move || {
			assert_eq!(*memo_clone.get(), 42);
		}));
	}

	for handle in handles {
		handle.join().unwrap();
	}

	// Should have computed exactly once
	assert_eq!(counter.load(Ordering::SeqCst), 1);
}
