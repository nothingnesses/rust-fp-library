// Verifies that the Rc-store `Free::bind_multi_shot` requires `A: Clone`.
//
// The Rc store's shared storage makes its type-erased value cell
// (`Rc<dyn Any>`) potentially shared between branches; recovering an
// owned `A` from that cell falls back to cloning when the cell is
// shared. The `A: Clone` bound on `bind_multi_shot` enforces that the
// fallback is always available.

use fp_library::{
	brands::{
		IdentityBrand,
		RcBrand,
	},
	types::Free,
};

struct NotClone(i32);

fn main() {
	let program: Free<IdentityBrand, NotClone, RcBrand> =
		Free::<IdentityBrand, NotClone, RcBrand>::pure(NotClone(0));
	let _bound = program.bind_multi_shot(|x: NotClone| Free::pure(NotClone(x.0 + 1)));
}
