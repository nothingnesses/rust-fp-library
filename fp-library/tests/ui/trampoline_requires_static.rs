// Verifies that Trampoline requires 'static data.
// Borrowed (non-'static) references cannot be used with Trampoline.

use fp_library::types::Trampoline;

fn main() {
	let local = String::from("hello");
	let reference = &local;
	let _trampoline = Trampoline::new(|| reference.len());
}
