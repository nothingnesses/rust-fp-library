// Verifies that the single-shot Erased variant `Free<F, A>` does not
// implement `Clone`. Single-shot is the property: continuations are
// `Box<dyn FnOnce>`, so the program cannot be invoked twice.
//
// `Free` deliberately omits `#[derive(Clone)]` and does not implement it
// by hand. Multi-shot clients must pick `RcFree` or `ArcFree` (whose
// outer `Rc<Inner>` / `Arc<Inner>` wrapping makes Clone unconditionally
// O(1)) or the multi-shot Explicit variants `RcFreeExplicit` /
// `ArcFreeExplicit`.

use fp_library::{
	brands::ThunkBrand,
	types::Free,
};

fn main() {
	let free: Free<ThunkBrand, i32> = Free::pure(42);
	let _other = free.clone();
}
