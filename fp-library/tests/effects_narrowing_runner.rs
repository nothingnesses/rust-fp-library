//! The narrowing accumulator runner exercised over the public surface.
//!
//! Pins the generic runner's contract from the consumer side: one function,
//! generic over the eliminated effect brand, the source row, the residual
//! row, and the accumulator, eliminates the matched effect's operations by
//! threading an accumulator through them and re-emits every unmatched layer
//! into the residual row with the recursive continuation deferred, so the
//! walk is constant-stack per step. Two distinct instantiations pin the
//! genericity, an interpretation oracle pins the threading order, and depth
//! cases pin stack safety on both the matched (iterative) and unmatched
//! (deferred) paths.
#![cfg(feature = "effects")]

use {
	fp_library::{
		define_effect,
		define_row,
		types::{
			Coyoneda,
			Free,
			effects::handle::handle_accum,
		},
	},
	std::cell::Cell,
};

define_effect! {
	/// A running total: `add` contributes an amount and resumes with the
	/// updated total, so the effect's semantics are owned by whatever
	/// accumulator the interpreter threads.
	#[handler_state(threaded_by_value)]
	pub effect Counter {
		/// Add `amount` to the running total, resuming with the new total.
		fn add(amount: i32) -> i32;
	}
}

define_effect! {
	/// A unit step: each `tick` advances a counter the interpreter owns.
	#[handler_state(shared_by_reference)]
	pub effect Tick {
		/// Advance the counter, resuming with unit.
		fn tick() -> ();
	}
}

define_row! {
	/// The two-effect source row.
	pub row WideRow {
		CounterBrand,
		TickBrand,
	}
}

define_row! {
	/// The residual row once the counter cells are eliminated.
	pub row TickOnlyRow {
		TickBrand,
	}
}

define_row! {
	/// The residual row once the tick cells are eliminated.
	pub row CounterOnlyRow {
		CounterBrand,
	}
}

/// The residual-row driver for `TickOnlyRow`: interpret every tick against a
/// caller-owned counter and return the program's value.
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

/// The residual-row driver for `CounterOnlyRow`: interpret every add against
/// a caller-owned total and return the program's value.
fn run_counter<T: 'static>(
	mut program: Free<CounterOnlyRow, T>,
	total: &Cell<i32>,
) -> T {
	loop {
		let layer = match program.resume() {
			Ok(value) => return value,
			Err(layer) => layer,
		};
		let selected: Result<Coyoneda<'static, CounterBrand, Free<CounterOnlyRow, T>>, _> =
			layer.uninject();
		match selected {
			Ok(op) => match op.lower() {
				CounterF::Add(amount, resume) => {
					total.set(total.get() + amount);
					program = resume(total.get());
				}
			},
			Err(terminal) => match terminal {},
		}
	}
}

/// The accumulator step for the counter effect: thread the running total by
/// value and resume each operation with the updated total.
fn counter_step(
	s: i32,
	op: CounterF<'static, Free<WideRow, i32>>,
) -> (i32, Free<WideRow, i32>) {
	match op {
		CounterF::Add(amount, resume) => {
			let total = s + amount;
			(total, resume(total))
		}
	}
}

#[test]
fn threads_the_accumulator_through_matched_operations_in_program_order() {
	// tick; add 5 (total 5); tick; add 10 (total 15); the value observes both
	// intermediate totals, and the residual program still carries the ticks.
	let program: Free<WideRow, i32> = tick().bind(|()| {
		add(5).bind(|first| {
			tick().bind(move |()| add(10).bind(move |second| Free::pure(first * 1000 + second)))
		})
	});
	let narrowed: Free<TickOnlyRow, (i32, i32)> =
		handle_accum::<CounterBrand, _, _, _, _, _, _>(0, program, counter_step);
	let ticks = Cell::new(0);
	let (total, value) = run_ticks(narrowed, &ticks);
	assert_eq!(total, 15);
	assert_eq!(value, 5015);
	assert_eq!(ticks.get(), 2);
}

#[test]
fn a_second_instantiation_folds_a_log_over_a_different_brand_and_residual_row() {
	// The same generic function eliminates the OTHER effect (ticks) from the
	// same source row, folding a textual trace, and leaves the adds to the
	// residual driver: nothing in the runner is specific to one brand.
	let program: Free<WideRow, i32> =
		tick().bind(|()| add(3).bind(|_| tick().bind(|()| Free::pure(7))));
	let narrowed: Free<CounterOnlyRow, (String, i32)> = handle_accum::<TickBrand, _, _, _, _, _, _>(
		String::new(),
		program,
		|mut log, op| match op {
			TickF::Tick(resume) => {
				log.push('t');
				(log, resume(()))
			}
		},
	);
	let total = Cell::new(0);
	let (log, value) = run_counter(narrowed, &total);
	assert_eq!(log, "tt");
	assert_eq!(value, 7);
	assert_eq!(total.get(), 3);
}

/// The depth every deep case drives; the same order of magnitude as the
/// crate's other stack-safety suites.
const DEPTH: usize = 100_000;

#[test]
fn deep_matched_runs_are_iterative() {
	// 100k matched operations drive the runner's iterative arm only.
	let mut program: Free<WideRow, i32> = add(1).bind(|_| Free::pure(0));
	for _ in 1 .. DEPTH {
		program = add(1).bind(move |_| program);
	}
	let narrowed: Free<TickOnlyRow, (i32, i32)> =
		handle_accum::<CounterBrand, _, _, _, _, _, _>(0, program, counter_step);
	let ticks = Cell::new(0);
	let (total, value) = run_ticks(narrowed, &ticks);
	assert_eq!(total, DEPTH as i32);
	assert_eq!(value, 0);
	assert_eq!(ticks.get(), 0);
}

#[test]
fn deep_unmatched_runs_defer_constant_stack_per_step() {
	// 100k UNMATCHED layers drive the runner's deferred arm only: every tick
	// is re-emitted into the residual row and the recursive continuation
	// hides inside `bind`, invoked one layer at a time by the residual
	// driver's loop.
	let mut program: Free<WideRow, i32> = tick().bind(|()| Free::pure(0));
	for _ in 1 .. DEPTH {
		program = tick().bind(move |()| program);
	}
	let narrowed: Free<TickOnlyRow, (i32, i32)> =
		handle_accum::<CounterBrand, _, _, _, _, _, _>(0, program, counter_step);
	let ticks = Cell::new(0);
	let (total, value) = run_ticks(narrowed, &ticks);
	assert_eq!(total, 0);
	assert_eq!(value, 0);
	assert_eq!(ticks.get(), DEPTH as u64);
}
