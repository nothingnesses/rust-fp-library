use fp_library::brands::RcFnBrand;
use fp_library::classes::send_clonable_fn::SendClonableFn;

fn main() {
	// Should fail because RcFnBrand does not implement SendClonableFn
	let _ = <RcFnBrand as SendClonableFn>::new_send(|x: i32| x);
}
