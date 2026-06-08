// Verifies that `RcFree::bind` requires `A: Clone`.
//
// `RcFree`'s shared-inner-state Clone makes its type-erased value cell
// (`Rc<dyn Any>`) potentially shared between branches; recovering an
// owned `A` from that cell uses `Rc::try_unwrap` and falls back to
// `(*shared).clone()` when the cell is shared. The `A: Clone` bound on
// `bind` enforces that the fallback is always available.

use fp_library::{
	brands::IdentityBrand,
	types::RcFree,
};

struct NotClone(i32);

fn main() {
	let program: RcFree<IdentityBrand, NotClone> = RcFree::pure(NotClone(0));
	let _bound = program.bind(|x: NotClone| RcFree::pure(NotClone(x.0 + 1)));
}
