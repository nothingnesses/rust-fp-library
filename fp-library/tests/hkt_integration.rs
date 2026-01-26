use fp_library::{
	brands::{EvalBrand, MemoBrand, RcFnBrand},
	classes::{foldable::Foldable, monad_rec::tail_rec_m, ref_functor::map_ref},
	types::{Eval, Memo, RcMemoConfig, Step},
};

#[test]
fn test_eval_monad_rec() {
	// Factorial using tail_rec_m
	fn factorial(n: u64) -> Eval<'static, u64> {
		tail_rec_m::<EvalBrand, _, _, _>(
			|(n, acc)| {
				if n == 0 {
					Eval::pure(Step::Done(acc))
				} else {
					Eval::pure(Step::Loop((n - 1, n * acc)))
				}
			},
			(n, 1),
		)
	}

	assert_eq!(factorial(5).run(), 120);
}

#[test]
fn test_eval_foldable() {
	// Eval contains a single value, so fold should just apply the function once

	// fold_right: (A, B) -> B
	let res_right = EvalBrand::fold_right::<RcFnBrand, _, _, _>(|a, b| a + b, 5, Eval::pure(10));
	assert_eq!(res_right, 15);

	// fold_left: (B, A) -> B
	let res_left = EvalBrand::fold_left::<RcFnBrand, _, _, _>(|b, a| b + a, 5, Eval::pure(10));
	assert_eq!(res_left, 15);
}

#[test]
fn test_memo_ref_functor() {
	let memo = Memo::<_, RcMemoConfig>::new(|| 10);

	// map_ref takes a reference to the value
	let mapped = map_ref::<MemoBrand<RcMemoConfig>, _, _, _>(|x: &i32| *x * 2, memo);

	assert_eq!(*mapped.get(), 20);
}
