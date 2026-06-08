// Verifies that `RefBracket` cannot be instantiated with a pointer
// brand that lacks refcounted pointer operations. RefBracket body and
// release closures receive cloneable pointer values, so a Box-backed
// pointer brand is not a valid RefBracket storage brand. The compiler
// reports the `ToDynCloneFn` bound first; that trait itself requires a
// refcounted pointer brand.

use fp_library::{
	brands::{
		BoxBrand,
		IdentityBrand,
	},
	types::effects::ref_bracket::RefBracket,
};

fn main() {
	let _ = core::mem::size_of::<RefBracket<'static, BoxBrand, IdentityBrand, i32, i32>>();
}
