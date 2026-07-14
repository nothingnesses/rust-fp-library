//! The transactional state combinator, driven end to end: `transact_state`
//! snapshots the outer state, interprets the action's `State` operations
//! against a local accumulator (re-emitting every other effect unchanged),
//! and commits the final local value with one `put` after the action
//! completes. Rollback on abort is structural: the abort discards the
//! continuation carrying the commit `put`, so the outer state never sees an
//! aborted action's writes.
//!
//! Pinned here: (1) the same-row re-embedding typechecks (the combinator's
//! local interpretation re-emits the remainder back into the row it came
//! from); (2) success commits and abort rolls back under the one-pass
//! handler surface (shared-cell arms); (3) success commits and branch death
//! rolls back under stacked narrowing runners (`handle_choose` inside,
//! `handle_state` outside); (4) a deep action drives iteratively.
#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::CNilBrand,
		define_effect,
		define_row,
		types::{
			Free,
			effects::{
				choose::{
					ChooseBrand,
					choose,
					empty,
					handle_choose,
				},
				handle::{
					RowHandler,
					extract,
				},
				state::{
					StateArms,
					StateBrand,
					get,
					handle_state,
					put,
					transact_state,
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

// -- The one-pass handler surface family --

define_row! {
	/// Integer state alongside a payload-carrying abort.
	#[handlers]
	pub row TxRow {
		StateBrand<i32>,
		FailBrand,
	}
}

/// Builds the handler set over a borrowed shared state cell.
fn tx_handlers(state: &Cell<i32>) -> TxRowHandlers<'_> {
	TxRowHandlers {
		state: StateArms {
			get: Box::new(|| state.get()),
			put: Box::new(|value| state.set(value)),
		},
		fail: FailArms,
	}
}

#[test]
fn success_commits_the_final_local_state_under_the_one_pass_surface() {
	// The action's own writes are visible to its own reads (the local
	// accumulator), and the final local value reaches the shared cell.
	let state = Cell::new(1);
	let handlers = tx_handlers(&state);
	let program: Free<TxRow, i32> =
		transact_state::<_, i32, _, _, _, _, _>(put(10).bind(|()| get()));
	assert_eq!(handlers.handle(program).ok(), Some(10));
	assert_eq!(state.get(), 10);
}

#[test]
fn the_snapshot_seeds_the_local_state_from_the_outer_state() {
	// A read inside the transaction, before any local write, sees the outer
	// state as it was at entry.
	let state = Cell::new(0);
	let handlers = tx_handlers(&state);
	let program: Free<TxRow, i32> =
		put(3).bind(|()| transact_state::<_, i32, _, _, _, _, _>(get()));
	assert_eq!(handlers.handle(program).ok(), Some(3));
	assert_eq!(state.get(), 3);
}

#[test]
fn abort_rolls_back_the_transaction_under_the_one_pass_surface() {
	// The action writes locally and then aborts: the abort re-emits and ends
	// interpretation, the commit continuation is discarded, and the shared
	// cell still holds the pre-transaction value.
	let state = Cell::new(1);
	let handlers = tx_handlers(&state);
	let program: Free<TxRow, i32> =
		transact_state::<_, i32, _, _, _, _, _>(put(10).bind(|()| fail::<i32, _, _>("boom")));
	assert!(matches!(handlers.handle(program), Err(TxRowAbort::Fail(FailAbort::Fail("boom")))));
	assert_eq!(state.get(), 1);
}

#[test]
fn writes_before_the_transaction_survive_its_abort() {
	// Only the transaction is rolled back; state written before it stands.
	let state = Cell::new(0);
	let handlers = tx_handlers(&state);
	let program: Free<TxRow, i32> = put(7).bind(|()| {
		transact_state::<_, i32, _, _, _, _, _>(put(10).bind(|()| fail::<i32, _, _>("boom")))
	});
	assert!(matches!(handlers.handle(program), Err(TxRowAbort::Fail(FailAbort::Fail("boom")))));
	assert_eq!(state.get(), 7);
}

// -- The stacked narrowing-runner family --

define_row! {
	/// Integer state alongside integer scoped choice.
	pub row TxChoiceRow {
		StateBrand<i32>,
		ChooseBrand<TxChoiceRow, i32>,
	}
}

define_row! {
	/// The residual after choice is eliminated from the choice-state row.
	pub row TxStateOnlyRow {
		StateBrand<i32>,
	}
}

#[test]
fn success_commits_under_stacked_narrowing_runners() {
	// One threaded accumulator outside: the transaction's commit becomes an
	// ordinary `put` against it.
	let program: Free<TxChoiceRow, i32> =
		transact_state::<_, i32, _, _, _, _, _>(put(10).bind(|()| get()));
	let no_choice: Free<TxStateOnlyRow, Option<i32>> = handle_choose(program);
	let narrowed: Free<CNilBrand, (i32, Option<i32>)> = handle_state(1, no_choice);
	assert_eq!(extract(narrowed), (10, Some(10)));
}

#[test]
fn branch_death_rolls_back_the_transaction_under_stacked_narrowing_runners() {
	// The left branch transacts a write and then dies: the commit never
	// runs, so the globally threaded state is untouched and the surviving
	// right branch reads the pre-transaction value.
	let program: Free<TxChoiceRow, Vec<i32>> = put(1).bind(|()| {
		choose(
			transact_state::<_, i32, _, _, _, _, _>(put(10).bind(|()| empty::<_, i32, _, _>())),
			get(),
		)
	});
	let no_choice: Free<TxStateOnlyRow, Option<Vec<i32>>> = handle_choose(program);
	let narrowed: Free<CNilBrand, (i32, Option<Vec<i32>>)> = handle_state(0, no_choice);
	assert_eq!(extract(narrowed), (1, Some(vec![1])));
}

#[test]
fn a_deep_transacted_action_drives_iteratively() {
	// 100k local writes inside one transaction: the local interpretation is
	// the accumulator runner's iterative loop, so depth costs no native
	// stack.
	const DEPTH: usize = 100_000;
	let state = Cell::new(0);
	let handlers = tx_handlers(&state);
	let mut action: Free<TxRow, i32> = put(0).bind(|()| get());
	for value in 1 .. DEPTH {
		action = put(value as i32).bind(move |()| action);
	}
	let program: Free<TxRow, i32> = transact_state::<_, i32, _, _, _, _, _>(action);
	assert_eq!(handlers.handle(program).ok(), Some(0));
	assert_eq!(state.get(), 0);
}
