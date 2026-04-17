// Verifies that Result<T, T> (diagonal case) cannot use brand
// inference with `map`, because both ResultErrAppliedBrand<T> and
// ResultOkAppliedBrand<T> match when A = T.
// Users must use `explicit::map` with an explicit brand.

use fp_library::functions::map;

fn main() {
	let _ = map(|x: i32| x + 1, Ok::<i32, i32>(5));
}
