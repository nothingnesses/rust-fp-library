// Verifies that `Free<F, A>` rejects non-`'static` payloads.
//
// The existing `Free` uses `Box<dyn Any>` internally for type-erased
// continuations. `dyn Any` requires `'static`, which propagates as an
// `A: 'static` bound on the struct. Attempting to hold a borrowed
// reference therefore fails to compile.
//
// This is the baseline contract that motivates the `FreeExplicit` sibling
// type in `tests/free_explicit_poc.rs`: the latter keeps the functor
// structure concrete, avoids `Box<dyn Any>` entirely, and therefore does
// accept non-`'static` payloads.

use fp_library::{
	brands::IdentityBrand,
	types::Free,
};

fn main() {
	let local = String::from("hello");
	let reference: &str = &local;
	let _free: Free<IdentityBrand, &str> = Free::pure(reference);
}
