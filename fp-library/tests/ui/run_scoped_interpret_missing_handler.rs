// Verifies that `interpret` rejects a scoped-handler list that does not
// cover every scoped effect in the scoped row. The scoped handler list
// walks the value-level scoped row in lock-step; an empty list can only
// handle an empty scoped row, not a row containing `BoxCatchBrand`.

use fp_library::{
	brands::{
		BoxBrand,
		BoxCatchBrand,
		CNilBrand,
		CoproductBrand,
	},
	handlers,
	scoped_handlers,
	types::effects::run::Run,
};

type ScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;
type Prog = Run<CNilBrand, ScopedRow, i32>;

fn main() {
	let prog: Prog = Run::catch::<&'static str, _>(Run::pure(42), |_err| Run::pure(0));

	let _result = prog.interpret(handlers! {}, scoped_handlers! {});
}
