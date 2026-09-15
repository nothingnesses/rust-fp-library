//! Handler names use a tag's final path segment, so qualified tags whose
//! final segments match must use explicit, distinctly named member aliases.

use fp_library::{
	define_row,
	types::effects::{
		state::StateBrand,
		tagged::TaggedBrand,
	},
};

mod first {
	pub struct Slot;
}

mod second {
	pub struct Slot;
}

define_row! {
	/// A row whose qualified labels derive the same handler stem.
	#[handlers]
	pub row CollisionRow {
		TaggedBrand<first::Slot, StateBrand<i32>>,
		TaggedBrand<second::Slot, StateBrand<i32>>,
	}
}

fn main() {}
