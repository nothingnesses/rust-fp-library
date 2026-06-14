//! POC-0 (foundation sweep, Tier A): order classification over a unified row.
//!
//! Charter question: can each effect brand carry an order marker, and can a
//! recursive `AllFirstOrder` constraint over the library's real
//! `CoproductBrand` row reject a row containing a higher-order effect with a
//! readable error?
//!
//! Harness (charter setup step S2): standalone from the effects subsystem (no
//! `Run`, `Node`, or handlers) but built on the library's real row primitives
//! (`CoproductBrand` / `CNilBrand`), so the technique is proven against the
//! actual encoding rather than a simplified stand-in. The `effects` feature is
//! enabled only to make those brand types available; nothing here uses the
//! Run/Node/handler machinery.
//!
//! This is throwaway spike code (charter S1/S2): it need only compile and pass
//! its assertions. The companion compile-fail case lives at
//! `tests/ui/poc_fs_order_marker_hoe.rs`.

#![cfg(feature = "effects")]

use fp_library::brands::{
	CNilBrand,
	CoproductBrand,
};

// Order markers: the Rust analog of data-effects'
// `EffectOrder = FirstOrder | HigherOrder` promoted kind.
pub struct FirstOrderMark;
pub struct HigherOrderMark;

// Per-effect order classification: the analog of the `OrderOf e` type family.
// Every effect brand declares its order via this associated type.
pub trait OrderedEffect {
	type Order;
}

// A first-order effect marker, implemented directly by each first-order effect
// (the analog of heftia's `FirstOrder` class, which is a marker derived
// separately from the `OrderOf` type family). Keeping it a directly-implemented
// marker rather than a blanket impl over `OrderedEffect<Order = FirstOrderMark>`
// is deliberate: a missing marker surfaces as an unsatisfied-trait-bound
// (E0277), which `#[diagnostic::on_unimplemented]` (stable since 1.78) can shape
// to name the offending effect. A blanket impl keyed on the associated type
// instead produces an associated-type-mismatch (E0271), which the attribute
// cannot customise. A `define_effect!`-style macro would emit this impl for
// first-order effects automatically, so the redundancy with `OrderedEffect` is
// not a per-effect authoring cost.
#[diagnostic::on_unimplemented(
	message = "the effect `{Self}` is not a first-order effect",
	label = "this effect is higher-order (or unclassified) but appears in a first-order-only row",
	note = "`AllFirstOrder` requires every effect in the row to be first-order; a higher-order effect must be elaborated or woven away before first-order interpretation"
)]
pub trait FirstOrderEffect {}

// Recursive "every effect in the row is first-order", walked over the real
// `CoproductBrand` chain terminating in `CNilBrand`. The analog of heftia's
// `FOEs es` constraint.
pub trait AllFirstOrder {}
impl AllFirstOrder for CNilBrand {}
impl<H: FirstOrderEffect, T: AllFirstOrder> AllFirstOrder for CoproductBrand<H, T> {}

// Sample effects.
pub struct GetBrand; // first-order (State-like)
impl OrderedEffect for GetBrand {
	type Order = FirstOrderMark;
}
impl FirstOrderEffect for GetBrand {}

pub struct AskBrand; // first-order (Reader-like)
impl OrderedEffect for AskBrand {
	type Order = FirstOrderMark;
}
impl FirstOrderEffect for AskBrand {}

pub struct CatchBrand; // higher-order (Catch-like): no `FirstOrderEffect` impl.
impl OrderedEffect for CatchBrand {
	type Order = HigherOrderMark;
}

type FoRow = CoproductBrand<GetBrand, CoproductBrand<AskBrand, CNilBrand>>;

fn require_all_first_order<R: AllFirstOrder>() {}

#[test]
fn all_first_order_accepts_an_empty_row() {
	require_all_first_order::<CNilBrand>();
}

#[test]
fn all_first_order_accepts_a_pure_first_order_row() {
	require_all_first_order::<FoRow>();
}

#[test]
fn first_order_effects_are_classified_first_order() {
	fn assert_foe<E: FirstOrderEffect>() {}
	assert_foe::<GetBrand>();
	assert_foe::<AskBrand>();
}

// Compile-fail evidence that a higher-order effect in the row is rejected lives
// in the trybuild case below; `require_all_first_order::<row-with-CatchBrand>()`
// must not compile.
#[test]
fn higher_order_row_is_rejected_with_readable_error() {
	let t = trybuild::TestCases::new();
	t.compile_fail("tests/ui/poc_fs_order_marker_hoe.rs");
}
