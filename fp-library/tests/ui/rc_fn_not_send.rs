use fp_library::{
	brands::RcFnBrand,
	classes::send_cloneable_fn::SendCloneableFn,
};

fn main() {
	// Should fail because RcFnBrand does not implement SendCloneableFn
	let _ = <RcFnBrand as SendLiftFn>::new(|x: i32| x);
}
