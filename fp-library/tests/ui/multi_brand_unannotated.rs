// Verifies that multi-brand types require closure input type
// annotations for brand inference. Without an annotation, the
// solver cannot determine A from the closure, so Brand remains
// ambiguous.

use fp_library::functions::map;

fn main() {
	let _ = map(|x| x + 1, Ok::<i32, String>(5));
}
