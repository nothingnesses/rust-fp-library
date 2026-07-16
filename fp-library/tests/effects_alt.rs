//! The first-order `Alt` effect driven end to end through its forking
//! runners, with the scoped-choice ordering zoo as the semantic oracle:
//! where a program composes the same choices and accumulator operations as
//! the scoped `Choose` zoo, the alt vocabulary must reproduce the zoo's
//! pinned outcomes, with the branch-local axis expressed by runner stacking
//! (the multi-shot `handle_accum` under `handle_alt` forks per branch)
//! instead of a dedicated accumulator-forking runner.

#![cfg(feature = "effects")]

use fp_library::{
	brands::{
		CNilBrand,
		RcBrand,
	},
	define_row,
	types::{
		Free,
		effects::{
			alt::{
				AltBrand,
				alt,
				empty,
				handle_alt,
				handle_alt_first,
			},
			handle::multi_shot::{
				extract,
				handle_accum,
			},
			state::{
				StateBrand,
				StateF,
				get,
				put,
			},
		},
	},
};

define_row! {
	/// A one-cell row of first-order choice.
	pub row AltRow {
		AltBrand,
	}
}

define_row! {
	/// First-order choice beside integer state, so the ordering zoo's
	/// state-under-choice programs re-express on this row.
	pub row AltStateRow {
		AltBrand,
		StateBrand<i32>,
	}
}

define_row! {
	/// The residual row after the alt cell is eliminated.
	pub row StateOnlyRow {
		StateBrand<i32>,
	}
}

/// The `State` step shared by the stacked-runner tests: `Get` resumes with
/// the accumulator, `Put` replaces it.
fn state_step<P>(
	s: i32,
	op: StateF<'static, i32, P>,
) -> (i32, P) {
	match op {
		StateF::Get(k) => (s, k(s)),
		StateF::Put(next, k) => (next, k(())),
	}
}

/// The state-under-choice zoo program on the alt row: a shared prefix
/// write, then two branches that each read, advance, and re-read the state
/// (the scoped zoo's shape, with the branch bodies riding the re-callable
/// continuation instead of being owned by the cell).
fn state_alt_zoo_program() -> Free<AltStateRow, i32, RcBrand> {
	put::<i32, AltStateRow, _, RcBrand>(1).bind_multi_shot(|()| {
		alt::<AltStateRow, _, RcBrand>().bind_multi_shot(|left: bool| {
			let advance = if left { 2 } else { 3 };
			get::<i32, AltStateRow, _, RcBrand>().bind_multi_shot(move |seen: i32| {
				put::<i32, AltStateRow, _, RcBrand>(seen + advance)
					.bind_multi_shot(|()| get::<i32, AltStateRow, _, RcBrand>())
			})
		})
	})
}

// Two stacked choices fan out to four leaves in true-first depth-first
// order, each branch re-entering the shared continuations independently.
#[test]
fn forking_collects_every_leaf() {
	let program: Free<AltRow, (bool, bool), RcBrand> =
		alt::<AltRow, _, RcBrand>().bind_multi_shot(|first: bool| {
			alt::<AltRow, _, RcBrand>()
				.bind_multi_shot(move |second: bool| Free::pure((first, second)))
		});
	let collected: Free<CNilBrand, Vec<(bool, bool)>, RcBrand> = handle_alt(program);
	assert_eq!(
		extract(collected),
		vec![(true, true), (true, false), (false, true), (false, false)]
	);
}

// `empty` kills its branch: the dead branch contributes no leaf, and the
// program continues with the surviving branches.
#[test]
fn empty_prunes_a_branch() {
	let program: Free<AltRow, i32, RcBrand> =
		alt::<AltRow, _, RcBrand>().bind_multi_shot(|kept: bool| {
			if kept { empty::<i32, AltRow, _, RcBrand>() } else { Free::pure(7) }
		});
	let collected: Free<CNilBrand, Vec<i32>, RcBrand> = handle_alt(program);
	assert_eq!(extract(collected), vec![7]);
}

// A top-level `empty` with no pending branch ends the program with no
// leaves: the empty collection is the vector analogue of the scoped
// runner's dead top-level branch.
#[test]
fn empty_at_the_top_level_collects_nothing() {
	let program: Free<AltRow, i32, RcBrand> = empty::<i32, AltRow, _, RcBrand>();
	let collected: Free<CNilBrand, Vec<i32>, RcBrand> = handle_alt(program);
	assert_eq!(extract(collected), Vec::<i32>::new());
}

// Global order (the zoo's state-outside-choice case): eliminating choice
// first sequences both branches' state operations into one residual, so
// the right branch reads the left branch's write (3) and advances it to 6;
// the oracle is the scoped zoo's (6, [3, 6]).
#[test]
fn state_outside_alt_threads_one_state_through_branches() {
	let narrowed: Free<StateOnlyRow, Vec<i32>, RcBrand> = handle_alt(state_alt_zoo_program());
	let stated: Free<CNilBrand, (i32, Vec<i32>), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(0, narrowed, state_step);
	assert_eq!(extract(stated), (6, vec![3, 6]));
}

// Branch-local order (the zoo's state-inside-choice case): the state fold
// stacked under the forking runner rides the re-emitted alt cell and is
// cloned into each branch at the fork, so both branches read the prefix
// state (1) and their writes die with the branch; each leaf pairs its own
// branch-final state, the oracle values being the scoped zoo's [3, 4].
#[test]
fn state_inside_alt_forks_per_branch() {
	let folded: Free<AltRow, (i32, i32), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(
			0,
			state_alt_zoo_program(),
			state_step,
		);
	let collected: Free<CNilBrand, Vec<(i32, i32)>, RcBrand> = handle_alt(folded);
	assert_eq!(extract(collected), vec![(3, 3), (4, 4)]);
}

// First success drops the pending branch unrun: none of the right branch's
// state operations reach the residual (the scoped zoo's
// first-success-drops-the-right-branch case, (5, 5)).
#[test]
fn first_success_drops_the_right_branch_unrun() {
	let program: Free<AltStateRow, i32, RcBrand> = alt::<AltStateRow, _, RcBrand>()
		.bind_multi_shot(|left: bool| {
			let value = if left { 5 } else { 9 };
			put::<i32, AltStateRow, _, RcBrand>(value)
				.bind_multi_shot(|()| get::<i32, AltStateRow, _, RcBrand>())
		});
	let narrowed: Free<StateOnlyRow, Option<i32>, RcBrand> = handle_alt_first(program);
	let stated: Free<CNilBrand, (i32, Option<i32>), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(0, narrowed, state_step);
	assert_eq!(extract(stated), (5, Some(5)));
}

// First success falls back to the right branch when the left dies (the
// scoped zoo's fallback case, (9, 9)).
#[test]
fn first_success_falls_back_to_the_right_branch() {
	let program: Free<AltStateRow, i32, RcBrand> = alt::<AltStateRow, _, RcBrand>()
		.bind_multi_shot(|left: bool| {
			if left {
				empty::<i32, AltStateRow, _, RcBrand>()
			} else {
				put::<i32, AltStateRow, _, RcBrand>(9)
					.bind_multi_shot(|()| get::<i32, AltStateRow, _, RcBrand>())
			}
		});
	let narrowed: Free<StateOnlyRow, Option<i32>, RcBrand> = handle_alt_first(program);
	let stated: Free<CNilBrand, (i32, Option<i32>), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(0, narrowed, state_step);
	assert_eq!(extract(stated), (9, Some(9)));
}

// When every branch dies, the first-success runner yields `None` and no
// branch effect reaches the residual.
#[test]
fn first_success_is_none_when_every_branch_dies() {
	let program: Free<AltStateRow, i32, RcBrand> = alt::<AltStateRow, _, RcBrand>()
		.bind_multi_shot(|_: bool| empty::<i32, AltStateRow, _, RcBrand>());
	let narrowed: Free<StateOnlyRow, Option<i32>, RcBrand> = handle_alt_first(program);
	let stated: Free<CNilBrand, (i32, Option<i32>), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(7, narrowed, state_step);
	assert_eq!(extract(stated), (7, None));
}
