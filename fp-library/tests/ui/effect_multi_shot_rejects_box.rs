//! A constructor marked `multi_shot` returns the Rc-store `Free` form. It
//! cannot construct a Box-store program whose single-use mapping closure would
//! panic when a handler resumes the operation's continuation twice.

use fp_library::{
	brands::BoxBrand,
	define_effect,
	define_row,
	types::Free,
};

define_effect! {
	/// A binary fork with a re-callable continuation.
	#[handler_state(none)]
	pub effect Fork {
		/// Select a branch under a forking interpreter.
		#[multi_shot]
		fn fork() -> bool;
	}
}

define_row! {
	/// A row containing the fork effect.
	pub row ForkRow {
		ForkBrand,
	}
}

fn main() {
	let _: Free<ForkRow, bool, BoxBrand> = fork::<ForkRow, _>();
}
