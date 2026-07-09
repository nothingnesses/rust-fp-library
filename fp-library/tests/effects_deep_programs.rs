//! Deep-program stack-safety over the effects public surface.
//!
//! These tests define their own effects and rows with the public
//! `define_effect!`/`define_row!` macros and interpret them with the
//! hand-written dispatch-loop shape the custom-effects guide teaches: an
//! iterative `Free::resume` loop that recurses only to elaborate a
//! higher-order cell. They pin the substrate's depth behaviour in the form
//! user code takes:
//!
//! - deep `bind` chains, left- and right-associated, on the order of 100k
//!   steps;
//! - a deep row-widening walk and a deep rewriting (interpose-shaped) walk,
//!   both lazy: the recursive step is deferred into each continuation via the
//!   row's `Coyoneda`-backed `Functor`, so native stack use per step is
//!   constant;
//! - a deep program under one elaborated catch (which also exercises the
//!   continuation queue's rotation at depth);
//! - the elaboration recursion contract: native stack grows with catch
//!   NESTING depth, not with program length, so a full-depth program under
//!   bounded nesting passes.
#![cfg(feature = "effects")]

use {
	fp_library::{
		Apply,
		classes::Functor,
		define_effect,
		define_row,
		// The `Apply!`/`Kind!` type annotations resolve the macro-generated
		// kind traits by their internal names, so the `kinds` glob must be in
		// scope (the effect macros wrap this glob for their own emissions).
		kinds::*,
		types::{
			Coyoneda,
			Free,
		},
	},
	std::{
		cell::Cell,
		rc::Rc,
	},
};

define_effect! {
	/// A unit step: each `tick` advances a counter the interpreter owns.
	#[handler_state(shared_by_reference)]
	pub effect Tick {
		/// Advance the counter, resuming with unit.
		fn tick() -> ();
	}
}

define_effect! {
	/// Throw with a unit error: the program aborts and carries no continuation.
	#[handler_state(none)]
	pub effect Throw {
		/// Abort the current program with a bare throw.
		fn throw() -> !;
	}
}

define_effect! {
	/// Catch owns an action sub-program and a recovery thunk; the interpreter
	/// elaborates it by interpreting the action recursively, so native stack
	/// grows with catch nesting depth (the contract the nesting test pins).
	#[handler_state(none)]
	pub effect Catch<RAction: 'static> {
		/// Run `action`, recovering a bare throw with `recover`.
		fn catch(action: Program<RAction>, recover: impl FnOnce() -> Program<RAction>) -> RAction;
	}
}

define_row! {
	/// The full row. The catch cell stores `Free<AppRow, i32>` sub-programs,
	/// the self-reference the nominal row brand makes legal.
	pub row AppRow {
		TickBrand,
		ThrowBrand,
		CatchBrand<AppRow, i32>,
	}
}

define_row! {
	/// A narrow row (`Tick` alone) for the widening walk.
	pub row NarrowRow {
		TickBrand,
	}
}

/// The catch cell pinned at the row (the parameterise-and-pin convention the
/// built-in catalog uses); the alias also keeps the interpreter's `uninject`
/// annotation under the type-complexity lint's threshold.
type CatchPinned = CatchBrand<AppRow, i32>;

/// The depth every "deep" case drives; the same order of magnitude as the
/// crate's other stack-safety suites.
const DEPTH: usize = 100_000;

/// The guide-shaped interpreter: an iterative `resume` loop dispatching by
/// brand, recursing only to elaborate `catch` (run the action; on a bare
/// throw, run the recovery; thread the value to the continuation).
fn run_app(
	mut program: Free<AppRow, i32>,
	ticks: &Cell<u64>,
) -> Result<i32, ()> {
	loop {
		let layer = match program.resume() {
			Ok(value) => return Ok(value),
			Err(layer) => layer,
		};
		let layer = {
			let selected: Result<Coyoneda<'static, TickBrand, Free<AppRow, i32>>, _> =
				layer.uninject();
			match selected {
				Ok(op) => {
					match op.lower() {
						TickF::Tick(resume) => {
							ticks.set(ticks.get() + 1);
							program = resume(());
						}
					}
					continue;
				}
				Err(rest) => rest,
			}
		};
		let layer = {
			let selected: Result<Coyoneda<'static, ThrowBrand, Free<AppRow, i32>>, _> =
				layer.uninject();
			match selected {
				Ok(op) => match op.lower() {
					ThrowF::Throw(_) => return Err(()),
				},
				Err(rest) => rest,
			}
		};
		let selected: Result<Coyoneda<'static, CatchPinned, Free<AppRow, i32>>, _> =
			layer.uninject();
		match selected {
			Ok(op) => match op.lower() {
				CatchF::Catch {
					action,
					recover,
					k,
				} => {
					let value = match run_app(action, ticks) {
						Ok(value) => value,
						Err(()) => run_app(recover(), ticks)?,
					};
					program = k(value);
				}
			},
			Err(terminal) => match terminal {},
		}
	}
}

/// A left-associated chain of `n` ticks ending in `pure(result)`.
fn tick_chain(
	n: usize,
	result: i32,
) -> Free<AppRow, i32> {
	let mut program: Free<AppRow, ()> = tick();
	for _ in 1 .. n {
		program = program.bind(|()| tick());
	}
	program.bind(move |()| Free::pure(result))
}

#[test]
fn deep_left_associated_bind_chain_runs_without_overflow() {
	let ticks = Cell::new(0);
	assert_eq!(run_app(tick_chain(DEPTH, 7), &ticks), Ok(7));
	assert_eq!(ticks.get(), DEPTH as u64);
}

#[test]
fn deep_right_associated_bind_chain_runs_without_overflow() {
	// Built back to front, so each step nests the rest of the program inside
	// its continuation closure.
	let mut program: Free<AppRow, i32> = Free::pure(7);
	for _ in 0 .. DEPTH {
		let rest = program;
		program = tick().bind(move |()| rest);
	}
	let ticks = Cell::new(0);
	assert_eq!(run_app(program, &ticks), Ok(7));
	assert_eq!(ticks.get(), DEPTH as u64);
}

/// Widen a narrow-row program into the full row, lazily: each step embeds one
/// layer and defers the widening of the continuation into the layer's
/// `Coyoneda` map, so native stack use per interpreted step is constant.
fn widen(program: Free<NarrowRow, i32>) -> Free<AppRow, i32> {
	match program.resume() {
		Ok(value) => Free::pure(value),
		Err(layer) => {
			let mapped = <NarrowRow as Functor>::map(widen, layer);
			let widened: Apply!(<AppRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<AppRow, i32>>) =
				mapped.embed();
			Free::wrap(widened)
		}
	}
}

#[test]
fn deep_row_widening_walk_runs_without_overflow() {
	let mut narrow: Free<NarrowRow, ()> = tick();
	for _ in 1 .. DEPTH {
		narrow = narrow.bind(|()| tick());
	}
	let narrow = narrow.bind(|()| Free::pure(7));
	let ticks = Cell::new(0);
	assert_eq!(run_app(widen(narrow), &ticks), Ok(7));
	assert_eq!(ticks.get(), DEPTH as u64);
}

/// An interpose-shaped rewriting walk: replace each matched `Tick` dispatch
/// with an equivalent one (counting the visit), re-embed every unmatched
/// layer, and defer the walk of every continuation into the layer's
/// `Coyoneda` map, so the walk is lazy and constant-stack per step.
fn walk(
	program: Free<AppRow, i32>,
	visited: Rc<Cell<u64>>,
) -> Free<AppRow, i32> {
	match program.resume() {
		Ok(value) => Free::pure(value),
		Err(layer) => {
			let deferred = visited.clone();
			let layer = <AppRow as Functor>::map(
				move |continuation| walk(continuation, deferred.clone()),
				layer,
			);
			let selected: Result<Coyoneda<'static, TickBrand, Free<AppRow, i32>>, _> =
				layer.uninject();
			match selected {
				Ok(op) => match op.lower() {
					TickF::Tick(k) => {
						visited.set(visited.get() + 1);
						tick().bind(move |()| k(()))
					}
				},
				Err(rest) => {
					let reembedded: Apply!(<AppRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<AppRow, i32>>) =
						rest.embed();
					Free::wrap(reembedded)
				}
			}
		}
	}
}

#[test]
fn deep_rewriting_walk_runs_without_overflow() {
	let visited = Rc::new(Cell::new(0));
	let walked = walk(tick_chain(DEPTH, 7), visited.clone());
	let ticks = Cell::new(0);
	assert_eq!(run_app(walked, &ticks), Ok(7));
	assert_eq!(visited.get(), DEPTH as u64);
	assert_eq!(ticks.get(), DEPTH as u64);
}

#[test]
fn deep_program_under_one_catch_runs_without_overflow() {
	let program = catch(tick_chain(DEPTH, 7), || Free::pure(-1));
	let ticks = Cell::new(0);
	assert_eq!(run_app(program, &ticks), Ok(7));
	assert_eq!(ticks.get(), DEPTH as u64);
}

#[test]
fn deep_aborting_action_recovers_and_keeps_its_steps() {
	// The counter is shared handler state: the deep action's ticks survive
	// the abort and the recovery, and the recovery's value is the result.
	let action = tick_chain(DEPTH, 0).bind(|_| throw::<i32, _, _>());
	let program = catch(action, || Free::pure(-1));
	let ticks = Cell::new(0);
	assert_eq!(run_app(program, &ticks), Ok(-1));
	assert_eq!(ticks.get(), DEPTH as u64);
}

#[test]
fn native_stack_grows_with_nesting_depth_not_program_length() {
	// The elaboration recursion contract: a full-depth program under bounded
	// catch nesting passes, because interpretation recurses once per nesting
	// level while the program's own length is driven iteratively.
	const NESTING: usize = 100;
	let mut program = tick_chain(DEPTH, 7);
	for _ in 0 .. NESTING {
		program = catch(program, || Free::pure(-1));
	}
	let ticks = Cell::new(0);
	assert_eq!(run_app(program, &ticks), Ok(7));
	assert_eq!(ticks.get(), DEPTH as u64);
}
