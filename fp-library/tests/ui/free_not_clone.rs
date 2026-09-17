// Verifies that the single-shot Erased variant `Free<F, A>` does not
// implement `Clone`. Single-shot is the property: continuations are
// `Box<dyn FnOnce>`, so the program cannot be invoked twice.
//
// `Free` deliberately omits `#[derive(Clone)]` and does not implement it
// by hand at the single-shot `Box` store. Multi-shot clients pick the
// `Rc` / `Arc` stores of the Explicit family (`FreeExplicit`), whose
// conditional `Clone` is an O(1) refcount bump.

use fp_library::{
	brands::ThunkBrand,
	types::Free,
};

fn main() {
	let free: Free<ThunkBrand, i32> = Free::pure(42);
	let _other = free.clone();
}
