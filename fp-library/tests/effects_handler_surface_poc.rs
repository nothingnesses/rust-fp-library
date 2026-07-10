//! Proof of concept for the one-pass handler surface: the hand-written
//! target expansion the `define_row!` extension will emit.
//!
//! Validates, over the public surface only, the arm grammar in which no arm
//! mentions the program result type: (1) first-order resumptive operations
//! get payloads-to-resume-value closure arms and the loop applies the
//! continuation itself; (2) no-resume operations get no arm and instead
//! populate a row-specific abort enum the loop returns through; (3) a
//! higher-order cell's elaboration arm is typed at the row's concrete pin
//! and receives a re-entry handle whose `Result` the arm inspects for
//! selective recovery; (4) one handler value drives programs at result
//! types other than the pin, and arm state lives behind interior mutability
//! so the loop and the re-entry share the handlers by reference.
#![cfg(feature = "effects")]

use {
	fp_library::{
		define_effect,
		define_row,
		types::{
			Coyoneda,
			Free,
			effects::{
				choose::{
					ChooseBrand,
					ChooseF,
					choose,
					empty,
				},
				state::{
					StateBrand,
					StateF,
					get,
					put,
				},
			},
		},
	},
	std::cell::Cell,
};

define_effect! {
	/// Abort the whole computation with a reason.
	#[handler_state(none)]
	pub effect Fail {
		/// Abort with `reason`; never resumes.
		fn fail(reason: &'static str) -> !;
	}
}

define_row! {
	/// Integer state, a payload-carrying abort, and integer scoped choice.
	pub row PocRow {
		StateBrand<i32>,
		FailBrand,
		ChooseBrand<PocRow, i32>,
	}
}

/// The row-specific abort union: one variant per no-resume operation in the
/// row, carrying that operation's payloads.
#[derive(Debug, PartialEq, Eq)]
enum PocAbort {
	/// The `fail` operation's payload.
	Fail(&'static str),
	/// The choice cell's branch-killing `empty`.
	ChooseEmpty,
}

/// The re-entry handle a higher-order arm receives: the loop instantiated at
/// the cell's pin type, over the same handler value.
type PocRetry<'r> = dyn Fn(Free<PocRow, i32>) -> Result<i32, PocAbort> + 'r;

/// The row's choice cell over programs yielding `T`, as selected by the
/// loop's brand-keyed `uninject`.
type PocChooseCell<T> = Coyoneda<'static, ChooseBrand<PocRow, i32>, Free<PocRow, T>>;

/// The handler list: one arm field per resumptive operation and per
/// higher-order cell; the no-resume operations have no field, because a
/// resume-less operation cannot be resumed, only reified into the abort
/// union.
struct PocHandlers<FGet, FPut, FChoose> {
	state_get: FGet,
	state_put: FPut,
	choose: FChoose,
}

/// The one-pass loop: brand-keyed dispatch over the whole row, generic over
/// the program result type so higher-order re-entry at the pin reuses the
/// same handler value.
fn poc_handle<T, FGet, FPut, FChoose>(
	mut program: Free<PocRow, T>,
	handlers: &PocHandlers<FGet, FPut, FChoose>,
) -> Result<T, PocAbort>
where
	T: 'static,
	FGet: Fn() -> i32,
	FPut: Fn(i32),
	FChoose: Fn(Free<PocRow, i32>, Free<PocRow, i32>, &PocRetry<'_>) -> Result<Vec<i32>, PocAbort>, {
	loop {
		let layer = match program.resume() {
			Ok(value) => return Ok(value),
			Err(layer) => layer,
		};
		let selected: Result<Coyoneda<'static, StateBrand<i32>, Free<PocRow, T>>, _> =
			layer.uninject();
		let layer = match selected {
			Ok(coyo) => {
				program = match coyo.lower() {
					StateF::Get(resume) => resume((handlers.state_get)()),
					StateF::Put(value, resume) => {
						(handlers.state_put)(value);
						resume(())
					}
				};
				continue;
			}
			Err(rest) => rest,
		};
		let selected: Result<Coyoneda<'static, FailBrand, Free<PocRow, T>>, _> = layer.uninject();
		let layer = match selected {
			Ok(coyo) => match coyo.lower() {
				FailF::Fail(reason, _) => return Err(PocAbort::Fail(reason)),
			},
			Err(rest) => rest,
		};
		let selected: Result<PocChooseCell<T>, _> = layer.uninject();
		match selected {
			Ok(coyo) => match coyo.lower() {
				ChooseF::Choose {
					left,
					right,
					k,
				} => {
					let retry = |sub: Free<PocRow, i32>| poc_handle(sub, handlers);
					let survivors = (handlers.choose)(left, right, &retry)?;
					program = k(survivors);
				}
				ChooseF::Empty(_) => return Err(PocAbort::ChooseEmpty),
			},
			Err(terminal) => match terminal {},
		}
	}
}

/// The collecting choice arm: each dead branch is recovered (it contributes
/// no survivor), every other abort propagates.
fn collecting_choose_arm(
	left: Free<PocRow, i32>,
	right: Free<PocRow, i32>,
	retry: &PocRetry<'_>,
) -> Result<Vec<i32>, PocAbort> {
	let mut survivors = Vec::new();
	for branch in [left, right] {
		match retry(branch) {
			Ok(value) => survivors.push(value),
			Err(PocAbort::ChooseEmpty) => {}
			Err(other) => return Err(other),
		}
	}
	Ok(survivors)
}

#[test]
fn one_handler_value_drives_a_program_at_a_result_type_other_than_the_pin() {
	// The program yields a String while the choice pin is i32; the state
	// arms are shared with the re-entry, so branch writes are global (the
	// right branch reads the left branch's write).
	let state = Cell::new(0);
	let handlers = PocHandlers {
		state_get: || state.get(),
		state_put: |value| state.set(value),
		choose: collecting_choose_arm,
	};
	let program: Free<PocRow, String> = put(1)
		.bind(|()| choose(put(10).bind(|()| get()), get()))
		.bind(|survivors: Vec<i32>| Free::pure(format!("{survivors:?}")));
	assert_eq!(poc_handle(program, &handlers), Ok("[10, 10]".to_string()));
	assert_eq!(state.get(), 10);
}

#[test]
fn a_no_resume_operation_reifies_into_the_row_abort_with_its_payload() {
	// Writes before the abort survive in the handler state (shared-cell
	// semantics); the abort carries the operation's payload.
	let state = Cell::new(0);
	let handlers = PocHandlers {
		state_get: || state.get(),
		state_put: |value| state.set(value),
		choose: collecting_choose_arm,
	};
	let program: Free<PocRow, i32> =
		put(7).bind(|()| fail::<i32, _, _>("boom")).bind(|value: i32| Free::pure(value));
	assert_eq!(poc_handle(program, &handlers), Err(PocAbort::Fail("boom")));
	assert_eq!(state.get(), 7);
}

#[test]
fn the_elaboration_arm_recovers_selectively_through_the_re_entry_result() {
	// A dead branch is recovered by the arm (no survivor); a failing branch
	// propagates through it untouched.
	let state = Cell::new(0);
	let handlers = PocHandlers {
		state_get: || state.get(),
		state_put: |value| state.set(value),
		choose: collecting_choose_arm,
	};
	let recovered: Free<PocRow, i32> = choose(empty::<_, i32, _, _>(), Free::pure(2))
		.bind(|survivors: Vec<i32>| Free::pure(survivors.iter().sum()));
	assert_eq!(poc_handle(recovered, &handlers), Ok(2));

	let propagated: Free<PocRow, i32> = choose(fail::<i32, _, _>("branch"), Free::pure(2))
		.bind(|survivors: Vec<i32>| Free::pure(survivors.iter().sum()));
	assert_eq!(poc_handle(propagated, &handlers), Err(PocAbort::Fail("branch")));
}

#[test]
fn deep_first_order_chains_drive_the_loop_iteratively() {
	// 100k state operations drive the loop's iterative matched arms only.
	const DEPTH: usize = 100_000;
	let state = Cell::new(0);
	let handlers = PocHandlers {
		state_get: || state.get(),
		state_put: |value| state.set(value),
		choose: collecting_choose_arm,
	};
	let mut program: Free<PocRow, i32> = put(0).bind(|()| get());
	for value in 1 .. DEPTH {
		program = put(value as i32).bind(move |()| program);
	}
	assert_eq!(poc_handle(program, &handlers), Ok(0));
	assert_eq!(state.get(), 0);
}
