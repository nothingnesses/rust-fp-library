// Feasibility tests for brand inference with the post-ref-borrow dispatch system.
//
// Validates that the `InferableBrand` trait + blanket `impl for &T` enables
// turbofish-free map/bind calls that compose with the two-impl FA dispatch
// pattern for both Val and Ref dispatch.

use fp_library::{
	classes::Pointed,
	dispatch::{
		functor::FunctorDispatch,
		semimonad::BindDispatch,
	},
	kinds::{
		InferableBrand_cdc7cd43dac7585f,
		Kind_cdc7cd43dac7585f,
	},
	types::*,
};

// -- Inference-based map function --

fn map<'a, FA, A: 'a, B: 'a, Marker>(
	f: impl FunctorDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, B, FA, Marker>,
	fa: FA,
) -> <<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
where
	FA: InferableBrand_cdc7cd43dac7585f, {
	f.dispatch(fa)
}

// -- Val dispatch tests --

#[test]
fn val_option_infer() {
	let result: Option<i32> = map(|x: i32| x * 2, Some(5));
	assert_eq!(result, Some(10));
}

#[test]
fn val_vec_infer() {
	let result: Vec<i32> = map(|x: i32| x + 1, vec![1, 2, 3]);
	assert_eq!(result, vec![2, 3, 4]);
}

#[test]
fn val_none_infer() {
	let result: Option<i32> = map(|x: i32| x * 2, None::<i32>);
	assert_eq!(result, None);
}

#[test]
fn val_different_output_type() {
	let result: Vec<String> = map(|x: i32| x.to_string(), vec![1, 2]);
	assert_eq!(result, vec!["1", "2"]);
}

#[test]
fn val_option_different_output_type() {
	let result = map(|x: i32| x.to_string(), Some(5));
	assert_eq!(result, Some("5".to_string()));
}

// -- Ref dispatch tests (the key new capability) --

#[test]
fn ref_option_infer() {
	let opt = Some(5);
	let result: Option<i32> = map(|x: &i32| *x * 2, &opt);
	assert_eq!(result, Some(10));
}

#[test]
fn ref_vec_infer() {
	let v = vec![1, 2, 3];
	let result: Vec<i32> = map(|x: &i32| *x + 1, &v);
	assert_eq!(result, vec![2, 3, 4]);
}

#[test]
fn ref_lazy_infer() {
	let lazy = RcLazy::pure(10);
	let result = map(|x: &i32| *x * 2, &lazy);
	assert_eq!(*result.evaluate(), 20);
}

#[test]
fn ref_option_reuse_after_map() {
	let opt = Some(5);
	let r1 = map(|x: &i32| *x * 2, &opt);
	let r2 = map(|x: &i32| *x + 1, &opt);
	assert_eq!(r1, Some(10));
	assert_eq!(r2, Some(6));
}

#[test]
fn ref_vec_reuse_after_map() {
	let v = vec![1, 2, 3];
	let r1: Vec<i32> = map(|x: &i32| *x * 10, &v);
	let r2: Vec<i32> = map(|x: &i32| *x + 100, &v);
	assert_eq!(r1, vec![10, 20, 30]);
	assert_eq!(r2, vec![101, 102, 103]);
}

// -- Temporary borrow tests --

#[test]
fn ref_temporary_borrow() {
	let result: Vec<i32> = map(|x: &i32| *x + 1, &vec![1, 2, 3]);
	assert_eq!(result, vec![2, 3, 4]);
}

#[test]
fn ref_temporary_option() {
	let result: Option<i32> = map(|x: &i32| *x * 3, &Some(7));
	assert_eq!(result, Some(21));
}

// -- Mixed mode in same scope --

#[test]
fn mixed_val_then_ref() {
	let v = vec![1, 2, 3];
	let ref_result: Vec<i32> = map(|x: &i32| *x * 10, &v);
	let val_result: Vec<i32> = map(|x: i32| x + 1, v);
	assert_eq!(ref_result, vec![10, 20, 30]);
	assert_eq!(val_result, vec![2, 3, 4]);
}

// Note: the following should NOT compile because Result has no InferableBrand
// (multiple brands make it ambiguous):
// let _ = map(|x: &i32| *x, &Ok::<i32, String>(5));

// -- Inference-based bind function --

fn bind_infer<'a, FA, A: 'a, B: 'a, Marker>(
	fa: FA,
	f: impl BindDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, B, FA, Marker>,
) -> <<FA as InferableBrand_cdc7cd43dac7585f>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
where
	FA: InferableBrand_cdc7cd43dac7585f, {
	f.dispatch(fa)
}

// -- Inference-based pure function (no brand turbofish) --

#[expect(unused, reason = "Feasibility test, verifies compilation not runtime behavior")]
fn pure_infer<Brand: Pointed, A>(a: A) -> <Brand as Kind_cdc7cd43dac7585f>::Of<'static, A>
where
	A: 'static, {
	Brand::pure(a)
}

// -- bind_infer tests --

#[test]
fn bind_val_option_infer() {
	let result = bind_infer(Some(5), |x: i32| Some(x * 2));
	assert_eq!(result, Some(10));
}

#[test]
fn bind_val_vec_infer() {
	let result: Vec<i32> = bind_infer(vec![1, 2], |x: i32| vec![x, x * 10]);
	assert_eq!(result, vec![1, 10, 2, 20]);
}

#[test]
fn bind_ref_option_infer() {
	let opt = Some(5);
	let result = bind_infer(&opt, |x: &i32| Some(*x * 2));
	assert_eq!(result, Some(10));
}

#[test]
fn bind_ref_vec_infer() {
	let v = vec![1, 2];
	let result: Vec<i32> = bind_infer(&v, |x: &i32| vec![*x, *x * 10]);
	assert_eq!(result, vec![1, 10, 2, 20]);
}

#[test]
fn bind_ref_lazy_infer() {
	let lazy = RcLazy::pure(5);
	let result = bind_infer(&lazy, |x: &i32| {
		let v = *x * 2;
		Lazy::<_, RcLazyConfig>::new(move || v)
	});
	assert_eq!(*result.evaluate(), 10);
}

// -- pure return-type inference tests --
//
// CONFIRMED: pure's Brand CANNOT be inferred from bind's closure return type.
// Rust does not propagate return-type constraints backward through GAT
// projections. The compiler reports E0283 "type annotations needed" because
// multiple types implement Pointed and it cannot select one from the return
// position constraint alone.
//
// This means in inferred-mode m_do!/a_do!, `pure(expr)` cannot be used without
// a brand turbofish. Users must write the concrete constructor (e.g., Some(expr))
// or use the explicit-brand macro syntax.
//
// The following tests are commented out to document the confirmed failure:
//
// fn pure_inferred_from_bind_return_type() {
//     let result = bind_infer(Some(5), |x: i32| pure_infer(x + 1));
//     // E0283: cannot infer type of the type parameter `Brand`
// }
//
// fn pure_inferred_from_nested_bind() {
//     let result = bind_infer(Some(5), |x: i32| {
//         bind_infer(Some(x + 1), |y: i32| pure_infer(x + y))
//     });
//     // E0283: same error
// }
//
// fn pure_inferred_vec() {
//     let result: Vec<i32> = bind_infer(vec![1, 2], |x: i32| pure_infer(x * 10));
//     // E0283: same error
// }

// -- Tests using the real inference map from dispatch::inference --

#[test]
fn real_infer_val_option() {
	use fp_library::functions::map;
	assert_eq!(map(|x: i32| x * 2, Some(5)), Some(10));
}

#[test]
fn real_infer_val_vec() {
	use fp_library::functions::map;
	assert_eq!(map(|x: i32| x + 1, vec![1, 2, 3]), vec![2, 3, 4]);
}

#[test]
fn real_infer_ref_option() {
	use fp_library::functions::map;
	assert_eq!(map(|x: &i32| *x * 2, &Some(5)), Some(10));
}

#[test]
fn real_infer_ref_vec() {
	use fp_library::functions::map;
	let v = vec![1, 2, 3];
	assert_eq!(map(|x: &i32| *x + 10, &v), vec![11, 12, 13]);
}
