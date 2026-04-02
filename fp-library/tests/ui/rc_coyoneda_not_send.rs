use fp_library::{
	brands::VecBrand,
	types::RcCoyoneda,
};

fn assert_send<T: Send>(_: &T) {}

fn main() {
	let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
	// Should fail because RcCoyoneda is !Send (uses Rc internally).
	assert_send(&coyo);
}
