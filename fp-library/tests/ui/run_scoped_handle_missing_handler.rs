// Verifies that `handle` rejects a scoped-handler list that does not
// cover every scoped effect in the scoped row. The scoped handler list
// walks the value-level scoped row in lock-step; an empty list can only
// handle an empty scoped row, not a row containing `BoxCatchBrand`.
//
// The expected error mentions a remaining
// `Coproduct<BoxCatch<...>, CNil>` scoped-row head and a
// `ScopedHandlersNil` handler-list tail. That combination means the
// missing entry is
// `BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, CNilBrand, _>()`
// in `scoped_handlers!`. The first-order handler list is empty here
// because the program has no first-order effects; only the scoped
// Catch handler is missing.

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

	let _result = prog.handle(handlers! {}, scoped_handlers! {});
}
