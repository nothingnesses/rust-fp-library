// Verifies that `im_do!(ref ...)` rejects the two non-Clone Run
// wrappers (Run, RunExplicit). Their inherent surface deliberately
// omits `ref_bind` / `ref_pure`: materializing `Self` from `&self`
// is structurally impossible without `Clone`, and these wrappers
// are single-shot. The macro emits straight method calls, so the
// rejection comes from rustc's "no method named `ref_bind` found"
// error rather than a custom diagnostic.
//
// Only `Run` is exercised here (a single failure is enough to
// demonstrate the property). The same error pattern applies to
// `RunExplicit`.

use {
	fp_library::{
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
		},
		types::effects::run::Run,
	},
	fp_macros::im_do,
};

type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type Scoped = CNilBrand;

fn main() {
	let _result: Run<FirstRow, Scoped, i32> = im_do!(ref Run {
		x: &i32 <- Run::pure(2);
		pure(*x + 1)
	});
}
