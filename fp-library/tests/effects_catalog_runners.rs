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
				writer::{
					WriterBrand,
					fold_writer,
					handle_writer,
					tell,
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

define_row! {
	/// A string log alongside integer state.
	pub row LogAndStateRow {
		WriterBrand<String>,
		StateBrand<i32>,
	}
}

define_row! {
	/// A one-cell row telling integers.
	pub row TallyRow {
		WriterBrand<i32>,
	}
}

#[test]
fn fold_writer_folds_messages_in_program_order_and_leaves_state_to_the_residual() {
	// tell "a"; put 5; tell "b"; get: the fold sees "a" then "b", and the
	// state cells survive into the residual for handle_state to thread.
	let program: Free<LogAndStateRow, i32> =
		tell("a".to_string()).bind(|()| put(5).bind(|()| tell("b".to_string()).bind(|()| get())));
	let folded: Free<StateOnlyLogRow, (String, i32)> =
		fold_writer(String::new(), |log, message: String| log + &message, program);
	let narrowed: Free<CNilBrand, (i32, (String, i32))> = handle_state(0, folded);
	let (final_state, (log, value)) = extract(narrowed);
	assert_eq!(final_state, 5);
	assert_eq!(log, "ab");
	assert_eq!(value, 5);
}

define_row! {
	/// The residual row once the writer cells are eliminated.
	pub row StateOnlyLogRow {
		StateBrand<i32>,
	}
}

define_row! {
	/// A one-cell row telling strings.
	pub row LogOnlyRow {
		WriterBrand<String>,
	}
}

#[test]
fn handle_writer_accumulates_through_the_monoid() {
	let program: Free<LogOnlyRow, ()> =
		tell("Hello, ".to_string()).bind(|()| tell("World!".to_string()));
	let narrowed: Free<CNilBrand, (String, ())> = handle_writer(program);
	assert_eq!(extract(narrowed), ("Hello, World!".to_string(), ()));
}

#[test]
fn deep_writer_chains_fold_iteratively() {
	// 100k tells drive the runner's iterative matched arm only.
	let mut program: Free<TallyRow, ()> = tell(1);
	for _ in 1 .. DEPTH {
		program = tell(1).bind(move |()| program);
	}
	let narrowed: Free<CNilBrand, (i64, ())> =
		fold_writer(0_i64, |sum, message: i32| sum + i64::from(message), program);
	assert_eq!(extract(narrowed), (DEPTH as i64, ()));
}
