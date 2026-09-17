//! The argument-free `multi_shot` marker may occur at most once on an
//! operation so its continuation policy has one unambiguous source.

use fp_library::define_effect;

define_effect! {
	/// An effect with a duplicated operation marker.
	#[handler_state(none)]
	pub effect DuplicateMarker {
		/// Invalid because the marker is repeated.
		#[multi_shot]
		#[multi_shot]
		fn fork() -> bool;
	}
}

fn main() {}
