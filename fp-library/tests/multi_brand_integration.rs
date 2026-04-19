// Multi-brand integration tests for closure-directed inference.
//
// These tests validate InferableBrand-based inference for multi-brand types
// (Result, Pair, Tuple2, etc.). Tests are grouped by dispatch
// operation. Tests for operations not yet migrated to InferableBrand are
// #[ignore]d until their inference wrappers are rewritten.

use fp_library::{
	brands::*,
	classes::*,
	functions::*,
	types::*,
};

// -- map (functor) --

#[test]
fn p1_map_val_multi_brand_ok() {
	let r = map(|x: i32| x + 1, Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

#[test]
fn p1_map_val_multi_brand_err() {
	let r: Result<i32, usize> = map(|e: String| e.len(), Err::<i32, String>("hi".into()));
	assert_eq!(r, Err(2));
}

#[test]
fn p1_map_val_multi_brand_passthrough() {
	let r = map(|x: i32| x + 1, Err::<i32, String>("fail".into()));
	assert_eq!(r, Err("fail".to_string()));
}

#[test]
fn p1_map_ref_multi_brand_ok() {
	let ok: Result<i32, String> = Ok(5);
	let r: Result<i32, String> = map(|x: &i32| *x + 1, &ok);
	assert_eq!(r, Ok(6));
}

#[test]
fn p1_map_ref_multi_brand_err() {
	let err: Result<i32, String> = Err("hi".into());
	let r: Result<i32, usize> = map(|e: &String| e.len(), &err);
	assert_eq!(r, Err(2));
}

#[test]
fn p1_map_ref_multi_brand_passthrough() {
	let err: Result<i32, String> = Err("fail".into());
	let r: Result<i32, String> = map(|x: &i32| *x + 1, &err);
	assert_eq!(r, Err("fail".to_string()));
}

#[test]
fn p1_map_generic_fixed_param_ok() {
	fn process<E: 'static>(r: Result<i32, E>) -> Result<i32, E> {
		map(|x: i32| x + 1, r)
	}
	assert_eq!(process(Ok::<i32, String>(5)), Ok(6));
}

#[test]
fn p1_map_generic_fixed_param_passthrough() {
	fn process<E: 'static>(r: Result<i32, E>) -> Result<i32, E> {
		map(|x: i32| x + 1, r)
	}
	assert_eq!(process(Err::<i32, String>("x".into())), Err("x".to_string()));
}

#[test]
fn p1_map_generic_fixed_param_err_direction() {
	fn process<T: 'static>(r: Result<T, String>) -> Result<T, usize> {
		map(|e: String| e.len(), r)
	}
	assert_eq!(process(Err::<i32, String>("hi".into())), Err(2));
}

#[test]
fn p1_map_both_params_generic() {
	fn process<T: 'static + Clone, E: 'static>(r: Result<T, E>) -> Result<T, E> {
		map(|x: T| x.clone(), r)
	}
	assert_eq!(process(Ok::<i32, String>(10)), Ok(10));
}

#[test]
fn p1_map_ref_generic_fixed_param() {
	fn process<E>(r: &Result<i32, E>) -> Result<i32, E>
	where
		E: 'static + Clone, {
		map(|x: &i32| *x + 1, r)
	}
	assert_eq!(process(&Ok::<i32, String>(5)), Ok(6));
}

// -- bind (semimonad) --

#[test]
fn p2_bind_val_multi_brand() {
	let r: Result<String, String> = bind(Ok::<i32, String>(5), |x: i32| Ok(x.to_string()));
	assert_eq!(r, Ok("5".to_string()));
}

#[test]
fn p2_bind_val_multi_brand_passthrough() {
	let r: Result<String, String> =
		bind(Err::<i32, String>("fail".into()), |x: i32| Ok(x.to_string()));
	assert_eq!(r, Err("fail".to_string()));
}

#[test]
fn p2_bind_ref_multi_brand() {
	let ok: Result<i32, String> = Ok(5);
	let r: Result<String, String> = bind(&ok, |x: &i32| Ok(x.to_string()));
	assert_eq!(r, Ok("5".to_string()));
}

// -- bimap (bifunctor, arity 2) --
//
// ResultBrand's Of<'a, A, B> = Result<B, A> (error first, success second).
// So bimap's closure pair is (error_fn, success_fn) matching (A, B).

#[test]
fn p2_bimap_result_ok() {
	let r = bimap((|e: String| e.len(), |x: i32| x + 1), Ok::<i32, String>(5));
	assert_eq!(r, Ok(6));
}

#[test]
fn p2_bimap_result_err() {
	let r = bimap((|e: String| e.len(), |x: i32| x + 1), Err::<i32, String>("hi".into()));
	assert_eq!(r, Err(2));
}

// -- lift2 --

#[test]
fn p2_lift2_multi_brand() {
	let r = lift2(|a: i32, b: i32| a + b, Ok::<i32, String>(1), Ok::<i32, String>(2));
	assert_eq!(r, Ok(3));
}

#[test]
fn p2_lift2_multi_brand_short_circuit() {
	let r = lift2(|a: i32, b: i32| a + b, Ok::<i32, String>(1), Err::<i32, String>("x".into()));
	assert_eq!(r, Err("x".to_string()));
}

// -- fold_map (foldable) --

#[test]
fn p2_fold_map_multi_brand_ok() {
	let result = fold_map::<RcFnBrand, _, _, _, _>(|x: i32| x.to_string(), Ok::<i32, String>(5));
	assert_eq!(result, "5");
}

// -- traverse (traversable) --

#[test]
fn p2_traverse_multi_brand_ok() {
	let result =
		traverse::<RcFnBrand, _, _, _, OptionBrand, _>(|x: i32| Some(x + 1), Ok::<i32, String>(5));
	assert_eq!(result, Some(Ok(6)));
}

#[test]
fn p2_traverse_multi_brand_inner_failure() {
	let result =
		traverse::<RcFnBrand, _, _, _, OptionBrand, _>(|_x: i32| None::<i32>, Ok::<i32, String>(5));
	assert_eq!(result, None);
}

// -- apply (semiapplicative) --
//
// The apply inference wrapper cannot disambiguate multi-brand types like
// Result because Brand is inferred from the value container via
// InferableBrand, but Result has two impls (ResultErrAppliedBrand and
// ResultOkAppliedBrand). Multi-brand apply requires calling
// Semiapplicative::apply directly with an explicit Brand.

#[test]
fn p2_apply_val_multi_brand() {
	let f: Result<_, String> = Ok(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	let x: Result<i32, String> = Ok(5);
	let y: Result<i32, String> =
		ResultErrAppliedBrand::<String>::apply::<RcFnBrand, i32, i32>(f, x);
	assert_eq!(y, Ok(6));
}

#[test]
fn p2_apply_ref_multi_brand() {
	let f: Result<_, String> =
		Ok(std::rc::Rc::new(|x: &i32| *x + 1) as std::rc::Rc<dyn Fn(&i32) -> i32>);
	let x: Result<i32, String> = Ok(5);
	let y: Result<i32, String> =
		ResultErrAppliedBrand::<String>::ref_apply::<RcFnBrand, i32, i32>(&f, &x);
	assert_eq!(y, Ok(6));
}

// -- closureless multi-brand (explicit only) --

#[test]
fn p2_explicit_join_multi_brand() {
	use fp_library::functions::explicit::join as explicit_join;
	let r = explicit_join::<ResultErrAppliedBrand<String>, _, _>(Ok(Ok(5)));
	assert_eq!(r, Ok(5));
}

// -- closureless single-brand (must still infer via InferableBrand) --

#[test]
fn p2_join_single_brand_via_slot() {
	assert_eq!(join(Some(Some(5))), Some(5));
}

#[test]
fn p2_alt_single_brand_via_slot() {
	assert_eq!(alt(None::<i32>, Some(5)), Some(5));
}

// -- Other multi-brand types: Pair --

#[test]
fn p2_map_pair_first() {
	let p = Pair(5i32, "hello");
	let r = map(|x: i32| x + 1, p);
	assert_eq!(r, Pair(6, "hello"));
}

#[test]
fn p2_map_pair_second() {
	let p = Pair(5i32, "hello");
	let r: Pair<i32, usize> = map(|s: &str| s.len(), p);
	assert_eq!(r, Pair(5, 5));
}

// -- Other multi-brand types: Tuple2 --
//
// Tuple2 cannot use brand inference even with distinct types because
// it has multiple arity-1 brands. Use explicit::map with a turbofish.

#[test]
fn p2_map_tuple2_first() {
	use fp_library::functions::explicit::map as explicit_map;
	let r =
		explicit_map::<Tuple2SecondAppliedBrand<&str>, _, _, _, _>(|x: i32| x + 1, (5i32, "hello"));
	assert_eq!(r, (6, "hello"));
}

#[test]
fn p2_map_tuple2_second() {
	use fp_library::functions::explicit::map as explicit_map;
	let r = explicit_map::<Tuple2FirstAppliedBrand<i32>, _, _, _, _>(
		|s: &str| s.len(),
		(5i32, "hello"),
	);
	assert_eq!(r, (5, 5));
}

// -- Other multi-brand types: ControlFlow --

#[test]
fn p2_map_control_flow_continue() {
	use std::ops::ControlFlow;
	let r = map(|x: i32| x + 1, ControlFlow::<String, i32>::Continue(5));
	assert_eq!(r, ControlFlow::Continue(6));
}

// -- Other multi-brand types: TryThunk --

#[test]
fn p2_map_try_thunk_ok() {
	let t: TryThunk<i32, String> = TryThunk::pure(5);
	let r = map(|x: i32| x + 1, t);
	assert_eq!(r.evaluate(), Ok(6));
}
