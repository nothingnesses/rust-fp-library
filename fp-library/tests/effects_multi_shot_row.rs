//! The multi-shot interpretation surface driven end to end: effects rows
//! over the `Rc` and `Arc` stores run through the public
//! `handle::multi_shot` runners.
//!
//! `Free<Row, A, Store>` composes with any closure store; these tests
//! interpret rows on the multi-shot stores with the same narrowing
//! vocabulary the `Box` store ships: `handle_accum` eliminates one cell
//! and re-emits unmatched layers into the residual row (with the
//! recursive fold deferred into the layer's cell), runners stack, and
//! `extract` closes the fully narrowed pipeline. Programs are constructed
//! by the emitted store-generic constructors, naming the store once at
//! each program's head, and chained with `bind_multi_shot` (each
//! multi-shot store's continuation carries its own closure kind); the
//! interpretation calls are store-generic.

#![cfg(feature = "effects")]

use fp_library::{
	brands::{
		ArcBrand,
		CNilBrand,
		RcBrand,
	},
	define_row,
	types::{
		Free,
		effects::{
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
	/// A one-cell row holding integer state.
	pub row StateRow {
		StateBrand<i32>,
	}
}

define_row! {
	/// A two-cell row holding an integer and a byte state, so narrowing
	/// one cell re-emits the other.
	pub row TwoStateRow {
		StateBrand<i32>,
		StateBrand<u8>,
	}
}

define_row! {
	/// The residual row after the integer cell is eliminated.
	pub row ByteRow {
		StateBrand<u8>,
	}
}

/// The `State` step shared by every test: `Get` resumes with the
/// accumulator, `Put` replaces it.
fn state_step<T: Clone, P>(
	s: T,
	op: StateF<'static, T, P>,
) -> (T, P) {
	match op {
		StateF::Get(k) => (s.clone(), k(s)),
		StateF::Put(next, k) => (next, k(())),
	}
}

// The Rc store threads state through get/put/get: read 20, write 21,
// read back 21.
#[test]
fn rc_state_row_folds_through_the_public_runner() {
	let program: Free<StateRow, i32, RcBrand> = get::<i32, StateRow, _, RcBrand>()
		.bind_multi_shot(|s: i32| put::<i32, StateRow, _, RcBrand>(s + 1))
		.bind_multi_shot(|()| get::<i32, StateRow, _, RcBrand>());
	let folded: Free<CNilBrand, (i32, i32), RcBrand> = handle_accum(20, program, state_step);
	assert_eq!(extract(folded), (21, 21));
}

// The Arc store drives the identical store-generic runner; only the
// construction site differs (its `bind_multi_shot` requires `Send + Sync`
// closures, which these capture-free closures satisfy).
#[test]
fn arc_state_row_folds_through_the_public_runner() {
	let program: Free<StateRow, i32, ArcBrand> = get::<i32, StateRow, _, ArcBrand>()
		.bind_multi_shot(|s: i32| put::<i32, StateRow, _, ArcBrand>(s * 2))
		.bind_multi_shot(|()| get::<i32, StateRow, _, ArcBrand>());
	let folded: Free<CNilBrand, (i32, i32), ArcBrand> = handle_accum(4, program, state_step);
	assert_eq!(extract(folded), (8, 8));
}

// Stacked runners over a two-cell row: eliminating the integer cell
// re-emits the byte cell into the residual row (the deferred-recursion
// path), and the second runner picks it up.
#[test]
fn rc_stacked_runners_re_emit_the_unmatched_cell() {
	let program: Free<TwoStateRow, u8, RcBrand> = get::<i32, TwoStateRow, _, RcBrand>()
		.bind_multi_shot(|n: i32| put::<u8, TwoStateRow, _, RcBrand>((n + 1) as u8))
		.bind_multi_shot(|()| get::<u8, TwoStateRow, _, RcBrand>());
	let after_int: Free<ByteRow, (i32, u8), RcBrand> =
		handle_accum::<StateBrand<i32>, _, _, _, _, _, _, _>(41, program, state_step);
	let after_byte: Free<CNilBrand, (u8, (i32, u8)), RcBrand> =
		handle_accum::<StateBrand<u8>, _, _, _, _, _, _, _>(0, after_int, state_step);
	assert_eq!(extract(after_byte), (42, (41, 42)));
}

// A longer chain exercises the queue across many suspensions: ten
// increments accumulate iteratively.
#[test]
fn rc_state_row_folds_a_chained_program() {
	let mut program: Free<StateRow, (), RcBrand> = put::<i32, StateRow, _, RcBrand>(0);
	for _ in 0 .. 10 {
		program = program
			.bind_multi_shot(|()| get::<i32, StateRow, _, RcBrand>())
			.bind_multi_shot(|s: i32| put::<i32, StateRow, _, RcBrand>(s + 1));
	}
	let folded: Free<CNilBrand, (i32, ()), RcBrand> = handle_accum(99, program, state_step);
	assert_eq!(extract(folded), (10, ()));
}
