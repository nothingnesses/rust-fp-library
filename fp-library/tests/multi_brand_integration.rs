// Multi-brand integration tests for closure-directed inference.
//
// These tests validate Slot-based inference for multi-brand types
// (Result, Pair, Tuple2, etc.). Tests are grouped by dispatch
// operation. Tests for operations not yet migrated to Slot are
// #[ignore]d until their inference wrappers are rewritten.

use fp_library::functions::*;

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
#[ignore = "bind inference wrapper not yet migrated to Slot"]
fn p2_bind_val_multi_brand() {
	// bind(Ok::<i32, String>(5), |x: i32| Ok(x.to_string())) == Ok("5")
}

#[test]
#[ignore = "bind inference wrapper not yet migrated to Slot"]
fn p2_bind_val_multi_brand_passthrough() {
	// bind(Err::<i32, String>("fail".into()), |x: i32| Ok(x.to_string())) == Err("fail")
}

#[test]
#[ignore = "bind inference wrapper not yet migrated to Slot"]
fn p2_bind_ref_multi_brand() {
	// bind(&Ok::<i32, String>(5), |x: &i32| Ok(x.to_string())) == Ok("5")
}

// -- bimap (bifunctor, arity 2) --

#[test]
#[ignore = "bimap inference wrapper not yet migrated to Slot"]
fn p2_bimap_result_ok() {
	// bimap(|x: i32| x + 1, |e: String| e.len(), Ok::<i32, String>(5)) == Ok(6)
}

#[test]
#[ignore = "bimap inference wrapper not yet migrated to Slot"]
fn p2_bimap_result_err() {
	// bimap(|x: i32| x + 1, |e: String| e.len(), Err::<i32, String>("hi".into())) == Err(2)
}

// -- lift2 --

#[test]
#[ignore = "lift2 inference wrapper not yet migrated to Slot"]
fn p2_lift2_multi_brand() {
	// lift2(|a: i32, b: i32| a + b, Ok::<i32, String>(1), Ok::<i32, String>(2)) == Ok(3)
}

#[test]
#[ignore = "lift2 inference wrapper not yet migrated to Slot"]
fn p2_lift2_multi_brand_short_circuit() {
	// lift2(|a: i32, b: i32| a + b, Ok::<i32, String>(1), Err("x".into())) == Err("x")
}

// -- closureless single-brand (must still infer via Slot) --

#[test]
#[ignore = "join inference wrapper not yet migrated to Slot"]
fn p2_join_single_brand_via_slot() {
	// join(Some(Some(5))) == Some(5)
}

#[test]
#[ignore = "alt inference wrapper not yet migrated to Slot"]
fn p2_alt_single_brand_via_slot() {
	// alt(None::<i32>, Some(5)) == Some(5)
}
