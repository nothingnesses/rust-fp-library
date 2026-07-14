//! The coroutine step runner, driven end to end: `Coroutine<Out, In>`
//! yields an `Out` to the runner and resumes with an `In`, and
//! `handle_coroutine` drives a program to its first yield or its
//! completion, returning a yielded-or-done `Resume` whose continuation is
//! pre-folded through the runner itself, so a caller can never forget to
//! re-narrow. Every other effect re-emits into the residual row.
//!
//! Pinned here: (1) a yielded-or-done oracle passes over a row with a
//! residual `Writer`, the residual writes landing with the step that made
//! them; (2) a program with no yield is `Done` in one step.
#![cfg(feature = "effects")]

use fp_library::{
	brands::CNilBrand,
	define_row,
	types::{
		Free,
		effects::{
			coroutine::{
				CoroutineBrand,
				Resume,
				handle_coroutine,
				yield_value,
			},
			handle::extract,
			writer::{
				WriterBrand,
				handle_writer,
				tell,
			},
		},
	},
};

define_row! {
	/// Integer-for-integer yielding alongside a string log.
	pub row CoLogRow {
		CoroutineBrand<i32, i32>,
		WriterBrand<String>,
	}
}

define_row! {
	/// The residual after the coroutine cell is eliminated.
	pub row LogOnlyRow {
		WriterBrand<String>,
	}
}

/// The runner's step result at this suite's pins.
type LogResume = Resume<LogOnlyRow, i32, i32, i32>;

/// A fully narrowed step: the log the step wrote paired with its result.
type LogStep = Free<CNilBrand, (String, LogResume)>;

#[test]
fn the_runner_steps_to_each_yield_and_residual_effects_land_with_their_step() {
	// Step one: the program logs, yields 1, and suspends; the log written
	// before the yield comes out with the first step.
	let program: Free<CoLogRow, i32> = tell("start".to_string())
		.bind(|()| yield_value(1))
		.bind(|got: i32| tell(format!("got {got}")).bind(move |()| Free::pure(got + 1)));
	let stepped: Free<LogOnlyRow, LogResume> = handle_coroutine(program);
	let narrowed: LogStep = handle_writer(stepped);
	let (first_log, resume) = extract(narrowed);
	assert_eq!(first_log, "start");
	let suspended = match resume {
		Resume::Next(output, continuation) => Some((output, continuation)),
		Resume::Done(_) => None,
	};
	assert!(suspended.is_some(), "the program yields before completing");
	let Some((output, continuation)) = suspended else { return };
	assert_eq!(output, 1);

	// Step two: feeding 10 resumes the continuation, whose log and result
	// come out with the second step.
	let narrowed: LogStep = handle_writer(continuation(10));
	let (second_log, resume) = extract(narrowed);
	assert_eq!(second_log, "got 10");
	let done = match resume {
		Resume::Done(value) => Some(value),
		Resume::Next(..) => None,
	};
	assert_eq!(done, Some(11));
}

#[test]
fn a_program_with_no_yield_is_done_in_one_step() {
	let program: Free<CoLogRow, i32> = tell("only".to_string()).bind(|()| Free::pure(5));
	let stepped: Free<LogOnlyRow, LogResume> = handle_coroutine(program);
	let narrowed: LogStep = handle_writer(stepped);
	let (log, resume) = extract(narrowed);
	assert_eq!(log, "only");
	let done = match resume {
		Resume::Done(value) => Some(value),
		Resume::Next(..) => None,
	};
	assert_eq!(done, Some(5));
}
