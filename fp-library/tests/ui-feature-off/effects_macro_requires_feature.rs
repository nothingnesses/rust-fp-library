//! Public effect macros should fail at the fp-library boundary when the
//! optional effects subsystem is disabled.

use fp_library::effects;

type Row = effects![];

fn main() {
	let _ = core::any::type_name::<Row>();
}
