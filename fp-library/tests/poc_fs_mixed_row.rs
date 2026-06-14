//! POC-3 (foundation sweep, Tier A): one unified row holding a first-order and
//! a higher-order effect over the real `Free` substrate.
//!
//! Charter question: can one row hold both a first-order and a higher-order
//! effect (each a valid `Functor`/`WrapDrop` row cell), support an
//! order-directed `peel` that tells the two kinds apart, and drop a deep chain
//! without overflowing the stack?
//!
//! Harness (charter setup step S2): a public-API integration test over the real
//! `Free` substrate, `Coyoneda`, and the real `CoproductBrand`/`CNilBrand` row,
//! with the effects defined via the public `impl_kind!`. Per setup step S3 the
//! row is uniformly `Coyoneda`-wrapped, which supplies `Functor` and `WrapDrop`
//! for every cell, so the custom effects need only a `Kind` projection.
//! Throwaway spike code.

#![cfg(feature = "effects")]
#![allow(dead_code, reason = "POC op payloads are matched structurally, not read")]

use fp_library::{
	Apply,
	brands::{
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
	},
	impl_kind,
	kinds::*,
	types::{
		Coyoneda,
		Free,
		effects::coproduct::{
			CNil,
			Coproduct,
		},
	},
};

// Order classification (as in POC-0).
pub struct FirstOrderMark;
pub struct HigherOrderMark;
pub trait OrderedEffect {
	type Order;
}

// A runtime order tag, so an order-directed peel can branch on the active
// effect's order.
#[derive(Debug, PartialEq, Eq)]
enum OrderTag {
	First,
	Higher,
}
trait OrderTagged {
	fn tag() -> OrderTag;
}
impl OrderTagged for FirstOrderMark {
	fn tag() -> OrderTag {
		OrderTag::First
	}
}
impl OrderTagged for HigherOrderMark {
	fn tag() -> OrderTag {
		OrderTag::Higher
	}
}

// First-order effect: emits an i32 and carries the next program directly.
struct OutBrand;
enum OutF<A> {
	Out(i32, A),
}
impl_kind! {
	impl for OutBrand {
		type Of<'a, A: 'a>: 'a = OutF<A>;
	}
}
impl OrderedEffect for OutBrand {
	type Order = FirstOrderMark;
}

// Higher-order effect: carries an action sub-program and the continuation, both
// in the program (continuation) position `A`. This is the shape that makes an
// effect higher-order (it owns a sub-computation).
struct ScopeBrand;
enum ScopeF<A> {
	Scope(A, A),
}
impl_kind! {
	impl for ScopeBrand {
		type Of<'a, A: 'a>: 'a = ScopeF<A>;
	}
}
impl OrderedEffect for ScopeBrand {
	type Order = HigherOrderMark;
}

// Read the order off a Coyoneda-wrapped row cell value (value-level adapter from
// the effect brand's order to the cell).
trait HasOrder {
	type Order;
}
impl<'a, E: OrderedEffect + Kind_cdc7cd43dac7585f, A> HasOrder for Coyoneda<'a, E, A> {
	type Order = E::Order;
}

// Classify the active arm of a peeled row coproduct by its effect's order. This
// is the order-directed step: after `peel`, decide first-order vs higher-order.
trait ClassifyActive {
	fn classify(&self) -> OrderTag;
}
impl ClassifyActive for CNil {
	fn classify(&self) -> OrderTag {
		match *self {}
	}
}
impl<Cell, Rest> ClassifyActive for Coproduct<Cell, Rest>
where
	Cell: HasOrder,
	Cell::Order: OrderTagged,
	Rest: ClassifyActive,
{
	fn classify(&self) -> OrderTag {
		match self {
			Coproduct::Inl(_) => <Cell::Order as OrderTagged>::tag(),
			Coproduct::Inr(rest) => rest.classify(),
		}
	}
}

// The unified row: a first-order and a higher-order effect in one row, each
// Coyoneda-wrapped (so each cell is a `Functor`/`WrapDrop`), terminated by
// `CNilBrand`.
type Row =
	CoproductBrand<CoyonedaBrand<OutBrand>, CoproductBrand<CoyonedaBrand<ScopeBrand>, CNilBrand>>;

// Inject the first-order effect at the row's head.
fn out_program(value: i32) -> Free<Row, i32> {
	let coyo: Coyoneda<'static, OutBrand, i32> = Coyoneda::lift(OutF::Out(value, value));
	let node: Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, i32>) =
		Coproduct::inject(coyo);
	Free::lift_f(node)
}

// Inject the higher-order effect.
fn scope_program(value: i32) -> Free<Row, i32> {
	let coyo: Coyoneda<'static, ScopeBrand, i32> = Coyoneda::lift(ScopeF::Scope(value, value));
	let node: Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, i32>) =
		Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[test]
fn peel_classifies_a_first_order_effect_as_first_order() {
	match out_program(7).resume() {
		Ok(_) => panic!("expected a suspended first-order effect"),
		Err(node) => assert_eq!(node.classify(), OrderTag::First),
	}
}

#[test]
fn peel_classifies_a_higher_order_effect_as_higher_order() {
	match scope_program(7).resume() {
		Ok(_) => panic!("expected a suspended higher-order effect"),
		Err(node) => assert_eq!(node.classify(), OrderTag::Higher),
	}
}

#[test]
fn deep_bind_chain_of_the_unified_row_drops_without_overflow() {
	// The realistic deep structure for an effect program is queue-deep: many
	// `bind`s extend the continuation queue, not the `Wrap` nesting. This is the
	// drop-safety property that must hold.
	let depth = 100_000;
	let mut free: Free<Row, i32> = out_program(0);
	for _ in 0 .. depth {
		free = free.bind(|_| out_program(0));
	}
	// Dropping the deep program must not overflow the stack.
	drop(free);
}

// Finding (recorded in the POC-3 findings doc, not a unified-row regression): a
// deep chain built by explicit `Free::wrap` nesting of Coyoneda-wrapped cells
// overflows on drop, because `CoyonedaBrand::drop` returns `None` (it cannot
// generically extract the inner program), so `Free`'s iterative `WrapDrop`
// drain cannot drain a Coyoneda-tipped `Wrap` spine. This is a property of the
// existing Coyoneda row encoding (the library's own deep-wrap-drop test uses
// `ThunkBrand`, whose `drop` returns `Some`), shared by the dual row, and does
// not arise from realistic effect programs (which are queue-deep). It is left
// ignored here as documentation of the shared limitation.
#[test]
#[ignore = "deep Wrap-nesting of Coyoneda cells is not drop-safe in either the current or the unified row, because CoyonedaBrand::drop returns None; realistic programs are queue-deep and covered by the test above"]
fn deep_wrap_nesting_of_coyoneda_overflows_known_limitation() {
	let depth = 100_000;
	let mut free: Free<Row, i32> = Free::pure(0);
	for _ in 0 .. depth {
		let inner = free;
		let coyo: Coyoneda<'static, OutBrand, Free<Row, i32>> = Coyoneda::lift(OutF::Out(0, inner));
		let node: Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<Row, i32>>) =
			Coproduct::inject(coyo);
		free = Free::wrap(node);
	}
	drop(free);
}
