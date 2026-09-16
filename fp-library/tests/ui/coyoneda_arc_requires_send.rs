use fp_library::{
	brands::{
		ArcBrand,
		VecBrand,
	},
	types::Coyoneda,
};

fn main() {
	// Rc<i32> is !Send, so the Arc-store lift should fail.
	let coyo = Coyoneda::<VecBrand, _, ArcBrand>::lift(vec![std::rc::Rc::new(1)]);
}
