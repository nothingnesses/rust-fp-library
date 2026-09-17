//! A `#[handlers]` row cannot hold a `#[multi_shot]` effect's cell: the
//! one-pass handler surface is single-shot by construction (an arm resumes
//! exactly once), so `define_effect!` withholds the `HandlerPieces` impl
//! from any effect with a re-callable operation. Such effects interpret
//! through the narrowing runners and a forking fold instead.
#![cfg(feature = "effects")]

use fp_library::{
	define_effect,
	define_row,
	types::effects::state::StateBrand,
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
	/// Integer state alongside the re-callable choice cell.
	#[handlers]
	pub row AppRow {
		StateBrand<i32>,
		ChooseBrand,
	}
}

fn main() {}
