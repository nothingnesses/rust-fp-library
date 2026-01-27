//! Integration tests for the pointer abstraction.

use fp_library::{brands::*, functions::*, types::*};
use std::sync::{Arc, Mutex};
use std::thread;

/// Tests basic functionality of `ArcBrand` pointer creation and dereferencing.
///
/// Verifies that:
/// 1. `send_ref_counted_pointer_new` creates a valid pointer.
/// 2. The pointer can be dereferenced to access the value.
/// 3. Cloning the pointer works and preserves the value.
#[test]
fn test_arc_ptr_basic() {
	let ptr = send_ref_counted_pointer_new::<ArcBrand, _>(42);
	assert_eq!(*ptr, 42);
	let clone = ptr.clone();
	assert_eq!(*clone, 42);
}

/// Tests basic functionality of `ArcFnBrand` cloneable function.
///
/// Verifies that:
/// 1. `send_cloneable_fn_new` creates a callable function wrapper.
/// 2. The wrapper can be called to produce a result.
/// 3. Cloning the wrapper works and the clone produces the same result.
#[test]
fn test_arc_fn_basic() {
	let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x + 1);
	assert_eq!(f(1), 2);
	let clone = f.clone();
	assert_eq!(clone(1), 2);
}

/// Tests thread safety of `ArcFnBrand`.
///
/// Verifies that:
/// 1. The function wrapper is `Send` (can be moved to another thread).
/// 2. The function can be executed in a separate thread.
#[test]
fn test_arc_fn_thread_safety() {
	let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x + 1);
	let handle = thread::spawn(move || f(10));
	assert_eq!(handle.join().unwrap(), 11);
}

/// Tests basic functionality of `ArcLazy`.
///
/// Verifies that:
/// 1. `ArcLazy` can be created with a thread-safe thunk.
/// 2. `Memo::get` correctly evaluates the thunk and returns the value.
#[test]
fn test_arc_memo_basic() {
	let memo = ArcLazy::new(|| 42);
	assert_eq!(*memo.get(), 42);
}

/// Tests shared memoization semantics of `ArcLazy`.
///
/// Verifies that:
/// 1. The thunk is executed only once, even when accessed via multiple clones of the `Memo` value.
/// 2. The result is cached and shared across clones.
///
/// This ensures that `ArcLazy` implements "call-by-need" semantics with shared state.
#[test]
fn test_arc_memo_shared_memoization() {
	let counter = Arc::new(Mutex::new(0));
	let counter_clone = counter.clone();

	let memo = ArcLazy::new(move || {
		let mut guard = counter_clone.lock().unwrap();
		*guard += 1;
		42
	});

	let memo_clone = memo.clone();

	assert_eq!(*counter.lock().unwrap(), 0);
	assert_eq!(*memo.get(), 42);
	assert_eq!(*counter.lock().unwrap(), 1);

	// Should use cached value
	assert_eq!(*memo_clone.get(), 42);
	assert_eq!(*counter.lock().unwrap(), 1);
}

/// Tests thread safety of `ArcLazy`.
///
/// Verifies that:
/// 1. `ArcLazy` is `Send` and `Sync` (when T is Send + Sync).
/// 2. It can be cloned and sent to another thread.
/// 3. It can be forced in a separate thread.
#[test]
fn test_arc_memo_thread_safety() {
	let memo = ArcLazy::new(|| 42);
	let memo_clone = memo.clone();

	let handle = thread::spawn(move || *memo_clone.get());

	assert_eq!(handle.join().unwrap(), 42);
	assert_eq!(*memo.get(), 42);
}

/// Tests basic functionality of `RcLazy`.
///
/// Verifies that:
/// 1. `RcLazy` can be created with a thunk.
/// 2. `Memo::get` correctly evaluates the thunk and returns the value.
#[test]
fn test_rc_memo_basic() {
	let memo = RcLazy::new(|| 42);
	assert_eq!(*memo.get(), 42);
}

/// Tests shared memoization semantics of `RcLazy`.
///
/// Verifies that:
/// 1. The thunk is executed only once, even when accessed via multiple clones of the `Memo` value.
/// 2. The result is cached and shared across clones.
///
/// This ensures that `RcLazy` implements "call-by-need" semantics with shared state.
#[test]
fn test_rc_memo_shared_memoization() {
	use std::cell::RefCell;
	use std::rc::Rc;

	let counter = Rc::new(RefCell::new(0));
	let counter_clone = counter.clone();

	let memo = RcLazy::new(move || {
		*counter_clone.borrow_mut() += 1;
		42
	});

	let memo_clone = memo.clone();

	assert_eq!(*counter.borrow(), 0);
	assert_eq!(*memo.get(), 42);
	assert_eq!(*counter.borrow(), 1);

	// Should use cached value (shared memoization)
	assert_eq!(*memo_clone.get(), 42);
	assert_eq!(*counter.borrow(), 1);
}
