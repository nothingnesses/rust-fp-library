//! Forking interpretation on the multi-shot stores, driving the
//! `#[multi_shot]` cell `define_effect!` emits.
//!
//! A `#[multi_shot]` operation's continuation is emitted in the `Rc`
//! store's re-callable form (`Rc<dyn Fn>` instead of `Box<dyn FnOnce>`),
//! so a forking runner may call one captured continuation once per
//! branch. The `Free` spine cooperates because the multi-shot `to_view`
//! clones the continuation queue into the layer's mapping closure, so
//! each re-entry carries its own queue.
//!
//! The mixed-row case pins the adopted re-entry semantics for threaded
//! accumulators: a narrowing `handle_accum` fold re-emitted into a
//! `Choose` cell is cloned into each branch at the fork, so every branch
//! continues from the accumulator as of capture (fork-the-accumulator,
//! not shared mutation).

#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::RcBrand,
		classes::{
			Functor,
			WrapDrop,
		},
		define_effect,
		define_row,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			FreeStep,
			effects::{
				coproduct::CoprodInjector,
				handle::multi_shot::handle_accum,
				state::{
					StateBrand,
					StateF,
				},
			},
		},
	},
	std::rc::Rc,
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

/// Injects a `choose` operation over the `Rc` store by hand (the emitted
/// `choose` constructor returns the `Box`-store `Free` default).
fn choose_rc<Row, I>() -> Free<Row, bool, RcBrand>
where
	Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
	<Row as LifetimeUnaryKind>::Of<'static, bool>:
		CoprodInjector<Coyoneda<'static, ChooseBrand, bool>, I>, {
	let coyo: Coyoneda<'static, ChooseBrand, bool> =
		Coyoneda::lift(ChooseF::Choose(Rc::new(|b| b)));
	Free::lift_f(CoprodInjector::inject(coyo))
}

/// Injects a `get` into the mixed row.
fn get() -> Free<ChooseStateRow, i32, RcBrand> {
	let coyo: Coyoneda<'static, StateBrand<i32>, i32> =
		Coyoneda::lift(StateF::Get(Box::new(|s| s)));
	Free::lift_f(CoprodInjector::inject(coyo))
}

/// Injects a `put` into the mixed row.
fn put(next: i32) -> Free<ChooseStateRow, (), RcBrand> {
	let coyo: Coyoneda<'static, StateBrand<i32>, ()> =
		Coyoneda::lift(StateF::Put(next, Box::new(|()| ())));
	Free::lift_f(CoprodInjector::inject(coyo))
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
	let program: Free<ChooseRow, (bool, bool), RcBrand> =
		choose_rc::<ChooseRow, _>().bind(|first: bool| {
			choose_rc::<ChooseRow, _>().bind(move |second: bool| Free::pure((first, second)))
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
	let program: Free<ChooseStateRow, i32, RcBrand> = get().bind(|start: i32| {
		choose_rc::<ChooseStateRow, _>()
			.bind(move |branch: bool| put(start + if branch { 1 } else { 2 }).bind(|()| get()))
	});
	let narrowed: Free<ChooseRow, (i32, i32), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(10, program, |s, op| match op {
			StateF::Get(k) => (s, k(s)),
			StateF::Put(next, k) => (next, k(())),
		});
	assert_eq!(run_choose_all(narrowed), vec![(11, 11), (12, 12)]);
}
