#[cfg(test)]
mod tests {
	use fp_library::brands::{ArcFnBrand, LazyBrand, VecBrand};
	use fp_library::classes::foldable::Foldable;
	use fp_library::classes::monoid::Monoid;
	use fp_library::classes::par_foldable::ParFoldable;
	use fp_library::classes::semigroup::Semigroup;
	use fp_library::classes::send_cloneable_fn::SendCloneableFn;
	use fp_library::classes::send_defer::SendDefer;
	use fp_library::types::lazy::{ArcLazy, ArcLazyConfig, Lazy, RcLazy, RcLazyConfig};
	use quickcheck_macros::quickcheck;

	// Monoid for testing (Sum of i64 using wrapping_add to avoid overflow)
	#[derive(Clone, Debug, PartialEq, Eq)]
	struct Sum(i64);

	impl Semigroup for Sum {
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			Sum(a.0.wrapping_add(b.0))
		}
	}

	impl Monoid for Sum {
		fn empty() -> Self {
			Sum(0)
		}
	}

	// i64 is Send + Sync, so Sum is Send + Sync automatically.

	/// Verifies that `par_fold_map` correctly sums a large vector (100,000 elements)
	/// without overflow or errors, ensuring basic correctness for large datasets.
	#[test]
	fn test_large_vector_par_fold_map() {
		let xs: Vec<i32> = (0..100000).collect();
		let f_par = <ArcFnBrand as SendCloneableFn>::send_cloneable_fn_new(|x: i32| Sum(x as i64));
		let res = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par, xs);
		assert_eq!(res, Sum(4999950000));
	}

	/// Property test asserting that parallel `par_fold_map` produces the exact same result
	/// as sequential `fold_map` for any `Vec<i32>`.
	#[quickcheck]
	fn prop_par_fold_map_equals_fold_map(xs: Vec<i32>) -> bool {
		let f_seq = |x: i32| Sum(x as i64);
		let f_par = <ArcFnBrand as SendCloneableFn>::send_cloneable_fn_new(|x: i32| Sum(x as i64));

		// Foldable::fold_map takes (f, fa)
		let seq_res = VecBrand::fold_map::<ArcFnBrand, _, _, _>(f_seq, xs.clone());
		let par_res = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par, xs);

		seq_res == par_res
	}

	/// Property test asserting that parallel `par_fold_right` produces the same result
	/// as sequential `fold_right` for any `Vec<i32>`.
	#[quickcheck]
	fn prop_par_fold_right_equals_fold_right(xs: Vec<i32>) -> bool {
		// Foldable::fold_right takes Fn(A, B) -> B (two args)
		// Use wrapping_add to avoid overflow panics in debug mode
		let f_seq = |a: i32, b: i32| a.wrapping_add(b);
		// ParFoldable::par_fold_right takes Fn((A, B)) -> B (tuple arg)
		let f_par = <ArcFnBrand as SendCloneableFn>::send_cloneable_fn_new(|(a, b): (i32, i32)| {
			a.wrapping_add(b)
		});
		let init = 0;

		let seq_res = VecBrand::fold_right::<ArcFnBrand, _, _, _>(f_seq, init, xs.clone());
		let par_res =
			<VecBrand as ParFoldable<ArcFnBrand>>::par_fold_right(f_par, init, xs.clone());

		if seq_res != par_res {
			println!("Fold right mismatch: seq={}, par={}, xs={:?}", seq_res, par_res, xs);
			return false;
		}
		true
	}

	/// Property test asserting that `par_fold_map` on an empty vector returns the Monoid's empty value.
	#[quickcheck]
	fn prop_par_fold_map_empty_is_empty(xs: Vec<i32>) -> bool {
		if !xs.is_empty() {
			return true;
		}

		let f_par = <ArcFnBrand as SendCloneableFn>::send_cloneable_fn_new(|x: i32| Sum(x as i64));
		let par_res = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par, xs);

		par_res == Sum::empty()
	}

	/// Property test asserting that `par_fold_map` is deterministic (returns the same result
	/// when called twice on the same input).
	#[quickcheck]
	fn prop_par_fold_map_deterministic(xs: Vec<i32>) -> bool {
		let f_par: <ArcFnBrand as SendCloneableFn>::SendOf<'_, i32, Sum> =
			<ArcFnBrand as SendCloneableFn>::send_cloneable_fn_new(|x: i32| Sum(x as i64));

		let res1 = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par.clone(), xs.clone());
		let res2 = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par, xs);
		if res1 != res2 {
			println!("Deterministic fail: {:?} != {:?}", res1, res2);
			return false;
		}
		true
	}

	// =========================================================================
	// Lazy Property Tests
	// =========================================================================

	// -------------------------------------------------------------------------
	// Memoization Properties
	// -------------------------------------------------------------------------

	/// Property: Forcing a lazy value twice returns the same result
	/// Verifies that `RcLazy` memoizes its result; forcing it twice returns the same value without re-executing the thunk.
	#[quickcheck]
	fn prop_rc_lazy_force_memoization(x: i32) -> bool {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(move |_| x * 2));
		let result1 = Lazy::force(&lazy).cloned();
		let result2 = Lazy::force(&lazy).cloned();
		result1 == result2
	}

	/// Property: Forcing a lazy value twice returns the same result (Arc version)
	/// Verifies that `ArcLazy` memoizes its result; forcing it twice returns the same value.
	#[quickcheck]
	fn prop_arc_lazy_force_memoization(x: i32) -> bool {
		let lazy = ArcLazy::new(ArcLazyConfig::new_thunk(move |_| x * 2));
		let result1 = Lazy::force(&lazy).cloned();
		let result2 = Lazy::force(&lazy).cloned();
		result1 == result2
	}

	/// Property: force_cloned returns the same value as force
	/// Verifies that `Lazy::force_cloned` is equivalent to `Lazy::force` followed by cloning the result.
	#[quickcheck]
	fn prop_lazy_force_cloned_equals_force(x: i32) -> bool {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(move |_| x));
		let force_result = Lazy::force(&lazy).cloned();
		let force_cloned_result = Lazy::force_cloned(&lazy);
		force_result == force_cloned_result
	}

	// -------------------------------------------------------------------------
	// Clone Equivalence Properties
	// -------------------------------------------------------------------------

	/// Property: Cloning a lazy value shares state - forcing clone gives same result
	/// Verifies that cloning an `RcLazy` shares the underlying state; forcing the clone yields the same result as the original.
	#[quickcheck]
	fn prop_rc_lazy_clone_shares_state(x: i32) -> bool {
		let lazy1 = RcLazy::new(RcLazyConfig::new_thunk(move |_| x));
		let lazy2 = lazy1.clone();

		let result1 = Lazy::force(&lazy1).cloned();
		let result2 = Lazy::force(&lazy2).cloned();
		result1 == result2
	}

	/// Property: Cloning an ArcLazy shares state
	/// Verifies that cloning an `ArcLazy` shares the underlying state.
	#[quickcheck]
	fn prop_arc_lazy_clone_shares_state(x: i32) -> bool {
		let lazy1 = ArcLazy::new(ArcLazyConfig::new_thunk(move |_| x));
		let lazy2 = lazy1.clone();

		let result1 = Lazy::force(&lazy1).cloned();
		let result2 = Lazy::force(&lazy2).cloned();
		result1 == result2
	}

	/// Property: Forcing original then clone gives same result
	/// Verifies consistency when forcing the original lazy value first, then the clone.
	#[quickcheck]
	fn prop_lazy_force_original_then_clone(x: String) -> bool {
		let value = x.clone();
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(move |_| value.clone()));
		let lazy_clone = lazy.clone();

		// Force original first
		let result1 = Lazy::force(&lazy).cloned();
		// Then force clone
		let result2 = Lazy::force(&lazy_clone).cloned();

		result1 == result2
	}

	// -------------------------------------------------------------------------
	// Semigroup Associativity Property
	// -------------------------------------------------------------------------

	/// Property: Semigroup::append is associative for RcLazy<String>
	/// Verifies the associativity law `(a <> b) <> c == a <> (b <> c)` for `RcLazy<String>`.
	#[quickcheck]
	fn prop_rc_lazy_semigroup_associativity(
		a: String,
		b: String,
		c: String,
	) -> bool {
		let lazy_a1 = RcLazy::new(RcLazyConfig::new_thunk({
			let a = a.clone();
			move |_| a.clone()
		}));
		let lazy_b1 = RcLazy::new(RcLazyConfig::new_thunk({
			let b = b.clone();
			move |_| b.clone()
		}));
		let lazy_c1 = RcLazy::new(RcLazyConfig::new_thunk({
			let c = c.clone();
			move |_| c.clone()
		}));

		let lazy_a2 = RcLazy::new(RcLazyConfig::new_thunk({
			let a = a.clone();
			move |_| a.clone()
		}));
		let lazy_b2 = RcLazy::new(RcLazyConfig::new_thunk({
			let b = b.clone();
			move |_| b.clone()
		}));
		let lazy_c2 = RcLazy::new(RcLazyConfig::new_thunk({
			let c = c.clone();
			move |_| c.clone()
		}));

		// (a <> b) <> c
		let left = Semigroup::append(Semigroup::append(lazy_a1, lazy_b1), lazy_c1);
		// a <> (b <> c)
		let right = Semigroup::append(lazy_a2, Semigroup::append(lazy_b2, lazy_c2));

		Lazy::force(&left).cloned() == Lazy::force(&right).cloned()
	}

	/// Property: Semigroup::append is associative for ArcLazy<String>
	/// Verifies the associativity law for `ArcLazy<String>`.
	#[quickcheck]
	fn prop_arc_lazy_semigroup_associativity(
		a: String,
		b: String,
		c: String,
	) -> bool {
		let lazy_a1 = ArcLazy::new(ArcLazyConfig::new_thunk({
			let a = a.clone();
			move |_| a.clone()
		}));
		let lazy_b1 = ArcLazy::new(ArcLazyConfig::new_thunk({
			let b = b.clone();
			move |_| b.clone()
		}));
		let lazy_c1 = ArcLazy::new(ArcLazyConfig::new_thunk({
			let c = c.clone();
			move |_| c.clone()
		}));

		let lazy_a2 = ArcLazy::new(ArcLazyConfig::new_thunk({
			let a = a.clone();
			move |_| a.clone()
		}));
		let lazy_b2 = ArcLazy::new(ArcLazyConfig::new_thunk({
			let b = b.clone();
			move |_| b.clone()
		}));
		let lazy_c2 = ArcLazy::new(ArcLazyConfig::new_thunk({
			let c = c.clone();
			move |_| c.clone()
		}));

		// (a <> b) <> c
		let left = Semigroup::append(Semigroup::append(lazy_a1, lazy_b1), lazy_c1);
		// a <> (b <> c)
		let right = Semigroup::append(lazy_a2, Semigroup::append(lazy_b2, lazy_c2));

		Lazy::force(&left).cloned() == Lazy::force(&right).cloned()
	}

	// -------------------------------------------------------------------------
	// Monoid Identity Properties
	// -------------------------------------------------------------------------

	/// Property: Monoid left identity - append(empty, a) == a
	/// Verifies the left identity law `empty <> a == a` for `RcLazy<String>`.
	#[quickcheck]
	fn prop_rc_lazy_monoid_left_identity(s: String) -> bool {
		let lazy_a = RcLazy::new(RcLazyConfig::new_thunk({
			let s = s.clone();
			move |_| s.clone()
		}));
		let empty: RcLazy<String> = Monoid::empty();

		let result = Semigroup::append(empty, lazy_a);
		Lazy::force(&result).cloned() == Ok(s)
	}

	/// Property: Monoid right identity - append(a, empty) == a
	/// Verifies the right identity law `a <> empty == a` for `RcLazy<String>`.
	#[quickcheck]
	fn prop_rc_lazy_monoid_right_identity(s: String) -> bool {
		let lazy_a = RcLazy::new(RcLazyConfig::new_thunk({
			let s = s.clone();
			move |_| s.clone()
		}));
		let empty: RcLazy<String> = Monoid::empty();

		let result = Semigroup::append(lazy_a, empty);
		Lazy::force(&result).cloned() == Ok(s)
	}

	/// Property: Monoid left identity for ArcLazy
	/// Verifies the left identity law for `ArcLazy<String>`.
	#[quickcheck]
	fn prop_arc_lazy_monoid_left_identity(s: String) -> bool {
		let lazy_a = ArcLazy::new(ArcLazyConfig::new_thunk({
			let s = s.clone();
			move |_| s.clone()
		}));
		let empty: ArcLazy<String> = Monoid::empty();

		let result = Semigroup::append(empty, lazy_a);
		Lazy::force(&result).cloned() == Ok(s)
	}

	/// Property: Monoid right identity for ArcLazy
	/// Verifies the right identity law for `ArcLazy<String>`.
	#[quickcheck]
	fn prop_arc_lazy_monoid_right_identity(s: String) -> bool {
		let lazy_a = ArcLazy::new(ArcLazyConfig::new_thunk({
			let s = s.clone();
			move |_| s.clone()
		}));
		let empty: ArcLazy<String> = Monoid::empty();

		let result = Semigroup::append(lazy_a, empty);
		Lazy::force(&result).cloned() == Ok(s)
	}

	/// Property: empty() is idempotent - multiple calls return equivalent values
	/// Verifies that `Monoid::empty()` is idempotent (multiple calls produce equivalent empty values).
	#[quickcheck]
	fn prop_lazy_empty_idempotent(_: ()) -> bool {
		let empty1: RcLazy<String> = Monoid::empty();
		let empty2: RcLazy<String> = Monoid::empty();

		Lazy::force(&empty1).cloned() == Lazy::force(&empty2).cloned()
	}

	// -------------------------------------------------------------------------
	// SendDefer Properties
	// -------------------------------------------------------------------------

	/// Property: SendDefer produces equivalent results to direct construction
	/// Verifies that `SendDefer::send_defer` produces a lazy value equivalent to one created directly.
	#[quickcheck]
	fn prop_send_defer_equivalent_to_direct(x: i32) -> bool {
		let direct = ArcLazy::new(ArcLazyConfig::new_thunk(move |_| x.wrapping_mul(3)));
		let deferred = <LazyBrand<ArcLazyConfig> as SendDefer>::send_defer(move || {
			ArcLazy::new(ArcLazyConfig::new_thunk(move |_| x.wrapping_mul(3)))
		});

		Lazy::force(&direct).cloned() == Lazy::force(&deferred).cloned()
	}

	/// Property: SendDefer memoizes the outer thunk
	/// Verifies that `SendDefer` memoizes the outer thunk (the one creating the inner lazy value).
	#[quickcheck]
	fn prop_send_defer_memoization(x: i32) -> bool {
		use std::sync::Arc;
		use std::sync::atomic::{AtomicUsize, Ordering};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = counter.clone();

		let deferred = <LazyBrand<ArcLazyConfig> as SendDefer>::send_defer(move || {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			ArcLazy::new(ArcLazyConfig::new_thunk(move |_| x))
		});

		let _ = Lazy::force(&deferred);
		let _ = Lazy::force(&deferred);
		let _ = Lazy::force(&deferred);

		// Outer thunk should only be called once
		counter.load(Ordering::SeqCst) == 1
	}

	// -------------------------------------------------------------------------
	// Error State Properties
	// -------------------------------------------------------------------------

	/// Property: is_poisoned is false before forcing
	/// Verifies a lazy value is not "poisoned" before being forced.
	#[quickcheck]
	fn prop_lazy_not_poisoned_before_force(_: ()) -> bool {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
		!Lazy::is_poisoned(&lazy)
	}

	/// Property: is_poisoned is false after successful force
	/// Verifies a lazy value is not "poisoned" after successful forcing.
	#[quickcheck]
	fn prop_lazy_not_poisoned_after_success(x: i32) -> bool {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(move |_| x));
		let _ = Lazy::force(&lazy);
		!Lazy::is_poisoned(&lazy)
	}

	/// Property: get_error returns None before forcing
	/// Verifies `get_error` returns `None` before forcing.
	#[quickcheck]
	fn prop_lazy_no_error_before_force(_: ()) -> bool {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(|_| 42));
		Lazy::get_error(&lazy).is_none()
	}

	/// Property: get_error returns None after successful force
	/// Verifies `get_error` returns `None` after successful forcing.
	#[quickcheck]
	fn prop_lazy_no_error_after_success(x: i32) -> bool {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(move |_| x));
		let _ = Lazy::force(&lazy);
		Lazy::get_error(&lazy).is_none()
	}

	// -------------------------------------------------------------------------
	// Determinism Properties
	// -------------------------------------------------------------------------

	/// Property: Lazy computation is deterministic
	/// Verifies that two independent lazy values with the same logic produce the same result.
	#[quickcheck]
	fn prop_lazy_deterministic(
		x: i32,
		y: i32,
	) -> bool {
		let lazy1 = RcLazy::new(RcLazyConfig::new_thunk(move |_| x.wrapping_add(y)));
		let lazy2 = RcLazy::new(RcLazyConfig::new_thunk(move |_| x.wrapping_add(y)));

		Lazy::force(&lazy1).cloned() == Lazy::force(&lazy2).cloned()
	}

	/// Property: force_or_panic returns same value as force on success
	/// Verifies `force_or_panic` behaves like `force` on success.
	#[quickcheck]
	fn prop_force_or_panic_equals_force(x: i32) -> bool {
		let lazy1 = RcLazy::new(RcLazyConfig::new_thunk(move |_| x));
		let lazy2 = RcLazy::new(RcLazyConfig::new_thunk(move |_| x));

		let force_result = Lazy::force(&lazy1).cloned().unwrap();
		let panic_result = Lazy::force_or_panic(&lazy2);

		force_result == panic_result
	}

	/// Property: force_ref_or_panic returns same reference as force on success
	/// Verifies `force_ref_or_panic` behaves like `force` (returning a reference) on success.
	#[quickcheck]
	fn prop_force_ref_or_panic_equals_force(x: i32) -> bool {
		let lazy = RcLazy::new(RcLazyConfig::new_thunk(move |_| x));

		let force_ref = Lazy::force(&lazy).unwrap();
		let panic_ref = Lazy::force_ref_or_panic(&lazy);

		std::ptr::eq(force_ref, panic_ref)
	}

	// -------------------------------------------------------------------------
	// Numeric Semigroup/Monoid Tests with Sum
	// -------------------------------------------------------------------------

	/// Property: Lazy<Sum> Semigroup is associative
	/// Verifies associativity for `Lazy<Sum>` (a numeric wrapper).
	#[quickcheck]
	fn prop_lazy_sum_semigroup_associative(
		a: i64,
		b: i64,
		c: i64,
	) -> bool {
		let lazy_a1 = RcLazy::new(RcLazyConfig::new_thunk(move |_| Sum(a)));
		let lazy_b1 = RcLazy::new(RcLazyConfig::new_thunk(move |_| Sum(b)));
		let lazy_c1 = RcLazy::new(RcLazyConfig::new_thunk(move |_| Sum(c)));

		let lazy_a2 = RcLazy::new(RcLazyConfig::new_thunk(move |_| Sum(a)));
		let lazy_b2 = RcLazy::new(RcLazyConfig::new_thunk(move |_| Sum(b)));
		let lazy_c2 = RcLazy::new(RcLazyConfig::new_thunk(move |_| Sum(c)));

		let left = Semigroup::append(Semigroup::append(lazy_a1, lazy_b1), lazy_c1);
		let right = Semigroup::append(lazy_a2, Semigroup::append(lazy_b2, lazy_c2));

		Lazy::force(&left).cloned() == Lazy::force(&right).cloned()
	}

	/// Property: Lazy<Sum> Monoid identity laws
	/// Verifies monoid identity laws for `Lazy<Sum>`.
	#[quickcheck]
	fn prop_lazy_sum_monoid_identity(x: i64) -> bool {
		let lazy_x1 = RcLazy::new(RcLazyConfig::new_thunk(move |_| Sum(x)));
		let lazy_x2 = RcLazy::new(RcLazyConfig::new_thunk(move |_| Sum(x)));
		let empty1: RcLazy<Sum> = Monoid::empty();
		let empty2: RcLazy<Sum> = Monoid::empty();

		let left_identity = Semigroup::append(empty1, lazy_x1);
		let right_identity = Semigroup::append(lazy_x2, empty2);

		let expected = Sum(x);
		Lazy::force(&left_identity).cloned() == Ok(expected.clone())
			&& Lazy::force(&right_identity).cloned() == Ok(expected)
	}
}
