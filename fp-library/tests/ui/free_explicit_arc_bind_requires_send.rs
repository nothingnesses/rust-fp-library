// Verifies that the Arc-store `FreeExplicit::bind` rejects a closure
// that is not `Send + Sync`.
//
// The Arc store's `bind` stores the user closure in an
// `Arc<dyn Fn + Send + Sync>` continuation cell, so the closure must be
// `Send + Sync`. Capturing an `Rc<...>` (which is `!Send` and `!Sync`)
// poisons the closure's auto-trait derivation and the bind call fails to
// compile. Multi-shot single-thread programs should use the Rc store
// instead.

use {
	fp_library::{
		brands::{
			ArcBrand,
			IdentityBrand,
		},
		types::FreeExplicit,
	},
	std::rc::Rc,
};

fn main() {
	let captured: Rc<i32> = Rc::new(7);
	let program: FreeExplicit<'_, IdentityBrand, i32, ArcBrand> =
		FreeExplicit::<'_, IdentityBrand, i32, ArcBrand>::pure(0);
	let _bound = program.bind(move |x: i32| {
		let _ = &captured;
		FreeExplicit::<'_, IdentityBrand, i32, ArcBrand>::pure(x + 1)
	});
}
