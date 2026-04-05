use {
	fp_library::{
		brands::ArcFnBrand,
		classes::send_clone_fn::SendCloneFn,
	},
	std::cell::RefCell,
};

fn main() {
	let cell = RefCell::new(42);
	// Should fail because cell is not Sync, so the closure is not Sync
	let _ = <ArcFnBrand as SendLiftFn>::new(move |_: ()| {
		println!("{:?}", cell);
	});
}
