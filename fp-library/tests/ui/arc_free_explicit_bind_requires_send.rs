// Verifies that `ArcFreeExplicit::bind` rejects a closure that is not
// `Send + Sync`.
//
// `ArcFreeExplicit::bind` stores the user closure in an
// `Arc<dyn Fn + Send + Sync>` continuation cell, so the closure must be
// `Send + Sync`. Capturing an `Rc<...>` (which is `!Send` and `!Sync`)
// poisons the closure's auto-trait derivation and the bind call fails to
// compile. Multi-shot single-thread programs should use `RcFreeExplicit`
// instead.

use {
	fp_library::{
		brands::IdentityBrand,
		types::ArcFreeExplicit,
	},
	std::rc::Rc,
};

fn main() {
	let captured: Rc<i32> = Rc::new(7);
	let program: ArcFreeExplicit<'_, IdentityBrand, i32> = ArcFreeExplicit::pure(0);
	let _bound = program.bind(move |x: i32| {
		let _ = &captured;
		ArcFreeExplicit::pure(x + 1)
	});
}
