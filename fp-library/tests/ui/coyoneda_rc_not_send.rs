use fp_library::{
	brands::{
		RcBrand,
		VecBrand,
	},
	types::Coyoneda,
};

fn assert_send<T: Send>(_: &T) {}

fn main() {
	let coyo = Coyoneda::<VecBrand, _, RcBrand>::lift(vec![1, 2, 3]);
	// Should fail because the Rc-store Coyoneda is !Send (uses Rc internally).
	assert_send(&coyo);
}
