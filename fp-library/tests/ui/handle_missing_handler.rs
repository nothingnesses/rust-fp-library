// Verifies that `handle` rejects a handler list that doesn't
// cover every effect in the row. The
// [`DispatchHandlers`](fp_library::types::effects::interpreter::DispatchHandlers)
// trait walks the handler list and the row in lock-step:
// `HandlersNil` only matches `CNil`, and
// `HandlersCons<Handler<E, F>, T>` only matches a row whose head is
// `E`. A handler list shorter than the row produces a
// trait-not-implemented error at the `handle` call site.
//
// Here the row carries two effects (`IdentityBrand` and
// `OptionBrand`) but the handler list covers only `IdentityBrand`.
// The expected error mentions the remaining
// `Coyoneda<'_, OptionBrand, ...>` row head and a handler-list tail of
// `HandlersNil`; that combination means the missing entry is
// `OptionBrand: ...` in `handlers!`.

use fp_library::{
	brands::{
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
		IdentityBrand,
		OptionBrand,
	},
	handlers,
	types::{
		Identity,
		effects::run::Run,
	},
};

type FirstRow = CoproductBrand<
	CoyonedaBrand<IdentityBrand>,
	CoproductBrand<CoyonedaBrand<OptionBrand>, CNilBrand>,
>;

fn main() {
	let prog: Run<FirstRow, CNilBrand, i32> = Run::lift::<IdentityBrand, _>(Identity(42));

	// Handler list missing the OptionBrand handler -- DispatchHandlers
	// is not implemented for HandlersCons<Handler<IdentityBrand, _>, HandlersNil>
	// against a row whose tail is non-empty.
	let _result = prog.handle(
		handlers! {
			IdentityBrand: |op: Identity<Run<FirstRow, CNilBrand, i32>>| op.0,
		},
		fp_library::types::effects::scoped_nt(),
	);
}
