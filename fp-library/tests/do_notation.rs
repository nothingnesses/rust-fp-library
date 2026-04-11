use {
	fp_library::{
		brands::*,
		functions::*,
	},
	fp_macros::m_do,
};

#[test]
fn basic_bind_chain() {
	let result = m_do!(OptionBrand {
		x <- Some(5);
		y <- Some(x + 1);
		pure(x + y)
	});
	assert_eq!(result, Some(11));
}

#[test]
fn let_binding() {
	let result = m_do!(OptionBrand {
		x <- Some(5);
		let z = x * 2;
		pure(z)
	});
	assert_eq!(result, Some(10));
}

#[test]
fn typed_let_binding() {
	let result = m_do!(OptionBrand {
		x <- Some(5);
		let z: i32 = x * 2;
		pure(z)
	});
	assert_eq!(result, Some(10));
}

#[test]
fn discard_bind() {
	let result = m_do!(OptionBrand {
		_ <- Some(());
		pure(42)
	});
	assert_eq!(result, Some(42));
}

#[test]
fn sequence_statement() {
	let result = m_do!(OptionBrand {
		Some(());
		pure(42)
	});
	assert_eq!(result, Some(42));
}

#[test]
fn short_circuit_on_none() {
	let result: Option<i32> = m_do!(OptionBrand {
		x <- Some(5);
		_ <- None::<()>;
		pure(x)
	});
	assert_eq!(result, None);
}

#[test]
fn pure_auto_rewriting() {
	// `pure(x)` is rewritten to `pure::<OptionBrand, _>(x)`
	let result = m_do!(OptionBrand {
		x <- Some(5);
		y <- pure(x + 1);
		pure(x + y)
	});
	assert_eq!(result, Some(11));
}

#[test]
fn pure_in_sequence_position() {
	let result = m_do!(OptionBrand {
		pure(());
		pure(42)
	});
	assert_eq!(result, Some(42));
}

#[test]
fn only_final_expression() {
	let result = m_do!(OptionBrand {
		pure(42)
	});
	assert_eq!(result, Some(42));
}

#[test]
fn vec_bind() {
	let result = m_do!(VecBrand {
		x <- vec![1, 2];
		y <- vec![10, 20];
		pure(x + y)
	});
	assert_eq!(result, vec![11, 21, 12, 22]);
}

#[test]
fn result_bind() {
	let result: Result<i32, &str> = m_do!(ResultErrAppliedBrand<&str> {
		x <- Ok(5);
		y <- Ok(x + 1);
		pure(x + y)
	});
	assert_eq!(result, Ok(11));
}

#[test]
fn result_short_circuit() {
	let result: Result<i32, &str> = m_do!(ResultErrAppliedBrand<&str> {
		x <- Ok(5);
		_: i32 <- Err("oops");
		pure(x)
	});
	assert_eq!(result, Err("oops"));
}

#[test]
fn equivalent_to_manual_bind() {
	// m_do! expansion should produce the same result as hand-written nested binds
	let do_result = m_do!(OptionBrand {
		x <- Some(5);
		y <- Some(x + 1);
		let z = x * y;
		pure(z)
	});

	let manual_result = bind_explicit::<OptionBrand, _, _, _, _>(Some(5), move |x| {
		bind_explicit::<OptionBrand, _, _, _, _>(Some(x + 1), move |y| {
			let z = x * y;
			pure::<OptionBrand, _>(z)
		})
	});

	assert_eq!(do_result, manual_result);
}

#[test]
fn multiple_let_bindings() {
	let result = m_do!(OptionBrand {
		x <- Some(3);
		let a = x + 1;
		let b = a * 2;
		y <- Some(b);
		pure(x + y)
	});
	assert_eq!(result, Some(11));
}

#[test]
fn typed_bind() {
	let result = m_do!(OptionBrand {
		x: i32 <- Some(5);
		pure(x * 2)
	});
	assert_eq!(result, Some(10));
}

#[test]
fn complex_expressions_in_bind() {
	let result = m_do!(OptionBrand {
		x: Vec<i32> <- Some(vec![1, 2, 3]);
		let sum: i32 = x.iter().sum();
		pure(sum)
	});
	assert_eq!(result, Some(6));
}

// -- Ref mode tests --

#[test]
fn ref_mode_option_typed() {
	let result = m_do!(ref OptionBrand {
		x: &i32 <- Some(5);
		pure(*x * 2)
	});
	assert_eq!(result, Some(10));
}

#[test]
fn ref_mode_option_untyped() {
	let result = m_do!(ref OptionBrand {
		x <- Some(5);
		pure(*x * 3)
	});
	assert_eq!(result, Some(15));
}

#[test]
fn ref_mode_option_short_circuit() {
	let result: Option<i32> = m_do!(ref OptionBrand {
		x: &i32 <- Some(5);
		let x_val = *x;
		_: &() <- None::<()>;
		pure(x_val)
	});
	assert_eq!(result, None);
}

#[test]
fn ref_mode_option_multi_bind() {
	let result = m_do!(ref OptionBrand {
		x: &i32 <- Some(5);
		let x_val = *x;
		y: &i32 <- Some(10);
		pure(x_val + *y)
	});
	assert_eq!(result, Some(15));
}

#[test]
fn ref_mode_option_let_only() {
	let result = m_do!(ref OptionBrand {
		let x = 42;
		pure(x)
	});
	assert_eq!(result, Some(42));
}

#[test]
fn ref_mode_option_sequence() {
	let result = m_do!(ref OptionBrand {
		Some(());
		pure(42)
	});
	assert_eq!(result, Some(42));
}

#[test]
fn ref_mode_option_zero_bind() {
	let result: Option<i32> = m_do!(ref OptionBrand {
		pure(99)
	});
	assert_eq!(result, Some(99));
}

#[test]
fn ref_mode_vec_typed() {
	let result = m_do!(ref VecBrand {
		x: &i32 <- vec![1, 2];
		let x_val = *x;
		y: &i32 <- vec![10, 20];
		pure(x_val + *y)
	});
	assert_eq!(result, vec![11, 21, 12, 22]);
}

#[test]
fn ref_mode_vec_untyped() {
	let result = m_do!(ref VecBrand {
		x <- vec![1, 2, 3];
		pure(*x * 10)
	});
	assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn ref_mode_result_typed() {
	let result: Result<i32, &str> = m_do!(ref ResultErrAppliedBrand<&str> {
		x: &i32 <- Ok(5);
		pure(*x * 2)
	});
	assert_eq!(result, Ok(10));
}

#[test]
fn ref_mode_result_short_circuit() {
	let result: Result<i32, &str> = m_do!(ref ResultErrAppliedBrand<&str> {
		x: &i32 <- Ok::<i32, &str>(5);
		let x_val = *x;
		_: &i32 <- Err::<i32, &str>("oops");
		pure(x_val)
	});
	assert_eq!(result, Err("oops"));
}
