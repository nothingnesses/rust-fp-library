// Verifies that a smart constructor's result type is bound to the
// row's effect parameterization, not free to vary. `Run::ask::<Idx>()`
// produces `Run<R, S, A>` where `A` is the environment type fixed by
// the row's `BoxReaderBrand<BoxBrand, A>` member. Ascribing a
// different `A` at the binding site should fail with a type mismatch.
//
// Here the row carries `BoxReaderBrand<BoxBrand, String>` (the
// environment is `String`), but the binding ascribes
// `Run<FirstRow, Scoped, i32>`. Rust's type inference unifies the
// row's `A` with the ascription's `A`, producing a mismatch on
// `String != i32`.

use fp_library::{
	brands::{
		BoxBrand,
		BoxReaderBrand,
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
	},
	types::effects::run::Run,
};

type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, String>>, CNilBrand>;
type Scoped = CNilBrand;

fn main() {
	let _prog: Run<FirstRow, Scoped, i32> = Run::ask();
}
