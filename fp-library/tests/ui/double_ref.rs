// Verifies that double references (&&T) are not supported by
// FunctorDispatch's Ref impl. The Ref impl matches &Brand::Of<A>,
// not &&Brand::Of<A>.

use fp_library::functions::map;

fn main() {
	let _ = map(|x: &i32| *x + 1, &&Some(5));
}
