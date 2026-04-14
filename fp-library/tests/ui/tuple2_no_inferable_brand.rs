// Verifies that (A, B) tuples cannot use brand inference with `map`,
// because they have multiple arity-1 brands (Tuple2FirstAppliedBrand,
// Tuple2SecondAppliedBrand, etc.).
// Users must use `explicit::map` with an explicit brand.

use fp_library::functions::map;

fn main() {
	let _ = map(|x: i32| x + 1, (1i32, 2i32));
}
