//! Effect type-parameter defaults are rejected at the macro boundary because
//! generated impl and constructor positions cannot legally preserve them.

use fp_library::define_effect;

define_effect! {
	/// An invalid effect with a defaulted type parameter.
	#[handler_state(none)]
	pub effect Defaulted<T: 'static = i32> {
		/// Resume with the parameter value.
		fn value() -> T;
	}
}

fn main() {}
