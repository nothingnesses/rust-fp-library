// Verifies that a program whose scoped row is empty cannot construct a
// scoped operation. Scoped smart constructors require the corresponding
// scoped-effect brand to be present in `S`; `CNilBrand` is uninhabited,
// so the `span` constructor cannot inject a `BoxSpan` layer into it.

use fp_library::{
	brands::CNilBrand,
	types::effects::run::Run,
};

fn main() {
	let action: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
	let _program: Run<CNilBrand, CNilBrand, i32> = Run::span::<&'static str, _>("request", action);
}
