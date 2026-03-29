// Verifies that RcLazy cannot be sent across threads.
// RcLazy uses Rc internally, which is !Send.

use fp_library::types::RcLazy;

fn main() {
	let lazy = RcLazy::new(|| 42);
	std::thread::spawn(move || {
		lazy.evaluate();
	});
}
