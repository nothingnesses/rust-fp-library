//! The public catalog effects driven through their threaded narrowing
//! runners.
//!
//! Each runner is pinned by a threading oracle (interpretation results and
//! final accumulators over a mixed-effect program, against hand-computed
//! expectations) and a depth case on the order of 100k steps (the runners
//! defer their unmatched-path recursion, so native stack use per step is
//! constant).
#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::CNilBrand,
		define_effect,
		define_row,
		types::{
			Coyoneda,
			Free,
			effects::{
				handle::extract,
				state::{
					StateBrand,
					get,
					handle_state,
					put,
				},
			},
		},
	},
	std::cell::Cell,
};

define_effect! {
	/// A unit step: each `tick` advances a counter the interpreter owns.
	#[handler_state(shared_by_reference)]
	pub effect Tick {
		/// Advance the counter, resuming with unit.
		fn tick() -> ();
	}
}

define_row! {
	/// Integer state alongside an unrelated effect.
	pub row AppRow {
		StateBrand<i32>,
		TickBrand,
	}
}

define_row! {
	/// The residual row once the state cells are eliminated.
	pub row TickOnlyRow {
		TickBrand,
	}
}

define_row! {
	/// A one-cell row holding integer state.
	pub row StateOnlyRow {
		StateBrand<i32>,
	}
}

/// The residual-row driver: interpret every tick against a caller-owned
/// counter and return the program's value.
fn run_ticks<T: 'static>(
	mut program: Free<TickOnlyRow, T>,
	ticks: &Cell<u64>,
) -> T {
	loop {
		let layer = match program.resume() {
			Ok(value) => return value,
			Err(layer) => layer,
		};
		let selected: Result<Coyoneda<'static, TickBrand, Free<TickOnlyRow, T>>, _> =
			layer.uninject();
		match selected {
			Ok(op) => match op.lower() {
				TickF::Tick(resume) => {
					ticks.set(ticks.get() + 1);
					program = resume(());
				}
			},
			Err(terminal) => match terminal {},
		}
	}
}

#[test]
fn handle_state_threads_puts_and_gets_in_program_order() {
	// get (0); put 5; tick; get (5); the result observes both reads and the
	// final state is the last write; the tick survives into the residual.
	let program: Free<AppRow, i32> = get().bind(|first: i32| {
		put(5).bind(move |()| {
			tick().bind(move |()| get().bind(move |second| Free::pure(first * 100 + second)))
		})
	});
	let narrowed: Free<TickOnlyRow, (i32, i32)> = handle_state(0, program);
	let ticks = Cell::new(0);
	let (final_state, value) = run_ticks(narrowed, &ticks);
	assert_eq!(final_state, 5);
	assert_eq!(value, 5);
	assert_eq!(ticks.get(), 1);
}

#[test]
fn handle_state_narrows_a_single_effect_row_to_the_empty_row() {
	let program: Free<StateOnlyRow, i32> = put(4).bind(|()| get());
	let narrowed: Free<CNilBrand, (i32, i32)> = handle_state(0, program);
	assert_eq!(extract(narrowed), (4, 4));
}

/// The depth every deep case drives; the same order of magnitude as the
/// crate's other stack-safety suites.
const DEPTH: usize = 100_000;

#[test]
fn deep_state_chains_thread_iteratively() {
	// 100k puts drive the runner's iterative matched arm only.
	let mut program: Free<StateOnlyRow, i32> = put(0).bind(|()| get());
	for value in 1 .. DEPTH {
		program = put(value as i32).bind(move |()| program);
	}
	// The last write wins textually first: the chain is built back to front,
	// so the outermost operation is `put(DEPTH - 1)` and the innermost is
	// `put(0)` followed by the final `get`.
	let narrowed: Free<CNilBrand, (i32, i32)> = handle_state(-1, program);
	assert_eq!(extract(narrowed), (0, 0));
}

#[test]
fn deep_foreign_chains_defer_constant_stack_per_step() {
	// 100k unmatched layers drive the runner's deferred arm only.
	let mut program: Free<AppRow, i32> = tick().bind(|()| get());
	for _ in 1 .. DEPTH {
		program = tick().bind(move |()| program);
	}
	let narrowed: Free<TickOnlyRow, (i32, i32)> = handle_state(9, program);
	let ticks = Cell::new(0);
	let (final_state, value) = run_ticks(narrowed, &ticks);
	assert_eq!(final_state, 9);
	assert_eq!(value, 9);
	assert_eq!(ticks.get(), DEPTH as u64);
}
