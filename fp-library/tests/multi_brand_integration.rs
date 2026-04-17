// Multi-brand integration tests for closure-directed inference.
//
// These tests validate the new Slot-based inference for multi-brand
// types (Result, Pair, Tuple2, ControlFlow, TryThunk). Tests are
// organized by phase and start #[ignore]d. Each phase's tests are
// un-ignored when the corresponding dispatch modules are migrated.

// -- Phase 1: map (functor) --

// Tests in this section are un-ignored in phase 1 step 7 once
// the Slot-based map dispatch is in place. Until then, the test
// bodies are empty stubs because the calls would not compile
// against the current InferableBrand-based dispatch (Result does
// not implement InferableBrand at arity 1).

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_val_multi_brand_ok() {
	// map(|x: i32| x + 1, Ok::<i32, String>(5)) == Ok(6)
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_val_multi_brand_err() {
	// map(|e: String| e.len(), Err::<i32, String>("hi".into())) == Err(2)
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_val_multi_brand_passthrough() {
	// map(|x: i32| x + 1, Err::<i32, String>("fail".into())) == Err("fail")
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_ref_multi_brand_ok() {
	// map(|x: &i32| *x + 1, &Ok::<i32, String>(5)) == Ok(6)
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_ref_multi_brand_err() {
	// map(|e: &String| e.len(), &Err::<i32, String>("hi".into())) == Err(2)
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_ref_multi_brand_passthrough() {
	// map(|x: &i32| *x + 1, &Err::<i32, String>("fail".into())) == Err("fail")
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_generic_fixed_param_ok() {
	// fn f<E: 'static>(r: Result<i32, E>) -> Result<i32, E> { map(|x: i32| x + 1, r) }
	// f(Ok::<i32, String>(5)) == Ok(6)
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_generic_fixed_param_passthrough() {
	// fn f<E: 'static>(r: Result<i32, E>) -> Result<i32, E> { map(|x: i32| x + 1, r) }
	// f(Err::<i32, String>("x".into())) == Err("x")
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_generic_fixed_param_err_direction() {
	// fn f<T: 'static>(r: Result<T, String>) -> Result<T, usize> { map(|e: String| e.len(), r) }
	// f(Err("hi".into())) == Err(2)
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_both_params_generic() {
	// fn f<T: 'static + Clone, E: 'static>(r: Result<T, E>) -> Result<T, E> {
	//     map(|x: T| x.clone(), r)
	// }
	// f(Ok::<i32, String>(10)) == Ok(10)
}

#[test]
#[ignore = "phase 1: requires Slot-based map dispatch"]
fn p1_map_ref_generic_fixed_param() {
	// fn f<E: 'static + Clone>(r: &Result<i32, E>) -> Result<i32, E> {
	//     map(|x: &i32| *x + 1, r)
	// }
	// f(&Ok::<i32, String>(5)) == Ok(6)
}

// -- Phase 2: bind (semimonad) --

#[test]
#[ignore = "phase 2: requires Slot-based bind dispatch"]
fn p2_bind_val_multi_brand() {
	// bind(Ok::<i32, String>(5), |x: i32| Ok(x.to_string())) == Ok("5")
}

#[test]
#[ignore = "phase 2: requires Slot-based bind dispatch"]
fn p2_bind_val_multi_brand_passthrough() {
	// bind(Err::<i32, String>("fail".into()), |x: i32| Ok(x.to_string())) == Err("fail")
}

#[test]
#[ignore = "phase 2: requires Slot-based bind dispatch"]
fn p2_bind_ref_multi_brand() {
	// bind(&Ok::<i32, String>(5), |x: &i32| Ok(x.to_string())) == Ok("5")
}

// -- Phase 2: bimap (bifunctor, arity 2) --

#[test]
#[ignore = "phase 2: requires Slot-based bimap dispatch"]
fn p2_bimap_result_ok() {
	// bimap(|x: i32| x + 1, |e: String| e.len(), Ok::<i32, String>(5)) == Ok(6)
}

#[test]
#[ignore = "phase 2: requires Slot-based bimap dispatch"]
fn p2_bimap_result_err() {
	// bimap(|x: i32| x + 1, |e: String| e.len(), Err::<i32, String>("hi".into())) == Err(2)
}

// -- Phase 2: lift2 --

#[test]
#[ignore = "phase 2: requires Slot-based lift dispatch"]
fn p2_lift2_multi_brand() {
	// lift2(|a: i32, b: i32| a + b, Ok::<i32, String>(1), Ok::<i32, String>(2)) == Ok(3)
}

#[test]
#[ignore = "phase 2: requires Slot-based lift dispatch"]
fn p2_lift2_multi_brand_short_circuit() {
	// lift2(|a: i32, b: i32| a + b, Ok::<i32, String>(1), Err("x".into())) == Err("x")
}

// -- Phase 2: closureless single-brand (must still infer via Slot) --

#[test]
#[ignore = "phase 2: requires Slot-based join dispatch"]
fn p2_join_single_brand_via_slot() {
	// join(Some(Some(5))) == Some(5)
	// Confirms single-brand closureless inference still works after Slot migration.
}

#[test]
#[ignore = "phase 2: requires Slot-based alt dispatch"]
fn p2_alt_single_brand_via_slot() {
	// alt(None::<i32>, Some(5)) == Some(5)
	// Confirms single-brand closureless inference still works after Slot migration.
}
