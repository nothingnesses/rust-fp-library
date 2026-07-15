//! An effects row interpreted on the multi-shot stores.
//!
//! `Free<Row, A, Store>` composes with any closure store, but until now
//! only the `Box` store had an interpretation path. These tests drive a
//! one-cell `State` row over the `Rc` and `Arc` stores with a single
//! store-generic fold: the multi-shot `to_view` steps the spine, the
//! coproduct `uninject` selects the cell, and the Box-backed `Coyoneda`
//! cell lowers exactly once per encounter, so the cell's one-shot
//! `FnOnce` continuation is compatible with a non-forking interpreter on
//! every store. Programs are constructed per store (each store's `bind`
//! carries its own closure kind); only interpretation is shared.

#![cfg(feature = "effects")]

use fp_library::{
	brands::{
		ArcBrand,
		RcBrand,
	},
	define_row,
	types::{
		Coyoneda,
		Free,
		FreeStep,
		cat_queue::CatQueue,
		closure_storage::{
			ClosureStorage,
			MultiShotStore,
			ValueFor,
		},
		effects::{
			coproduct::CoprodInjector,
			state::{
				StateBrand,
				StateF,
			},
		},
		free::Continuation,
	},
};

define_row! {
	/// A one-cell row holding integer state.
	pub row StateRow {
		StateBrand<i32>,
	}
}

/// Store-generic `get`: injects the cell by hand because the emitted
/// constructors return the `Box`-store `Free` default.
fn get_row<S>() -> Free<StateRow, i32, S>
where
	S: ClosureStorage,
	i32: ValueFor<S>, {
	let coyo: Coyoneda<'static, StateBrand<i32>, i32> =
		Coyoneda::lift(StateF::Get(Box::new(|s| s)));
	Free::lift_f(CoprodInjector::inject(coyo))
}

/// Store-generic `put`, by the same hand-injected route.
fn put_row<S>(next: i32) -> Free<StateRow, (), S>
where
	S: ClosureStorage,
	(): ValueFor<S>, {
	let coyo: Coyoneda<'static, StateBrand<i32>, ()> =
		Coyoneda::lift(StateF::Put(next, Box::new(|()| ())));
	Free::lift_f(CoprodInjector::inject(coyo))
}

/// One generic interpreter body over both multi-shot stores: threads the
/// state through the fold, consuming each cell exactly once.
fn run_state<S, A>(
	initial: i32,
	program: Free<StateRow, A, S>,
) -> (i32, A)
where
	S: MultiShotStore,
	A: ValueFor<S>,
	<S as ClosureStorage>::Queue<Continuation<StateRow, S>>:
		CatQueue<Continuation<StateRow, S>> + Clone, {
	let mut state = initial;
	let mut program = program;
	loop {
		match program.to_view() {
			FreeStep::Done(result) => return (state, result),
			FreeStep::Suspended(layer) => {
				let cell: Coyoneda<'static, StateBrand<i32>, Free<StateRow, A, S>> =
					match layer.uninject() {
						Ok(cell) => cell,
						Err(rest) => match rest {},
					};
				match cell.lower() {
					StateF::Get(k) => program = k(state),
					StateF::Put(next, k) => {
						state = next;
						program = k(());
					}
				}
			}
		}
	}
}

// The Rc store threads state through get/put/get: read 20, write 21,
// read back 21.
#[test]
fn rc_state_row_threads_through_the_fold() {
	let program: Free<StateRow, i32, RcBrand> = get_row::<RcBrand>()
		.bind(|s: i32| put_row::<RcBrand>(s + 1))
		.bind(|()| get_row::<RcBrand>());
	assert_eq!(run_state(20, program), (21, 21));
}

// The Arc store drives the identical generic interpreter body; only the
// construction site differs (its `bind` requires `Send + Sync` closures,
// which these capture-free closures satisfy).
#[test]
fn arc_state_row_drives_the_same_generic_body() {
	let program: Free<StateRow, i32, ArcBrand> = get_row::<ArcBrand>()
		.bind(|s: i32| put_row::<ArcBrand>(s * 2))
		.bind(|()| get_row::<ArcBrand>());
	assert_eq!(run_state(4, program), (8, 8));
}

// A longer chain exercises the queue across many suspensions: ten
// increments accumulate iteratively.
#[test]
fn rc_state_row_folds_a_chained_program() {
	let mut program: Free<StateRow, (), RcBrand> = put_row::<RcBrand>(0);
	for _ in 0 .. 10 {
		program = program.bind(|()| get_row::<RcBrand>()).bind(|s: i32| put_row::<RcBrand>(s + 1));
	}
	assert_eq!(run_state(99, program), (10, ()));
}
