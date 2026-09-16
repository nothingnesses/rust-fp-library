//! A handler literal naming a cell the row does not have is a compile
//! error: the handler list is a named-field struct, so a misspelled or
//! missing cell name cannot construct.
#![cfg(feature = "effects")]

use fp_library::{
	define_effect,
	define_row,
	types::effects::state::{
		StateArms,
		StateBrand,
	},
};

define_effect! {
	/// A unit step.
	#[handler_state(shared_by_reference)]
	pub effect Tick {
		/// Advance, resuming with unit.
		fn tick() -> ();
	}
}

define_row! {
	/// Integer state alongside ticking.
	#[handlers]
	pub row AppRow {
		StateBrand<i32>,
		TickBrand,
	}
}

fn main() {
	let state = std::cell::Cell::new(0);
	let _handlers = AppRowHandlers {
		state: StateArms {
			get: Box::new(|| state.get()),
			put: Box::new(|value| state.set(value)),
		},
		tick_counter: TickArms {
			tick: Box::new(|| ()),
		},
	};
}
