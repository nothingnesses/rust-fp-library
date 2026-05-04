// Verifies that single-shot wrappers (`Run`, `RunExplicit`) reject
// the `Choose` smart constructor. `Choose`'s handler runs the
// continuation twice (once per branch), which requires the
// continuation to be cloneable; single-shot wrappers cannot host
// this effect, and their inherent surface deliberately omits
// `choose`. The constructor only exists on the four multi-shot
// wrappers (`RcRun`, `RcRunExplicit`, `ArcRun`, `ArcRunExplicit`).
//
// Only `Run` is exercised here (a single failure is enough to
// demonstrate the property). The same error pattern applies to
// `RunExplicit`.

use fp_library::{
	brands::{
		CNilBrand,
		ChooseBrand,
		CoproductBrand,
		CoyonedaBrand,
		RcBrand,
	},
	types::effects::run::Run,
};

type FirstRow = CoproductBrand<CoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
type Scoped = CNilBrand;

fn main() {
	let _prog: Run<FirstRow, Scoped, bool> = Run::choose();
}
