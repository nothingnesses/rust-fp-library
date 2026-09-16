//! The `multi_shot` operation marker is argument-free; accepting payloads
//! would silently discard misspelled or unsupported configuration.

use fp_library::define_effect;

define_effect! {
	/// An effect with a malformed multi-shot marker.
	#[handler_state(none)]
	pub effect Malformed {
		/// Attempt to fork.
		#[multi_shot(unexpected)]
		fn fork() -> bool;
	}
}

fn main() {}
