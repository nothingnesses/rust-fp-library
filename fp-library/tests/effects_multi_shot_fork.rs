//! Forking interpretation on the multi-shot stores, driving the
//! `#[multi_shot]` cell and the store-generic constructors
//! `define_effect!` emits.
//!
//! A `#[multi_shot]` operation's continuation is emitted in the `Rc`
//! store's re-callable form (`Rc<dyn Fn>` instead of `Box<dyn FnOnce>`),
//! so a forking runner may call one captured continuation once per
//! branch. The `Free` spine cooperates because the multi-shot `to_view`
//! clones the continuation queue into the layer's mapping closure, so
//! each re-entry carries its own queue. Every program here is built from
//! the emitted constructors with the store named once at the program's
//! head (`choose::<Row, _, RcBrand>()`), the store-generic emission's
//! end-to-end evidence.
//!
//! The mixed-row case pins the adopted re-entry semantics for threaded
//! accumulators: a narrowing `handle_accum` fold re-emitted into a
//! `Choose` cell is cloned into each branch at the fork, so every branch
//! continues from the accumulator as of capture (fork-the-accumulator,
//! not shared mutation).

#![cfg(feature = "effects")]

use fp_library::{
	brands::RcBrand,
	define_effect,
	define_row,
	types::{
		Coyoneda,
		Free,
		FreeStep,
		effects::{
			handle::multi_shot::handle_accum,
			state::{
				StateBrand,
				StateF,
				get,
				put,
			},
		},
	},
};

define_effect! {
	/// Binary nondeterministic choice; the continuation is re-callable,
	/// once per branch.
	#[handler_state(none)]
	pub effect Choose {
		/// Chooses one branch.
		#[multi_shot]
		fn choose() -> bool;
	}
}

define_row! {
	/// A one-cell row of binary choice.
	pub row ChooseRow {
		ChooseBrand,
	}
}

define_row! {
	/// Choice beside integer state, so a state fold re-emits into the
	/// choice cell and forks per branch.
	pub row ChooseStateRow {
		ChooseBrand,
		StateBrand<i32>,
	}
}

/// The forking runner: lowers each `Choose` cell once, then calls the
/// composed continuation once per branch (true first), collecting every
/// leaf depth-first.
fn run_choose_all<A: Clone + 'static>(program: Free<ChooseRow, A, RcBrand>) -> Vec<A> {
	match program.to_view() {
		FreeStep::Done(leaf) => vec![leaf],
		FreeStep::Suspended(layer) => {
			let cell: Coyoneda<'static, ChooseBrand, Free<ChooseRow, A, RcBrand>> =
				match layer.uninject() {
					Ok(cell) => cell,
					Err(rest) => match rest {},
				};
			let ChooseF::Choose(k) = cell.lower();
			// The multi-shot moment: one captured continuation, two calls.
			let mut leaves = run_choose_all(k(true));
			leaves.extend(run_choose_all(k(false)));
			leaves
		}
	}
}

// Two stacked choices fan out to four leaves in depth-first order, each
// branch re-entering the shared continuations independently.
#[test]
fn forking_runner_collects_every_branch() {
	let program: Free<ChooseRow, (bool, bool), RcBrand> = choose::<ChooseRow, _, RcBrand>()
		.bind_multi_shot(|first: bool| {
			choose::<ChooseRow, _, RcBrand>()
				.bind_multi_shot(move |second: bool| Free::pure((first, second)))
		});
	assert_eq!(
		run_choose_all(program),
		vec![(true, true), (true, false), (false, true), (false, false)]
	);
}

// A state fold narrowed out of the mixed row rides the re-emitted choice
// cell; forking clones the fold, so each branch continues from the
// accumulator as of capture and the branches do not observe each other's
// writes.
#[test]
fn forked_state_fold_is_local_per_branch() {
	let program: Free<ChooseStateRow, i32, RcBrand> = get::<i32, ChooseStateRow, _, RcBrand>()
		.bind_multi_shot(|start: i32| {
			choose::<ChooseStateRow, _, RcBrand>().bind_multi_shot(move |branch: bool| {
				put::<i32, ChooseStateRow, _, RcBrand>(start + if branch { 1 } else { 2 })
					.bind_multi_shot(|()| get::<i32, ChooseStateRow, _, RcBrand>())
			})
		});
	let narrowed: Free<ChooseRow, (i32, i32), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(10, program, |s, op| match op {
			StateF::Get(k) => (s, k(s)),
			StateF::Put(next, k) => (next, k(())),
		});
	assert_eq!(run_choose_all(narrowed), vec![(11, 11), (12, 12)]);
}
