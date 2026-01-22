use fp_library::brands::RcFnBrand;
use fp_library::classes::send_cloneable_fn::SendCloneableFn;

fn main() {
	// Should fail because RcFnBrand does not implement SendCloneableFn
	let _ = <RcFnBrand as SendCloneableFn>::send_cloneable_fn_new(|x: i32| x);
}
