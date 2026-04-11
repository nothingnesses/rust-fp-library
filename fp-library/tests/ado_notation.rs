use {
	fp_library::{
		brands::*,
		functions::*,
	},
	fp_macros::a_do,
};

// --- 0 binds: pure ---

#[test]
fn zero_binds_wraps_in_pure() {
	let result: Option<i32> = a_do!(OptionBrand { 42 });
	assert_eq!(result, Some(42));
}

#[test]
fn zero_binds_vec() {
	let result: Vec<i32> = a_do!(VecBrand { 7 });
	assert_eq!(result, vec![7]);
}

// --- 1 bind: map ---

#[test]
fn single_bind_uses_map() {
	let result = a_do!(OptionBrand {
		x <- Some(5);
		x * 2
	});
	assert_eq!(result, Some(10));
}

#[test]
fn single_bind_none_propagates() {
	let result = a_do!(OptionBrand {
		x: i32 <- None;
		x * 2
	});
	assert_eq!(result, None);
}

// --- 2 binds: lift2 ---

#[test]
fn two_binds_uses_lift2() {
	let result = a_do!(OptionBrand {
		x <- Some(3);
		y <- Some(4);
		x + y
	});
	assert_eq!(result, Some(7));
}

#[test]
fn two_binds_none_short_circuits() {
	let result = a_do!(OptionBrand {
		x <- Some(3);
		y: i32 <- None;
		x + y
	});
	assert_eq!(result, None);
}

// --- 3 binds: lift3 ---

#[test]
fn three_binds_uses_lift3() {
	let result = a_do!(OptionBrand {
		a <- Some(1);
		b <- Some(2);
		c <- Some(3);
		a + b + c
	});
	assert_eq!(result, Some(6));
}

// --- 4 binds: lift4 ---

#[test]
fn four_binds_uses_lift4() {
	let result = a_do!(OptionBrand {
		a <- Some(1);
		b <- Some(2);
		c <- Some(3);
		d <- Some(4);
		a + b + c + d
	});
	assert_eq!(result, Some(10));
}

// --- 5 binds: lift5 ---

#[test]
fn five_binds_uses_lift5() {
	let result = a_do!(OptionBrand {
		a <- Some(1);
		b <- Some(2);
		c <- Some(3);
		d <- Some(4);
		e <- Some(5);
		a + b + c + d + e
	});
	assert_eq!(result, Some(15));
}

// --- let bindings ---

#[test]
fn let_binding_inside_closure() {
	let result = a_do!(OptionBrand {
		x <- Some(5);
		y <- Some(3);
		let z = x + y;
		z * 2
	});
	assert_eq!(result, Some(16));
}

#[test]
fn typed_let_binding() {
	let result = a_do!(OptionBrand {
		x <- Some(5);
		let z: i32 = x * 2;
		z + 1
	});
	assert_eq!(result, Some(11));
}

#[test]
fn leading_let_hoisted_outside() {
	let result = a_do!(OptionBrand {
		let factor = 10;
		x <- Some(3);
		x * factor
	});
	assert_eq!(result, Some(30));
}

#[test]
fn leading_let_used_in_bind_expr() {
	let result = a_do!(OptionBrand {
		let base = 100;
		x <- Some(base + 1);
		x * 2
	});
	assert_eq!(result, Some(202));
}

// --- sequence (discard) ---

#[test]
fn sequence_as_discard_bind() {
	let result = a_do!(OptionBrand {
		Some(());
		x <- Some(42);
		x
	});
	assert_eq!(result, Some(42));
}

// --- pure rewriting in bind exprs ---

#[test]
fn pure_rewritten_in_bind_expr() {
	let result = a_do!(OptionBrand {
		x <- pure(10);
		y <- pure(20);
		x + y
	});
	assert_eq!(result, Some(30));
}

// --- Vec brand ---

#[test]
fn vec_brand_lift2() {
	let result = a_do!(VecBrand {
		x <- vec![1, 2];
		y <- vec![10, 20];
		x + y
	});
	assert_eq!(result, vec![11, 21, 12, 22]);
}

#[test]
fn vec_brand_map() {
	let result = a_do!(VecBrand {
		x <- vec![1, 2, 3];
		x * 10
	});
	assert_eq!(result, vec![10, 20, 30]);
}

// --- equivalence with manual calls ---

#[test]
fn equivalent_to_manual_lift2() {
	let ado_result = a_do!(OptionBrand {
		a <- Some(5);
		b <- Some(3);
		a - b
	});
	let manual_result =
		lift2_explicit::<OptionBrand, _, _, _, _, _, _>(|a, b| a - b, Some(5), Some(3));
	assert_eq!(ado_result, manual_result);
}

#[test]
fn equivalent_to_manual_map() {
	let ado_result = a_do!(OptionBrand {
		x <- Some(7);
		x + 1
	});
	let manual_result = map_explicit::<OptionBrand, _, _, _, _>(|x| x + 1, Some(7));
	assert_eq!(ado_result, manual_result);
}

#[test]
fn equivalent_to_manual_pure() {
	let ado_result: Option<i32> = a_do!(OptionBrand { 99 });
	let manual_result = pure::<OptionBrand, _>(99);
	assert_eq!(ado_result, manual_result);
}

// -- Ref mode tests --

#[test]
fn ref_mode_zero_binds() {
	let result: Option<i32> = a_do!(ref OptionBrand { 42 });
	assert_eq!(result, Some(42));
}

#[test]
fn ref_mode_single_bind_typed() {
	let result = a_do!(ref OptionBrand {
		x: &i32 <- Some(5);
		*x * 2
	});
	assert_eq!(result, Some(10));
}

#[test]
fn ref_mode_single_bind_untyped() {
	let result = a_do!(ref OptionBrand {
		x <- Some(5);
		*x * 3
	});
	assert_eq!(result, Some(15));
}

#[test]
fn ref_mode_two_binds() {
	let result = a_do!(ref OptionBrand {
		x: &i32 <- Some(3);
		y: &i32 <- Some(4);
		*x + *y
	});
	assert_eq!(result, Some(7));
}

#[test]
fn ref_mode_three_binds() {
	let result = a_do!(ref OptionBrand {
		a: &i32 <- Some(1);
		b: &i32 <- Some(2);
		c: &i32 <- Some(3);
		*a + *b + *c
	});
	assert_eq!(result, Some(6));
}

#[test]
fn ref_mode_none_short_circuits() {
	let result = a_do!(ref OptionBrand {
		x: &i32 <- Some(3);
		y: &i32 <- None;
		*x + *y
	});
	assert_eq!(result, None);
}

#[test]
fn ref_mode_sequence() {
	let result = a_do!(ref OptionBrand {
		Some(());
		x: &i32 <- Some(42);
		*x
	});
	assert_eq!(result, Some(42));
}

#[test]
fn ref_mode_vec_two_binds() {
	let result = a_do!(ref VecBrand {
		x: &i32 <- vec![1, 2];
		y: &i32 <- vec![10, 20];
		*x + *y
	});
	assert_eq!(result, vec![11, 21, 12, 22]);
}

#[test]
fn ref_mode_vec_single_bind() {
	let result = a_do!(ref VecBrand {
		x: &i32 <- vec![1, 2, 3];
		*x * 10
	});
	assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn ref_mode_let_binding() {
	let result = a_do!(ref OptionBrand {
		x: &i32 <- Some(5);
		y: &i32 <- Some(3);
		let z = *x + *y;
		z * 2
	});
	assert_eq!(result, Some(16));
}

#[test]
fn ref_mode_leading_let() {
	let result = a_do!(ref OptionBrand {
		let factor = 10;
		x: &i32 <- Some(3);
		*x * factor
	});
	assert_eq!(result, Some(30));
}

#[test]
fn ref_mode_equivalent_to_manual_ref_lift2() {
	let ado_result = a_do!(ref OptionBrand {
		a: &i32 <- Some(5);
		b: &i32 <- Some(3);
		*a - *b
	});
	let manual_result = lift2_explicit::<OptionBrand, _, _, _, _, _, _>(
		|a: &i32, b: &i32| *a - *b,
		&Some(5),
		&Some(3),
	);
	assert_eq!(ado_result, manual_result);
}

#[test]
fn ref_mode_equivalent_to_manual_ref_map() {
	let ado_result = a_do!(ref OptionBrand {
		x: &i32 <- Some(7);
		*x + 1
	});
	let manual_result = map_explicit::<OptionBrand, _, _, _, _>(|x: &i32| *x + 1, &Some(7));
	assert_eq!(ado_result, manual_result);
}

// -- Inferred mode tests --

#[test]
fn inferred_single_bind() {
	let result = a_do!({
		x <- Some(5);
		x * 2
	});
	assert_eq!(result, Some(10));
}

#[test]
fn inferred_two_binds() {
	let result = a_do!({
		x <- Some(5);
		y <- Some(10);
		x + y
	});
	assert_eq!(result, Some(15));
}

#[test]
fn inferred_vec_single_bind() {
	let result: Vec<i32> = a_do!({
		x <- vec![1, 2, 3];
		x * 10
	});
	assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn inferred_with_let() {
	let result = a_do!({
		let base = 100;
		x <- Some(5);
		base + x
	});
	assert_eq!(result, Some(105));
}
