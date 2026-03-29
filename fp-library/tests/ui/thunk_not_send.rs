// Verifies that Thunk cannot be sent across threads.
// Thunk contains Box<dyn FnOnce() -> A> which is !Send.

use fp_library::types::Thunk;

fn main() {
	let thunk = Thunk::new(|| 42);
	std::thread::spawn(move || {
		thunk.evaluate();
	});
}
