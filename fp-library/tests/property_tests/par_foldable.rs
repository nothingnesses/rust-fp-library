use super::common::Sum;
use fp_library::brands::{ArcFnBrand, VecBrand};
use fp_library::classes::{Foldable, Monoid, ParFoldable, SendCloneableFn};
use quickcheck_macros::quickcheck;

/// Verifies that `par_fold_map` correctly sums a large vector (100,000 elements)
/// without overflow or errors, ensuring basic correctness for large datasets.
#[test]
fn test_large_vector_par_fold_map() {
	let xs: Vec<i32> = (0..100000).collect();
	let f_par = <ArcFnBrand as SendCloneableFn>::send_cloneable_fn_new(|x: i32| Sum(x as i64));
	let res = VecBrand::par_fold_map::<ArcFnBrand, _, _>(f_par, xs);
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
	let par_res = VecBrand::par_fold_map::<ArcFnBrand, _, _>(f_par, xs);

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
	let par_res = VecBrand::par_fold_right::<ArcFnBrand, _, _>(f_par, init, xs.clone());

	if seq_res != par_res {
		println!("Fold right mismatch: seq={seq_res}, par={par_res}, xs={xs:?}");
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
	let par_res = VecBrand::par_fold_map::<ArcFnBrand, _, _>(f_par, xs);

	par_res == Sum::empty()
}

/// Property test asserting that `par_fold_map` is deterministic (returns the same result
/// when called twice on the same input).
#[quickcheck]
fn prop_par_fold_map_deterministic(xs: Vec<i32>) -> bool {
	let f_par: <ArcFnBrand as SendCloneableFn>::SendOf<'_, i32, Sum> =
		<ArcFnBrand as SendCloneableFn>::send_cloneable_fn_new(|x: i32| Sum(x as i64));

	let res1 = VecBrand::par_fold_map::<ArcFnBrand, _, _>(f_par.clone(), xs.clone());
	let res2 = VecBrand::par_fold_map::<ArcFnBrand, _, _>(f_par, xs);
	if res1 != res2 {
		println!("Deterministic fail: {:?} != {:?}", res1, res2);
		return false;
	}
	true
}
