#[cfg(test)]
mod tests {
	use fp_library::brands::{ArcFnBrand, VecBrand};
	use fp_library::classes::foldable::Foldable;
	use fp_library::classes::monoid::Monoid;
	use fp_library::classes::par_foldable::ParFoldable;
	use fp_library::classes::semigroup::Semigroup;
	use fp_library::classes::send_clonable_fn::SendClonableFn;
	use quickcheck_macros::quickcheck;

	// Monoid for testing (Sum of i64 to avoid overflow)
	#[derive(Clone, Debug, PartialEq, Eq)]
	struct Sum(i64);

	impl Semigroup for Sum {
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			Sum(a.0 + b.0)
		}
	}

	impl Monoid for Sum {
		fn empty() -> Self {
			Sum(0)
		}
	}

	// i64 is Send + Sync, so Sum is Send + Sync automatically.

	#[test]
	fn test_large_vector_par_fold_map() {
		let xs: Vec<i32> = (0..100000).collect();
		let f_par = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| Sum(x as i64));
		let res = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par, xs);
		assert_eq!(res, Sum(4999950000));
	}

	#[quickcheck]
	fn prop_par_fold_map_equals_fold_map(xs: Vec<i32>) -> bool {
		let f_seq = |x: i32| Sum(x as i64);
		let f_par = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| Sum(x as i64));

		// Foldable::fold_map takes (f, fa)
		let seq_res = VecBrand::fold_map::<ArcFnBrand, _, _, _>(f_seq, xs.clone());
		let par_res = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par, xs);

		seq_res == par_res
	}

	#[quickcheck]
	fn prop_par_fold_right_equals_fold_right(xs: Vec<i32>) -> bool {
		// Foldable::fold_right takes Fn(A, B) -> B (two args)
		// Use wrapping_add to avoid overflow panics in debug mode
		let f_seq = |a: i32, b: i32| a.wrapping_add(b);
		// ParFoldable::par_fold_right takes Fn((A, B)) -> B (tuple arg)
		let f_par =
			<ArcFnBrand as SendClonableFn>::new_send(|(a, b): (i32, i32)| a.wrapping_add(b));
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

	#[quickcheck]
	fn prop_par_fold_map_empty_is_empty(xs: Vec<i32>) -> bool {
		if !xs.is_empty() {
			return true;
		}

		let f_par = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| Sum(x as i64));
		let par_res = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par, xs);

		par_res == Sum::empty()
	}

	#[quickcheck]
	fn prop_par_fold_map_deterministic(xs: Vec<i32>) -> bool {
		let f_par: <ArcFnBrand as SendClonableFn>::SendOf<'_, i32, Sum> =
			<ArcFnBrand as SendClonableFn>::new_send(|x: i32| Sum(x as i64));

		let res1 = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par.clone(), xs.clone());
		let res2 = <VecBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f_par, xs);

		if res1 != res2 {
			println!("Deterministic fail: {:?} != {:?}", res1, res2);
			return false;
		}
		true
	}
}
