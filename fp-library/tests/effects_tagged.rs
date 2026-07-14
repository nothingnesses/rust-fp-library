//! Tagged (labelled) effects, driven end to end: `TaggedBrand<Label,
//! EBrand>`'s identity changes the dispatch key while its kind projection
//! reuses the effect's own operations enum, so two same-type effects
//! coexist in one row, selected by label.
//!
//! Pinned here: (1) the emitted `<name>_at` constructors inject at the
//! tagged brand, so both tagged `State` cells dispatch independently
//! through the generic runner; (2) a `#[handlers]` row derives one field
//! per tagged cell from the label joined to the effect's stem, so one
//! handler list serves two same-type cells; (3) a step written for the
//! bare effect serves each labelled cell through the `tag_step` adapter,
//! leaving the bare step's own inference untouched.
#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::CNilBrand,
		define_row,
		types::{
			Free,
			effects::{
				handle::{
					AccumStep,
					RowHandler,
					extract,
					handle_accum,
				},
				state::{
					StateArms,
					StateBrand,
					StateStep,
					get_at,
					put_at,
				},
				tagged::{
					TaggedBrand,
					tag_step,
				},
			},
		},
	},
	std::cell::Cell,
};

/// The first label.
pub struct Fst;

/// The second label.
pub struct Snd;

define_row! {
	/// Two integer states, distinguished by label alone.
	#[handlers]
	pub row TwoStateRow {
		TaggedBrand<Fst, StateBrand<i32>>,
		TaggedBrand<Snd, StateBrand<i32>>,
	}
}

define_row! {
	/// The residual after the first label is eliminated.
	pub row SndOnlyRow {
		TaggedBrand<Snd, StateBrand<i32>>,
	}
}

/// Writes the first cell and reads both: the labelled constructors pick
/// each cell by label, so the result encodes which cell served each read.
fn write_fst_read_both() -> Free<TwoStateRow, i32> {
	put_at::<Fst, i32, _, _>(10).bind(|()| {
		get_at::<Fst, i32, _, _>()
			.bind(|x: i32| get_at::<Snd, i32, _, _>().bind(move |y: i32| Free::pure(x * 100 + y)))
	})
}

#[test]
fn two_tagged_state_cells_dispatch_independently_by_label() {
	let program = write_fst_read_both();
	// Each label eliminates with the bare effect's step through the
	// `tag_step` adapter; the adapter picks the brand, so the bare step's
	// own call sites stay untouched.
	let fst_step = tag_step::<Fst, _>(StateStep);
	let fst_handled: Free<SndOnlyRow, (i32, i32)> =
		handle_accum::<TaggedBrand<Fst, StateBrand<i32>>, _, _, _, _, _, _>(
			1,
			program,
			move |s, op| fst_step.step(s, op),
		);
	let snd_step = tag_step::<Snd, _>(StateStep);
	let snd_handled: Free<CNilBrand, (i32, (i32, i32))> =
		handle_accum::<TaggedBrand<Snd, StateBrand<i32>>, _, _, _, _, _, _>(
			2,
			fst_handled,
			move |s, op| snd_step.step(s, op),
		);
	let (snd_final, (fst_final, result)) = extract(snd_handled);
	assert_eq!(result, 1002);
	assert_eq!(fst_final, 10);
	assert_eq!(snd_final, 2);
}

#[test]
fn a_two_states_handler_row_serves_each_label_with_its_own_arms() {
	// The handler list's field names derive from the labels joined to the
	// effect stem (`fst_state`, `snd_state`), so both same-type cells get
	// their own arms and the writes land in the right cell.
	let fst = Cell::new(1);
	let snd = Cell::new(2);
	let handlers = TwoStateRowHandlers {
		fst_state: StateArms {
			get: Box::new(|| fst.get()),
			put: Box::new(|value| fst.set(value)),
		},
		snd_state: StateArms {
			get: Box::new(|| snd.get()),
			put: Box::new(|value| snd.set(value)),
		},
	};
	assert_eq!(handlers.handle(write_fst_read_both()).ok(), Some(1002));
	assert_eq!(fst.get(), 10);
	assert_eq!(snd.get(), 2);
}
