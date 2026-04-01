use fp_library::{
	brands::VecBrand,
	types::ArcCoyoneda,
};

fn main() {
	// Rc<i32> is !Send, so ArcCoyoneda::lift should fail.
	let coyo = ArcCoyoneda::<VecBrand, _>::lift(vec![std::rc::Rc::new(1)]);
}
