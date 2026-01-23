#![cfg(loom)]

use fp_library::types::lazy::*;
use loom::{
	sync::Arc,
	sync::atomic::{AtomicUsize, Ordering},
	thread,
};

// =============================================================================
// Core Force/Memoization Tests
// =============================================================================

/// Tests that `ArcLazy` correctly handles concurrent forcing from two threads,
/// ensuring the thunk is executed exactly once and both threads get the result.
#[test]
fn arc_lazy_concurrent_force() {
	loom::model(|| {
		// Create a lazy value that tracks execution count
		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();

		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(move |_| {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			42
		}));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&lazy2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		// Both should succeed with the same value
		assert_eq!(r1.unwrap(), 42);
		assert_eq!(r2.unwrap(), 42);

		// Thunk should have been called exactly once
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	});
}

/// Tests direct Lazy::force (returns reference) under concurrent access
/// Tests that `ArcLazy` correctly handles concurrent calls to `Lazy::force` (returning a reference), ensuring safety and correctness.
#[test]
fn arc_lazy_concurrent_force_ref() {
	loom::model(|| {
		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();

		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(move |_| {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			"hello"
		}));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || {
			let result = Lazy::force(&lazy1);
			result.map(|s| *s) // Copy the &str
		});
		let t2 = thread::spawn(move || {
			let result = Lazy::force(&lazy2);
			result.map(|s| *s)
		});

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		assert_eq!(r1.unwrap(), "hello");
		assert_eq!(r2.unwrap(), "hello");
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	});
}

// =============================================================================
// Panic Propagation Tests
// =============================================================================

/// Tests that if the thunk panics, the panic is correctly propagated to all threads waiting on the value, and they all receive the error.
#[test]
fn arc_lazy_panic_propagation() {
	loom::model(|| {
		let lazy =
			ArcLazy::new(ArcLazyConfig::new_thunk(|_| -> i32 { panic!("intentional test panic") }));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&lazy2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		// BOTH threads should see Err(LazyError), not Ok
		assert!(r1.is_err());
		assert!(r2.is_err());

		// Both should see the same panic message
		assert_eq!(r1.unwrap_err().panic_message(), Some("intentional test panic"));
		assert_eq!(r2.unwrap_err().panic_message(), Some("intentional test panic"));
	});
}

/// Tests panic with String payload (not &str)
/// Tests panic propagation when the panic payload is a `String` (heap allocated) rather than a static `&str`.
#[test]
fn arc_lazy_panic_string_payload() {
	loom::model(|| {
		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(|_| -> i32 {
			panic!("{}", "string panic message".to_string())
		}));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&lazy2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		assert!(r1.is_err());
		assert!(r2.is_err());

		// The panic message should be captured
		assert!(r1.unwrap_err().panic_message().is_some());
		assert!(r2.unwrap_err().panic_message().is_some());
	});
}

// =============================================================================
// Concurrent State Inspection Tests
// =============================================================================

/// Tests is_poisoned and get_error during concurrent forcing
/// Tests `is_poisoned` and `get_error` behavior when accessed concurrently with a thread that causes a panic (poisoning the lazy value).
#[test]
fn arc_lazy_concurrent_is_poisoned() {
	loom::model(|| {
		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(|_| -> i32 { panic!("poisoned") }));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();
		let lazy3 = lazy.clone();

		// Thread 1 forces the value
		let t1 = thread::spawn(move || {
			let _ = Lazy::force(&lazy1);
		});

		// Thread 2 checks is_poisoned
		let t2 = thread::spawn(move || Lazy::is_poisoned(&lazy2));

		// Thread 3 checks get_error
		let t3 = thread::spawn(move || Lazy::get_error(&lazy3).is_some());

		t1.join().unwrap();
		let poisoned = t2.join().unwrap();
		let has_error = t3.join().unwrap();

		// After force, the lazy should be poisoned
		assert!(Lazy::is_poisoned(&lazy));

		// Depending on timing, threads may or may not see poisoned state
		// But after joining, poison state should be consistent
		if poisoned {
			assert!(has_error);
		}
	});
}

/// Tests get_error with successful lazy value
/// Tests `get_error` behavior when accessed concurrently with a thread that successfully forces the value (should return `None`).
#[test]
fn arc_lazy_concurrent_get_error_success() {
	loom::model(|| {
		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(|_| 42));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));
		let t2 = thread::spawn(move || Lazy::get_error(&lazy2));

		let r1 = t1.join().unwrap();
		let _error = t2.join().unwrap();

		assert_eq!(r1.unwrap(), 42);
		// After successful force, get_error should return None
		assert!(Lazy::get_error(&lazy).is_none());
		assert!(!Lazy::is_poisoned(&lazy));
	});
}

// =============================================================================
// Clone During Force Tests
// =============================================================================

/// Tests cloning a Lazy value while another thread is forcing it
/// Tests that cloning an `ArcLazy` while another thread is forcing it works correctly and the clone shares the state (thunk executed once).
#[test]
fn arc_lazy_clone_during_force() {
	loom::model(|| {
		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();

		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(move |_| {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			100
		}));

		let lazy1 = lazy.clone();
		let lazy_for_clone = lazy.clone();

		// Thread 1 forces
		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));

		// Thread 2 clones and then forces the clone
		let t2 = thread::spawn(move || {
			let cloned = lazy_for_clone.clone();
			Lazy::force_cloned(&cloned)
		});

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		assert_eq!(r1.unwrap(), 100);
		assert_eq!(r2.unwrap(), 100);
		// Thunk should only be called once
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	});
}

// =============================================================================
// Semigroup Tests
// =============================================================================

/// Tests concurrent Semigroup::append for ArcLazy
/// Tests concurrent forcing of a value created via `Semigroup::append` (combining two lazy values).
#[test]
fn arc_lazy_concurrent_semigroup_append() {
	use fp_library::classes::semigroup::Semigroup;

	loom::model(|| {
		let x = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
		let y = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "World!".to_string()));

		let z = Semigroup::append(x, y);
		let z1 = z.clone();
		let z2 = z.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&z1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&z2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		assert_eq!(r1.unwrap(), "Hello, World!");
		assert_eq!(r2.unwrap(), "Hello, World!");
	});
}

/// Tests Semigroup::append when first lazy panics
/// Tests `Semigroup::append` behavior when the *first* lazy value in the chain panics; ensures error propagation.
#[test]
fn arc_lazy_semigroup_append_first_panics() {
	use fp_library::classes::semigroup::Semigroup;

	loom::model(|| {
		let x = ArcLazy::new(ArcLazyConfig::new_thunk(|_| -> String { panic!("first panics") }));
		let y = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "World!".to_string()));

		let z = Semigroup::append(x, y);
		let z1 = z.clone();
		let z2 = z.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&z1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&z2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		// Both should see an error (from resume_unwind in append)
		assert!(r1.is_err());
		assert!(r2.is_err());
	});
}

/// Tests Semigroup::append when second lazy panics
/// Tests `Semigroup::append` behavior when the *second* lazy value in the chain panics; ensures error propagation.
#[test]
fn arc_lazy_semigroup_append_second_panics() {
	use fp_library::classes::semigroup::Semigroup;

	loom::model(|| {
		let x = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "Hello, ".to_string()));
		let y = ArcLazy::new(ArcLazyConfig::new_thunk(|_| -> String { panic!("second panics") }));

		let z = Semigroup::append(x, y);
		let z1 = z.clone();
		let z2 = z.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&z1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&z2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		// Both should see an error
		assert!(r1.is_err());
		assert!(r2.is_err());
	});
}

// =============================================================================
// Monoid Tests
// =============================================================================

/// Tests concurrent Monoid::empty for ArcLazy
/// Tests concurrent forcing of `Monoid::empty()` for `ArcLazy`.
#[test]
fn arc_lazy_concurrent_monoid_empty() {
	use fp_library::classes::monoid::Monoid;

	loom::model(|| {
		let empty1: ArcLazy<String> = Monoid::empty();
		let empty2: ArcLazy<String> = Monoid::empty();

		let t1 = thread::spawn(move || Lazy::force_cloned(&empty1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&empty2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		assert_eq!(r1.unwrap(), "");
		assert_eq!(r2.unwrap(), "");
	});
}

/// Tests Monoid identity with Semigroup::append under concurrency
/// Tests concurrent forcing of a value created via `Semigroup::append` with `Monoid::empty()` (identity property).
#[test]
fn arc_lazy_monoid_identity_concurrent() {
	use fp_library::classes::{monoid::Monoid, semigroup::Semigroup};

	loom::model(|| {
		let empty: ArcLazy<String> = Monoid::empty();
		let value = ArcLazy::new(ArcLazyConfig::new_thunk(|_| "test".to_string()));

		let result = Semigroup::append(empty, value);
		let result1 = result.clone();
		let result2 = result.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&result1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&result2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		assert_eq!(r1.unwrap(), "test");
		assert_eq!(r2.unwrap(), "test");
	});
}

// =============================================================================
// SendDefer Tests
// =============================================================================

/// Tests SendDefer::send_defer for ArcLazy
/// Tests `SendDefer::send_defer` with `ArcLazy` under concurrent access, ensuring the outer thunk is executed once.
#[test]
fn arc_lazy_send_defer() {
	use fp_library::brands::LazyBrand;
	use fp_library::classes::send_defer::SendDefer;

	loom::model(|| {
		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();

		let lazy = <LazyBrand<ArcLazyConfig> as SendDefer>::send_defer(move || {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			ArcLazy::new(ArcLazyConfig::new_thunk(|_| 42))
		});

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&lazy2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		assert_eq!(r1.unwrap(), 42);
		assert_eq!(r2.unwrap(), 42);
		// The outer thunk should only be called once
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	});
}

/// Tests SendDefer::send_defer when inner lazy panics
/// Tests `SendDefer::send_defer` when the inner lazy value panics, ensuring error propagation.
#[test]
fn arc_lazy_send_defer_inner_panics() {
	use fp_library::brands::LazyBrand;
	use fp_library::classes::send_defer::SendDefer;

	loom::model(|| {
		let lazy = <LazyBrand<ArcLazyConfig> as SendDefer>::send_defer(|| {
			ArcLazy::new(ArcLazyConfig::new_thunk(|_| -> i32 {
				panic!("inner panic in send_defer")
			}))
		});

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&lazy2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		// Both should see errors
		assert!(r1.is_err());
		assert!(r2.is_err());
	});
}

// =============================================================================
// force_or_panic / force_ref_or_panic Tests
// =============================================================================

/// Tests concurrent force_or_panic with successful evaluation
/// Tests `Lazy::force_or_panic` under concurrent access when the computation succeeds.
#[test]
fn arc_lazy_concurrent_force_or_panic_success() {
	loom::model(|| {
		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();

		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(move |_| {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			99
		}));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_or_panic(&lazy1));
		let t2 = thread::spawn(move || Lazy::force_or_panic(&lazy2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		assert_eq!(r1, 99);
		assert_eq!(r2, 99);
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	});
}

/// Note: We can't easily test force_or_panic with panics in loom because
/// the panic would cause the test to fail. The panic path is tested via
/// force() returning Err.

// =============================================================================
// Three-Thread Tests
// =============================================================================

/// Tests three concurrent threads forcing the same lazy value
/// Tests `ArcLazy` with three concurrent threads forcing the value, ensuring synchronization works for >2 threads.
#[test]
fn arc_lazy_three_threads_force() {
	loom::model(|| {
		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();

		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(move |_| {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			"three"
		}));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();
		let lazy3 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&lazy2));
		let t3 = thread::spawn(move || Lazy::force_cloned(&lazy3));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();
		let r3 = t3.join().unwrap();

		assert_eq!(r1.unwrap(), "three");
		assert_eq!(r2.unwrap(), "three");
		assert_eq!(r3.unwrap(), "three");
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	});
}

/// Tests three threads with panic propagation
/// Tests panic propagation with three concurrent threads.
#[test]
fn arc_lazy_three_threads_panic() {
	loom::model(|| {
		let lazy =
			ArcLazy::new(ArcLazyConfig::new_thunk(|_| -> i32 { panic!("three thread panic") }));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();
		let lazy3 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));
		let t2 = thread::spawn(move || Lazy::force_cloned(&lazy2));
		let t3 = thread::spawn(move || Lazy::force_cloned(&lazy3));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();
		let r3 = t3.join().unwrap();

		assert!(r1.is_err());
		assert!(r2.is_err());
		assert!(r3.is_err());

		assert_eq!(r1.unwrap_err().panic_message(), Some("three thread panic"));
		assert_eq!(r2.unwrap_err().panic_message(), Some("three thread panic"));
		assert_eq!(r3.unwrap_err().panic_message(), Some("three thread panic"));
	});
}

// =============================================================================
// Debug Format Tests
// =============================================================================

/// Tests Debug formatting while another thread is forcing
/// Tests that `Debug` formatting is safe to call while another thread is forcing the value.
#[test]
fn arc_lazy_debug_during_force() {
	loom::model(|| {
		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(|_| 42i32));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || Lazy::force_cloned(&lazy1));
		let t2 = thread::spawn(move || format!("{:?}", lazy2));

		let r1 = t1.join().unwrap();
		let debug_output = t2.join().unwrap();

		assert_eq!(r1.unwrap(), 42);
		// Debug output should be valid (either showing value or not)
		assert!(debug_output.contains("Lazy"));
	});
}

// =============================================================================
// LazyError Tests
// =============================================================================

/// Tests LazyError::from_panic with &str payload
/// Tests `LazyError` creation from a panic with a `&str` payload.
#[test]
fn lazy_error_from_panic_str() {
	loom::model(|| {
		let payload: Box<dyn std::any::Any + Send + 'static> = Box::new("str payload");
		let error = LazyError::from_panic(payload);
		assert_eq!(error.panic_message(), Some("str payload"));
	});
}

/// Tests LazyError::from_panic with String payload
/// Tests `LazyError` creation from a panic with a `String` payload.
#[test]
fn lazy_error_from_panic_string() {
	loom::model(|| {
		let payload: Box<dyn std::any::Any + Send + 'static> =
			Box::new("String payload".to_string());
		let error = LazyError::from_panic(payload);
		assert_eq!(error.panic_message(), Some("String payload"));
	});
}

/// Tests LazyError::from_panic with unknown payload type
/// Tests `LazyError` creation from a panic with an unknown payload type.
#[test]
fn lazy_error_from_panic_unknown() {
	loom::model(|| {
		let payload: Box<dyn std::any::Any + Send + 'static> = Box::new(42i32);
		let error = LazyError::from_panic(payload);
		assert_eq!(error.panic_message(), None);
	});
}

/// Tests LazyError Display implementation
/// Tests the `Display` implementation of `LazyError`.
#[test]
fn lazy_error_display() {
	loom::model(|| {
		let payload: Box<dyn std::any::Any + Send + 'static> = Box::new("test message");
		let error = LazyError::from_panic(payload);
		let display = format!("{}", error);
		assert!(display.contains("thunk panicked during evaluation"));
		assert!(display.contains("test message"));
	});
}

/// Tests LazyError Display with no message
/// Tests the `Display` implementation of `LazyError` when no message is available.
#[test]
fn lazy_error_display_no_message() {
	loom::model(|| {
		let payload: Box<dyn std::any::Any + Send + 'static> = Box::new(123u64);
		let error = LazyError::from_panic(payload);
		let display = format!("{}", error);
		assert_eq!(display, "thunk panicked during evaluation");
	});
}
