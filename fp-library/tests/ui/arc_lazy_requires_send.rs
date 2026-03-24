// Verifies that ArcLazy::new requires a Send closure.
// A closure capturing Rc (which is !Send) should be rejected.

use fp_library::types::ArcLazy;
use std::rc::Rc;

fn main() {
	let rc = Rc::new(42);
	let _lazy = ArcLazy::new(move || *rc);
}
