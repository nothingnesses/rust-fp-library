# Step 5: Concurrency Testing with Loom

This step focuses on verifying the thread safety of the `ArcLazy` implementation using the `loom` crate for deterministic concurrency testing.

## Goals

1.  Add `loom` as a dev dependency in `fp-library/Cargo.toml`.
2.  Create `fp-library/tests/loom_tests.rs` with concurrent lazy tests.
3.  Run loom tests to verify synchronization correctness.

## Technical Design

### Loom Integration

Loom exhaustively tests all possible thread interleavings, finding race conditions that random testing might miss. This is critical for verifying that:

1.  `OnceLock::get_or_init` correctly synchronizes access.
2.  `Mutex::lock` on the thunk cell doesn't cause deadlocks.
3.  Panic propagation works correctly across threads.
4.  The memoized value is visible to all threads after forcing.

### Test Cases

#### Concurrent Force

Verify that multiple threads forcing the same `ArcLazy` value result in a single execution of the thunk and all threads receiving the correct value.

```rust
#[test]
fn arc_lazy_concurrent_force() {
	loom::model(|| {
		// Create a lazy value that tracks execution count
		let counter = Arc::new(loom::sync::atomic::AtomicUsize::new(0));
		let counter_clone = counter.clone();

		let lazy = Arc::new(lazy_new::<ArcLazyConfig, _>(
			send_clonable_fn_new::<ArcFnBrand, _, _>(move |_| {
				counter_clone.fetch_add(1, loom::sync::atomic::Ordering::SeqCst);
			})
		));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || lazy_force_cloned::<ArcLazyConfig, _>(&*lazy1));
		let t2 = thread::spawn(move || lazy_force_cloned::<ArcLazyConfig, _>(&*lazy2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		// Both should succeed with the same value
		assert_eq!(r1, Ok(42));
		assert_eq!(r2, Ok(42));

		// Thunk should have been called exactly once
		assert_eq!(counter.load(loom::sync::atomic::Ordering::SeqCst), 1);
	});
}
```

#### Panic Propagation

Verify that if the thunk panics, all threads forcing the `ArcLazy` value receive the error, and the panic message is preserved.

```rust
fn arc_lazy_panic_propagation() {
	loom::model(|| {
		let lazy = Arc::new(lazy_new::<ArcLazyConfig, _>(
			send_clonable_fn_new::<ArcFnBrand, _, _>(|_| -> i32 {
				panic!("intentional test panic")
			})
		));

		let lazy1 = lazy.clone();
		let lazy2 = lazy.clone();

		let t1 = thread::spawn(move || lazy_force::<ArcLazyConfig, _>(&*lazy1));
		let t2 = thread::spawn(move || lazy_force::<ArcLazyConfig, _>(&*lazy2));

		let r1 = t1.join().unwrap();
		let r2 = t2.join().unwrap();

		// BOTH threads should see Err(LazyError), not Ok
		assert!(r1.is_err());
		assert!(r2.is_err());

		// Both should see the same panic message
		assert_eq!(
			r1.unwrap_err().panic_message(),
			Some("intentional test panic")
		);
		assert_eq!(
			r2.unwrap_err().panic_message(),
			Some("intentional test panic")
		);
	});
}
```

## Checklist

- [x] Add `loom` as a dev dependency in `fp-library/Cargo.toml`
- [x] Create `fp-library/tests/loom_tests.rs`
- [x] Implement `arc_lazy_concurrent_force` test
- [x] Implement `arc_lazy_panic_propagation` test
- [x] Run loom tests with `RUSTFLAGS="--cfg loom" cargo test --test loom_tests`
