//! The emitted one-pass handler surface, driven end to end: `#[handlers]`
//! rows compose the per-effect pieces `define_effect!` emits into a handler
//! struct, a row abort union, and the `RowHandler` loop.
//!
//! Pinned here: (1) first-order resumptive arms are payloads-to-resume-value
//! closures and the loop applies the continuation; (2) a no-resume operation
//! has no arm and reifies into the row abort with its payload; (3) a
//! higher-order cell's elaboration arm receives its owned sub-programs and a
//! re-entry handle at the row's pin, and recovers selectively by inspecting
//! the re-entry's `Result`; (4) one handler value drives programs at result
//! types other than the pin; (5) deep first-order chains drive the loop
//! iteratively.
#![cfg(feature = "effects")]

use {
	fp_library::{
		define_effect,
		define_row,
		types::{
			Free,
			effects::{
				choose::{
					ChooseAbort,
					ChooseArms,
					ChooseBrand,
					choose,
					empty,
				},
				handle::RowHandler,
				state::{
					StateArms,
					StateBrand,
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
	#[handlers]
	pub row AppRow {
		StateBrand<i32>,
		FailBrand,
		ChooseBrand<AppRow, i32>,
	}
}

/// Builds the collecting handler set over a borrowed state cell: dead
/// branches are recovered by the choice arm, every other abort propagates.
fn collecting_handlers(state: &Cell<i32>) -> AppRowHandlers<'_> {
	AppRowHandlers {
		state: StateArms {
			get: Box::new(|| state.get()),
			put: Box::new(|value| state.set(value)),
		},
		fail: FailArms,
		choose: ChooseArms {
			choose: Box::new(|left, right, reenter| {
				let mut survivors = Vec::new();
				for branch in [left, right] {
					match reenter(branch) {
						Ok(value) => survivors.push(value),
						Err(AppRowAbort::Choose(ChooseAbort::Empty)) => {}
						Err(other) => return Err(other),
					}
				}
				Ok(survivors)
			}),
		},
	}
}

#[test]
fn one_handler_value_drives_a_program_at_a_result_type_other_than_the_pin() {
	// The program yields a String while the choice pin is i32; the state
	// arms are shared with the re-entry, so branch writes are global (the
	// right branch reads the left branch's write).
	let state = Cell::new(0);
	let handlers = collecting_handlers(&state);
	let program: Free<AppRow, String> = put(1)
		.bind(|()| choose(put(10).bind(|()| get()), get()))
		.bind(|survivors: Vec<i32>| Free::pure(format!("{survivors:?}")));
	assert_eq!(handlers.handle(program).ok(), Some("[10, 10]".to_string()));
	assert_eq!(state.get(), 10);
}

#[test]
fn a_no_resume_operation_reifies_into_the_row_abort_with_its_payload() {
	// Writes before the abort survive in the handler state (shared-cell
	// semantics); the abort carries the operation's payload through the
	// cell's variant.
	let state = Cell::new(0);
	let handlers = collecting_handlers(&state);
	let program: Free<AppRow, i32> =
		put(7).bind(|()| fail::<i32, _, _, _>("boom")).bind(|value: i32| Free::pure(value));
	assert!(matches!(handlers.handle(program), Err(AppRowAbort::Fail(FailAbort::Fail("boom")))));
	assert_eq!(state.get(), 7);
}

#[test]
fn the_elaboration_arm_recovers_selectively_through_the_re_entry_result() {
	// A dead branch is recovered by the arm (no survivor); a failing branch
	// propagates through it untouched.
	let state = Cell::new(0);
	let handlers = collecting_handlers(&state);
	let recovered: Free<AppRow, i32> = choose(empty::<_, i32, _, _, _>(), Free::pure(2))
		.bind(|survivors: Vec<i32>| Free::pure(survivors.iter().sum()));
	assert_eq!(handlers.handle(recovered).ok(), Some(2));

	let propagated: Free<AppRow, i32> = choose(fail::<i32, _, _, _>("branch"), Free::pure(2))
		.bind(|survivors: Vec<i32>| Free::pure(survivors.iter().sum()));
	assert!(matches!(
		handlers.handle(propagated),
		Err(AppRowAbort::Fail(FailAbort::Fail("branch")))
	));
}

#[test]
fn a_misordered_handler_literal_still_constructs() {
	// Named-field construction is order-insensitive: the fields appear in
	// an order different from the row's declared cells.
	let state = Cell::new(3);
	let handlers = AppRowHandlers {
		choose: ChooseArms {
			choose: Box::new(|_left, _right, _reenter| Ok(Vec::new())),
		},
		fail: FailArms,
		state: StateArms {
			get: Box::new(|| state.get()),
			put: Box::new(|value| state.set(value)),
		},
	};
	let program: Free<AppRow, i32> = get();
	assert_eq!(handlers.handle(program).ok(), Some(3));
}

#[test]
fn deep_first_order_chains_drive_the_loop_iteratively() {
	// 100k state operations drive the loop's iterative matched arms only.
	const DEPTH: usize = 100_000;
	let state = Cell::new(0);
	let handlers = collecting_handlers(&state);
	let mut program: Free<AppRow, i32> = put(0).bind(|()| get());
	for value in 1 .. DEPTH {
		program = put(value as i32).bind(move |()| program);
	}
	assert_eq!(handlers.handle(program).ok(), Some(0));
	assert_eq!(state.get(), 0);
}
