use fp_library::{
	brands::VecBrand,
	types::Coyoneda,
};

fn assert_clone<T: Clone>(_: &T) {}

fn main() {
	let coyo = Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3]);
	// Should fail because Coyoneda is !Clone (uses Box internally).
	assert_clone(&coyo);
}
