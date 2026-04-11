// Verifies that Result<A, E> cannot use brand inference with `map`,
// because it has multiple arity-1 brands (ResultErrAppliedBrand,
// ResultOkAppliedBrand, etc.).
// Users must use `map_explicit` with an explicit brand.

use fp_library::functions::map;

fn main() {
	let _ = map(|x: i32| x + 1, Ok::<i32, String>(5));
}
