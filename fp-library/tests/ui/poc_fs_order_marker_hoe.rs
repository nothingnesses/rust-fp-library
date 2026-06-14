// Compile-fail (trybuild) companion to poc_fs_order_marker.rs (foundation
// sweep POC-0). A row containing a higher-order effect must be rejected by the
// `AllFirstOrder` constraint, and the error should name the offending effect.
//
// trybuild compiles this as a standalone program, so the order-classification
// machinery is duplicated here rather than imported from the test module.

use fp_library::brands::{
	CNilBrand,
	CoproductBrand,
};

pub struct FirstOrderMark;
pub struct HigherOrderMark;

pub trait OrderedEffect {
	type Order;
}

#[diagnostic::on_unimplemented(
	message = "the effect `{Self}` is not a first-order effect",
	label = "this effect is higher-order (or unclassified) but appears in a first-order-only row",
	note = "`AllFirstOrder` requires every effect in the row to be first-order; a higher-order effect must be elaborated or woven away before first-order interpretation"
)]
pub trait FirstOrderEffect {}

pub trait AllFirstOrder {}
impl AllFirstOrder for CNilBrand {}
impl<H: FirstOrderEffect, T: AllFirstOrder> AllFirstOrder for CoproductBrand<H, T> {}

pub struct GetBrand;
impl OrderedEffect for GetBrand {
	type Order = FirstOrderMark;
}
impl FirstOrderEffect for GetBrand {}

pub struct CatchBrand;
impl OrderedEffect for CatchBrand {
	type Order = HigherOrderMark;
}

type HoRow = CoproductBrand<GetBrand, CoproductBrand<CatchBrand, CNilBrand>>;

fn require_all_first_order<R: AllFirstOrder>() {}

fn main() {
	require_all_first_order::<HoRow>();
}
