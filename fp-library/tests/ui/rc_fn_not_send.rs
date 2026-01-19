use fp_library::brands::RcFnBrand;
use fp_library::classes::send_clonable_fn::SendClonableFn;

fn main() {
	// Should fail because RcFnBrand does not implement SendClonableFn
	let _ = <RcFnBrand as SendClonableFn>::send_clonable_fn_new(|x: i32| x);
}
