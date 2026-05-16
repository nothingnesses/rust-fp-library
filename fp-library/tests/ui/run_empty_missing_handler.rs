// Verifies that `handle` rejects a first-order handler list that does
// not cover every first-order effect in the row. The first-order
// handler list walks the value-level row in lock-step; an empty list
// can only handle an empty first-order row, not a row containing
// `EmptyBrand`.
//
// The expected error mentions a remaining
// `Coproduct<Coyoneda<EmptyBrand, ...>, CNil>` first-order-row head
// and a `HandlersNil` handler-list tail. That combination means the
// missing entry is `EmptyBrand: |op| ...` in `handlers!`. The scoped
// handler list is empty here because the program has no scoped effects;
// only the first-order Empty handler is missing.

use fp_library::{
	brands::{
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
		EmptyBrand,
	},
	handlers,
	types::effects::{
		run::Run,
		scoped_nt,
	},
};

type FirstRow = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
type Prog = Run<FirstRow, CNilBrand, i32>;

fn main() {
	let prog: Prog = Run::empty();

	let _result = prog.handle(handlers! {}, scoped_nt());
}
