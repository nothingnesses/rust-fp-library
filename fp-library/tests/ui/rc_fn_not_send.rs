use fp_library::{
	brands::RcFnBrand,
	classes::SendLiftFn,
};

fn main() {
	// Should fail because RcFnBrand does not implement SendCloneFn
	let _ = <RcFnBrand as SendLiftFn>::new(|x: i32| x);
}
