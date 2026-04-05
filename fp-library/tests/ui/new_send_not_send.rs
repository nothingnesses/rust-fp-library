use {
	fp_library::{
		brands::ArcFnBrand,
		classes::send_cloneable_fn::SendCloneableFn,
	},
	std::rc::Rc,
};

fn main() {
	let rc = Rc::new(42);
	// Should fail because rc is not Send
	let _ = <ArcFnBrand as SendLiftFn>::new(move |_: ()| {
		println!("{}", rc);
	});
}
