use {
	core::ops::ControlFlow,
	fp_library::{
		brands::{
			LazyBrand,
			RcFnBrand,
			ThunkBrand,
		},
		classes::{
			foldable::Foldable,
			functor_dispatch::map,
			monad_rec::tail_rec_m,
		},
		types::{
			Lazy,
			RcLazyConfig,
			Thunk,
		},
	},
};

#[test]
fn test_eval_monad_rec() {
	// Factorial using tail_rec_m
	fn factorial(n: u64) -> Thunk<'static, u64> {
		tail_rec_m::<ThunkBrand, _, _>(
			|(n, acc)| {
				if n == 0 {
					Thunk::pure(ControlFlow::Break(acc))
				} else {
					Thunk::pure(ControlFlow::Continue((n - 1, n * acc)))
				}
			},
			(n, 1),
		)
	}

	assert_eq!(factorial(5).evaluate(), 120);
}

#[test]
fn test_eval_foldable() {
	// Thunk contains a single value, so fold should just apply the function once

	// fold_right: (A, B) -> B
	let res_right = ThunkBrand::fold_right::<RcFnBrand, _, _>(|a, b| a + b, 5, Thunk::pure(10));
	assert_eq!(res_right, 15);

	// fold_left: (B, A) -> B
	let res_left = ThunkBrand::fold_left::<RcFnBrand, _, _>(|b, a| b + a, 5, Thunk::pure(10));
	assert_eq!(res_left, 15);
}

#[test]
fn test_memo_ref_functor() {
	let memo = Lazy::<_, RcLazyConfig>::new(|| 10);

	// map_ref takes a reference to the value
	let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, memo);

	assert_eq!(*mapped.evaluate(), 20);
}
