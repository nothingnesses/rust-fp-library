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
				choose::{
					ChooseBrand,
					choose,
					empty,
					handle_choose,
					handle_choose_accum,
				},
				handle::extract,
				state::{
					StateBrand,
					StateStep,
					get,
					handle_state,
					put,
				},
				writer::{
					FoldWriterStep,
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

define_row! {
	/// Integer choice alongside integer state. The row names itself in the
	/// choice cell (branch sub-programs run over the full row), legal
	/// through the nominal row brand's lazy kind projection.
	pub row ChoiceStateRow {
		ChooseBrand<ChoiceStateRow, i32>,
		StateBrand<i32>,
	}
}

define_row! {
	/// A one-cell row choosing integers.
	pub row ChoiceOnlyRow {
		ChooseBrand<ChoiceOnlyRow, i32>,
	}
}

#[test]
fn handle_choose_collects_surviving_branch_values() {
	// choose(1, 2) resumes exactly once with [1, 2]; the continuation sums.
	let program: Free<ChoiceStateRow, i32> = choose(Free::pure(1), Free::pure(2))
		.bind(|values: Vec<i32>| Free::pure(values.iter().sum()));
	let narrowed: Free<StateOnlyRow, Option<i32>> = handle_choose(program);
	let stated: Free<CNilBrand, (i32, Option<i32>)> = handle_state(0, narrowed);
	assert_eq!(extract(stated), (0, Some(3)));
}

#[test]
fn empty_kills_a_branch_and_the_top_level() {
	// A dead left branch contributes nothing; the survivor list is [2].
	let program: Free<ChoiceStateRow, i32> = choose(empty::<_, i32, _, _>(), Free::pure(2))
		.bind(|values: Vec<i32>| Free::pure(values.iter().sum()));
	let narrowed: Free<StateOnlyRow, Option<i32>> = handle_choose(program);
	let stated: Free<CNilBrand, (i32, Option<i32>)> = handle_state(0, narrowed);
	assert_eq!(extract(stated), (0, Some(2)));

	// A top-level empty kills the whole program: the top level is a branch.
	let dead: Free<ChoiceStateRow, i32> = empty::<_, i32, _, _>();
	let narrowed: Free<StateOnlyRow, Option<i32>> = handle_choose(dead);
	let stated: Free<CNilBrand, (i32, Option<i32>)> = handle_state(7, narrowed);
	assert_eq!(extract(stated), (7, None));
}

#[test]
fn handle_choose_accum_forks_the_accumulator_per_branch() {
	// Branch-local order: the left branch writes 10 and reads it back; the
	// right branch reads its own untouched fork of the initial state; the
	// trunk continues with the original accumulator.
	let program: Free<ChoiceStateRow, Vec<i32>> =
		choose(put(10).bind(|()| get()), get()).bind(|values: Vec<i32>| Free::pure(values));
	let narrowed: Free<CNilBrand, (i32, Option<Vec<i32>>)> =
		handle_choose_accum(1, program, StateStep);
	assert_eq!(extract(narrowed), (1, Some(vec![10, 1])));
}

#[test]
fn stacked_choose_and_state_runners_thread_one_global_state() {
	// Global order: eliminating choice first sequences both branches' state
	// operations into one residual, so the state runner stacked outside
	// threads one state through them and the right branch observes the
	// left's write.
	let program: Free<ChoiceStateRow, Vec<i32>> =
		choose(put(10).bind(|()| get()), get()).bind(|values: Vec<i32>| Free::pure(values));
	let narrowed: Free<StateOnlyRow, Option<Vec<i32>>> = handle_choose(program);
	let stated: Free<CNilBrand, (i32, Option<Vec<i32>>)> = handle_state(1, narrowed);
	assert_eq!(extract(stated), (10, Some(vec![10, 10])));
}

#[test]
fn deep_choose_chains_defer_constant_stack_per_step() {
	// 100k sequential chooses, each resumed exactly once with its survivor
	// pair; the runner defers each continuation into `bind`, so native stack
	// use per step is constant (it grows with choice nesting depth, and this
	// chain nests none).
	let mut program: Free<ChoiceOnlyRow, i32> = Free::pure(0);
	for _ in 0 .. DEPTH {
		program = choose(Free::pure(1), Free::pure(2)).bind(move |values: Vec<i32>| {
			let step_total: i32 = values.iter().sum();
			program.bind(move |acc| Free::pure(acc + step_total))
		});
	}
	let narrowed: Free<CNilBrand, Option<i32>> = handle_choose(program);
	assert_eq!(extract(narrowed), Some(3 * DEPTH as i32));
}

#[test]
fn deep_foreign_chains_under_choose_defer_constant_stack_per_step() {
	// 100k unmatched state layers drive handle_choose's deferred arm only.
	let mut program: Free<ChoiceStateRow, i32> = put(0).bind(|()| get());
	for value in 1 .. DEPTH {
		program = put(value as i32).bind(move |()| program);
	}
	let narrowed: Free<StateOnlyRow, Option<i32>> = handle_choose(program);
	let stated: Free<CNilBrand, (i32, Option<i32>)> = handle_state(-1, narrowed);
	assert_eq!(extract(stated), (0, Some(0)));
}

/// The state-under-choice program the ordering zoo drives: a shared prefix
/// write, then two branches that each read, advance, and re-read the state.
/// Whether the right branch observes the left branch's write is exactly what
/// the runner order decides.
fn state_choice_zoo_program() -> Free<ChoiceStateRow, Vec<i32>> {
	put(1)
		.bind(|()| {
			choose(
				get().bind(|seen: i32| put(seen + 2).bind(move |()| get())),
				get().bind(|seen: i32| put(seen + 3).bind(move |()| get())),
			)
		})
		.bind(|values: Vec<i32>| Free::pure(values))
}

#[test]
fn state_inside_choice_scopes_writes_per_branch() {
	// Branch-local order: each branch runs on its own fork of the state after
	// the prefix write, so both read 1, and their writes die with the branch;
	// the trunk continues with the prefix state.
	let narrowed: Free<CNilBrand, (i32, Option<Vec<i32>>)> =
		handle_choose_accum(0, state_choice_zoo_program(), StateStep);
	assert_eq!(extract(narrowed), (1, Some(vec![3, 4])));
}

#[test]
fn state_outside_choice_threads_one_state_through_branches() {
	// Global order: eliminating choice first sequences both branches' state
	// operations into one residual, so the right branch reads the left
	// branch's write (3) and advances it to 6.
	let narrowed: Free<StateOnlyRow, Option<Vec<i32>>> = handle_choose(state_choice_zoo_program());
	let stated: Free<CNilBrand, (i32, Option<Vec<i32>>)> = handle_state(0, narrowed);
	assert_eq!(extract(stated), (6, Some(vec![3, 6])));
}

define_row! {
	/// Boolean choice alongside an integer log.
	pub row ChoiceLogRow {
		ChooseBrand<ChoiceLogRow, bool>,
		WriterBrand<i32>,
	}
}

/// The writer-under-choice program the ordering zoo drives: a shared prefix
/// tell, then two branches that each tell their own message. Whether the
/// branch messages reach one shared log or die with their branches is what
/// the runner order decides.
fn writer_choice_zoo_program() -> Free<ChoiceLogRow, Vec<bool>> {
	tell(1)
		.bind(|()| {
			choose(tell(2).bind(|()| Free::pure(true)), tell(3).bind(|()| Free::pure(false)))
		})
		.bind(|values: Vec<bool>| Free::pure(values))
}

#[test]
fn writer_inside_choice_drops_branch_logs_with_their_branches() {
	// Branch-local order: each branch folds into its own fork of the log and
	// the fork dies with the branch, so only the prefix tell survives.
	let narrowed: Free<CNilBrand, (i32, Option<Vec<bool>>)> = handle_choose_accum(
		0,
		writer_choice_zoo_program(),
		FoldWriterStep(|sum, message: i32| sum + message),
	);
	assert_eq!(extract(narrowed), (1, Some(vec![true, false])));
}

#[test]
fn writer_outside_choice_folds_one_log_across_branches() {
	// Global order: eliminating choice first sequences both branches' tells
	// into one residual, so the fold outside sees the prefix once and both
	// branch messages.
	let narrowed: Free<TallyRow, Option<Vec<bool>>> = handle_choose(writer_choice_zoo_program());
	let folded: Free<CNilBrand, (i32, Option<Vec<bool>>)> =
		fold_writer(0, |sum, message: i32| sum + message, narrowed);
	assert_eq!(extract(folded), (6, Some(vec![true, false])));
}
