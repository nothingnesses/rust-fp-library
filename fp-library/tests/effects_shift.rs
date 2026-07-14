//! One-shot delimited continuations, driven end to end: `shift` captures
//! the continuation from the operation site to the delimiter as a
//! first-class one-shot value, and `run_shift` is the delimiter, a
//! narrowing fold whose return clause maps the program's result into the
//! prompt's answer program.
//!
//! Pinned here: (1) resuming the captured continuation runs the rest of
//! the source program to the delimiter; (2) dropping the continuation is
//! abort, structurally (the unresumed tail never runs); (3) the shift body
//! post-processes the resumed answer; (4) the prompt interleaves with
//! another effect through the re-emission path, the body reading state the
//! source program wrote.
#![cfg(feature = "effects")]

use fp_library::{
	brands::CNilBrand,
	define_row,
	types::{
		Free,
		effects::{
			handle::extract,
			shift::{
				ShiftBrand,
				run_shift,
				shift,
			},
			state::{
				StateBrand,
				get,
				handle_state,
				put,
			},
		},
	},
};

define_row! {
	/// A one-cell prompt delimiting straight to the empty row.
	pub row PureShiftRow {
		ShiftBrand<CNilBrand, i32, i32>,
	}
}

define_row! {
	/// The residual row a stateful prompt delimits to.
	pub row StateRow {
		StateBrand<i32>,
	}
}

define_row! {
	/// A prompt alongside integer state, delimiting to the state-only row.
	pub row PromptRow {
		ShiftBrand<StateRow, i32, i32>,
		StateBrand<i32>,
	}
}

#[test]
fn resuming_the_continuation_runs_the_rest_of_the_program() {
	let program: Free<PureShiftRow, i32> =
		shift::<_, _, i32, _, _>(|exit| exit(5)).bind(|v: i32| Free::pure(v + 1));
	let narrowed: Free<CNilBrand, i32> = run_shift(program, Free::pure);
	assert_eq!(extract(narrowed), 6);
}

#[test]
fn dropping_the_continuation_aborts_the_unresumed_tail() {
	// The body never invokes the continuation, so the source program's tail
	// (the +1) is discarded with it; the prompt answers 99 directly.
	let program: Free<PureShiftRow, i32> =
		shift::<_, _, i32, _, _>(|_exit| Free::pure(99)).bind(|v: i32| Free::pure(v + 1));
	let narrowed: Free<CNilBrand, i32> = run_shift(program, Free::pure);
	assert_eq!(extract(narrowed), 99);
}

#[test]
fn the_body_post_processes_the_resumed_answer() {
	// exit(5) runs the tail (+1) to the delimiter yielding 6; the body then
	// doubles the completed answer: shift's defining composition.
	let program: Free<PureShiftRow, i32> =
		shift::<_, _, i32, _, _>(|exit| exit(5).bind(|ans: i32| Free::pure(ans * 2)))
			.bind(|v: i32| Free::pure(v + 1));
	let narrowed: Free<CNilBrand, i32> = run_shift(program, Free::pure);
	assert_eq!(extract(narrowed), 12);
}

#[test]
fn the_prompt_interleaves_with_state_through_the_re_emission_path() {
	// The source program writes state before capturing; the body, running
	// over the residual row, reads that state and resumes with it; the
	// resumed tail doubles. 1 written, body reads 1, resumes 11, tail 22.
	let program: Free<PromptRow, i32> = put(1)
		.bind(|()| shift::<_, _, i32, _, _>(|exit| get().bind(move |s: i32| exit(s + 10))))
		.bind(|v: i32| Free::pure(v * 2));
	let narrowed: Free<StateRow, i32> = run_shift(program, Free::pure);
	let handled: Free<CNilBrand, (i32, i32)> = handle_state(0, narrowed);
	assert_eq!(extract(handled), (1, 22));
}
